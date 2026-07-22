# 02 — Shell notes: the WRAC plugin (M4 v0, headless)

*Build record, 2026-07-22. The shell is `wrac/plugins/clg/src-plugin`,
adapted from opq's shell (template conventions per
`~/Developer/open-plugin-template`). Headless: hosts present their
generic parameter editor; the panel is M5.*

## Identity — ABI, never change after release

| field | value |
|---|---|
| package | `clg_plugin_wrac` |
| plugin_id | `org.open-clang.clg` |
| plugin_name | open-clang |
| vendor / company_name | open-clang |
| bundle_identifier | `org.open-clang.clg` |
| vst3_component_id | `aa746857-0729-49ef-92f6-097512510960` (pre-swap; build.rs applies the Steinberg TUID byte-flip) |
| auv2 | type **`aumu`** (music device / instrument), subtype `Clg1`, manufacturer `Oclg` |
| clap_features | instrument, drum, stereo |
| vst3_subcategories | Instrument\|Drum |
| formats | clap, vst3, au (no AAX; standalone unbuildable on CLT — ibtool) |

## Parameter table (ids are ABI; append, never renumber)

| id | name | range (default) | engine field |
|---|---|---|---|
| 0 | Material | Membrane/Plate/Bar (Membrane) | arch |
| 1 | Tune | 20–500 Hz (36) | f0 base; note offsets: f0 = Tune × 2^((key−60)/12) |
| 2 | Strike Stiffness | 0–1 (0.55) | stiffness |
| 3 | Decay | 0.1–4 s (1.5) | t60_base |
| 4 | Damping Tilt | 0.3–3 (2.0) | tilt |
| 5 | Mode Density | 4–14 (8) | n_axial |
| 6 | Glide | 0–12 st (8) | glide_st |
| 7 | Transect Tilt | −12..+6 dB/oct (−7) | out_tilt_db_oct |
| 8–12 | Cascade / Time / Attack / Conserve / Coherent | (0, 50 ms, 0, On, Off) | cascade_* |
| 13 | Bracing | 0–1 (0) | apply_brace_macro |
| 14 | Rattle | None/Wires/Loose/Trash (None) | sat_* presets (CLI-identical) |
| 15–17 | Dust / Threshold / Follow | (0, −40 dB, 1.0) | dust_* |
| 18 | Strike Position | 0–1 (0.35) | position |
| 19 | Listen Position | 0–1 (0.31) | listen_pos |
| 20 | Output | −24..+12 dB (0) | shell gain (×0.005 engine-scale norm) |
| 21 | Bypass | Off/On (Off) | shell mute (voices keep decaying) |

## Voice architecture

8 × `clg_engine::Engine` in the processor; note-on builds a full
`EngineParams` snapshot from the atomics (`state.rs
engine_params_for_note`) and triggers a voice — round-robin over free
voices, steal-quietest by `stored_energy()` when full. One-shot
voices; note-off ignored (choke arrives with MPE work). Velocity →
EngineParams.velocity (CLAP normalized / MIDI ÷127). Output = voice
sum, dual-mono to all channels (real stereo = per-mode decoherence,
M5+). Zero latency. Params applied at note-on only (a ringing voice
keeps its birth params — correct for drums).

## Deviations from opq's shell

