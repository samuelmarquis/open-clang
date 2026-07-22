# 04 — Probe pack p2, measured (Torque fine structure; Vocodex mechanism resolved)

*open-clang research corpus, entry 04. Supersedes the mechanism claims
of entries 02 §F1-caveat and 03 (hypothesis section) where noted.
Source: `clang-probes-p2-return.zip` (97 renders: 16 Torque via REAPER
API-driven, j00 bit-null QC passed; 81 Vocodex via the Live 12
concat/slice method, offset 0 verified on all 27 passes, Warp
confirmed off). Operator evidence files quoted throughout; analysis:
`tools/analyze_p2.py`, `tools/analyze_p2_map.py`. The operator
delivered 27 Vocodex settings against 17 requested, including a full
envelope-follower grid, band count/width/order extremes, and four
per-band maps.*

---

## Part I — Torque

### T-1. The shift ratio is EXACT; 02's "few-% deviation" is retracted

Single 220 Hz partial, ten Torque values, added-line frequencies
measured to ~0.01 Hz (operator, 65536-pt FFT + parabolic
interpolation; independently spot-verified here): **every shift lands
within ±0.02 Hz (~0.15 cents) of the exact ratio** (e.g. −1200 →
110.00, +700 → 329.61 vs 329.63 expected, +1200 → 439.99).

### T-2. The apparent deviation was a fixed +440 Hz sideband

All 14 q01/q02 renders carry an added line at **f_shifted + 440 Hz**,
invariant to shift amount and Focus (tested F=150/220/330). Verified
here on T+1200: lines at 440.00 (0 dB), 220.02 (residual, −26.7),
659.98 (= 220+440, −32.4), 880.00 (= 440+440, −35.4); on T−1200:
110.00 (0 dB), 330.0 (= 3×110, −44.4), 550.0 (= 110+440, −40.3).
A fixed-Hz offset misread as a ratio is exactly a "few-% error that
grows with shift" — which is how 02 misread it from the p1 grid.
**RESOLVED (p3)**: the f₀ = 175 probe settles it — the offset is
**2·f₀**, and the full structure is sum-and-difference sidebands
2·f₀ ± f_shifted (heterodyne). See `05-p3-measured.md` §R1.

### T-3. Focus suppresses; it does not select

Two partials (220 + 330), Focus placed *on* 330: the shifted line is
still derived from **220** (−700 → 146.7, not 330→220.2). What Focus
changed was the **fate of the unshifted 330**: suppressed below
−45 dB at F=330, surviving at −24 dB at F=150. Focus is a
selectivity/rejection window around the *output* region, not a
"which component do I grab" selector — the tracker appears to lock
the dominant/fundamental component regardless. Secondary: with two
partials the shifted line wanders 0.2–0.4 Hz (tracker pulled by the
second component); with one partial it is 0.02 Hz-exact.

### Torque model for open-clang (updated)

Pitch-tracked fundamental → exact-ratio resynthesis (sub-cent) +
residual mix + a fixed-offset intermodulation family (character, or
artifact — p3 tells). Focus = output-region emphasis/rejection. Our
modal Retune knob keeps the earlier design; T-3 adds one idea worth
stealing: **a rejection skirt around the retuned group** (Torque's
F=330 suppression is audibly "cleaning" the region it retunes into).

---

## Part II — Vocodex (clean path: SG 0, MOD/CAR pass-through 0, noise 0)

### V-1. Corrections on the record

- **03's central hypothesis is falsified** (operator Step 0, twice
  confirmed): MOD and CAR readouts are truly 0; CAR is not even a
  control (23 params total; exactly one pitch parameter). The octave
  is **param 17, `Modulator pitch shift` (the ♂/♀ knob), pinned at
  −12 st — the extreme of its ±12 st range**.
- **p1's dry contamination re-explained**: WET is a *level* (0.0 dB
  unity), not a blend. The dry in p1 renders was **MOD pass-through
  at 80%** (+ CAR pass-through 22%) — autovocoding deliberately mixes
  most of the raw modulator into its output. p1-derived spectral
  findings (03 §F1, §F2) were contaminated by that leak and are
  superseded below.
- The operator correctly identified that p2's scripted "clean path"
  (WET to max) would not have cleaned anything, and substituted
  pass-through zeroing — documented deviation, accepted.

### V-2. The mechanism, measured (three facts)

Sweep spectrograms per shift setting (plate:
`out/analysis-p1/plates/vocodex-p2-sweep-spectrograms.png`):

1. **Output frequency placement belongs to band *placement*, not the
   pitch knob.** For tonal input the vocoded output sits on the input
   line at −12/0/+12 st alike (gated dominant-frequency ratio 0.987/
   0.967/0.995). But `bdcram200` (all bands dragged below ~200 Hz)
   relocates output wholesale: ratio median **0.473**, measured band
   centers 18.8/70.0 Hz only. Where the sound *lives* is the
   distribution curve. Where the sound *comes from* is the input.
2. **The modulator-pitch knob is an alignment/texture control, and
   its effect is enormous.** At 0 st (analysis aligned with
   synthesis) the sweep output is **~79× louder** than at −12 st,
   with a dense harmonic fan; at ±12 st the vocoder's output
   collapses to a thin pitch-locked trace. On the kick, the spectral
   *shape* swings hard: the 31–62 Hz band sits 19.6 dB below 62–125
   at −12 st, 22.7 dB at 0 st, but only **6.6 dB at +12 st** — the
   +12 setting is the sub-heavy one on this routing. `autovocoding`
   at −12 st is therefore choosing maximum *misalignment* in the
   upward-opening direction: a deliberately starved, upper-region
   band-opening voicing over an 80% raw-modulator bed.
