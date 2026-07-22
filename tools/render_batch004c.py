#!/usr/bin/env python3
"""render_batch004c.py — Batch 004c: the SIZE macro law.

Batch 004b discovery (Sam): nonlinear commotion reads as SMALLNESS —
velocity and deep transfer both shrank the object. Physics agrees: FvK
nonlinearity scales with (deflection/thickness)^2, so susceptibility
falls with size. The Size macro therefore co-scales:
  f0 ~ 1/size, mode density ~ up, T60 ~ up, cascade tau ~ up,
  and NONLINEAR DRIVE ~ velocity^2 / size^1.5   (the crucial term)
"""
import os, sys
import soundfile as sf
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-004c")
os.makedirs(OUT, exist_ok=True)

def size_patch(size, velocity):
    drive = min(1.0, (velocity ** 2) / (size ** 1.5))
    return dict(
        arch="plate", exciter="mallet", position=0.35, stiffness=0.7,
        velocity=velocity,
        f0=50.0 / size,
        n_axial=max(8, min(14, int(round(10 + 3 * size)))),
        t60_base=2.0 * (size ** 0.7),
        tilt=0.9,
        glide_st=7.0 * drive,
        cascade_amt=0.9 * drive,
        cascade_tau=0.05 * (size ** 1.3),
        cascade_conserve=True, cascade_split=5.0,
    )

JOBS = [
    # the size ladder at fixed velocity
    (0.5, 0.95, "b004c_plate_SIZE0.5_v0.95.wav"),
    (1.0, 0.95, "b004c_plate_SIZE1.0_v0.95.wav"),
    (2.0, 0.95, "b004c_plate_SIZE2.0_v0.95.wav"),
    # the velocity ladder at fixed size: does velocity now read as FORCE?
    (1.0, 0.4, "b004c_plate_SIZE1.0_v0.4.wav"),
    (1.0, 0.7, "b004c_plate_SIZE1.0_v0.7.wav"),
    # the gentle giant
    (2.0, 0.5, "b004c_plate_SIZE2.0_v0.5.wav"),
]
for size, vel, name in JOBS:
    p = size_patch(size, vel)
    x = render_hit(**p)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    print(f"  {name:36s} f0={p['f0']:5.1f}Hz drive={min(1.0,(vel**2)/(size**1.5)):4.2f} "
          f"tau={p['cascade_tau']*1000:5.1f}ms t60={p['t60_base']:4.2f}s {len(x)/SR:5.2f}s")
print(f"\n{len(JOBS)} renders -> {os.path.abspath(OUT)}")
