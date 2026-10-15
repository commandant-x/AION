use super::{SurfaceWater,HydroThermal};
use aion_world::hydrothermal::CELL_SIDE_M;
impl SurfaceWater {
 /// One-way K_unsaturated*area*dt transfer to aligned 2m hydro top cells.
 /// Not Richards flow: capillarity and exfiltration are omitted. HydroThermal
 /// accounts for transferred sensible energy at the prescribed water T [K].
 pub fn infiltrate_into(&mut self,hydro:&mut HydroThermal,dt:f64,
                        temperature_k:f64)->Result<f64,String>{
  if self.cell_side_m!=CELL_SIDE_M||hydro.dims[0]!=self.width||
     hydro.dims[2]!=self.depth||!dt.is_finite()||dt<=0.0||dt>60.0||
     !temperature_k.is_finite()||!(200.0..=500.0).contains(&temperature_k){
   return Err("Infiltration needs aligned 2m grids and valid inputs".into())
  }
  let mut total=0.0;
  let area=self.cell_side_m.powi(2);
  for z in 0..self.depth{for x in 0..self.width{
   let i=z*self.width+x;
   let j=hydro.index(x,hydro.dims[1]-1,z).unwrap();
   let wanted=(hydro.cells[j].unsaturated_conductivity_m_s()*area*dt)
    .min(self.cells[i].water_m3);
   let accepted=hydro.inject(j,wanted,temperature_k)?;
   self.cells[i].water_m3-=accepted;total+=accepted;
  }}
  self.infiltrated_m3+=total;Ok(total)
 }
}
