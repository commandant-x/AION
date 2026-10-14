use aion_world::World;
fn main() {
    let args:Vec<String>=std::env::args().collect();
    if args.len()!=5 {
        eprintln!("usage: aion-world-export <seed-u64> <chunk-x> <chunk-z> <output.obj>");
        std::process::exit(2);
    }
    let seed=args[1].parse::<u64>().expect("invalid seed");
    let x=args[2].parse::<i32>().expect("invalid chunk X");
    let z=args[3].parse::<i32>().expect("invalid chunk Z");
    let chunk=World::new(seed).chunk(x,z);
    std::fs::write(&args[4],chunk.to_obj()).expect("unable to write OBJ file");
    println!("PRIMORDIA chunk ({x},{z}) seed {seed}: {} vertices",chunk.heights.len());
}