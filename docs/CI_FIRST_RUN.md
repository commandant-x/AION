# Première validation native Rust — AION CORE 0.9

Cette branche déclenche les tests du moteur sur GitHub Actions.

## Critères d'acceptation

- `cargo test --workspace --all-targets` passe sur Linux et Windows.
- Le programme `aion-rust-headless` est compilé en mode release.
- Les autotests de sauvegarde et de rejeu s'exécutent sur Windows.
- Les 99 mesures des neuf expériences sont comparées au moteur C.
- Un artefact `.exe` est publié **seulement si** les validations précédentes réussissent.

Ne pas fusionner la branche tant que les résultats ne sont pas compris et vérifiés.
