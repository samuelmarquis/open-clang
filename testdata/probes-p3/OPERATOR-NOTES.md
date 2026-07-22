# OPERATOR-NOTES — clang-probes-p3

*Small, surgical, final (probably). Three questions, ~12 renders. Same
ground rules, naming grammar, QC gates, and return packaging as
p1/p2. Your p2 return was outstanding — Step 0 falsified our central
hypothesis exactly as designed, and the effect-size and envelope
tables went into the corpus nearly verbatim. Context for this pack:
Sam has clarified that `autovocoding` is SELF-vocoding — modulator and
carrier are the same signal, with the modulator branch analyzed −12 st.
p3 pins the routing explicitly and tests the self-sub-gating story.*

## Probes (3)

`q03_partial_175` (new: 175 Hz decaying partial, T60 2 s);
`p06_sweep_slow`, `r01_kick_catsum` (bit-identical p1 carries).

## Torque — 3 renders (REAPER, API-set, item fades zero)

- `q03__j00` bypass.
- `q03 × T+1200, F=175, Th −70, S 15, std` → if the sideband sits at
  **790 Hz** (= 350 + 440) the offset is a fixed +440 Hz; if at
  **700 Hz** (= 350 + 2·175) it is +2·f₀. One render settles it.
- `q03 × T−700, F=175` (same logic on a downshift: 233.7 vs 350+?—
  measure whatever lines appear; redundancy is cheap).

## Vocodex — ~9 renders (Live host as before)

**STEP 0 (routing, non-negotiable):** configure Sam's real topology
explicitly — **the same probe signal feeding BOTH modulator and
carrier inputs** — and document in NOTES.md exactly how the host
routing achieves this (screenshot of the Live device chain / routing
panel). Also state what carrier source the p2 renders actually used,
if determinable in hindsight — it affects how we read p2.

Clean path as p2 (SG 0, pass-throughs 0, noise 0, WET unity).
Self-routing, subjects {p06, r01} unless noted:

| render suffix | settings | question |
|---|---|---|
| `_self` | as-is preset otherwise (−12 st, as-is band dist.) | baseline in the true topology |
| `_self_mp0` | shift 0 | alignment reference |
| `_self_floor0` | −12 st, band-dist LEFT NODE dragged to floor (low bands exist down to DC) | **the prediction**: r01's sub should BLOOM vs `_self` — upper content now opens existing low bands over the self-carrier |
| `_self_mp+12` | +12 st, as-is dist. | sign/direction check in self-topology |
| (r01 only) `_self_floor0_mp0` | shift 0 + floor 0 | 2×2 corner |
| (p06 only) `_noisecarrier` | carrier = noise, −12 st | kills residual carrier ambiguity from p2 |

- **Unison**: on any one self render, set **Modulator unison order > 1
  first**, then max unison shift → `x__..._unison-live.wav` (one
  render; if order can't exceed 1, log it and we close unison as
  inert).

## Return

`clang-probes-p3-return.zip` → taildrop to **smq**. The prediction in
`_self_floor0` is falsifiable and stated in advance — if r01's sub
does NOT bloom, say so plainly; that kills the self-sub-gating story
and we want it dead if it's wrong.
