"""open-clang prototype lab — FROZEN 2026-07-22.

The Rust engine (rt/engine, `clg-engine`) is the sole canonical
implementation as of M3 parity (LISTENING-LOG: "keep this, I like
this"). This module is kept as the algorithm-exploration lab and
historical record for Batches 001-005; do not extend it.

Known drift vs Rust: closed-form additive synthesis vs per-sample
coupled-form resonators; coherent-only cascade (Rust defaults
stochastic with a coherent toggle); analytic bank-energy glide vs
running-max TD tracker; `brace` macro here vs granular
coupling/choke/tension/t60 params in Rust; mono only.
"""

import numpy as np

SR = 44100
PHI = 0.618033988749895


# ---------------------------------------------------------------- archetypes

def modes_membrane(f0, n_axial=6, aspect=0.94):
    """Rectangular membrane: f ~ sqrt(m^2 + (g*n)^2). Near-harmonic-ish low end,
    dense inharmonic upper cloud. Returns (freqs, (m, n)) unscaled to f0 of (1,1)."""
    m, n = np.meshgrid(np.arange(1, n_axial + 1), np.arange(1, n_axial + 1))
    m, n = m.ravel(), n.ravel()
    r = np.sqrt(m ** 2 + (aspect * n) ** 2)
    return f0 * r / r.min(), (m, n)


def modes_plate(f0, n_axial=6, aspect=0.79):
    """Simply-supported stiff plate: f ~ m^2 + (g*n)^2 — the clang spectrum."""
    m, n = np.meshgrid(np.arange(1, n_axial + 1), np.arange(1, n_axial + 1))
    m, n = m.ravel(), n.ravel()
    r = m ** 2 + (aspect * n) ** 2
    return f0 * r / r.min(), (m, n)


BAR_RATIOS = np.array([1.0, 2.756, 5.404, 8.933, 13.345, 18.638])


def modes_bar(f0):
    """Free-free bar partials."""
    k = np.arange(1, len(BAR_RATIOS) + 1)
    return f0 * BAR_RATIOS, (k, np.ones_like(k))


ARCHETYPES = {"membrane": modes_membrane, "plate": modes_plate, "bar": modes_bar}


# ---------------------------------------------------------------- weights

def strike_weights(arch, mn, pos):
    """Mode weights from strike position pos in [0,1] (edge->center diagonal)."""
    m, n = mn
    if arch == "bar":
        return np.abs(np.cos(m * np.pi * pos)) + 0.05
    px = 0.08 + 0.42 * pos          # walk the diagonal, edge-ish to center-ish
    py = 0.06 + 0.38 * pos
    return np.abs(np.sin(m * np.pi * px) * np.sin(n * np.pi * py)) + 0.01


def listen_weights(arch, mn, lpos=0.31):
    return strike_weights(arch, mn, lpos)


# ---------------------------------------------------------------- exciters

def mallet_spectrum(freqs, velocity, stiffness):
    """|force spectrum| of a Hann contact pulse at each mode freq.
    Contact time shrinks with stiffness and velocity (harder+faster -> brighter)."""
    tau = 0.004 * (1.0 - 0.75 * stiffness) / (0.35 + 0.65 * velocity)  # seconds
    n = max(8, int(SR * tau))
    pulse = 0.5 - 0.5 * np.cos(2 * np.pi * np.arange(n) / n)
    spec = np.abs(np.fft.rfft(pulse, n=1 << 16))
    spec /= spec.max()
    idx = np.clip((freqs / (SR / 2) * (len(spec) - 1)).astype(int), 0, len(spec) - 1)
    return velocity * spec[idx]


def burst_spectrum(freqs, velocity, seed=7, lo=400.0, hi=9000.0):
    """Noise-burst excitation: flat-ish in [lo, hi] with seeded ripple."""
    rng = np.random.default_rng(seed)
    ripple = 0.6 + 0.4 * rng.random(len(freqs))
    band = 1.0 / (1.0 + (lo / np.maximum(freqs, 1.0)) ** 2) \
         / (1.0 + (np.maximum(freqs, 1.0) / hi) ** 2)
    return velocity * ripple * band


# ---------------------------------------------------------------- damping

def t60_of(freqs, t60_base, tilt=1.2, f_ref=900.0):
    """Frequency-dependent decay: low modes ring, highs die by `tilt`."""
    return t60_base / (1.0 + (freqs / f_ref) ** tilt)


# ---------------------------------------------------------------- render

