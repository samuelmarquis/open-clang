# 01 — Architecture: the open-clang engine

*Pre-implementation plan, 2026-07-21. This is the before-the-fact
document (the as-shipped walkthrough will be `docs/DSP.md`, written
after the engine survives listening). Built on the evidence in
`docs/research/01-design-space.md`; claims cited there are not
re-cited here. Conventions inherited from
`~/Developer/open-plugin-template/README.md`.*

## The idea in one paragraph

open-clang is a drum/impact synthesizer whose grit is *mechanistic*,
not cosmetic: a modal resonator core enriched with the three verified
physical grit mechanisms — amplitude-dependent stiffening (pitch
glide), explicit collision satellites (rattle/buzz), and post-strike
spectral cascade (clang/crash buildup) — driven by pluggable exciters
("strikes") and steered by material-language controls rather than
drum-kit taxonomy. Velocity and position do what physics says they do:
velocity deepens the glide and feeds the cascade, position reweights
the modes, and a bracing axis decides how much of the blow the body is
allowed to keep. The anti-physical pole (deliberate aliasing,
filters-as-resonators, digital rawness) is a first-class material, not
a failure mode.

## Sound targets

The engine is tuned against percepts, in this priority order:

1. **The unbraced blow** — deep, wet, structural impact; energy
   entering a body with its damping down. Sub-tactile low end with
   dense low-mode beating (the clustered-band lesson).
2. **The clang** — struck metal with post-strike HF buildup (~50 ms
   cascade), inharmonic, position-sensitive.
3. **The rattle** — snare/buzz as real intermittent contact, gated by
   the resonator's own motion, never a noise layer.
4. **The crumple/click** — discrete buckling events, stochastic,
   power-law amplitudes.
5. **The latex/elastic family** — rubbery pitched drums: FM-ish
   exciters with unstable pitch, keytracked damping (the SOPHIE
   fragments).
6. **The digital-raw pole** — aliased sweeps, absurd-Q noise collapse
   (the Microtonic lesson).

## Non-goals (v1)

- No 3D air embedding, no full FDTD instruments (offline-verified
  territory; see research §A.3). If we ever want reference renders, we
  do them offline in the lab, not in the plugin.
