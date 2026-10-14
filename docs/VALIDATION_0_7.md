# AION CORE 0.7 — Validation ledger

Status: **PARTIAL**. Canonical C-reference checkpoint milestone finished, Rust migration **NOT finished**.

## Actually executed (Linux authoring host)

- `bash tools/test_native.sh`: success.
- Native C mechanics: **99 assertions** passed, including deterministic 10,000-step replay and mass checks.
- C materials: **27 unique assertions** passed (called twice by inherited script).
- C 0.6 checkpoint regression: **29 assertions** passed.
- New portable C codec: **66 assertions** passed (canonical binary roundtrip, multiple material presets, replay, branches, corruption, truncation, invalid physically impossible state, no mutation on failure).
- Independent Python AION7 reader: **4 unittest cases** passed, including C-generated fixture, modified bytes, truncated files and exact payload size.
- Scientific reference Python: **8/8** pytest cases passed.
- Stress: **three independent 15,000-step material simulations** (45,000 simulated steps total).
- C portable tests compiled with `-fsanitize=address,undefined`: passed.
- Clang + LLD produced two x86-64 PE32+ Windows executables, a GUI and console diagnostic, linked to C reference physics and canonical C checkpoint codec.
- `tests/fixtures/sample_v7.aion7` is a binary output of the C codec; read by independent Python parser.

## Not executed or certified

- Rust codec + Rust physics compilation and execution: **NOT EXECUTED** (no `rustc` / `cargo` installed; external toolchain downloads unavailable).
- Rust headless Windows `aion-rust-headless.exe`: **NOT GENERATED HERE**. GitHub Actions workflow to build/test on a Windows runner is in the sources; a workflow definition is not a completed run.
- Rust-physics-based Windows GUI: **NOT BUILT**. The shipped Windows GUI uses the existing **C** physics solver.
- Real Windows smoke test of the GUI and file I/O: **NOT RUN**. PE target inspection is not a runtime test.
- Cross-CPU exact replay: **NOT PROVEN**, even though byte-level checkpoint portability is tested.
- Concurrent use of `aion7_read`: **NOT SAFE** in C because it uses static staging memory. Only use on one UI/thread until rewritten with a caller-provided workspace or Rust-owned state.
- The checksum is FNV-1a for error detection, not cryptographic authentication.

## Scope and scientific integrity

The lab has 73 coarse-grained nodes and 404 elastic/damage bonds initially. This is **not atomistic chemistry, quantum field physics, biology or a world with real agents**. The code is designed to support reproducible experiments on a path toward those larger goals.

## Rebuild

`bash tools/test_native.sh && bash tools/build_windows_from_linux.sh`

Expected Windows x64 outputs: `bin/AION_CORE_0_7_C_REFERENCE.exe` and `bin/AION_CORE_0_7_C_REFERENCE_TEST.exe`.
