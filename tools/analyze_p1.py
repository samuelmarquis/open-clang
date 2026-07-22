#!/usr/bin/env python3
"""analyze_p1.py — measured characterization of Torque + Vocodex from probe-pack p1 returns.

Reads out/clang-probes-p1-return/renders/*, compares against out/clang-probes-p1/probes/*,
prints a compact evidence report, saves plates to out/analysis-p1/plates/.

Run: nix develop -c python3 tools/analyze_p1.py
"""

import os
import numpy as np
import soundfile as sf
from scipy import signal

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = os.path.join(os.path.dirname(__file__), "..")
PROBES = os.path.join(ROOT, "out", "clang-probes-p1", "probes")
TORQUE = os.path.join(ROOT, "out", "clang-probes-p1-return", "renders", "torque")
VDX = os.path.join(ROOT, "out", "clang-probes-p1-return", "renders", "vocodex")
PLATES = os.path.join(ROOT, "out", "analysis-p1", "plates")
os.makedirs(PLATES, exist_ok=True)
SR = 44100


def rd(path):
    x, sr = sf.read(path, always_2d=True)
    assert sr == SR, f"{path}: sr={sr}"
    return x.mean(axis=1)


def dbv(x):
    return 20 * np.log10(max(x, 1e-12))


def rms(x):
    return float(np.sqrt(np.mean(x ** 2))) if len(x) else 0.0


def probe(name):
    return rd(os.path.join(PROBES, name + ".wav"))


def tq(name):
    return rd(os.path.join(TORQUE, name + ".wav"))


def vx(name):
    return rd(os.path.join(VDX, name + ".wav"))


def pad_to(x, n):
    return np.pad(x, (0, max(0, n - len(x))))[:n]


def null_fit(out, dry):
    """least-squares gain of dry in out; return (gain, residual_db_rel_dry)."""
    n = min(len(out), len(dry))
    o, d = out[:n], dry[:n]
    g = float(np.dot(o, d) / np.dot(d, d))
    res = rms(o - g * d) / max(rms(d), 1e-12)
    return g, 20 * np.log10(max(res, 1e-12))


def xcorr_lag(out, ref, max_lag=4096):
    n = min(len(out), len(ref))
    c = signal.correlate(out[:n], ref[:n], mode="full")
    lags = signal.correlation_lags(n, n, mode="full")
    m = np.abs(lags) <= max_lag
    return int(lags[m][np.argmax(c[m])])


def spec_peaks(x, t0, t1, fmin, fmax, n_peaks=8, floor_db=-60):
    seg = x[int(SR * t0):int(SR * t1)]
    seg = seg * np.hanning(len(seg))
    X = np.abs(np.fft.rfft(seg, n=1 << 18))
    f = np.fft.rfftfreq(1 << 18, 1 / SR)
    sel = (f >= fmin) & (f <= fmax)
    Xs, fs_ = X[sel], f[sel]
    ref = Xs.max()
    idx, _ = signal.find_peaks(20 * np.log10(Xs / ref + 1e-12), height=floor_db, distance=200)
    order = np.argsort(Xs[idx])[::-1][:n_peaks]
    pk = sorted(fs_[idx[order]])
    return [round(float(p), 1) for p in pk]


def octave_bands(x, fmin=31.25, n=10):
    X = np.abs(np.fft.rfft(x * np.hanning(len(x)))) ** 2
    f = np.fft.rfftfreq(len(x), 1 / SR)
    out = []
    lo = fmin
    for _ in range(n):
        hi = lo * 2
        e = X[(f >= lo) & (f < hi)].sum()
        out.append(10 * np.log10(e + 1e-18))
        lo = hi
    return np.array(out)


def ridge(x, fmin, fmax, nperseg=4096, hop=1024, gate_db=-70):
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
P("=" * 72)
P("TORQUE")
P("=" * 72)

