#!/usr/bin/env python3
"""make_probes_p2.py — probe pack p2: Torque fine structure + Vocodex clean-path isolation.

New synth probes q01/q02; carries p02, p06, r01 over from p1 so the pack is
self-contained. Output: out/clang-probes-p2/probes/

Run: nix develop -c python3 tools/make_probes_p2.py
"""

import os
import shutil
import numpy as np
import soundfile as sf

SR = 44100
ROOT = os.path.join(os.path.dirname(__file__), "..")
P1 = os.path.join(ROOT, "out", "clang-probes-p1", "probes")
OUT = os.path.join(ROOT, "out", "clang-probes-p2", "probes")
os.makedirs(OUT, exist_ok=True)
manifest = []


def db(x):
    return 10.0 ** (x / 20.0)


def fade(x, ms_in, ms_out):
    n_in, n_out = int(SR * ms_in / 1000), int(SR * ms_out / 1000)
    if n_in:
        x[:n_in] *= 0.5 - 0.5 * np.cos(np.pi * np.arange(n_in) / n_in)
    if n_out:
        x[-n_out:] *= 0.5 + 0.5 * np.cos(np.pi * np.arange(n_out) / n_out)
    return x


def write(name, x, desc):
    x = np.clip(x, -1, 1)
    p = np.max(np.abs(x))
    x = x * (db(-6) / p)
    sf.write(os.path.join(OUT, name), x, SR, subtype="PCM_24")
    manifest.append((name, desc))
    print(f"  {name:28s} {len(x)/SR:5.2f}s  {desc}")


# q01: single long decaying partial — fine-structure target for Torque.
dur, f0, t60 = 2.5, 220.0, 2.0
t = np.arange(int(SR * dur)) / SR
x = np.sin(2 * np.pi * f0 * t) * 10 ** (-3 * t / t60)
write("q01_partial_220.wav", fade(x, 2, 30),
      "single decaying partial 220 Hz, T60 2 s; fine shift-ratio measurement")

# q02: two partials 220+330 (3:2), distinct decays — component-selection probe.
x = np.sin(2 * np.pi * 220.0 * t) * 10 ** (-3 * t / 2.0) \
    + 0.7 * np.sin(2 * np.pi * 330.0 * t + 1.3) * 10 ** (-3 * t / 1.2)
write("q02_partials_220_330.wav", fade(x, 2, 30),
      "two partials 220/330 Hz; which component does Focus grab?")

# carried over from p1 (bit-identical copies)
for nm, why in [("p02_dirac_steps.wav", "level steps -60..-1 dBFS; RAW envelope law (clean path)"),
                ("p06_sweep_slow.wav", "20-20k log sweep; band map & MOD/CAR isolation"),
                ("r01_kick_catsum.wav", "real kick; low-end voicing subject")]:
    shutil.copy2(os.path.join(P1, nm), os.path.join(OUT, nm))
    manifest.append((nm, f"(carried from p1) {why}"))
    print(f"  {nm:28s}  carried from p1")

with open(os.path.join(OUT, "MANIFEST.tsv"), "w") as f:
    f.write("file\tdescription\n")
    for name, desc in manifest:
        f.write(f"{name}\t{desc}\n")
print(f"\n{len(manifest)} probes -> {os.path.abspath(OUT)}")
