$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw 'Rust/Cargo absent. Install the current stable toolchain from https://rustup.rs/ and restart PowerShell.'
}
cargo test --workspace
if ($LASTEXITCODE -ne 0) { throw 'Rust tests failed.' }
cargo run -p aion-lab -- --seed 42026 --steps 5000 | Set-Content -Encoding utf8 'aion_lab.csv'
if ($LASTEXITCODE -ne 0) { throw 'Rust experiment failed.' }
Write-Host 'AION CORE: native checks succeeded; results saved to aion_lab.csv.'
