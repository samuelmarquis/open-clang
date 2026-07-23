# PROVENANCE — origin and IP record

*Factual record of where everything in this project came from.
Written July 2026 for internal use and future counsel review. This
is a statement of facts by the people who did the work, not legal
advice.*

## 1. Authorship and ownership

- All code in this repository was written in July 2026, in this
  repository, by Claude (Anthropic) operating under the continuous
  direction of Samuel Marquis. Under Anthropic's commercial terms,
  model output belongs to the user; the sole rights-holder is
  Samuel Marquis.
- Human creative direction is documented decision-by-decision in
  `LISTENING-LOG.md` (kept local, never committed): every
  mechanism, parameter, and voicing choice in the engine traces to
  a specific listening verdict. `docs/design/` records the design
  reasoning; `PATHS-NOT-TAKEN.md` records rejected directions.
- There are no third-party contributors and no contributor
  agreements. Single owner.
- **No LICENSE file has ever existed in this repository. No
  open-source grant has been made.** The repository was publicly
  visible at github.com/samuelmarquis/open-clang from 2026-07-22
  until made private; public visibility on GitHub grants no
  copyright license to viewers.

## 2. What the product contains

- `rt/engine` (`clg-engine`) — the entire DSP engine. Original
  Rust. **Zero dependencies** (not even libc beyond Rust's core).
- `rt/cli` — offline renderer. Original Rust, standard library
  only.
- `wrac/` — plugin shell built on the WRAC template, which is the
  author's own sister project (`open-pitch-quant`), plus the
  third-party components listed in §4.
- **No third-party audio, samples, presets, impulse responses,
  wavetables, or datasets ship in any build artifact.** The
  shipped binaries contain code and numeric constants only.

## 3. Where the synthesis techniques come from

Every mechanism was implemented directly in this repository from
published literature or first principles. No external source code
was consulted, ported, or translated at any point (the only code
reuse in the project is the author's own WRAC template).

- Modal synthesis and coupled-form (rotation) resonators —
  textbook digital signal processing, in the public literature
  since the 1980s.
- Stiffening pitch glide — reduction of Föppl–von Kármán plate
  mechanics (1907-era continuum mechanics) as treated in published
  musical-acoustics research (Bilbao; Ducceschi/Touzé).
- Contact mechanisms (satellites, wire bank) — published
  penalty/Hertzian contact models and standard vibrating-table
  dynamics.
- Spectral cascade — original mechanism, loosely inspired by
  published wave-turbulence observations.
- Buckling exciter — published model (Cirio et al.: power-law
  click trains).
- NESS project (University of Edinburgh) — **papers only.** NESS
  source code was never obtained, opened, or read.
- Known patent history: the widely-cited physical-modeling patent
  families (Stanford/Yamaha digital waveguides, 1980s–90s) expired
  years before this project began. No live patent is knowingly
  practiced by the shipped engine.

## 4. Third-party components and their licenses

| component | role | license | action needed |
|---|---|---|---|
| CLAP API | plugin format | MIT | none |
| clap-wrapper (free-audio) | wraps CLAP into VST3/AU | MIT | none |
| Steinberg VST3 SDK | VST3 interface (via clap-wrapper) | dual: GPLv3 **or** Steinberg proprietary agreement | **sign the (free) Steinberg agreement before any closed-source distribution; follow VST trademark guidelines** |
| Apple AudioUnit SDK | AU interface | standard Apple SDK terms | none |
| Rust crates: `atomic_float`, `log`, `serde`, `serde_json`, `toml` (build-time) | shell utilities | MIT/Apache-2.0 | none |
| `wrac_*` crates | shell framework | author-owned | none |

Development-time tools (clap-validator, auval, Ableton Live, the
Python analysis lab) do not ship and impose nothing on the product.
Re-verify the dependency tree mechanically (`cargo deny` /
`cargo license`) at ship time.

## 5. Study of competing products (the clean-room record)

Two commercial plugins were studied to inform design: Waves Torque
and Image-Line Vocodex.

- **Method**: public documentation plus black-box input/output
  measurement — prepared probe audio was run through the author's
  licensed retail copies and the rendered output was measured. The
  probe packs and returned renders are archived in the repository
  (`clang-probes-p1..p3-return.zip`) and the findings in
  `docs/research/02–05`. **At no point was either product
  disassembled, decompiled, or inspected at the code level.**
- **What was taken**: measured behavior — band placements, level
  laws, time constants, shift accuracy. These are facts about how
  the products respond to signals, used as design targets for
  original, differently-built mechanisms.
- **What was not taken**: no code, no presets, no artwork, no UI,
  and no use of either product's name in the product itself.
- **Open patent gate**: Waves holds patents around its "Organic
  ReSynthesis" technology (inventor Meir Shashoua et al.). The
  currently shipped synthesizer performs **no analysis or
  resynthesis of external audio** and is believed to be nowhere
  near those claims. However, one deferred, not-yet-built feature — the
  audio-input "Retune"/effect mode — was designed with Torque's
  measured behavior as a reference. **Before that feature is built
  or shipped, obtain a patent claims review.** This is the single
  known forward-looking IP action item in the project.

## 6. Reference audio used for measurement

Real drum recordings were analyzed to set numeric design targets
(fundamental frequencies, decay times, spectral peak counts —
recorded in `docs/research/06-snare-measured.md`):

- Superior Drummer 3 output (author's licensed copy);
- a commercially licensed sample pack (Angelo Mides drum loops);
- the author's own processed material.

The audio was used solely for measurement. **No recording is
redistributed, embedded, resynthesized, or used as source material
by the product.** The derived quantities (e.g. "tail decays in
0.45 s", "16 spectral peaks") are unprotectable facts.

## 7. AI-authorship note

Substantially all code text was generated by an AI system under
continuous, documented human direction. Current US Copyright Office
guidance limits copyright registration for purely AI-generated
material; how much protection the human-directed record here
supports is a question for counsel. Practical posture:

1. The human contribution is unusually well documented (the
   listening log is a decision-by-decision authorship diary).
2. Closed-source distribution protects the code as a trade secret
   independent of copyright.
3. Selling the software requires no copyright determination at
   all; the question only matters for enforcement against copiers.

## 8. Naming

"open-clang" is a working title. It implies an open-source grant
that was never made, and it collides with LLVM's "Clang" compiler
trademark in spirit. Rename before any commercial release.

## 9. Open actions

- [ ] Make the repository private (owner).
- [ ] Sign the Steinberg VST3 agreement before closed-source
      distribution.
- [ ] Patent claims review (Waves / Organic ReSynthesis) **before**
      building the audio-input Retune/effect mode.
- [ ] Mechanical license audit (`cargo deny`) at ship time.
- [ ] Counsel review of this document.
- [ ] Product rename.
