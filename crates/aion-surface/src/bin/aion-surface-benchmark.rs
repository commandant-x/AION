use aion_surface::SurfaceWater;
use aion_world::{World,VoxelWorld,VoxelKey,HydroThermal};
fn main()->Result<(),String>{
 let path=std::env::args().nth(1).ok_or("usage: aion-surface-benchmark output.csv")?;
 let world=World::new(77);
 let mut vox=VoxelWorld::new(77);
 let h=world.elevation_m(0.0,0.0);
 let origin=VoxelKey::at(0.0,h-50.0,0.0).ok_or("Invalid origin")?;
 for z in 0..2{for x in 0..2{
  vox.dig(&world,origin.neighbour([x,0,z]).unwrap())?;
 }}
 let mut hydro=HydroThermal::from_voxels(&world,&vox,origin,[2,1,2])?;
 let mut surface=SurfaceWater::new(2,2,2.0,vec![0.4,0.2,0.2,0.0],0.08)?;
 surface.add_water(0,0.6)?;surface.add_water(3,0.1)?;
 let mut csv=String::from("step,time_s,surface_m3,hydro_m3,supplied_m3,rain_m3,infiltrated_m3,surface_residual_m3,hydro_residual_m3,hydro_energy_residual_j\n");
 for step in 0..=60{
  if step>0{
   surface.step(10.0,1.0e-5)?;
   surface.infiltrate_into(&mut hydro,10.0,290.0)?;
   hydro.step(10.0)?;
  }
  let r=surface.report();
  csv.push_str(&format!("{step},{:.1},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{:.6}\n",
   r.time_s,r.surface_water_m3,hydro.water_m3(),r.supplied_m3,
   r.rain_m3,r.infiltrated_m3,r.residual_m3,
   hydro.mass_residual_m3(),hydro.energy_residual_j()));
 }
 std::fs::write(path,csv).map_err(|e|e.to_string())
}
