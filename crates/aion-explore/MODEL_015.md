# PRIMORDIA CORE 0.15 — Laboratoire hydrothermique conservatif

**But :** dépasser les aspects « jeu » et construire des modèles physiques explicitement testés, en unités SI et avec limites scientifiques identifiées. Le Goal PRIMORDIA reste ouvert.

## Échelle / discretisation

- Volume de contrôle **2 × 2 × 2 m = 8 m³** aligné sur les voxels 0.14.
- Domaine fini de référence **8 × 6 × 8 = 384 cellules**, parois externes fermées ; jusqu'à 32³ cellules par patch.
- Classification lithologique issue de `VoxelWorld::material()`, classes de sable, terre, argile, gravier, tourbe, calcaire, granite, basalte, et vide d'excavation.
- État **continu local** par cellule : volume d'eau [m³], énergie sensible [J], température [K], saturation [0..1], capacité de pore [m³].
- Paramètres de matériaux : porosité [-], conductivité hydraulique saturée [m/s], conduction thermique [W/(m·K)], capacité thermique volumique [J/(m³·K)], échelle de succion capillaire [m]. **Ces paramètres sont des exemples d'ordres de grandeur, non des mesures ni des constantes universelles**.

## Équations et bilans

- Échange interne approché par une forme de **Darcy** : `Q = K_face A/L (H_i − H_j)` [m³/s], avec `A = 4 m²`, `L = 2 m`, `K_face` moyenne harmonique des conductivités effectives.
- Charge hydraulique `H = z − ψ(1−S)` [m]. Le terme de rétention capillaire est un **substitut linéaire**, pas une vraie loi de van Genuchten ni l'équation complète de Richards.
- Une loi de conductivité relative bornée `K_eff = Ksat (0.02 + 0.98 S³)` sert uniquement de fermeture simplifiée en zone partiellement saturée.
- Les transferts sont bornés pour ne jamais excéder l'eau disponible chez le donneur ou le volume de pore libre du receveur. L'extérieur est fermé, sauf les injections et prélèvements explicites.
- Transport d'énergie par convection avec l'eau `ΔE = ρ_eau c_p eau ΔV T_donneur` et par conduction de Fourier `ΔE = λ_face A/L (T_i − T_j) Δt`. Les deux opérations sont des transferts antisymétriques conservatifs, la conduction est bornée pour éviter l'inversion non physique du gradient sur un pas.
- Énergie stockée autour du zéro absolu, capacité thermique des solides, de l'eau et d'une fraction de gaz simplifiée. **Aucune chaleur latente, transition de phase, réaction chimique, advection de sédiment, nappe réelle ni couplage avec l'océan.**

Bilan vérifié à chaque expérience : `eau_finale = eau_initiale + injection − prélèvement`, `E_finale = E_initiale + entrée − sortie`. Les erreurs dues aux arrondis flottants sont quantifiées. Étape temporelle bornée à 60 s ; aucune prétention à la stabilité globale, à la convergence numérique ou à une validation terrain.

## Reproductibilité

- Nouveau module Rust `crates/aion-world/src/hydrothermal.rs` et nouveaux tests unitaires : conservation avec domaine fermé, entrées/sorties explicites, flux gravitaire, contraste de conductivité gravier/argile, conduction thermique, séquence reproductible, cellule creusée, contrôle des paramètres et indices, sauvegarde binaire.
- Format de sauvegarde **AIONH015**, indépendant des sauvegardes AIONV014/AIONED13. Petit-en-boutiste, contrôle d'intégrité FNV-1a, graine et géométrie validées, bilan physique vérifié au chargement.
- Utilitaire sans interface GPU `aion-explore.exe --hydro-benchmark reference.csv` : 61 échantillons déterministes sur 1 200 s, injection d'eau explicitement comptabilisée et trace CSV. L'exécutable reste une vraie application graphique lorsqu'il est lancé sans arguments.
- Validateur Python **indépendant** `tests/validate_hydro_015.py` : conservation eau, énergie, unités, monotonie des apports, absence de NaN, 61 horodatages et empreintes de rejeu.
- CI GitHub Actions Windows/Linux : tests, programme release, laboratoire CSV + validateur, et smoke test de création de fenêtre sous Windows.

## Limitations et prochaines tâches du Goal

Ce sous-système constitue un **laboratoire local couplé au type de roche des voxels** au moment de l'initialisation ; il n'est **pas** encore propagé en temps réel sur les milliers de chunks ni couplé dynamiquement aux creusements réalisés pendant l'exploration. Le rendu de surface d'eau de l'océan reste un calque indépendant. Les forces de pression sur la roche, la migration de solutés, les fluides dans les fractures, la conductivité anisotrope et les bilans multi-échelles doivent être ajoutés et confrontés à des résultats de référence, sans faire passer des simplifications pour une Terre réelle.