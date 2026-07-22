# 03 — Vocodex `autovocoding`, measured

> **SUPERSEDED IN PART (p2, 2026-07-21)**: the "octave lives in
> MOD/CAR carrier transposition" hypothesis is **falsified** — the
> octave is the ♂/♀ `Modulator pitch shift` knob (param 17) at −12 st,
> its extreme. WET is a *level*, not a blend; p1's dry contamination
> was **80% MOD pass-through**, so this entry's §F1/§F2 spectral
> findings are contaminated and superseded. See `04-p2-measured.md`
> Part II for the clean-path mechanism (placement moves the sound;
> the knob is an alignment/texture axis; carrier appears
> pitch-tracked).

*open-clang research corpus, entry 03. Empirical characterization of
Sam's Vocodex preset `autovocoding` from probe pack p1 (2026-07-21).
55 renders produced on sam-pc: FL absent, Vocodex is a legacy 2016
VST2 whose editor crashes REAPER, so the operator ran a
concatenate/slice round-trip through Ableton Live 12 (Sam drove the
five passes by hand); slice alignment proven via silent inter-probe
gaps landing on expected sample offsets, global offset 0 (Live's PDC
absorbs Vocodex's 16-sample latency). Full evidence:
`out/clang-probes-p1-return/NOTES.md`. Analysis:
`tools/analyze_p1.py`.*

## ⚠️ Standing caveats (inherited from the operator, plus one correction)

1. Every render passed through the preset's working signal path:
   **+NOISE engaged and Soundgoodizer at 0.15** — dynamics and
   spectral measurements are of *vocoder → compressor*, not the raw
   vocoder. The p02 dynamics map is confounded (§F5).
2. **Correction to the "50% wet" warning:** the param census shows
   `Wet level = 0.5`, but measurement contradicts a 50/50 blend —
   least-squares dry-component fit in the outputs is tiny
   (|g| ≤ 0.12, typically ≤ 0.04). The renders are effectively
   **all-wet**; `Wet level 0.5` appears to be a wet *gain* default,
   not a blend position. (Static fitting under compression is
   imperfect; but a true 50% dry blend could not hide from it.)
3. Renders are 32-bit float stereo (deliberate, logged); analysis
   uses the mid channel.

## Preset anatomy (screenshots + census + measurement)

- **47 bands** (not the 100 max; readout `47`, screenshot).
- **Band Distribution**: a straight ramp on the UI scale whose
  **left node is raised off the floor**. Measured consequence (band
  centers from the time-averaged wet spectrum of the sweep pass):
  quasi-log spacing, first centers ≈ 26.9 / 80.7 / 129.2 / 172.3 /
  215.3 / 279.9 / 344.5 / 393 Hz — **9 bands below 500 Hz**. In the
  `_bdlinear` variant (node dragged to the floor = 0 Hz/DC) the
  bottom band falls to ≈21.5 Hz and the low stack re-spaces
  (70.0/123.8/183.0/242.2…). The raised floor keeps the bottom band
  *above* DC-mud and re-crowds the sub-100 Hz region — this is a
  deliberate low-end voicing, subtle but real (dragging it flat
  changed kick renders by 212% RMS).
- **Modulator pitch shift envelope: FLAT at 0** (screenshot). The
  octave relationship is **not** the per-band modulator-pitch curve.
- **Working hypothesis — the octave lives in the MOD/CAR carrier
  transposition.** Evidence: (a) the sweep activation map is
  unshifted in *every* variant (out/in frequency ratio median
  0.986–0.990 — bands fire where the modulator is); (b) yet the
  `_mp0` variant (mod-pitch control centred) changes output level and
  texture massively (562% RMS on the sweep; the operator's discarded
  1 kHz marker came out ~60× louder in mp0 passes); (c) the preset's
  carrier is Vocodex's internal "Carrier tone". Transposing the
  carrier relocates what's *inside* each band — level, beating
  density, sub-weight — without moving which bands fire. That
  reproduces every observation, and it matches the original
  recollection ("pitch down the carrier (or is it the modulator)").
  **Unproven until p2 isolates MOD vs CAR explicitly.**

## Findings

### F1. The band map is an identity map; the trick is intra-band

Sweep → output ridge ratio ≈ 0.99 (p10–p90 within ±4%) for as-is,
`_bdlinear`, and `_mp0` alike. Vocodex `autovocoding` does not
relocate energy in frequency; it re-*textures* it. Plate:
`out/analysis-p1/plates/vocodex-sweep-ratio.png`.

### F2. The low-end voicing is band placement, not EQ

Kick (r01) wet octave-band profile (31→1k, dB): as-is
[37.2, 25.5, 17.7, −7.4, −28.4, −35.9] — a steep, controlled rolloff
with the mass in 31–250 Hz across exactly the ~9 sub-500 bands.
`_mp0` fattens the mids instead ([…, 26.8, +1.5, −15.3, …] at
125 Hz–1k): with the carrier transposition off, energy spreads
upward. The as-is preset is *suppressing* mid clutter and
concentrating output in the bottom bands — tactility as band
economy.

### F3. Dynamics track smoothly (confounded)

Level-stepped diracs: wet RMS per hit rises monotonically
−125 → −44 dB across −60 → −1 dBFS input, mild expansion in the
upper range, no hard gate. −60 dBFS impulses produce no output
(island detection: the p02 slice starts 0.25 s late because the first
hits render silence). Soundgoodizer is in this path — treat as the
*preset's* dynamics, not Vocodex's envelope follower law.

### F4. Both isolations interact

`_bdlinear` and `_mp0` each produce large, distinct deltas, and the
operator's improvised `x__*_bdlinear-mp0` corner exists to test
separability — full 2×2 analysis deferred to the p2 write-up.

## Implications for open-clang

1. **The placement transect is vindicated as the signature control.**
   The measured mechanism of "tactile low end" is: (a) a raised
   frequency floor (bottom band ~27 Hz, not DC), (b) quasi-log
   clustering putting ~9 narrow bands below 500 Hz, (c) sub-octave
   carrier content *beating inside* those narrow bands. In our modal
   engine: transect low-cluster + a per-mode sub-octave enrichment
   (satellite partial or ring-mod term per low mode) reproduces the
   physics of the trick without a vocoder in sight.
2. **Floor-raising deserves its own knob** on the transect (the
   difference between 21 Hz and 27 Hz bottom bands was audible enough
   that Sam voiced it deliberately).
3. Band economy over band count: 47 < 100 was a choice; density
   should cluster where the material lives.

## p2 needs (ranked) — **PACK SENT 2026-07-21** (`clang-probes-p2.zip`
→ sam-pc; protocol: `testdata/probes-p2/OPERATOR-NOTES.md`)

1. Soundgoodizer = 0, +NOISE off, Wet max: re-run p02 (raw envelope
   law) and r01/p06.
2. Explicit MOD vs CAR isolation: ±1200 on each alone, sweep + kick
   subjects — settles the F-hypothesis. Read the MOD/CAR displays
   in the as-is state *before* touching anything and log them.
3. Band-cluster extremes: distribution curve fully crammed <200 Hz
   vs fully spread, same subjects (calibrates our transect ranges).
4. Torque fine-structure probe (see 02 §caveats) can ride the same
   pack.