- No sample playback, no GM-drum emulation, no "acoustic kit" preset
  ambitions (Collision owns the clean end; we don't).
- No effect-chain maximalism: one drive, one convolution slot, one
  space. Depth goes into the resonator, not the rack.

## Voice architecture

```
        ┌─────────────┐      ┌──────────────────────────────┐
 MIDI ─▶│  EXCITER    │─────▶│  RESONATOR NETWORK           │──▶ post ──▶ out
        │  (strike)   │ brace│                              │
        └─────────────┘  ─▶  │  R1 ⇄ R2   (bidirectional    │
          E1 mallet          │            coupling)         │
          E2 slap/burst      │  each R: modal bank          │
          E3 buckling        │   + NL1 stiffening glide     │
          E4 audio-in        │   + NL2 contact satellites   │
          E5 raw/digital     │   + NL3 spectral cascade     │
                             │  R-alt: 2-D waveguide mesh   │
                             └──────────────────────────────┘
```

### Exciters (the "acoustic shader" slot)

A common contract: an exciter renders a force signal (plus optional
sustained contact state) given `(velocity, position, material params)`.
Pluggable implementations:

- **E1 mallet/contact** — nonlinear contact pulse; stiffness and
  velocity set width/brightness (harder+faster → shorter, brighter).
  Penalty-potential contact against the resonator surface state, so a
  soft mallet on a moving membrane re-contacts naturally (flams,
  buzz-rolls emerge).
- **E2 slap/burst** — shaped noise burst with fast spectral envelope;
  hand/stick/brush family.
- **E3 buckling source** — stochastic click train, power-law amplitude
  distribution, rate driven by an "instability" input (Cirio-style
  enrichment as an *exciter*, so it can strike any resonator).
- **E4 audio-in** — external input as excitation (effect mode; the
  NESS Zero Code precedent: the resonator network as a physically
  nonlinear "reverb").
- **E5 raw/digital** — single-sample impulses, aliased chirps, DC
  kicks. No apologies.

### Resonator core: the modal bank

Per resonator, N modes (target N=64 default, 256 max), each a
time-varying resonator `{f_k, T60_k, g_k}` updated block-rate with
per-sample-stable forms (coupled-form/SVF family; coefficient
interpolation to avoid zipper glide).

**Mode placement — the transect.** Two sources, blendable:
1. **Archetype tables**: membrane, plate, beam/bar, tube, shell —
   published mode-ratio families, with strike-position → mode-weight
   from the archetype's mode shapes, and a listening-position weight
   set on output taps.
2. **The drawable placement envelope** (the Vocodex generalization):
   a user-drawn curve mapping mode index → frequency across the
   spectrum — plus a **gain lane** (mode gain vs frequency; Batch
   003b/003c: −8 dB/oct output tilt is what makes a 22 Hz membrane
   actually *sound* 22 Hz). Cluster low for tactile beating density; spread high for
   air. A second curve sets per-region damping. This is the signature
   control; it must be operable from the panel and from the CLI patch
   file identically. Measurement (`04-p2-measured.md` §V-2) settles
   the mechanism: **placement decides where output lives** (cramming
   the reference's bands below 200 Hz relocated output wholesale,
   ratio 0.47), so the transect carries a first-class **floor
   control**; and the vocoder's octave trick is really an
   **alignment detune** — which input region opens which modes,
   offset by up to ±1 octave. That becomes the **Alignment** control
   on the audio-in path (measured stakes: 79× level swing, 13–16 dB
   sub-shape swing). Skirt/order steepness per mode is exposed too:
   shallow skirts measurably lengthen ring (249 vs 171 ms at −40 dB
   in the reference).

**Damping law**: global T60 scale + frequency-dependent tilt +
keytracked damping option (SOPHIE fragment: keytracking filter →
keytracked decay).

### The three nonlinear enrichments

- **NL1 — stiffening glide (FvK-inspired, Berger-style reduction).**
  Track total bank energy E(t); effective stiffness scales mode
  frequencies by `(1 + β·E)^½`-shaped law → hard hits start sharp and
  glide down as energy decays. Velocity→glide-depth for free. Cheap,
  global per resonator; per-mode-group coupling is the v2 refinement
  (Ducceschi/Touzé route if listening demands it).
- **NL2 — contact satellites (rattle/buzz) + the dust layer.** Small
  lumped mass/string objects in intermittent penalty contact with the
  resonator's output displacement (snare wires, loose fittings,
  jingles). Gated by the body's own motion: rattle dies as the body
  calms — the verified mechanism. **Plus** (Batch 003 verdict) a
  **dust layer**: envelope-gated filtered noise as the statistical
  limit of many micro-contacts (snare-bed texture) — controls:
  level, activity threshold, and a loudness→dust *follow law*
  (linear↔expansion). Discrete contact switches off; dust fades —
  both are real materials, both ship.
- **NL3 — spectral cascade (clang builder).** Energy transfer from
  low modes to high modes after hard strikes: an energy-dependent
  coupling that pumps a high-mode shelf with ~10–80 ms buildup
  (size-scaled), approximating wave-turbulence buildup. Stochastic
  phase; deterministic energy audit. Batch 004 verdicts: must be
  **energy-conserving** (lows deplete as highs bloom — additive
  cascade reads as a noise swell, τ fails as a size percept), and
  ships with an **attack-balance knob** (buildup↔static blend; the
  static extreme is a legitimately compelling harder-impact flavor).
  Size = τ *and* mode density together, not τ alone.

All three carry a **discrete energy ledger** (NESS discipline):
in debug and in viz, every element reports energy in/out; the audit
failing is a bug even when the ear hasn't caught it yet.

### R-alt: the 2-D waveguide mesh

A small rectilinear DWM membrane (multiply-free junctions) as an
alternative resonator type for true 2-D transients: position-dependent
early wavefronts are physicality the modal bank can't fake. Fixed-point
inner loop with energy-preserving rounding; dispersion accepted as
character first, allpass-corrected later if listening objects. Mesh
sizes ~24×24..48×48; CFL at the limit.

