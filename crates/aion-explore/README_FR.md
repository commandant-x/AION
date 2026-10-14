# AION / PRIMORDIA 0.11 — Premier monde GPU explorable

Cette application est un **premier visualiseur de terrain** en 3D GPU réelle (Rust + wgpu).
Elle ne représente **pas** la simulation complète du monde ni l'intelligence des agents.

## Lancement Windows

Double-cliquer sur `aion-explore.exe`. La première ouverture peut nécessiter quelques secondes pour initialiser le pilote GPU et générer les premiers maillages. L'application affiche un message d'erreur Windows si l'initialisation échoue.

- `W`, `A`, `S`, `D` : marcher dans le paysage ; `Shift` : accélérer.
- Clic gauche : capturer la souris et regarder autour de soi ; `Échap` : libérer la souris.
- `F` : marche / vol libre. En vol, `Espace` monte et `Ctrl` descend.
- `R` : réinitialiser la caméra et recharger le paysage.
- Ligne de commande : `aion-explore.exe --seed 42` pour choisir un monde déterministe.
- Vérification sans affichage : `aion-explore.exe --self-test`.

## Fonctionnel

- Maillages de terrain continus, normales, éclairage directionnel, profondeur et brouillard atmosphérique (GPU).
- Chunks 256 m × 256 m chargés dynamiquement autour du joueur (rayon 3 ; 49 chunks).
- Mouvement lié au relief et option vol (pas de collision avec les arbres et objets).
- Végétation conifère **visuelle uniquement**, placée de façon procédurale et déterministe.
- Exécutable Windows natif, sources Rust, tests géométriques, tests de caméra et compilation CI Windows/Linux.

## Limites

- Pas de physique des terrains, eau liquide, système écologique évolutif, interaction avec la matière ou agents.
- Le `--window-smoke` CI valide la création d'une fenêtre Windows. **L'affichage GPU interactif sur le matériel de l'utilisateur doit encore être testé.**
- Les arbres sont des maillages approximatifs, pas des organismes simulés.
- Sur les GPU sans prise en charge de DirectX 12/Vulkan appropriée, le programme peut refuser de démarrer et afficher une boîte de dialogue explicite.

Le Goal global reste ouvert jusqu'à satisfaction de **toutes** les exigences de PRIMORDIA dans `GOAL.md` du dépôt.