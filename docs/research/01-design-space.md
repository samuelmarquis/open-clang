# 01 — The design space of physical / gritty drum synthesis

*open-clang research corpus, entry 01. Compiled 2026-07-21 from a
multi-agent research sweep (6 search angles, 25 sources fetched, 113
claims extracted, top 25 adversarially verified by 3-vote panels).
Confidence is tiered honestly: **§A** survived 3-0 adversarial
verification against primary sources; **§B** is sourced and quoted
verbatim but was not put through verification; **§C** is what we could
not find. Design implications in **§D** are our synthesis, not sourced
fact.*

*Clean-room hygiene as in open-pitch-quant: public documentation only,
no disassembly. Where commercial behavior matters (Torque, Vocodex),
the plan is measured characterization via a probe pack — see §C.3.*

---

## §A. Verified findings (3-0 adversarial panels, primary sources)

### A.1 The "gritty" percepts are emergent, not add-ons

The defining hard-hit drum percepts — dramatic attack, downward pitch
glide, snare rattle, crash-like high-frequency buildup — fall out of
two specific mechanisms in the NESS (Edinburgh, PI Stefan Bilbao) FDTD
models; they are not effects layered on top:

- **Föppl–von Kármán (FvK) membrane/plate nonlinearity** produces
  amplitude-dependent pitch glide and the dramatic attack of hard-hit
  bass drums and timpani. Strike **position controls timbre; strike
  amplitude controls pitch glide** — a physically meaningful
  velocity→glide, position→timbre parameter mapping for free.
  [Bilbao et al., *The NESS Project*, CMJ 43:2/3 (2020),
  doi:10.1162/COMJ_a_00516; Rhaouti/Chaigne/Joly, JASA 105(6) 1999;
  Torin & Bilbao, SMAC 2013]
- **Explicitly simulated collisions, not added noise**, make the
  rattle: the NESS snare model (two membranes + rigid air cavity,
  bottom membrane against stiff snares) gets its rattle from repeated
  snare–membrane collisions via an energy-conserving contact-dynamics
  FD scheme; mallet–membrane impacts are handled the same way rather
  than as a prescribed excitation signal. No noise source appears
  anywhere in the model. [Torin/Hamilton/Bilbao, DAFx-14, pp.145-152;
  Bilbao/Torin/Chatziioannou, Acta Acustica 101(1) 2015,
  arXiv:1405.2589]

### A.2 The nonlinear plate is the clang

