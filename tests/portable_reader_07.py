"""Independent AION7 binary format validator (no Python bindings to C)."""
from __future__ import annotations
from pathlib import Path
import math
import struct
import unittest

MASK = 0xFFFFFFFFFFFFFFFF


def fnv64(data: bytes) -> int:
    h = 1469598103934665603
    for b in data:
        h = ((h ^ b) * 1099511628211) & MASK
    return h


def parse(data: bytes) -> dict:
    if len(data) < 132 or len(data) > 113796:
        raise ValueError("invalid size")
    if data[:8] != b"AIONS007":
        raise ValueError("bad magic")
    version, size, h, marker, reserved = struct.unpack_from("<IIQII", data, 8)
    if (version, size, marker, reserved) != (7, len(data), 0x01020304, 0):
        raise ValueError("bad header")
    if h != fnv64(data[32:]):
        raise ValueError("bad checksum")
    n, e, fractures, contacts, mode, rejected, plastic, fatigued, floor = struct.unpack_from('<9i', data, 32)
    if not (0 <= n <= 128 and 0 <= e <= 1024 and 0 <= mode <= 2 and floor in (0, 1)):
        raise ValueError("bad counts")
    if len(data) != 132 + n * 88 + e * 100:
        raise ValueError("bad payload size")
    floats = struct.unpack_from('<8d', data, 68)
    if not all(math.isfinite(x) for x in floats):
        raise ValueError("nonfinite metadata")
    off = 132
    nodes = []
    for _ in range(n):
        vals = struct.unpack_from('<10d2i', data, off)
        if not all(math.isfinite(v) for v in vals[:10]) or vals[7] <= 0 or vals[6] <= 0:
            raise ValueError("invalid node")
        nodes.append(vals)
        off += 88
    edges = []
    for _ in range(e):
        vals = struct.unpack_from('<3i11d', data, off)
        if not (0 <= vals[0] < n and 0 <= vals[1] < n) or not all(math.isfinite(v) for v in vals[3:]):
            raise ValueError("invalid edge")
        edges.append(vals)
        off += 100
    return {"nodes": nodes, "edges": edges, "time": floats[0], "mode": mode,
            "fractures": fractures, "contacts": contacts}


class PortableFormatTests(unittest.TestCase):
    fixture = Path(__file__).with_name('fixtures') / 'sample_v7.aion7'

    def test_fixture(self):
        state = parse(self.fixture.read_bytes())
        self.assertEqual(len(state['nodes']), 73)
        self.assertEqual(len(state['edges']), 404)
        self.assertEqual(state['mode'], 1)
        self.assertAlmostEqual(state['time'], 1.05)
        self.assertTrue(all(node[7] > 0 for node in state['nodes']))

    def test_truncated(self):
        payload = self.fixture.read_bytes()
        for n in (0, 32, 132, len(payload)-1):
            with self.assertRaises(ValueError):
                parse(payload[:n])

    def test_modified(self):
        payload = bytearray(self.fixture.read_bytes())
        payload[512] ^= 1
        with self.assertRaises(ValueError):
            parse(payload)

    def test_lengths(self):
        raw = self.fixture.read_bytes()
        state = parse(raw)
        self.assertEqual(len(raw), 132+len(state['nodes'])*88+len(state['edges'])*100)

if __name__ == '__main__':
    unittest.main()
