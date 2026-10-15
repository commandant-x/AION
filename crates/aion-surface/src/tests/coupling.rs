use super::*;
use aion_world::{World,VoxelWorld,VoxelKey};
#[test] fn infiltration_conserves_water_and_hydrothermal_enthalpy(){
 let world=World::new(77);
 let mut voxels=VoxelWorld::new(77);
 let key=VoxelKey::at(0.0,world.elevation_m(0.0,0.0)-14.0,0.0).unwrap();
 voxels.dig(&world,key).unwrap();
 let mut hydro=HydroThermal::from_voxels(&world,&voxels,key,[1,1,1]).unwrap();
 let mut surface=grid(vec![key.center()[1]+1.0],1,0.04);
 surface.add_water(0,1.0).unwrap();
 let before=hydro.water_m3();
 let moved=surface.infiltrate_into(&mut hydro,10.0,290.0).unwrap();
 assert!(moved>0.0&&moved<1.0);
 assert!((hydro.water_m3()-before-moved).abs()<1e-12);
 assert!(surface.residual_m3().abs()<1e-12);
 assert!(hydro.mass_residual_m3().abs()<1e-12);
 assert!(hydro.energy_residual_j().abs()<1e-3);
}
