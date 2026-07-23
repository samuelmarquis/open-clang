# null_ab.py — M12 drift gate: peak |a−b| relative to peak |a|, in dB.
# Usage: python null_ab.py A.wav B.wav  (prints one line, exit 1 if > -80 dB)
import sys
import numpy as np
from scipy.io import wavfile


def load(path):
    sr, x = wavfile.read(path)
    if x.dtype == np.int16:
        x = x.astype(np.float64) / 32768.0
    elif x.dtype == np.int32:
        x = x.astype(np.float64) / 2147483648.0
    else:
        x = x.astype(np.float64)
    return sr, x


def main():
    a_path, b_path = sys.argv[1], sys.argv[2]
    sra, a = load(a_path)
    srb, b = load(b_path)
    assert sra == srb, "sample-rate mismatch"
    n = min(len(a), len(b))
    a, b = a[:n], b[:n]
    pa = np.abs(a).max()
    pd = np.abs(a - b).max()
    db = 20 * np.log10(pd / pa) if pd > 0 and pa > 0 else -np.inf
    tag = "PASS" if db <= -80.0 else "FAIL"
    print(f"{tag} {db:8.1f} dB  {a_path.split('/')[-1]}")
    sys.exit(0 if db <= -80.0 else 1)


if __name__ == "__main__":
    main()
