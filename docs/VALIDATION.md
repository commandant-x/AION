# Protocole de validation — de la formule à l'émergence

## Trois niveaux de vérité

1. **Exactitude numérique** : comparer les noyaux élémentaires à des solutions analytiques, un solveur indépendant ou des mesures connues; surveiller masse, énergie, moment et bilans élémentaires selon les frontières ouvertes/fermées.
2. **Crédibilité écologique et cognitive** : reproduire des régularités robustes, y compris absences d'émergence; plusieurs seeds et intervalles de confiance, pas de vidéo séduisante choisie parmi mille échecs.
3. **Qualité ludique et graphique** : captures natives, sessions de test utilisateur, performances réelles; ce n'est pas une preuve de physique ni d'intelligence.

## Suites impératives

- Tests de propriétés physiques : masse, impulsion en système isolé, énergie totale en incluant chaleur, direction du flux thermique, non-négativité des stocks et températures.
- Chimie : bilans élémentaires exacts à tolérance numérique, réactions impossibles rejetées, enthalpie totale, scénarios de mélange contrôlés et cas extrêmes.
- Géométrie : aucun artefact topologique après découpe/rupture; pas d'interpénétration persistante; précision adaptée à la vitesse.
- Biosphère : bilan de matière/énergie de la chaîne alimentaire; organismes incapables de survivre sans eau, oxygène ou nourriture quand leur physiologie le nécessite.
- Cerveau : départ aléatoire vérifié, contrôle caché des poids, aucune observation d'un identifiant de ressource, absence de récompense directe du type « âge industriel atteint ».
- Générativité : objets et comportements testés sur des tâches **retenues hors entraînement**; vocabulaire et techniques ne figurent pas dans le moteur d'action.
- Communication : des paires entraînées réussissent mieux qu'une population où les signaux sont mélangés aléatoirement; distinction corrélation/communication causale.
- Culture : les descendants échouent en isolation et réussissent après interaction, sans copie des poids entraînés ni nom de recette héréditaire.
- Replay : même graine + même ordre d'actions → mêmes checksums dans le mode déterministe; GPU rapide peut diverger et doit être caractérisé.
- Performances : profil par sous-système (jeu, physique, écologie, IA, streaming, rendu). Un gain de FPS obtenu en supprimant des lois doit être indiqué.

## Contre-expériences nécessaires

- IA vs marche aléatoire, vs script simple, vs politique préentraînée : isoler l'effet réel de l'apprentissage.
- Avec vs sans culture, avec vs sans vocalisation, avec vs sans mémoire, avec vs sans curiosité.
- Même région sur au moins 20 graines; distribution des résultats, nombre de générations, décès et innovations utiles.
- Holdout : physique identique dans des conditions géographiques inédites, pas de technologie apprise par cœur.

## Registre des raccourcis techniques

Le noyau 0.1 contient **des sphères**, des **ressorts cassables**, des **réactions chimiques explicitement définies** pour valider la conservation, et un **petit classifieur à renforcement**. Il n'a ni solveur FEM/MPM, ni réseau récurrent suffisamment riche, ni flore/faune, ni UI 3D native. Il serait incorrect de décrire ce prototype comme « simulateur de l'évolution humaine ». Son rôle est d'établir les contrats, tests et fondations du moteur.
