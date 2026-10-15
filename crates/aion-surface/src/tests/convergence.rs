use super::*;
#[test] fn timestep_refinement_approaches_reference(){
 fn evolve(dt:f64,n:usize)->f64{
  let mut s=grid(vec![0.05,0.0],2,0.2);
  s.add_water(0,0.4).unwrap();
  for _ in 0..n{s.step(dt,0.0).unwrap();}
  s.cells[1].water_m3
 }
 let reference=evolve(0.001,1000);
 let coarse=evolve(0.1,10);
 let fine=evolve(0.01,100);
 assert!((fine-reference).abs()<(coarse-reference).abs());
}
#[test] fn sealed_catchment_preserves_volume_over_100_steps(){
 let beds=(0..100).map(|i| {
  let x=(i%10) as f64-5.0;let z=(i/10) as f64-5.0;
  0.01*x*x+0.015*z*z
 }).collect();
 let mut s=grid(beds,10,0.12);
 for i in 0..100{s.add_water(i,0.02*(i%7) as f64).unwrap();}
 for _ in 0..100{s.step(0.5,2.0e-7).unwrap();}
 assert!(s.cells.iter().all(|c|c.water_m3.is_finite()&&c.water_m3>=0.0));
 assert!(s.residual_m3().abs()<1e-9);
}