# --- T6/T5: alignment + component phase behavior -------------------------
P("\n[T5/T6] alignment & null vs dry (gain-fitted residual, dB rel dry):")
for nm, dryn in [("r01_kick_catsum__T-700_F300_Th-48_S15_std", "r01_kick_catsum"),
                 ("r01_kick_catsum__T-700_F300_Th-48_S15_live", "r01_kick_catsum"),
                 ("p07_sweep_fast__T-700_F300_Th-48_S15_std", "p07_sweep_fast"),
                 ("p07_sweep_fast__T-700_F300_Th-48_S15_live", "p07_sweep_fast")]:
    o, d = tq(nm), probe(dryn)
    lag = xcorr_lag(o, pad_to(d, len(o)))
    g, res = null_fit(o, pad_to(d, len(o)))
    P(f"  {nm:52s} lag={lag:+4d} smp  g={g:5.2f}  resid={res:6.1f} dB")

# --- T1: partials vs envelope on p11 -------------------------------------
P("\n[T1] p11 modal stack partials (Hz), window 0.05-1.0 s, 60-2000 Hz:")
exp = [round(110 * r, 1) for r in [1.0, 1.594, 2.136, 2.296, 2.653, 2.918]]
P(f"  expected dry: {exp}")
for nm in ["p11_modal_stack__j00",
           "p11_modal_stack__T-1200_F300_Th-48_S15_std",
           "p11_modal_stack__T+1200_F300_Th-48_S15_std",
           "p11_modal_stack__T-1200_F98_Th-48_S15_std",
           "p11_modal_stack__T-1200_F900_Th-48_S15_std"]:
    P(f"  {nm:44s} {spec_peaks(tq(nm), 0.05, 1.0, 60, 2000, 6)}")

# octave-band envelope shift for the same renders
P("\n[T1b] p11 octave-band energy delta vs j00 (dB), bands 31..16k:")
ref = octave_bands(tq("p11_modal_stack__j00"))
for nm in ["p11_modal_stack__T-1200_F300_Th-48_S15_std",
           "p11_modal_stack__T+1200_F300_Th-48_S15_std"]:
    d = octave_bands(tq(nm)) - ref
    P(f"  {nm:44s} {[round(float(v),1) for v in d]}")

# --- T2: focus region on r02 ---------------------------------------------
P("\n[T2] r02 snare, T-1200: octave-band delta vs j00 per Focus (dB):")
ref = octave_bands(tq("r02_snare_catsum__j00"))
for F in [98, 300, 900]:
    d = octave_bands(tq(f"r02_snare_catsum__T-1200_F{F}_Th-48_S15_std")) - ref
    P(f"  F={F:3d}  {[round(float(v),1) for v in d]}")

# --- T3/T4a: transients & IR ---------------------------------------------
P("\n[T3] transient integrity:")
for nm, dryn in [("p01_dirac__T-1200_F300_Th-70_S15_std", "p01_dirac"),
                 ("p01_dirac__T+1200_F300_Th-70_S15_std", "p01_dirac"),
                 ("p03_click_lp2k__T-1200_F300_Th-70_S15_std", "p03_click_lp2k"),
                 ("p07_sweep_fast__T-1200_F300_Th-70_S15_std", "p07_sweep_fast")]:
    o, d = tq(nm), probe(dryn)
    eo = np.abs(signal.hilbert(o))
    ed = np.abs(signal.hilbert(pad_to(d, len(o))))
    po, pd = int(np.argmax(eo)), int(np.argmax(ed))
    # significant length: last sample above -60 dB rel peak
    lo = np.where(eo > eo.max() * 1e-3)[0]
    ld = np.where(ed > ed.max() * 1e-3)[0]
    P(f"  {nm:44s} peakΔ={po-pd:+5d} smp  siglen {ld[-1]-ld[0]:6d}→{lo[-1]-lo[0]:6d} smp")

