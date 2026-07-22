#!/usr/bin/env python3
"""render_batch004.py — Batch 004: the cascade (NL3 clang builder vs static-bright)."""
import os, sys
import soundfile as sf
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-004")
os.makedirs(OUT, exist_ok=True)

P50 = dict(arch="plate", f0=50.0, exciter="mallet", position=0.35, stiffness=0.7,
           t60_base=2.0, tilt=0.9, glide_st=7.0, n_axial=12, cascade_split=5.0)
JOBS = [
    (dict(P50, velocity=0.5, cascade_amt=0.7, cascade_tau=0.05),
     "b004_plate_f50_casc0.7_tau50_v0.5.wav"),
    (dict(P50, velocity=0.95, cascade_amt=0.7, cascade_tau=0.05),
     "b004_plate_f50_casc0.7_tau50_v0.95.wav"),
    (dict(P50, velocity=0.95, cascade_amt=0.0),
     "b004_plate_f50_cascOFF_v0.95.wav"),
    (dict(P50, velocity=0.95, cascade_amt=0.7, cascade_tau=0.05, cascade_static=True),
     "b004_plate_f50_cascSTATIC_v0.95.wav"),
    # buildup-time-scales-with-size percept
    (dict(P50, velocity=0.95, cascade_amt=0.7, cascade_tau=0.02),
     "b004_plate_f50_casc0.7_tau20_v0.95.wav"),
    (dict(P50, velocity=0.95, cascade_amt=0.7, cascade_tau=0.10),
     "b004_plate_f50_casc0.7_tau100_v0.95.wav"),
    # other bodies
    (dict(arch="plate", f0=90.0, exciter="mallet", velocity=0.95, position=0.35,
          stiffness=0.7, t60_base=1.6, tilt=0.9, glide_st=5.0, n_axial=10,
          cascade_amt=0.7, cascade_tau=0.03),
     "b004_plate_f90_casc0.7_tau30_v0.95.wav"),
    (dict(arch="bar", f0=220.0, exciter="mallet", velocity=0.95, position=0.35,
          stiffness=0.7, t60_base=1.8, tilt=0.7, glide_st=0.0,
          cascade_amt=0.7, cascade_tau=0.04, cascade_split=3.0),
     "b004_bar_f220_casc0.7_tau40_v0.95.wav"),
    (dict(arch="membrane", f0=110.0, exciter="mallet", velocity=0.95, position=0.35,
          stiffness=0.6, t60_base=0.6, tilt=1.2, glide_st=4.0, n_axial=10,
          cascade_amt=0.5, cascade_tau=0.03),
     "b004_membrane_f110_casc0.5_tau30_v0.95.wav"),
]
for params, name in JOBS:
    x = render_hit(**params)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    print(f"  {name:44s} {len(x)/SR:5.2f}s")
print(f"\n{len(JOBS)} renders -> {os.path.abspath(OUT)}")
