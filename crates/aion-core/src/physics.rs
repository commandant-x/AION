//! Local, coarse-grained 3D matter particles and breakable bonds.
//! Distances: metres; masses: kilograms; time: seconds; temperature: kelvin.
//! The geometry and mechanical properties are physics data, NOT craft recipes.
use crate::math::Vec3;
use std::f64::consts::PI;

#[derive(Clone,Copy,Debug)]
pub struct Material {
    pub density:f64, pub specific_heat:f64, pub conductivity:f64,
    pub restitution:f64, pub friction:f64,
}
impl Material {
    pub fn validate(self)->bool {
        self.density>0. && self.specific_heat>0. && self.conductivity>=0. &&
        (0.0..=1.0).contains(&self.restitution) && (0.0..=1.0).contains(&self.friction) &&
        [self.density,self.specific_heat,self.conductivity].iter().all(|v|v.is_finite())
    }
}
pub const STONE:Material=Material{density:2650.,specific_heat:790.,conductivity:2.5,restitution:0.25,friction:0.65};
pub const WOOD:Material=Material{density:700.,specific_heat:1700.,conductivity:0.12,restitution:0.25,friction:0.45};

#[derive(Clone,Debug)]
pub struct Particle {
    pub pos:Vec3, pub vel:Vec3, pub radius:f64, pub mass:f64,
    pub temp:f64, pub material:Material,
}
impl Particle {
    pub fn sphere(pos:Vec3,radius:f64,material:Material,temp:f64)->Self {
        assert!(material.validate() && radius>0. && radius.is_finite() && temp>=0. && temp.is_finite());
        let mass=4./3.*PI*radius.powi(3)*material.density;
        Self{pos,vel:Vec3::ZERO,radius,mass,temp,material}
    }
    pub fn capacity(&self)->f64 {self.mass*self.material.specific_heat}
    pub fn heat(&self)->f64 {self.capacity()*self.temp}
    pub fn kinetic(&self)->f64 {0.5*self.mass*self.vel.norm2()}
}
#[derive(Clone,Debug)]
pub struct Bond {
    pub a:usize,pub b:usize,pub rest:f64,pub stiffness:f64,pub break_force:f64,
    pub heat_conductance:f64,pub alive:bool,
}
impl Bond {
    pub fn new(a:usize,b:usize,rest:f64,stiffness:f64,break_force:f64,heat_conductance:f64)->Self {
        assert!(a!=b && rest>0. && stiffness>0. && break_force>0. && heat_conductance>=0.);
        Self{a,b,rest,stiffness,break_force,heat_conductance,alive:true}
    }
}
#[derive(Clone,Debug,PartialEq)]
pub enum Event {Fracture{bond:usize},Contact{a:usize,b:usize}}
#[derive(Clone,Debug)]
pub struct World {
    pub particles:Vec<Particle>,pub bonds:Vec<Bond>,pub gravity:Vec3,
    pub floor:bool,pub time:f64,pub events:Vec<Event>,
}
impl World {
    pub fn new()->Self {Self{particles:Vec::new(),bonds:Vec::new(),gravity:Vec3::new(0.,-9.81,0.),floor:true,time:0.,events:Vec::new()}}
    pub fn add(&mut self,p:Particle)->usize{assert!(p.pos.finite()&&p.vel.finite());let id=self.particles.len();self.particles.push(p);id}
    pub fn bond(&mut self,b:Bond){assert!(b.a<self.particles.len()&&b.b<self.particles.len());self.bonds.push(b);}
    pub fn total_mass(&self)->f64 {self.particles.iter().map(|p|p.mass).sum()}
    pub fn momentum(&self)->Vec3 {self.particles.iter().fold(Vec3::ZERO,|acc,p|acc+p.vel*p.mass)}
    pub fn total_energy(&self)->f64 {
        let body:f64=self.particles.iter().map(|p|p.kinetic()+p.heat()-p.mass*self.gravity.dot(p.pos)).sum();
        let springs:f64=self.bonds.iter().filter(|b|b.alive).map(|b| {
            let ext=(self.particles[b.b].pos-self.particles[b.a].pos).norm()-b.rest;
            0.5*b.stiffness*ext*ext
        }).sum();body+springs
    }
    pub fn step(&mut self,dt:f64) {
        assert!(dt.is_finite() && dt>0. && dt<=0.2,"step must be 0 < dt <= 0.2 s");
        // Bound numerical step. This is only a baseline, not a fully implicit stiff solver.
        let n=(dt/0.002).ceil() as usize;
        let h=dt/n as f64;
        self.events.clear();
        for _ in 0..n {self.substep(h);self.time+=h;}
        debug_assert!(self.particles.iter().all(|p|p.pos.finite()&&p.vel.finite()&&p.temp.is_finite()&&p.temp>=0.));
    }
    fn substep(&mut self,dt:f64){
        let mut force:Vec<Vec3>=self.particles.iter().map(|p|self.gravity*p.mass).collect();
        for (index,b) in self.bonds.iter_mut().enumerate(){
            if !b.alive {continue;}
            let (a,c)=(&self.particles[b.a],&self.particles[b.b]);
            let delta=c.pos-a.pos;let dist=delta.norm();let f=b.stiffness*(dist-b.rest);
            if f.abs()>b.break_force {
                b.alive=false;
                let stored=0.5*b.stiffness*(dist-b.rest).powi(2);
                add_heat_pair(&mut self.particles,b.a,b.b,stored);
                self.events.push(Event::Fracture{bond:index});
                continue;
            }
            let unit=delta.normalized();force[b.a]+=unit*f;force[b.b]-=unit*f;
            // Conservative, capacity-limited heat exchange (W/K * K * s).
            let ca=a.capacity();let cb=c.capacity();
            let difference=a.temp-c.temp;
            let max_q=difference.abs()*ca*cb/(ca+cb);
            let q=(b.heat_conductance*difference*dt).clamp(-max_q,max_q);
            self.particles[b.a].temp-=q/ca;
            self.particles[b.b].temp+=q/cb;
        }
        for (p,f) in self.particles.iter_mut().zip(force) {
            p.vel+=f*(dt/p.mass);p.pos+=p.vel*dt;
        }
        // Penalty-free impulse contacts among spherical particles (O(n^2), local toy scale only).
        for (a,b) in crate::spatial::candidate_pairs(&self.particles) {
            let d=self.particles[b].pos-self.particles[a].pos;
            let r=self.particles[a].radius+self.particles[b].radius;
            let dist=d.norm();if dist>=r {continue;}
            let normal=d.normalized();let inv_a=1./self.particles[a].mass;let inv_b=1./self.particles[b].mass;
            let correction=normal*((r-dist)/(inv_a+inv_b));
            self.particles[a].pos-=correction*inv_a;self.particles[b].pos+=correction*inv_b;
            let vr=(self.particles[b].vel-self.particles[a].vel).dot(normal);
            if vr<0. {
                let before=self.particles[a].kinetic()+self.particles[b].kinetic();
                let e=self.particles[a].material.restitution.min(self.particles[b].material.restitution);
                let j=-(1.+e)*vr/(inv_a+inv_b);
                self.particles[a].vel-=normal*(j*inv_a);self.particles[b].vel+=normal*(j*inv_b);
                let after=self.particles[a].kinetic()+self.particles[b].kinetic();
                add_heat_pair(&mut self.particles,a,b,(before-after).max(0.));
                self.events.push(Event::Contact{a,b});
            }
        }
        if self.floor {for p in &mut self.particles {
            if p.pos.y < p.radius {
                p.pos.y=p.radius;
                if p.vel.y<0. {
                    let before=p.kinetic();
                    p.vel.y=-p.vel.y*p.material.restitution;
                    let damp=(1.-p.material.friction*dt*60.).clamp(0.,1.);
                    p.vel.x*=damp;p.vel.z*=damp;
                    p.temp+=(before-p.kinetic()).max(0.)/p.capacity();
                }
            }
        }}
    }
}
fn add_heat_pair(v:&mut [Particle],a:usize,b:usize,joules:f64) {
    let ca=v[a].capacity();let cb=v[b].capacity();
    v[a].temp+=joules/(ca+cb);v[b].temp+=joules/(ca+cb);
}
impl Default for World {fn default()->Self{Self::new()}}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn conservation_of_mass(){let mut w=World::new();w.add(Particle::sphere(Vec3::new(0.,4.,0.),.1,STONE,300.));w.add(Particle::sphere(Vec3::new(0.,8.,0.),.1,WOOD,280.));let m=w.total_mass();for _ in 0..100{w.step(.01);}assert_eq!(w.total_mass(),m);}
    #[test]fn balanced_thermal_exchange(){let mut w=World::new();w.floor=false;w.gravity=Vec3::ZERO;let a=w.add(Particle::sphere(Vec3::new(0.,2.,0.),.1,STONE,400.));let b=w.add(Particle::sphere(Vec3::new(1.,2.,0.),.1,STONE,200.));w.bond(Bond::new(a,b,1.,10.,1000.,1000.));let e=w.total_energy();for _ in 0..100{w.step(.02);}assert!((w.total_energy()-e).abs()<1e-5);assert!(w.particles[a].temp<400.&&w.particles[b].temp>200.);}
    #[test]fn collision_preserves_horizontal_momentum(){let mut w=World::new();w.floor=false;w.gravity=Vec3::ZERO;let a=w.add(Particle::sphere(Vec3::new(-.2,2.,0.),.1,STONE,300.));let b=w.add(Particle::sphere(Vec3::new(.2,2.,0.),.1,STONE,300.));w.particles[a].vel.x=1.;w.particles[b].vel.x=-1.;let m=w.momentum();for _ in 0..50{w.step(.01);}assert!((w.momentum()-m).norm()<1e-9);}
    #[test]fn breaking_releases_strain_as_heat(){let mut w=World::new();w.floor=false;w.gravity=Vec3::ZERO;let a=w.add(Particle::sphere(Vec3::new(0.,2.,0.),.1,STONE,300.));let b=w.add(Particle::sphere(Vec3::new(3.,2.,0.),.1,STONE,300.));w.bond(Bond::new(a,b,1.,100.,1.,0.));let energy=w.total_energy();w.step(.001);assert!(!w.bonds[0].alive);assert!(w.events.contains(&Event::Fracture{bond:0}));assert!((energy-w.total_energy()).abs()<1e-4);}
}
