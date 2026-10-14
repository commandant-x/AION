# Validation — AION CORE 0.4

## Capacités vérifiées

- 99 assertions historiques des mécanismes de contact, rupture et inertie.
- 27 assertions ciblées des comportements plastiques / fatigue / dashpot.
- Tests host du binaire de diagnostic Windows avec les mêmes sources.
- Trois simulations de résistance à 15 000 pas, une par matériau.
- Huit tests Python de référence physique expérimentale.
- Compilation croisée des deux exécutables Windows PE32+ AMD64 avec clang/lld-link.

## Blocages de validation

- Aucune exécution sur Windows dans cet environnement ; pas de Wine.
- Pas de compilateur Rust ; le crate Rust et l'ABI ne sont pas testés
  par `cargo` dans cette session.
- Le rendu est Win32 GDI, non accéléré GPU et non photoréaliste.
- Les matériaux ne sont pas calibrés expérimentalement; déformations,
  amortissement et fatigue ne sont qu'un modèle numérique exploratoire.
- L'intégrateur Euler semi-implicite n'assure pas une conservation exacte
  de l'énergie mécanique; les tests acceptent une tolérance annoncée.

## Actions de validation sur Windows

```powershell
./bin/AION_CORE_0_4_TEST.exe
./bin/AION_CORE_0_4.exe
cargo test --workspace
```

Inspecter la fenêtre, essayer `I`, `T`, `H`, `1` à `3`, vérifier qu'aucun
crash ni nombre impossible n'apparaisse pendant au moins quinze minutes.
