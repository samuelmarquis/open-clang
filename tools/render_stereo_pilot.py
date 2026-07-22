#!/usr/bin/env python3
"""render_stereo_pilot.py — x-pilot: 3D lowend via dual listening positions.

Out-of-sequence taste preview (not a mechanism batch): L/R = the same
strike heard at two positions on the same object. Sub coherent, uppers
decorrelated. Full stereo program (satellite panning, detuned width
voices per research 05 R3) lands with the bracing/space batches.
"""
import os, sys
import numpy as np, soundfile as sf
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "x-stereo-pilot")
os.makedirs(OUT, exist_ok=True)

def stereo(name, lp_l, lp_r, **kw):
    L = render_hit(listen_pos=lp_l, **kw)
    R = render_hit(listen_pos=lp_r, **kw)
    n = min(len(L), len(R))
    x = np.stack([L[:n], R[:n]], axis=1)
    x *= 10 ** (-3/20) / np.max(np.abs(x))
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    c = float(np.corrcoef(L[:n], R[:n])[0, 1])
    print(f"  {name:52s} LR corr={c:5.2f}")

K = dict(arch="membrane", f0=28.0, exciter="mallet", velocity=0.95,
         position=1.0, stiffness=0.6, t60_base=1.5, tilt=2.2, glide_st=9.0)
stereo("x_membrane_f28_kick_stereo-near.wav", 0.80, 0.95, **K)
stereo("x_membrane_f28_kick_stereo-wide.wav", 0.35, 0.95, **K)
P = dict(arch="plate", f0=50.0, exciter="mallet", velocity=0.95,
         position=0.35, stiffness=0.7, t60_base=2.0, tilt=1.0, glide_st=7.0)
stereo("x_plate_f50_clang_stereo.wav", 0.18, 0.62, **P)
stereo("x_plate_f50_clang_stereo-extreme.wav", 0.05, 0.92, **P)
