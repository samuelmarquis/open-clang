# snare_autopsy.py — M10.5: the snare autopsy (measurement, no engine code)
#
# Probe methodology turned on real snares. Batteries:
#   fundamental (+ top peaks), attack rise + band balance,
#   per-band tail T60, tail RINGINESS (tonal-peak-to-noise-floor),
#   tail centroid/rolloff.
# Run on: Angelo Mides loops (backbeat isolation) + out/snare-v2 renders.
# Findings feed docs/research/06-snare-measured.md and set M11 scope.

import sys
import numpy as np
from scipy.io import wavfile
from scipy.signal import stft, medfilt

# ---------- loading ----------

def load_mono(path):
    sr, x = wavfile.read(path)
    if x.dtype == np.int16:
        x = x.astype(np.float64) / 32768.0
    elif x.dtype == np.int32:
        x = x.astype(np.float64) / 2147483648.0
    elif x.dtype == np.uint8:
        x = (x.astype(np.float64) - 128.0) / 128.0
    else:
        x = x.astype(np.float64)
    if x.ndim == 2:
        x = x.mean(axis=1)
    return sr, x

# ---------- onset detection (spectral flux) ----------

def onsets(sr, x, min_sep=0.09):
    nper = 1024
    hop = 256
    f, t, Z = stft(x, sr, nperseg=nper, noverlap=nper - hop, padded=False)
    mag = np.abs(Z)
    flux = np.maximum(mag[:, 1:] - mag[:, :-1], 0.0).sum(axis=0)
    flux = flux / (flux.max() + 1e-12)
    thr = np.convolve(flux, np.ones(31) / 31, mode="same") * 1.8 + 0.02
    hits = []
    last = -1.0
    for i in range(1, len(flux) - 1):
        if flux[i] > thr[i] and flux[i] >= flux[i - 1] and flux[i] >= flux[i + 1]:
            tt = t[i + 1]
            if tt - last >= min_sep:
                hits.append(tt)
                last = tt
    # refine each onset to the local energy-rise point (5 ms grid)
    ref = []
    for tt in hits:
        i0 = max(0, int((tt - 0.02) * sr))
        i1 = min(len(x), int((tt + 0.02) * sr))
        seg = np.abs(x[i0:i1])
        if len(seg) < 8:
            ref.append(tt)
            continue
        env = np.convolve(seg, np.ones(32) / 32, mode="same")
        d = np.diff(env)
        ref.append((i0 + int(np.argmax(d))) / sr)
    return ref

# ---------- band tools ----------

def band_db(sr, seg, lo, hi):
    n = max(2048, 1 << int(np.ceil(np.log2(max(len(seg), 2)))))
    S = np.abs(np.fft.rfft(seg * np.hanning(len(seg)), n)) ** 2
    fr = np.fft.rfftfreq(n, 1 / sr)
    m = (fr >= lo) & (fr < hi)
    if not m.any():
        return -180.0
    return 10 * np.log10(S[m].sum() + 1e-30)

CLASS_BANDS = dict(lf=(30, 120), body=(120, 350), crack=(1500, 5000), top=(5000, 12000))

def classify(sr, x, t):
    i0 = int(t * sr)
    seg = x[i0:i0 + int(0.06 * sr)]
    if len(seg) < 256:
        return None, {}
    d = {k: band_db(sr, seg, *v) for k, v in CLASS_BANDS.items()}
    ref = max(d.values())
    r = {k: v - ref for k, v in d.items()}
    # snare: crack strong AND body present; kick: lf dominant; hat: no body
    if r["lf"] == 0 and r["crack"] < -18:
        lab = "kick"
    elif r["body"] < -22 and (r["crack"] > -8 or r["top"] > -8):
        lab = "hat"
    elif r["crack"] > -14 and r["body"] > -16:
        lab = "SNARE"
    else:
        lab = "other"
    return lab, r

# ---------- batteries ----------

ATTACK_BANDS = [(100, 500), (500, 1500), (1500, 4000), (4000, 8000), (8000, 14000)]
TAIL_BANDS = [(150, 400), (400, 1000), (1000, 2000), (2000, 4000), (4000, 8000), (8000, 14000)]

