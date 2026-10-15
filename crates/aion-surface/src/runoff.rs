mod flow;
mod flux;
use super::SurfaceWater;
impl SurfaceWater {
 /// Rainfall [m/s] enters as an external source; sealed boundaries.
 pub fn step(&mut self,dt:f64,rain_rate:f64)->Result<(),String>{
  if !dt.is_finite()||dt<=0.0||dt>60.0||!rain_rate.is_finite()||rain_rate<0.0{
   return Err("Invalid rain/dt".into())
  }
  let rain=rain_rate*self.cell_side_m.powi(2)*dt;
  if !rain.is_finite()||!self.cells.iter().all(|c|(c.water_m3+rain).is_finite()){
   return Err("Rain overflow".into())
  }
  for c in &mut self.cells{c.water_m3+=rain}
  self.rain_m3+=rain*self.cells.len() as f64;
  self.route(dt);
  self.time_s+=dt;Ok(())
 }
}
