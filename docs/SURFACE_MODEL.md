# AION — prototype d'hydrologie de surface (travail expérimental)

Le crate `aion-surface` représente une grille 2D de cellules de côté 2 m (ou autre pas pour le ruissellement seul), chacune avec une cote de fond en mètres et un stock d'eau en m³. Les frontières sont fermées. La pluie uniforme en m/s constitue un apport externe comptabilisé.

Aux faces, le flux de ruissellement est une **approximation d'onde diffusive de Manning** : `Q = largeur × h_amont^(5/3) × sqrt(|Δsurface_libre|/distance) / n` en m³/s. Les transferts sont évalués simultanément, limités par le volume du donneur et par l'égalisation de surface libre de chaque paire. Il n'y a **pas** de conservation de quantité de mouvement ni de solveur de Saint-Venant.

Le couplage `infiltrate_into` transfère l'eau de surface à la couche supérieure d'un patch `HydroThermal` aligné (cellules 2 m). Le débit est plafonné par `K_non_saturé × aire`, le stock de surface et la capacité de pores restante. Le volume et l'enthalpie sont injectés dans les bilans du sous-sol à la température imposée. **Ce n'est pas une résolution de Richards** : pas d'exfiltration, de charge capillaire interfaciale, de nappe libre résolue ni d'évolution thermique de l'eau en surface.

Les coefficients Manning et les propriétés hydrauliques restent **non calibrés**. Le relief 3D GPU et les excavations en direct ne pilotent pas encore cette grille ; pas d'érosion, de transport de sédiments, de rivière, d'évaporation ni de végétation dynamique.

Validation : tests Rust de conservation, positivité, raffinement temporel et transfert d'enthalpie ; expérience déterministe de 61 points avec validation indépendante Python, exécutée par la CI Linux/Windows. Les résultats effectifs sont ceux des workflows, pas des hypothèses.
