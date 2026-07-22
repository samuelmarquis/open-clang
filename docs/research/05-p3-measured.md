# 05 — Probe pack p3, measured: the corpus closes

*open-clang research corpus, entry 05 (final planned entry). Source:
`clang-probes-p3-return.zip` (17 renders; j00 bit-null QC; offset-0
slicing; three settings-identical cells reproduced p2 to 0.00 dB —
the measurement rig is exact across sessions, and hand-drawn band
curves carry ~0.3 dB tolerance). All three standing questions
resolved; no cells skipped.*

## R1. Torque's sidebands are a heterodyne at 2·f₀ ± f_shifted

The 175 Hz probe breaks the p2 degeneracy: T+1200 sideband at
699.99 Hz (= 2f₀ + f_shifted; constant-440 predicted 790 ✗), T−700 at
466.78 (= 350 + 116.79 ✓ vs 556.8 ✗). The downshift also carries
233.20 Hz = **2f₀ − f_shifted** (matches to 0.01 Hz; the
2·f_shifted alternative misses by 0.38). So Torque's added structure
is **sum-and-difference sidebands of the shifted line against a
component at twice the original fundamental** — a ring-mod/heterodyne
residue, not output harmonics. Final Torque model: tracked
fundamental → sub-cent-exact ratio resynthesis + residual + a
2f₀-referenced heterodyne family ~30–40 dB down.
**Engine note:** that residue is a *character* we can optionally
emulate (ring-mod of the retuned group against 2× the pre-retune
fundamental) — logged as a Retune "flavor" toggle candidate, not a
requirement.

## R2. Self-sub-gating CONFIRMED, with a negative control

Routing first: self-vocoding verified by three independent
observations, **and p1/p2 were already self-routed** — all prior
findings hold in the real topology. (Operator self-correction on the
record: an external carrier *is* possible via sidechain; none was
configured.)

The pre-registered prediction: at −12 st, dragging the band floor to
DC should make r01's sub *bloom*. Measured (band energy, dB):

| render | 20–60 Hz | 60–120 | 120–250 |
|---|---|---|---|
| `_self` (−12 st, as-is floor) | 31.4 | 48.2 | 44.6 |
| `_self_floor0` | **55.3** | 47.3 | 45.1 |
| `_self_mp0` | 34.7 | 57.8 | 59.1 |
| `_self_floor0_mp0` | **71.3** | 57.6 | 60.2 |
| `_noisecarrier` | **8.5** | 49.1 | 39.3 |

**+23.9 dB, confined to exactly the predicted band** (adjacent bands
move ≤0.9 dB). Same test run retroactively on p2 data: +24.1 dB. The
**noise-carrier negative control kills the leak hypothesis**: swap
the carrier and the sub vanishes (8.5 dB) — the sub is *generated*
by the self-carrier being gated open in low bands, not passed
through.

**Super-additivity**: floor0 alone +23.9, mp0 alone +3.3, together
**+39.9 dB** (~12.7 dB beyond the sum). Floor creates the bands;
alignment fills them; neither suffices alone.

## R3. Unison is live (p2 null fully explained)

With unison **order = 2** + panning 100% (Sam's refinement): side
channel goes from identically zero to **+16.8 dB, above the mid**.
Unison shift is inert at order 1 — p2's "unmeasured" call was
correct and is now resolved. A self-vocoded mono signal becomes
hard-wide. **Engine note for "3d lowend"**: detuned/panned satellite
voices per mode group is the measured reference for width.

## Final engine translations (now all measurement-closed)

1. **Transect floor × Alignment are a coupled pair** — expose both,
   and expect (design for) super-additive behavior at the
   floor-down/aligned corner. `autovocoding` itself lives at the
   *restrained* corner (raised floor, −12 st); the +40 dB sub corner
   is ours to make playable.
2. **Self-gating subharmonic articulation requires the self-carrier**
   — in modal terms: low modes must be *excited by the signal path
   itself* and gated by upper-mode envelopes; a noise/synthetic sub
   source gated the same way will not produce the measured bloom
   character.
3. **Retune** ships exact (sub-cent) with optional 2f₀ heterodyne
   flavor (R1) and the rejection skirt (04 Part I).
4. **Width** via per-mode-group satellite voices with detune+pan
   (R3).

## Program status

Probe program **CLOSED**. Open but optional: Vocodex saturation curve
(Sam auditioned: "exactly what you'd expect"). Remaining engine
questions are listening questions, not measurement questions —
answered by batches, not packs.
