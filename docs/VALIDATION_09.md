# AION CORE 0.9 — Rapport de validation honnête

## Vérifié localement sur Linux

- `tools/test_native.sh` : 99 assertions C, 27 assertions matériaux, 29 checkpoint 0.6, 66 AION7 et 840 tests du protocole expérimental ; 8 tests Python.
- Stress : 3 × 15 000 pas de simulation.
- `tests/validate_experiments_08.py` valide les 99 mesures C et leur schéma.
- `tests/validate_rust_benchmark_09.py` vérifié avec la référence C en entrée des deux côtés (teste le **validateur**, pas le solveur Rust).
- `tools/build_windows_from_linux.sh` : deux binaires Win64 (`PE32+ x86-64`) compilés, tous deux **moteur C**.
- `Cargo.toml`, Rust CLI benchmark et workflow CI créés ; compilation Rust **non exécutée localement**.

## En attente du runner GitHub Actions

- `cargo test --workspace --all-targets` sous Linux et Windows.
- `cargo build --release --bin aion-rust-headless` sous Windows.
- Exécution réelle de l'exécutable Rust Windows, rejouabilité des checkpoints AION7.
- Benchmark 9 × 11 en Rust et comparaison aux références C.
- Téléchargement de l'artefact Windows Rust vérifié.

## Limites scientifiques

- Les particules sont des éléments de matière **méso-échelle** reliés par ressorts ; elles ne sont ni des atomes ni des molécules.
- La mécanique et la dissipation constituent des approximations expérimentales ; pas de certification du premier principe de la thermodynamique.
- Le banc d'essai n'établit pas un comportement crédible de roche, de bois ou de verre réels sans calibration.
- Les checksums servent au rejeu **dans une même implémentation** et ne sont pas des preuves cryptographiques ni des identités garanties entre compilateurs.