### Coupling and the bracing axis

- **R1 ⇄ R2 bidirectional coupling** (Chromaphone precedent; membrane↔
  cavity↔membrane topologies for drum-like builds).
- **Bracing** (the Houdini axis): a macro driving **split
  sub-controls** (Batch 005 verdict): **Coupling** (exciter→body
  energy transfer) and **Choke** (early damping that releases,
  ~50 ms) exposed separately, with tension pitch-up and T60 scaling
  riding the macro. Braced = tense surface, energy reflected, dry
  thwack. Unbraced = coupling up, low modes keep the blow (×1.9
  low-mode ring measured in the lab). Headline performance control
  (map to MPE pressure); validated as the project's best sounds in
  Batch 005.

### Post section

- **Drive**: DEMOTED (2026-07-22, PATHS-NOT-TAKEN 005) — the
  clean-fucked-fidelity doctrine: harmonic richness through
  mechanism, never waveshaping. Revisit only in a post-panel effects
  era.
- **Convolution slot**: short IRs (≤ ~1.5 s) for material transfer
  (struck-object recordings, not halls); partitioned, zero-latency
  head.
- **Retune knob** (settled by measurement — `02-torque-measured.md`):
  Torque's architecture is "intact residual + ratio-shifted copy of
  the focus-region resonance," which a modal engine gets natively as
  a **low-mode-group retune** (±1200¢, soft-bounded region), with a
  blend/replace choice Torque doesn't offer. Effect-mode (audio-in)
  path adopts Torque's measured defaults: soft per-hit threshold,
  15–50 ms process time constant, no FFT (32-sample budget proven
  sufficient by the reference). p2 addenda (`04` Part I): the
  reference's shift is sub-cent exact — ours should be too; and a
  **rejection skirt** around the retune target region (Torque's
  Focus measurably suppresses competing partials by >20 dB) is part
  of why retuned drums sound clean.
