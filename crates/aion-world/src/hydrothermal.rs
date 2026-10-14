//! PRIMORDIA 0.15: conservative, finite-volume porous-water + sensible-heat prototype.
//!
//! Domain: a *finite patch* of 2 m cubic cells. Solid material comes from the
//! geologic voxel model. Porosity, saturated conductivity, thermal conductivity,
//! suction and heat capacity are synthetic illustrative parameterizations, NOT
//! calibrated field measurements or a full Richards / Navier-Stokes solver.
//!
//! For each internal shared face: Q = K_harmonic * A/L * (H_i-H_j) [m3/s],
//! with H = elevation + linear capillary proxy. Q is bounded by source water
//! and target pore volume. Water carries sensible energy rho*cp*T; conductive
//! energy uses Fourier's law. Sealed exterior boundaries. No external water or
//! heat unless explicitly injected/withdrawn. Energy is referenced to 0 K.
use std::path::Path;
use crate::{World, VoxelWorld, VoxelKey, Material, VOXEL_VOLUME_M3};

pub const WATER_DENSITY_KG_M3: f64=1000.0;
pub const WATER_HEAT_J_KG_K:f64=4180.0;
pub const CELL_SIDE_M:f64=2.0;
pub const FACE_AREA_M2:f64=4.0;
pub const MAX_PATCH_CELLS:usize=32*32*32;
const MAGIC:&[u8;8]=b"AIONH015";
const MAX_RECORD_BYTES:usize=108+17*MAX_PATCH_CELLS;

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct HydroMaterial {
    pub porosity:f64,
    /// Saturated hydraulic conductivity (m/s), not intrinsic permeability (m2).
    pub conductivity_m_s:f64,
    pub thermal_conductivity_w_mk:f64,
    pub solid_heat_capacity_j_m3k:f64,
    pub capillary_suction_m:f64,
}
impl HydroMaterial {
    pub fn from_material(m:Option<Material>)->Self {
        // Representative broad orders of magnitude, not universal constants.
        match m {
            Some(Material::Soil)=>Self::new(0.35,1.0e-5,1.4,1.8e6,2.0),
            Some(Material::Sand)=>Self::new(0.34,1.5e-4,1.6,1.6e6,0.8),
            Some(Material::Clay)=>Self::new(0.43,3.0e-9,1.2,1.9e6,9.0),
            Some(Material::Gravel)=>Self::new(0.27,5.0e-4,1.8,1.9e6,0.3),
            Some(Material::Peat)=>Self::new(0.75,4.0e-6,0.6,0.8e6,4.0),
            Some(Material::Limestone)=>Self::new(0.12,5.0e-7,2.2,2.2e6,3.0),
            Some(Material::Granite)=>Self::new(0.015,1.0e-10,2.7,2.2e6,5.0),
            Some(Material::Basalt)=>Self::new(0.035,8.0e-9,1.9,2.4e6,4.0),
            None=>Self::new(1.0,2.0e-2,0.026,1.2e3,6.0), // cave air: an unconfined proxy
        }
    }
    pub const fn new(porosity:f64,conductivity_m_s:f64,thermal_conductivity_w_mk:f64,solid_heat_capacity_j_m3k:f64,capillary_suction_m:f64)->Self {
        Self{porosity,conductivity_m_s,thermal_conductivity_w_mk,solid_heat_capacity_j_m3k,capillary_suction_m}
    }
    pub fn validate(&self)->bool {
        self.porosity>0.0&&self.porosity<=1.0&&self.conductivity_m_s>0.0&&
        self.thermal_conductivity_w_mk>0.0&&self.solid_heat_capacity_j_m3k>0.0&&
        self.capillary_suction_m>=0.0&&[
            self.porosity,self.conductivity_m_s,self.thermal_conductivity_w_mk,
            self.solid_heat_capacity_j_m3k,self.capillary_suction_m
        ].iter().all(|v|v.is_finite())
    }
}

