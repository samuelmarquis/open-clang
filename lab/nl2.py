"""NL2 — contact satellites (rattle/buzz), time-domain lab implementation.

FROZEN 2026-07-22 with lab/engine.py (see its header). The Rust port
(M3.2) is canonical; known drift: offline two-pass seat calibration
here vs the Rust engine's at-trigger analytic estimate.
"""

_ORIGINAL_DOC = """

The verified mechanism (research 01 §A.1/A.2): rattle is repeated
satellite-resonator collisions gated by the body's own motion, never a
noise layer. v0: modal bank stepped per-sample as two-pole resonators
(vectorized over modes); satellites are mass-spring objects in
penalty contact (Hertzian p^1.5) with the surface displacement at
their seat position; contact force feeds BACK into the bank (two-way).

Includes the deliberate IMPOSTER for A/B: bandpassed noise gated by
the surface envelope — the thing the verified mechanism is not.
"""

import numpy as np
from engine import (SR, PHI, ARCHETYPES, strike_weights, listen_weights,
                    mallet_spectrum, t60_of)


def _bank_coeffs(freqs, t60):
    r = np.exp(-6.9078 / (np.maximum(t60, 1e-3) * SR))
    c = 2.0 * r * np.cos(2 * np.pi * freqs / SR)
    return c, r * r


def render_hit_td(arch="membrane", f0=110.0, velocity=0.9, position=0.35,
                  stiffness=0.5, t60_base=0.8, tilt=1.4, dur=1.6,
                  satellites=None, imposter_noise=False, seed=23, dust=None):
    """satellites: list of dicts {fs, t60, seat, rest, level} —
    fs: satellite resonance Hz; seat: its position on the surface [0,1];
    rest: hover gap as a fraction of peak surface displacement
    (small=tight buzz, large=loose rattle); level: radiated gain."""
    freqs, mn = ARCHETYPES[arch](f0)
    keep = freqs < SR * 0.45
    freqs, mn = freqs[keep], (mn[0][keep], mn[1][keep])
    K = len(freqs)

    w_str = strike_weights(arch, mn, position)
    w_lst = listen_weights(arch, mn)
    amp = w_str * mallet_spectrum(freqs, velocity, stiffness)
    c1, c2 = _bank_coeffs(freqs, t60_of(freqs, t60_base, tilt))

    N = int(SR * dur)
    # excitation force: Hann pulse, contact time as in engine.mallet_spectrum
    tau = 0.004 * (1.0 - 0.75 * stiffness) / (0.35 + 0.65 * velocity)
    npulse = max(8, int(SR * tau))
    force = np.zeros(N)
    force[:npulse] = velocity * (0.5 - 0.5 * np.cos(2 * np.pi * np.arange(npulse) / npulse))

    # linear pre-pass to calibrate displacement scale at each satellite seat
    sats = satellites or []
    w_seat = [strike_weights(arch, mn, s["seat"]) for s in sats]
    if sats:
        y1 = np.zeros(K)
        y2 = np.zeros(K)
        peak_seat = np.zeros(len(sats))
        for n in range(N):
            y0 = c1 * y1 - c2 * y2 + amp * force[n]
            y2, y1 = y1, y0
            for j, ws in enumerate(w_seat):
                peak_seat[j] = max(peak_seat[j], abs(float(ws @ y0)))
        peak_seat = np.maximum(peak_seat, 1e-9)

    # satellite state + coefficients
    zs = np.array([s["rest"] * peak_seat[j] for j, s in enumerate(sats)]) if sats else np.zeros(0)
    vs = np.zeros(len(sats))
    om = np.array([2 * np.pi * s["fs"] for s in sats])
    ze = np.array([6.9078 / (s["t60"] * 2 * np.pi * s["fs"]) for s in sats])
    rest = zs.copy()
    kc = np.array([80.0 * (om[j] ** 2) for j in range(len(sats))])  # penalty stiffness
    dt = 1.0 / SR

    rng = np.random.default_rng(seed)
    y1 = np.zeros(K)
    y2 = np.zeros(K)
    out = np.zeros(N)
    sat_out = np.zeros(N)
    contacts = 0
    for n in range(N):
        f_ext = amp * force[n]
        seat_disp = np.array([float(ws @ y1) for ws in w_seat]) if sats else np.zeros(0)
        if sats:
            pen = seat_disp - zs  # surface pushing up into satellite
            hit = pen > 0
            Fc = np.where(hit, kc * np.abs(pen) ** 1.5, 0.0)
            Fc = np.minimum(Fc, 50.0)  # sanity clamp
            contacts += int(np.count_nonzero(hit))
            # satellite dynamics (symplectic Euler): own resonance + contact kick
            acc = -(om ** 2) * (zs - rest) - 2 * ze * om * vs + Fc
            vs += dt * acc
            zs += dt * vs
            # reaction into the bank at each seat
            for j, ws in enumerate(w_seat):
                f_ext = f_ext - 0.02 * Fc[j] * ws
        y0 = c1 * y1 - c2 * y2 + f_ext
        y2, y1 = y1, y0
        out[n] = float(w_lst @ y0)
        if sats:
            sat_out[n] = float(np.sum([s["level"] * vs[j] for j, s in enumerate(sats)]))

    mix = out / max(np.max(np.abs(out)), 1e-12)
    if sats and np.max(np.abs(sat_out)) > 0:
        mix = mix + 0.5 * sat_out / np.max(np.abs(sat_out))

    if dust:
        # THE DUST LAYER (Batch 003 verdict: the 'imposter' promoted to a
        # feature — statistical limit of many micro-contacts, snare-bed
        # texture). Controls: level, thr_db (activity threshold),
        # follow (loudness->dust law: 1 linear, >1 expansion).
        from scipy import signal as sg
        env = np.abs(out) / max(np.max(np.abs(out)), 1e-12)
        k = int(SR * 0.004)
        env = np.convolve(env, np.ones(k) / k, mode="same")
        thr = 10 ** (dust.get("thr_db", -40) / 20)
        g = np.maximum(env - thr, 0.0) / (1.0 - thr)
        g = g ** dust.get("follow", 1.0)
        noise = rng.standard_normal(N)
        b, a_ = sg.butter(2, [dust.get("lo", 1500) / (SR / 2),
                              dust.get("hi", 6500) / (SR / 2)], "bandpass")
        mix = mix + dust.get("level", 0.5) * sg.lfilter(b, a_, noise) * g

    if imposter_noise:
        # THE IMPOSTER: noise gated by the surface envelope (what rattle isn't)
        env = np.abs(out) / max(np.max(np.abs(out)), 1e-12)
        k = int(SR * 0.004)
        env = np.convolve(env, np.ones(k) / k, mode="same")
        noise = rng.standard_normal(N)
        from scipy import signal as sg
        b, a = sg.butter(2, [1500 / (SR / 2), 6500 / (SR / 2)], "bandpass")
        mix = mix + 0.5 * sg.lfilter(b, a, noise) * env

    n_atk = int(SR * 0.0015)
    mix[:n_atk] *= np.linspace(0, 1, n_atk)
    peak = np.max(np.abs(mix))
    return (mix / peak) * 10 ** (-3 / 20), contacts
