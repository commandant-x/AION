use super::*;
#[test] fn uniform_rain_is_accounted_for(){
 let mut s=grid(vec![0.0;16],4,0.04);
 s.step(60.0,1.0e-5).unwrap();
 let r=s.report();
 assert!((r.rain_m3-0.0384).abs()<1e-12);
 assert!((r.surface_water_m3-r.rain_m3).abs()<1e-12);
 assert!(r.residual_m3.abs()<1e-12);
}
#[test] fn downhill_runoff_keeps_positive_water(){
 let mut s=grid(vec![1.0,0.0],2,0.03);
 s.add_water(0,0.4).unwrap();s.step(10.0,0.0).unwrap();
 assert!(s.cells[1].water_m3>0.0);
 assert!(s.cells.iter().all(|c|c.water_m3>=0.0));
 assert!(s.residual_m3().abs()<1e-12);
}
#[test] fn invalid_inputs_do_not_change_state(){
 let mut s=grid(vec![0.0,0.0],2,0.05);
 assert!(s.step(-1.0,0.0).is_err());
 assert!(s.step(1.0,f64::NAN).is_err());
 assert!(s.add_water(4,1.0).is_err());
 assert_eq!(s.report().time_s,0.0);
 assert_eq!(s.water_m3(),0.0);
}