#[derive(Clone,Debug)]
pub struct HydroCell {
    pub key:VoxelKey,
    pub material:Option<Material>,
    pub properties:HydroMaterial,
    pub water_m3:f64,
    pub energy_j:f64,
}
impl HydroCell {
    pub fn capacity_m3(&self)->f64{self.properties.porosity*VOXEL_VOLUME_M3}
    pub fn saturation(&self)->f64{(self.water_m3/self.capacity_m3()).clamp(0.0,1.0)}
    pub fn heat_capacity_j_k(&self)->f64{
        VOXEL_VOLUME_M3*(1.0-self.properties.porosity)*self.properties.solid_heat_capacity_j_m3k
          +self.water_m3*WATER_DENSITY_KG_M3*WATER_HEAT_J_KG_K
          +VOXEL_VOLUME_M3*self.properties.porosity*1.2e3 // pore gas, simple constant
    }
    pub fn temperature_k(&self)->f64{self.energy_j/self.heat_capacity_j_k()}
    pub fn hydraulic_head_m(&self)->f64{
        // Capillary pressure-head approximation; not a calibrated retention curve.
        self.key.center()[1] - self.properties.capillary_suction_m*(1.0-self.saturation())
    }
    pub fn unsaturated_conductivity_m_s(&self)->f64{
        let s=self.saturation();
        self.properties.conductivity_m_s*(0.02+0.98*s*s*s)
    }
    pub fn valid(&self)->bool{
        self.properties.validate() && self.water_m3>=0.0&&self.water_m3<=self.capacity_m3()+1.0e-9&&
        self.water_m3.is_finite()&&self.energy_j.is_finite()&&self.temperature_k()>0.0&&self.temperature_k()<1000.0
    }
}