def render_hit(arch="membrane", f0=110.0, exciter="mallet", velocity=0.8,
               position=0.4, stiffness=0.4, t60_base=1.0, tilt=1.2,
               dur=None, seed=11, glide_st=0.0, glide_fake=False,
               listen_pos=0.31, n_axial=6, out_tilt_db_oct=0.0,
               cascade_amt=0.0, cascade_tau=0.05, cascade_split=5.0,
               cascade_static=False, cascade_attack=0.0,
               cascade_conserve=False, brace=None):
    """glide_st > 0 enables NL1 (energy-tracked stiffening): mode freqs scale
    as sqrt(1 + (r^2-1) * velocity^2 * E(t)), r = 2^(glide_st/12), with E(t)
    the bank's own normalized decaying energy — hard hits start sharp and
    fall as the object calms. glide_fake=True instead applies a uniform
    velocity-independent exponential pitch envelope (tau 60 ms): the
    909-style imposter, rendered for A/B honesty.

    out_tilt_db_oct: output modal-gain curve (the transect's gain lane,
    Batch 003b verdict) — dB per octave relative to the lowest mode;
    negative = fundamental-dominant voicing.
    cascade_*: NL3 spectral cascade — modes above cascade_split*f0 get
    injected energy that BUILDS UP (tau, size-scaled) scaled by the low
    bank's own energy; cascade_static=True renders the A/B imposter
    (same energy, present from onset — 'just brighter')."""
    if arch == "bar":
        freqs, mn = ARCHETYPES[arch](f0)
    else:
        freqs, mn = ARCHETYPES[arch](f0, n_axial=n_axial)
    keep = freqs < SR * 0.45
    freqs, mn = freqs[keep], (mn[0][keep], mn[1][keep])

    w = strike_weights(arch, mn, position) * listen_weights(arch, mn, listen_pos)
    if out_tilt_db_oct != 0.0:
        w = w * (freqs / freqs.min()) ** (out_tilt_db_oct / 6.02)

    # BRACING — the Houdini axis. brace in [0,1]: 0 = unbraced (the body
    # keeps the blow: full coupling, low modes ring long), 1 = braced
    # (tensed: energy reflected, slight pitch-up from tension, early choke,
    # dry thwack). None = axis disengaged (pre-005 behavior).
    if brace is not None:
        b = float(np.clip(brace, 0.0, 1.0))
        stiffness = min(1.0, stiffness + 0.30 * b)   # tense surface = harder contact
        freqs = freqs * (1.0 + 0.05 * b)             # tension pitch-up
    if exciter == "mallet":
        a = w * mallet_spectrum(freqs, velocity, stiffness)
    else:
        a = w * burst_spectrum(freqs, velocity, seed=seed)
    t60 = t60_of(freqs, t60_base, tilt)

    if brace is not None:
        b = float(np.clip(brace, 0.0, 1.0))
        a = a * (1.0 - 0.55 * b)                     # coupling: reflected energy
        t60 = t60 * (1.0 - 0.45 * b)                 # tense = shorter overall
        low = freqs < 4.0 * f0
        t60[low] = t60[low] * (1.0 + 0.9 * (1.0 - b))  # unbraced: the body keeps it

    if dur is None:
        dur = float(np.clip(1.15 * t60.max(), 0.5, 3.5))
    t = np.arange(int(SR * dur)) / SR

    # deterministic per-mode phases (golden-ratio sequence, position-salted)
    ph = 2 * np.pi * ((np.arange(len(freqs)) * PHI + position * 7.13) % 1.0)

    envs = np.exp(-6.9078 * t[None, :] / np.maximum(t60, 1e-3)[:, None])  # (K, T)
    if brace is not None and brace > 0:
        # early choke that releases: the caught blow
        envs = envs * np.exp(-t * (14.0 * float(brace)) * np.exp(-t / 0.05))[None, :]

    if glide_st > 0 and not glide_fake:
        # NL1: bank energy from the mode envelopes themselves
        E = ((a[:, None] * envs) ** 2).sum(axis=0)
        E /= max(E[0], 1e-12)
        r2 = 2.0 ** (glide_st / 6.0)          # r^2 for glide_st semitones
        mult = np.sqrt(1.0 + (r2 - 1.0) * (velocity ** 2) * E)
    elif glide_fake:
        mult = 2.0 ** ((glide_st / 12.0) * np.exp(-t / 0.060))
    else:
        mult = None

    # NL3 prep: buildup curve + (optionally) energy-CONSERVING depletion of
    # the low bank — the transfer, not an addition (Batch 004 verdict: tau
    # read as decay time, not size; conservation is the candidate fix —
    # bigger object = lows hold longer before surrendering energy upward).
    casc = None
    if cascade_amt > 0:
        hi = freqs > cascade_split * f0
        if hi.any() and (~hi).any():
            if cascade_static:
                buildup = np.ones_like(t)
            else:
                buildup = cascade_attack + (1.0 - cascade_attack) \
                          * (1.0 - np.exp(-t / cascade_tau))
            if cascade_conserve:
                dep = np.sqrt(np.clip(
                    1.0 - 0.8 * cascade_amt * (velocity ** 2) * buildup,
                    0.05, 1.0))
                envs[~hi] = envs[~hi] * dep[None, :]
            casc = (hi, buildup)

    x = np.zeros_like(t)
    if mult is None:
        for fk, ak, tk_env, pk in zip(freqs, a, envs, ph):
            x += ak * np.sin(2 * np.pi * fk * t + pk) * tk_env
    else:
        dphase = 2 * np.pi * mult / SR  # shared shape; scaled per mode
        for fk, ak, tk_env, pk in zip(freqs, a, envs, ph):
            x += ak * np.sin(np.cumsum(fk * dphase) + pk) * tk_env

    if casc is not None:
        hi, buildup = casc
        E_low = ((a[~hi][:, None] * envs[~hi]) ** 2).sum(axis=0)
        E_low /= max(E_low[0], 1e-12)
        inj = cascade_amt * (velocity ** 2) * max(np.median(a[~hi]), 1e-9)
        w_hi = (freqs[hi] / f0) ** -0.3
        for fk, tk_env, wk, pk in zip(freqs[hi], envs[hi], w_hi, ph[hi]):
            x += inj * wk * np.sin(2 * np.pi * fk * t + pk) \
                 * buildup * E_low * tk_env

    # 1.5 ms attack ramp: the force pulse's rise, not a click
    n_atk = int(SR * 0.0015)
    x[:n_atk] *= np.linspace(0, 1, n_atk)
    peak = np.max(np.abs(x))
    return (x / peak) * 10 ** (-3 / 20) if peak > 0 else x
