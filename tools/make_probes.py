#!/usr/bin/env python3
"""make_probes.py — generate probe pack p1 for Torque/Vocodex characterization.

Synth probes are deterministic (seeded); real-material probes are excerpted
from the local sample library (loudest-onset window, mono, 44.1k, -6 dBFS).
Output: out/clang-probes-p1/probes/*.wav + MANIFEST.tsv

Run inside the flake shell:  nix develop -c python3 tools/make_probes.py
"""

import os
import numpy as np
import soundfile as sf
from scipy import signal

SR = 44100
OUT = os.path.join(os.path.dirname(__file__), "..", "out", "clang-probes-p1", "probes")
os.makedirs(OUT, exist_ok=True)
rng = np.random.default_rng(20260721)
manifest = []


def db(x):
    return 10.0 ** (x / 20.0)


def norm_peak(x, dbfs=-6.0):
    p = np.max(np.abs(x))
    return x * (db(dbfs) / p) if p > 0 else x


def fade(x, ms_in=0.0, ms_out=10.0):
    n_in, n_out = int(SR * ms_in / 1000), int(SR * ms_out / 1000)
    if n_in > 0:
        x[:n_in] *= 0.5 - 0.5 * np.cos(np.pi * np.arange(n_in) / n_in)
    if n_out > 0:
        x[-n_out:] *= 0.5 + 0.5 * np.cos(np.pi * np.arange(n_out) / n_out)
    return x


def write(name, x, desc):
    x = np.clip(x, -1.0, 1.0)
    sf.write(os.path.join(OUT, name), x.astype(np.float64), SR, subtype="PCM_24")
    manifest.append((name, desc))
    print(f"  {name:34s} {len(x)/SR:6.2f}s  {desc}")


def silence(sec):
    return np.zeros(int(SR * sec))


# ---------------- synthetic probes ----------------

# p01 dirac
x = silence(1.0)
x[1000] = db(-6)
write("p01_dirac.wav", x, "single-sample impulse @ -6 dBFS; raw system response")

# p02 dirac level steps (threshold characterization)
levels = [-60, -48, -36, -24, -12, -6, -1]
x = silence(0.5 * len(levels) + 0.5)
for i, lv in enumerate(levels):
    x[int(SR * (0.25 + 0.5 * i))] = db(lv)
write("p02_dirac_steps.wav", x,
      f"impulses every 500 ms at {levels} dBFS; threshold/gating map")

# p03 band-limited click
x = silence(1.0)
x[1000] = 1.0
b, a = signal.butter(4, 2000 / (SR / 2), "low")
x = signal.filtfilt(b, a, x)
write("p03_click_lp2k.wav", norm_peak(x, -6), "click lowpassed @2 kHz; LF transient")

# p04/p05 noise bursts
for name, ms in [("p04_burst_5ms.wav", 5), ("p05_burst_50ms.wav", 50)]:
    n = int(SR * ms / 1000)
    x = rng.standard_normal(n)
    x = fade(x, ms_in=0.5, ms_out=ms / 4)
    x = np.concatenate([silence(0.02), norm_peak(x, -12), silence(0.8)])
    write(name, x, f"{ms} ms white-noise burst @ -12 dBFS; slap-family transient")

# p06 slow log sweep (band mapping / quasi-LTI characterization)
dur = 10.0
t = np.arange(int(SR * dur)) / SR
f0, f1 = 20.0, 20000.0
k = np.log(f1 / f0)
phase = 2 * np.pi * f0 * dur / k * (np.exp(t / dur * k) - 1)
x = np.sin(phase) * db(-12)
x = fade(x, ms_in=20, ms_out=20)
write("p06_sweep_slow.wav", np.concatenate([x, silence(0.5)]),
      "20 Hz-20 kHz log sweep, 10 s @ -12 dBFS; band-distribution mapping")

# p07 fast chirp (transient sweep)
dur = 0.03
t = np.arange(int(SR * dur)) / SR
phase = 2 * np.pi * f0 * dur / k * (np.exp(t / dur * k) - 1)
x = np.sin(phase)
x = fade(x, ms_in=0.2, ms_out=2)
write("p07_sweep_fast.wav", np.concatenate([norm_peak(x, -6), silence(0.97)]),
      "30 ms exponential chirp; transient-smearing detector")

# p08-p10 decaying tones (drum-enveloped sines)
for name, f in [("p08_tone_decay_55.wav", 55.0),
                ("p09_tone_decay_110.wav", 110.0),
                ("p10_tone_decay_220.wav", 220.0)]:
    dur, t60 = 1.5, 0.8
    t = np.arange(int(SR * dur)) / SR
    env = 10 ** (-3 * t / t60)
    x = np.sin(2 * np.pi * f * t) * env
    x = fade(x, ms_in=2, ms_out=20)
    write(name, norm_peak(x, -6), f"decaying sine {f:.0f} Hz, T60 0.8 s; pitch-tracking probe")

