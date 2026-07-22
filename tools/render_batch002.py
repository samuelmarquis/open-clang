#!/usr/bin/env python3
"""render_batch002.py — Batch 002: the glide, in kick country.

Run: nix develop -c python3 tools/render_batch002.py
"""

import os
import sys
import soundfile as sf

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR  # noqa: E402

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-002")
os.makedirs(OUT, exist_ok=True)

MEM = dict(arch="membrane", exciter="mallet", position=0.35, stiffness=0.55,
           t60_base=1.3, tilt=1.8)
JOBS = []
# velocity ladder, glide on, three sub fundamentals
for f0 in (28.0, 36.0, 45.0):
    for vel in (0.35, 0.65, 0.95):
        JOBS.append((dict(MEM, f0=f0, velocity=vel, glide_st=9.0),
                     f"b002_membrane_f{f0:g}_v{vel:g}_glide9.wav"))
# glide off references at full velocity
for f0 in (28.0, 36.0, 45.0):
    JOBS.append((dict(MEM, f0=f0, velocity=0.95, glide_st=0.0),
                 f"b002_membrane_f{f0:g}_v0.95_glideOFF.wav"))
# the imposter: uniform 909-style pitch env, velocity-independent
JOBS.append((dict(MEM, f0=36.0, velocity=0.95, glide_st=9.0, glide_fake=True),
             "b002_membrane_f36_v0.95_glideFAKE.wav"))
# subby plate clang
for gl, tag in ((0.0, "glideOFF"), (7.0, "glide7")):
    JOBS.append((dict(arch="plate", f0=50.0, exciter="mallet", velocity=0.95,
                      position=0.35, stiffness=0.7, t60_base=2.0, tilt=1.0,
                      glide_st=gl),
                 f"b002_plate_f50_v0.95_{tag}.wav"))

for params, name in JOBS:
    x = render_hit(**params)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    print(f"  {name:44s} {len(x)/SR:5.2f}s")

print(f"\n{len(JOBS)} renders -> {os.path.abspath(OUT)}")
