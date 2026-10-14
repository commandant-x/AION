# PRIMORDIA 0.15 — Physique hydrothermique vérifiable

La priorité scientifique prend le pas sur l'ajout de fonctionnalités de jeu. Un nouveau sous-système **Rust à volumes finis** représente l'eau et les transferts thermiques au sein d'un petit volume de roche géologique. Chacune de ses cellules (2 × 2 × 2 m) porte une quantité d'eau [m³], une énergie sensible [J], une température [K], ainsi que des caractéristiques de porosité, conductivité hydraulique, succion et propriétés thermiques.

Les transferts sur les faces utilisent une fermeture **approchée** de Darcy et la conduction de Fourier. Les volumes et flux d'énergie sont transférés symétriquement entre cellules pour éviter la création ou destruction artificielle d'eau ou de chaleur. Les limites externes du laboratoire sont fermées, avec apports d'eau comptabilisés séparément. Des expériences indépendantes comparant argile et gravier, conduction, gravité et rejeu déterministe sont incluses.

**Essai numérique sans interface graphique** :
```powershell
.\aion-explore.exe --hydro-benchmark bilan.csv
python tests/validate_hydro_015.py bilan.csv
```
Le premier exécutable Windows demeure aussi un explorateur GPU lorsqu'il est ouvert normalement. La nouvelle expérience produit **61 états successifs** avec conservation, apports, températures et empreintes déterministes. Voir [documentation du modèle](crates/aion-explore/MODEL_015.md) et [tests officiels](https://github.com/commandant-x/AION/actions/workflows/aion-gpu-explorer.yml).

**Niveau de fidélité actuel : laboratoire numérique expérimental.** La roche est une classe synthétique non calibrée, le débit hydraulique en régime non saturé utilise une approximation linéaire de la succion, la chaleur latente et les réactions chimiques ne sont pas modélisées. Ce laboratoire ne met pas encore à jour l'ensemble de la planète ni la physique des creusements en temps réel. Aucune prétention à un modèle de nappe ou d'aquifère validé sur le terrain.

---

# PRIMORDIA 0.14 — Excavation volumique et premières cavités 3D

La version 0.14 contient un système de cellules volumétriques éparses de **2 × 2 × 2 m**, indépendantes des couches de surface. B permet de creuser **8 m³** de sol ou roche devant le joueur ; N permet de déposer un bloc de la matière sélectionnée dans l'espace libre adjacent. Les faces de cavité sont désormais rendues dans le maillage GPU et les blocs de surface retirés ouvrent le terrain.

Les systèmes Q/E de la version 0.13 (surface, 2 m³) fonctionnent toujours et **possèdent temporairement un stock distinct** de l'inventaire volumique B/N. Les touches 1–8 sélectionnent la matière ; P/L sauvegardent et rechargent les deux fichiers `.aion13` et `.aionv14`.

**Limite structurante :** le déplacement du personnage ne tient toujours pas compte des caves, le contrôleur suit la surface d'altitude. Les cavités sont réellement représentées en 3D dans la structure et le rendu, mais il n'est pas encore possible de les parcourir à pied. Ni résistance de toiture, ni fluides, ni tectonique calculée. La fenêtre Win32 a été testée en CI ; l'affichage GPU interactif sur le PC utilisateur reste à confirmer.

