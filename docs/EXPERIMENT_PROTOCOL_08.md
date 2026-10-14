# AION 0.8 — standardized reference experiments

## Experimental question

How do three parameterized coarse-grained material networks react to a reproducible loading schedule? These are **engine regression scenarios**, not calibrated predictions for natural rock, wood, or glass.

## Initial conditions

`aion_reset(mode)` initializes **73 particle nodes and 404 bonds**. Most particles form a bonded three-dimensional lattice; the last particle is a falling impact body. The simulation has gravity and a planar floor. No random numbers are sampled in this benchmark.

## Independent variables

* Material `0`: stone-like; material `1`: fibrous; material `2`: brittle. They differ in spring stiffness, yield rate, damage rate and damping.
* Protocol `0`: simply simulate (gravity and the impact body are still active).
* Protocol `1`: at steps 180, 360, 540, 720 and 900, call `aion_pull`, which imparts an outward velocity to particles at the left/right boundaries.
* Protocol `2`: same tensile impulses, plus `aion_heat` at steps 250, 500, 750 and 1000, adding 25 Kelvin to a selected subset of particles per event. Heating is externally injected energy; it is not energy created spontaneously.

Each run uses 1000 calls to `aion_step(..., .003)` and records steps 0,100,...,1000. This corresponds to three simulated seconds with internal substeps of at most `.002`.

## Outputs

`material,protocol,step,time_s,mass_kg,alive_bonds,fractures,plastic_events,thermal_j,mechanical_j,plastic_strain,max_damage,checksum`

* `alive_bonds + fractures == 404` is checked for this protocol.
* `mass_kg` is conserved because no particles are created or destroyed in this experiment.
* `plastic_strain` measures maximum normalized change of bond rest length, **not** measured bulk tensile strain.
* `max_damage` is a scalar constitutive damage proxy, **not** a physical crack length.
* `mechanical_j` uses the current simplified energy diagnostic. It does not establish energy conservation of the integration scheme and must not be used as a certified thermodynamic balance.
* `thermal_j` includes absolute thermal state energy and explicit heating. It does not mean thermal energy was created by mechanical dissipation alone.
* `checksum` is FNV-1a over a selected subset of state fields. It is a replay fingerprint, not a cryptographic integrity proof.

## Acceptance criteria

1. 9 runs × 11 samples, canonical CSV header and complete rows.
2. All masses remain identical within each run; fracture events never reverse; counts remain in bounds.
3. Re-running a scenario with the same binary gives exactly the same C state fingerprints.
4. Each loading protocol changes the trajectory relative to the unforced protocol for a given material.
5. Invalid material/protocol arguments fail without an out-of-bounds access.
6. CSV serializer refuses insufficient buffer space; a separate Python implementation validates file syntax and physics invariants.
7. Win64 viewer and console self-test build from **one** reference solver file. The viewer's benchmark must not modify the user's live scene.
8. No claim that this reproduces physical properties of real stone, wood or glass without experimental calibration.

## Future C ↔ Rust parity protocol

With Rust toolchain available, duplicate `aion_bench_run` in the Rust-owned simulation, export the same columns and compare **physical quantities with stated numeric tolerances** and the event schedules. Do not require hash equality across different floating-point implementations. Review major divergence before replacing C. The final AION Rust executable must be built on Windows and run on a real Windows runner; release build provenance and tests must be retained.
