use super::*;
use aion_world::{World,VoxelWorld,VoxelKey};

#[test]
fn invalid_steps_do_not_change_water_or_time() {
    let mut s = grid(vec![0.0, 0.1], 2, 0.1);
    s.add_water(0, 0.5).unwrap();
    let before = s.report();
    for (dt, rain) in [(0.0, 0.0), (-1.0, 0.0), (61.0, 0.0),
                       (1.0, -1e-6), (f64::NAN, 0.0),
                       (1.0, f64::INFINITY)] {
        assert!(s.step(dt, rain).is_err());
        assert_eq!(s.report().time_s, before.time_s);
        assert_eq!(s.report().surface_water_m3, before.surface_water_m3);
        assert_eq!(s.report().rain_m3, before.rain_m3);
    }
}

#[test]
fn failed_infiltration_is_transactional() {
    let world = World::new(77);
    let mut voxels = VoxelWorld::new(77);
    let key = VoxelKey::at(0.0, world.elevation_m(0.0, 0.0)-14.0, 0.0).unwrap();
    voxels.dig(&world,key).unwrap();
    let mut hydro = HydroThermal::from_voxels(&world,&voxels,key,[1,1,1]).unwrap();
    let mut surface = grid(vec![0.0],1,0.1);
    surface.add_water(0,0.5).unwrap();
    let before_surface = surface.water_m3();
    let before_hydro = hydro.water_m3();
    assert!(surface.infiltrate_into(&mut hydro,10.0,f64::NAN).is_err());
    assert_eq!(surface.water_m3(),before_surface);
    assert_eq!(hydro.water_m3(),before_hydro);
}
