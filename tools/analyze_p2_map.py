#!/usr/bin/env python3
"""analyze_p2_map.py — resolve the Modulator-pitch-shift band-map mechanism visually + gated."""

import os
import numpy as np
import soundfile as sf
from scipy import signal

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = os.path.join(os.path.dirname(__file__), "..")
VX = os.path.join(ROOT, "out", "clang-probes-p2-return", "renders", "vocodex")
PLATES = os.path.join(ROOT, "out", "analysis-p1", "plates")
SR = 44100


def rd(name):
    p = os.path.join(VX, name + ".wav")
    if not os.path.exists(p):
        p = os.path.join(VX, "x__" + name + ".wav")
    x, sr = sf.read(p, always_2d=True)
    return x.mean(axis=1)


variants = [("", "clean (-12 st)"), ("_mod0car0", "mod 0 st"), ("_mod+1200", "mod +12 st")]

fig, axes = plt.subplots(3, 1, figsize=(12, 10), sharex=True)
print("gated dominant-frequency ratio out/in (frames within 25 dB of render's loudest):")
for (var, lbl), ax in zip(variants, axes):
    x = rd(f"p06_sweep_slow__vdx-av-clean{var}")
    f, t, S = signal.spectrogram(x, SR, nperseg=8192, noverlap=8192 - 1024, mode="magnitude")
    frame_rms = S.sum(axis=0)
    gate = frame_rms > frame_rms.max() * 10 ** (-25 / 20)
    fin = 20.0 * np.exp(t / 10.0 * np.log(1000.0))
    ratios = []
    for i in range(S.shape[1]):
        if gate[i] and 0.5 < t[i] < 9.8:
            ratios.append(f[np.argmax(S[:, i])] / fin[i])
    r = np.array(ratios)
    print(f"  {lbl:16s} n={len(r):4d}  median={np.median(r):5.3f}  "
          f"p25={np.percentile(r,25):5.3f}  p75={np.percentile(r,75):5.3f}")
    ax.pcolormesh(t, f, 20 * np.log10(S + 1e-9), vmin=-100, vmax=-20, cmap="magma",
                  shading="auto")
    ax.plot(t, fin, "c--", lw=0.8, label="input sweep")
    ax.plot(t, fin / 2, "w:", lw=0.8, label="input/2")
    ax.plot(t, fin * 2, "g:", lw=0.8, label="input*2")
    ax.set(yscale="log", ylim=(20, 20000), ylabel=f"{lbl}\nHz")
    ax.legend(loc="upper left", fontsize=7)
axes[-1].set_xlabel("time (s)")
fig.suptitle("Vocodex clean path: sweep spectrograms per modulator-pitch setting")
fig.tight_layout()
fig.savefig(os.path.join(PLATES, "vocodex-p2-sweep-spectrograms.png"), dpi=110)
print(f"plate -> {PLATES}/vocodex-p2-sweep-spectrograms.png")
