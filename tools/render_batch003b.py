#!/usr/bin/env python3
"""render_batch003b.py — kick-voicing addendum: center strikes, sub fundamentals.

Batch 002 verdict: f28 'does not sound like 28hz... loudest freq is 80hz'.
Cause: off-center strike (p0.35) feeds the (2,n)/(1,2) mode cloud. Fix:
center strike + center listening null the even modes and hand dominance
to the fundamental. Also honors 'you could go lower' (f22) and softens
glide on nothing -- glide depth per Batch 002 kept at 9 st here, subjects
are all sub-fundamental.

Run: nix develop -c python3 tools/render_batch003b.py
"""
import os, sys
import soundfile as sf
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-003b")
os.makedirs(OUT, exist_ok=True)

BASE = dict(arch="membrane", exciter="mallet", position=1.0, listen_pos=1.0,
            stiffness=0.6, t60_base=1.5, tilt=2.2, glide_st=9.0)
JOBS = []
for f0 in (22.0, 28.0):
    for vel in (0.65, 0.95):
        JOBS.append((dict(BASE, f0=f0, velocity=vel),
                     f"b003b_membrane_f{f0:g}_pcenter_v{vel:g}_glide9.wav"))
    JOBS.append((dict(BASE, f0=f0, velocity=0.95, glide_st=0.0),
                 f"b003b_membrane_f{f0:g}_pcenter_v0.95_glideOFF.wav"))
# the old voicing at f28 for direct A/B
JOBS.append((dict(BASE, f0=28.0, velocity=0.95, position=0.35, listen_pos=0.31),
             "b003b_membrane_f28_pOLD035_v0.95_glide9.wav"))

for params, name in JOBS:
    x = render_hit(**params)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    print(f"  {name:52s} {len(x)/SR:5.2f}s")
print(f"\n{len(JOBS)} renders -> {os.path.abspath(OUT)}")
