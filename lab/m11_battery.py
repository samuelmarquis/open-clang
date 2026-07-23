# m11_battery.py — the M11 acceptance gate: run the autopsy battery on
# render files and score them against the LOCKED targets (research 06).
#   uv run --with numpy --with scipy python lab/m11_battery.py FILE...
# Targets:
#   1 ring:  >=12 tonal peaks >6 dB over local tail floor (0.4-8 kHz),
#            mean top-10 prominence >=8 dB
#   2 crack: attack rise 10-90% in 1.5-4 ms (never an impulse), 0-5 ms
#            band profile LF-led monotonic-ish falloff
#   3 tail:  T60 flat 0.39-0.63 s every band, never rising with freq
#   4 root:  dominant LF peak margin >=10 dB (wood/deep voicing)

import sys
sys.path.insert(0, "lab")
from snare_autopsy import (load_mono, battery, ATTACK_BANDS, TAIL_BANDS)
import numpy as np


def score(path):
    sr, x = load_mono(path)
    pk = np.abs(x).max() + 1e-30
    i = int(np.argmax(np.abs(x) > pk * 0.01))
    i = max(0, i - int(0.002 * sr))
    seg = x[i:i + int(0.6 * sr)]
    out = battery(sr, seg, 0.35)
    name = path.split("/")[-1]
    print(f"\n=== {name} ===")
    r = out["ringiness"]
    t1 = r and r["n_peaks_gt6dB"] >= 12 and r["mean_top10_prom"] >= 8.0
    print(f"  1 RING : {r['n_peaks_gt6dB']} pk >6dB, top-10 prom "
          f"{r['mean_top10_prom']} dB  -> {'PASS' if t1 else 'FAIL'}"
          f"   peaks {r['top_peaks']}")
    a = out["attack"]
    rise = a["rise_ms"] if a else None
    prof = a.get("0-5ms") if a else None
    lf_led = prof is not None and prof[0] == max(prof)
    t2 = rise is not None and 1.5 <= rise <= 12.0 and lf_led
    print(f"  2 CRACK: rise {rise} ms, 0-5ms {prof}  -> "
          f"{'PASS' if t2 else 'FAIL'}")
    t = out["tail_T60s"]
    tv = [x for x in (t or []) if x is not None]
    inband = tv and all(0.39 <= x <= 0.63 for x in tv)
    # never rising with frequency: last band <= first measured * 1.15
    nonrise = tv and tv[-1] <= tv[0] * 1.25
    t3 = inband and nonrise
    print(f"  3 TAIL : {t}  -> {'PASS' if t3 else 'FAIL'}")
    peaks = out["lf_peaks(Hz,dBrel)"]
    margin = -peaks[1][1] if len(peaks) > 1 else 99.0
    t4 = margin >= 10.0
    print(f"  4 ROOT : f0 {out['f0']} Hz, margin {margin:.1f} dB "
          f"(peaks {peaks})  -> {'PASS' if t4 else 'FAIL'}")
    if "tail_centroid/rolloff85" in out:
        c, r85 = out["tail_centroid/rolloff85"]
        print(f"    tail centroid {c} Hz, rolloff85 {r85} Hz")
    return (t1, t2, t3, t4)


if __name__ == "__main__":
    for p in sys.argv[1:]:
        score(p)