#[derive(Clone,Copy,Debug)]
pub struct HydroReport {
    pub time_s:f64,pub total_water_m3:f64,pub injected_water_m3:f64,
    pub withdrawn_water_m3:f64,pub total_energy_j:f64,
    pub energy_in_j:f64,pub energy_out_j:f64,
    pub mean_temperature_k:f64,pub fingerprint:u64,
}
#[derive(Clone,Debug)]
pub struct HydroThermal {
    pub seed:u64,
    pub origin:VoxelKey,
    pub dims:[usize;3],
    pub cells:Vec<HydroCell>,
    pub time_s:f64,
    initial_water_m3:f64,
    initial_energy_j:f64,
    injected_water_m3:f64,
    withdrawn_water_m3:f64,
    energy_in_j:f64,
    energy_out_j:f64,
}
impl HydroThermal {
    pub fn from_voxels(world:&World,voxels:&VoxelWorld,origin:VoxelKey,dims:[usize;3])->Result<Self,String>{
        if voxels.seed!=world.seed {return Err("Hydrology: world seed mismatch".into());}
        let count=dims.iter().try_fold(1usize,|a,&n|if n==0||n>32 {None}else{a.checked_mul(n)})
            .filter(|&n|n<=MAX_PATCH_CELLS).ok_or("Hydrology: invalid patch dimensions")?;
        let mut cells=Vec::with_capacity(count);
        for z in 0..dims[2]{for y in 0..dims[1]{for x in 0..dims[0]{
            let key=origin.neighbour([x as i32,y as i32,z as i32]).ok_or("Patch coordinate overflow")?;
            let material=voxels.material(world,key);
            let properties=HydroMaterial::from_material(material);
            let init_fraction=if material.is_none(){0.0}else{0.42};
            let water_m3=properties.porosity*VOXEL_VOLUME_M3*init_fraction;
            let mut cell=HydroCell{key,material,properties,water_m3,energy_j:0.0};
            // Synthetic temperature gradient of 24 K/km, tied to depth not time.
            let temperature_k=(283.15+(world.elevation_m(key.center()[0],key.center()[2])-key.center()[1]).max(0.0)*0.024).min(400.0);
            cell.energy_j=cell.heat_capacity_j_k()*temperature_k;
            cells.push(cell);
        }}}
        let initial_water_m3=cells.iter().map(|c|c.water_m3).sum();
        let initial_energy_j=cells.iter().map(|c|c.energy_j).sum();
        Ok(Self{seed:world.seed,origin,dims,cells,time_s:0.0,initial_water_m3,initial_energy_j,
            injected_water_m3:0.0,withdrawn_water_m3:0.0,energy_in_j:0.0,energy_out_j:0.0})
    }
    pub fn index(&self,x:usize,y:usize,z:usize)->Option<usize>{
        if x>=self.dims[0]||y>=self.dims[1]||z>=self.dims[2]{None}
        else{Some((z*self.dims[1]+y)*self.dims[0]+x)}
    }
    pub fn water_m3(&self)->f64{self.cells.iter().map(|c|c.water_m3).sum()}
    pub fn energy_j(&self)->f64{self.cells.iter().map(|c|c.energy_j).sum()}
    pub fn mass_residual_m3(&self)->f64{
        self.water_m3()-(self.initial_water_m3+self.injected_water_m3-self.withdrawn_water_m3)
    }
    pub fn energy_residual_j(&self)->f64{
        self.energy_j()-(self.initial_energy_j+self.energy_in_j-self.energy_out_j)
    }
    pub fn inject(&mut self,index:usize,want_m3:f64,temperature_k:f64)->Result<f64,String>{
        if !want_m3.is_finite()||want_m3<0.0||!temperature_k.is_finite()||!(200.0..=500.0).contains(&temperature_k){return Err("Invalid rain parameters".into());}
        let cell=self.cells.get_mut(index).ok_or("Cell out of range")?;
        let accepted=want_m3.min((cell.capacity_m3()-cell.water_m3).max(0.0));
        let energy=accepted*WATER_DENSITY_KG_M3*WATER_HEAT_J_KG_K*temperature_k;
        cell.water_m3+=accepted;cell.energy_j+=energy;
        self.injected_water_m3+=accepted;self.energy_in_j+=energy;
        Ok(accepted)
    }
    pub fn withdraw(&mut self,index:usize,want_m3:f64)->Result<f64,String>{
        if !want_m3.is_finite()||want_m3<0.0{return Err("Invalid withdrawal".into())}
        let cell=self.cells.get_mut(index).ok_or("Cell out of range")?;
        let actual=want_m3.min(cell.water_m3);
        let energy=actual*WATER_DENSITY_KG_M3*WATER_HEAT_J_KG_K*cell.temperature_k();
        cell.water_m3-=actual;cell.energy_j-=energy;
        self.withdrawn_water_m3+=actual;self.energy_out_j+=energy;
        Ok(actual)
    }
    pub fn step(&mut self,dt:f64)->Result<(),String>{
        if !dt.is_finite()||dt<=0.0||dt>60.0{return Err("Hydrology dt must be (0,60] s".into())}
        for z in 0..self.dims[2] {for y in 0..self.dims[1] {for x in 0..self.dims[0] {
            let i=self.index(x,y,z).unwrap();
            for &(dx,dy,dz) in &[(1,0,0),(0,1,0),(0,0,1)] {
                if let Some(j)=self.index(x+dx,y+dy,z+dz){
                    let (left,right)=self.cells.split_at_mut(j);
                    exchange(&mut left[i],&mut right[0],dt);
                }
            }
        }}}
        self.time_s+=dt;
        if !self.cells.iter().all(HydroCell::valid){return Err("Nonphysical hydrology state".into())}
        Ok(())
    }
    pub fn report(&self)->HydroReport{
        let weighted=self.cells.iter().map(|c|c.temperature_k()*c.heat_capacity_j_k()).sum::<f64>();
        let hc=self.cells.iter().map(HydroCell::heat_capacity_j_k).sum::<f64>();
        HydroReport{time_s:self.time_s,total_water_m3:self.water_m3(),
            injected_water_m3:self.injected_water_m3,withdrawn_water_m3:self.withdrawn_water_m3,
            total_energy_j:self.energy_j(),energy_in_j:self.energy_in_j,energy_out_j:self.energy_out_j,
            mean_temperature_k:weighted/hc,fingerprint:self.fingerprint()}
    }
    pub fn fingerprint(&self)->u64{
        let mut h=0xcbf29ce484222325u64;
        for v in std::iter::once(self.time_s).chain(self.cells.iter().flat_map(|c|[c.water_m3,c.energy_j])){
            for b in v.to_bits().to_le_bytes(){h=(h^u64::from(b)).wrapping_mul(0x100000001b3);}
        }h
    }
    /// Portable AIONH015 snapshot; fixed little-endian, checksum and budget checks.
    /// The voxel/earth geometry must be restored first: rock material is validated
    /// against the original reference cells before applying the saved fluid state.
    pub fn save(&self,path:&Path)->Result<(),String>{
        if !self.cells.iter().all(HydroCell::valid)||self.mass_residual_m3().abs()>1.0e-7||
           self.energy_residual_j().abs()>1.0e-8*self.initial_energy_j.abs().max(1.0){
            return Err("Hydrology save: invalid state or budget".into());
        }
        let mut b=Vec::with_capacity(108+self.cells.len()*17);
        b.extend_from_slice(MAGIC);
        b.extend_from_slice(&self.seed.to_le_bytes());
        for v in [self.origin.x,self.origin.y,self.origin.z] {b.extend_from_slice(&v.to_le_bytes());}
        for v in self.dims {b.extend_from_slice(&(v as u32).to_le_bytes());}
        for v in [self.time_s,self.initial_water_m3,self.initial_energy_j,
                  self.injected_water_m3,self.withdrawn_water_m3,self.energy_in_j,self.energy_out_j] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b.extend_from_slice(&(self.cells.len() as u32).to_le_bytes());
        for c in &self.cells {
            b.push(c.material.map(|m|m as u8).unwrap_or(255));
            b.extend_from_slice(&c.water_m3.to_le_bytes());
            b.extend_from_slice(&c.energy_j.to_le_bytes());
        }
        let sum=checksum64(&b);
        b.extend_from_slice(&sum.to_le_bytes());
        std::fs::write(path,b).map_err(|e|e.to_string())
    }
    pub fn load(path:&Path,world:&World,voxels:&VoxelWorld)->Result<Self,String>{
        let b=std::fs::read(path).map_err(|e|e.to_string())?;
        if b.len()<108||b.len()>MAX_RECORD_BYTES||&b[0..8]!=MAGIC{return Err("Invalid AIONH015 header/size".into());}
        let get4=|at:usize|i32::from_le_bytes(b[at..at+4].try_into().unwrap());
        let getu4=|at:usize|u32::from_le_bytes(b[at..at+4].try_into().unwrap());
        let get8=|at:usize|f64::from_le_bytes(b[at..at+8].try_into().unwrap());
        let seed=u64::from_le_bytes(b[8..16].try_into().unwrap());
        if seed!=world.seed||seed!=voxels.seed{return Err("Hydrology save belongs to another world".into());}
        let origin=VoxelKey{x:get4(16),y:get4(20),z:get4(24)};
        let dims=[getu4(28) as usize,getu4(32) as usize,getu4(36) as usize];
        let count=getu4(96) as usize;
        if count>MAX_PATCH_CELLS||b.len()!=108+17*count{return Err("Invalid hydro-cell count".into());}
        let expected=u64::from_le_bytes(b[b.len()-8..].try_into().unwrap());
        if checksum64(&b[..b.len()-8])!=expected{return Err("Hydrology checksum mismatch".into());}
        let mut s=Self::from_voxels(world,voxels,origin,dims)?;
        if s.cells.len()!=count{return Err("Hydrology patch shape mismatch".into());}
        s.time_s=get8(40);s.initial_water_m3=get8(48);s.initial_energy_j=get8(56);
        s.injected_water_m3=get8(64);s.withdrawn_water_m3=get8(72);
        s.energy_in_j=get8(80);s.energy_out_j=get8(88);
        for (i,c) in s.cells.iter_mut().enumerate(){
            let offset=100+i*17;
            let encoded=b[offset];
            if encoded!=c.material.map(|m|m as u8).unwrap_or(255){return Err("Hydrology rock geometry changed".into())}
            c.water_m3=get8(offset+1);
            c.energy_j=get8(offset+9);
        }
        if !s.time_s.is_finite()||s.time_s<0.0||
            ![s.initial_water_m3,s.initial_energy_j,s.injected_water_m3,
              s.withdrawn_water_m3,s.energy_in_j,s.energy_out_j].iter().all(|v|v.is_finite()&&*v>=0.0)||
            !s.cells.iter().all(HydroCell::valid)||
            s.mass_residual_m3().abs()>1.0e-8*(1.0+s.initial_water_m3)||
            s.energy_residual_j().abs()>1.0e-8*(1.0+s.initial_energy_j){
            return Err("Hydrology physical conservation/invariant violated".into());
        }
        Ok(s)
    }
    pub fn write_benchmark(path:&Path,seed:u64)->Result<(),String>{
        let world=World::new(seed);
        let vox=VoxelWorld::new(seed);
        let h=world.elevation_m(0.0,0.0);
        let origin=VoxelKey::at(-8.0,(h/2.0).floor()*2.0-10.0,-8.0).ok_or("Invalid origin")?;
        let mut sim=Self::from_voxels(&world,&vox,origin,[8,6,8])?;
        let mut out=String::from("time_s,water_m3,injected_m3,withdrawn_m3,water_residual_m3,energy_j,energy_residual_j,mean_temp_k,fingerprint\n");
        for step in 0..=60 {
            if step>0 {
                let surface=sim.index(4,5,4).unwrap();
                let _=sim.inject(surface,0.03,285.15)?;
                sim.step(20.0)?;
            }
            let r=sim.report();
            out.push_str(&format!("{:.3},{:.12},{:.12},{:.12},{:.12},{:.6},{:.6},{:.6},{:016x}\n",
                r.time_s,r.total_water_m3,r.injected_water_m3,r.withdrawn_water_m3,sim.mass_residual_m3(),
                r.total_energy_j,sim.energy_residual_j(),r.mean_temperature_k,r.fingerprint));
        }
        std::fs::write(path,out).map_err(|e|e.to_string())
    }
}
fn checksum64(bytes:&[u8])->u64{
    bytes.iter().fold(0xcbf29ce484222325u64,|h,&b|(h^u64::from(b)).wrapping_mul(0x100000001b3))
}
fn harmonic(a:f64,b:f64)->f64 {if a<=0.0||b<=0.0 {0.0}else{2.0*a*b/(a+b)}}
fn exchange(a:&mut HydroCell,b:&mut HydroCell,dt:f64){
    let conductivity=harmonic(a.unsaturated_conductivity_m_s(),b.unsaturated_conductivity_m_s());
    let flux_m3=conductivity*FACE_AREA_M2/CELL_SIDE_M*(a.hydraulic_head_m()-b.hydraulic_head_m())*dt;
    if flux_m3>0.0 {transport(a,b,flux_m3)}
    else if flux_m3<0.0{transport(b,a,-flux_m3)}
    let t_a=a.temperature_k();let t_b=b.temperature_k();
    let thermal=harmonic(a.properties.thermal_conductivity_w_mk,b.properties.thermal_conductivity_w_mk);
    let q=thermal*FACE_AREA_M2/CELL_SIDE_M*(t_a-t_b)*dt;
    let capacity_a=a.heat_capacity_j_k();let capacity_b=b.heat_capacity_j_k();
    let max_q=(t_a-t_b)/(1.0/capacity_a+1.0/capacity_b);
    // Bound by a complete equalization: avoid thermal overshoot under large dt.
    let moved=if q>=0.0{q.min(max_q.max(0.0))}else{q.max(max_q.min(0.0))};
    a.energy_j-=moved;b.energy_j+=moved;
}
fn transport(donor:&mut HydroCell,recipient:&mut HydroCell,requested_m3:f64){
    let delta=requested_m3.min(donor.water_m3).min((recipient.capacity_m3()-recipient.water_m3).max(0.0));
    if delta<=0.0{return}
    let e=delta*WATER_DENSITY_KG_M3*WATER_HEAT_J_KG_K*donor.temperature_k();
    donor.water_m3-=delta;recipient.water_m3+=delta;
    donor.energy_j-=e;recipient.energy_j+=e;
}

