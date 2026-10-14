# AION CORE 0.5 — statut exact de validation

## Règle stricte
Un `.exe` Win64 généré par le compilateur C n'atteste PAS que Rust a été compilé.
La 0.5 fournie ici est une **candidate de migration**, et NON une livraison
entièrement Rust. Le critère « Rust source unique de vérité » reste ouvert.

## Vérifications exécutées dans l'environnement de création

| Élément | Résultat | Limite |
| --- | --- | --- |
| Programme Windows C preview | PE32+ x86-64 généré avec clang/lld | Pas exécuté sur Windows |
| Diagnostic Windows C preview | PE32+ x86-64 généré | Pas exécuté sur Windows |
| Moteur C de référence | 99 assertions natives | Ne teste pas le moteur Rust |
| Réponse mécanique additionnelle | 27 assertions | Tests C |
| Test C de l'application (sur Linux) | PASS | Ce n'est pas Wine/Windows |
| Tests de référence numériques Python | 8 réussis | Tests indépendants |
| Stabilité C | 3 × 15 000 pas réussis | Ne démontre pas une stabilité universelle |
| Golden traces Rust/C | Traces C générées, tests Rust écrits | Pas exécutés côté Rust |
| Cargo test Rust/Linux/Windows | BLOQUÉ | rustc et cargo indisponibles ici |
| GUI Rust-linked Windows | NON COMPILÉE | Exigence de jalon non satisfaite |
| Résultat en conditions réelles Windows | NON VÉRIFIÉ | À valider sur un PC Windows / CI |

## Trajectoires de référence
`tests/reference_trace_05.c` génère des mesures à 0, 1, 5, 20, 80,
400 et 2000 pas, pour chacun des 3 matériaux. Le fichier
`crates/aion-rust-sim/tests/fixtures/c_reference.csv` conserve ces mesures.
Le test Rust `reference_parity.rs` impose la parité à court horizon, sur 3 modes.
Les longues trajectoires de fractures sont particulièrement sensibles à l'ordre
flottant : une erreur de 1e-15 ne doit pas automatiquement être confondue avec
un défaut de physique. L'analyse doit comparer les invariants et statistiques.

## Invariants scientifiques et limites
- Masse constante, interactions internes conservant l'impulsion (sans plancher
  ni gravité), chaleur de conduction conservée sous les hypothèses du modèle.
- Le modèle de liaisons produit des dommages irréversibles et dissipation.
- Un compteur de chaleur dissipée ne garantit pas à lui seul un bilan d'énergie
  exact : notre intégrateur semi-implicite a une dérive numérique documentée.
- Le diagnostic historique d'énergie mécanique ne pondère pas correctement
  la réserve élastique par `(1 - damage)`; cette incohérence est une dette de
  modélisation à résoudre AVANT toute affirmation sur l'énergie totale.
- Les particules représentent des masses discrétisées, **pas des atomes**;
  aucun électron, ion quantique, chimie de liaisons électroniques, ni monde
  physiquement exact n'est encore présent dans ce module.

## Critères restant indispensables pour fermer le goal 0.5
1. `cargo test --workspace --all-targets` réussit sur Linux et Windows.
2. Génération de `AION_CORE_0_5_RUST.exe` à partir du solveur Rust uniquement.
3. Exécution de ce `.exe` et de son diagnostic sur runner Windows.
4. Comparaison de performances et de précision C/Rust, puis audit scientifique.
5. Tests de l'ABI C/Rust et validation par sanitizers / Miri si possible.
6. Produire la nouvelle release complète et ne plus utiliser le C preview.
