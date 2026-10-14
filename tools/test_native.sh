#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p bin
cc -O2 -Wall -Wextra -std=c99 tests/test_sim.c native/aion_sim.c -lm -o bin/aion_physics_checks_linux
cc -O2 -Wall -Wextra -Werror -std=c11 tests/material_04.c native/aion_sim.c -lm -o bin/aion_material_checks_linux
bin/aion_material_checks_linux
bin/aion_physics_checks_linux
cc -O2 -Wall -Wextra -Werror -std=c11 tests/material_04.c native/aion_sim.c -lm -o bin/aion_material_checks_linux
bin/aion_material_checks_linux
cc -O2 -std=c99 -DAION_TEST_HOST windows_viewer/windows_selftest.c native/aion_sim.c native/checkpoint_portable.c native/experiment.c -lm -o bin/aion_windows_checks_linux
bin/aion_windows_checks_linux
cc -O2 -Wall -Wextra -std=c99 tests/stress.c native/aion_sim.c -lm -o bin/aion_stress_checks_linux
bin/aion_stress_checks_linux
python3 -m pytest reference/test_reference.py -q

cc -O2 -Wall -Wextra -Werror -std=c11 tests/checkpoint_06.c native/aion_sim.c -lm -o bin/aion_checkpoint_checks_linux
bin/aion_checkpoint_checks_linux

cc -O2 -Wall -Wextra -Werror -std=c11 tests/portable_07.c native/checkpoint_portable.c native/aion_sim.c -lm -o bin/aion_portable_checks_linux
bin/aion_portable_checks_linux

# AION CORE 0.8: reproducible batch experiments and CSV protocol.
cc -O2 -Wall -Wextra -Werror -std=c11 tests/experiment_08.c native/experiment.c native/aion_sim.c -lm -o bin/aion_experiment_checks_linux
bin/aion_experiment_checks_linux
python3 tests/validate_experiments_08.py experiments/AION_CORE_0_8_reference.csv
