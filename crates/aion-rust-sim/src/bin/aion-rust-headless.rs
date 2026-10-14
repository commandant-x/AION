//! Native Rust experimental runner. No C physics kernel is linked to this binary.
//! This build must be executed by CI; no Rust compiler exists in the authoring sandbox.
use aion_rust_sim::AionSim;
use std::{env, fs, process};

/// Rust-owned material benchmark: three synthetic materials x three protocols.
/// 11 samples per run (at steps 0,100,...,1000); matches AION 0.8 protocol.
/// Cross-language checks compare physical quantities with tolerances, not FNV hashes.
fn benchmark_csv() -> String {
    let mut out=String::from("material,protocol,step,time_s,mass_kg,alive_bonds,fractures,plastic_events,thermal_j,mechanical_j,plastic_strain,max_damage,checksum\n");
    for material in 0..3 {
        for protocol in 0..3 {
            let mut sim=AionSim::laboratory(material);
            for step in 0..=1000 {
                if step!=0 {
                    if protocol != 0 && step % 180 == 0 { sim.pull(); }
                    if protocol == 2 && step % 250 == 0 { sim.heat(); }
                    sim.step(.003);
                }
                if step % 100 == 0 {
                    out.push_str(&format!(
                        "{material},{protocol},{step},{:.6},{:.6},{},{},{},{:.6},{:.6},{:.6},{:.6},{:016x}\n",
                        sim.sim_time,sim.mass(),sim.alive_bonds(),sim.fractures,
                        sim.plastic_events,sim.thermal_energy(),sim.mechanical_energy(),
                        sim.max_plastic_strain(),sim.max_damage,sim.checksum()
                    ));
                }
            }
        }
    }
    out
}


fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--self-test") => {
            let mut a=AionSim::laboratory(1);
            let starting_mass=a.mass();
            for _ in 0..350 {a.step(.003)}
            let checkpoint=a.encode_v7().expect("canonical checkpoint must encode");
            let mut b=AionSim::decode_v7(&checkpoint).expect("canonical checkpoint must decode");
            assert_eq!(b.encode_v7().unwrap(),checkpoint);
            assert_eq!(a.checksum(),b.checksum());
            assert_eq!(a.mass(),starting_mass);
            for _ in 0..300 {a.step(.003);b.step(.003)}
            assert_eq!(a.checksum(),b.checksum());
            println!("AION Rust physics self-test PASS / nodes={} bonds={} / t={:.3}s / checkpoint={} bytes",a.node_count,a.edge_count,a.sim_time,checkpoint.len());
        }
        Some("--benchmark") if args.len() == 3 => {
            let csv=benchmark_csv();
            fs::write(&args[2],csv).expect("benchmark write failed");
            println!("AION pure Rust benchmark: 9 scenarios / 99 samples -> {}",args[2]);
        }
        Some("--save") if args.len() == 3 => {
            let mut world=AionSim::laboratory(0);
            for _ in 0..1000 {world.step(.003)}
            let blob=world.encode_v7().expect("world must serialize");
            fs::write(&args[2],&blob).expect("checkpoint write failed");
            println!("Saved {} bytes of AION7 portable state",blob.len());
        }
        Some("--resume") if args.len() == 3 => {
            let bytes=fs::read(&args[2]).expect("checkpoint read failed");
            let mut world=AionSim::decode_v7(&bytes).expect("invalid or incompatible checkpoint");
            for _ in 0..1000 {world.step(.003)}
            println!("Resumed Rust world to t={:.3}s / checksum={:016x}",world.sim_time,world.checksum());
        }
        _ => {
            eprintln!("AION Rust headless: --self-test | --benchmark <file.csv> | --save <file.aion7> | --resume <file.aion7>");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::benchmark_csv;
    #[test]
    fn benchmark_has_99_complete_samples_and_conserves_mass() {
        let csv=benchmark_csv();
        let mut lines=csv.lines();
        let header=lines.next().unwrap();
        assert_eq!(header.split(',').count(),13);
        let mut seen=[0usize;9];
        for line in lines {
            let parts:Vec<&str>=line.split(',').collect();
            assert_eq!(parts.len(),13);
            let material:usize=parts[0].parse().unwrap();
            let protocol:usize=parts[1].parse().unwrap();
            let step:usize=parts[2].parse().unwrap();
            let mass:f64=parts[4].parse().unwrap();
            let intact:i32=parts[5].parse().unwrap();
            let fractures:i32=parts[6].parse().unwrap();
            assert!(material<3 && protocol<3);
            assert_eq!(step,seen[material*3+protocol]*100);
            assert_eq!(intact+fractures,404);
            let reference_mass=if material==1 { 57.8 } else if material==2 { 111.8 } else { 97.4 };
            assert!((mass-reference_mass).abs()<1e-5);
            seen[material*3+protocol]+=1;
        }
        assert!(seen.iter().all(|count|*count==11));
    }
}
