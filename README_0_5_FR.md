# AION PRIMORDIA CORE 0.5 — RUST MIGRATION CANDIDATE

**Statut : migration en cours ; PAS encore un solveur Rust livré sous Windows.**

## Objectif
Remplacer le noyau mécanique C par un portage **entièrement Rust**, tout en
préservant l'ABI `aion_sim.h` et le visualiseur Win32 existant.

## Contenu
- `crates/aion-rust-sim/src/lib.rs` : solveur Rust indépendant, sans appel à C,
  intégrant les ressorts, plasticité, fatigue, propagation thermique, contacts
  entre fragments, plancher et instrumentation.
- `crates/aion-rust-sim/tests/reference_parity.rs` : contrôle du portage contre
  les traces de référence C, sur les trois matériaux.
- `tests/reference_trace_05.c` : générateur de données comparatives.
- `native/aion_sim.c` : ancienne référence C conservée pour comparaison.
- `windows_viewer/` : laboratoire Win32/GDI de la 0.4, avec réétiquetage clair
  0.5 C preview ; **cet exécutable de secours exécute encore C**.
- `.github/workflows/rust-physics-05.yml` : objectifs de CI Linux / Windows.
- `tools/build_rust_windows.ps1` : script proposé de liaison Rust vers Windows.
- `docs/VALIDATION_0_5.md` : tests réellement effectués et blocages.

## Exécuter les tests existants sur Linux
```sh
./tools/test_native.sh
cargo test -p aion-rust-sim --all-targets
cargo test --workspace --all-targets
```

## Créer le véritable exécutable avec le solveur Rust (Windows)
En terminal PowerShell Developer VS disposant de Rust, MSVC et Clang :
```powershell
.\tools\build_rust_windows.ps1
.\bin\AION_CORE_0_5_RUST_TEST.exe
.\bin\AION_CORE_0_5_RUST.exe
```
Le script de compilation Rust/Windows est **non validé** dans l'environnement
actuel, dépourvu de `rustc` et `cargo`. En cas de problème de linker, corriger
sur Windows et documenter le correctif avant publication.

## Livrables disponibles tout de suite
`AION_CORE_0_5_C_PREVIEW.exe` et le programme `_TEST.exe` sont deux
applications Windows x64 compilées ici à partir de l'ancien noyau C.
Elles permettent de continuer les vérifications sous Windows, mais **ne
constituent pas la preuve de la migration Rust**.

## État du goal
Voir `GOAL.md`, `docs/VALIDATION_0_5.md` et `docs/GOAL_PROGRESS.md`.
Ne pas appeler cette version « Rust 0.5 stable » avant les tests CI.