A simple FDTD scheme for the dynamic von Kármán plate (rectangular,
free boundaries) reproduces the key nonlinear gong/cymbal timbres —
transient pitch glides and rapid buildup of high-frequency energy —
from **a few physically meaningful parameters** (E, thickness, density,
Poisson's ratio, tension, two damping coefficients, dimensions). The
crash character appears as a **harmonic buildup within ~the first
50 ms** after a high-velocity strike; buildup time scales with plate
size. [Bilbao, *Sound Synthesis for Nonlinear Plates*, DAFx-05]
Caveat from the source: a rectangular plate is a first approximation —
real gongs/cymbals are shells.

**Refuted (0-3)**: "direct numerical simulation is the *only* reliable
route to nonlinear gong/cymbal dynamics." Nonlinear **modal**
approaches (Ducceschi/Touzé, J. Sound & Vib. 2015 lineage) are viable —
do not treat FDTD as mandatory for clang synthesis. This is the single
most implementation-relevant verdict in the sweep.

### A.3 Compute tiers: what is and isn't real-time

- **Full 3D embeddings are offline territory.** NESS embeds membranes/
  plates in a 3D air box (component-tailored grids, pressure/velocity
  continuity coupling, rigid-wall conditions) and needed GPGPU to run
  at all [Bilbao & Webb, JAES 61(10) 2013]. Real-time was achieved only
  for brass and modular networks; the percussion systems were never
  released as tools. Verified current through 2026: no real-time
  full-3D air-embedded drum simulation exists.
- **Wave-based cost scales as V·f⁴** (cells ∝ V·f³, timestep ∝ f from
  CFL) — full-audio-bandwidth FDTD of large domains is out of reach at
  plugin runtime; this is *why* modal methods (van den Doel & Pai
  lineage, Presence 1998; FoleyAutomatic, SIGGRAPH 2001) are the
  canonical real-time family. [Liu & Manocha survey, arXiv:2011.05538]
  Scoping caveat: a *small* 2D membrane/plate FDTD can still run
  real-time in a plugin — the infeasibility claim is about large
  volumetric domains.
- **CFL discipline**: for the 1-D FDTD wave equation, stability needs
  Courant number λ ≤ 1; **at λ=1 the scheme is exact** (full bandwidth,
  harmonic partials); at λ=0.6 bandwidth collapses to ~⅓ Nyquist and
  partials go inharmonic. Run as close to the stability limit as
  possible; in 2-D the bound tightens to λ ≤ 1/√2. [CMJ 2020]

### A.4 The 2-D digital waveguide mesh is the cheap membrane

Van Duyne & Smith's 2-D DWM (ICMC 1993) is mathematically equivalent to
the standard second-order FD membrane scheme at the CFL limit (lowest
possible dispersion error for that scheme), and each equal-impedance
4-port scattering junction computes **multiply-free: 7 add/subtracts +
1 bit-shift** (junction velocity = ½·sum of incoming; each outgoing =
junction velocity − corresponding incoming). Known artifact:
direction-dependent dispersion (zero along diagonals, HF slowdown along
axes → mistuned modes), with a closed-form speed formula; remedies are
allpass correction, boundary warping, or oversample+lowpass.
[ccrma.stanford.edu/~jos/pdf/mesh.pdf; Smith, PASP; Savioja & Välimäki
2000] Caveats: bit-shift economy presumes fixed-point with
energy-preserving rounding; "lowest error" is per-scheme, not zero.

### A.5 Thin-shell buckling is the crumple/click grit

Nonlinear thin shells (trash cans, oil drums, tin roofs) produce
clicking transients tied to **discrete buckling events**. Cirio et
al.'s two methods — substructured modal analysis + stochastic power-law
enrichment of buckling events (SIGGRAPH Asia 2016), and multi-scale
reduced wave-turbulence simulation (SIGGRAPH 2018, "tens of times
faster") — are a concrete algorithmic route to metallic clang/crumple
textures. The buckling-event→click mapping is directly implementable as
a percept model.

### A.6 The "audio raytracing" demo: probably wave-based, misremembered

The likely candidates for the remembered polyvocal demo are **not
ray-traced** — they're wave-based computer-animation sound systems from
Doug James's group:
- **Stanford wavesolver** (Wang/Qu/Langlois/James, SIGGRAPH 2018): a
  sharp-interface FDTD solver over animated geometry with **"acoustic
  shaders"** abstracting source-specific acceleration boundary
  conditions — one unified solver produces ringing near-rigid bodies
  with acceleration noise, FE thin shells, bubble-based water, and
  virtual characters. [graphics.stanford.edu/projects/wavesolver/]
- **KleinPAT** (Wang & James, SIGGRAPH 2019): precomputes *all*
  acoustic transfer fields of a linear modal model at once by grouping
  modes into "chords" — one time-domain wave sim per chord instead of
  per-mode Helmholtz solves, 100–1000× speedup. Its demo video of many
  simultaneously clanging modal objects matches a "polyvocal"
  recollection. [graphics.stanford.edu/projects/kleinpat/]
Both are offline/precomputation systems. The "acoustic shader" concept
— a pluggable excitation abstraction over a shared solver — is a
design idea worth stealing outright.

### A.7 NESS modular environments: the abstract-percussion pattern

- **Zero Code**: nonlinear interconnection of *plates* driven by
  percussive input, later refined to accept **audio input** — i.e., a
  physically nonlinear plate network usable as an effect/resonator
  (the physical big sibling of convolution-as-resonator).
- **Net1 Code**: strings and bars interconnected by **"rattling"
  nonlinear connections** characterized by mass, damping, stiffness —
  which may be nonlinear (hardening springs, intermittent contact
  loss).
- **Discrete energy balances were the design principle for all NESS
  code** — the numerical energy balance doubles as the stability
  condition and the debugging instrument. Directly transferable
  engineering practice for our engine. [CMJ 2020, pp. 24, 27]

### A.8 Vocodex: the documented mechanisms behind the trick

Verified verbatim against Image-Line's current manual:
- Band count **5–100**.
- **Band Distribution**: a freely drawable mapping envelope placing
  bands across **0 Hz–20 kHz or any sub-range** — deliberate clustering
  or spreading; the manual itself says it has "a dramatic effect on the
  vocoder process."
- **Modulator pitch shift**: per-band mapping envelope, **−1200 to
  +1200 cents** — modulator analysis bands shifted up to a full octave
  relative to carrier bands. This is the sole documented mechanism
  behind the octave-offset carrier/modulator pairing.
Note: that these yield "tactile, physical low end" is our perceptual
interpretation; the verified facts are the mechanisms.

---

## §B. Sourced but unverified (recovered extractions, verbatim quotes)

*These claims were extracted with quotes from the named sources but
were not adversarially verified (budget cut). Treat as strong leads.*

### B.1 Waves Torque / Organic ReSynthesis (ORS)

Sources: waves.com ORS page ("Sound Synthesis Renewed"); official
manual (assets.wavescdn.com/pdf/plugins/torque.pdf); Sound On Sound
review.

- ORS decomposes a signal into independently manipulable elements —
  **pitch, formant, carrier, envelope** — and reconstructs; "ORS
  algorithms are not time-dependent: manipulating one characteristic
  will not affect the time constants of the others" (their stated
  transient-preservation mechanism).
- **Torque is a formant shifter, not a broadband pitch shifter**: the
  main knob is ±1200 cents of *formant* shift acting mostly around a
  user-positioned **Focus band, 98–988 Hz (G2–B5)** — "the best Focus
  frequency for a kick drum might be in the area of 900 Hz" (manual:
  aim at a resonant peak / second harmonic, not the fundamental).