- **Stereo**: primary direction (x-stereo-pilot verdict): **per-mode
  L/R decoherence** — micro-detune/phase/damping divergence per mode
  with a coherence control (opq's coherence, drum-sized), keeping the
  sub coherent by construction. Demoted to seasoning:
  listening-position pairs (PATHS-NOT-TAKEN 003) and detuned/panned
  satellite unison (05 §R3 — measured but "detuned width" reads
  uninteresting to the user). Uncorrelated dust/noise components
  per channel stay (Microtonic dispersion). No reverb in v1.
- **Floor × Alignment coupling** (05 §R2): the transect floor and the
  alignment offset interact super-additively (+23.9 dB and +3.3 dB
  alone, +39.9 dB together on the reference). Design them as a
  coupled pair; the floor-down/aligned corner is the "depth-charge"
  zone and must be playable, not a trap. Low modes are excited by
  the signal path itself and gated by upper-mode envelopes — a
  synthetic sub source gated identically will not reproduce the
  measured character (noise-carrier control, 05 §R2).

## Control model

- **Material taxonomy**: presets and macro names speak material —
  Membrane / Plate / Shell / Latex / Glass / Trash / Raw — not
  kick/snare/hat. Six macros per patch:
  **Material** (archetype + damping law), **Size** (a *law*, not a
  scale — Batch 004b/004c: co-scales f0 ∝ 1/size, mode density ↑,
  T60 ↑, cascade τ ↑, and **nonlinear susceptibility ↓** as
  drive ∝ v²/size^k; nonlinear commotion is itself a smallness cue —
  FvK nonlinearity goes as (deflection/thickness)², so big objects
  are hard to drive nonlinear and velocity must read as force, not
  size-shrink), **Strike** (exciter select + stiffness/position),
  **Bracing**, **Grit** (NL2 dust/satellites + NL3 + drive weights),
  **Air** (HF damping tilt + width).
- **MIDI**: note → size/pitch (with keytracked damping option),
  velocity → strike velocity (glide depth + cascade feed), with an
  **exposed velocity-response/ladder control** (Batch 002 verdict:
  the velocity ladder is a feature, not plumbing — curve + depth on
  the Strike macro). **MPE**: pressure → bracing/choke, slide →
  strike position.
- Latency: **zero**. This is an instrument; nothing in the v1 graph
  needs lookahead.

## Engine/shell contract (template §3)

- `Engine::new(sr, channels)`, `process_block(io, events, p:
  &EngineParams)`, `reset()`, `latency_samples() == 0`.
- `EngineParams`: flat `Copy` struct; enums for exciter type, archetype,
  drive mode. The placement/damping transect curves ride alongside as
  fixed-size arrays (e.g. `[f32; 64]` knots), still `Copy`.
- `VizFrame`: per-mode energies (the **modal transect** — the panel's
  drum: mode energy vs time, glide visible as comb bend), collision
  event ticks (NL2), buckling ticks (E3), cascade shelf level, energy
  ledger {in, stored, dissipated, out} whose imbalance drives an
  honest alarm lamp on the panel.
- Strict **no-alloc audio thread from day one**; install the
  `assert_no_alloc` allocator in debug builds (the discipline opq
  deferred — a percussion engine has no excuse).

## CLI (`clg`) — deliberately not named after the compiler on your PATH

- `clg render patch.toml out.wav [--vel 0.9 --pos 0.3 --note 36]` —
  single hits.
- `clg bank patch.toml outdir/ --sweep vel=0.1..1.0:8 --sweep
  pos=0..1:4` — cartesian sweep renders for listening batches; the
  filename is the settings sheet.
- `--viz-dump trace.jsonl` — same VizFrame stream the panel draws.
- Patch files are TOML mirroring `EngineParams` 1:1 (flags override
  fields). Probes and batches are driven by `tools/render_batch.py`.

## Numbers (targets, to be revised by measurement)

- 48 kHz reference; 44.1–192 supported. Block-rate coefficient update
  every 32 samples with interpolation.
- Budget: 8 voices × (256 modes + 1 mesh 32×32 + 4 satellites) ≤ ~20%
  of one modern core. First measurement milestone gates the mesh's
  default-on status.
- Fixed voice memory, preallocated at `new()`; voice stealing by
  energy floor.

## Prototype lab (`lab/`)

Python (numpy/scipy/soundfile), same flake as opq. Strategy toggles
inside `lab/engine.py`, numbered in LISTENING-LOG, frozen when Rust
becomes canonical. First batches:

- **Batch 001 — linear dignity check**: archetype modal banks + E1/E2,
  no NL. Question: does the linear core already sound like *objects*?
- **Batch 002 — the glide**: NL1 on/off at three velocities. Question:
  does energy-tracked stiffening read as "hit harder," or as pitch-env
  fakery?
- **Batch 003 — the rattle**: NL2 satellites vs gated-noise imposter
  A/B. Question: can you hear the contact gating?
- **Batch 004 — the clang**: NL3 cascade vs static bright tilt.
  Question: does buildup-time-scales-with-size land?
- **Batch 005 — bracing**: the Houdini axis across its range on one
  material. Question: is one macro enough?

## Milestones

- **M0** — scaffold: flake, workspaces, docs, probes protocol, lab
  skeleton. (In progress; probe pack for Torque/Vocodex measurement is
  a parallel M0 artifact.)
- **M1** — lab renders Batches 001–002; LISTENING-LOG has verdicts.
- **M2** — lab renders 003–005; design freeze of v1 percept set.
- **M3** — `rt/engine` + `clg` CLI reach listening parity with the
  frozen lab (A/B rendered, logged). Energy-audit tests green;
  no-alloc verified.
- **M4** — WRAC shell: params, MIDI/MPE, state; `auval` passes;
  daily-drivable in Ableton.
- **M5** — STEREO, as EXPLORATION (re-scoped 2026-07-22, Sam): no
  fixed program — prototype several knobs, listen, keep/kill. No
  strict-mono-lowend guarantee ("not afraid of negative
  correlation"; crazy-3d lowend is the goal — sub protection is a
  *param* (stereo_floor), not a doctrine). Round 1 (built): width
  (per-mode phase divergence), decohere (per-mode L/R micro-detune —
  conceptually disliked, may die in listening), stereo_floor,
  satellite panning, per-channel dust. Round 2 candidates: per-mode
  hard L/R mode allocation (split the bank spatially), per-channel
  damping asymmetry, per-event contact panning, sub quadrature
  rotation, dual offset excitation. Sequenced AFTER the Size-macro/
  housekeeping round.
- **M6** — Size macro + housekeeping (Sam-ordered): expose Size (the
  004c law incl. drive-susceptibility with a soft ceiling),
  velocity-response curve (Batch 002 promise), glide-depth-scales-
  with-size (Batch 002 action), sample-rate correctness pass (44.1k
  hardcoded coefficients), sub-block note timing, MPE/choke decision.
- **M9** — the wire-bed (snare round, 2026-07-23): dust promoted to a
  true snare mechanism — own attack/release envelope (release may
  EXCEED body T60: the decay inversion, tone-dies-noise-hangs),
  selectable drive source region, wire-comb shimmer, brightness,
  compressive follow. Cheapest path to "reads as snare."
- **M10** — NESS topology round (BUILT 2026-07-24; scope set by the
  M9 verdict — "MAYBE a rimshot. MAYBE."): (1) second head + cavity
  coupling (R1⇄cavity⇄R2, research 01 §A.2): 4 cavity air modes +
  12-mode resonant head, volume coupling via the odd×odd/(m·n) law,
  skew-symmetric pressure exchange, every exchange normalized by the
  receiver's dissipation (1−r) — the M10 margin law; bed source
  graduates from proxy band to real R2 motion; params 44–47.
  (2) **Stick exciter** — sub-ms Hertzian contact + direct contact
  radiation ("the tick") through its own Cheby-II state: at snare
  tunes the modal bank tops out ~1.1 kHz, so the crack must radiate
  directly. (3) **Biquad infrastructure** — Cheby-II order 12
  replaces the one-pole fleet: flat to 0.40·sr, −63 dB at 0.45·sr;
  Bed Bright 1.0 now reaches 15.6 kHz. (4) Bed Release remapped
  30 ms–0.35 s. Also: the CLI's 1.5 ms render fade-in was found to
  have erased sub-ms attacks since batch 001 — removed.
- **M11 (queued)** — Net1 rattling interconnections (satellite
  chains; the bar-rescue vehicle) — deferred from M10 (the round was
  full); plus whatever the M10 snare verdict demands.
- **M-panel** — DEFERRED (2026-07-22): the panel waits on Sam's
  separate design-system effort (shared visual language for the
  sister plugins); re-enters the roadmap on his call. Interaction
  model per Sam (2026-07-23): **Vocodex-style drawable graph lanes**
  — placement transect, gain lane, **decay-law curve** (draw T60 vs
  frequency: kills the lows-always-ring bias, would have partially
  solved snare on its own), alignment curves (the articulator),
  envelope-follower maps. Transect drum, honest alarms, catalogue
  v01 — all specs stand.
- **M6** — measured-reference integrations: `02-torque-measured.md` →
  formant knob decision; `03-vocodex-measured.md` → placement-envelope
  calibration against the original trick.

Evaluation is the listening log, an energy-audit test suite, and the
CPU budget above. Taste is data; the log outranks this document.

## Risks / open bets

1. **Real-time nonlinear-modal stability** — NL1/NL3 must stay passive
   under parameter motion; the energy ledger is the tripwire, SAV/IEQ
   schemes the fallback (research §C.4).
2. **Contact stiffness at audio rate** — NL2/E1 penalty contacts may
   need local oversampling; budgeted as a known unknown.
3. **Glide zipper** — block-rate frequency motion on 256 modes needs
   careful interpolation; coupled-form resonators chosen for graceful
   time-variation.
4. **Param surface explosion** — the transect curves must carry most
   of the expressive load, or the macro story collapses. Batches 001-005
   exist to catch this early.
5. **The name** — binary is `clg`; nothing shadows `clang`. The
   project name stays, because impacts.
