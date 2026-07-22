#!/usr/bin/env python3
"""render_batch004b.py — Batch 004b: energy-conserving cascade (does tau become SIZE?)."""
import os, sys
import soundfile as sf
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-004b")
os.makedirs(OUT, exist_ok=True)

P = dict(arch="plate", f0=50.0, exciter="mallet", position=0.35, stiffness=0.7,
         t60_base=2.0, tilt=0.9, glide_st=7.0, n_axial=12, cascade_split=5.0,
         cascade_amt=0.7, cascade_conserve=True)
JOBS = [
    (dict(P, velocity=0.95, cascade_tau=0.02), "b004b_plate_f50_CONSERVE_tau20_v0.95.wav"),
    (dict(P, velocity=0.95, cascade_tau=0.05), "b004b_plate_f50_CONSERVE_tau50_v0.95.wav"),
    (dict(P, velocity=0.95, cascade_tau=0.10), "b004b_plate_f50_CONSERVE_tau100_v0.95.wav"),
    (dict(P, velocity=0.5, cascade_tau=0.10), "b004b_plate_f50_CONSERVE_tau100_v0.5.wav"),
    # the attack-balance control (Batch 004: 'all three useful' -> one knob)
    (dict(P, velocity=0.95, cascade_tau=0.05, cascade_attack=0.5),
     "b004b_plate_f50_CONSERVE_tau50_attack0.5_v0.95.wav"),
]
for params, name in JOBS:
    x = render_hit(**params)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    print(f"  {name:52s} {len(x)/SR:5.2f}s")
print(f"\n{len(JOBS)} renders -> {os.path.abspath(OUT)}  (non-conserve refs live in batch-004)")
