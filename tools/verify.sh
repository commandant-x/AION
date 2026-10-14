#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
python3 -m pytest reference/test_reference.py -q
if command -v cargo >/dev/null 2>&1; then
  cargo test --workspace
  cargo run -p aion-lab -- --seed 42026 --steps 1000 > /tmp/aion_lab_42026.csv
  echo 'Native Rust tests and deterministic CSV smoke run done.'
else
  echo 'Rust toolchain unavailable: Python reference checks only. Native Rust build unverified.' >&2
fi
