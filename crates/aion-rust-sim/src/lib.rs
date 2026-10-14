//! AION CORE 0.5: candidate *pure Rust* replacement for the C99 mechanics kernel.
//!
//! Units: metres, kilograms, seconds, joules and kelvin. Nodes are coarse-grained
//! masses, not atoms. Bonds use Hookean elasticity, Kelvin damping, simplified
//! plastic flow and monotone damage. The numerical integrator is semi-implicit
//! Euler with bounded substeps; its energy drift must be measured, not hidden.
//!
//! ABI exports are provided for the existing Windows C Win32/GDI visualizer.
//! The Rust code has not been compiled in the authoring environment: use the
//! Windows CI workflow and `cargo test` before treating it as production-ready.

use core::ffi::c_int;
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};

pub const MAX_NODES: usize = 128;
pub const MAX_BONDS: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 { pub x: f64, pub y: f64, pub z: f64 }
impl V3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self { Self {x,y,z} }
    pub fn dot(self, b: Self) -> f64 { self.x*b.x+self.y*b.y+self.z*b.z }
    pub fn length(self) -> f64 { self.dot(self).sqrt() }
}
impl Add for V3 {type Output=Self; fn add(self,b:Self)->Self {Self::new(self.x+b.x,self.y+b.y,self.z+b.z)}}
impl Sub for V3 {type Output=Self; fn sub(self,b:Self)->Self {Self::new(self.x-b.x,self.y-b.y,self.z-b.z)}}
impl Mul<f64> for V3 {type Output=Self; fn mul(self,s:f64)->Self {Self::new(self.x*s,self.y*s,self.z*s)}}
impl AddAssign for V3 {fn add_assign(&mut self, b:Self){*self=*self+b}}
impl SubAssign for V3 {fn sub_assign(&mut self, b:Self){*self=*self-b}}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Node {
    pub p: V3, pub v: V3, pub r: f64, pub m: f64, pub t: f64,
    pub cv: f64, pub kind: c_int, pub component: c_int,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Edge {
    pub a: c_int, pub b: c_int, pub alive: c_int,
    pub rest: f64, pub k: f64, pub limit: f64, pub conductance: f64,
    pub original_rest: f64, pub yield_strain: f64, pub plastic_rate: f64,
    pub damage: f64, pub damage_onset: f64, pub damage_rate: f64,
    pub dashpot: f64,
}
#[repr(C)]
#[derive(Clone, Debug)]
pub struct AionSim {
    pub nodes: [Node; MAX_NODES],
    pub edges: [Edge; MAX_BONDS],
    pub node_count: c_int, pub edge_count: c_int,
    pub fractures: c_int, pub contacts: c_int, pub material_mode: c_int,
    pub rejected_steps: c_int, pub plastic_events: c_int, pub fatigue_fractures: c_int,
    pub sim_time: f64, pub gravity: f64, pub max_strain: f64,
    pub max_contact_impulse: f64, pub plastic_heat: f64,
    pub damage_heat: f64, pub damping_heat: f64, pub max_damage: f64,
    pub floor_enabled: c_int,
}
impl Default for AionSim {
    fn default() -> Self {
        Self { nodes:[Node::default();MAX_NODES], edges:[Edge::default();MAX_BONDS],
            node_count:0,edge_count:0,fractures:0,contacts:0,material_mode:0,
            rejected_steps:0,plastic_events:0,fatigue_fractures:0,
            sim_time:0.,gravity:-9.81,max_strain:0.,max_contact_impulse:0.,
            plastic_heat:0.,damage_heat:0.,damping_heat:0.,max_damage:0.,floor_enabled:1 }
    }
}
impl AionSim {
    /// Reset all state including the inactive slots; required for ABI parity and stable replay.
    pub fn reset(&mut self, mode: i32) {
        *self=Self::default();
        self.material_mode=if (0..=2).contains(&mode){mode}else{0};
        let (stiff,limit,mass)=match self.material_mode {
            1=>(600.,38.,0.65), 2=>(2500.,34.,1.4), _=>(1500.,62.,1.2)
        };
        let mut ids=[[[0usize;3];4];6];
        for x in 0..6 { for y in 0..4 { for z in 0..3 {
            ids[x][y][z]=self.add_node(
                V3::new((x as f64-2.5)*0.23,1.1+y as f64*0.23,(z as f64-1.)*0.23),
                0.070,mass,293.,self.material_mode);
        }}}
        for x in 0..6 { for y in 0..4 { for z in 0..3 {
            let a=ids[x][y][z];
            if x<5 {self.add_edge(a,ids[x+1][y][z],stiff,limit)}
            if y<3 {self.add_edge(a,ids[x][y+1][z],stiff,limit)}
            if z<2 {self.add_edge(a,ids[x][y][z+1],stiff,limit)}
            if x<5&&y<3 {
                self.add_edge(a,ids[x+1][y+1][z],stiff*0.56,limit*0.7);
                self.add_edge(ids[x+1][y][z],ids[x][y+1][z],stiff*0.56,limit*0.7);
            }
            if x<5&&z<2 {
                self.add_edge(a,ids[x+1][y][z+1],stiff*0.56,limit*0.7);
                self.add_edge(ids[x+1][y][z],ids[x][y][z+1],stiff*0.56,limit*0.7);
            }
            if y<3&&z<2 {
                self.add_edge(a,ids[x][y+1][z+1],stiff*0.56,limit*0.7);
                self.add_edge(ids[x][y+1][z],ids[x][y][z+1],stiff*0.56,limit*0.7);
            }
        }}}
        let hammer=self.add_node(V3::new(0.12,3.90,0.01),0.23,11.,293.,3);
        self.nodes[hammer].v=V3::new(0.,-4.,0.);
    }
    pub fn laboratory(mode: i32) -> Self { let mut world=Self::default();world.reset(mode);world }
    fn add_node(&mut self,p:V3,r:f64,m:f64,t:f64,kind:i32)->usize {
        assert!((self.node_count as usize)<MAX_NODES);
        let i=self.node_count as usize;self.node_count+=1;
        self.nodes[i]=Node {p,v:V3::default(),r,m,t,cv:m*if kind==1{1650.}else{780.},kind,component:i as i32};
        i
    }
    fn add_edge(&mut self,a:usize,b:usize,k:f64,limit:f64){
        assert!(a!=b&&(self.edge_count as usize)<MAX_BONDS);
        let rest=(self.nodes[a].p-self.nodes[b].p).length();
        let (yield_strain,plastic_rate,damage_onset,damage_rate,dashpot)=match self.material_mode{
            1=>(0.055,8.,0.10,0.65,1.5), 2=>(0.20,0.,0.042,8.,0.22),
            _=>(0.10,2.5,0.065,2.5,0.55)
        };
        self.edges[self.edge_count as usize]=Edge {a:a as i32,b:b as i32,alive:1,
            rest,k,limit,conductance:3.2,original_rest:rest,yield_strain,plastic_rate,
            damage:0.,damage_onset,damage_rate,dashpot};
        self.edge_count+=1;
    }
    fn add_heat(&mut self,a:usize,b:usize, joules:f64){
        if joules<=0. {return}
        let rise=joules/(self.nodes[a].cv+self.nodes[b].cv);
        self.nodes[a].t+=rise;self.nodes[b].t+=rise;
    }
    fn update_components(&mut self){
        let mut parent=[0usize;MAX_NODES];
        for (i,v) in parent.iter_mut().enumerate().take(self.node_count as usize){*v=i}
        fn root(parent:&mut [usize;MAX_NODES],mut x:usize)->usize {
            while parent[x]!=x {parent[x]=parent[parent[x]];x=parent[x]}
            x
        }
        for i in 0..self.edge_count as usize {
            let e=self.edges[i];if e.alive==0 {continue}
            let a=root(&mut parent,e.a as usize);
            let b=root(&mut parent,e.b as usize);
            if a!=b {parent[b]=a}
        }
        for i in 0..self.node_count as usize {
            self.nodes[i].component=root(&mut parent,i) as i32;
        }
    }
    pub fn step(&mut self, dt:f64){
        if !dt.is_finite()||dt<=0.||dt>0.1 {self.rejected_steps+=1;return}
        let sub=((dt/0.002+0.999999) as usize).max(1);
        let h=dt/sub as f64;
        for _ in 0..sub {self.substep(h)}
    }
    fn substep(&mut self,dt:f64){
        let n=self.node_count as usize;
        let mut forces=[V3::default();MAX_NODES];
        for (f,p) in forces.iter_mut().zip(self.nodes.iter()).take(n){
            *f=V3::new(0.,self.gravity*p.m,0.);
        }
        for index in 0..self.edge_count as usize {
            let e=self.edges[index];if e.alive==0 {continue}
            let a=e.a as usize;let b=e.b as usize;
            let delta=self.nodes[b].p-self.nodes[a].p;
            let dist=delta.length();if dist<=1e-12{continue}
            let dir=delta*(1./dist);
            let mut ext=dist-e.rest;
            let mut strain=if e.rest>1e-12{(ext/e.rest).abs()}else{0.};
            self.max_strain=self.max_strain.max(strain);
            let mut changed=e;
            if changed.plastic_rate>0.&&strain>changed.yield_strain{
                let before=0.5*changed.k*(1.-changed.damage)*ext*ext;
                let target=dist-ext.signum()*changed.yield_strain*changed.rest;
                let alpha=(changed.plastic_rate*dt).min(1.);
                changed.rest+=(target-changed.rest)*alpha;
                ext=dist-changed.rest;
                let after=0.5*changed.k*(1.-changed.damage)*ext*ext;
                if before>after {
                    let heat=before-after;self.add_heat(a,b,heat);self.plastic_heat+=heat;
                }
                self.plastic_events+=1;
            }
            strain=if changed.rest>1e-12{(ext/changed.rest).abs()}else{0.};
            if changed.damage_rate>0.&&strain>changed.damage_onset{
                let previous=changed.damage;
                changed.damage=(changed.damage+changed.damage_rate*(strain-changed.damage_onset)*dt).min(1.);
                if changed.damage>previous{
                    let heat=0.5*changed.k*(changed.damage-previous)*ext*ext;
                    self.add_heat(a,b,heat);self.damage_heat+=heat;
                }
            }
            self.max_damage=self.max_damage.max(changed.damage);
            let spring_force=changed.k*(1.-changed.damage)*ext;
            if spring_force.abs()>changed.limit||changed.damage>=0.999 {
                if changed.damage>=0.999{self.fatigue_fractures+=1}
                changed.alive=0;self.fractures+=1;
                let heat=0.5*changed.k*(1.-changed.damage)*ext*ext;
                self.add_heat(a,b,heat);self.damage_heat+=heat;
                self.edges[index]=changed;
                continue;
            }
            let rel=(self.nodes[b].v-self.nodes[a].v).dot(dir);
            let axial=dir*(spring_force-changed.dashpot*rel);
            forces[a]+=axial;forces[b]-=axial;
            let heat=changed.dashpot*rel*rel*dt;
            if heat>0.{self.add_heat(a,b,heat);self.damping_heat+=heat}
            let delta_t=self.nodes[a].t-self.nodes[b].t;
            let ca=self.nodes[a].cv;let cb=self.nodes[b].cv;
            let maximum=delta_t.abs()*ca*cb/(ca+cb);
            let transfer=(changed.conductance*delta_t*dt).clamp(-maximum,maximum);
            self.nodes[a].t-=transfer/ca;self.nodes[b].t+=transfer/cb;
            self.edges[index]=changed;
        }
        for (p,force) in self.nodes.iter_mut().zip(forces.iter()).take(n){
            p.v+=*force*(dt/p.m);p.p+=p.v*dt;
        }
        self.update_components();
        // Broad phase: sort by X, sweep only neighbours whose separation fits.
        let mut order=[0usize;MAX_NODES];let mut max_radius:f64=0.;
        for (i,slot) in order.iter_mut().enumerate().take(n){
            *slot=i;max_radius=max_radius.max(self.nodes[i].r);
        }
        // Insertion sort matches the old reference's deterministic tie-breaking.
        for i in 1..n {
            let id=order[i];let mut j=i;
            while j>0&&self.nodes[order[j-1]].p.x>self.nodes[id].p.x{
                order[j]=order[j-1];j-=1;
            }
            order[j]=id;
        }
        for ai in 0..n {for bi in ai+1..n {
            let i=order[ai];let j=order[bi];
            if self.nodes[j].p.x-self.nodes[i].p.x>2.*max_radius{break}
            if self.nodes[i].component==self.nodes[j].component{continue}
            let delta=self.nodes[j].p-self.nodes[i].p;
            let rr=self.nodes[i].r+self.nodes[j].r;
            if delta.y.abs()>=rr||delta.z.abs()>=rr||delta.x.abs()>=rr{continue}
            let dist=delta.length();if dist>=rr{continue}
            let dir=if dist>1e-10{delta*(1./dist)}else{V3::new(1.,0.,0.)};
            let ia=1./self.nodes[i].m;let ib=1./self.nodes[j].m;
            let fix=dir*((rr-dist)/(ia+ib));
            self.nodes[i].p-=fix*ia;self.nodes[j].p+=fix*ib;
            let rel=(self.nodes[j].v-self.nodes[i].v).dot(dir);
            if rel<0.{
                let before=0.5*self.nodes[i].m*self.nodes[i].v.dot(self.nodes[i].v)
                    +0.5*self.nodes[j].m*self.nodes[j].v.dot(self.nodes[j].v);
                let impulse=-(1.+0.24)*rel/(ia+ib);
                self.nodes[i].v-=dir*(impulse*ia);
                self.nodes[j].v+=dir*(impulse*ib);
                let after=0.5*self.nodes[i].m*self.nodes[i].v.dot(self.nodes[i].v)
                    +0.5*self.nodes[j].m*self.nodes[j].v.dot(self.nodes[j].v);
                self.add_heat(i,j,before-after);
                self.max_contact_impulse=self.max_contact_impulse.max(impulse);
                self.contacts+=1;
            }
        }}
        if self.floor_enabled!=0{
            for p in self.nodes.iter_mut().take(n){
                if p.p.y>=p.r{continue}
                p.p.y=p.r;
                if p.v.y<0.{
                    let before=0.5*p.m*p.v.dot(p.v);
                    p.v.y=-p.v.y*0.2;p.v.x*=0.91;p.v.z*=0.91;
                    let after=0.5*p.m*p.v.dot(p.v);
                    p.t+=(before-after)/p.cv;
                }
            }
        }
        self.sim_time+=dt;
    }
    pub fn impact(&mut self){if self.node_count>0{self.nodes[self.node_count as usize-1].v.y-=7.}}
    pub fn pull(&mut self){for p in self.nodes.iter_mut().take(self.node_count as usize){
        if p.kind==3{continue}
        if p.p.x< -0.35{p.v.x-=1.8}
        else if p.p.x>0.35{p.v.x+=1.8}
    }}
    pub fn heat(&mut self){for p in self.nodes.iter_mut().take((self.node_count-1).max(0) as usize){if p.p.x< -0.25{p.t+=25.}}}
    pub fn alive_bonds(&self)->i32{self.edges.iter().take(self.edge_count as usize).filter(|e|e.alive!=0).count() as i32}
    pub fn mass(&self)->f64{self.nodes.iter().take(self.node_count as usize).map(|n|n.m).sum()}
    pub fn thermal_energy(&self)->f64{self.nodes.iter().take(self.node_count as usize).map(|n|n.t*n.cv).sum()}
    pub fn mechanical_energy(&self)->f64{
        let mut result=0.;
        for n in self.nodes.iter().take(self.node_count as usize){result+=0.5*n.m*n.v.dot(n.v)-n.m*self.gravity*n.p.y}
        for e in self.edges.iter().take(self.edge_count as usize).filter(|e|e.alive!=0){
            let ext=(self.nodes[e.a as usize].p-self.nodes[e.b as usize].p).length()-e.rest;
            result+=0.5*e.k*ext*ext;
        }
        result
    }
    pub fn total_energy(&self)->f64{self.mechanical_energy()+self.thermal_energy()}
    pub fn max_plastic_strain(&self)->f64{
        self.edges.iter().take(self.edge_count as usize)
        .filter(|e|e.original_rest>1e-12)
        .map(|e|((e.rest-e.original_rest)/e.original_rest).abs()).fold(0.,f64::max)
    }
    pub fn checksum(&self)->u64{
        let mut h=1469598103934665603u64;
        fn feed(h:&mut u64, bytes:&[u8]){for &b in bytes {*h^=b as u64;*h=h.wrapping_mul(1099511628211)}}
        feed(&mut h,&self.node_count.to_ne_bytes());feed(&mut h,&self.edge_count.to_ne_bytes());
        feed(&mut h,&self.sim_time.to_ne_bytes());
        for n in self.nodes.iter().take(self.node_count as usize){
            for x in [n.p.x,n.p.y,n.p.z,n.v.x,n.v.y,n.v.z,n.t]{feed(&mut h,&x.to_ne_bytes())}
        }
        for e in self.edges.iter().take(self.edge_count as usize){
            feed(&mut h,&e.alive.to_ne_bytes());feed(&mut h,&e.rest.to_ne_bytes());
            feed(&mut h,&e.damage.to_ne_bytes());
        }
        h
    }
}

// C-compatible exports, designed to eventually replace native/aion_sim.c
// in the exact same existing Windows visualizer. The caller must pass a
// writable, aligned, correctly-sized AionSim; non-null pointers are validated.
#[no_mangle] pub unsafe extern "C" fn aion_reset(s:*mut AionSim,mode:c_int){
    if let Some(s)=unsafe{s.as_mut()}{s.reset(mode)}
}
#[no_mangle] pub unsafe extern "C" fn aion_step(s:*mut AionSim,dt:f64){
    if let Some(s)=unsafe{s.as_mut()}{s.step(dt)}
}
#[no_mangle] pub unsafe extern "C" fn aion_impact(s:*mut AionSim){if let Some(s)=unsafe{s.as_mut()}{s.impact()}}
#[no_mangle] pub unsafe extern "C" fn aion_pull(s:*mut AionSim){if let Some(s)=unsafe{s.as_mut()}{s.pull()}}
#[no_mangle] pub unsafe extern "C" fn aion_heat(s:*mut AionSim){if let Some(s)=unsafe{s.as_mut()}{s.heat()}}
#[no_mangle] pub unsafe extern "C" fn aion_max_plastic_strain(s:*const AionSim)->f64{
    unsafe{s.as_ref()}.map_or(0.,AionSim::max_plastic_strain)
}
#[no_mangle] pub unsafe extern "C" fn aion_alive_bonds(s:*const AionSim)->c_int{
    unsafe{s.as_ref()}.map_or(0,AionSim::alive_bonds)
}
#[no_mangle] pub unsafe extern "C" fn aion_mass(s:*const AionSim)->f64{
    unsafe{s.as_ref()}.map_or(0.,AionSim::mass)
}
#[no_mangle] pub unsafe extern "C" fn aion_thermal_energy(s:*const AionSim)->f64{
    unsafe{s.as_ref()}.map_or(0.,AionSim::thermal_energy)
}
#[no_mangle] pub unsafe extern "C" fn aion_mechanical_energy(s:*const AionSim)->f64{
    unsafe{s.as_ref()}.map_or(0.,AionSim::mechanical_energy)
}
#[no_mangle] pub unsafe extern "C" fn aion_total_energy(s:*const AionSim)->f64{
    unsafe{s.as_ref()}.map_or(0.,AionSim::total_energy)
}
#[no_mangle] pub unsafe extern "C" fn aion_checksum(s:*const AionSim)->u64{
    unsafe{s.as_ref()}.map_or(0,AionSim::checksum)
}
#[no_mangle] pub extern "C" fn aion_abi_size()->u32{
    core::mem::size_of::<AionSim>() as u32
}

