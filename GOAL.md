# AION / PRIMORDIA — GOAL FINAL ET PERMANENT

**Règle** : le Goal reste ouvert jusqu'à la réalisation et la validation de TOUS les objectifs ci-dessous. Aucun jalon intermédiaire, version, démonstration ou passage au vert de la CI n'est une condition de clôture.

## Définition de terminé (tous les critères requis)

- [ ] Monde 3D GPU natif Windows réellement explorable à la première personne : vastes paysages, streaming, collisions, interactions physiques et sauvegardes persistantes.
- [ ] Modèles multirésolution de matériaux, déformations, fractures, température, transferts d'énergie et conservations quantitatives vérifiées.
- [ ] Géologie, eau, climat, sols, ressources et écologie évolutive cohérents, avec limites de modèle explicites.
- [ ] Organismes incarnés : sens, déplacement, nourriture, eau, repos, physiologie, croissance, reproduction, mortalité et écologie.
- [ ] Agents partant sans connaissances culturelles, expérimentant et apprenant à survivre par interactions causales avec le monde.
- [ ] Transformations non-scriptées d'objets géométriques et inventions sans arbre technologique ni recettes pré-écrites.
- [ ] Communication émergente, transmission culturelle, coopération, désaccords et sociétés évolutives.
- [ ] Simulations durables, sauvegardes/reprises, tests robustes sur le matériel cible et expérience exploitable par l'utilisateur.

Ces critères constituent un programme de recherche et développement ouvert, sans garantie de simulation atomique/quantique intégrale ni de comportements intelligents humains.

## Règles de travail

À chaque jalon VALIDÉ, informer l'utilisateur des fonctions effectives, des changements de code, des validations Windows/Linux, des limites et du prochain chantier. Le travail continue ensuite.

À chaque VERSION livrée, fournir un véritable exécutable Windows .exe, les sources complètes, les tests et un rapport honnête. Un module interne non livré ne constitue pas une nouvelle version.

## État technique

- AION 0.10 : noyau mécanique Rust et visualiseur diagnostic Win32/GDI, préservés pour les expériences de référence.
- AION 0.11 : premier explorateur GPU natif Rust (`aion-explore`, wgpu), terrain déterministe, chunks continus chargés autour du joueur, caméra FPS avec suivi de hauteur, vol libre, éclairage/brouillard et végétation purement visuelle. Tests Windows/Linux et création de fenêtre Windows en CI réussis ; fonctionnement GPU sur PC utilisateur encore non testé.
- AION 0.12 : saut contrôlé avec gravité, rendu du niveau marin et des fonds océaniques, maillages à LOD 2/4/8 m, géologie procédurale (soulèvement, dureté, incision), génération des chunks sur thread séparé. Tests Rust et fenêtre Windows CI au vert ; rendu GPU à valider sur le matériel cible. Ce ne sont toujours pas des modèles fluides/géologiques dynamiques.
- AION 0.13 : première matérialité du terrain : huit types de sol/roche avec densités et duretés synthétiques, horizons liés aux champs géologiques, creusement et remblai en cellules de 2 m × 2 m par couches de 0,5 m, conservation discrète par matériau, inventaire, édition du relief dans le GPU, collisions adaptées et sauvegardes vérifiées. IMPORTANT : modèle encore à surface 2.5D, sans cavernes/surplombs ni résistance mécanique réelle des outils.
- Chantier actif : représentation volumique adaptée (octree/voxels et frontières des phases), outils/énergie/contraintes de matériaux, érosion et transport de sédiments, hydrographie dynamique et conservation, interactions avec objets physiques dans le paysage, textures puis écologie et agents incarnés.
- Physique de particules simplifiée ; ne pas la présenter comme simulation atomique.

**NE JAMAIS MARQUER LE GOAL TERMINÉ SUR UN SEUL JALON.**