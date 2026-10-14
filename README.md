# AION CORE 0.9 — Portage Rust : compilation contrôlée

> **État validé (14 octobre 2026)** : le moteur `aion-rust-sim` et son exécutable console `aion-rust-headless.exe` ont été compilés et exécutés dans GitHub Actions sous Windows et Linux. Tests, sauvegarde AION7 et comparaison d'expériences : [voir l'exécution réussie](https://github.com/commandant-x/AION/actions/runs/38558593186). Le visualiseur graphique historique reste basé sur C ; aucun moteur GPU 3D Rust n'est encore livré.


> État : code Rust candidat, **non compilé dans ce conteneur**. Deux `.exe` C de référence sont fournis. Pour générer et tester le véritable exécutable Rust, suivre [docs/INSTALL_GITHUB_CI_FR.md](docs/INSTALL_GITHUB_CI_FR.md).

# AION / PRIMORDIA CORE 0.8 — REPRODUCIBLE EXPERIMENTS

**Delivery status: C reference application; Rust-owned Windows engine NOT validated.**

This is a freestanding Win32 laboratory built around the same *C reference physics solver* that runs in Linux unit tests. The new 0.8 feature is a **deterministic material benchmark** with independent CSV validation. No HTML, game recipe system, neural civilization, or atomic-scale claims are involved.

## Windows: quick start

1. Open `bin/AION_CORE_0_8_C_REFERENCE.exe` (Windows x64).
2. Press **B**. The app runs nine experiments and saves `AION_CORE_0_8_experiments.csv` in its **working directory** (typically the directory from which you launched the app).
3. Compare `fractures`, `plastic_strain`, `max_damage`, mechanical and thermal indicators between materials and loading conditions. See `docs/EXPERIMENT_PROTOCOL_08.md` before drawing physical conclusions.
4. Press **S** to save, **L** to restore, **I** for impact, **T** for tensile impulse, **H** to heat, `1`/`2`/`3` to select material, `Space` to pause, **R** to reset.
5. Launch `bin/AION_CORE_0_8_C_REFERENCE_TEST.exe` from a console for a diagnostic (the process exits with code 0 if successful).

**Windows executable limitation:** generated and PE32+ x86-64 format validated on Linux; **not run in Windows** in this environment. First execution may reveal platform problems; report them.

## Experiments

* Materials: **0** stone-like, **1** fibrous, **2** brittle (experimental parameters, not calibrated real materials).
* Protocols: **0** no extra loading, **1** repeated tensile impulses, **2** repeated impulses + incremental heating.
* Every run begins from a freshly reset state. Same model, timestep (`0.003` simulated seconds), duration (1000 steps = 3 seconds) and fixed intervention schedule.
* 11 measurements per run, yielding **99 samples** across 9 experiments.
* The experiment runner uses its own scratch simulation and does **not** alter the player's current laboratory state.
* CSV has a stable explicit header and CRLF line endings. Hashes identify bitwise C-state reproducibility on a given build/platform; cross-compiler float bitwise parity is not guaranteed.

## Build and verify

On Linux with GCC/Clang + LLD:

```bash
bash tools/test_native.sh
bash tools/build_windows_from_linux.sh
file bin/AION_CORE_0_8_C_REFERENCE.exe
```

On Windows with Clang/LLD in `PATH`:

```powershell
.\tools\build_windows.ps1
.\bin\AION_CORE_0_8_C_REFERENCE_TEST.exe
```

Rust tests require the Rust toolchain and are configured in `.github/workflows` but **could not be run locally**. `cargo test --workspace --all-targets` is an acceptance requirement, not a verified result. The `aion-rust-sim` prototype and AION7 decoder are included as **uncompiled candidate source**.

## Long-term goal

The goal remains a Rust-owned multiscale physical world from which tool use, biology, language and culture might emerge without predefined recipes. See `GOAL.md` and `docs/ROADMAP_1_0.md`. The benchmark helps quantify whether future changes to physics are regressions or improvements; it is not evidence of successful emergence.
