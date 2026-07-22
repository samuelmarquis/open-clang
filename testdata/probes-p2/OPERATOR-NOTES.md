# OPERATOR-NOTES — clang-probes-p2

*Same operator, same rules as p1 (your NOTES.md was exemplary — the
j00 fade-in catch and the island-alignment proof both made it into the
research corpus verbatim). p2 is small and surgical: it exists to
close the specific holes p1 left. Ranked needs are from
`03-vocodex-measured.md` §p2 and `02-torque-measured.md` §caveats.
Ground rules, naming grammar, QC gates, and return packaging are
identical to p1 unless stated. Sam is babysitting this run.*

## Probes (5)

`q01_partial_220`, `q02_partials_220_330` (new, analytical);
`p02_dirac_steps`, `p06_sweep_slow`, `r01_kick_catsum` (bit-identical
carries from p1 — use these copies, not the p1 ones, so hashes match).

---

## STEP 0 — document the as-is state BEFORE touching anything (highest value)

Open `autovocoding` fresh in the Live host and, before any control is
moved:

1. **Read and log the MOD and CAR pitch displays** (the two
   seven-segment readouts above the MOD/CAR sliders in the Mixet
   section) — exact values and units as displayed. Our p1 screenshots
   show `0 / 0` but were possibly captured post-reset; this reading
   decides the central hypothesis of `03-vocodex-measured.md` (octave
   lives in MOD/CAR carrier transposition vs somewhere else).
2. Log the **WET knob** position, **SG slider**, **+NOISE** state, the
   **carrier selector** ("Carrier tone"?), and both **Contour** boxes.
3. Close-up screenshot of the Mixet section → `screenshots/`.
4. If the MOD/CAR displays are BOTH truly 0 in the untouched preset,
   say so loudly in NOTES.md and add your best evidence for where the
   octave character comes from instead (e.g. the carrier selector's
   own tuning) — that would falsify our hypothesis, which is exactly
   what this step is for.

## Vocodex grids (host in Live as before; concat/slice at your discretion)

**Clean path** for all Vocodex grids = from as-is, set **Soundgoodizer
to 0, +NOISE off, WET to maximum**. Everything else stays as-is unless
the grid says otherwise.

- **V-A clean baseline**: subjects {p02, p06, r01} →
  `<probe>__vdx-av-clean.wav` (3). p02 here is the raw envelope-law
  measurement p1 couldn't give us.
- **V-B MOD/CAR isolation**: from clean state, first zero BOTH MOD and
  CAR (log what zeroing changed if they weren't 0). Then one octave in
  whatever units the controls expose (±12 st / ±1200¢):
  MOD ∈ {−oct, +oct} with CAR 0, and CAR ∈ {−oct, +oct} with MOD 0,
  plus the both-zero reference → 5 settings × {p06, r01} = 10 renders:
  `<probe>__vdx-av-clean_mod0car0.wav`, `_mod-1200.wav`,
  `_mod+1200.wav`, `_car-1200.wav`, `_car+1200.wav`.
- **V-C band-distribution extremes** (clean state, MOD/CAR back to
  as-is): (a) `_bdcram200`: distribution curve dragged so ALL bands
  map below ≈200 Hz; (b) `_bdlinear`: full-range linear ramp, left
  node on the floor (same shape as p1's variant, now on the clean
  path). × {p06, r01} = 4 renders. Screenshot each curve.

Vocodex total: 17 renders + screenshots.

## Torque grids (REAPER, API-set params as in p1; item fades zero)

Fixed: Th −70, S 15 ms, std component. j00 bypass for q01 and q02 (2).

- **T-F fine structure**: q01 × T ∈ {−1200, −700, −200, −100, −50,
  +50, +100, +200, +700, +1200}, F = 220 Hz → 10 renders. This
  resolves whether the few-% deviation of the added spectral lines
  from exact shift ratios (p1, `02` §F1) is systematic or tracking
  noise — with a single 2 s partial we can measure added-line
  frequencies to sub-Hz.
- **T-G component selection**: q02 × T ∈ {−700, +700} × F ∈ {150,
  330} → 4 renders. Question: does the 330 Hz partial get grabbed
  when Focus sits on it, and does 220 survive untouched (and vice
  versa)?

Torque total: 16 renders.

## Return

`clang-probes-p2-return.zip` (same layout: renders/torque,
renders/vocodex, versions.txt, NOTES.md, screenshots/) → taildrop to
**smq**. QC gates as in p1 (j00 bit-null for the two new probes; count
match or logged skip; no normalize/trim).

Improvise freely beyond the grids (`x__` prefix) — your p1
improvisations (the 2×2 corner, funkyone) were both kept.
