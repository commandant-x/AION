# AION CORE — état de reprise au jalon 0.5

## Objectif prioritaire
Faire de Rust le solveur unique pour Win32 + tests, sans changer les lois par
accident. Conserver le moteur C comme outil de référence non lié à la release.

## Déjà produit
- Implémentation initiale de mécanique en Rust pur dans `aion-rust-sim`.
- Types C ABI compatibles selon les tailles calculées par le compilateur C.
- Tests Rust d'invariants, de reproductibilité, plasticité, fatigue, chaleur.
- Traces numériques C de 3 matériaux ; tests de parité Rust programmés.
- Script Windows pour relier le visualiseur Win32 au nouveau solveur Rust.
- Plan CI Linux / Windows avec archive Windows issue du noyau Rust.
- Deux exécutables Win64 de **secours C** construits et inspectés.

## Bloquant actuel
Il n'y a pas de `rustc` / `cargo` installés, et l'environnement ne peut pas
résoudre le domaine public de téléchargement du toolchain. Aucun `cargo test`
ni binaire réellement lié à Rust n'a pu être lancé ici.

## Actions prioritaires lors d'une session Codex / CI
1. Installer/activer Rust stable et exécuter `cargo fmt --check` puis `cargo test`.
2. Corriger toute erreur Rust ou écart avec le golden C.
3. Compiler `AION_CORE_0_5_RUST.exe` et `_RUST_TEST.exe` sur Windows.
4. Lancer réellement les deux exécutables et archiver les logs.
5. Comparer énergies, masse, fractures, impulsions et timings CPU C vs Rust.
6. Fermer les régressions d'ABI et corriger la dette sur l'énergie des bonds.
7. Produire la release native Rust signée par SHA-256 ; déprécier C preview.

## Règle de reprise
Ne jamais déclarer ce goal réussi si le visualiseur tourne sur le vieux C.
