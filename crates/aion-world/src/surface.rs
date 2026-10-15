//! Conservative 2-D finite-volume surface water [SI].
//! Sealed boundaries; uniform rain is a measured external source.
//! Manning diffusive-wave runoff and one-way infiltration are approximations,
//! not a momentum solver or a calibrated catchment model.
use crate::hydrothermal::HydroThermal;
mod runoff;
mod infiltration;
#[cfg(test)] mod tests;

#[derive(Clone,Copy,Debug)]
pub struct SurfaceCell {pub bed_m:f64,pub water_m3:f64}
#[derive(Clone,Copy,Debug)]
pub struct SurfaceReport {
    pub time_s:f64,pub surface_water_m3:f64,pub supplied_m3:f64,
    pub rain_m3:f64,pub infiltrated_m3:f64,pub residual_m3:f64,
}
#[derive(Clone,Debug)]
pub struct SurfaceWater {
    pub width:usize,pub depth:usize,pub cell_side_m:f64,
    pub manning_n:f64,pub cells:Vec<SurfaceCell>,pub time_s:f64,
    supplied_m3:f64,rain_m3:f64,infiltrated_m3:f64,
}
impl SurfaceWater {
    pub fn new(width:usize,depth:usize,side_m:f64,beds:Vec<f64>,n:f64)->Result<Self,String>{
        let count=width.checked_mul(depth).ok_or("Surface dimensions overflow")?;
        if width==0||depth==0||width>256||depth>256||beds.len()!=count||
           !side_m.is_finite()||side_m<=0.0||!n.is_finite()||
           !(0.01..=1.0).contains(&n)||!beds.iter().all(|b|b.is_finite()){
            return Err("Invalid surface grid or Manning n".into());
        }
        Ok(Self{width,depth,cell_side_m:side_m,manning_n:n,
            cells:beds.into_iter().map(|bed_m|SurfaceCell{bed_m,water_m3:0.0}).collect(),
            time_s:0.0,supplied_m3:0.0,rain_m3:0.0,infiltrated_m3:0.0})
    }
    pub fn index(&self,x:usize,z:usize)->Option<usize>{
        if x<self.width&&z<self.depth{Some(z*self.width+x)}else{None}
    }
    pub fn water_m3(&self)->f64{self.cells.iter().map(|c|c.water_m3).sum()}
    pub fn residual_m3(&self)->f64{
        self.water_m3()+self.infiltrated_m3-self.supplied_m3-self.rain_m3
    }
    pub fn report(&self)->SurfaceReport{
        SurfaceReport{time_s:self.time_s,surface_water_m3:self.water_m3(),
            supplied_m3:self.supplied_m3,rain_m3:self.rain_m3,
            infiltrated_m3:self.infiltrated_m3,residual_m3:self.residual_m3()}
    }
    pub fn add_water(&mut self,i:usize,volume_m3:f64)->Result<(),String>{
        if !volume_m3.is_finite()||volume_m3<0.0{return Err("Invalid water volume".into())}
        let c=self.cells.get_mut(i).ok_or("Invalid surface index")?;
        if !(c.water_m3+volume_m3).is_finite(){return Err("Surface volume overflow".into())}
        c.water_m3+=volume_m3;self.supplied_m3+=volume_m3;Ok(())
    }
}
