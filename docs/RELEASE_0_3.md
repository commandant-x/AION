# Version 0.3-alpha — SHARED MECHANICS / dossier de validation

## Contrat de livraison

| Livrable | État |
| --- | --- |
| Application native Windows x64 | Binaire PE généré, exécution Windows non testée |
| Autotest Windows x64 | Binaire PE généré, chemin de test exécuté sous Linux seulement |
| Code source complet | Inclus |
| Noyau mécanique commun au GUI et au pont Rust | Source C identique ; intégration Rust non compilée localement |
| Rust `aion-core` comme source autoritative | **Non** : ancien solveur Rust toujours expérimental |
| Rendu GPU photoréaliste | **Non** : GDI / projection 3D |
| Simulation atomique ou chimie quantique | **Non** |

## Hypothèses scientifiques

- Réseau de points massiques reliés par ressorts linéaires soumis à une force de rupture ; masse concentrée aux nœuds.
- Gravité uniforme, sol plan, collisions entre sphères et frottement simplifié lors des impacts sur le sol.
- Contact par impulsion instantanée et correction de pénétration ; collisions continues, friction de Coulomb complète et auto-contact après forte déformation non garantis.
- Les nœuds appartenant au même composant connexe de ressorts actifs ne se repoussent pas (choix macroscopique). Dès qu'ils sont déconnectés, ils peuvent entrer en collision.
- Pas semi-implicite d'au plus 2 ms ; schéma explicitement borné mais **pas inconditionnellement stable** pour des ressorts arbitrairement raides.
- Ruptures : énergie élastique du ressort rompant immédiatement convertie en chaleur. C'est une fermeture énergétique simplifiée et non un modèle de surface de fracture.
- Aucun atome, ion, molécule ou rayon lumineux microscopique n'est représenté ici.

## Validation locale

### Tests automatisés
- `cc -O2 tests/test_sim.c native/aion_sim.c -lm` : **99 assertions**, vérification de trajectoires sur 10 000 pas.
- `cc -O2 -DAION_TEST_HOST windows_viewer/windows_selftest.c native/aion_sim.c -lm` : autotest portable **PASS**.
- `pytest reference/test_reference.py` : **8/8 PASS**.
- Stress trois matériaux : 3 × 15 000 pas validés, plus essai AddressSanitizer/UndefinedBehaviorSanitizer sur les tests natifs.

### Exécutables
- `clang --target=x86_64-pc-windows-msvc` + `lld-link` : produit `AION_CORE_0_3.exe` (GUI PE32+) et `AION_CORE_0_3_TEST.exe` (console PE32+).
- Dépendances dynamiques constatées : DLL Windows système (`KERNEL32`, `USER32`, `GDI32` pour le GUI ; `KERNEL32` pour le diagnostic).
- Aucune exécution sous Windows ni Windows VM/ Wine disponible.
- Aucun outil `cargo`/`rustc` accessible. Les tests Rust sont inscrits dans le dépôt mais ne sont pas déclarés exécutés.

## Jalons à venir, arrêt qualité

**0.3.1** : CI Windows qui compile, exécute `AION_CORE_0_3_TEST.exe` et `cargo test --workspace`; corriger tout défaut d'ABI avant une release stabilisée. Vérifier GUI sur un véritable Windows 10/11.

**0.4** : soit porter `native/aion_sim.c` intégralement en Rust et remplacer l'adaptateur C par une API dynamique, soit conserver le C comme bibliothèque backend interne mais documenter le choix. Ajouter un harness comparatif bit-à-bit ou avec tolérances scientifiquement justifiées.

**0.5+** : modèle des matériaux, stabilité et performances ; seulement ensuite backend de rendu GPU (wgpu ou Direct3D/Vulkan), prototypage d'organismes incarnés et expérimentation de comportements émergents.
