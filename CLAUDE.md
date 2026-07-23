# open-clang — session conventions

Standing instructions for Claude sessions in this repo. The full
project story lives in `docs/design/01-architecture.md`,
`docs/research/01..05`, `LISTENING-LOG.md` (local-only, gitignored),
and `PATHS-NOT-TAKEN.md`.

## Working rules

- **File edits use the Edit/Write tools, never shell scripts or
  python heredocs** — Sam reads the diffs in chat; opaque edits are
  not acceptable.
- **Never touch `~/Developer/open-pitch-quant`** — read-only
  reference project.
- **LISTENING-LOG.md is the primary decision record**: every render
  batch/build round gets an entry (config, asks, Sam's VERBATIM
  verdict, interpretation + action). It is gitignored — keep writing
  it, never commit it.
- Commit style: concise message + `Co-Authored-By: Claude Fable 5
  <noreply@anthropic.com>`. Push to `origin main`
  (github.com/samuelmarquis/open-clang).

## Engineering law (earned the hard way)

- **Pre-1.0 ABI policy**: no saved presets exist — breaking changes
  (param removal, renumbering) are allowed and preferred over
  deprecation. Param ids stay DENSE (enforced by
  `state::tests::param_table_matches_store`). Policy inverts at 1.0.
- **Batch param additions** — Ableton caches the param interface per
  project; every addition costs Sam a patch rebuild.
- **QC must exercise the CLAP process path** (clap-validator against
  the installed artifact), not just engine/CLI renders — engine-only
  regression and auval both passed while the plugin was mute (M7
  no-sound incident).
- **Passivity: bound the geometry, not the force** — no clamped
  forces in feedback paths (satellite limit-cycle incident); coupled-
  form resonators for anything with time-varying frequency (direct-
  form II explodes under glide).
- **Clean-fucked-fidelity doctrine**: harmonic richness through
  mechanism (cascade, collisions, buckling, alignment), never
  waveshaping; everything band-limited (≥60 dB down above 0.45·sr).
- **Normalizers must never include their consumers** — a running-peak
  normalizer whose input contains its own normalized output is a
  feedback gain > 1 → inf (M8 ring-radiation bug).
- **Unilateral reactions rectify into pumps at high contact duty** —
  a one-sided contact force at ~100% duty is a parametric pump that
  keeps the bank alive forever; floor the gap, gate the reaction
  (M8 gap→0 bug).
- **In-process fuzz first**: `param_fuzz_stress_host_path`
  (per-sample finiteness under full param fuzz, 8 voices) catches in
  0.6 s what black-box clap-validator takes 45 s to hang on — run it
  before every install.
- **Coupled-bank exchanges: normalize by the receiver's dissipation
  (1−r)** — a per-sample-driven resonator accumulates with gain
  1/(1−r) ≈ 10³ at resonance, so raw displacement→force coupling
  multiplies thousands into every loop (M10 cavity blew up to 1.8e38).
  Normalized, the loop gain is just the product of the dimensionless
  coupling constants — boundable by geometry.
- **QC paths must be bit-honest about transients** — the CLI's 1.5 ms
  render fade-in silently erased sub-ms attacks from batch 001
  through M9 (the Stick tick was in the plugin but not the renders).
  Never let a measurement/render path post-process what the plugin
  ships.
- **Install is `--release`, verified by hash** — `cargo xtask
  install` defaults to the DEBUG profile; the omitted flag shipped
  debug builds through M9–M11 (17.6× slower engine — 8 voices ran at
  0.7× realtime and underran Sam's M4 in a blank project, while the
  release engine idled at 12% budget). `tools/qc.zsh` is the only
  sanctioned install path: it fuzzes, builds+installs release,
  HASH-VERIFIES installed == release artifact, validates, and
  benches. (M12.)
- **No round ships without `clg bench`** — fixed methodology (recipe
  v3, 64-frame blocks, retriggered voices); the gate is the
  `full x8` worst-block row: < 25 % of block budget (M12 landed it
  at 11.7 %). Underruns live in the worst block, not the average.
- **Perf-work drift policy** — bit-exact where free; where float
  order changes, gate with a null test vs prior HEAD over the
  six-patch matrix (v3/wood/sats/casc/buck/dust, `lab/null_ab.py`):
  peak diff ≤ −80 dB rel peak. The gate caught two real bugs in M12
  (a sleep floor referencing the wrong peak, and single-block sleep
  on bursty tails — contact chaos amplifies ULP differences past
  −80 dB, so satellite contact keeps exact powf).
- **Finiteness gates are blind to BOUNDED eternal oscillation** —
  the M12.1 scream (loud, constant, hard-panned, param-immune;
  reset-only recovery) passed every finiteness fuzz. Standing cures:
  contact integrators resolve a contact over many samples (ω clamp
  0.5·sr — clamp the TOTAL stiffness, not the spring); the
  entry-rate fuse (a discrete rattle cannot re-enter contact >1 kHz
  for 300 ms — sustained machine-gun rate = pump by definition);
  the output airbag (4 s continuous envelope above 5 % of own peak
  → voice dead; healthy-unreachable). Standing gates: engine
  `decay_gate` tests (everything must decay to −80 dB UNASSISTED,
  zero airbag trips) and `clg hunt N SEED` for corpus sweeps. Fixes
  that touch the healthy signal path can never pass the null gate —
  two candidates (reaction LP, restitution exit guard) were
  correctly rejected by it; cures must be zero-effect until the
  pathology discriminator trips. (M12.1.)
- **A voice must DIE — bound lifetime by construction** (Sam's law,
  M13): downstream compression makes ANY immortal floor audible, and
  a never-sleeping voice is a CPU leak. The lifetime watchdog:
  patch-derived ceiling max(4 s, 2.25 × longest active T60, cap
  30 s), 250 ms fade, dead latch. Don't chase each quiet pump —
  bound the time, like bounding the geometry. Corollary for
  verification tooling: measurement horizons must account for event
  offsets (a retrigger-offset blind spot minted four phantom
  IMMORTAL verdicts before it was caught).

## Current state (2026-07-23)

Instrument installed (CLAP/VST3/AU, `aumu`/`Clg1`/`Oclg`,
RELEASE-profile, hash-verified), params 0–56 dense, engine =
`rt/engine` (canonical since M3 parity; Python `lab/` frozen). M11
wire bank battery-gated; snare "pretty good," voicing settled low
(f0 ≤ 155, out/snare-v3b). M12 perf (debug-install fixed, worst
block ~12–14 %), M12.1 scream killed, M13 watchdog (immortal floors
extinct, 12k-config sweep) + fittings network (Net1 params 53–56,
bar-rescue bet, out/net-v1 verdict pending). Queued: M11 asks 2–7,
M12 underrun retest. Deferred: panel (waits on Sam's external design
system; drawable graph lanes incl. decay-law curve), Alignment
articulator (post-panel), effect mode (sidechain vs separate plugin,
undecided — PROVENANCE patent gate first), presets (Sam's friends,
post-panel), MPE/choke (designed, post-panel).
