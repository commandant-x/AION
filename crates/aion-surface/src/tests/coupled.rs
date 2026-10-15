use super::*;
use crate::CoupledCatchment;
use aion_world::{World,VoxelWorld,VoxelKey};

fn catchment()->CoupledCatchment {
    let world=World::new(77);
    let mut voxels=VoxelWorld::new(77);
    let key=VoxelKey::at(0.0,world.elevation_m(0.0,0.0)-14.0,0.0).unwrap();
    voxels.dig(&world,key).unwrap();
    let hydro=HydroThermal::from_voxels(&world,&voxels,key,[1,1,1]).unwrap();
    let mut surface=grid(vec![0.0],1,0.1);
    surface.add_water(0,0.5).unwrap();
    CoupledCatchment::new(surface,hydro).unwrap()
}
#[test]
fn coupled_rain_infiltration_conserves_global_mass_and_energy() {
    let mut sim=catchment();
    let initial=sim.surface.water_m3()+sim.hydro.water_m3();
    for _ in 0..30 {
        let report=sim.step(1.0,1e-6,290.0).unwrap();
        assert!(report.total_mass_residual_m3.abs()<1e-8);
        assert!(report.subsurface_energy_residual_j.abs()<1e-2);
    }
    let report=sim.report();
    assert!(report.infiltration_m3>0.0);
    assert!((report.surface_m3+report.subsurface_m3-initial-report.rain_m3).abs()<1e-8);
    assert!((report.time_s-30.0).abs()<1e-10);
}
#[test]
fn coupled_step_is_atomic_on_invalid_inputs() {
    let mut sim=catchment();
    let before=sim.report();
    for (dt,rain,temp) in [(0.0,0.0,290.0),(1.0,-1.0,290.0),
                           (1.0,0.0,f64::NAN),(61.0,0.0,290.0)] {
        assert!(sim.step(dt,rain,temp).is_err());
        assert_eq!(sim.report().time_s,before.time_s);
        assert_eq!(sim.report().surface_m3,before.surface_m3);
        assert_eq!(sim.report().subsurface_m3,before.subsurface_m3);
    }
}
#[test]
fn coupling_rejects_misaligned_grid() {
    let sim=catchment();
    let bad=grid(vec![0.0,0.0],2,0.1);
    assert!(CoupledCatchment::new(bad,sim.hydro).is_err());
}