- Per-hit gating: **Threshold** −70..0 dBFS (default −48) decides which
  hits get retuned; **Torque Speed** attack/release 15–50 ms.
- **Latency: 32 samples @44.1/48k** (64 @88.2/96k, 128 @176.4/192k),
  phase-coherent; the Live components are zero-latency,
  non-phase-coherent. *Implication: far too small for a large-FFT
  phase vocoder — this is a time-domain or very-short-window
  resynthesis.* ORS is shared across Torque, Smack Attack, Sibilance,
  Submarine, OVox. No inventors, patents, or papers are named on the
  ORS page; the patent hunt came up empty (see §C.2).

### B.2 SOPHIE / Monomachine (the material-first doctrine)

Sources: MusicRadar "Pioneers: SOPHIE"; pcmusic.boards.net transcript
of the Elektron interview; Elektronauts threads; archived Sup Mag 2013
interview.

- "I've synthesized ideas for **latex, balloons, bubbles, metal,
  plastic, elastic** all on the Mono." — waveforms "pushed into shapes
  and materials," explicitly *not* samples.
- On "Hard" (2015): "hundreds of completely unique clangs, squeaks,
  bangs and squelches" instead of drum samples.
- Doctrine: "the language of electronic music shouldn't still be
  referencing obsolete instruments like kick drum or clap" —
  **material categories, not drum-kit taxonomy**. This is practically a
  product brief for open-clang's preset ontology.
