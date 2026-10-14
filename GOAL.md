# AION PRIMORDIA CORE — objectif prioritaire

Faire fonctionner un solveur mécanique Rust natif et produire un exécutable Windows issu du code Rust, avec une suite de tests reproductibles.

## Conditions de validation

- Importer intégralement les sources AION CORE 0.9 sans perdre les fichiers de test.
- Compiler avec `cargo test --workspace --all-targets` sur Windows et Linux.
- Construire en mode release `cargo build --release -p aion-rust-sim --bin aion-rust-headless`.
- Exécuter réellement sous Windows le binaire obtenu et ses autotests.
- Comparer les 99 mesures des neuf expériences aux références C avec tolérances explicites.
- Lire et réécrire les sauvegardes AION7 ; vérifier le rejeu déterministe.
- Publier l'exécutable Windows `.exe`, ses sources, et son rapport de tests uniquement après réussite de la CI.

**État actuel :** dépôt GitHub identifié. L'import intégral des sources et la première compilation CI restent à effectuer. Aucun `.exe` Rust n'a encore été validé.
