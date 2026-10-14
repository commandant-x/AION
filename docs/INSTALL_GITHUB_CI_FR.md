# AION CORE 0.9 — Compiler le VRAI `.exe` Rust

## Pourquoi cette CI est nécessaire

Le conteneur de développement utilisé pour préparer cette version n'a ni `rustc`, ni `cargo`, ni accès DNS vers les archives Rust. Il est **impossible d'y certifier** la compilation Rust. Un exécutable C renommé `.exe` Rust ne serait pas acceptable.

La solution reproductible est une compilation sur **GitHub Actions Windows** (et contrôle Linux), à partir des mêmes sources. Le workflow `.github/workflows/aion-rust-verified-09.yml` fait toutes les étapes ci-dessous et **ne publie pas l'exécutable si une étape échoue**.

## Avec GitHub Actions — recommandé

1. Créer un dépôt GitHub dédié, par exemple `aion-primordia-core` (privé possible).
2. Extraire l'archive de sources AION 0.9 et pousser **son contenu** à la racine de ce dépôt. Le répertoire `.github/workflows` doit être conservé.
3. Dans le dépôt : **Actions → AION CORE 0.9 - verified native Rust builds → Run workflow**.
4. Attendre les jobs Windows et Linux. En cas d'échec, consulter les erreurs : **aucun exécutable Rust n'est alors validé**.
5. Si le job Windows est vert, télécharger l'artefact `AION_CORE_0_9_RUST_WINDOWS_X64_VERIFIED` qui contient `aion-rust-headless.exe` (vrai moteur Rust), un checkpoint et les mesures.
6. Partager l'URL du dépôt ou les journaux du job en échec pour pouvoir corriger le code.

L'application Rust à ce stade est un **laboratoire en ligne de commande**, pas encore une interface 3D. Les applications `*_C_REFERENCE.exe` livrées séparément restent des visualiseurs utilisant le solveur C.

## Compilation directe sur Windows

Installer Rust stable MSVC depuis <https://rustup.rs/>, ainsi que Visual Studio Build Tools avec MSVC, puis, dans le dossier source :

```powershell
.\tools\build_rust_windows_09.ps1
```

Après réussite des tests, le script produit `dist\AION_CORE_0_9_RUST_WINDOWS_X64.exe`.

## Critères de validation du moteur Rust

- `cargo test --workspace --all-targets` passe sur `windows-latest` et `ubuntu-latest`.
- `aion-rust-headless.exe --self-test` passe **après son exécution native Windows**.
- La lecture des checkpoints AION7 générés en C fonctionne et une sauvegarde Rust se recharge.
- Le programme Rust produit 9 expériences × 11 points, vérifiées par `tests/validate_rust_benchmark_09.py`.
- Les grandeurs initiales concordent avec la référence C ; divergence tardive de trajectoires non linéaires signalée, jamais masquée par des tests artificiellement exacts.
- Le `.exe` se trouve dans les artefacts du workflow vert.

Cette procédure déverrouille **la compilation et la preuve d'exécution du moteur Rust**. L'intégration dans le visualiseur 3D est un jalon ultérieur, et l'émergence d'une civilisation n'est pas encore implémentée.