# --- T4b: threshold gating on p02 ----------------------------------------
P("\n[T4b] p02 step gating: per-impulse processed-diff RMS (dBFS), steps -60..-1:")
d = probe("p02_dirac_steps")
j = tq("p02_dirac_steps__j00")
for Th in [-70, -48, -24, -6]:
    o = tq(f"p02_dirac_steps__T-700_F300_Th{Th}_S15_std")
    n = min(len(o), len(j))
    diff = o[:n] - j[:n]
    row = []
    for i in range(7):
        w0, w1 = int(SR * (0.25 + 0.5 * i - 0.02)), int(SR * (0.25 + 0.5 * i + 0.4))
        row.append(round(dbv(rms(diff[w0:w1])), 1))
    P(f"  Th={Th:+4d}  {row}")

# --- glide interaction on p12 --------------------------------------------
P("\n[p12] glide ridge mean ratio out/dry over 0-250 ms (30-300 Hz):")
td, fd = ridge(probe("p12_glide_kick"), 30, 300, nperseg=2048, hop=256)
for nm in ["p12_glide_kick__T-1200_F98_Th-48_S15_std",
           "p12_glide_kick__T-1200_F300_Th-48_S15_std",
           "p12_glide_kick__T+1200_F98_Th-48_S15_std"]:
    to, fo = ridge(tq(nm), 30, 300, nperseg=2048, hop=256)
    m = (td <= 0.25)
    ratio = []
    for t_, f_ in zip(td[m], fd[m]):
        if len(to):
            k = np.argmin(np.abs(to - t_))
            if abs(to[k] - t_) < 0.02:
                ratio.append(fo[k] / f_)
    P(f"  {nm:44s} ratio={np.mean(ratio):5.3f} (n={len(ratio)})")

# --- p06 slow sweep through Torque ---------------------------------------
P("\n[p06] sweep ridge ratio out/in, frames 1-9 s:")
for nm in ["p06_sweep_slow__T-1200_F300_Th-70_S15_std",
           "p06_sweep_slow__T+1200_F300_Th-70_S15_std"]:
    to, fo = ridge(tq(nm), 25, 20000)
    fin = 20.0 * np.exp(to / 10.0 * np.log(1000.0))
    m = (to > 1) & (to < 9)
    r = fo[m] / fin[m]
    P(f"  {nm:44s} median={np.median(r):5.3f}  p10={np.percentile(r,10):5.3f}  p90={np.percentile(r,90):5.3f}")

# plate: focus-region spectra
fig, ax = plt.subplots(figsize=(10, 5))
refm = np.abs(np.fft.rfft(tq("r02_snare_catsum__j00")))
f = np.fft.rfftfreq(len(tq("r02_snare_catsum__j00")), 1 / SR)
for F in [98, 300, 900]:
    X = np.abs(np.fft.rfft(tq(f"r02_snare_catsum__T-1200_F{F}_Th-48_S15_std")))
    n = min(len(X), len(refm))
    sm = signal.savgol_filter(20 * np.log10((X[:n] + 1e-9) / (refm[:n] + 1e-9)), 401, 2)
    ax.semilogx(f[:n], sm, label=f"Focus {F} Hz")
ax.set(xlim=(40, 20000), xlabel="Hz", ylabel="dB vs dry",
       title="Torque T-1200: spectral delta vs Focus (r02 snare)")
ax.grid(True, which="both", alpha=0.3); ax.legend()
fig.tight_layout(); fig.savefig(os.path.join(PLATES, "torque-focus-delta.png"), dpi=120)

P("\n" + "=" * 72)
P("VOCODEX (all renders are 50%-wet + Soundgoodizer; see caveats)")
P("=" * 72)

# --- dry cancellation ----------------------------------------------------
P("\n[V0] dry-component fit (gain of dry in output; wet_est = out - g*dry):")
wet = {}
for pn in ["r01_kick_catsum", "p06_sweep_slow", "p02_dirac_steps", "r07_amen_loop"]:
    d = probe(pn)
    for var in ["", "_bdlinear", "_mp0"]:
        nm = f"{pn}__vdx-autovocoding{var}"
        o = vx(nm)
        g, res = null_fit(o, pad_to(d, len(o)))
        wet[nm] = o - g * pad_to(d, len(o))
        P(f"  {nm:52s} g={g:5.3f}  resid={res:6.1f} dB")

