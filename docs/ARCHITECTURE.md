> NOTE 0.3-alpha : le simulateur effectivement exécuté par le laboratoire Windows est maintenant `native/aion_sim.c`; `crates/aion-native` est son adaptateur Rust FFI. `crates/aion-core` conserve d'anciens modèles exploratoires distincts. Consulter `docs/RELEASE_0_3.md` pour les limites et travaux restants.

# Architecture AION CORE — ADR 001

## Répartition des responsabilités

**Rust** est l'autorité sur l'état de la simulation. Les sous-systèmes ne communiquent jamais en modifiant directement les données internes des autres : un ordonnanceur explicite orchestre les échanges via ports versionnés et journaux d'événements. Les unités physiques sont documentées (SI) et les invariants vérifiés au niveau du noyau.

### Modules du futur moteur

| Domaine | Représentation principale | Échelle / cadence visée | Couplage |
|---|---|---|---|
| Univers, espace, climat | cellules géophysiques, relief, hydrologie, météorologie | heures–années simulées, loin de la caméra | chaleur, eau, flux solaires |
| Sols et géologie | strates, transport sédimentaire, minéraux | heures–millénaires | bilans d'éléments, altération |
| Mécanique | corps rigides/structures articulées, FEM/MPM local selon tâche | 60–240 Hz zone active | forces, chaleur, fractures |
| Matières et transformations | constitutifs, phases, chimie réactionnelle | sous-pas adaptés à la raideur | masses élémentaires, enthalpie |
| Fluides | volumes, incompressibilité, SPH/FLIP local | adaptatif | transport de chaleur et matière |
| Rayonnement/optique | illumination solaire, diffusion, rendu PBR GPU | simulation environnement lente; rendu 60+ Hz | énergie radiative et perception visuelle |
| Acoustique | propagation par bandes, occlusion, réverbération | 20–100 Hz audio/jeu | perception auditive des agents |
| Écologie | individus ou cohortes, ressources biotiques, chaînes trophiques | minutes–saisons | matière, énergie, prédation |
| Organismes | squelette, métabolisme, sensorimotricité, développement | mouvements 30–120 Hz; besoins 1–10 Hz | calories, eau, chaleur, perception |
| Cognition | observations non symboliques, mémoire, politiques vierges | 1–30 décisions/s agent | actions vers corps, récompense intrinsèque |
| Société | interactions, imitation, apprentissage social, vocalisation | événements | transmission de connaissances non génétiques |
| Monde persistant | chunk streaming, versionnement, relecture | au besoin | stockage, audit causal |
| Présentation | rendu 3D natif Rust via API GPU (wgpu envisagé) | fréquence écran | lecture de snapshots, jamais autorité physique |

### Fidélité adaptative

- **Échelle A** autour du joueur et des mains d'un agent : objets séparés, contacts, géométrie modifiable, transfert thermique et chimie ciblée. Pas de dynamique atomique généralisée.
- **Échelle B** zone visible : rigid bodies et fluides approximés, agents actifs percevant leur environnement, végétation explicite.
- **Échelle C** arrière-plan régional : processus statistiques et continus, populations/cohortes, météorologie à maille élargie.
- **Échelle D** monde éloigné : bilans et événements saisonniers/annuels, puis raffinement lors de l'arrivée du joueur.

La bascule entre échelles doit préserver les stocks conservés, l'historique, les lignées et les statistiques. Aucun passage en haute résolution ne crée gratuitement de matière ou de savoir. Les entités lointaines ne sont pas simplement gelées.

### Valeurs de base

- *Observations des agents* : pixels/champs visuels, pression des capteurs, signaux acoustiques, proprioception, indicateurs physiologiques (et non « minerai de cuivre à gauche »).
- *Effecteurs* : forces/angles musculaires, saisir, relâcher, vocaliser, ingérer, respirer. Ce n'est pas un menu d'inventions.
- *Apprentissage* : politique initialisée à partir d'un seed, RL contraint par budget énergétique et horizon de survie, mémoire épisodique ; tests aveugles sur environnements nouveaux. Un minimum d'instinct inné physiologique et moteur est acceptable si explicitement mesuré.
- *Objets* : entités matérielles de géométrie et assemblages continus, pas des classes « roue » ou « hache ». Les noms et rôles ne sont que des étiquettes d'analyse hors des cerveaux.
- *Émergence du langage* : vocalisation et imitation sur canaux acoustiques sans lexique ; tests de référentialité, généralisation et transmission intergénérationnelle.

### Couplages critiques

1. **Matière ↔ chimie** : bilan élémentaire / enthalpie, prise en compte des propriétés après réaction et changements de phase.
2. **Matière ↔ mécanique** : rigidité, rupture, plasticité et adhérence dépendent de la composition et de la température.
3. **Mécanique ↔ biologie** : dommages corporels, fatigue, dépense d'énergie, locomotion.
4. **Biologie ↔ apprentissage** : récompenses internes fondées sur besoins réels ; observation bruitée, pas de connaissance privilégiée.
5. **Apprentissage ↔ culture** : comportements acquis transmis par observation et pratique, jamais copiés dans le génome.
6. **Culture ↔ environnement** : outils conservés, lieux modifiés, traces écrites/sonores éventuelles, savoirs perdus ou diffusés.

### Partition et parallélisme

- État temporel versionné par `SimTick`. Échanges intersystèmes par buffers d'événements explicitement ordonnés.
- Chargement d'une zone en streaming via chunks spatiaux à coût amorti borné.
- Multithreading déterministe pour simulations scientifiques de référence ; voie GPU et flottants rapides pour jeu avec divergence tolérée, mais détectée par comparaison de statistiques.
- Mode `research`: déterminisme renforcé, traces de causalité, énergie/matière auditées, CPU ; mode `play`: budget d'image prioritaire et approximations quantifiées.

### Choix technologiques envisagés

- Cœur : Rust stable, `cargo test`, profiling `tracing`, benchmarks `criterion` plus tard.
- Calcul géométrique / rigide : kernels Rust dédiés avec validations contre solutions analytiques ; possible intégration de Rapier/Jolt pour les contacts génériques afin de ne pas réinventer sans vérification toute la dynamique.
- Rendu : wgpu/winit, pipeline PBR, terrain streaming, LOD, virtual texturing si justifié, TAA, traitement d'image et animation via GPU ; formats glTF pour contenus scannés/modélisés.
- Neuronal : PyTorch/JAX hors moteur pour entraînement distribué (CPU/GPU), export ONNX/format compatible pour inférence ; enveloppe Rust claire. Une implémentation 100 % Rust est étudiable une fois les expériences validées.

Ceci est une architecture cible, non un inventaire de fonctionnalités implémentées.
