//! Baseline computed by native/aion_sim.c on Linux x64, in fixtures/c_reference.csv.
//! Checks the port's early trajectory (and the first brittle-material ruptures).
//! Late nonlinear fracture/contact trajectories are not expected to be bitwise
//! identical between C and Rust across compilers or platforms.
use aion_rust_sim::AionSim;
fn near(actual:f64,expected:f64,relative:f64,absolute:f64){
    assert!((actual-expected).abs() <= absolute + relative * expected.abs(),
        "expected {expected:0.12}, got {actual:0.12}");
}
#[test]
fn compare_c_golden_short_trajectories(){
    let csv=include_str!("fixtures/c_reference.csv");
    let mut worlds:Vec<AionSim>=(0..3).map(|mode|{
        let mut s=AionSim::laboratory(mode);s.pull();s.heat();s
    }).collect();
    let mut completed=[0usize;3];
    for line in csv.lines().skip(1){
        let values:Vec<&str>=line.split(',').collect();
        let mode:usize=values[0].parse().unwrap();
        let steps:usize=values[1].parse().unwrap();
        if steps>20 {continue}
        let numbers:Vec<f64>=values[2..].iter().map(|v|v.parse().unwrap()).collect();
        let s=&mut worlds[mode];
        for _ in completed[mode]..steps{s.step(0.003)}
        completed[mode]=steps;
        near(s.sim_time,numbers[0],1e-10,1e-10);
        near(s.mass(),numbers[1],1e-12,1e-10);
        assert_eq!(s.alive_bonds(),numbers[2] as i32);
        assert_eq!(s.fractures,numbers[3] as i32);
        assert_eq!(s.contacts,numbers[4] as i32);
        near(s.thermal_energy(),numbers[5],1e-9,0.05);
        near(s.mechanical_energy(),numbers[6],1e-5,0.1);
        near(s.max_damage,numbers[7],1e-4,0.00005);
        near(s.max_plastic_strain(),numbers[8],1e-4,0.00005);
        near(s.nodes[0].p.y,numbers[9],1e-5,0.00005);
        near(s.nodes[1].p.x,numbers[10],1e-5,0.00005);
    }
}
