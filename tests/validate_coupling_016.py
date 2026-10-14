#!/usr/bin/env python3
"""Independent checks for PRIMORDIA 0.16 coupled voxel/hydro benchmark."""
import csv, math, sys
from pathlib import Path

def validate(path):
    with Path(path).open(newline='', encoding='utf-8') as handle:
        rows=list(csv.DictReader(handle))
    expected=['step','time_s','water_m3','cave_cells','withdrawn_m3','water_residual_m3','energy_residual_j','changed_cells','drained_m3','fingerprint']
    assert rows and list(rows[0])==expected, 'wrong schema'
    assert len(rows)==61, f'expected 61 samples, got {len(rows)}'
    last_withdraw=0.0
    for idx, row in enumerate(rows):
        assert int(row['step'])==idx, 'step sequence'
        assert abs(float(row['time_s'])-10*idx)<1e-7, 'simulation time'
        for k in expected[1:-1]:
            v=float(row[k]); assert math.isfinite(v), (idx,k)
        assert abs(float(row['water_residual_m3']))<1e-8, 'water conservation'
        assert abs(float(row['energy_residual_j']))<1e-3, 'energy accounting'
        withdrawn=float(row['withdrawn_m3'])
        assert withdrawn>=last_withdraw-1e-10
        last_withdraw=withdrawn
        assert len(row['fingerprint'])==16 and all(c in '0123456789abcdef' for c in row['fingerprint'])
    assert int(rows[0]['cave_cells'])==0
    assert int(rows[1]['cave_cells'])==1
    assert int(rows[30]['cave_cells'])==1
    assert int(rows[31]['cave_cells'])==0
    assert float(rows[2]['water_m3'])>float(rows[1]['water_m3'])+1.0
    assert float(rows[31]['withdrawn_m3'])>0.0, 'expected overflow drainage on infill'
    assert float(rows[31]['drained_m3'])>0.0
    print('PASS AION 0.16: 61 deterministic samples, excavated cavity, water injection, infill drainage, budget and checksum columns')
if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit('usage: validate_coupling_016.py <csv>')
    validate(sys.argv[1])