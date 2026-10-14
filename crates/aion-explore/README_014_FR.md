# AION CORE 0.14 — premières cavités volumiques (recherche)

Cette version poursuit le Goal PRIMORDIA sans prétendre avoir terminé une croûte terrestre simulée.

## Nouvelles interactions

- **B** : viser la roche devant soi et excaver un véritable voxel cubique de 2 mètres de côté, volume **8 m³**. Les blocs enfouis peuvent être retirés sous un toit intact, pour commencer un tunnel.
- **N** : déposer une cellule volumique de 8 m³ de la matière sélectionnée dans l'air juste devant le point visé (à condition de la posséder).
- **1 à 8** : choisir l'un des huit matériaux existants.
- **P / L** : enregistrer et restaurer les deux jeux de modifications (terrain 0.13 et voxels 0.14). Le nouveau fichier porte l'extension `.aionv14` à côté de la sauvegarde `.aion13`.
- **Q / E** : restent les outils de surface antérieurs (2 m³ par intervention), avec leur inventaire propre.

Les faces internes découvertes sont rendues dans le maillage GPU et les cellules excavées au sommet percent visuellement le maillage de terrain de haute résolution. Les couches des voxels sont des matières déterministes liées aux horizons déjà simulés.

## Limitations actuelles

Le rendu volumique est cubique et local. La géométrie des trous n'est pas lissée sur les frontières du heightfield, et les systèmes de stockage de surface et de voxels sont temporairement séparés. La caméra possède toujours le contrôleur de collision par heightfield : **elle ne peut pas encore entrer physiquement dans les galeries**, même si leur volume et leurs parois existent. Les visuels ne constituent ni des contraintes géomécaniques ni une simulation tectonique. Les remplissages d'eau, la stabilité mécanique des toits et les effondrements sont à construire.

Tests attendus : conservation par matériau et volume, galeries verticales/horizontales, déterminisme, sauvegarde/verrouillage de graine, meshing GPU, restauration, CI Windows/Linux, véritable exe Windows.