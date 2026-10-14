# AION CORE 0.8 — Verification record

## Environment

Linux container, GCC and Clang with LLD cross-linker; Rust stable compiler absent and not downloadable from current environment. Windows x64 PE binaries built with `clang --target=x86_64-pc-windows-msvc` and `lld-link`. No Wine or Windows runtime in container.

## Executed

- `bash tools/test_native.sh`: legacy C mechanics 99 assertions, materials 27 assertions (two runs), checkpoint 0.6 29 assertions, portable AION7 format 66 assertions, Python scientific reference 8 tests, Win32 self-test through Linux host shim PASS, 3×15,000-step C stress simulations PASS.
- New `experiment_08.c`: **840 C checks** PASS, 9 protocols x 11 samples, CSV 10,158 bytes.
- `tests/validate_experiments_08.py`: parsed 99 CSV records and verified mass, event count, timing, damage bounds and checksums; PASS.
- Separate Clang build of the experiment runner: 840 checks PASS; CSV checksum identical to GCC build.
- `-fsanitize=undefined -fno-sanitize-recover=all` on experiment runner: 840 checks PASS.
- `tools/build_windows_from_linux.sh`: Win64 GUI and Win64 console self-test generated. `file` identifies both as PE32+ Windows x86-64.

## Not executed / not proven

- Windows GUI/keyboard actions have not been executed on a real Windows host.
- Rust crate compilation, Rust tests, Rust FFI numerical comparison and Rust native Windows executable are blocked.
- Physical models are synthetic coarse-grained networks, not calibrated materials. Renderer remains GDI projected 3D, not GPU 3D.
- CSV printed values are rounded to six decimal places. Fingerprints are reproducible under the tested compilers but are not cryptographic or cross-platform guarantees.