3. **The pitch lock suggests the "Carrier tone" carrier tracks the
   modulator's pitch** (output glued to the input line in every
   setting; level, not frequency, responds to the knob). Hypothesis,
   testable in one p3 render: carrier = noise → output should sit at
   band centers and the knob's alignment effect should become
   frequency-visible.

### V-3. Raw envelope law (the p1 gap, closed)

`p02_dirac_steps`, clean path: impulses at −60 **and −48 dBFS produce
no output at all**; from −36 dBFS the per-hit wet RMS rises smoothly
−116 → −46 dB across −36…−1 dBFS (slope ≈ 2 dB/dB — expansion-like).
Envelope ranges (panel): Hold 0–250 ms (preset 0.9), Attack 0–1000 ms
(preset 2.0), Release 0–2000 ms (preset 30). **Attack and Hold sit at
their effective floor in the preset** (moving them to 0 changes
output by 0.1–0.3%). The **MIN TIMES clamp is one of the largest
single effects in the plugin** (84–93% at nominal-zero envelope
times): the preset's "fast" envelope is really the clamp's floor.
Measured decay: release 30 ms → 29.0 ms observed; release 0 +
clamp on → 3.99 ms; clamp off → 10.52 ms (operator-reported
anomaly, unexplained; treat the clamp-off point as uncertain).

### V-4. Structural ranking (operator effect-size table, confirmed direction)

Band **count** (5 vs 47) and band **width** dominate everything —
including every envelope control. Band **order** (skirt steepness)
matters at the bottom end: order 1 rings 249 ms vs 171 ms (order 2)
on the kick at −40 dB — shallow skirts = overlap = longer effective
ring, *the* clang-relevant band parameter. Unison: **unmeasured**
(bit-identical null; the modulator-unison *order* control was likely
at 1 — p3 must raise it first). Saturation: skipped by Sam's
judgement ("exactly what you'd expect"), uncharacterized.

### Vocodex → open-clang (replaces 03's implication #1)

The tactile-low-end recipe, correctly attributed:

1. **Placement is the instrument.** The transect (mode placement
   curve) decides where output lives — cramming genuinely relocates
   energy (V-2.1). Floor control stays (03's §F2 voicing observation
   survives, now uncontaminated: clean band floor ≈ 83 Hz raised vs
   18.8 Hz linear).
2. **Alignment detune is a first-class texture axis.** An
   "excitation↔mode alignment" control (which modes a given input
   region opens, detunable ±1 octave) buys a measured 79×
   level swing and a 13–16 dB sub-shape swing — as a *performance*
   control this is the bracing axis's spectral sibling. Added to the
   architecture as the **Alignment** control on the audio-in path.
3. **Resonance economy over resonance count; skirt order as a ring
   control** (V-4) — maps directly onto our per-mode Q/order and
   validates exposing skirt steepness, not just T60.
4. The 80% pass-through lesson: the reference sound is a *blend* —
   our effect mode ships with a real dry path, not an afterthought.

---

### V-5. Addendum (Sam, 2026-07-21): autovocoding is SELF-vocoding

Sam clarifies the actual patch topology: **modulator and carrier are
the same input signal**; the −12 st knob pitches the modulator
analysis branch down an octave relative to the carrier branch. One
signal in. This reframes V-2.3: the pitch-lock needs no
pitch-tracking carrier — if the carrier *is* the input, output is
trivially glued to the input's spectrum. The DSP story becomes:
**the input's content at f opens band-gates at f/2, which pass the
input's own f/2-region content** — the sub-octave region is
articulated (gated) by the dynamics of the octave above it. Sam's
read on the thin-sub measurement matches the data: the clean
distribution's raised floor (≈83 Hz) means a 55 Hz kick fundamental
asks for a ~27 Hz band that does not exist, killing the self-sub-gate
path on that probe. Uncertainty flagged: whether the p2 Live renders
actually fed the probe to both branches (vs the internal Carrier
tone) is not established — p3 must pin the routing explicitly.

**Engine translation (supersedes the Alignment framing's texture-only
reading):** the tactile mechanism is a **self-gating subharmonic
articulator** — envelope followers on band n drive gain on band n−12
(or arbitrary offset per the alignment curve) of the *same* signal
path. In the modal engine: per-mode output envelopes can gate/excite
modes an alignment-offset below, with the transect floor deciding how
far down the chain reaches. This is cheap (envelope × gain per mode
group) and now measurement-grounded end-to-end.

## p3 queue (small, targeted)

1. Torque: one partial at f₀ = 175 Hz, T+1200 → is the sideband at
   +440 Hz or +2f₀?
2. Vocodex, **routing pinned to Sam's real topology** (same signal to
   both mod and carrier, confirmed at the panel): sweep + kick at
   shift −12/0/+12, with band-distribution (a) as-is and (b) floor
   dragged to DC — the self-sub-gating story predicts the kick's sub
   *blooms* at −12 st once low bands exist. Also one pass carrier =
   noise (kills any residual carrier ambiguity).
3. Vocodex: modulator unison order > 1, then unison shift sweep.
   **RESOLVED (Sam, 2026-07-21): unison order was 0 in p1/p2 — the
   unison shift dial is ignored unless order is on. The p2 null was
   cause (a), control inert, as the operator suspected. p3 includes
   an order-2 example.**
4. (Optional) saturation curve if we ever want it.
