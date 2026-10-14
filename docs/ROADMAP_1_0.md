> Mise à jour 0.3-alpha : unification du noyau utilisé par l'exécutable et du noyau accessible via FFI Rust. Cela ne constitue **pas** encore le portage autoritatif du moteur en Rust ni un rendu GPU ; voir `docs/RELEASE_0_3.md`.

# AION — Jalons de PRIMORDIA CORE 0.1 à jeu expérimental 1.0

**Règle : aucun passage au jalon suivant sans critères de sortie testés.** Une version chiffrée décrit le progrès technique, pas l'avènement garanti d'une société industrielle dans les simulations.

| Version | Livrable démontrable | Critère de sortie observable |
|---|---|---|
| 0.1 — Noyau | dépôt Rust modulaire, expérience mécanique/chimique/neurale jouet, tests analytiques | masses et éléments conservés; seed reproductible; compilation/test native vérifiés sur Rust stable |
| 0.2 — Mécanique | contacts robustes 6DOF, joints, frottements, plasticité, fracture de géométries simples | bancs de pendule/chute/impact/rupture comparés à solutions et mesures de référence; stabilité 10^6 pas |
| 0.3 — Matière | multi-matériaux, température, phases, réactions bornées, fluides locaux | bilans énergie/éléments et transitions sous protocoles connus; aucune réaction « magie » de craft |
| 0.4 — Monde | terrain 3D en chunks, cycle eau/carbone, météo et biomes, gisements localisés | migration A↔B↔C sans pertes de stocks; mémoire persistante des transformations |
| 0.5 — Corps | bipède articulé, préhension, locomotion, vision, audition, besoins | l'individu peut prendre/déposer des objets arbitraires, boire, se reposer, tomber et se relever sans téléportation |
| 0.6 — Apprentissage | agents à politique aléatoire, entraînement entraînable, mémoire et contrôles d'instinct | apprentissage locomotion/évitement/récupération dans des mondes non vus ; comparaison aux heuristiques |
| 0.7 — Fabrication ouverte | découpe, abrasion, assemblage, séchage, combustion, propriétés émergentes | production d'une structure utile non référencée comme objet connu; test sur tâches jamais utilisées en entraînement |
| 0.8 — Écologie | faune et flore, reproduction, nutrition, risques, habitats; milliers de jours simulés | bilan trophique stable ou extinction explicable; jamais de nourriture créée arbitrairement |
| 0.9 — Culture | groupes, imitation, signaux acoustiques, enfants, diffusion/perte de pratiques | apprentissage intergénérationnel meilleur avec interaction qu'en isolation; langage complexe non garanti |
| 0.95 — Présentation | vraie vue à la première personne, corps et forêt convaincants, animation, audio spatialisé | profil 1440p avec cibles de framerate définies, absence de saccades lors du streaming, tests d'interaction utilisateur |
| 0.99 — Intégration | monde persistant, analyses, replays, sauvegardes, ablations, profiler | runs longs sans corruption ni fuite mesurable, profils CPU/GPU publiés, bugs critiques corrigés |
| 1.0 — Expérience scientifique jouable | sandbox de survie incarnée + rapports de découverte et expériences répétables | les agents apprennent des compétences de survie dans de nouveaux environnements; au moins un mécanisme matériel/culturel émergent démontré sans recette cachée |

## Ce que **1.0 ne promet pas**

- Une Terre atomiquement exacte, l'invention spontanée de tous les éléments de notre civilisation ou une simulation fidèle de 3 millions d'années d'évolution.
- Une intelligence humaine vierge fiable et autonome : l'apprentissage en monde ouvert demeure un problème de recherche.
- Un moteur AAA comparable à des productions de plusieurs centaines de personnes en une livraison unique.

Ces sujets peuvent rester des axes de recherche bien après 1.0. La finition visuelle est un jalon auditable, pas une promesse vague.

## Ordre des prototypes en vertical slices

1. **Une pierre** : fissure, chaleur, friction, masse et énergie mesurables, transformations sauvegardées.
2. **Une main** : saisir, manipuler, voir et ressentir sans classe d'objet.
3. **Un organisme** : faim, soif, fatigue, proie/prédateur; conséquences du terrain et de la météo.
4. **Deux individus** : imitation, vocalisation, compétition pour un objet, conservation de leur mémoire individuelle.
5. **Une génération** : héritage biologique vs apprentissage social, sans copie génétique de connaissances.
6. **Un territoire** : écologie persistante et ressources localement rares, évolution de groupe sans arbre technologique.

## Rôle des décisions humaines

Chaque jalon publie un rapport de validation, démos enregistrées, résultats de seed multiples, comparaisons avec et sans règles d'apprentissage et un inventaire public de raccourcis/approximations. Les décisions sur la fidélité, l'étendue géographique, la violence, le métabolisme et la complexité visuelle doivent être contrôlables dans le simulateur.

## Contrat de livraison Windows (ajout 0.2-alpha)

Chaque version doit fournir un `.exe` Windows x64, des tests reproductibles, et son code source. Le type de `.exe` et son degré d'intégration avec le noyau Rust doivent être documentés. La version 0.2-alpha dispose d'un visualiseur natif autonome en C et d'un autotest natif Windows ; elle **ne franchit pas encore** tous les critères de sortie mécaniques de la version 0.2 inscrits ci-dessus. Les futurs jalons exigeront le raccordement au solveur Rust, puis un rendu GPU natif vérifié.
