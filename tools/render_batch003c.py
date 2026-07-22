#!/usr/bin/env python3
"""render_batch003c.py — out-curve kicks + dust controls (Batch 003/003b actions)."""
import os, sys
import soundfile as sf
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lab"))
from engine import render_hit, SR
from nl2 import render_hit_td

OUT = os.path.join(os.path.dirname(__file__), "..", "out", "batch-003c")
os.makedirs(OUT, exist_ok=True)

# kicks with the output modal-gain curve: fundamental-dominant voicing
K = dict(arch="membrane", exciter="mallet", velocity=0.95, position=1.0,
         listen_pos=1.0, stiffness=0.6, t60_base=1.5, tilt=2.2, glide_st=9.0)
for f0 in (22.0, 28.0):
    x = render_hit(f0=f0, out_tilt_db_oct=-8.0, **K)
    n = f"b003c_membrane_f{f0:g}_pcenter_outtilt-8_v0.95_glide9.wav"
    sf.write(os.path.join(OUT, n), x, SR, subtype="PCM_24"); print(f"  {n}")

WIRES = [dict(fs=1900.0, t60=0.10, seat=0.22, rest=0.15, level=1.0),
         dict(fs=2700.0, t60=0.08, seat=0.61, rest=0.22, level=0.8)]
DUSTS = [("dustA-thr40-fol1", dict(level=0.45, thr_db=-40, follow=1.0)),
         ("dustB-thr25-fol2", dict(level=0.6, thr_db=-25, follow=2.0))]
for tag, d in DUSTS:
    x, c = render_hit_td(arch="membrane", f0=110.0, velocity=0.95, t60_base=0.5,
                         tilt=1.4, satellites=WIRES, dust=d)
    n = f"b003c_membrane_f110_wires+{tag}_v0.95.wav"
    sf.write(os.path.join(OUT, n), x, SR, subtype="PCM_24"); print(f"  {n} contacts={c}")
x, c = render_hit_td(arch="membrane", f0=110.0, velocity=0.95, t60_base=0.5,
                     tilt=1.4, satellites=None,
                     dust=dict(level=0.6, thr_db=-35, follow=1.5))
n = "b003c_membrane_f110_dustonly-thr35-fol1.5_v0.95.wav"
sf.write(os.path.join(OUT, n), x, SR, subtype="PCM_24"); print(f"  {n}")
print(f"5 renders -> {os.path.abspath(OUT)}")
