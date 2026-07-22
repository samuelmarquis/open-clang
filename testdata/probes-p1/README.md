# clang-probes-p1 — Torque / Vocodex characterization pack

Probe set for empirically characterizing Waves Torque and Image-Line
Vocodex (preset: `autovocoding`) as design references for the
**open-clang** drum-synthesis engine. Clean-room protocol: public
documentation + measured input/output behavior only.

- `probes/` — 20 mono WAVs, 44.1 kHz / 24-bit. `p*` are synthetic
  (deterministic, seeded); `r*` are excerpts from Sam's own library,
  peak-normalized to −6 dBFS. See `probes/MANIFEST.tsv` for the
  one-line purpose of each file.
- `OPERATOR-NOTES.md` — instructions for the operator (a Claude
  driving REAPER on sam-pc) — render matrix, naming grammar, QC
  gates, return packaging.

Generated 2026-07-21 by `tools/make_probes.py` in
`~/Developer/open-clang` (macOS side). Analysis of the returns lands in
`docs/research/02-torque-measured.md` and `03-vocodex-measured.md`.
