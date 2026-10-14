# AION CORE 0.9 — Rust delivery gate

- Ajout du banc de mesures **à exécuter en Rust** : 9 mondes/protocoles et 99 observations.
- Ajout d'un validateur Python des invariants et des états initiaux en regard du moteur C.
- Workflow CI Windows et Linux compilant, testant puis exécutant le solveur Rust ; publication conditionnelle des artefacts.
- Script PowerShell pour compiler et tester le vrai Rust directement sur un PC Windows.
- Production d'un visualiseur et d'un diagnostic Windows **basés sur la référence C**.
- Le blocage du compilateur dans le conteneur persiste ; le vrai `.exe` Rust doit être produit dans la CI ou sur le PC.
