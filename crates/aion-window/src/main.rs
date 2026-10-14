#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod windows_gui;

#[cfg(windows)]
fn diagnostics() -> i32 {
    let mut report = String::from("AION native Rust GUI self-test\n");
    let mut sim = aion_rust_sim::AionSim::laboratory(0);
    let mass = sim.mass();
    report.push_str(&format!("Initial: {} nodes, {} bonds, mass={mass}\n",sim.node_count,sim.edge_count));
    for _ in 0..120 { sim.step(1.0 / 120.0); }
    report.push_str(&format!("Time: {}, mass={}, temperature={}, energy={}\n",sim.sim_time,sim.mass(),sim.nodes[0].t,sim.total_energy()));
    let code = if (sim.mass()-mass).abs()>1e-7 { 11 }
        else if !(sim.sim_time>0.0 && sim.node_count>0){12}
        else if let Some(data) = sim.encode_v7() {
            if let Some(copy) = aion_rust_sim::AionSim::decode_v7(&data) {
                if sim.checksum()!=copy.checksum(){14}else{0}
            }else{13}
        }else{15};
    report.push_str(&format!("EXIT CODE: {code}\n"));
    let path=std::env::temp_dir().join("aion-gui-self-test.txt");
    let _=std::fs::write(&path, report);
    code
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|s| s == "--self-test") {
        std::process::exit(diagnostics());
    }
    windows_gui::run(args.iter().any(|s| s == "--window-smoke"));
}

#[cfg(not(windows))]
fn main() {
    eprintln!("AION CORE Visualizer: graphical executable targets Windows x64.");
}