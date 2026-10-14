# CORE 0.6 — validation ledger

## Executed locally (Linux, October 2026)

- `bash tools/test_native.sh`: PASS.
- C physics reference: **99** assertions (determinism over 10,000 steps, mass, energy, thermal and contacts).
- C materials: **27** assertions (plasticity, damage, damping; material test is invoked twice by the inherited script).
- Python scientific reference: **8/8** tests.
- New C checkpoint tests: **29** assertions (round-trip, deterministic continuation, branching, checksum, size, invalid mass and edge references).
- C reference stress: **three 15,000-step cases**, one per material profile.
- Clang `-fsanitize=address,undefined` on checkpoint suite: **PASS**.
- Windows GUI and Windows diagnostic cross-compiled into x86-64 PE files using `clang` + `lld-link`; PE formats verified using `file`.

## Not executed / not proven

- No real Windows instance or Wine runner: Win32 `.exe` runtime/graphics and file IO unverified on Windows.
- No `rustc`/`cargo` in this environment: new/existing pure Rust crate was not compiled, tested or linked into these `.exe` files.
- No atomistic, chemical or machine-learning claims follow from the mechanics reference model.

## Important constraints

The checkpoint record is a **version- and ABI-bound binary laboratory record** (`AIONCK06`, version 6, little-endian header, fixed-ABI binary payload, FNV-1a integrity). Its stability is only established on the C Linux test binary used here and is not guaranteed across different toolchains.

The checkpoint decoder is single-thread-oriented (static staging memory), performs bounds and key scalar validation, and only commits state after verifying the format. FNV is noncryptographic; never load untrusted files.

The Windows diagnostic exercises in-memory checkpoint codec, while the Windows GUI uses OS file APIs. The file-IO controls require a Windows smoke test.
