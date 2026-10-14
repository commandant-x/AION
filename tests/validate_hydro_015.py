#!/usr/bin/env python3
"""Independent, standard-library verification of PRIMORDIA groundwater CSV.
Does not merely trust solver's checksum: checks dimensions, external water/energy
ledgers, thermodynamic bounds, monotone rainfall, time and repeatability format.
"""
import csv
import math
import pathlib
import re
import sys


def validate(path: pathlib.Path) -> None:
    with path.open(newline='', encoding='utf-8') as stream:
        rows = list(csv.DictReader(stream))
    assert len(rows) == 61, f"expected 61 samples, got {len(rows)}"
    required = {'time_s', 'water_m3', 'injected_m3', 'withdrawn_m3',
                'water_residual_m3', 'energy_j', 'energy_residual_j',
                'mean_temp_k', 'fingerprint'}
    assert set(rows[0]) == required, f"unexpected headers: {list(rows[0])}"
    baseline_mass = float(rows[0]['water_m3'])
    baseline_energy = float(rows[0]['energy_j'])
    before_added = 0.0
    hashes = set()
    for i, row in enumerate(rows):
        sample = {k: float(v) for k, v in row.items() if k != 'fingerprint'}
        assert all(math.isfinite(v) for v in sample.values()), f"nonfinite state at {i}"
        assert abs(sample['time_s'] - 20*i) < 1e-9, f"time step {i}"
        assert sample['water_m3'] >= 0.0 and sample['energy_j'] >= 0.0
        assert sample['injected_m3'] >= before_added-1e-10, f"rain ledger regressed at {i}"
        assert sample['withdrawn_m3'] == 0.0, f"unexpected withdrawal at {i}"
        assert 250 <= sample['mean_temp_k'] <= 400, f"temperature out of bounds at {i}"
        assert abs(sample['water_residual_m3']) < 1e-8, f"mass residual at {i}"
        assert abs(sample['energy_residual_j']) < 1.0, f"energy residual at {i}"
        assert abs(sample['water_m3']-(baseline_mass+sample['injected_m3'])) < 1e-8, f"mass budget {i}"
        assert re.fullmatch('[0-9a-f]{16}', row['fingerprint']), f"bad replay fingerprint {i}"
        hashes.add(row['fingerprint'])
        before_added = sample['injected_m3']
    assert rows[0]['fingerprint'] != rows[-1]['fingerprint'], 'simulation did not evolve'
    assert len(hashes) == len(rows), 'duplicate state fingerprints'
    assert before_added > 0, 'no infiltrating water injected'
    assert baseline_energy > 0
    print(f'PASS: {len(rows)} independent hydrothermal samples; water in = {before_added:.9f} m3; '
          f'mass conserved to <1e-8 m3; energy conserved to <1 J')


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('Usage: validate_hydro_015.py <csv>')
    validate(pathlib.Path(sys.argv[1]))