# 06 — The snare, measured (M10.5 autopsy)

*2026-07-24. The probe methodology turned on real snares, after two
rounds (M9 wire-bed, M10 NESS) built snare mechanisms from intuition
and both failed the same way ("maybe rimshots, with a weaker attack…
no detectable wire-snare-head ringing, it just sounds like white
noise… the fundamental is too high"). Sources: backbeat hits isolated
from two Angelo Mides acoustic-kit loops (FABLE_82: ~20 clean snare
windows; WORKOUT_138: 4 highly consistent hits of one snare) via
spectral-flux onset detection + band classification. Tool:
`lab/snare_autopsy.py`. A Superior Drummer one-shot pack (velocity
ladder, rimshot/sidestick, WIRES-OFF hit, buzz roll) is incoming and
will refine these numbers on uncontaminated material; loop-bleed
caveats are flagged inline. Comparison set: `out/snare-v2/` renders
(v095 recipe, mallet/cavOFF/M9equiv isolators).*

## The scoreboard

| metric | real snares | ours (v095) | verdict |
|---|---|---|---|
| dominant LF peak | 235.5 Hz (WORKOUT, ×4 identical) / ~144–152 Hz (FABLE) | **386 Hz** | Sam's "fundamental too high," quantified |
| 2nd LF peak vs f0 | −9 to −14 dB | **−0.1 dB** (487 Hz ties the f0 peak) | the fundamental never dominates |
| tail T60, 8–14 kHz | 0.33–0.36 s | **1.32 s** | HF noise hangs ~4× too long |
| tail T60 profile across bands | flat: ~0.27–0.5 s in EVERY band | 0.45 s (LF) rising to 1.45 s (HF) | **inverted** — real snares decay top-down-equal, ours ring longest where it hurts most |
| tail spectral centroid | 120 Hz–3.6 kHz (WORKOUT: 121–255 Hz!) | **9.7 kHz** | our tail is parked in the hiss octave |
| tail ringiness (tonal peaks >6 dB over local noise floor, 0.4–8 kHz) | **8–13 peaks**, mean top-10 prominence 7–8 dB, scattered 1–8 kHz | **2 peaks**, both below 600 Hz | "just sounds like white noise," quantified — zero tonal structure where wire/head modes live |
| attack rise 10–90 % | **2–4 ms** (clean hits) | 0.05 ms | the tick is a click, not a crack |
| attack band leader, 0–5 ms | **4–8 kHz** (most clean hits) | 1.5–4 kHz | the crack lives an octave above where we put it |
| attack 5–20 ms | 4–8 kHz *still* at 0 dB | 1.5–4 kHz | wires answer the hit instantly and brightly; ours don't |

## Findings

### F1 — Fundamental dominance, not just fundamental frequency

Real snare: strongest spectral peak in the LF window IS the
fundamental, with everything above it 9–14 dB down (WORKOUT's four
hits: 235.5 Hz at 0 dB, next peaks −9 to −21 dB). Ours: nominal
recipe f0 is 190 Hz, but the measured dominant peak is 386 Hz with
487 Hz *tied* — the modal weighting parks the energy on modes 2–4
and the nominal fundamental is a bystander. Sam's "too high" is
partly tuning but mostly **distribution**: fixing f0 without fixing
mode weighting will not move the percept. (This is also the
decay-law/gain-lane story from the panel era arriving early.)

Real fundamentals observed: ~148 Hz (FABLE kit) and 235 Hz
(WORKOUT kit) — the usable snare-body range straddles our 190; the
tuning is defensible, the weighting is not.

### F2 — The tail decay profile is FLAT in frequency; ours is inverted

The single most surprising number. Every band of a real snare tail —
150 Hz to 14 kHz — decays with T60 ≈ 0.3–0.5 s. The wires do not
hiss on: they ring briefly and die WITH the drum. Ours: LF dies
politely at ~0.45 s while the 8–14 kHz band rings for 1.3–1.45 s at
a 9.7 kHz centroid. That combination (flat-spectrum noise, top-heavy
centroid, seconds-long flat decay) is the *definition* of "white
noise laid over a drum," which is verbatim what Sam heard. The M9/M10
pride metric — decay inversion, noise over tone +27 dB in the tail —
**overshoots reality by an order of magnitude**; in real snares the
tail's ENERGY centroid can sit as low as 121 Hz (WORKOUT) because the
body tone carries the tail while wire noise textures it. Inversion is
real but it is a last-100-ms phenomenon, not the tail's identity.

### F3 — "Wire ring" is discrete tonal peaks poking through the noise

Real tails carry 8–13 spectral peaks >6 dB above the local noise
floor between ~0.4 and 8 kHz (top prominences 8–14 dB, e.g. FABLE
3962 Hz @ +14.1 dB; WORKOUT 3273 Hz @ +8.9–10.2 dB recurring across
all four hits — that's a *wire/head mode*, stable per instrument).
This is ~20 detuned coiled strings + resonant-head partials ringing
against the noise. Our bed (filtered noise + sparse comb) measures 2
peaks, both body modes below 600 Hz: the 1–8 kHz region is
statistically FLAT. No amount of envelope or EQ on noise produces
discrete stable peaks — this requires actual resonators (Net1
territory: wires as modal strings in intermittent contact).

### F4 — The crack is a 2–4 ms event centered at 4–8 kHz, not an impulse

Clean real hits rise in 2–4 ms and their first 5 ms is led by the
4–8 kHz band, which then REMAINS the leader through 5–20 ms (the
wires answering the hit — the "wire slap"). Our stick tick rises in
50 µs (an impulse — reads as a click and, at loudness-matched level,
as a *weak* attack, exactly Sam's words) and leads in 1.5–4 kHz, an
octave low. The crack the ear wants ≈ a dense micro-burst of wire/
head contact events spread over ~2–4 ms with 4–8 kHz spectral center
— a mechanism (wire throw + re-landing at impact), not a pulse
shape.

### Caveats

- Loop bleed: FABLE hits with measured f0 63–98 Hz are
  backbeat+bass/kick overlaps; excluded from the f0 conclusions.
  Hats contaminate some FABLE tail windows (T60 outliers marked
  None/∞ in raw output were discarded). WORKOUT's four hits are
  internally consistent to ±1.4 Hz and ±0.05 s and anchor the
  tables.
- Both loops are processed/mixed material (compression will flatten
  T60 profiles somewhat and could exaggerate F2's flatness). The
  Superior Drummer pack (dry close mic, wires-off isolator, rimshot
  contrast) is the controlled experiment; re-run this battery on it
  before hard-coding any target numbers.

## M11 design targets (loop-derived — SUPERSEDED by the LOCKED list below)

1. **Wires as resonators (Net1)** — target: ≥8 stable tonal peaks
   >6 dB prominence scattered 1–8 kHz in the tail, T60 ≈ 0.35 s,
   riding intermittent contact with R2. The noise bed demotes to
   dust/texture under them.
2. **The crack** — attack energy spread over 2–4 ms, band leader
   4–8 kHz through the first 20 ms (wire-throw microburst at
   impact); the 50 µs tick alone is convicted.
3. **HF tail discipline** — SOLVED by ablation (M10.5 diagnosis,
   `out/diag-hfhang/`, `lab/hf_hang.py`): the wire-bed release
   follower alone; every other suspect (comb, satellites, cavity,
   cascade, decohere, exciter) moves the hang <0.1 s, and with the
   bed off the residual 8–14 k content sits at −123 dBFS. The knob
   is calibrated in exponential time-constant units τ, but the ear
   hears **T60 ≈ 6.91·τ/dust_follow** (−60 dB = ln(1000)·τ; the
   sub-unity follow exponent stretches the dB slope by 1/follow) —
   verified by sweep within ~10 % except at knob max (source re-feed
   inflates it to 4.7 s). Recipe's "release 0.6 ≈ 131 ms" is
   audibly 1.4 s; knob min (0.30 s audible) is already the
   real-snare zone — the whole realistic range is crammed below ~5 %
   of the throw. The M10 remap fixed the SPAN but not the UNITS:
   same bug class one level down. M11 fix: recalibrate the knob in
   perceived-T60 seconds — `rel_t = T60_knob·dust_follow/6.908`,
   T60_knob log-mapped ~0.15–1.5 s (real-snare zone lower-middle,
   inversion/gated-reverb at top). Mapping-only change, no new ids.
4. **Fundamental dominance** — dominant measured peak AT f0, next
   partials ≥9 dB down (weighting law or tilt fix, verify by
   re-running the battery on renders).

The battery (`lab/snare_autopsy.py`) is the acceptance test: M11
renders go through the same table, next to the references.

## SD pack — controlled confirmation (2026-07-24)

The Superior Drummer pack arrived: four DRY drums + Halo-Feeder
(PROCESSED — Vocodex et al. — Sam's aesthetic target, not a snare
reference). Each file: velocity ramps × three hit positions, 17–18
isolated hits, no bleed. Battery: `lab/sd_snares.py` (adds a
floor-guarded T60 fit and ramp-restart position segmentation, which
split every file cleanly into three ~6-hit ramps). Position 1 reads
as center (f0 at base tune, max dominance margin); position 3 as
edge/rim (f0 percept jumps to an overtone ×1.4–2.2, margin
collapses).

### Per-drum medians (all hits)

| drum | f0 @ center | dominance margin @ center | tail T60 LF→8–14 k | ring n / prom | attack 0–5 ms (dB rel per band) | rise, loud hits |
|---|---|---|---|---|---|---|
| 4×14 Black Beauty | 218 Hz | 7.0 dB | 0.58 → 0.43 s | 15.5 / 8.6 dB | 0 / −7 / −14 / −21 / −34 | ~2 ms |
| 5×14 Solid Aluminum | 210 Hz | 0.9 dB | 0.52 → 0.44 s | 17 / 8.4 dB | 0 / −9 / −15 / −22 / −30 | ~1.8 ms |
| 8×14 Pear Stave | 155 Hz | 14.4 dB | 0.57 → 0.46 s | 13.5 / 8.1 dB | 0 / −10 / −14 / −21 / −32 | ~10–13 ms |
| 8×14 Coliseum | 262 Hz | 3.3 dB | 0.58 → 0.43 s | 16 / 9.2 dB | −1 / 0 / −8 / −18 / −30 | ~2 ms |

(Attack bands: 0.1–0.5 k / 0.5–1.5 k / 1.5–4 k / 4–8 k / 8–14 k.)

### What the controlled data confirms, tightens, or overturns

- **F2 CONFIRMED and tightened.** Dry, uncompressed: every band of
  every drum decays in 0.39–0.63 s, with a gentle monotone droop
  toward HF. The flat profile was not a mixing artifact. Locked:
  ~0.55 s at LF → ~0.43 s at 8–14 k, never rising with frequency.
- **F3 CONFIRMED and raised.** Dry close mic shows MORE ring than
  the loops: median 13.5–17 peaks >6 dB, mean prominence 8–9 dB
  (Coliseum edge hits reach 12 dB). Per-drum recurring mode
  frequencies exist (Aluminum 743/1184 Hz, Pear Stave 463 Hz,
  Coliseum 592/937 Hz — the instrument fingerprint, like WORKOUT's
  3273 Hz). **Velocity finding: soft/ghost hits expose MORE
  countable peaks (15–23) than loud hits (11–16)** — loud hits
  raise the broadband wire-noise floor between the peaks. The wire
  resonators must ring at ghost level; a noise-only ghost note can
  never read.
- **F4 AMENDED — the loops lied about the crack's band.** The dry
  drum's 0–5 ms attack is LF/mid-led with a smooth monotonic HF
  falloff (see table); the "4–8 kHz leads" observation from the
  loops was mix EQ/compression, not the instrument. What survives:
  the attack is spread over milliseconds — rise 1.5–2.7 ms on loud
  hits (deep wood ~10–13 ms; soft hits 10–24 ms) — and our 50 µs
  impulse remains convicted. The produced-brightness question moves
  to the Halo delta below.
- **F1 REFINED — dominance is the position axis, and material
  scales it.** Center: 14.4 dB (deep wood), 7 dB (shallow metal),
  0.9–3.3 dB (bright metal — a legitimate material flavor, not a
  failure). Edge: margin ~0–3 dB everywhere, dominant peak moves to
  the overtone. Our render's −0.1 dB *at nominal center* stays
  convicted; the strike-position/weighting control must sweep the
  measured center↔edge range.

### The Halo-Feeder delta (the fucked-fidelity spec)

Same battery on the processed file, diffed against the dry drums:

| metric | dry drums | Halo-Feeder | the processing did |
|---|---|---|---|
| f0 | 155–280 Hz | 158 Hz | tuned/pitched LOW |
| dominance margin @ center | 3–14 dB | **25.4 dB** | **crushed the fundamental UP** |
| tail T60 profile | ~0.55→0.43 s flat | ~0.54–0.61 s flat | left it alone |
| ring | 13.5–17 pk / 8–9 dB | 10 pk / 8.0 dB | slightly smoothed, ring survives |
| tail centroid | ~920–1410 Hz | **642 Hz** | pulled energy DOWN |
| attack (edge position) | LF-led | [−26, −9, −1, 0, −4] | LF cut, mid/HF-led crack layer |
| rise | 2–12 ms | ~13 ms | compression stretch |

The aesthetic Sam is chasing is **more fundamental, not more hiss**:
a dry snare with its f0 dominance pushed from ~10 dB to ~25 dB, tail
length untouched, ring intact, overall centroid pulled down, and an
optional produced crack layer with the LF cut out of the first 5 ms.
Every row of that column is natively reachable with mechanisms we
planned for other reasons — floor/placement (Vocodex lesson), tilt/
gain-lane, and the M11 crack with a band-tilt control.

## M11 targets — LOCKED

1. **Wire ring (Net1, the headline):** ≥12 tonal peaks >6 dB above
   the local tail floor (mean top-10 prominence ≥8 dB), scattered
   0.4–8 kHz, stable per instrument, **present at ghost velocity**
   (soft hits must show ≥ as many peaks as loud). Wires = actual
   detuned resonators in intermittent contact with R2; the noise
   bed demotes to dust/texture.
2. **The crack:** attack energy spread over ~2–4 ms (10–90 % rise;
   up to ~12 ms legitimately for deep/soft), never an impulse.
   Band profile at 0–5 ms follows the dry law by default
   (≈ 0 / −8 / −14 / −21 / −32) with existing bright/Ex controls
   able to tilt toward the Halo produced profile (mid/HF-led, LF
   cut). The loops' "4–8 k leader" target is retired.
3. **Tail T60:** flat with gentle HF droop — ~0.55 s LF → ~0.43 s
   at 8–14 k (acceptance band 0.39–0.63 s, NEVER rising with
   frequency). Bed Release recalibration law stands:
   `rel_t = T60_knob·dust_follow/6.908`, knob log-mapped
   ~0.15–1.5 s — the entire dry-snare zone lands mid-throw.
4. **Fundamental dominance & position:** center ≥10 dB margin for
   the wood/deep voicing (metal flavors legitimately 1–3 dB), edge
   → overtone-dominant (~0–3 dB, f0 percept ×1.4–2.2). Our −0.1 dB
   at center remains the conviction to overturn. The Halo spec —
   ~25 dB margin, centroid ~640 Hz — is the *produced* extreme the
   floor/tilt controls must reach.

Acceptance: M11 renders through `lab/snare_autopsy.py` +
`lab/sd_snares.py` aggregation, tabled next to the four dry drums
and Halo-Feeder.
