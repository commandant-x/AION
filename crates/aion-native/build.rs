//! Compiles exactly the native/aion_sim.c source used by the Windows viewer.
//! No third-party build dependencies are required.
use std::{env, path::PathBuf, process::Command};
fn command(mut cmd: Command, description: &str) {
    let status = cmd.status().unwrap_or_else(|e| panic!("{description}: {e}"));
    assert!(status.success(), "{description} exited with {status}");
}
fn main() {
    let target = env::var("TARGET").unwrap();
    let workspace = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let file = workspace.join("native/aion_sim.c");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj = out.join(if target.contains("msvc") { "aion_sim.obj" } else { "aion_sim.o" });
    println!("cargo:rerun-if-changed={}", file.display());
    println!("cargo:rerun-if-changed={}", workspace.join("native/aion_sim.h").display());
    if target.contains("msvc") {
        let mut cc=Command::new(env::var("CC").unwrap_or_else(|_| "cl".into()));
        cc.arg("/nologo").arg("/O2").arg("/c").arg(&file).arg(format!("/Fo{}",obj.display()));
        command(cc,"MSVC C compilation");
        let lib=out.join("aion_sim.lib");
        let mut librarian=Command::new(env::var("AR").unwrap_or_else(|_| "lib".into()));
        librarian.arg("/nologo").arg(format!("/OUT:{}",lib.display())).arg(&obj);
        command(librarian,"MSVC library creation");
    } else {
        let mut cc=Command::new(env::var("CC").unwrap_or_else(|_| "cc".into()));
        cc.arg("-std=c99").arg("-O2").arg("-fno-math-errno").arg("-c").arg(&file).arg("-o").arg(&obj);
        command(cc,"native C compilation");
        let lib=out.join("libaion_sim.a");
        let mut ar=Command::new(env::var("AR").unwrap_or_else(|_| "ar".into()));
        ar.arg("crs").arg(&lib).arg(&obj);
        command(ar,"native archive creation");
    }
    println!("cargo:rustc-link-search=native={}",out.display());
    println!("cargo:rustc-link-lib=static=aion_sim");
}
