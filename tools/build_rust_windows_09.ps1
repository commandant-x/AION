# AION CORE 0.9: builds and executes the PURE Rust physics engine on Windows.
# Prerequisite: Rust installed from https://rustup.rs/ (stable MSVC toolchain).
# No installer is downloaded or executed by this script.
$ErrorActionPreference = 'Stop'
Set-Location (Resolve-Path (Join-Path $PSScriptRoot '..'))
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw 'Rust/Cargo not found. Install Rust stable MSVC via rustup.rs, reopen PowerShell, rerun.'
}
function Check-Exit([string] $step) {
  if ($LASTEXITCODE -ne 0) { throw "$step failed (exit code $LASTEXITCODE)" }
}
Write-Host 'AION CORE 0.9: REAL RUST BUILD' -ForegroundColor Cyan
rustc --version; Check-Exit 'rustc version'
cargo test --workspace --all-targets; Check-Exit 'Rust workspace tests'
cargo build --release -p aion-rust-sim --bin aion-rust-headless; Check-Exit 'Rust Windows compilation'
$exe = 'target\release\aion-rust-headless.exe'
& $exe --self-test; Check-Exit 'Rust physics self-test'
& $exe --resume tests\fixtures\sample_v7.aion7; Check-Exit 'Read C fixture from Rust'
& $exe --save aion09-test.aion7; Check-Exit 'Rust checkpoint write'
& $exe --resume aion09-test.aion7; Check-Exit 'Rust checkpoint replay'
& $exe --benchmark aion09-rust.csv; Check-Exit 'Rust benchmark'
python tests\validate_rust_benchmark_09.py aion09-rust.csv experiments\AION_CORE_0_8_reference.csv
Check-Exit 'Compare Rust / C reference invariants'
New-Item -ItemType Directory -Force -Path dist | Out-Null
Copy-Item $exe dist\AION_CORE_0_9_RUST_WINDOWS_X64.exe -Force
Write-Host 'PASS: dist\AION_CORE_0_9_RUST_WINDOWS_X64.exe' -ForegroundColor Green
