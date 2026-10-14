//! Standalone research CLI: writes deterministic trajectories as CSV.
use aion_core::math::Vec3;
use aion_core::physics::{Bond,Particle,World,STONE,WOOD};
use aion_core::brain::Brain;
use aion_core::rng::Rng;
use std::env;

fn main(){
    let args:Vec<String>=env::args().collect();
    let seed=args.windows(2).find(|x|x[0]=="--seed").map(|x|x[1].parse::<u64>().expect("seed must be u64")).unwrap_or(42);
    let steps=args.windows(2).find(|x|x[0]=="--steps").map(|x|x[1].parse::<usize>().expect("steps must be usize")).unwrap_or(1500).min(200_000);
    let mut rng=Rng::new(seed);
    let mut world=World::new();
    let a=world.add(Particle::sphere(Vec3::new(0.,2.1,0.),0.08,STONE,350.));
    let b=world.add(Particle::sphere(Vec3::new(0.28,2.1,0.),0.08,WOOD,280.));
    world.bond(Bond::new(a,b,0.28,100.,80.,12.));
    let mut brain=Brain::<4,5>::blank(&mut rng,0.015);
    println!("step,time_s,body_a_y_m,body_b_y_m,temperature_a_k,temperature_b_k,total_mass_kg,energy_j,bonds_alive,policy_action0");
    for step in 0..steps {
        world.step(0.01);
        let obs=[(world.particles[0].temp/400.).min(1.),(world.particles[0].pos.y/2.).min(1.),0.,1.];
        let action=brain.select(obs,&mut rng);
        let reward=if action==0 {0.05} else {0.};
        brain.update(obs,action,reward,0.01);
        if step%10==0 {println!("{},{:0.5},{:0.8},{:0.8},{:0.6},{:0.6},{:0.8},{:0.4},{},{:0.5}",step,world.time,world.particles[0].pos.y,world.particles[1].pos.y,world.particles[0].temp,world.particles[1].temp,world.total_mass(),world.total_energy(),world.bonds.iter().filter(|b|b.alive).count(),brain.probabilities(obs)[0]);}
    }
    eprintln!("AION LAB seed={seed} steps={steps} completed");
}
