"""Independent ledger validation for the 61-step surface/hydro benchmark."""
import csv
import math
import sys

with open(sys.argv[1], newline="", encoding="utf-8") as stream:
    rows = list(csv.DictReader(stream))
assert len(rows) == 61, f"expected 61 samples, got {len(rows)}"
first = {k: float(v) for k, v in rows[0].items()}
initial_hydro = first["hydro_m3"]
last_infiltration = -1.0
for step, row in enumerate(rows):
    v = {k: float(x) for k, x in row.items()}
    assert all(math.isfinite(x) for x in v.values()), (step, v)
    assert v["step"] == step and abs(v["time_s"] - 10 * step) < 1e-9
    assert v["surface_m3"] >= -1e-10 and v["hydro_m3"] >= -1e-10
    assert abs(v["supplied_m3"] - 0.7) < 1e-9
    assert abs(v["rain_m3"] - step * 0.0016) < 1e-9
    assert v["infiltrated_m3"] + 1e-10 >= last_infiltration
    last_infiltration = v["infiltrated_m3"]
    assert abs(v["surface_m3"] + v["infiltrated_m3"]
               - v["supplied_m3"] - v["rain_m3"]) < 1e-8
    assert abs(v["hydro_m3"] - initial_hydro - v["infiltrated_m3"]) < 1e-8
    assert abs(v["surface_residual_m3"]) < 1e-8
    assert abs(v["hydro_residual_m3"]) < 1e-8
    assert abs(v["hydro_energy_residual_j"]) < 0.01
assert last_infiltration > 0.01, "no measurable surface-to-groundwater coupling"
print(f"PASS 61 steps; infiltration={last_infiltration:.6f} m3; sealed budgets")
