# CORE 0.8 — Reproducible experiment benchmark

- Added `native/experiment.{h,c}`: independent, deterministic 9-run materials suite.
- Added canonical CSV serializer independent of C standard library, compatible with freestanding Windows builds.
- Added `tests/experiment_08.c` with 840 benchmark assertions.
- Added `tests/validate_experiments_08.py` for independent CSV structural and physical validity checks.
- Added `B` key in native Windows viewer to export experiments without disturbing the live scene.
- Extended native Windows console selftest to exercise benchmark module and verify noninterference.
- Updated Windows Clang/LLD build scripts and GitHub Actions C-reference CI.
- Retained AION7 portable saves and all previous physical tests.
- **Explicitly not completed**: Rust compilation, Rust GUI integration, GPU renderer, real Windows GUI execution.