#[cfg(test)] mod tests {
    use super::*;
    fn patch(seed:u64)->HydroThermal {
        let w=World::new(seed);let v=VoxelWorld::new(seed);
        HydroThermal::from_voxels(&w,&v,VoxelKey::at(0.,-24.,0.).unwrap(),[4,4,4]).unwrap()
    }
    #[test] fn parameters_are_positive_and_rocks_differ(){
        for m in Material::ALL {assert!(HydroMaterial::from_material(Some(m)).validate());}
        assert!(HydroMaterial::from_material(Some(Material::Gravel)).conductivity_m_s>
                HydroMaterial::from_material(Some(Material::Clay)).conductivity_m_s*1000.0);
        assert!(HydroMaterial::from_material(None).porosity>0.99);
    }
    #[test] fn sealed_domain_preserves_mass_and_enthalpy(){
        let mut s=patch(102);let m0=s.water_m3();let e0=s.energy_j();
        for _ in 0..600{s.step(20.0).unwrap();}
        assert!((s.water_m3()-m0).abs()<1e-8);
        assert!((s.energy_j()-e0).abs()/e0<1e-12);
        assert!(s.cells.iter().all(HydroCell::valid));
        assert_eq!(s.time_s,12000.0);
    }
    #[test] fn injection_withdrawal_have_explicit_open_budget(){
        let mut s=patch(41);let index=3;let e0=s.energy_j();
        let added=s.inject(index,0.3,293.15).unwrap();assert!(added>0.0);
        let removed=s.withdraw(index,0.05).unwrap();assert!(removed>0.0);
        for _ in 0..50{s.step(12.0).unwrap();}
        assert!(s.mass_residual_m3().abs()<1e-9);
        assert!(s.energy_residual_j().abs()/e0<1e-12);
        assert!(s.inject(index,0.4,-1.0).is_err());
        assert!(s.step(-1.0).is_err());
    }
    #[test] fn gravity_moves_water_downward_in_open_cells(){
        let mut s=patch(44);
        let a=s.index(1,1,1).unwrap();let b=s.index(1,0,1).unwrap();
        for &i in &[a,b] {
            s.cells[i].material=None;
            s.cells[i].properties=HydroMaterial::from_material(None);
            s.cells[i].water_m3=0.0;
            s.cells[i].energy_j=s.cells[i].heat_capacity_j_k()*290.0;
        }
        s.cells[a].water_m3=2.0;
        s.cells[a].energy_j=s.cells[a].heat_capacity_j_k()*290.0;
        let prior=s.cells[b].water_m3;
        s.step(20.0).unwrap();
        assert!(s.cells[b].water_m3>prior);
        assert!(s.cells[a].water_m3<2.0);
    }
    #[test] fn different_conductivities_change_transport_by_orders_of_magnitude(){
        let mut gravel=patch(8);let mut clay=gravel.clone();
        let upper=gravel.index(1,2,1).unwrap();let lower=gravel.index(1,1,1).unwrap();
        for sim in [&mut gravel,&mut clay] {
            for i in [upper,lower] {sim.cells[i].water_m3=sim.cells[i].capacity_m3()*0.5;sim.cells[i].energy_j=sim.cells[i].heat_capacity_j_k()*290.0;}
        }
        for (sim,mat) in [(&mut gravel,Material::Gravel),(&mut clay,Material::Clay)] {
            for i in [upper,lower] {
                sim.cells[i].material=Some(mat);
                sim.cells[i].properties=HydroMaterial::from_material(Some(mat));
                sim.cells[i].water_m3=sim.cells[i].capacity_m3()*0.5;
                sim.cells[i].energy_j=sim.cells[i].heat_capacity_j_k()*290.0;
            }
        }
        // Isolate transfer by invoking only the shared face, rather than other neighbours.
        let (ga,gb)=gravel.cells.split_at_mut(upper);let old_g=ga[lower].water_m3;
        exchange(&mut gb[0],&mut ga[lower],20.0);
        let moved_g=ga[lower].water_m3-old_g;
        let (ca,cb)=clay.cells.split_at_mut(upper);let old_c=ca[lower].water_m3;
        exchange(&mut cb[0],&mut ca[lower],20.0);
        let moved_c=ca[lower].water_m3-old_c;
        assert!(moved_g>0.0 && moved_c>0.0 && moved_g>moved_c*1000.0);
    }
    #[test] fn conduction_equalizes_temperature_without_creating_energy(){
        let mut s=patch(1);let low=s.index(0,0,0).unwrap();let high=s.index(1,0,0).unwrap();
        for i in [low,high] {s.cells[i].water_m3=0.0;s.cells[i].energy_j=s.cells[i].heat_capacity_j_k()*(if i==low{320.0}else{280.0});}
        let before=s.cells[low].energy_j+s.cells[high].energy_j;
        let t0=s.cells[low].temperature_k();let t1=s.cells[high].temperature_k();
        let (a,b)=s.cells.split_at_mut(high);exchange(&mut a[low],&mut b[0],60.0);
        assert!(s.cells[low].temperature_k()<t0 && s.cells[high].temperature_k()>t1);
        let after=s.cells[low].energy_j+s.cells[high].energy_j;
        assert!((after-before).abs()<1e-5);
    }
    #[test] fn deterministic_steps_and_fingerprint(){
        let mut a=patch(42);let mut b=patch(42);
        for i in 0..40 {a.inject(6,0.04,285.0).unwrap();b.inject(6,0.04,285.0).unwrap();a.step(10.0).unwrap();b.step(10.0).unwrap();if i==20{assert_eq!(a.fingerprint(),b.fingerprint());}}
        assert_eq!(a.fingerprint(),b.fingerprint());
        assert_ne!(a.fingerprint(),patch(42).fingerprint());
    }
    #[test] fn volume_state_follows_excavated_voxels(){
        let w=World::new(51);let mut v=VoxelWorld::new(51);
        let k=VoxelKey::at(0.,-21.,0.).unwrap();v.dig(&w,k).unwrap();
        let h=HydroThermal::from_voxels(&w,&v,k,[2,2,2]).unwrap();
        assert_eq!(h.cells[0].material,None);
        assert_eq!(h.cells[0].capacity_m3(),VOXEL_VOLUME_M3);
    }
    #[test] fn snapshot_roundtrip_and_tamper_rejection(){
        let w=World::new(62);let vox=VoxelWorld::new(w.seed);let mut sim=patch(w.seed);
        sim.inject(4,0.08,285.0).unwrap();
        for _ in 0..12{sim.step(10.0).unwrap()}
        let path=std::env::temp_dir().join(format!("aionh015-{}-{}.bin",std::process::id(),w.seed));
        sim.save(&path).unwrap();
        let copy=HydroThermal::load(&path,&w,&vox).unwrap();
        assert_eq!(copy.fingerprint(),sim.fingerprint());
        assert!((copy.water_m3()-sim.water_m3()).abs()<1e-11);
        assert!(HydroThermal::load(&path,&World::new(63),&VoxelWorld::new(63)).is_err());
        let mut bytes=std::fs::read(&path).unwrap();bytes[110]^=0x42;
        std::fs::write(&path,&bytes).unwrap();
        assert!(HydroThermal::load(&path,&w,&vox).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test] fn report_units_and_cell_indices(){
        let s=patch(3);assert!(s.index(4,0,0).is_none());assert!(s.index(0,4,0).is_none());
        let r=s.report();assert_eq!(r.time_s,0.0);assert!(r.mean_temperature_k>250.0);
    }
}