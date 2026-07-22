# 02 — Waves Torque, measured

> **CORRECTION (p2/p3, 2026-07-21)**: §F1's "added lines sit within a
> few % of exact ratios" understated the truth — the shift is **exact
> to ~0.15 cents**; the apparent deviation was a sideband family,
> finally resolved (p3) as a **heterodyne at 2·f₀ ± f_shifted**.
> Focus **suppresses** the untracked partial rather than selecting the
> tracked one. See `04-p2-measured.md` Part I and
> `05-p3-measured.md` §R1.

*open-clang research corpus, entry 02. Empirical characterization from
probe pack p1 (2026-07-21): 136 renders produced on sam-pc by the
operator (REAPER, parameters set via VST3 API with read-back
verification, zero landing failures; full log in
`out/clang-probes-p1-return/`). Rig verified transparent: j00 bypass
renders null bit-perfectly against the probes (maxAbsDiff = 0) after
the operator caught and eliminated REAPER's default item fade-in.
Analysis: `tools/analyze_p1.py`. Clean-room: measured I/O + public
docs only.*

## Verified plumbing

- **PDC: exactly as documented.** Standard component 32 samples
  @44.1k; Live component 0. (Manual claim → measured, confirmed.)
- **Parameter laws** (operator census): Torque ±1200¢ linear,
  Threshold −70..0 dB linear, **Focus 98→988 Hz logarithmic**,
  **Speed 15→50 ms logarithmic**. Defaults left in place: Trim 0 dB,
  Output −6 dB.

## Findings

### F1. Dual-path resynthesis: originals kept, a shifted copy added

On the modal-stack probe (partials at 110/175.3/235.0/252.6/291.8/
321.0 Hz), T±1200 F300 renders retain the original partials at their
original frequencies (±0.5 Hz) **and add new spectral lines near
shift-ratio × the low partials**:

| render | measured peaks (Hz) |
|---|---|
| j00 (dry) | 110.0, 175.3, 235.0, 291.9 |
| T−1200 F300 | **86.0, 121.1**, 176.0, 235.4, 291.9, 340.0 |
| T+1200 F300 | 109.7, 174.5, **218.5**, 290.9, **343.2, 458.3** |

218.5 ≈ 2×110; 343.2 ≈ 2×175.3 (−2%); 458.3 ≈ 2×235 (−2.5%); 86.0 ≈
175.3/2 (−2%). The added components sit within a few percent of exact
ratio-shifted copies — Torque **extracts a Focus-region component,
transposes it, and remixes it with an intact residual**. It does not
transpose the signal.

### F2. Steady tonal content passes untouched

The 10 s log sweep through T±1200 (Th −70, always engaged): output
ridge / input frequency median **0.994 / 0.995** (p10–p90 within
±2%). A tone is not re-pitched even at full ±1200¢. Torque's audible
pitch action exists only where the material's energy is concentrated
in a resonance the Focus band can grab — which is exactly what a drum
is.

### F3. Action is localized below ~1 kHz and follows Focus

Snare octave-band deltas at T−1200 (dB vs dry): F98 → +9.8 @62–125,
+8.8 @125–250, −4.9 @250–500; F300 → +10.7/+8.5/−4.8 same bands;
F900 → boost moves up (+3.0 @62–125, **+10.9 @125–250**). Every band
above 1 kHz: ≤0.8 dB change. Plate:
`out/analysis-p1/plates/torque-focus-delta.png`.

### F4. Transients survive; downshift adds ring

Dirac through T±1200: envelope peak time shift **0 samples**; click
(2 kHz LP) peak −4 samples. T−1200 stretches the dirac's significant
response 1,270 → 2,718 samples (≈29 → 62 ms) — the added sub-octave
component rings; T+1200 does not lengthen it (1,246). No pre-ring
observed anywhere (consistent with 32-sample latency and the j00
nulls).

### F5. Threshold is a progressive per-hit gate

Level-stepped diracs (−60…−1 dBFS), processed-minus-dry RMS per hit:
engagement is strictly ordered by Threshold and **soft** — hits just
above threshold process barely (−157 dB diff), rising smoothly with
level above it (to −56 dB diff at −1 dBFS, Th −70). No binary gate,
no chatter.

### F6. On drum-like material the re-pitch is strong

Glide-kick LF ridge (0–250 ms, 30–300 Hz), out/dry frequency ratio:
T−1200 F98 → **0.821**; T−1200 F300 → 0.857; T+1200 F98 → **2.242**
(ridge captured by the shifted copy). With energy concentrated in the
focus resonance, the shifted path dominates perception even though
F2 shows tonal material passing through unchanged.

### F7. Standard vs Live

On the fast chirp, gain-fitted null vs dry: std g=0.70, residual
−3.1 dB; live g=0.45, residual −0.1 dB — the standard component
preserves substantially more dry-correlated structure (consistent
with its "phase coherent" designation); both align at 0-lag.

## Implications for open-clang

1. **We get Torque's trick for free, natively.** "Shifted resonance +
   intact residual" is exactly what per-mode retuning of a modal bank
   does. A `retune` transform on the low-mode group (ratio ±1200¢,
   soft-bounded region ≈ our Focus analog) reproduces the architecture
   without any analysis stage — and our version can *replace* rather
   than *add* the shifted copy, or blend (F1 suggests Torque blends).
2. The **soft per-hit threshold** (F5) and **15–50 ms process
   time-constant** range are sensible defaults to copy for any
   audio-in (effect-mode) retuning path.
3. The 32-sample budget proves resonance-localized re-pitch needs no
   FFT; a handful of tracking band-passes suffices. Keeps the
   effect-mode path zero-ish latency.

## Caveats / open

- Grid covered one instance, default Trim/Output; no stereo material.
- The few-% deviation of added lines from exact ratios (F1) is
  unexplained — could be tracking, could be intentional detune;
  a p2 probe (single decaying partial, fine T steps) would resolve it.
- Ridge methods can hop between components on dense spectra (the
  2.242 in F6 is a capture artifact of a *correct* effect, not a
  +1400¢ shift).
