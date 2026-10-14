"""Independent CSV validation: no C structs or shared serializer involved."""
import csv
import decimal
import hashlib
import pathlib
import sys

COLUMNS = ["material", "protocol", "step", "time_s", "mass_kg", "alive_bonds", "fractures", "plastic_events", "thermal_j", "mechanical_j", "plastic_strain", "max_damage", "checksum"]


def validate(path):
    raw = pathlib.Path(path).read_bytes()
    assert raw.endswith(b"\r\n") and b"\r\n" in raw
    assert all(byte >= 32 or byte in (10, 13) for byte in raw)
    with pathlib.Path(path).open(newline="") as source:
        reader = csv.DictReader(source)
        assert reader.fieldnames == COLUMNS
        rows = list(reader)
    assert len(rows) == 99, len(rows)
    signatures = {}
    for m in range(3):
        for p in range(3):
            trial = [r for r in rows if (int(r["material"]), int(r["protocol"])) == (m, p)]
            assert len(trial) == 11
            starting_mass = decimal.Decimal(trial[0]["mass_kg"])
            previous_fractures = 0
            for i, item in enumerate(trial):
                assert int(item["step"]) == i * 100
                assert abs(decimal.Decimal(item["time_s"]) - decimal.Decimal(i) * decimal.Decimal("0.3")) < decimal.Decimal("0.00001")
                assert decimal.Decimal(item["mass_kg"]) == starting_mass
                fractures = int(item["fractures"])
                assert fractures >= previous_fractures
                assert int(item["alive_bonds"]) + fractures == 404
                assert decimal.Decimal(item["thermal_j"]) > 0
                assert 0 <= decimal.Decimal(item["max_damage"]) <= 1
                assert len(item["checksum"]) == 16 and all(c in "0123456789abcdef" for c in item["checksum"])
                previous_fractures = fractures
            signatures[(m, p)] = trial[-1]["checksum"]
    assert len(set(signatures.values())) >= 6
    print(f"CSV verification PASS: {len(rows)} data points, SHA256 {hashlib.sha256(raw).hexdigest()}")


if __name__ == "__main__":
    validate(sys.argv[1])
