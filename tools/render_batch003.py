#!/usr/bin/env python3
"""render_batch003.py — Batch 003: the rattle (NL2 contact satellites vs imposter).

Run: nix develop -c python3 tools/render_batch003.py
"""

import os
import sys
import soundfile as sf

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from nl2 import render_hit_td, SR  # noqa: E402

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-003")
os.makedirs(OUT, exist_ok=True)

WIRES = [dict(fs=1900.0, t60=0.10, seat=0.22, rest=0.15, level=1.0),
         dict(fs=2700.0, t60=0.08, seat=0.61, rest=0.22, level=0.8)]
LOOSE = [dict(fs=900.0, t60=0.15, seat=0.45, rest=0.55, level=1.0)]
TRASH = [dict(fs=1300.0, t60=0.12, seat=0.18, rest=0.30, level=1.0),
         dict(fs=2100.0, t60=0.10, seat=0.52, rest=0.45, level=0.9),
         dict(fs=3400.0, t60=0.07, seat=0.80, rest=0.20, level=0.7)]

JOBS = [
    # snare-land: membrane + wire pair, velocity pair (rattle should die with the body)
    (dict(arch="membrane", f0=110.0, velocity=0.5, t60_base=0.5, tilt=1.4,
          satellites=WIRES), "b003_membrane_f110_wires_v0.5.wav"),
    (dict(arch="membrane", f0=110.0, velocity=0.95, t60_base=0.5, tilt=1.4,
          satellites=WIRES), "b003_membrane_f110_wires_v0.95.wav"),
    # tight vs loose seating on the same object
    (dict(arch="membrane", f0=110.0, velocity=0.95, t60_base=0.5, tilt=1.4,
          satellites=[dict(WIRES[0], rest=0.05), dict(WIRES[1], rest=0.08)]),
     "b003_membrane_f110_wires-tight_v0.95.wav"),
    # kick-land: sub membrane + one loose knocker
    (dict(arch="membrane", f0=36.0, velocity=0.95, t60_base=1.1, tilt=1.8,
          satellites=LOOSE), "b003_membrane_f36_loose_v0.95.wav"),
    # trash plate
    (dict(arch="plate", f0=90.0, velocity=0.95, t60_base=1.6, tilt=1.0,
          satellites=TRASH), "b003_plate_f90_trash_v0.95.wav"),
    # the mangle test: bar + wires
    (dict(arch="bar", f0=220.0, velocity=0.95, t60_base=1.8, tilt=0.8,
          satellites=WIRES), "b003_bar_f220_wires_v0.95.wav"),
    # clean references
    (dict(arch="membrane", f0=110.0, velocity=0.95, t60_base=0.5, tilt=1.4,
          satellites=None), "b003_membrane_f110_CLEAN_v0.95.wav"),
    (dict(arch="bar", f0=220.0, velocity=0.95, t60_base=1.8, tilt=0.8,
          satellites=None), "b003_bar_f220_CLEAN_v0.95.wav"),
    # THE IMPOSTERS: surface-envelope-gated noise, no contact anywhere
    (dict(arch="membrane", f0=110.0, velocity=0.95, t60_base=0.5, tilt=1.4,
          satellites=None, imposter_noise=True),
     "b003_membrane_f110_IMPOSTER_v0.95.wav"),
    (dict(arch="bar", f0=220.0, velocity=0.95, t60_base=1.8, tilt=0.8,
          satellites=None, imposter_noise=True),
     "b003_bar_f220_IMPOSTER_v0.95.wav"),
]

for params, name in JOBS:
    x, contacts = render_hit_td(**params)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    print(f"  {name:44s} {len(x)/SR:5.2f}s  contacts={contacts}")

print(f"\n{len(JOBS)} renders -> {os.path.abspath(OUT)}")
