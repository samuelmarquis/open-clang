# sd_snares.py — M10.5 part 2: the autopsy battery on the Superior
# Drummer reference pack (dry isolated hits, velocity ramps x three
# hit positions per drum; Halo-Feeder = processed aesthetic target).
#
# Reuses lab/snare_autopsy.py's battery; adds a floor-guarded T60
# fit (dry one-shots hit the noise floor inside the window — the
# unguarded fit inflates T60 there), per-hit velocity proxy (peak
# dBFS), position segmentation (ramp-restart detection on the peak-
# level sequence, fallback = thirds by hit index), and aggregation.

import os
import sys
import numpy as np
from scipy.signal import stft

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from snare_autopsy import (load_mono, ATTACK_BANDS, TAIL_BANDS,
                           fundamental, attack_profile, ringiness,
                           centroid_rolloff)

BASE = "/Users/sam/Dropbox/Samples/Drums/SD-Snares/"
FILES = [
    ("BlackBeauty-4x14", "4x14 Ludwig Black Beauty 20s.wav", True),
    ("Aluminum-5x14", "5x14 Gretsch Solid Aluminum.wav", True),
    ("PearStave-8x14", "8x14 Lignum Custom Pear Stave.wav", True),
    ("Coliseum-8x14", "8x14 Ludwig Coliseum.wav", True),
    ("HaloFeeder", "Halo-Feeder-Snares.wav", False),  # processed
]

BAND_NAMES_A = [f"{lo/1000:g}-{hi/1000:g}k" for lo, hi in ATTACK_BANDS]
BAND_NAMES_T = [f"{lo/1000:g}-{hi/1000:g}k" for lo, hi in TAIL_BANDS]


def oneshot_onsets(sr, x, min_sep=0.12):
    """Rising-edge onsets on a smoothed dB envelope; built for dry
    one-shot files where ghost hits sit far below the loud ones."""
    env = np.abs(x)
    win = int(0.004 * sr)
    sm = np.convolve(env, np.ones(win) / win, mode="same")
    db = 20 * np.log10(sm + 1e-9)
    floor = np.percentile(db, 15)
    thr = max(floor + 15.0, db.max() - 55.0)
    look = int(0.020 * sr)
    hits, i = [], look
    while i < len(db):
        if db[i] > thr and db[i] - db[i - look] > 12.0:
            j0 = max(0, i - int(0.006 * sr))
            seg = sm[j0:i + int(0.010 * sr)]
            if len(seg) > 2:
                hits.append((j0 + int(np.argmax(np.diff(seg)))) / sr)
            else:
                hits.append(i / sr)
            i += int(min_sep * sr)
        else:
            i += 1
    return hits


def tail_t60_guarded(sr, seg, t_from=0.060, t_to=0.35):
    """Per-band T60, fitting only until the band falls 45 dB below
    its first tail frame (noise-floor guard for dry one-shots)."""
    res = []
    i0, i1 = int(t_from * sr), min(int(t_to * sr), len(seg))
    if i1 - i0 < int(0.08 * sr):
        return None
    nper, hop = 1024, 256
    f, t, Z = stft(seg[i0:i1], sr, nperseg=nper, noverlap=nper - hop,
                   padded=False)
    E = np.abs(Z) ** 2
    for lo, hi in TAIL_BANDS:
        m = (f >= lo) & (f < hi)
        if not m.any():
            res.append(None)
            continue
        db = 10 * np.log10(E[m].sum(axis=0) + 1e-30)
        lim = db[0] - 45.0
        below = np.where(db < lim)[0]
        end = below[0] if len(below) else len(db)
        if end < 5:
            res.append(None)
            continue
        tt, dd = t[:end], db[:end]
        A = np.vstack([tt, np.ones_like(tt)]).T
        slope = np.linalg.lstsq(A, dd, rcond=None)[0][0]
        t60 = -60.0 / slope if slope < -1e-6 else np.inf
        res.append(round(float(t60), 3)
                   if np.isfinite(t60) and t60 < 30 else None)
    return res


