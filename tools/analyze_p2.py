#!/usr/bin/env python3
"""analyze_p2.py — p2 return analysis: the clean-path Vocodex mechanism + Torque spot-checks.

Run: nix develop -c python3 tools/analyze_p2.py
"""

import os
import numpy as np
import soundfile as sf
from scipy import signal

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = os.path.join(os.path.dirname(__file__), "..")
PROBES = os.path.join(ROOT, "out", "clang-probes-p2", "probes")
TQ = os.path.join(ROOT, "out", "clang-probes-p2-return", "renders", "torque")
VX = os.path.join(ROOT, "out", "clang-probes-p2-return", "renders", "vocodex")
PLATES = os.path.join(ROOT, "out", "analysis-p1", "plates")
os.makedirs(PLATES, exist_ok=True)
SR = 44100


def rd(d, name):
    p = os.path.join(d, name + ".wav")
    if not os.path.exists(p):
        p = os.path.join(d, "x__" + name + ".wav")
    x, sr = sf.read(p, always_2d=True)
    assert sr == SR
    return x.mean(axis=1)


def rms(x):
    return float(np.sqrt(np.mean(x ** 2))) if len(x) else 0.0


def dbv(x):
    return 20 * np.log10(max(x, 1e-12))


def peaks_fine(x, fmin, fmax, floor_db=-55, n=8):
    X = np.abs(np.fft.rfft(x * np.hanning(len(x)), n=1 << 19))
    f = np.fft.rfftfreq(1 << 19, 1 / SR)
    sel = (f >= fmin) & (f <= fmax)
    Xs, fs_ = X[sel], f[sel]
    L = 20 * np.log10(Xs / Xs.max() + 1e-12)
    idx, _ = signal.find_peaks(L, height=floor_db, distance=400)
    order = np.argsort(Xs[idx])[::-1][:n]
    out = []
    for i in sorted(idx[order]):
        # parabolic interp
        a, b, c = L[i - 1], L[i], L[i + 1]
        d = 0.5 * (a - c) / (a - 2 * b + c + 1e-12)
        out.append((round(float(fs_[i] + d * (fs_[1] - fs_[0])), 2), round(float(b), 1)))
    return out


def ridge(x, fmin, fmax, nperseg=8192, hop=2048, gate_db=-60):
    f, t, S = signal.spectrogram(x, SR, nperseg=nperseg, noverlap=nperseg - hop,
                                 mode="magnitude")
    sel = (f >= fmin) & (f <= fmax)
    S, f = S[sel], f[sel]
    ref = S.max()
    fr, tt = [], []
    for i in range(S.shape[1]):
        col = S[:, i]
        if col.max() > ref * 10 ** (gate_db / 20):
            fr.append(f[np.argmax(col)])
            tt.append(t[i])
    return np.array(tt), np.array(fr)


P = print
P("=" * 74)
P("TORQUE p2 spot-checks (operator tables already in return)")
P("=" * 74)
P("\nq01 T+1200 (expect main 440.00, sideband 880 if +2f0 / +440):")
P(" ", peaks_fine(rd(TQ, "q01_partial_220__T+1200_F220_Th-70_S15_std")[:int(SR*2)], 60, 2000))
P("q01 T-1200 (expect main 110.00, sideband 550 = 110+440):")
P(" ", peaks_fine(rd(TQ, "q01_partial_220__T-1200_F220_Th-70_S15_std")[:int(SR*2)], 60, 2000))

P("\n" + "=" * 74)
P("VOCODEX p2 — clean path (pass-through 0): the actual mechanism")
P("=" * 74)

# --- activation map: the definitive octave measurement -------------------
P("\n[M1] sweep→output ridge ratio out/in (1-9 s), clean path:")
for var in ["", "_mod0car0", "_mod+1200", "_bdcram200", "_bdlinear"]:
    x = rd(VX, f"p06_sweep_slow__vdx-av-clean{var}")
    to, fo = ridge(x, 20, 20000)
    fin = 20.0 * np.exp(to / 10.0 * np.log(1000.0))
    m = (to > 1) & (to < 9)
    if m.sum() < 10:
        P(f"  {var or '(clean=-12st)':22s} [too few frames above gate: {m.sum()}]")
        continue
    r = fo[m] / fin[m]
    P(f"  {var or '(clean=-12st)':22s} median={np.median(r):5.3f}  p10={np.percentile(r,10):5.3f}  "
      f"p90={np.percentile(r,90):5.3f}  rms={rms(x):8.2e}")