**Validation** : [tests Windows et Linux AION 0.14](https://github.com/commandant-x/AION/actions/runs/38600186620) — noyau voxels, mesh, tests, exécutable release et démarrage fenêtre.

---
# PRIMORDIA 0.13 — Première interaction avec la croûte terrestre

Le moteur Rust GPU permet maintenant de **creuser et remblayer le relief**. Il distingue huit types de matières (terre, sable, argile, gravier, tourbe, calcaire, granite et basalte), avec des strates liées aux champs géologiques, des densités et des duretés synthétiques. Les changements restent visibles lors du streaming du terrain. Un inventaire comptabilise les matériaux extraits et empêche la création de matière lors du dépôt.

**Commandes** : `Q` creuser 2 m³ (une couche de 0,5 m sur une maille de 2 m × 2 m) ; `1` à `8` choisir une matière ; `E` déposer une unité disponible ; `P` sauvegarder auprès du programme ; `L` charger ; `R` revenir au point de départ. Pour la caméra : clic gauche, WASD, Espace (saut), Shift (sprint), F (vol), Échap.

Le détail de la matière sélectionnée, le volume restant et le bilan des cellules éditées sont visibles dans le titre de la fenêtre.

**Limites** : il s'agit encore d'une **surface à couches stratifiées**, et non d'un monde volumique 3D avec grottes ou surplombs. L'énergie nécessaire pour casser la roche, l'érosion et les processus tectoniques ne sont pas encore calculés. La simulation géologique complète reste un objectif ouvert.

**Validation** : [GitHub Actions Windows/Linux](https://github.com/commandant-x/AION/actions/workflows/aion-gpu-explorer.yml), avec tests de conservation et tests de fenêtre.

---

# PRIMORDIA 0.12 — saut, géologie structurée et océan visible

**Nouveau :** saut (ESPACE), gravité et collisions kinematiques avec le sol ; haute résolution adaptative 2/4/8 m et génération asynchrone pour préserver la réactivité ; surface océanique translucide à zéro mètre et fond marin ; soulèvements, dureté de roche et vallées incisées par champs géologiques procéduraux.

**Commandes :** clic gauche pour activer la souris, WASD marcher, Espace sauter, Shift courir, F voler, Espace/CTRL monter/descendre en vol, R réinitialiser, Échap libérer la souris. Paramètre `--seed 42`.

**Limites :** la mer n'est pas un fluide simulé ; il n'y a pas encore de transport de sédiments, de tectonique réelle ni d'écologie. Les nouveaux champs sont procéduraux, pas une simulation physique complète. Le Goal final reste ouvert.

**Validation :** tests terrain/locomotion et compilation Windows/Linux, autotest et création de fenêtre sur runner CI ([workflow](https://github.com/commandant-x/AION/actions/workflows/aion-gpu-explorer.yml)). Validation graphique réelle sur le GPU utilisateur : à confirmer.

---
# PRIMORDIA 0.11 — Premier monde 3D GPU explorable

**NOUVEAU :** `crates/aion-explore` est un explorateur Windows/Linux construit en Rust avec wgpu et winit. Paysages procéduraux, continuité de terrain, streaming de chunks, caméra à la première personne, couleurs de biomes, éclairage et brouillard. La végétation est **visuelle uniquement**.

Lancez `aion-explore.exe` (fichier fourni en artefact par [la CI Windows](https://github.com/commandant-x/AION/actions/workflows/aion-gpu-explorer.yml)). Commandes : WASD marcher, Shift sprinter, clic gauche regarder (capturer la souris), Échap libérer, F voler, Espace/Ctrl monter/descendre en vol, R réinitialiser. `--seed 42` sélectionne la graine. `--self-test` et `--window-smoke` sont les modes de validation.

**Validation réelle :** [Windows et Linux réussis](https://github.com/commandant-x/AION/actions/runs/38573649405) — tests, construction release, autotest et création de fenêtre Windows. L'affichage GPU interactif sur la machine de l'utilisateur n'a pas encore été confirmé.

**Limites importantes :** pas encore d'eau simulée, d'objets physiques manipulables, d'écologie vivante ou d'agents apprenants. Le Goal PRIMORDIA reste ouvert et ne sera clôturé qu'une fois ses critères complets satisfaits.

---

# AION CORE 0.10 — Fenêtre Windows native en Rust

**Nouvelle application graphique :** `aion-window.exe` (`cargo build --release -p aion-window`). Elle affiche une projection 3D interactive du solveur mécanique Rust : particules, liaisons, fractures, caméra orbitale, impact, traction, chaleur, pause et sauvegarde AION7. Il ne s'agit pas encore d'un moteur graphique GPU.

**Validation CI Windows :** compilation, autotest du moteur, création de fenêtre Win32 et rendu graphique sur contexte GDI dans [GitHub Actions](https://github.com/commandant-x/AION/actions/runs/38562213009). Application affichée sur un PC utilisateur : à confirmer.

**Commandes :** I impact, T traction, H chaleur, espace pause, R reset, S sauvegarde, L chargement, 1/2/3 matériaux, souris pour la caméra.

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