# --- V1/V2: band map + octave trick on the sweep -------------------------
P("\n[V1/V2] sweep→output ridge ratio (out_freq / in_freq), 1-9 s:")
for var in ["", "_bdlinear", "_mp0"]:
    nm = f"p06_sweep_slow__vdx-autovocoding{var}"
    to, fo = ridge(wet[nm], 25, 20000)
    fin = 20.0 * np.exp(to / 10.0 * np.log(1000.0))
    m = (to > 1) & (to < 9)
    r = fo[m] / fin[m]
    P(f"  {nm:52s} median={np.median(r):5.3f}  p10={np.percentile(r,10):5.3f}  p90={np.percentile(r,90):5.3f}")

# band-center census from time-averaged wet spectrum of the sweep
P("\n[V1b] estimated carrier-band centers (peaks of avg wet spectrum, sweep pass):")
for var in ["", "_bdlinear"]:
    nm = f"p06_sweep_slow__vdx-autovocoding{var}"
    x = wet[nm]
    fspec, t, S = signal.spectrogram(x, SR, nperseg=8192, noverlap=4096, mode="magnitude")
    avg = S.mean(axis=1)
    ref = avg.max()
    idx, _ = signal.find_peaks(20 * np.log10(avg / ref + 1e-12), height=-50, distance=8)
    centers = fspec[idx]
    lows = centers[centers < 500]
    P(f"  {nm:52s} n={len(centers)}  <500Hz: {len(lows)}  first 8: {[round(float(c),1) for c in centers[:8]]}")

# --- V3: gate map --------------------------------------------------------
P("\n[V3] p02 per-impulse wet RMS (dBFS) — vocoder+Soundgoodizer dynamics:")
for var in ["", "_mp0"]:
    nm = f"p02_dirac_steps__vdx-autovocoding{var}"
    x = wet[nm]
    row = []
    for i in range(7):
        w0, w1 = int(SR * (0.25 + 0.5 * i - 0.02)), int(SR * (0.25 + 0.5 * i + 0.4))
        row.append(round(dbv(rms(x[w0:w1])), 1))
    P(f"  {nm:52s} {row}")

# --- V4: kick low-end ----------------------------------------------------
P("\n[V4] r01 kick wet_est octave bands 31..1k (dB), per variant:")
for var in ["", "_bdlinear", "_mp0"]:
    nm = f"r01_kick_catsum__vdx-autovocoding{var}"
    b = octave_bands(wet[nm], fmin=31.25, n=6)
    P(f"  {nm:52s} {[round(float(v),1) for v in b]}")

# plate: sweep ridge scatter per variant
fig, ax = plt.subplots(figsize=(10, 5))
for var, c in [("", "C0"), ("_bdlinear", "C1"), ("_mp0", "C2")]:
    nm = f"p06_sweep_slow__vdx-autovocoding{var}"
    to, fo = ridge(wet[nm], 25, 20000)
    fin = 20.0 * np.exp(to / 10.0 * np.log(1000.0))
    m = (to > 1) & (to < 9)
    ax.semilogx(fin[m], fo[m] / fin[m], ".", ms=3, color=c, label=var or "as-is")
ax.axhline(1.0, color="k", lw=0.5)
ax.set(xlabel="input sweep freq (Hz)", ylabel="out/in freq ratio",
       title="Vocodex autovocoding: sweep transfer ratio per variant", ylim=(0, 3))
ax.grid(True, which="both", alpha=0.3); ax.legend()
fig.tight_layout(); fig.savefig(os.path.join(PLATES, "vocodex-sweep-ratio.png"), dpi=120)

P(f"\nplates -> {os.path.abspath(PLATES)}")