- Concrete technique fragments from the community: Monomachine's
  built-in reverb; **very short delay times**; the **keytracking
  filter**; "LFO on the pitch of an FM bass-drum voice, letting it get
  loose, crazy, high-pitched" (the rubbery/latex recipe); attributed
  "pot-and-pan" metallic percussion (audio ref: youtu.be/tgV4tZRR7p0,
  opening). Community consensus attributes part of the character to the
  Monomachine's early-2000s digital rawness (artifacts as material).

### B.3 Comparative landscape (control-surface reference points)

- **Chromaphone/Collision** (shared AAS engine): exciter→resonator
  with **8 resonator types** (beam, marimba, string, drumhead/membrane,
  plate, open/closed tubes, manual), **bidirectional two-resonator
  coupling** (AAS claims first-in-class), mallet controls (stiffness,
  noise %, color) + filtered-noise exciter; per-resonator **hit
  position, listening position, decay, brightness, inharmonicity** —
  the canonical user-facing parameterization of a modal resonator
  (mode gains from strike/pickup position, damping, tilt, detuning).
  Collision sits at the clean/acoustic end of the space.
- **Microtonic** (manual + Lidström interview): 100% synthetic,
  sample-free; per-channel voice = pitch-modulated osc (sine/tri/saw)
  + noise through multimode filter → mix → **distortion that is
  deliberately not anti-aliased** (aliasing as character), and a noise
  filter Q reaching 10,000 where noise collapses into an "irregular
  sine tone" — i.e., a filter *becoming* a resonator. Uncorrelated
  stereo noise emulates reverb-like dispersion. Grit by intentional
  imperfection, the anti-physical pole of the space.
- **Convolution-as-resonator** (ModeAudio et al.): any sample loaded
  as an IR filters the input with its spectrum — striking a stone
  slab, convolving drums with it, transfers the object's character.
  Cross-convolution of percussion with tonal/ambient material as a
  deliberate hybrid technique.
- **Vocodex resonator-adjacent controls** (manual): per-band filter
  ORDER 1–4, band width, "filter flatness" (pointy↔flat peaks),
  modulator bandwidth multiplier (resonant ↔ breathy/raspy); per-band
  envelope follower Hold/Attack/Release with ×0.125–×8 per-band
  mapping — long release is explicitly "laggy and reverberant"
  (resonator-decay-like tail shaping).

---

## §C. Gaps and next actions

