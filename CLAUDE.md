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

## Current state (2026-07-22)

Instrument installed (CLAP/VST3/AU, `aumu`/`Clg1`/`Oclg`), params
0–32 dense, engine = `rt/engine` (canonical since M3 parity; Python
`lab/` frozen). Deferred/queued: panel (waits on Sam's external
design system), Alignment articulator (post-panel, drawable curves),
satellite redesign (cascade-coupling headline), effect mode
(sidechain vs separate plugin, undecided), presets (Sam's friends,
post-panel), MPE/choke (designed, post-panel).