- No GUI modules at all (M4 headless; opq's board/canvas/drum/gui not copied).
- Audio ports: **no inputs**, one fixed stereo out (instrument);
  opq's configurable-ports extension dropped (nothing to configure).
- Note port named "Trigger"; latency 0 (opq: N_FFT).
- Bypass = output mute (engine has no dry path to align).
- `// M5: viz` markers where the viz queue and per-voice VizFrame
  publishing will land.
- One new template patch, tagged "Patched for open-clang"
  (`wrac_xtask/src/commands.rs` VST3-validator configure): only pass
  `-G Xcode` when the platform resolves an Xcode generator — consistent
  with the three opq CLT patches (all preserved).

## Build & validation status (2026-07-22)

- `cargo xtask build -p clg_plugin_wrac --release --target clap --target vst3 --target au`: **all built**.
- WRAC production-readiness checks: **pass** (bypass rule satisfied by param 21).
- clap-validator 0.3.2: **21 tests, 18 passed, 0 failed, 3 skipped**.
- `cargo xtask install` (user-local) + `auval -v aumu Clg1 Oclg`: **AU VALIDATION SUCCEEDED**
  (note: requires `killall AudioComponentRegistrar` after first install — stale registry
  cache reports "didn't find the component").
- VST3 validator: **unbuildable on CLT-only machines** — the VST3 SDK's
  hosting-examples build hard-requires Xcode ≥9 (`SMTG_DetectPlatform`).
  Not a plugin defect; opq never ran it either. Run on an Xcode machine
  if ever needed.
- Standalone: unbuildable on CLT (`ibtool`), as documented in the template.

## Known gaps (tracked, not bugs)

- Panel + viz feed (M5): VizFrame plumbing, modal transect drum,
  energy-ledger honest alarm.
- MPE (pressure → bracing, slide → position) and note-off choke.
- Macros beyond Bracing (Size law lives in batch scripts/CLI only;
  the shell exposes granular params).
- Sample-accurate note offsets (currently block-granular).
- Output normalization constant (0.005) is a first calibration —
  listening will revoice it.
- SDK checkouts now gitignored (mirroring opq); re-fetch per
  `wrac/.gitmodules`.

## M4 fix round (2026-07-22)

Sam's first-play bug list, triaged and resolved:

1. **DC click (cascade attack + coherent) — FIXED, engine.** The
   coherent shadow ring initialized in-phase (`u2 = inj·40` → output
   steps at sample 0). Now armed in quadrature (`v2`); first output
   sample measured exactly 0.0.
2. **Positive transect tilt distorts — FIXED, engine.** The gain lane
   had no energy compensation. Tilt now rescales to constant Σamp²
   (unity energy): spectrum reshapes (centroid 59→198 Hz across
   −8..+6 dB/oct on the kick), level stays put.
3. **Dust inaudible in host — NOT REPRODUCIBLE; chain verified.**
   Engine A/B at plugin-default params: 56 dB band delta. New seam
   test `state::tests::dust_reaches_engine_via_host_path` replicates
   the host param path (set_parameter_value → engine_params_for_note
   → render) and passes. Best hypotheses for the session report:
   masking by uncompensated-tilt distortion (bug #2, fixed) or by
   stochastic cascade wash (bug #6, default changed); dust applies at
   note-on (knob changes affect the *next* hit). Retest requested.
4. **Cascade Time "does nothing" — WORKING, SUBTLE.** Plumbing
   verified; measured HF-bloom peak 18→37 ms across the 10→150 ms
   range (coherent). Consistent with Batch 004 (τ reads weakly);
   size lives in the Size law, not τ alone. No change.
5. **"MIDI controller slider" — FIXED, shell.** clap-wrapper's VST3
   publishes IMidiMapping CC proxy parameters (ParamID 0xb00000+, one
   set per MIDI channel) whenever the CLAP note port declares the
   MIDI dialect. We consume note events only → note port is now
   CLAP-dialect-only; the proxies disappear. (Same species as the 130
   inert MIDI-CC entries in Torque's param census — full circle.)
6. **Stochastic cascade = "white noise sweep" — VOICING, not a bug.**
   Plumbing identical to CLI. Dense noise-driven receivers read as
   wash at higher tunes (canonized as pleasant at f0 50, Batch M3
   parity). Plugin default flipped to **Coherent ON**; stochastic
   kept as character. Kill/keep A/B for Sam: `out/m4-fixes/
   casc_f50_{stoch,coh}.wav`, `casc_f110_{stoch,coh}.wav`.
7. **Rebuild/validate/reinstall**: CLAP+VST3+AU rebuilt and
   reinstalled user-local; clap-validator tasks ok; auval SUCCEEDED
   (after `killall AudioComponentRegistrar`); standalone + Steinberg
   VST3-validator remain CLT-blocked (known). QC renders in
   `out/m4-fixes/`.

## STEREO prototypes round 1 (2026-07-22)

Engine goes true stereo: `Engine::process(&mut [f32], &mut [f32])`.
With width = decohere = 0, both channels are the canonical mono voice
**bit-identically** (regression: max diff vs pre-stereo render 6.7e-05,
fully explained by the fix-round tilt-compensation constant; R==L
exact). These are PROTOTYPES, not the final stereo program.

- **Width** (param 22, 0..1): per-mode L/R phase tap — L = u,
  R = u·cosθ_k + v·sinθ_k, θ_k = π·width·ramp_k. Zero extra state.
- **Decohere** (param 23, 0..1): per-mode L/R micro-detune, dual
  rotor, ±8 cents max, golden-ratio salted. Skipped entirely at 0.
- **Stereo Floor** (param 24, 0..1, default 0.3): how far DOWN the
  spectrum both effects reach. ramp_k = max(raw_ramp, 1−floor) where
  raw_ramp rises over ~3 octaves above 4·f0. Floor 1 = sub fully
  protected; floor 0 = full-spectrum decoherence, sub included —
  negative LR correlation is intended, not guarded against.
- Satellites: alternate-seat equal-power panning at ±width·0.7 (unity
  at center). Dust: two uncorrelated chains when stereo engaged;
  legacy single chain (bit-identity) otherwise.
- Cascade-attack DC click, real fix: coherent shadow rings now armed
  with golden-ratio per-mode PHASES (quadrature init alone left ~100
  aligned rising sines = coherent LF ramp). Measured: attack-1 LF
  (<30 Hz, first 5 ms) now equals attack-0 within 0.2 dB; first
  sample exactly 0.

Measurements (REPORTED, not acceptance gates — per Sam, mono
compatibility is not a design constraint):
- kick f36, w0.7 d0.5, floor 0.3: corr 20-120 Hz +1.000; 150-1000 Hz
  +0.976 (low-tuned membranes have few modes above 4·f0 — floor 0.3
  barely widens a kick).
- kick f36, w1.0 d0.8, **floor 0**: corr 20-120 Hz **−0.789**;
  150-1000 Hz −0.720 — the full-spectrum negative-correlation sub.
- plate f50 cascade, w0.7 d0.5, floor 0.3: corr 20-120 Hz +0.967;
  1-8 kHz −0.262.
- Mono-fold (L+R vs 2×mono): kick −0.87 dB, plate −3.49 dB (incoherent
  HF sum, physically expected), wires +0.11 dB.
- Cost: 2 s render wall time 0.03 s at decohere 0 and 0.9 alike
  (≈70× realtime; dual-rotor cost unresolvable at timer floor).

Rebuilt, reinstalled user-local; WRAC production checks pass;
clap-validator --only-failed clean; auval SUCCEEDED. Params 22-24
additive; nothing renumbered. Dust regression test passes (note:
`cargo test` doc-test line reads "0 tests" — the unit test is there
and green).

## M6 — Size macro + housekeeping (2026-07-22)

- **Param 25 "Size"** (0.4–2.5, default 1.0): `clg_engine::apply_size_macro`,
  the Batch 004c law — f0 ∝ 1/size; density n_axial ×size^0.45 (49/100/196
  modes measured at 0.5/1/2 on the n10 plate); T60 ×size^0.7 (ring
  969/1406/2309 ms); cascade τ ×size^1.3; nonlinear drive
  d = 0.85·tanh((v²/size^1.5)/0.85) — soft ceiling, measured
  0.846/0.668/0.305: the ≈0.9 artificiality zone is unreachable.
  Algebra (no velocity double-count): cascade_amt ×= d/v²;
  glide_st → 6·log2(1+(2^(g/6)−1)·d/v²). Applied per note-on AFTER
  note/tune/vel-curve/brace. CLI `--size`.
- **Param 26 "Vel Curve"** (0.25–4.0, default 1): velocity^curve at
  note-on (the Batch 002 exposed-ladder promise). CLI `--vel-curve`.
- **Sample-rate pass**: dust one-poles (env 4 ms, HP 1.5 k, LP 6.5 k) and
  the NL1 e-smooth coefficient now derived from sr (were 44.1k literals;
  identical values at 44.1k to ~1e-5 — regression max diff 1.4e-05).
  44.1 vs 48 kHz dust-band centroid within 1.7%.
- **Sample-accurate note timing**: audio.rs renders in segments split at
  note-event offsets (was block-granular). Up to 64 notes/block.
- **MPE/choke (design note, not implemented)**: MPE pressure → bracing
  macro per-voice (coupling+choke continuous while held); slide →
  strike position on retrigger; choke = note-off optional damp (a
  brace_choke-style release envelope applied at note-off, param-gated
  so one-shot behavior stays default). Needs per-voice param
  modulation plumbing — post-panel work.

## STEREO round 2 (2026-07-22)

Params 27–30 (additive; defaults bit-identical — verified `cmp`
identical vs pre-round render at all-defaults):

- **27 Rattle Level** (0–1, default 0.5): the formerly hardcoded rattle
  mix — Sam's "push the satellites out further" knob.
- **28 Mode Spread** (0–1): per-mode equal-power pan, golden-salted,
  floor-ramped — the bank spatially split. Kick @ spread 1/floor 0:
  sub corr +0.12, mono-fold −4.6 dB.
- **29 Damp Asym** (0–1): ±25% max per-mode T60 divergence L vs R
  (dual-rotor; engages the R rotor even at decohere 0). NOTE: a *tail*
  effect — on a kick the early coherent bulk dominates (sub corr
  +0.98); expect it to read on long-ring plates.
- **30 Sub Rotate** (0–1): quadrature divergence via the INVERSE
  spectral ramp — acts on the low region, up to 90°. Kick @ 1.0: sub
  corr +0.02 (orthogonal bass — the "vast" geometry), mono-fold
  −2.8 dB. Three distinct sub-stereo characters now exist: decohere
  floor-0 (−0.79, beating), mode-spread (+0.12, split), sub-rotate
  (+0.02, orthogonal).
- **Per-channel satellites** (no param — rides stereo engagement): L
  bank listens to L rotors, R bank to the R signal (detuned rotor or
  width/sub-rotate tap), POST-decoherence; per-ear contact events
  (measured: L 1843 / R 1743 @ decohere 0.8; L 1907 / R 2509
  everything-on), per-ear radiation with independent normalizers.
  Compromise, documented: when the R rotor isn't engaged (width-only
  stereo), R-bank *reactions* have no independent rotor to push — R
  contacts radiate but their reaction is not fed back (L-bank reaction
  feeds the shared rotor). Full reaction symmetry requires the
  dual-rotor path (any decohere/damp-asym > 0).
- Cost: everything-on 2 s render = 0.074 s wall (≈27× real-time,
  single voice, release).

## M7 — the exciter family (2026-07-22)

The architecture's "acoustic shader" slot, completed under the
clean-fucked-fidelity doctrine (PNT 005): all exciters are band-limited
FORCE signals into the existing bank drive path; no waveshaping anywhere.

**Params (additive ABI; table now 0–33):**

| id | name | notes |
|----|------|-------|
| 31 | Exciter | Mallet / Burst / Buckling / Raw |
| 32 | Ex Color | Mallet: stiffness trim (0.5 neutral); Burst: dark↔bright (LP 2k→9k, HP 200→1k); Buckling: click sharpness (3× one-pole LP 1.5k→12k); Raw: soften LP 500 Hz→20 k |
| 33 | Ex Time | Mallet: contact-time ×4^(0.5−t) (0.5 = 1×); Burst: length 2–80 ms log; Buckling: base rate 30–900 clicks/s log; Raw: DC-kick tail 0–30 ms |

**Laws:** Mallet at 0.5/0.5 is bit-identical to M3 (regression-proven).
Burst: seeded xorshift noise × Hann env through HP + 2× one-pole LP.
Buckling: stochastic click train, power-law amplitudes (u^(−1/1.5)
clamped), exponential inter-click intervals with rate = rate0 ×
max(e_norm, 0.04) and click strength × √e_norm (+30 ms warmup window) —
crumple rides and dies with the bank's own energy; passivity by
weakening re-injection (no clamped-force feedback, per the satellite
lesson); 600-click backstop cap. 3× one-pole click filter meets the
band-limit spec: renders measure −64 dB above 0.45·sr. Raw: 1-sample
impulse (+optional sin-bump DC-kick tail) through 2× one-pole LP;
−126 dB above 0.45·sr.

**QC:** defaults regression bit-identical vs pre-M7 build (git-stash
A/B, `cmp` clean). Buckling die-off measured on plate f50: HF click
events 299/1136/1128/6 across 0–100/100–300/300–600/600–1000 ms — the
crumple follows the body's energy and dies. Listening set:
out/exciters-v1/ (17 files: 4 exciters × 3 bodies + buckling ex-time
sweep + burst color pair).