def measure_hit(sr, x, t, gap):
    i0 = max(0, int((t - 0.002) * sr))
    seg = x[i0:i0 + int(0.65 * sr)]
    tail_to = float(min(0.60, max(0.15, gap - 0.015)))
    pk = np.abs(seg[: int(0.030 * sr)]).max() + 1e-30
    f0, peaks = fundamental(sr, seg)
    margin = -peaks[1][1] if peaks and len(peaks) > 1 else None
    dom_is_lowest = None
    if peaks:
        dom_is_lowest = peaks[0][0] <= min(p[0] for p in peaks) + 1.0
    atk = attack_profile(sr, seg)
    lead5 = lead20 = None
    if atk:
        if "0-5ms" in atk:
            lead5 = BAND_NAMES_A[int(np.argmax(atk["0-5ms"]))]
        if "5-20ms" in atk:
            lead20 = BAND_NAMES_A[int(np.argmax(atk["5-20ms"]))]
    ring = ringiness(sr, seg, t_to=tail_to)
    cr = centroid_rolloff(sr, seg, t_to=tail_to)
    return dict(
        t=t, peak_db=20 * np.log10(pk), f0=f0, margin=margin,
        dom_is_lowest=dom_is_lowest,
        rise=atk["rise_ms"] if atk else None, lead5=lead5, lead20=lead20,
        atk5=atk.get("0-5ms") if atk else None,
        atk20=atk.get("5-20ms") if atk else None,
        t60=tail_t60_guarded(sr, seg, t_to=tail_to),
        ring_n=ring["n_peaks_gt6dB"] if ring else None,
        ring_prom=ring["mean_top10_prom"] if ring else None,
        ring_peaks=ring["top_peaks"] if ring else None,
        centroid=cr[0] if cr else None, rolloff=cr[1] if cr else None,
        tail_to=tail_to,
    )


