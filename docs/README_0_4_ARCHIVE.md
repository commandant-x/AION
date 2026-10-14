# AION / PRIMORDIA CORE 0.4 — DEFORMATION & FATIGUE

A **native Windows mechanics research laboratory**, backed by one freestanding C99
mechanics kernel shared by its visualizer, Windows diagnostic, Linux tests and a
Rust FFI crate. This is *not* yet a fully Rust physics engine, GPU renderer or
atomic/quantum chemistry simulator. The materials are stylized spring-lattice
models, not laboratory-calibrated stone, wood or metal.

## Run on Windows

- `bin/AION_CORE_0_4.exe`: native Windows 64-bit GUI (Win32/GDI 3D projection).
- `bin/AION_CORE_0_4_TEST.exe`: native Windows 64-bit diagnostic; code 0 = passed.

### Controls

| Action | Input |
|---|---|
| Orbit camera / zoom | Left mouse drag / mouse wheel |
| Impact test | `I` |
| Tensile impulse (new) | `T` |
| Heat specimen | `H` |
| Material profiles | `1`, `2`, `3` |
| Pause / resume | `Space` |
| Time scale | `+`, `-` |
| Reset specimen | `R` |

The viewer colors undamaged springs cyan, plastically elongated springs violet,
and damage-affected springs amber/red. The telemetry shows fracture count,
maximum strain and permanent extension. These are qualitative visual diagnostics.

## New physics experiment

- **Irreversible plastic rest-length change** above a strain threshold.
- **Accumulating scalar bond damage**, reducing effective stiffness and leading
  eventually to fracture even below immediate-force threshold.
- **Kelvin-type axial damping**, dissipating kinetic energy into sample heat.
- **Energy bookkeeping** for fracture, plastic adjustment and fatigue release;
  explicit semi-implicit integration still introduces measurable numerical error.
- **Tension test** to apply an opposite signed impulse to the outer sides of
  the sample. All three profiles respond differently.
- A deterministic signature now covers mutable bond lengths and damage.
- **No invention recipes**, no predefined technological progression.

**Physical limit:** this lattice is not an FEM solver, molecular dynamics,
verified continuum mechanics or an experimentally calibrated fracture law.
In particular, yield/damage constants are chosen for demonstrations and must
not be used to predict properties of real minerals.

## Native build and tests

### Linux cross-build to Windows (clang + lld-link)

```bash
bash tools/build_windows_from_linux.sh
bash tools/test_native.sh
```

### On Windows (PowerShell, LLVM/clang and lld-link)

```powershell
./tools/build_windows.ps1
./bin/AION_CORE_0_4_TEST.exe
# Rust optional, after installing Rust stable and MSVC Build Tools:
cargo test --workspace
```

The Rust crate `crates/aion-native` declares the same ABI as the C engine,
builds the C core and performs ABI-size checks before exposing simulator
state. Rust compilation could **not** be executed in the release environment
(no `cargo` or `rustc`); the Windows executables are C-compiled. The older
Rust prototype in `crates/aion-core` is not used by the Windows executable.
The authoritative Rust rewrite remains an explicit goal, not an accomplished task.

## Quality gates

- 99 assertions in original native mechanics test (10,000-step deterministic replay).
- 27 additional assertions for plasticity, non-healing fatigue, damping,
  conservation within numerical tolerance, and replay after tension.
- 3 material stress runs × 15,000 steps with finiteness/mass checks.
- 8 independent Python reference tests.
- Windows native GUI and diagnostic cross-built to PE32+ AMD64.
- Real execution on Windows **not available here**; selftest uses same sources on Linux.
- Rust `cargo test` **not executed here**; GitHub Actions workflow is provided.

See `GOAL.md` for persistent Codex Goal objective and release criteria, and
`docs/VALIDATION_0_4.md` for the evidence and blockers.