# --- band census on clean sweep ------------------------------------------
P("\n[M2] carrier-band centers, clean sweep (expect ~47 total, raised floor):")
for var in ["", "_bdlinear", "_bdcram200"]:
    x = rd(VX, f"p06_sweep_slow__vdx-av-clean{var}")
    f, t, S = signal.spectrogram(x, SR, nperseg=16384, noverlap=8192, mode="magnitude")
    avg = S.mean(axis=1)
    L = 20 * np.log10(avg / avg.max() + 1e-12)
    idx, _ = signal.find_peaks(L, height=-60, distance=6, prominence=6)
    centers = f[idx]
    P(f"  {var or '(clean)':12s} n={len(centers):3d}  <500Hz: {(centers<500).sum():2d}  "
      f"first6: {[round(float(c),1) for c in centers[:6]]}  last: {round(float(centers[-1]),1) if len(centers) else '-'}")

# --- kick low-end, mechanism isolated ------------------------------------
P("\n[M3] r01 kick octave bands 31..1k (dB, absolute), clean path:")
for var in ["", "_mod0car0", "_mod+1200", "_bdcram200"]:
    x = rd(VX, f"r01_kick_catsum__vdx-av-clean{var}")
    X = np.abs(np.fft.rfft(x * np.hanning(len(x)))) ** 2
    fr = np.fft.rfftfreq(len(x), 1 / SR)
    row, lo = [], 31.25
    for _ in range(6):
        row.append(round(10 * np.log10(X[(fr >= lo) & (fr < lo * 2)].sum() + 1e-18), 1))
        lo *= 2
    P(f"  {var or '(clean=-12st)':22s} {row}  rms={rms(x):8.2e}")

# --- raw envelope law ----------------------------------------------------
P("\n[M4] p02 per-impulse wet RMS (dBFS) — RAW law (no SG, no pass-through):")
for var in ["", "_envs0-mintimes-off", "_envs0-mintimes-on", "_release-min"]:
    x = rd(VX, f"p02_dirac_steps__vdx-av-clean{var}")
    row = []
    for i in range(7):
        w0, w1 = int(SR * (0.25 + 0.5 * i - 0.02)), int(SR * (0.25 + 0.5 * i + 0.4))
        row.append(round(dbv(rms(x[w0:w1])), 1))
    P(f"  {var or '(clean)':22s} {row}")

# --- band order (clang steepness) ----------------------------------------
P("\n[M5] band ORDER 1/2(clean)/3/4 — r01 spectral tilt + ring time:")
for var, lbl in [("_bandorder-1", "order1"), ("", "order2(clean)"),
                 ("_bandorder-3", "order3"), ("_bandorder-4", "order4")]:
    x = rd(VX, f"r01_kick_catsum__vdx-av-clean{var}")
    env = np.abs(signal.hilbert(x))
    pk = env.max()
    above = np.where(env > pk * 10 ** (-40 / 20))[0]
    ring_ms = 1000 * (above[-1] - above[0]) / SR if len(above) else 0
    P(f"  {lbl:14s} rms={rms(x):8.2e}  ring(-40dB)={ring_ms:7.1f} ms")

# plate: the M1 mechanism
fig, ax = plt.subplots(figsize=(10, 5))
for var, c, lbl in [("", "C3", "clean (-12 st)"), ("_mod0car0", "C0", "mod 0 st"),
                    ("_mod+1200", "C2", "mod +12 st")]:
    x = rd(VX, f"p06_sweep_slow__vdx-av-clean{var}")
    to, fo = ridge(x, 20, 20000)
    fin = 20.0 * np.exp(to / 10.0 * np.log(1000.0))
    m = (to > 1) & (to < 9)
    ax.semilogx(fin[m], fo[m] / fin[m], ".", ms=3, color=c, label=lbl)
for y in [0.5, 1.0, 2.0]:
    ax.axhline(y, color="k", lw=0.5, alpha=0.5)
ax.set(xlabel="input sweep freq (Hz)", ylabel="out/in freq ratio",
       title="Vocodex clean path: modulator pitch shift IS the map shift", ylim=(0, 3))
ax.grid(True, which="both", alpha=0.3); ax.legend()
fig.tight_layout(); fig.savefig(os.path.join(PLATES, "vocodex-p2-map-shift.png"), dpi=120)
P(f"\nplate -> {PLATES}/vocodex-p2-map-shift.png")
