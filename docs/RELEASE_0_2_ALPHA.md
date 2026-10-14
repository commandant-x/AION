# AION CORE 0.2-alpha — Mechanics laboratory

## What is delivered

- `bin/AION_CORE_0_2.exe`: **real Windows x64 graphical executable** built in freestanding C from `windows_viewer/aion_win32.c` with Windows GDI. 3D *projected* spring/particle lattice, gravity, contact impulses, breakable links, thermal conversion and interactive camera.
- `bin/AION_CORE_0_2_TEST.exe`: Windows x64 console executable built from the same C physics reference, with automatic invariants and deterministic replay checks. **Not executed in this Linux-only environment**.
- `crates/aion-core`: Rust core physics + broad phase for 3D sphere collisions, breakable elastic springs, heat exchange, and earlier chemistry / electrostatics / waves / toy policy learning. **Rust code is not linked into the graphical `.exe`**.
- `verification/mechanics_invariants.c`: four extra physics tests compiled and executed on Linux.
- `reference/test_reference.py`: eight independent Python tests of mathematical reference equations.
- Cross-platform build and CI scripts, with a Windows workflow for eventual compilation and actual execution of native tests.

## Native viewer controls

| Input | Action |
|---|---|
| Drag inside viewport | Orbit the camera |
| Mouse wheel | Zoom |
| I | Add downward impulse to the impactor sphere |
| H | Heat the left half of the lattice by 25 K |
| 1 / 2 / 3 | Change material profile and reset world |
| R | Reset experiment |
| Space | Pause or resume |
| Plus / minus | Increase / decrease time scale |

The button shortcuts for **I**, **H**, **R** also exist in the left sidebar.

## Validation actually done in the build environment

- `file` / PE/COFF inspection verifies x64 GUI and console executables and Windows `KERNEL32`, `USER32`, `GDI32` imports.
- Linux executable of the same C mechanical model: stress (2,000 steps), material heat injection, contacts, fractures and mass invariance: **PASS**.
- Long-run stress on three material profiles, each 80,000 steps (240 simulated seconds): **PASS**.
- Four separate Linux C tests: heat exchange, contact momentum/energy, spring fracture energy and deterministic replay: **PASS**.
- Eight independent Python tests: **PASS**.

## Validation not done

- No access to Windows / Wine; `.exe` files were generated and statically inspected but **not run** on Windows. User-side smoke test required.
- Rust compiler/Cargo unavailable: Rust core could not be compiled or tested here, and its numeric implementation has not been matched against the C viewer step by step.
- Full 3D GPU renderer, volumetric fracturing, arbitrary rigid meshes, joints, fluid dynamics, and material chemistry are **not delivered**. This is a research mechanics preview, not an achieved 0.2 milestone acceptance gate.

## Physics limitations

Spheres joined by linear springs with force-threshold fracture and pairwise impulses are only an illustrative model. Ground friction and heat generation are approximate. The contact algorithm in the visual C viewer is O(N²) and is limited to dozens of particles. The Rust spatial broad phase allows larger sets but is not yet coupled to the GUI. Temperature is internally parameterized in kelvin. No quantum chemistry, real atomic bonds, first-person agents, survival or learned invention at this stage.

## Next formal gate

1. Compile and test native Rust library on Windows with stable Rust; expose C ABI (`cdylib`) or a stable inter-process simulation protocol.
2. Remove duplicated physics ownership (C viewer should render Rust-authoritative trajectories instead of maintaining its own solver).
3. Measure explicit solver energy drift, contact precision and broad-phase scaling on larger scenes, with fixed deterministic seeds.
4. Introduce an OpenGL/D3D12/wgpu renderer in the graphical executable, and provide measured CPU/GPU profiles.
5. Publish a Windows `.exe` with each future release, accompanied by compiled source, tests and a reproducibility log.
