# AION / PRIMORDIA CORE — prochaine mission

## Jalon validé : exécution Rust native

La première chaîne native Rust a été validée par GitHub Actions le 14 octobre 2026.

- Linux : tests du workspace complet, puis construction et vérification de la CLI Rust.
- Windows : tests des modules purement Rust, construction de la CLI Windows x64, exécution des autotests, sauvegarde/rejeu AION7 et comparaison de 99 mesures avec le modèle C.
- Workflow validé : https://github.com/commandant-x/AION/actions/runs/38558593186
- La compilation historique C/MSVC reste un problème distinct : elle n'entre pas dans le binaire `aion-rust-headless.exe`.

Le programme natif Rust est **une application en ligne de commande** et non un moteur graphique 3D.

## Objectif suivant : moteur visuel 100 % Rust

1. Coupler directement le solveur `aion-rust-sim` à une interface graphique Windows écrite en Rust.
2. Introduire un rendu 3D GPU de laboratoire, avec caméra libre et télémétrie.
3. Conserver le banc d'expériences reproductibles et la compatibilité AION7.
4. Tester l'application Windows sur runner CI, y compris démarrage non interactif, puis sur un poste physique.
5. Ne pas confondre moteur de matière mésoscopique et simulation atomique/quantique fidèle.

## Conditions de livraison

Chaque jalon comprend un `.exe` Windows, les sources, les tests et un rapport de validation. Ne jamais déclarer réussi un test qui n'a pas été exécuté.