# p11 modal stack (membrane-ratio partials)
f0 = 110.0
ratios = [1.0, 1.594, 2.136, 2.296, 2.653, 2.918]
dur = 1.5
t = np.arange(int(SR * dur)) / SR
x = np.zeros_like(t)
for i, r in enumerate(ratios):
    t60 = 0.9 / (r ** 0.5)
    x += (1.0 / (i + 1) ** 0.8) * np.sin(2 * np.pi * f0 * r * t + rng.uniform(0, 2 * np.pi)) \
         * 10 ** (-3 * t / t60)
x = fade(x, ms_in=1, ms_out=20)
write("p11_modal_stack.wav", norm_peak(x, -6),
      "6-partial membrane-ratio stack, f0 110 Hz; inharmonic content probe")

# p12 glide kick
dur = 1.0
t = np.arange(int(SR * dur)) / SR
fk = 50.0 + (180.0 - 50.0) * np.exp(-t / 0.040)
phase = 2 * np.pi * np.cumsum(fk) / SR
x = np.sin(phase) * 10 ** (-3 * t / 0.5)
x = np.tanh(1.5 * x)
x = fade(x, ms_in=0.5, ms_out=20)
write("p12_glide_kick.wav", norm_peak(x, -3),
      "synth kick, 180->50 Hz exp glide (tau 40 ms), soft drive; glide interaction probe")

# p13 synth snare
dur = 0.8
t = np.arange(int(SR * dur)) / SR
body = (np.sin(2 * np.pi * 190 * t) + 0.6 * np.sin(2 * np.pi * 330 * t)) * 10 ** (-3 * t / 0.25)
noise = rng.standard_normal(len(t))
b, a = signal.butter(2, [1000 / (SR / 2), 8000 / (SR / 2)], "bandpass")
noise = signal.lfilter(b, a, noise) * 10 ** (-3 * t / 0.15)
x = fade(0.8 * body + 0.7 * noise, ms_in=0.5, ms_out=20)
write("p13_snare_synth.wav", norm_peak(x, -6),
      "two-mode body (190/330 Hz) + bandpassed noise; tone/noise split probe")


# ---------------- real-material excerpts ----------------

def excerpt(src, name, dur, desc, whole=False):
    data, sr = sf.read(src, always_2d=True)
    x = data.mean(axis=1)
    if sr != SR:
        g = np.gcd(SR, sr)
        x = signal.resample_poly(x, SR // g, sr // g)
    if whole:
        seg = x[: int(SR * dur)]
    else:
        onset = int(np.argmax(np.abs(x)))
        start = max(0, onset - int(0.005 * SR))
        seg = x[start: start + int(SR * dur)].copy()
    seg = fade(seg, ms_in=0.0 if not whole else 2, ms_out=15)
    write(name, norm_peak(seg, -6), desc)


LIB = "/Users/sam/Dropbox/Samples"
excerpt(f"{LIB}/Stems/catsum/catsum Kick.wav", "r01_kick_catsum.wav", 0.7,
        "real kick (catsum stem, loudest hit); primary Torque subject")
excerpt(f"{LIB}/Stems/catsum/catsum Snare.wav", "r02_snare_catsum.wav", 0.7,
        "real snare (catsum stem, loudest hit); primary Torque subject")
excerpt(f"{LIB}/Stems/catsum/catsum Hat.wav", "r03_hat_catsum.wav", 0.4,
        "real hat (catsum stem, loudest hit); HF/threshold subject")
excerpt(f"{LIB}/Stems/Way Elm Stems/Kick.wav", "r04_kick_wayelm.wav", 0.8,
        "real kick (Way Elm stem, loudest hit); secondary kick voicing")
excerpt(f"{LIB}/SAMUEL MARQUIS - CIRCLES/c8_fraud/metal birds.wav",
        "r05_metalbirds.wav", 1.5,
        "metallic texture excerpt (metal birds); clang-family subject")
excerpt(f"{LIB}/Sound Design/Rec-22.11.15-18h07m21s bt minimal kicks.wav",
        "r06_kick_btrec.wav", 0.8,
        "recorded minimal kick (loudest hit); acoustic-ish LF subject")
excerpt(f"{LIB}/Drums/Breaks/rhythm-lab.com_amen_vol.1/WAV/cw_amen01_175.wav",
        "r07_amen_loop.wav", 3.0,
        "amen break excerpt (polyphonic material); full-loop behavior", whole=True)

with open(os.path.join(OUT, "MANIFEST.tsv"), "w") as f:
    f.write("file\tdescription\n")
    for name, desc in manifest:
        f.write(f"{name}\t{desc}\n")

print(f"\n{len(manifest)} probes -> {os.path.abspath(OUT)}")
