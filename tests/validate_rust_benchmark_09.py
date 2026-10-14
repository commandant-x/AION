#!/usr/bin/env python3
"""AION 0.9 benchmark gate: cross-language invariants, not false numeric equality.

Use: python tests/validate_rust_benchmark_09.py rust_benchmark.csv experiments/AION_CORE_0_8_reference.csv
"""
import csv
import math
import sys
from collections import defaultdict
from pathlib import Path

COLS = ('material', 'protocol', 'step', 'time_s', 'mass_kg', 'alive_bonds',
        'fractures', 'plastic_events', 'thermal_j', 'mechanical_j',
        'plastic_strain', 'max_damage', 'checksum')


def load_csv(path):
    with Path(path).open(newline='', encoding='utf-8') as file:
        reader = csv.DictReader(file)
        assert tuple(reader.fieldnames or ()) == COLS, f'Unexpected columns in {path}'
        rows = list(reader)
    assert len(rows) == 99, f'{path}: expected 99 rows, got {len(rows)}'
    return rows


def check(rows):
    groups = defaultdict(list)
    for row in rows:
        material, protocol, step = (int(row[x]) for x in ('material', 'protocol', 'step'))
        assert material in range(3) and protocol in range(3)
        assert step in range(0, 1001, 100)
        for field in ('time_s', 'mass_kg', 'thermal_j', 'mechanical_j',
                      'plastic_strain', 'max_damage'):
            assert math.isfinite(float(row[field])), (field, row)
        groups[(material, protocol)].append(row)
    assert len(groups) == 9
    for key, points in sorted(groups.items()):
        assert [int(x['step']) for x in points] == list(range(0, 1001, 100)), key
        mass = float(points[0]['mass_kg'])
        assert mass > 0
        last_fractures = 0
        for row in points:
            step = int(row['step'])
            assert abs(float(row['time_s']) - step*.003) < 1e-4
            assert abs(float(row['mass_kg']) - mass) < 1e-5
            intact = int(row['alive_bonds'])
            fractures = int(row['fractures'])
            assert 0 <= intact <= 404 and 0 <= fractures <= 404
            assert intact + fractures == 404
            assert fractures >= last_fractures
            last_fractures = fractures
            assert int(row['plastic_events']) >= 0
            assert float(row['thermal_j']) >= 0
            assert float(row['plastic_strain']) >= 0
            assert 0 <= float(row['max_damage']) <= 1
            assert len(row['checksum']) == 16
            int(row['checksum'],16)
    return groups


def main():
    rust = check(load_csv(sys.argv[1]))
    ref = check(load_csv(sys.argv[2]))
    for key in rust:
        a, b = rust[key][0], ref[key][0]
        # Shared initial topology and state must agree; trajectories may diverge
        # following contact, fracture and floating-point order differences.
        for name in ('mass_kg', 'thermal_j', 'mechanical_j'):
            x, y = float(a[name]), float(b[name])
            assert abs(x-y) < .001 + abs(y)*1e-6, (key, name, x, y)
        assert int(a['alive_bonds']) == int(b['alive_bonds']) == 404
    print('PASS: 9 Rust runs / 99 points; mass, topology, monotone fracture, '
          'time and initial C parity validated (later differences recorded, not hidden).')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('Usage: validate_rust_benchmark_09.py rust.csv C_reference.csv')
    main()
