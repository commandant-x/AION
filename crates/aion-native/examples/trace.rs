use aion_native::Simulator;
fn main() {
    let mut world = Simulator::new(0);
    println!("AION CORE 0.3 — shared C physics through Rust FFI");
    for tick in 0..3000 {
        world.step(0.003);
        if tick % 500 == 0 {
            println!("t={:0.3}s particles={} broken={} contacts={} mass={:0.3}kg total={:0.1}J",
                world.state().sim_time,world.state().node_count,
                world.state().fractures,world.state().contacts,
                world.mass(),world.total_energy());
        }
    }
}
