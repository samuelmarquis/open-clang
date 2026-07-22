import numpy as np, soundfile as sf
def stats(p):
    x, sr = sf.read(p)
    env = np.abs(x); pk = env.max()
    above = np.where(env > pk * 1e-2)[0]
    ring = 1000 * (above[-1] - above[0]) / sr
    tail = x[len(x)//4:]
    X = np.abs(np.fft.rfft(tail))**2; f = np.fft.rfftfreq(len(tail), 1/sr)
    dom = f[np.argmax(X)]
    Xa = np.abs(np.fft.rfft(x))**2; fa = np.fft.rfftfreq(len(x), 1/sr)
    cent = (fa*Xa).sum()/Xa.sum()
    return dom, ring, cent
pairs = [("out/parity-m3/rust_kick_brace0.wav", "out/batch-005/b005_membrane_f36_brace0_v0.95.wav"),
         ("out/parity-m3/rust_kick_brace1.wav", "out/batch-005/b005_membrane_f36_brace1_v0.95.wav"),
         ("out/parity-m3/rust_plate_casc.wav", "out/batch-005/b005_plate_f50_casc_brace0_v0.95.wav")]
print(f"{'render (rust/lab)':36s} {'dom Hz':>14s} {'ring(-40dB) ms':>16s} {'centroid Hz':>14s}")
for r, l in pairs:
    dr, rr, cr = stats(r); dl, rl, cl = stats(l)
    print(f"{r.split('/')[-1]:36s} {dr:6.1f}/{dl:6.1f} {rr:7.0f}/{rl:7.0f} {cr:7.0f}/{cl:7.0f}")
