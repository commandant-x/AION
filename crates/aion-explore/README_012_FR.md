# AION / PRIMORDIA CORE 0.12 — relief, sauts, littoral

> Jalon partiel du GOAL global PRIMORDIA. Ne correspond pas à une planète vivante ou une simulation géologique complète.

## Changements

- ESPACE fait sauter le joueur en mode marche ; gravité, vitesse verticale, atterrissage et pente maximale. En vol (F), ESPACE monte et CTRL descend.
- Maillage GPU local adaptatif et génération de maillage sur un thread dédié : centre 2 m (128 segments de 256 m), voisinage 4 m, extérieur 8 m. Jupes verticales dissimulant les discontinuités d'un niveau à l'autre.
- Géologie procédurale structurée : soulèvement spatialement corrélé, dureté variable, vallées incisées et vastes bassins. Aucun calcul de plaques tectoniques ni transport réel de sédiments.
- Surface de mer translucide à 0 m, profondeur dessinée en dessous ; houle seulement visuelle. L'eau n'est pas encore un fluide ni un réseau de rivières.
- La marche au niveau de l'océan utilise un plancher de surface approximatif : aucune flottabilité physique.
- Point de départ recherché près d'une côte lorsque cela est possible pour la graine choisie.

## Commandes

Clic gauche pour capturer la souris, WASD déplacement, ESPACE saut, Shift sprint, F vol, CTRL descendre en vol, R réinitialiser, Échap relâcher le curseur.

## Vérifications

`cargo test -p aion-world -p aion-explore --all-targets` puis `cargo build --release -p aion-explore`. `aion-explore.exe --self-test` vérifie le modèle sans fenêtre ; `--window-smoke` vérifie la création d'une fenêtre Win32 sans moteur GPU, sur CI Windows.

**Limitations documentées** : pas d'érosion en temps simulé, pas de sédimentation, pas de cours d'eau animés, pas de météo, pas de textures PBR, pas de végétation vivante. Détail adaptatif généré sur un thread de travail ; le streaming peut cependant prendre quelques secondes après un déplacement rapide. GPU interactif du PC utilisateur à tester.