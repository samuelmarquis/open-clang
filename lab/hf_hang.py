# hf_hang.py — M10.5 diagnostic: per-band tail T60 of render files.
# Reuses the snare_autopsy battery. Usage:
#   python lab/hf_hang.py out/diag-hfhang/*.wav
# Prints one line per file: T60s for the six TAIL_BANDS over the
# 60-350 ms post-onset window (same window as the autopsy).

import sys
import numpy as np

sys.path.insert(0, "lab")
from snare_autopsy import load_mono, tail_t60, TAIL_BANDS  # noqa: E402

def main(paths):
    names = [f"{lo/1000:g}-{hi/1000:g}k" for lo, hi in TAIL_BANDS]
    print(f"{'file':38s}  " + "  ".join(f"{n:>9s}" for n in names))
    for p in paths:
        sr, x = load_mono(p)
        pk = np.abs(x).max() + 1e-30
        i = int(np.argmax(np.abs(x) > pk * 0.01))
        i = max(0, i - int(0.002 * sr))
        seg = x[i:i + int(0.5 * sr)]
        t = tail_t60(sr, seg, t_to=0.35)
        base = p.split("/")[-1].replace(".wav", "")
        if t is None:
            print(f"{base:38s}  (segment too short)")
            continue
        cells = ["      inf" if v is None else f"{v:9.3f}" for v in t]
        print(f"{base:38s}  " + "  ".join(cells))

if __name__ == "__main__":
    main(sys.argv[1:])
