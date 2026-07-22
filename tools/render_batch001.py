#!/usr/bin/env python3
"""render_batch001.py — Batch 001: linear dignity check.

The question (docs/design/01-architecture.md, Lab plan): does the linear
modal core already sound like OBJECTS — membranes, plates, bars with
believable strike/position behavior — before any nonlinearity?

Run: nix develop -c python3 tools/render_batch001.py
"""

import os
import sys
import soundfile as sf

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR  # noqa: E402

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-001")
os.makedirs(OUT, exist_ok=True)

JOBS = []
for arch, f0, t60, tilt in [("membrane", 55.0, 0.9, 1.5),
                            ("membrane", 110.0, 0.7, 1.5),
                            ("plate", 90.0, 1.8, 0.9),
                            ("bar", 220.0, 2.2, 0.7)]:
    for vel in (0.35, 0.95):
        for pos in (0.12, 0.45):
            JOBS.append(dict(arch=arch, f0=f0, exciter="mallet", velocity=vel,
                             position=pos, t60_base=t60, tilt=tilt))
# two burst-excited outliers
JOBS.append(dict(arch="membrane", f0=55.0, exciter="burst", velocity=0.9,
                 position=0.3, t60_base=0.9, tilt=1.5))
JOBS.append(dict(arch="plate", f0=90.0, exciter="burst", velocity=0.9,
                 position=0.3, t60_base=1.8, tilt=0.9))

for j in JOBS:
    name = (f"b001_{j['arch']}_f{j['f0']:g}_{j['exciter']}"
            f"_v{j['velocity']:g}_p{j['position']:g}.wav")
    x = render_hit(**j)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    print(f"  {name:44s} {len(x)/SR:5.2f}s")

print(f"\n{len(JOBS)} renders -> {os.path.abspath(OUT)}")