def segment_positions(rows):
    """Ramp-restart detection: a new position group starts when the
    velocity proxy drops >8 dB below the previous hit after >=3 hits
    in the current group. Fallback: thirds by hit index."""
    bounds = [0]
    for i in range(1, len(rows)):
        if (rows[i]["peak_db"] < rows[i - 1]["peak_db"] - 8.0
                and i - bounds[-1] >= 3 and len(bounds) < 3):
            bounds.append(i)
    if len(bounds) != 3:
        n = len(rows)
        return [0, n // 3, 2 * n // 3], "thirds-fallback"
    return bounds, "ramp-restart"


def med(vals):
    v = [x for x in vals if x is not None]
    return round(float(np.median(v)), 3) if v else None


def med_lists(lists):
    ls = [l for l in lists if l]
    if not ls:
        return None
    n = len(ls[0])
    return [med([l[k] for l in ls]) for k in range(n)]


def mode_str(vals):
    v = [x for x in vals if x]
    if not v:
        return None
    names, counts = np.unique(v, return_counts=True)
    return str(names[np.argmax(counts)])


def aggregate(rows):
    return dict(
        n=len(rows),
        f0=med([r["f0"] for r in rows]),
        margin=med([r["margin"] for r in rows]),
        dom_lowest_pct=round(100 * np.mean(
            [bool(r["dom_is_lowest"]) for r in rows])) if rows else None,
        rise=med([r["rise"] for r in rows]),
        lead5=mode_str([r["lead5"] for r in rows]),
        lead20=mode_str([r["lead20"] for r in rows]),
        t60=med_lists([r["t60"] for r in rows]),
        atk5=med_lists([r["atk5"] for r in rows]),
        atk20=med_lists([r["atk20"] for r in rows]),
        ring_n=med([r["ring_n"] for r in rows]),
        ring_prom=med([r["ring_prom"] for r in rows]),
        centroid=med([r["centroid"] for r in rows]),
    )


def show_agg(label, a):
    print(f"  [{label}] n={a['n']}  f0 {a['f0']} Hz  "
          f"margin {a['margin']} dB  dom-is-lowest {a['dom_lowest_pct']}%")
    print(f"      rise {a['rise']} ms  lead 0-5ms {a['lead5']}  "
          f"5-20ms {a['lead20']}  ring {a['ring_n']} pk / "
          f"{a['ring_prom']} dB  centroid {a['centroid']} Hz")
    print(f"      T60 {BAND_NAMES_T}")
    print(f"          {a['t60']} s")
    print(f"      atk bands {BAND_NAMES_A}")
    print(f"        0-5ms  {a['atk5']}")
    print(f"        5-20ms {a['atk20']}")


def stable_ring_freqs(rows, tol=60.0):
    """Ring peaks recurring in >=60% of loud-half hits (per-drum
    'wire mode' fingerprint)."""
    loud = sorted(rows, key=lambda r: -r["peak_db"])[:max(3, len(rows) // 2)]
    allp = []
    for r in loud:
        if r["ring_peaks"]:
            allp.append([p[0] for p in r["ring_peaks"]])
    if not allp:
        return []
    out = []
    for f in allp[0]:
        hits = sum(1 for ps in allp if any(abs(f - q) < tol for q in ps))
        if hits / len(allp) >= 0.6:
            out.append(int(f))
    return out


def main():
    per_drum = {}
    for label, fname, dry in FILES:
        sr, x = load_mono(BASE + fname)
        ts = oneshot_onsets(sr, x)
        print(f"\n########## {label}  (sr {sr}, {len(x)/sr:.1f} s, "
              f"{len(ts)} hits, {'dry' if dry else 'PROCESSED'}) ##########")
        rows = []
        for k, t in enumerate(ts):
            gap = (ts[k + 1] - t) if k + 1 < len(ts) else (len(x) / sr - t)
            rows.append(measure_hit(sr, x, t, gap))
        for k, r in enumerate(rows):
            t60s = r["t60"]
            print(f"  #{k:02d} {r['t']:6.2f}s pk{r['peak_db']:6.1f} "
                  f"f0 {str(r['f0']):>6} mg {str(r['margin']):>5} "
                  f"rise {str(r['rise']):>5} l5 {str(r['lead5']):>7} "
                  f"l20 {str(r['lead20']):>7} ring {str(r['ring_n']):>4}"
                  f"/{str(r['ring_prom']):>4} cen {str(r['centroid']):>5} "
                  f"t60(8-14k) {t60s[-1] if t60s else None}")
        bounds, how = segment_positions(rows)
        print(f"  position segmentation: {how} at hit indices {bounds}")
        groups = [rows[bounds[0]:bounds[1]], rows[bounds[1]:bounds[2]],
                  rows[bounds[2]:]]
        for gi, g in enumerate(groups):
            if g:
                show_agg(f"pos{gi+1}", aggregate(g))
        allagg = aggregate(rows)
        show_agg("ALL", allagg)
        # velocity terciles pooled across positions
        srt = sorted(rows, key=lambda r: r["peak_db"])
        n3 = max(1, len(srt) // 3)
        for lab2, sel in [("soft", srt[:n3]), ("loud", srt[-n3:])]:
            a = aggregate(sel)
            print(f"  [{lab2}] rise {a['rise']} ms  lead5 {a['lead5']}  "
                  f"lead20 {a['lead20']}  ring {a['ring_n']}pk/"
                  f"{a['ring_prom']}dB  margin {a['margin']} dB  "
                  f"t60(8-14k) {a['t60'][-1] if a['t60'] else None}")
        print(f"  stable ring peaks (>=60% of loud hits): "
              f"{stable_ring_freqs(rows)}")
        per_drum[label] = allagg

    # Halo-Feeder delta vs mean of dry drums
    dry_labels = [l for l, _, d in FILES if d]
    print("\n########## HALO-FEEDER DELTA (vs mean of dry drums) ##########")
    halo = per_drum["HaloFeeder"]
    for key in ("f0", "margin", "rise", "ring_n", "ring_prom", "centroid"):
        dv = [per_drum[l][key] for l in dry_labels
              if per_drum[l][key] is not None]
        if dv and halo[key] is not None:
            print(f"  {key}: dry mean {np.mean(dv):.1f}  "
                  f"halo {halo[key]}  delta {halo[key]-np.mean(dv):+.1f}")
    dt = [per_drum[l]["t60"] for l in dry_labels if per_drum[l]["t60"]]
    if dt and halo["t60"]:
        dmean = [np.nanmean([(t[k] if t[k] is not None else np.nan)
                             for t in dt]) for k in range(len(TAIL_BANDS))]
        print(f"  T60 dry mean: {[round(v,3) for v in dmean]}")
        print(f"  T60 halo:     {halo['t60']}")


if __name__ == "__main__":
    main()