def fundamental(sr, seg):
    """Strongest peaks below 600 Hz over the 10-150 ms body window."""
    a, b = int(0.010 * sr), int(0.150 * sr)
    w = seg[a:min(b, len(seg))]
    if len(w) < 512:
        return None, []
    n = 1 << int(np.ceil(np.log2(len(w) * 4)))
    S = np.abs(np.fft.rfft(w * np.hanning(len(w)), n))
    fr = np.fft.rfftfreq(n, 1 / sr)
    m = (fr >= 60) & (fr <= 600)
    Sm, frm = S[m], fr[m]
    pk = []
    for i in range(1, len(Sm) - 1):
        if Sm[i] > Sm[i - 1] and Sm[i] >= Sm[i + 1]:
            pk.append((frm[i], 20 * np.log10(Sm[i] + 1e-30)))
    pk.sort(key=lambda p: -p[1])
    top = pk[:5]
    if not top:
        return None, []
    ref = top[0][1]
    return top[0][0], [(round(f_, 1), round(db - ref, 1)) for f_, db in top]

def attack_profile(sr, seg):
    """Rise time + band balance in 0-5 ms and 5-20 ms."""
    env = np.abs(seg[: int(0.03 * sr)])
    if len(env) < 64:
        return None
    sm = np.convolve(env, np.ones(16) / 16, mode="same")
    pk = sm.max() + 1e-30
    i10 = np.argmax(sm > 0.1 * pk)
    i90 = np.argmax(sm > 0.9 * pk)
    rise_ms = max(0.0, (i90 - i10) / sr * 1e3)
    out = {"rise_ms": round(rise_ms, 2)}
    for name, (a, b) in [("0-5ms", (0, 0.005)), ("5-20ms", (0.005, 0.020))]:
        s = seg[int(a * sr):int(b * sr)]
        if len(s) < 32:
            continue
        d = [band_db(sr, s, lo, hi) for lo, hi in ATTACK_BANDS]
        ref = max(d)
        out[name] = [round(v - ref, 1) for v in d]
    return out

def tail_t60(sr, seg, t_from=0.060, t_to=0.250):
    """Per-band decay slope -> T60 estimate over the tail window."""
    res = []
    i0, i1 = int(t_from * sr), min(int(t_to * sr), len(seg))
    if i1 - i0 < int(0.08 * sr):
        return None
    for lo, hi in TAIL_BANDS:
        # narrowband envelope via STFT band energy per frame
        nper = 1024
        hop = 256
        f, t, Z = stft(seg[i0:i1], sr, nperseg=nper, noverlap=nper - hop, padded=False)
        m = (f >= lo) & (f < hi)
        if not m.any():
            res.append(None)
            continue
        e = (np.abs(Z[m]) ** 2).sum(axis=0)
        db = 10 * np.log10(e + 1e-30)
        # linear fit
        A = np.vstack([t, np.ones_like(t)]).T
        slope, _ = np.linalg.lstsq(A, db, rcond=None)[0]
        t60 = -60.0 / slope if slope < -1e-6 else np.inf
        res.append(round(float(t60), 3) if np.isfinite(t60) else None)
    return res