#[cfg(test)] mod tests {
    use super::*;
    fn sim(material:i32)->AionSim{AionSim::laboratory(material)}
    #[test] fn topology_and_mass(){for m in 0..3{
        let s=sim(m);assert_eq!(s.node_count,73);assert_eq!(s.edge_count,404);
        assert_eq!(s.alive_bonds(),404);
        assert!(s.mass()>0.);assert_eq!(s.nodes[72].kind,3);
    }}
    #[test] fn is_reproducible(){let mut a=sim(0);let mut b=sim(0);
        for _ in 0..2000{a.step(0.003);b.step(0.003)}
        assert_eq!(a.checksum(),b.checksum());assert!(a.fractures>0);
    }
    #[test] fn mass_and_nonnegative_temperature(){let mut a=sim(1);let mass=a.mass();
        for _ in 0..3000 {a.step(0.003)}
        assert_eq!(mass,a.mass());
        for p in a.nodes.iter().take(a.node_count as usize){
            assert!(p.p.x.is_finite()&&p.p.y.is_finite()&&p.p.z.is_finite());
            assert!(p.t.is_finite()&&p.t>=0.);
            assert!(p.p.y>=p.r-1e-8);
        }
    }
    #[test] fn isolated_collision_preserves_momentum(){let mut s=sim(0);
        s.node_count=2;s.edge_count=0;s.floor_enabled=0;s.gravity=0.;
        for i in 0..2{s.nodes[i].p=V3::new(if i==0{-0.19}else{0.19},2.,0.);
            s.nodes[i].m=1.;s.nodes[i].r=0.2;s.nodes[i].cv=500.;}
        s.nodes[0].v=V3::new(1.,0.,0.);s.nodes[1].v=V3::new(-1.,0.,0.);
        s.step(0.003);
        assert!(s.contacts>0);assert!((s.nodes[0].v.x+s.nodes[1].v.x).abs()<1e-12);
    }
    #[test] fn thermal_exchange_is_conservative(){let mut s=sim(0);
        s.node_count=2;s.edge_count=1;s.floor_enabled=0;s.gravity=0.;
        s.nodes[0].t=350.;s.nodes[1].t=250.;
        s.nodes[0].v=V3::default();s.nodes[1].v=V3::default();
        s.edges[0].a=0;s.edges[0].b=1;s.edges[0].alive=1;
        s.edges[0].k=0.;s.edges[0].dashpot=0.;s.edges[0].damage_rate=0.;
        s.edges[0].plastic_rate=0.;s.edges[0].conductance=50.;
        let before=s.thermal_energy();s.step(0.002);
        assert!((before-s.thermal_energy()).abs()<1e-7);
        assert!(s.nodes[0].t<350.&&s.nodes[1].t>250.);
    }
    #[test] fn rejects_invalid_steps(){let mut s=sim(0);let checksum=s.checksum();
        for dt in [f64::NAN,f64::INFINITY,0.,-0.1,0.101]{s.step(dt)}
        assert_eq!(s.rejected_steps,5);assert_eq!(checksum,s.checksum());
    }
    #[test] fn rust_abi_matches_c_reference_layout(){
        assert_eq!(core::mem::size_of::<V3>(),24);
        assert_eq!(core::mem::size_of::<Node>(),88);
        assert_eq!(core::mem::size_of::<Edge>(),104);
        // Last structure offset and padding depend only on repr(C) and these fixed types.
        assert_eq!(core::mem::size_of::<AionSim>(),117864);
    }
    #[test] fn plasticity_accumulates_irreversible_rest_change(){
        let mut s=sim(0);s.node_count=2;s.edge_count=1;s.gravity=0.;s.floor_enabled=0;
        s.nodes[0].p=V3::new(0.,1.,0.);
        s.nodes[1].p=V3::new(1.25,1.,0.);
        for n in s.nodes.iter_mut().take(2){n.v=V3::default();n.r=0.05;n.m=1.;n.cv=500.;n.t=300.;}
        s.edges[0]=Edge{a:0,b:1,alive:1,rest:1.,k:20.,limit:5000.,
            conductance:0.,original_rest:1.,yield_strain:0.05,plastic_rate:30.,
            damage:0.,damage_onset:0.30,damage_rate:0.,dashpot:0.};
        let before=s.total_energy();s.step(0.002);
        assert!(s.edges[0].rest>1.);
        assert!(s.plastic_heat>0.);
        assert!((s.total_energy()-before).abs()<0.006);
        let rest=s.edges[0].rest;s.edges[0].plastic_rate=0.;s.step(0.002);
        assert_eq!(s.edges[0].rest,rest);
    }
    #[test] fn cumulative_damage_does_not_self_heal(){
        let mut s=sim(0);s.node_count=2;s.edge_count=1;s.gravity=0.;s.floor_enabled=0;
        s.nodes[0].p=V3::new(0.,2.,0.);
        s.nodes[1].p=V3::new(1.25,2.,0.);
        for n in s.nodes.iter_mut().take(2){n.v=V3::default();n.m=1.;n.r=0.05;n.cv=500.;n.t=300.;}
        s.edges[0]=Edge{a:0,b:1,alive:1,rest:1.,k:20.,limit:5000.,
            conductance:0.,original_rest:1.,yield_strain:1.,plastic_rate:0.,
            damage:0.,damage_onset:0.05,damage_rate:10.,dashpot:0.};
        s.step(0.002);let damaged=s.edges[0].damage;assert!(damaged>0.);
        s.nodes[1].p=V3::new(1.,2.,0.);s.nodes[0].v=V3::default();s.nodes[1].v=V3::default();
        s.step(0.002);assert_eq!(s.edges[0].damage,damaged);
    }
    #[test] fn heating_and_pull_have_effect(){let mut s=sim(0);
        let energy=s.total_energy();s.heat();assert!(s.total_energy()>energy);
        s.pull();assert!(s.nodes[0].v.x<0.);
    }
}

mod checkpoint_v7;
