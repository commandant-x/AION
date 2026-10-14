# AION — Direction visuelle et critères de rendu (ADR 002)

## Intention : nature observée à hauteur d'un organisme

L'image ne sera ni une carte de stratégie, ni une vitrine de shaders gratuits. Référence artistique : photographie naturaliste de terrain, lumière atmosphérique, roche humide, microrelief, végétation irrégulière, poussière, brouillard matinal, saisons, traces au sol et corps crédibles. Les inventions doivent pouvoir être vues, touchées, manipulées et dégradées physiquement.

### Caméra

- Vue à la première personne, taille d'œil attachée au corps, accélérations vestibulaires filtrées pour le confort. Vision périphérique / FOV et limitations d'attention des agents configurables.
- Observer un agent en vue subjective ne lui donne pas de connaissances supplémentaires. Caméra spectateur scientifique indépendante de sa perception.
- Modes : témoin libre, incarné (prise de contrôle optionnelle), spectateur d'un agent, microscope local de laboratoire. L'interface du jeu ne doit pas révéler à l'agent les paramètres cachés de l'environnement.

### PBR et lumière

- Matériaux physiquement plausibles (albédo mesuré, rugosité, normale et réponse humide); propagation multi-échelle de la lumière sans simuler tous les photons.
- Soleil/lune, ombres douces cohérentes, brume volumétrique locale, climat et micro-occlusion. Représenter honnêtement le comportement thermique/radiatif simulé, sans imposer un coût de lancer de rayons complet au gameplay.
- Géométrie : terrain en tuiles et LOD, végétation instanciée avec variations biologiques, surfaces déformables près des actions pertinentes, textures/meshes virtuels lorsque la mémoire le justifie.
- Optique : exposition stable, colorimétrie, anti-crénelage TAA ou solution mesurée, réduction des scintillements des feuillages et des ombres, pas de sur-netteté artificielle.

### Animation des organismes

- Bipède articulé, contraintes anatomiques, marche et préhension avec cinématique inverse, contacts au sol, centre de masse, fatigue et blessures; transition pilotée par une politique motrice sans animation « magique » de craft.
- Expressions et gestes sobres; absence de parole scriptée. Sons corporels et vocalisations générés acoustiquement.
- L'animation visuelle peut interpoler l'état de la physique, jamais fabriquer un outil ou déplacer un corps sans coût correspondant.

### Son

- Propagation, atténuation et occlusion acoustiques, réverbération selon le milieu; cri et bruit de matériau servent à la perception des agents, pas seulement à l'ambiance.
- Différencier audio de jeu (haute fidélité perceptive) et capteurs simulés (bande passante explicite, bruit) pour éviter les informations privilégiées.

## Benchmarks visuels proposés (à valider sur matériel réel)

| Test | Cible initiale, non garantie |
|---|---|
| PC milieu/haut de gamme, 2560×1440 | 60 images/s tenues dans une scène chargée, options adaptatives |
| Latence interaction proche | < 80 ms depuis action jusqu'au premier effet visible, à mesurer |
| Sim physique active | 120 Hz localement si stable; sous-pas adaptés |
| Streaming | déplacement continu sans blocage long ni popping notable |
| VR, phase post-1.0 | 90 Hz sur profils compatibles, confort d'abord |
| Fidélité | comparaisons image-référence et inspections manuelles sur pierre, bois, eau, peau, nuit, feu |

Chaque capture de jalon doit être produite **par le moteur exécuté**, avec version, seed, réglages, matériel et FPS. Les illustrations d'intention ne doivent jamais être présentées comme des captures de jeu.

## Priorité

Mieux vaut un seul vallon extrêmement crédible et cohérent avec la simulation qu'une planète entière graphiquement pauvre et peuplée de robots animés sans contact. Le premier vertical slice rendu visera une vallée tempérée avec sol, eau, roche, végétation et deux organismes observables.
