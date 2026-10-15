# Validation du prototype de ruissellement et d'infiltration — 15 octobre 2026

**Statut : expérimentation scientifique validée en CI, pas une version finale ni une validation sur le PC utilisateur.** Commit testé : `6e7735e85a449837bab5416125afac7abe421db1`.

## Exécutions effectives

- Workflow World Model Windows + Linux : https://github.com/commandant-x/AION/actions/runs/38695658772 — **succès sur les deux OS**.
- Chaque OS : **6 tests `aion-surface` et 31 tests `aion-world` réussis**, 0 échec ; construction release du programme de référence et du nouvel exécutable de banc d'essai.
- Validation Python indépendante : **61 observations (t = 0 à 600 s)**, succès Windows et Linux. Infiltration cumulée : **0,796000 m³** ; apport initial 0,700000 m³ et pluie cumulée 0,096000 m³. Le cas de référence est à frontières fermées ; l'intégralité de l'eau apportée a pénétré dans le patch au terme des 600 s.
- Le validateur vérifie à chaque observation : résidus de masse de surface et du sous-sol < **1e-8 m³**, résidu d'énergie du sous-sol < **0,01 J**, positivité, monotonie de l'infiltration et correspondance des deux bilans. Ce sont des **seuils de validation**, pas des mesures d'erreur de discrétisation.
- Artefact Windows : `AION-SURFACE-EXPERIMENT-WIN64` (ID 11954103219), contenant le **véritable `aion-surface-benchmark.exe`**, le code Rust, les tests, le CSV de 61 points, le script indépendant et `surface-validation.txt`. Il est disponible sur la page du workflow ci-dessus.

## Limites et suites

L'onde diffusive de Manning est un modèle d'écoulement **approché** ; l'infiltration par `K_non_saturé × aire` n'est pas Richards. Pas de quantité de mouvement, de sédiments, d'exfiltration, de transfert thermique à la surface, de calibration de terrain ou de synchronisation aux excavations GPU. Les essais de raffinement temporel ne démontrent pas une convergence spatiale ni la validité physique à l'échelle du bassin. La prochaine étape est l'alignement causal avec la géométrie voxel dynamique, puis les tests de raffinement spatial et de conditions aux limites ouvertes.

**GOAL.md demeure ouvert.** Aucun organisme incarné, agent apprenant ou monde géophysique complet n'est validé par ce jalon.