def ringiness(sr, seg, t_from=0.060, t_to=0.250, lo=400, hi=8000):
    """Tonal structure of the tail: spectral peaks above the local
    median floor, 400 Hz-8 kHz. Returns (n_peaks>6dB, mean top-10
    prominence dB, list of top peak freqs)."""
    i0, i1 = int(t_from * sr), min(int(t_to * sr), len(seg))
    if i1 - i0 < int(0.08 * sr):
        return None
    nper = 4096
    f, t, Z = stft(seg[i0:i1], sr, nperseg=nper, noverlap=nper - nper // 4, padded=False)
    S = (np.abs(Z) ** 2).mean(axis=1)
    db = 10 * np.log10(S + 1e-30)
    # local median floor, ~300 Hz wide
    k = int(300 / (sr / nper)) | 1
    floor = medfilt(db, kernel_size=k)
    prom = db - floor
    m = (f >= lo) & (f <= hi)
    pk = []
    idx = np.where(m)[0]
    for i in idx[1:-1]:
        if db[i] > db[i - 1] and db[i] >= db[i + 1] and prom[i] > 3.0:
            pk.append((float(f[i]), float(prom[i])))
    pk.sort(key=lambda p: -p[1])
    n6 = sum(1 for _, p in pk if p > 6.0)
    top10 = pk[:10]
    mean10 = float(np.mean([p for _, p in top10])) if top10 else 0.0
    return dict(n_peaks_gt6dB=n6, mean_top10_prom=round(mean10, 1),
                top_peaks=[(round(f_), round(p, 1)) for f_, p in top10[:6]])

def centroid_rolloff(sr, seg, t_from=0.060, t_to=0.250):
    i0, i1 = int(t_from * sr), min(int(t_to * sr), len(seg))
    if i1 - i0 < 256:
        return None
    w = seg[i0:i1]
    n = 1 << int(np.ceil(np.log2(len(w))))
    S = np.abs(np.fft.rfft(w * np.hanning(len(w)), n)) ** 2
    fr = np.fft.rfftfreq(n, 1 / sr)
    c = float((S * fr).sum() / (S.sum() + 1e-30))
    csum = np.cumsum(S)
    r85 = float(fr[np.searchsorted(csum, 0.85 * csum[-1])])
    return round(c), round(r85)

def battery(sr, seg, tail_to):
    f0, peaks = fundamental(sr, seg)
    out = {
        "f0": round(f0, 1) if f0 else None,
        "lf_peaks(Hz,dBrel)": peaks,
        "attack": attack_profile(sr, seg),
        "tail_T60s": tail_t60(sr, seg, t_to=tail_to),
        "ringiness": ringiness(sr, seg, t_to=tail_to),
    }
    cr = centroid_rolloff(sr, seg, t_to=tail_to)
    if cr:
        out["tail_centroid/rolloff85"] = cr
    return out

def show(name, out):
    print(f"\n=== {name} ===")
    print(f"  f0: {out['f0']} Hz   LF peaks: {out['lf_peaks(Hz,dBrel)']}")
    a = out["attack"]
    if a:
        print(f"  attack rise 10-90%: {a['rise_ms']} ms")
        print(f"    bands {[f'{lo/1000:g}-{hi/1000:g}k' for lo,hi in ATTACK_BANDS]}")
        for k in ("0-5ms", "5-20ms"):
            if k in a:
                print(f"    {k}: {a[k]} dB rel")
    t = out["tail_T60s"]
    if t:
        print(f"  tail T60 {[f'{lo/1000:g}-{hi/1000:g}k' for lo,hi in TAIL_BANDS]}")
        print(f"    {t} s")
    r = out["ringiness"]
    if r:
        print(f"  RINGINESS: {r['n_peaks_gt6dB']} peaks >6 dB over floor, "
              f"mean top-10 prominence {r['mean_top10_prom']} dB")
        print(f"    top peaks (Hz, dB over floor): {r['top_peaks']}")
    if "tail_centroid/rolloff85" in out:
        c, r85 = out["tail_centroid/rolloff85"]
        print(f"  tail centroid {c} Hz, 85% rolloff {r85} Hz")

# ---------- drivers ----------

def analyze_loop(path, label):
    sr, x = load_mono(path)
    print(f"\n########## {label}  (sr {sr}, {len(x)/sr:.1f} s) ##########")
    hits = onsets(sr, x)
    rows = []
    for t in hits:
        lab, r = classify(sr, x, t)
        if lab:
            rows.append((t, lab, r))
    print("onsets:")
    for t, lab, r in rows:
        print(f"  {t:7.3f}s  {lab:6s}  " +
              "  ".join(f"{k}:{v:6.1f}" for k, v in r.items()))
    sn = [t for t, lab, _ in rows if lab == "SNARE"]
    all_t = [t for t, _, _ in rows]
    for j, t in enumerate(sn):
        nxt = min([u for u in all_t if u > t + 0.01], default=t + 0.4)
        tail_to = min(0.35, nxt - t - 0.015)
        if tail_to < 0.12:
            continue  # too contaminated for tail work
        i0 = int((t - 0.002) * sr)
        seg = x[i0:i0 + int(0.4 * sr)]
        show(f"{label} snare hit @{t:.3f}s (clean window {tail_to*1e3:.0f} ms)",
             battery(sr, seg, tail_to))

def analyze_oneshot(path, label):
    sr, x = load_mono(path)
    # find onset = first crossing of -40 dB rel peak
    pk = np.abs(x).max() + 1e-30
    i = int(np.argmax(np.abs(x) > pk * 0.01))
    i = max(0, i - int(0.002 * sr))
    seg = x[i:i + int(0.5 * sr)]
    show(f"{label} (sr {sr})", battery(sr, seg, 0.35))

if __name__ == "__main__":
    base = "/Users/sam/Dropbox/Samples/Drums/100/"
    analyze_loop(base + "ANGELO_MIDES_DRUMLOOP_FABLE_82 .wav", "FABLE_82")
    analyze_loop(base + "ANGELO_MIDES_DRUMLOOP_WORKOUT_138.wav", "WORKOUT_138")
    ours = "/Users/sam/Developer/open-clang/out/snare-v2/"
    for f, lab in [("snare_v095.wav", "OURS v095"),
                   ("snare_mallet_v095.wav", "OURS mallet"),
                   ("snare_cavOFF_v095.wav", "OURS cavOFF"),
                   ("snare_M9equiv_v095.wav", "OURS M9equiv")]:
        analyze_oneshot(ours + f, lab)
