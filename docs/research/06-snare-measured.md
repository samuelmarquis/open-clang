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

## M11 design targets (measured, provisional until SD pack confirms)

1. **Wires as resonators (Net1)** — target: ≥8 stable tonal peaks
   >6 dB prominence scattered 1–8 kHz in the tail, T60 ≈ 0.35 s,
   riding intermittent contact with R2. The noise bed demotes to
   dust/texture under them.
2. **The crack** — attack energy spread over 2–4 ms, band leader
   4–8 kHz through the first 20 ms (wire-throw microburst at
   impact); the 50 µs tick alone is convicted.
3. **HF tail discipline** — find why the render's 8–14 kHz T60 is
   1.3–1.45 s (bed release 0.6 ≈ 230 ms cannot explain it — suspects:
   comb feedback ring, satellites, cascade shelf, dust follower) and
   bring the full-band tail profile to ~0.3–0.5 s flat.
4. **Fundamental dominance** — dominant measured peak AT f0, next
   partials ≥9 dB down (weighting law or tilt fix, verify by
   re-running the battery on renders).

The battery (`lab/snare_autopsy.py`) is the acceptance test: M11
renders go through the same table, next to the references.
