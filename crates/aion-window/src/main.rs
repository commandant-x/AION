#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod windows_gui;

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|s| s == "--self-test") {
        let mut s = aion_rust_sim::AionSim::laboratory(0);
        let mass = s.mass();
        for _ in 0..120 { s.step(1.0 / 120.0); }
        assert!((s.mass() - mass).abs() < 1e-7, "mass conservation failed");
        assert!(s.sim_time > 0.0 && s.node_count > 0);
        let data = s.encode_v7().expect("serialize physical state");
        let loaded = aion_rust_sim::AionSim::decode_v7(&data).expect("deserialize physical state");
        assert_eq!(s.checksum(), loaded.checksum(), "AION7 roundtrip failed");
        std::process::exit(0);
    }
    windows_gui::run(args.iter().any(|s| s == "--window-smoke"));
}

#[cfg(not(windows))]
fn main() {
    eprintln!("AION CORE Visualizer: the graphical executable targets Windows x64. Use the core simulation on Linux.");
}