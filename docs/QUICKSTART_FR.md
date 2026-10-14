# AION CORE 0.3 — Guide de démarrage Windows

## 1. Lancer

Double-cliquez sur `AION_CORE_0_3.exe` (Windows 10/11, 64 bits). Pas d'installation ni d'accès Internet requis. Si Windows SmartScreen affiche un avertissement lié à l'absence de signature du logiciel, vérifiez l'origine du téléchargement et son empreinte SHA-256 avant de décider de l'ouvrir. **L'exécutable n'est pas signé.**

## 2. Utiliser le laboratoire

- Glisser-déposer à la souris : faire tourner la caméra autour de l'échantillon.
- Molette : zoomer.
- `I` : appliquer une impulsion à l'impacteur.
- `H` : chauffer un côté du réseau de particules.
- `1`, `2`, `3` : choisir une des trois lois de matériau **expérimentales**.
- `Espace` : pause/reprise.
- `+`, `-` : changer la vitesse de simulation.
- `R` : recréer l'échantillon depuis son état initial.

Regardez le **graphe vert** de l'énergie mécanique et les **marqueurs orange** de rupture, sous le monde 3D. La colonne droite montre les particules, contraintes intactes, ruptures, impulsions et déformations.

## 3. Tester le moteur

Ouvrez l'invite de commandes dans le dossier, puis exécutez :

```bat
AION_CORE_0_3_TEST.exe
```

Un résultat `PASS` et un code retour `0` indiquent que les vérifications intégrées réussissent sur votre Windows. Vous pouvez également utiliser `RUN_TESTS.cmd` dans l'archive.

## 4. Limites importantes

Ceci est un **laboratoire mécanique simplifié** avec une interface native Win32/GDI. Ce n'est **pas** encore un environnement photoréaliste, une simulation d'atomes/ions, un organisme vivant ou un moteur GPU. Les particules représentent des éléments mécaniques macroscopiques ; leurs liaisons sont des ressorts.

Le moteur exécutable est actuellement écrit en C. Une passerelle Rust utilise le même noyau, mais sa compilation Rust doit encore être vérifiée. Le portage complet du solveur en Rust n'est donc **pas** achevé.
