# open-clang

A drum/impact synthesizer whose grit is mechanistic, not cosmetic:
modal resonator core + the three measured/verified physical grit
mechanisms (amplitude-dependent stiffening glide, explicit collision
satellites, post-strike spectral cascade), pluggable exciters,
material-language controls. Oriented toward the sound of the impact
that killed Houdini. No relation to the compiler; the CLI binary is
`clg` so your PATH stays safe.

- **`docs/design/01-architecture.md`** — the engine plan.
- **`docs/research/`** — evidence corpus: 01 design-space survey
  (verified), 02–04 measured characterizations of Waves Torque and
  Image-Line Vocodex (probe-pack protocol, packs p1/p2 rendered on a
  remote rig; protocols in `testdata/probes-p*/`).
- **`lab/`** — Python prototype lab (exploration record; the Rust
  engine will be canonical once it exists).
- **`LISTENING-LOG.md`** — every batch, every verdict. Taste is data.
- Stack per `~/Developer/open-plugin-template`: Rust engine + CLI
  (`rt/`), WRAC shell for CLAP/VST3/AU (`wrac/`), Nix flake, native
  CPU-pixel panel. Plugin shell arrives at M4.

Build (engine era — the Rust engine is canonical as of M3 parity):

```sh
nix develop
cd rt && cargo build --release
./target/release/clg render hit.wav --arch membrane --f0 36 --vel 0.95 \
  --pos 1.0 --listen-pos 1.0 --glide 8 --out-tilt -7 --brace 0
```

The Python lab (`lab/`) is frozen — the exploration record for
Batches 001–005. Development is driven by a listening log kept
outside the repo: every mechanism here survived a human verdict
before it stayed.
