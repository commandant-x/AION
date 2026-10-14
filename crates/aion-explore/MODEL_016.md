# AION / PRIMORDIA 0.16 — Couplage expérimental voxel / hydrologie

Ce jalon connecte le volume rocheux excavable (AION 0.14) au prototype hydrothermique local (AION 0.15). Il corrige le cas où l'eau continuait de se comporter comme si une roche encore présente avait été creusée, ou disparaissait lorsqu'une cavité était rebouchée.

## Nouveau mécanisme

- `HydroThermal::reconcile_voxels(&World, &VoxelWorld)` détecte les changements de nature des cellules dans un patch borné.
- Une excavation change les propriétés locales de la roche vers une cellule de vide : la capacité de stockage d'eau augmente.
- Une remise en place de roche réduit cette capacité : l'eau excédentaire est **comptabilisée comme drainage hors du patch**, sans perte cachée.
- Les échanges d'énergie sensible attribués à l'entrée/sortie de roche sont consignés séparément dans les budgets `energy_in_j` / `energy_out_j`.
- L'opération est transactionnelle : en cas de rejet de paramètres ou de bilan incohérent, l'état initial reste intact.
- Le banc `aion-explore.exe --hydro-coupling <csv>` reproduit 61 observations espacées de 10 s. Le contrôle Python indépendant vérifie géométrie, bilans et événements.

## Limites scientifiques

- Le changement de matière rocheuse retire/ajoute ici une capacité thermique à température locale égale ; c'est une approximation d'échange enthalpique, et non une géomécanique ni une thermo-chimie de l'excavation.
- Le drainage excédentaire est exporté hors du patch, pas redistribué dans un bassin versant ou le terrain voisin. Il est comptabilisé, mais pas encore rendu visuellement.
- La physique reste un modèle local à volume fini de taille bornée, à paramètres synthétiques, sans écoulement turbulent, ruissellement, ni interface hydrogéologique à l'échelle d'un continent.
- Le mode graphique GPU général n'exécute pas encore le solveur hydrologique en continu pendant le gameplay. Le couplage est vérifié en simulation locale reproductible et constitue une fondation pour le raccordement au monde interactif.
- Le GOAL complet de PRIMORDIA est ouvert.

## Validation

CI GitHub Actions Windows + Linux : tests Rust du modèle et du programme, compilation release, bench couplage à 61 lignes, script indépendant `tests/validate_coupling_016.py`, test de fenêtre Windows. Ne déclarer le jalon validé qu'après passage réel.