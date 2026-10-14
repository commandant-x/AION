# AION7 — Portable physics checkpoint specification

This is a compact, canonical, **experimental** binary format for coarse-grained material simulations, not atoms and not a stable public game save format.

## Binary layout

All integer fields are 32-bit little-endian signed except header sizes/version (unsigned). All real values are IEEE-754 binary64 little-endian. There is no alignment padding. Bytes 0-31 are the 32-byte header:

| Bytes | Contents |
|---|---|
| 0–7 | ASCII `AIONS007` |
| 8–11 | Unsigned version = 7 |
| 12–15 | Unsigned total record length |
| 16–23 | FNV-1a 64-bit of all payload bytes starting at offset 32, seed `1469598103934665603`, multiplier `1099511628211` |
| 24–27 | Byte-order marker 0x01020304 |
| 28–31 | Reserved zeros |

Next is a 100-byte fixed section: nine `i32` values (node_count, edge_count, fractures, contacts, material_mode, rejected_steps, plastic_events, fatigue_fractures, floor_enabled), followed by eight `f64` values (sim_time, gravity, max_strain, max_contact_impulse, plastic_heat, damage_heat, damping_heat, max_damage).

For each active node: **88 bytes**: 10 `f64` values (`position.xyz`, `velocity.xyz`, radius, mass, temperature, heat_capacity), then 2 `i32` values (kind, component).

For each active edge: **100 bytes**: 3 `i32` values (a, b, alive), followed by 11 `f64` values (rest, stiffness, rupture_limit, conductance, original_rest, yield_strain, plastic_rate, damage, damage_onset, damage_rate, dashpot).

Exact total length is `132 + node_count * 88 + edge_count * 100`. Maximum counts are 128 nodes and 1024 edges. No inactive entries or unused structure bytes are serialized.

## Validation

The parser rejects unexpected versions, truncated/oversized records, invalid lengths, bad hashes, nonfinite scalars, out-of-range indices, negative mass or heat capacity and malformed edge definitions, *before altering the destination state*. A noncryptographic FNV checksum only detects accidental corruption; it is **not an authenticity guarantee**. In the freestanding C reference GUI, the decoder uses one static scratch buffer and is not thread-safe; calls must be serialized. The Rust candidate allocates the temporary state per decoder call.

## Cross-language fixture

`tests/fixtures/sample_v7.aion7` is generated from the C reference with material mode 1 and 350 steps of 0.003 simulated seconds. It is decoded by the independent Python reader and is used as a golden fixture by the proposed Rust codec. Rust compilation and execution are **pending**.

The legacy 0.6 raw ABI-bound file `AIONCK06` is still readable using `aion_checkpoint_read` in C but not automatically loaded by the 0.7 GUI. This deliberate separation prevents accidentally misinterpreting an old raw-memory dump as portable data. Explicit migration tooling can be added after a real Windows test.

For deterministic replay, use the same floating-point configuration and solver version. Canonical checkpoint files are portable between architectures that implement the required types, but **bitwise-identical physics across different CPU instruction sets is not guaranteed**.
