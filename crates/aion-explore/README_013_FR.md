# PRIMORDIA CORE 0.13 — Croûte terrestre interactive (première fondation)

Cette version crée les premiers matériaux de terrain **interactifs**. Huit catégories synthétiques sont distinguées : terre, sable, argile, gravier, tourbe, calcaire, granite et basalte. Leur densité et leur dureté sont des paramètres simplifiés. Le type de couche dépend des données géologiques existantes, de l'altitude, de la pente, de l'humidité et de la profondeur.

## Commandes

- WASD : déplacement ; Espace : saut ; Shift : sprint ; F : vol libre ; clic gauche : capturer la souris, Échap : libérer.
- Regarder la zone de sol à modifier (à moins d'environ 40 m). `Q` : extraire une couche de 0,5 m sur une empreinte de 2×2 m, soit 2 m³ de matériau récupéré.
- Touches `1` à `8` : sélectionner terre, sable, argile, gravier, tourbe, calcaire, granite ou basalte.
- `E` : déposer 2 m³ du matériau sélectionné là où la caméra regarde, si disponible dans l'inventaire ; aucun dépôt ne peut créer une matière absente.
- `P` : sauvegarder les modifications dans un fichier `.aion13` situé à côté du `.exe`. `L` : charger la sauvegarde en vérifiant intégrité, cohérence des couches et conservation de matière. Une sauvegarde saine est chargée automatiquement au lancement.
- `R` : revenir à la position initiale sans effacer les travaux.

L'état des modifications et le volume du matériau sélectionné s'affichent dans le titre de la fenêtre. La géométrie GPU se reconstruit de façon asynchrone après chaque modification, en conservant le streaming des zones. Les collisions de marche prennent en compte le terrain modifié.

## Modèle et limites scientifiques

Ce jalon est une **surface maillée avec couches verticales** — pas une simulation volumique 3D continue de la croûte terrestre. Pas encore de tunnels, cavernes, surplombs, minéralogie chimique, métamorphisme, écoulements de sédiments, fractures de la roche ni simulation complète de tectonique. La conservation des volumes déposés et extraits est effectuée en unités discrètes de 2 m³ ; l'inventaire conserve chaque classe de matériau. L'évaluation physique des actions (effort, outils, énergie et temps) reste à ajouter.

## Validation

Le compilateur Rust teste le déterminisme des couches, leur diversité, leurs propriétés, la conservation exacte, les limites de creusement, les coordonnées négatives, la sauvegarde/relecture et les données altérées. La CI GitHub doit construire et tester sur Windows/Linux et générer le vrai `.exe` Windows avant publication.