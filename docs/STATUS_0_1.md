# État auditable — AION CORE 0.1

Date de création : 2026-10-14.

| Domaine | Livré en code | Test numérique de référence ici | Test Rust natif ici | Qualité production |
|---|---|---|---|---|
| Algèbre vectorielle SI | oui | indirect | indisponible | expérimental |
| RNG reproductible | oui | oui | indisponible | expérimental |
| Masse, collisions sphériques et ressorts cassables | oui | oui, lois élémentaires | indisponible | insuffisant pour le jeu |
| Transfert thermique | oui | oui | indisponible | modèle simplifié |
| Réactions stœchiométriques validées | oui | oui | indisponible | inventaire chimique minimal |
| Charges électriques classiques régularisées | oui | oui | indisponible | pas de physique quantique |
| Propagation d'onde scalaire 2D | oui | oui | indisponible | ni onde électromagnétique 3D ni son complet |
| Petit cerveau à renforcement, initialisé aléatoirement | oui | oui | indisponible | pas incarné, pas d'intelligence humaine |
| Noyau multirésolution couplé | non | non | non | prévu |
| Sols, biomes, hydrologie et animaux | non | non | non | prévu |
| Vue subjective GPU / éclairage / animation | non | non | non | prévu |
| Apprentissage culturel et naissance du langage | non | non | non | prévu |
| Simulation de l'évolution humaine | non | non | non | objectif exploratoire sans garantie |

8 tests Python indépendants réussis lors de la création. Aucun résultat de `cargo test` n'est revendiqué : le compilateur Rust était absent et le réseau inaccessible. Ce document doit être mis à jour après le premier build sur une machine Rust.
