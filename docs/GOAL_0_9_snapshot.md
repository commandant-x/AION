# GOAL: AION CORE 0.9 — Lever le blocage Rust pour de bon

**Définition de terminé** : un job GitHub Actions Windows affiche `success`, a exécuté `aion-rust-headless.exe --self-test`, les tests AION7 et 99 mesures, et a publié `AION_CORE_0_9_RUST_WINDOWS_X64_VERIFIED`.

Ce n'est **PAS terminé** dans le conteneur actuel, qui n'a pas de compilateur Rust.

## Actions restantes

1. Mettre ce projet dans un dépôt GitHub dédié (aucun dépôt AION utilisable n'était accessible lors de la préparation de la version).
2. Déclencher `.github/workflows/aion-rust-verified-09.yml`.
3. Examiner les erreurs de compilation, de parité ou d'exécution, corriger dans le dépôt jusqu'à ce que tous les jobs passent.
4. Télécharger et exécuter l'artefact Rust réel sous Windows.
5. Remplacer ensuite le solveur C dans le visualiseur, sans casser la parité scientifique ; utiliser la CI comme verrou de qualité.

## Livrables existants

- Noyau Rust candidat et CLI `--self-test`, `--save`, `--resume`, `--benchmark`.
- Validateur Rust/C multi-scénarios, test automatisé de conservation, pipeline Windows/Linux.
- Exécutables `.exe` **de référence C**, correctement identifiés.
- Scripts et instructions de build Windows autonome.

Ne jamais renommer le visualiseur C en moteur Rust ; ne jamais déclarer le solveur Rust vérifié sans compilation ET exécution.