1. **Audeka & Rawtekk: nothing survived, nothing recovered.** Their
   technique documentation lives mostly in video (YouTube masterclasses,
   e.g. Audeka's metallic drum design) — text-searchable sources are
   thin. Next: targeted video-transcript dive, and/or the user's own
   notes; treat as future corpus entry.
2. **Torque internals unverified; no patents found.** The presumed
   Shashoua/Waves patent trail produced nothing verifiable.
   **RESOLVED BY MEASUREMENT 2026-07-21** → `02-torque-measured.md`:
   dual-path resynthesis (intact residual + ratio-shifted copy of the
   Focus-region component), tonal sweeps pass untransposed, action
   localized <1 kHz, 32/0-sample PDC confirmed, soft per-hit
   threshold.
3. **Probe pack for the Windows rig** — **RETURNED & ANALYZED
   2026-07-21** (see `02-torque-measured.md`, `03-vocodex-measured.md`;
   p2 needs ranked in 03 §p2). Original dispatch record:
   `clang-probes-p1.zip` (20 probes: 13 synthetic, 7 library
   excerpts; generator `tools/make_probes.py`) taildropped to sam-pc,
   where an operator Claude automates the render matrix through
   REAPER (and FL for Vocodex, preset `autovocoding`). Protocol +
   operator notes: `testdata/probes-p1/`. Matrix: ~142 Torque renders
   (pitch×focus grid, threshold, speed, std/live phase, analytical)
   + ~15 Vocodex renders incl. band-distribution/mod-pitch isolation
   variants. Returns → `02-torque-measured.md` /
   `03-vocodex-measured.md`.
4. **Open question (verified as open)**: can FvK-grade nonlinear
   behavior run in a real-time budget via nonlinear-modal schemes
   (Ducceschi/Touzé; scalar-auxiliary-variable / energy-quadratization
   methods) — and what quality is lost vs offline FDTD? This is
   open-clang's central research bet, and §A.2's refutation says the
   door is open.

---

## §D. Design implications for open-clang (synthesis, not sourced fact)

1. **Modal core, nonlinearly enriched.** The real-time-viable center
   of the space is a modal resonator bank with the three verified grit
   mechanisms grafted on: (a) amplitude-dependent stiffening/glide
   (FvK-style, via nonlinear modal coupling — the Ducceschi/Touzé
   route); (b) explicit collision objects (rattles/snares as
   secondary mass-contact systems, not noise layers); (c) stochastic
   buckling-event enrichment (Cirio-style clicks) for crumple/clang.
2. **The verified percept mappings become the performance controls:**
   velocity→pitch glide depth, strike position→timbre/mode weighting,
   plate size→HF-buildup time, bracing/coupling→how much energy enters
   the resonant body (the Houdini axis: braced vs unbraced).
3. **Band distribution generalizes into the resonator bank**: a
   drawable envelope over mode/band *placement* (cluster low for
   tactile beating density, spread high for air) is the Vocodex trick
   reborn as a synthesis-side control — with per-band octave offset as
   the low-end thickener.
4. **Exciter/resonator separation with "acoustic shaders"**: pluggable
   excitation models (mallet, slap, buckling burst, audio-in) over a
   shared resonator engine, borrowing the Stanford abstraction.
5. **Energy balance as engineering discipline** (NESS): every
   nonlinear element ships with a discrete energy audit; instability
   is a bug caught by the meter, not by the ear. Pairs with strict
   no-alloc RT discipline from day one.
6. **Material taxonomy, not drum taxonomy** (SOPHIE): presets and UI
   speak latex/metal/plastic/membrane/air, not kick/snare/clap.
7. **Keep the anti-physical pole in reach** (Microtonic): deliberate
   aliasing, filters-as-resonators at absurd Q, digital rawness as a
   *material*. Grit is not only physics.

## Sources (primary unless noted)

- Bilbao et al., "Physical Modeling, Algorithms, and Sound Synthesis:
  The NESS Project," CMJ 43:2/3, 2020 (mdphys.org/PDF/cmj_2020.pdf)
- NESS archives: ness.music.ed.ac.uk (3d-embeddings, releases,
  modular-environments)
- Bilbao, "Sound Synthesis for Nonlinear Plates," DAFx-05
- Torin/Hamilton/Bilbao DAFx-14; Bilbao/Torin/Chatziioannou, Acta
  Acustica 2015 (arXiv:1405.2589); Bilbao & Webb, JAES 61(10) 2013
- Van Duyne & Smith, ICMC 1993 (ccrma.stanford.edu/~jos/pdf/mesh.pdf);
  Smith, *Physical Audio Signal Processing*
- Liu & Manocha, "Sound Synthesis, Propagation, and Rendering: A
  Survey" (arXiv:2011.05538); van den Doel & Pai 1998/2001
- Cirio et al., SIGGRAPH Asia 2016 & SIGGRAPH 2018
- Stanford wavesolver & KleinPAT project pages (graphics.stanford.edu)
- Image-Line Vocodex manual (parameters + tutorial pages)
- Waves ORS page; Torque manual PDF; Sound On Sound Torque review
  *(unverified tier)*
- MusicRadar "Pioneers: SOPHIE"; pcmusic.boards.net Elektron interview
  transcript; Elektronauts threads; Sup Mag 2013 (archive.org)
  *(unverified tier)*
- AAS Chromaphone 3 page; Ableton Collision page; MusicRadar
  Chromaphone review; Sonic Charge Microtonic manual; audionewsroom
  Lidström interview; ModeAudio convolution article *(unverified tier)*
