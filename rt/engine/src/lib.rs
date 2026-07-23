//! clg-engine — the open-clang modal drum engine (M3 v0).
//!
//! Real-time formulation of the frozen lab mechanisms (Batches 001-005):
//! per-sample two-pole modal resonators driven by an actual force pulse,
//! block-rate coefficient updates for the NL1 stiffening glide, an
//! energy-conserving NL3 cascade with attack-balance, the transect gain
//! lane (out-tilt v0), and split bracing (coupling / choke).
//! M3.2 (not yet ported): NL2 contact satellites + dust layer.
//!
//! No allocation after `Engine::new`. No host types. Single voice (the
//! plugin shell owns polyphony).

pub const MAX_MODES: usize = 256;
pub const MAX_SATS: usize = 4;
pub const SAT_PARTIALS: usize = 4; // M8: partials per satellite (multi-modal)
pub const CTRL_INTERVAL: usize = 32; // samples between coefficient updates
pub const R2_MODES: usize = 12; // M10: resonant (snare-side) head bank
pub const CAV_MODES: usize = 4; // M10: cavity air modes (Helmholtz + pipe)
pub const BED_BQ: usize = 6; // M10: bed band-limit biquad sections (order 12)
pub const N_WIRES: usize = 16; // M11: wire-bank resonators (Net1)

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arch {
    Membrane,
    Plate,
    Bar,
}

/// M7 — the exciter family (the architecture's "acoustic shader" slot).
/// All exciters emit a FORCE signal consumed by the bank drive path;
/// everything band-limited per the clean-fucked-fidelity doctrine
/// (PATHS-NOT-TAKEN 005: richness through mechanism, never waveshaping).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Exciter {
    /// Hann contact pulse (the M3 original; default).
    Mallet,
    /// E2 — shaped noise burst: the slap/brush family.
    Burst,
    /// E3 — stochastic click train, power-law amplitudes, rate riding the
    /// bank's own energy: crumple that dies with the body (Cirio, 01 §A.5).
    Buckling,
    /// E5 — single-sample impulse + optional soften/DC-kick tail. Clean.
    Raw,
    /// M10 — drumstick (Sam-requested): light, stiff Hertzian contact.
    /// Raised-cosine force pulse 0.08–0.8 ms — an order of magnitude
    /// shorter than Mallet, so the spectrum is flat well into the kHz
    /// range BY MECHANISM (the snap, doctrine-clean). Color = tip↔
    /// shoulder (contact time), Time = family contact-time scale.
    Stick,
}

/// Flat, Copy, one knob per field (template §3). The Bracing/Size macros
/// live in the shells; the engine takes the granular laws.
#[derive(Clone, Copy, Debug)]
pub struct EngineParams {
    pub arch: Arch,
    pub f0: f32,
    pub velocity: f32,
    pub position: f32,   // 0..1 edge->center diagonal
    pub listen_pos: f32,
    pub stiffness: f32,
    pub t60_base: f32,
    pub tilt: f32,
    pub n_axial: u32,
    pub glide_st: f32,
    pub out_tilt_db_oct: f32,
    pub cascade_amt: f32,
    pub cascade_tau: f32,
    pub cascade_split: f32,
    pub cascade_attack: f32,
    pub cascade_conserve: bool,
    /// true = coherent cascade (lab-faithful: receivers ring freely from a
    /// trigger kick, output-shaped by buildup x energy); false = stochastic
    /// TD (noise-driven receivers — brighter, dustier).
    pub cascade_coherent: bool,
    pub brace_coupling: f32, // 0 = full coupling (unbraced), 1 = max reflection
    pub brace_choke: f32,    // 0 = none, 1 = full early choke
    pub brace_tension: f32,  // pitch-up fraction at full brace (macro feeds this)
    pub brace_t60: f32,      // overall T60 scale (1 = neutral)
    pub brace_low_bonus: f32, // 0..1: low-mode ring bonus (unbraced body keeps the blow)
    // NL2 — contact satellites (M3.2 port of lab/nl2.py; M8 redesign)
    pub sat_count: u32,
    pub sat_fs: [f32; MAX_SATS],
    pub sat_t60: [f32; MAX_SATS],
    pub sat_seat: [f32; MAX_SATS],
    pub sat_rest: [f32; MAX_SATS], // hover gap, fraction of est. peak seat displacement
    pub sat_level: [f32; MAX_SATS],
    // M8 — multi-modal satellites: each satellite radiates a small modal
    // OBJECT (up to SAT_PARTIALS partials rung by contact), not a sine.
    // ratio 0.0 = unused slot. Partial decay derives from sat_t60 scaled
    // by ratio^-0.7 (higher partials die faster).
    pub sat_pr: [[f32; SAT_PARTIALS]; MAX_SATS], // partial freq ratios
    pub sat_pa: [[f32; SAT_PARTIALS]; MAX_SATS], // partial amplitudes
    // M8 — the rattle control surface (one batched drop, params 33-39)
    /// 0..1 — contact shocks pump the cascade receiver rings through their
    /// OWN drive path (collision-clang works with cascade_amt at 0).
    pub rattle_casc: f32,
    /// 0..1 — regime axis: 0 = pressed (legacy spring), 1 = thrown-and-
    /// settling (gravity + restitution < 1: geometric settle chatter).
    pub bounce: f32,
    /// 0..1 — static gap scale (0.5 = preset-neutral): tight buzz <-> loose.
    pub rattle_gap: f32,
    /// 0..1 — velocity->gap depth: 0 = static, 1 = fully velocity-affected
    /// (harder hit = thrown wider). Sam's addition to the M8 surface.
    pub gap_vel: f32,
    /// satellite retune in OCTAVES (±2), applied to all partials + contact ω.
    pub rattle_tune: f32,
    /// 0..1 — keytrack blend: 0 = fixed Hz, 1 = full note tracking (ref 36 Hz).
    pub rattle_track: f32,
    /// 0..1 — seat migration through the decay (per-channel salted: free
    /// rattle decorrelation whenever walk > 0).
    pub walk: f32,
    // NL2 — the dust layer (statistical micro-contacts; Batch 003 verdict)
    pub dust_level: f32,
    pub dust_thr_db: f32,
    pub dust_follow: f32,
    // M9 — the wire-bed (dust promoted to a snare mechanism). All four at
    // defaults (0/0/0/0.5) reproduce the legacy dust path bit-exactly.
    /// 0..1 -> release 30 ms..2.5 s (log). The decay INVERSION: release may
    /// exceed the body's T60 — tone dies, noise hangs (the snare tail).
    pub bed_release: f32,
    /// 0..1: follower drive from full-band output env (0, legacy) to a
    /// mid-band head-motion region ~150-800 Hz (1) — the cavity-resonance
    /// proxy until M10 builds the real cavity.
    pub bed_source: f32,
    /// 0..1: wire-comb shimmer (short feedback comb, per-channel salted,
    /// fb < 0.9 with in-loop LP per the band-limit doctrine).
    pub bed_comb: f32,
    /// 0..1: noise spectral center, dark (~0.7-4 kHz) to bright; 0.5 =
    /// the legacy 1.5-6.5 kHz band. M10: the top half steepens to reach
    /// the biquad-opened top octave (1.0 ≈ 3.3-15.6 kHz).
    pub bed_bright: f32,
    // M10 — NESS topology: batter ⇄ cavity air ⇄ resonant head (R1⇄R2,
    // the literal snare topology; the "hollow"). 0 = fully off, bit-exact.
    /// 0..1 coupling amount (drives cavity, R2, return reaction, and the
    /// R2 radiation mix together).
    pub cavity: f32,
    /// cavity fundamental, Hz (Helmholtz-ish; pipe partials ride it).
    pub cavity_tune: f32,
    /// resonant-head tuning interval vs f0, semitones (snare-side heads
    /// are typically tuned above the batter).
    pub head2_tune: f32,
    /// 0..1: resonant-head damping (0 = ringing head, 1 = choked/wire-
    /// loaded; the wire-bed rides this head's motion when cavity is on).
    pub head2_damp: f32,
    // M11 — the wire bank (Net1): wires as ACTUAL resonators in
    // intermittent contact with the snare-side head. The M10.5 autopsy
    // convicted the noise bed as a wire mechanism: real snare tails
    // carry 13–17 DISCRETE tonal peaks (0.4–8 kHz, stable per
    // instrument, MORE visible at ghost velocity) — statistics cannot
    // produce stable peaks, resonators can. 0 = off, bit-exact legacy.
    /// 0..1 — wire-bank level (mix vs bank peak, satellite pattern).
    pub wires: f32,
    /// wire-band center, Hz (fixed-per-patch golden-salted placement
    /// spans ±1.75 octaves around this — the instrument fingerprint).
    pub wire_tune: f32,
    /// wire ring T60, seconds (SD-measured dry-snare zone ≈ 0.39–0.63;
    /// per-wire salted ×0.75–1.25).
    pub wire_decay: f32,
    /// 0..1 — throw velocity at impact: wires ejected off the head
    /// re-land STAGGERED over ~2–4 ms (per-wire return-spring spread)
    /// — the re-landing contact burst IS the crack (the M10.5 law:
    /// rise 1.5–2.7 ms, never an impulse; same mechanism as the ring,
    /// faster timescale).
    pub wire_throw: f32,
    // M11 — Root Weight: the fundamental-dominance / un-rimshot axis.
    // Autopsy F1: real center hits put the strongest peak AT f0 with
    // partials 7–14 dB down (Halo-Feeder's produced extreme: 25.4 dB);
    // our modal weighting parked energy on modes 2–4 (−0.1 dB margin =
    // "reads as rimshot", literally an edge-hit spectrum).
    /// 0..1 → 0..25 dB attenuation of everything ≥ ~half an octave
    /// above f0 (mode-weight redistribution at trigger, energy-
    /// compensated like the transect tilt; no post-EQ). 0 = legacy.
    pub root_weight: f32,
    // STEREO v1 (M5): per-mode L/R decoherence. Both default 0 = the
    // canonical mono voice, bit-identical in both channels.
    /// 0..1 — per-mode L/R phase divergence (zero below 4·f0: the sub
    /// stays mono by construction; ramps to full over ~3 octaves above).
    /// 0..1 — per-mode L/R micro-detune (dual rotor, ±up to 8 cents at
    /// full, golden-ratio salted, same ramp as width).
    pub decohere: f32,
    /// 0..1 — how far DOWN the spectrum width/decohere reach. 1 = sub
    /// fully protected (ramp from 4·f0); 0 = full-spectrum decoherence,
    /// sub included — negative LR correlation allowed and intended
    /// ("crazy 3d lowend"). Default 0.3.
    pub stereo_floor: f32,
    // STEREO round 2 (all default-neutral):
    /// rattle mix ratio vs bank peak (formerly hardcoded 0.5) — Sam's
    /// "push the satellites out further" knob.
    pub rattle_level: f32,
    /// 0..1 — per-mode equal-power pan (golden-salted, floor-ramped):
    /// the bank spatially SPLIT across the field ("the object occupies
    /// the stereo field").
    pub mode_spread: f32,
    /// 0..1 — L/R damping asymmetry (dual-rotor): the object decays
    /// differently into each ear — orientation.
    pub damp_asym: f32,
    /// 0..1 — quadrature phase divergence applied with the INVERSE
    /// spectral ramp: acts on the LOW region (up to 90° at 1.0) — the
    /// vast-sub knob, complement to floor-0 decoherence.
    pub sub_rotate: f32,
    // M7 — exciter family
    pub exciter: Exciter,
    /// generic exciter shape: Mallet = stiffness trim (0.5 = neutral),
    /// Burst = dark..bright, Buckling = click sharpness, Raw = soften LP.
    pub ex_color: f32,
    /// generic exciter time: Mallet = contact-time scale (0.5 = 1x),
    /// Burst = length 2..80 ms, Buckling = base rate 30..900 /s,
    /// Raw = DC-kick tail 0..30 ms.
    pub ex_time: f32,
}

impl Default for EngineParams {
    fn default() -> Self {
        Self {
            arch: Arch::Membrane,
            f0: 110.0,
            velocity: 0.8,
            position: 0.4,
            listen_pos: 0.31,
            stiffness: 0.4,
            t60_base: 1.0,
            tilt: 1.2,
            n_axial: 6,
            glide_st: 0.0,
            out_tilt_db_oct: 0.0,
            cascade_amt: 0.0,
            cascade_tau: 0.05,
            cascade_split: 5.0,
            cascade_attack: 0.0,
            cascade_conserve: false,
            cascade_coherent: false,
            brace_coupling: 0.0,
            brace_choke: 0.0,
            brace_tension: 0.0,
            brace_t60: 1.0,
            brace_low_bonus: 1.0,
            sat_count: 0,
            sat_fs: [1900.0; MAX_SATS],
            sat_t60: [0.1; MAX_SATS],
            sat_seat: [0.3; MAX_SATS],
            sat_rest: [0.2; MAX_SATS],
            sat_level: [1.0; MAX_SATS],
            sat_pr: [[1.0, 0.0, 0.0, 0.0]; MAX_SATS], // single partial = legacy voice
            sat_pa: [[1.0, 0.0, 0.0, 0.0]; MAX_SATS],
            rattle_casc: 0.0,
            bounce: 0.0,
            rattle_gap: 0.5,
            gap_vel: 0.0,
            rattle_tune: 0.0,
            rattle_track: 0.0,
            walk: 0.0,
            dust_level: 0.0,
            dust_thr_db: -40.0,
            dust_follow: 1.0,
            bed_release: 0.0,
            bed_source: 0.0,
            bed_comb: 0.0,
            bed_bright: 0.5,
            cavity: 0.0,
            cavity_tune: 170.0,
            head2_tune: 7.0,
            head2_damp: 0.5,
            wires: 0.0,
            wire_tune: 1800.0,
            wire_decay: 0.45,
            wire_throw: 0.5,
            root_weight: 0.0,
            decohere: 0.0,
            stereo_floor: 0.3,
            rattle_level: 0.5,
            mode_spread: 0.0,
            damp_asym: 0.0,
            sub_rotate: 0.0,
            exciter: Exciter::Mallet,
            ex_color: 0.5,
            ex_time: 0.5,
        }
    }
}

const PHI: f32 = 0.618_033_9;

/// Coupled-form (rotation) resonator: state (u,v) rotated by ω and scaled
/// by r each sample. Frequency motion re-aims the rotation without touching
/// the state norm — passive under glide by construction (the architecture's
/// risk-#1 mitigation; direct-form II measurably explodes here).
#[derive(Clone, Copy, Default)]
struct Mode {
    freq: f32,     // current (post-tension) base frequency
    amp: f32,      // input coupling gain (strike x listen x tilt x coupling)
    t60: f32,
    cw: f32,       // cos(ω)
    sw: f32,       // sin(ω)
    r: f32,        // per-sample damping
    u: f32,
    v: f32,
    low: bool,     // below cascade split (donor) — else receiver
    inj: f32,      // cascade injection gain (receivers)
    inj_rc: f32,   // M8: rattle->cascade shock gain (receivers; own path)
    // coherent-cascade shadow state: kicked at trigger, rings freely,
    // contributes u2 * buildup * e_norm (receivers only). M8: also the
    // collision-clang rings — contact-entry shocks kick these states.
    u2: f32,
    v2: f32,
    mi: f32, // mode indices (bar: mi = partial number, ni = 1)
    ni: f32,
    // STEREO v1: right-channel rotor (decohere) + per-mode stereo geometry
    ur: f32,
    vr: f32,
    cwr: f32, // R rotation coeffs (detuned) — valid only when detuned
    swr: f32,
    eps: f32, // per-mode detune ratio offset (golden-salted, sub-protected)
    ct: f32,  // cos/sin of the width(+sub-rotate) phase tap θ_k
    st: f32,
    // STEREO round 2
    rr: f32,     // R-channel damping (damp_asym; == r when asym 0)
    asym_k: f32, // per-mode T60 asymmetry fraction (salted, ramped)
    pgl: f32,    // mode-spread pan gains (1.0 at spread 0)
    pgr: f32,
    // M10 — volume-coupling weight into the cavity: only odd×odd modes
    // displace net volume (even modes cancel), weight 1/(m·n) — the
    // physical air-coupling law for a rectangular membrane. 0 when the
    // cavity is off.
    cpl: f32,
}

pub struct Engine {
    sr: f32,
    modes: [Mode; MAX_MODES],
    n_modes: usize,
    p: EngineParams,
    // excitation pulse state
    pulse_len: usize,
    pulse_pos: usize,
    // control-rate state
    ctrl_count: usize,
    e0: f32,       // bank energy reference: RUNNING MAX of e_smooth
    e_smooth: f32, // smoothed bank energy (NL1 tracker)
    e_norm: f32,   // e_smooth / e0, clamped to [0,1]
    glide_r2: f32, // 2^(glide_st/6)
    t: f32,        // seconds since trigger
    rng: u32,
    active: bool,
    detuned: bool, // decohere > 0: R rotor path engaged
    // dust, right channel (engaged only when width/decohere > 0)
    dust_lp1r: f32,
    dust_lp2r: f32,
    // satellites
    n_sats: usize,
    sat_w: [[f32; MAX_MODES]; MAX_SATS], // seat weights per mode
    sat_z: [f32; MAX_SATS],   // normalized units (fractions of seat peak)
    sat_v: [f32; MAX_SATS],
    sat_speak: [f32; MAX_SATS], // running peak of |seat displacement| — the
                                // online calibration that replaces the lab's
                                // offline pre-pass (rest gap self-tunes, so
                                // contact recurs through the decay)
    sat_om: [f32; MAX_SATS],
    sat_ze: [f32; MAX_SATS],
    sat_kc: [f32; MAX_SATS],
    sat_gain: f32,  // mix ratio of normalized rattle vs bank peak
    sat_peak: f32,  // running peak of raw satellite radiation (normalizer)
    contacts: u32,
    entries: u32, // contact-ENTRY events, L+R (QC: pressed vs bouncing)
    // STEREO round 2: R-channel satellite bank (per-ear contact events).
    // Engaged only when the engine is stereo; otherwise the L bank feeds
    // both channels (bit-identity at defaults).
    sat_z_r: [f32; MAX_SATS],
    sat_v_r: [f32; MAX_SATS],
    sat_speak_r: [f32; MAX_SATS],
    sat_peak_r: f32,
    contacts_r: u32,
    // M8 — multi-modal satellite voices: partial rotors (coupled-form),
    // rung by contact impulses, per channel. Coefficients shared L/R.
    sat_pu: [[f32; SAT_PARTIALS]; MAX_SATS],
    sat_pv: [[f32; SAT_PARTIALS]; MAX_SATS],
    sat_pu_r: [[f32; SAT_PARTIALS]; MAX_SATS],
    sat_pv_r: [[f32; SAT_PARTIALS]; MAX_SATS],
    sat_pcw: [[f32; SAT_PARTIALS]; MAX_SATS],
    sat_psw: [[f32; SAT_PARTIALS]; MAX_SATS],
    sat_prr: [[f32; SAT_PARTIALS]; MAX_SATS], // per-partial damping
    sat_pamp: [[f32; SAT_PARTIALS]; MAX_SATS], // per-partial radiated amp
    // M8 — bounce/settle + gap laws
    sat_incontact: [bool; MAX_SATS],
    sat_incontact_r: [bool; MAX_SATS],
    sat_sdp: [f32; MAX_SATS],   // previous seat displacement (surface velocity)
    sat_sdp_r: [f32; MAX_SATS],
    sat_sdvp: [f32; MAX_SATS],  // previous surface velocity (surface accel)
    sat_sdvp_r: [f32; MAX_SATS],
    sat_carried: [bool; MAX_SATS], // bounce path: riding the surface (hops when
    sat_carried_r: [bool; MAX_SATS], // the table drops faster than gravity)
    sat_rest0: [f32; MAX_SATS],    // note-on gap (static x velocity laws)
    sat_rest_eff: [f32; MAX_SATS], // ctrl-rate: rest0 x decay-tightening
    sat_grav: f32,                 // gravity accel (normalized units), bounce path
    // M8 — rattle->cascade coupling
    rc_shock: f32,   // this-sample contact-entry shock (L+R summed, pre-mode-loop)
    // M8 — walk: drifting seats (R gets its own weights when walking)
    sat_w_r: [[f32; MAX_MODES]; MAX_SATS],
    walk_on: bool,
    // dust
    dust_env: f32,   // one-pole envelope of |y| (legacy path)
    dust_peak: f32,  // running peak of bank |y| (shared normalizer)
    dust_lp1: f32,   // crude bandpass: HP(lp1) then LP(lp2) states
    dust_lp2: f32,
    // M9 wire-bed state (engaged when any bed_* param is non-default;
    // the legacy dust path above stays bit-exact at defaults)
    bed_env: f32,       // attack/release follower
    bed_a_atk: f32,     // 1 ms attack (new(), sr-derived)
    bed_a_rel: f32,     // release coeff (trigger, from bed_release)
    bed_a_hp: f32,      // bright-derived noise band (trigger)
    bed_a_lp: f32,
    bed_src_s1: f32,    // source bandpass 150-800 Hz states (L tap)
    bed_src_s2: f32,
    a_src_hp: f32,      // 150 Hz / 800 Hz coeffs (new())
    a_src_lp: f32,
    comb_buf_l: [f32; 256], // wire-comb delay lines (max 0.6 ms @ 192k)
    comb_buf_r: [f32; 256],
    comb_pos: usize,
    comb_dly_l: usize,  // per-channel golden-ratio salted delays (new())
    comb_dly_r: usize,
    comb_lp_l: f32,     // in-loop LP (band-limit doctrine)
    comb_lp_r: f32,
    a_comb_lp: f32,     // M10: ~12 kHz (new()); loop gain still <= fb < 1
                        // at ALL frequencies (one-pole LP |H| <= 1, max at
                        // DC) — stability is independent of the cutoff
    // M10 — bed band-limit: Chebyshev-II lowpass cascade (order 12, 6
    // TDF2 sections, static per-sr coefficients — the passivity law is
    // about TIME-VARYING frequency; these never move). Stopband edge
    // 0.45·sr at −63 dB; −3 dB at ~0.367·sr (16.2 kHz @ 44.1k): the
    // top octave the M9 one-pole fleet was eating, opened. Replaces the
    // ZOH noise core + 6×9.5 kHz smoother outright.
    bed_bqc: [[f32; 5]; BED_BQ], // b0 b1 b2 a1 a2 per section
    bed_bq_l: [[f32; 2]; BED_BQ],
    bed_bq_r: [[f32; 2]; BED_BQ],
    // M8 — satellite radiation smoother (2x one-pole @10 kHz): bounce-mode
    // entry clicks are hard steps into the partial rotors; the doctrine's
    // band-limit gate demands softened edges (same lesson as buckling's
    // third filter stage)
    sat_rad_lp1: f32,
    sat_rad_lp2: f32,
    sat_rad_lp1r: f32,
    sat_rad_lp2r: f32,
    a_rad: f32,
    // sample-rate-correct coefficients (M6 pass; formerly 44.1k literals)
    a_env: f32, // dust envelope one-pole, tau 4 ms
    a_hp: f32,  // dust bandpass HP corner 1.5 kHz
    a_lp: f32,  // dust bandpass LP corner 6.5 kHz
    a_es: f32,  // NL1 e_smooth per-ctrl-block coefficient, tau ~6.9 ms
    // M7 — exciter state (fixed, no alloc)
    exc_lp1: f32,     // color filter chain (2x one-pole LP)
    exc_lp2: f32,
    exc_hp: f32,      // burst dark/bright HP state
    exc_a_lp: f32,    // color coeffs, sr-derived at trigger
    exc_a_hp: f32,
    buck_rate0: f32,  // buckling base rate (clicks/s)
    // buckling snap shaping: raised-cosine click cores (finite contact
    // width) instead of single-sample deltas — the delta edge read as
    // "digital crackle" (Sam, M8 verdict). Width rides ex_color.
    buck_plen: usize, // snap length in samples
    buck_ppos: usize, // position in the active snap (MAX = inactive)
    buck_pamp: f32,   // active snap amplitude (energy-matched to old delta)
    next_click: usize,
    clicks_left: u32, // runaway backstop (cap per trigger)
    clicks_fired: u32,
    raw_tail_len: usize,
    // M10 — Stick exciter: pulse amplitude, force-integral matched to
    // Mallet (amp·len ≈ const) so switching exciters holds level
    stick_amp: f32,
    // Stick contact radiation ("the tick"): a sub-ms contact radiates
    // audibly on its own — THE snap path, since at snare tunes the
    // modal bank tops out near 1 kHz and can't carry the crack. Feeds
    // the output directly (never the bank), band-limited through its
    // own Cheby-II cascade state (shared coefficients).
    stick_dir: f32,
    stick_bq: [[f32; 2]; BED_BQ],
    // M10 — cavity + resonant head (R1 ⇄ cavity ⇄ R2). All state fixed,
    // no alloc; engaged only when p.cavity > 0 (cav_on).
    cav_on: bool,
    n_r2: usize,
    r2_u: [f32; R2_MODES],
    r2_v: [f32; R2_MODES],
    r2_cw: [f32; R2_MODES],
    r2_sw: [f32; R2_MODES],
    r2_r: [f32; R2_MODES],
    r2_cpl: [f32; R2_MODES], // volume-coupling weights (odd·odd law)
    r2_out: [f32; R2_MODES], // listen-tap weights
    cav_u: [f32; CAV_MODES],
    cav_v: [f32; CAV_MODES],
    cav_cw: [f32; CAV_MODES],
    cav_sw: [f32; CAV_MODES],
    cav_r: [f32; CAV_MODES],
    cav_g: [f32; CAV_MODES],
    cav_kc: f32,  // (x1 − x2) volume drive into the cavity
    cav_ret: f32, // pressure reaction on the batter (mass asymmetry:
                  // the batter is the heavy head — the return path is
                  // deliberately weaker than the forward path, which
                  // bounds the R1→cav→R2→cav→R1 loop gain by GEOMETRY)
    cav_k2: f32,  // pressure force on the resonant head
    r2_rad: f32,  // resonant-head radiation into the output
    r2_x2out: f32, // this-sample R2 listen tap (bed source rides this)
    // running peak of |x2| — bed-source normalizer. Its input (R2
    // motion) is driven by exciter/cavity/batter only, NEVER by bed
    // output: the normalizer-consumer law holds.
    x2_peak: f32,
    // M11 — the wire bank (Net1). Each wire: one coupled-form ring
    // rotor (the tonal peak) + a stiff vertical DOF (tension return
    // spring, ω_z ~ 2π·150–450 Hz salted → re-landing staggered
    // 1–3 ms: the crack timing BY MECHANISM) in unilateral contact
    // with the snare-side head motion (R2 when the cavity is on,
    // batter net-volume displacement otherwise). Normalized frame
    // (running peak of head motion) — the satellite lesson: contact
    // recurs at EVERY velocity, which is why the ring survives ghost
    // hits (the M10.5 hard constraint).
    wire_on: bool,
    wire_u: [f32; N_WIRES],
    wire_v: [f32; N_WIRES],
    wire_cw: [f32; N_WIRES],
    wire_sw: [f32; N_WIRES],
    wire_r: [f32; N_WIRES],
    wire_z: [f32; N_WIRES],  // height above the head plane (normalized)
    wire_vz: [f32; N_WIRES],
    wire_omz: [f32; N_WIRES], // return-spring ω (tension, salted)
    wire_rest: [f32; N_WIRES], // per-wire seat gap (small, salted)
    wire_incontact: [bool; N_WIRES],
    wire_speak: f32, // running peak of |head motion| (normalizer; its
                     // input NEVER contains wire output — the law)
    wire_henv: f32,  // smoothed |hd_n| (~4 ms): contact-feed loudness
                     // follows the head's REAL envelope — the normalized
                     // frame keeps contacts RECURRING at every level
                     // (ghost ring ✓) but must not keep them equally
                     // LOUD forever (late-entry hiss ✗)
    wire_hdp: f32,   // previous head displacement (surface velocity)
    wire_peak: f32,  // running peak of wire radiation (mix normalizer)
    wire_react: f32, // this-sample reaction force onto the head
    wire_bq: [[f32; 2]; BED_BQ], // radiation band-limit (own Cheby-II
                                 // state, shared coefficients)
}

fn db_to_lin(db: f32) -> f32 {
    (10.0f32).powf(db / 20.0)
}

/// M10 — Chebyshev Type-II lowpass design for the bed band-limit gate:
/// order 12 (6 biquad sections), stopband edge at 0.45·sr, stopband
/// attenuation 63 dB (3 dB margin over the doctrine's 60). Flat
/// (monotone) passband, −3 dB at ~0.367·sr — a Butterworth cascade
/// cannot make this spec: 16 k → 19.8 k is 0.31 octave, and 60 dB in
/// 0.31 octave is ~194 dB/oct (order ~32); the inverse-Chebyshev zeros
/// buy it at order 12. Coefficients depend only on the 0.45 fraction,
/// so they are sample-rate-invariant after bilinear prewarp (the corner
/// scales with sr automatically). f64 design math, f32 runtime.
fn bed_bq_design() -> [[f32; 5]; BED_BQ] {
    let n = (2 * BED_BQ) as f64;
    let a_s = 63.0f64;
    let ws = (core::f64::consts::PI * 0.45).tan(); // prewarped stopband edge
    let lambda = (10.0f64).powf(a_s / 20.0);
    // eps = 1/sqrt(lambda^2 - 1); mu = asinh(1/eps)/n
    let mu = ((lambda * lambda - 1.0).sqrt()).asinh() / n;
    let sh = mu.sinh();
    let ch = mu.cosh();
    let mut out = [[0.0f32; 5]; BED_BQ];
    for (k, sec) in out.iter_mut().enumerate() {
        let theta = core::f64::consts::PI * (2.0 * k as f64 + 1.0) / (2.0 * n);
        // Chebyshev-I prototype pole; Cheby-II pole is ws/p (reciprocal)
        let pr = -sh * theta.sin();
        let pi_ = ch * theta.cos();
        let pm2 = pr * pr + pi_ * pi_;
        let cr = ws * pr / pm2;
        let ci = -ws * pi_ / pm2;
        // zeros at ±j·ws/cos(theta): analog section (s²+c)/(s²+a1·s+a0)
        let zi = ws / theta.cos();
        let c = zi * zi;
        let a1a = -2.0 * cr;
        let a0a = cr * cr + ci * ci;
        // bilinear s = (1−z⁻¹)/(1+z⁻¹) (prewarp folded into ws)
        let d0 = 1.0 + a1a + a0a;
        let b0 = (1.0 + c) / d0;
        let b1 = (2.0 * c - 2.0) / d0;
        let b2 = (1.0 + c) / d0;
        let a1 = (2.0 * a0a - 2.0) / d0;
        let a2 = (1.0 - a1a + a0a) / d0;
        // per-section unity DC gain (product then also unity at DC)
        let g = (1.0 + a1 + a2) / (b0 + b1 + b2);
        *sec = [
            (b0 * g) as f32,
            (b1 * g) as f32,
            (b2 * g) as f32,
            a1 as f32,
            a2 as f32,
        ];
    }
    out
}

impl Engine {
    pub fn new(sr: f32) -> Self {
        // M6 sample-rate pass: these were 44.1k-hardcoded literals
        // (0.9943 / 0.807 / 0.396 / 0.9); now derived, matching the old
        // values at 44.1k to 4 decimals.
        let a_rad = (-2.0 * core::f32::consts::PI * 10_000.0 / sr).exp();
        let a_env = (-1.0 / (0.004 * sr)).exp();
        let a_hp = (-2.0 * core::f32::consts::PI * 1500.0 / sr).exp();
        let a_lp = (-2.0 * core::f32::consts::PI * 6500.0 / sr).exp();
        // M9 wire-bed fixed coefficients (param-dependent ones set at trigger)
        let bed_a_atk = (-1.0 / (0.001 * sr)).exp();
        let a_src_hp = (-2.0 * core::f32::consts::PI * 150.0 / sr).exp();
        let a_src_lp = (-2.0 * core::f32::consts::PI * 800.0 / sr).exp();
        // M10: comb in-loop LP raised 8 k → 12 k (the biquad cascade now
        // owns the band-limit gate; the comb'd wires get to stay bright).
        // Loop stability proof: one-pole LP magnitude (1−a)/|1−a·e^{−jω}|
        // peaks at DC where it is exactly 1, so loop gain ≤ fb ≤ 0.88 < 1
        // at every frequency, for ANY cutoff.
        let a_comb_lp =
            (-2.0 * core::f32::consts::PI * (12_000.0f32).min(0.27 * sr) / sr).exp();
        // golden-ratio salted comb delays: 0.38 ms (L) / 0.53 ms (R)
        let comb_dly_l = ((0.00038 * sr) as usize).clamp(4, 255);
        let comb_dly_r = ((0.00053 * sr) as usize).clamp(4, 255);
        let a_es = (-(CTRL_INTERVAL as f32 / sr) / 0.006887).exp();
        Self {
            sr,
            modes: [Mode::default(); MAX_MODES],
            n_modes: 0,
            p: EngineParams::default(),
            pulse_len: 0,
            pulse_pos: usize::MAX,
            ctrl_count: 0,
            e0: 0.0,
            e_smooth: 0.0,
            e_norm: 0.0,
            glide_r2: 1.0,
            t: 0.0,
            rng: 0x9e3779b9,
            active: false,
            detuned: false,
            dust_lp1r: 0.0,
            dust_lp2r: 0.0,
            n_sats: 0,
            sat_w: [[0.0; MAX_MODES]; MAX_SATS],
            sat_z: [0.0; MAX_SATS],
            sat_v: [0.0; MAX_SATS],
            sat_speak: [0.0; MAX_SATS],
            sat_om: [0.0; MAX_SATS],
            sat_ze: [0.0; MAX_SATS],
            sat_kc: [0.0; MAX_SATS],
            sat_gain: 0.0,
            sat_peak: 0.0,
            contacts: 0,
            entries: 0,
            sat_z_r: [0.0; MAX_SATS],
            sat_v_r: [0.0; MAX_SATS],
            sat_speak_r: [0.0; MAX_SATS],
            sat_peak_r: 0.0,
            contacts_r: 0,
            sat_pu: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_pv: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_pu_r: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_pv_r: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_pcw: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_psw: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_prr: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_pamp: [[0.0; SAT_PARTIALS]; MAX_SATS],
            sat_incontact: [false; MAX_SATS],
            sat_incontact_r: [false; MAX_SATS],
            sat_sdp: [0.0; MAX_SATS],
            sat_sdp_r: [0.0; MAX_SATS],
            sat_sdvp: [0.0; MAX_SATS],
            sat_sdvp_r: [0.0; MAX_SATS],
            sat_carried: [false; MAX_SATS],
            sat_carried_r: [false; MAX_SATS],
            sat_rest0: [0.0; MAX_SATS],
            sat_rest_eff: [0.0; MAX_SATS],
            sat_grav: 0.0,
            rc_shock: 0.0,
            sat_w_r: [[0.0; MAX_MODES]; MAX_SATS],
            walk_on: false,
            dust_env: 0.0,
            dust_peak: 0.0,
            dust_lp1: 0.0,
            dust_lp2: 0.0,
            bed_env: 0.0,
            bed_a_atk,
            bed_a_rel: 0.0,
            bed_a_hp: a_hp,
            bed_a_lp: a_lp,
            bed_src_s1: 0.0,
            bed_src_s2: 0.0,
            a_src_hp,
            a_src_lp,
            comb_buf_l: [0.0; 256],
            comb_buf_r: [0.0; 256],
            comb_pos: 0,
            comb_dly_l,
            comb_dly_r,
            comb_lp_l: 0.0,
            comb_lp_r: 0.0,
            a_comb_lp,
            bed_bqc: bed_bq_design(),
            bed_bq_l: [[0.0; 2]; BED_BQ],
            bed_bq_r: [[0.0; 2]; BED_BQ],
            sat_rad_lp1: 0.0,
            sat_rad_lp2: 0.0,
            sat_rad_lp1r: 0.0,
            sat_rad_lp2r: 0.0,
            a_rad,
            a_env,
            a_hp,
            a_lp,
            a_es,
            exc_lp1: 0.0,
            exc_lp2: 0.0,
            exc_hp: 0.0,
            exc_a_lp: 0.0,
            exc_a_hp: 0.0,
            buck_rate0: 0.0,
            buck_plen: 0,
            buck_ppos: usize::MAX,
            buck_pamp: 0.0,
            next_click: 0,
            clicks_left: 0,
            clicks_fired: 0,
            raw_tail_len: 0,
            stick_amp: 0.0,
            stick_dir: 0.0,
            stick_bq: [[0.0; 2]; BED_BQ],
            cav_on: false,
            n_r2: 0,
            r2_u: [0.0; R2_MODES],
            r2_v: [0.0; R2_MODES],
            r2_cw: [0.0; R2_MODES],
            r2_sw: [0.0; R2_MODES],
            r2_r: [0.0; R2_MODES],
            r2_cpl: [0.0; R2_MODES],
            r2_out: [0.0; R2_MODES],
            cav_u: [0.0; CAV_MODES],
            cav_v: [0.0; CAV_MODES],
            cav_cw: [0.0; CAV_MODES],
            cav_sw: [0.0; CAV_MODES],
            cav_r: [0.0; CAV_MODES],
            cav_g: [0.0; CAV_MODES],
            cav_kc: 0.0,
            cav_ret: 0.0,
            cav_k2: 0.0,
            r2_rad: 0.0,
            r2_x2out: 0.0,
            x2_peak: 1e-9,
            wire_on: false,
            wire_u: [0.0; N_WIRES],
            wire_v: [0.0; N_WIRES],
            wire_cw: [0.0; N_WIRES],
            wire_sw: [0.0; N_WIRES],
            wire_r: [0.0; N_WIRES],
            wire_z: [0.0; N_WIRES],
            wire_vz: [0.0; N_WIRES],
            wire_omz: [0.0; N_WIRES],
            wire_rest: [0.0; N_WIRES],
            wire_incontact: [false; N_WIRES],
            wire_speak: 1e-9,
            wire_henv: 0.0,
            wire_hdp: 0.0,
            wire_peak: 1e-12,
            wire_react: 0.0,
            wire_bq: [[0.0; 2]; BED_BQ],
        }
    }

    /// Buckling clicks fired since trigger (QC/viz).
    pub fn clicks(&self) -> u32 {
        self.clicks_fired
    }

    /// Total contact-samples (L + R banks).
    pub fn contacts(&self) -> u32 {
        self.contacts + self.contacts_r
    }

    /// Per-channel contact-samples (stereo round 2 QC).
    pub fn contacts_lr(&self) -> (u32, u32) {
        (self.contacts, self.contacts_r)
    }

    /// Contact-ENTRY events (M8 QC: many brief entries = bouncing/chatter,
    /// few long contacts = pressed).
    pub fn entries(&self) -> u32 {
        self.entries
    }

    pub fn latency_samples(&self) -> usize {
        0
    }

    pub fn reset(&mut self) {
        for m in self.modes.iter_mut() {
            m.u = 0.0;
            m.v = 0.0;
            m.ur = 0.0;
            m.vr = 0.0;
            m.u2 = 0.0;
            m.v2 = 0.0;
        }
        self.cav_u = [0.0; CAV_MODES];
        self.cav_v = [0.0; CAV_MODES];
        self.r2_u = [0.0; R2_MODES];
        self.r2_v = [0.0; R2_MODES];
        self.wire_u = [0.0; N_WIRES];
        self.wire_v = [0.0; N_WIRES];
        self.wire_z = [0.0; N_WIRES];
        self.wire_vz = [0.0; N_WIRES];
        self.pulse_pos = usize::MAX;
        self.active = false;
    }

    fn mode_table(&mut self, p: &EngineParams) {
        // archetype ratio tables + position weights, mirroring lab/engine.py
        let mut k = 0usize;
        let nyq = self.sr * 0.45;
        let na = p.n_axial.max(1) as usize;
        let (aspect, quadratic) = match p.arch {
            Arch::Membrane => (0.94f32, false),
            Arch::Plate => (0.79, true),
            Arch::Bar => (0.0, false),
        };
        let bar_ratios = [1.0f32, 2.756, 5.404, 8.933, 13.345, 18.638];
        let px = 0.08 + 0.42 * p.position;
        let py = 0.06 + 0.38 * p.position;
        let lx = 0.08 + 0.42 * p.listen_pos;
        let ly = 0.06 + 0.38 * p.listen_pos;
        let tension = 1.0 + p.brace_tension;

        let raw = |m: f32, n: f32| -> f32 {
            if quadratic {
                m * m + (aspect * n) * (aspect * n)
            } else {
                (m * m + (aspect * n) * (aspect * n)).sqrt()
            }
        };
        let base = match p.arch {
            Arch::Bar => 1.0,
            _ => raw(1.0, 1.0),
        };

        let mut push = |freq: f32, w: f32, mi: f32, ni: f32, modes: &mut [Mode; MAX_MODES]| {
            if freq < nyq && k < MAX_MODES {
                modes[k].freq = freq;
                modes[k].amp = w;
                modes[k].mi = mi;
                modes[k].ni = ni;
                k += 1;
            }
        };

        match p.arch {
            Arch::Bar => {
                for (i, r) in bar_ratios.iter().enumerate() {
                    let m = (i + 1) as f32;
                    let w = (m * core::f32::consts::PI * p.position).cos().abs() + 0.05;
                    let wl = (m * core::f32::consts::PI * p.listen_pos).cos().abs() + 0.05;
                    push(p.f0 * r * tension, w * wl, m, 1.0, &mut self.modes);
                }
            }
            _ => {
                for mi in 1..=na {
                    for ni in 1..=na {
                        let (m, n) = (mi as f32, ni as f32);
                        let freq = p.f0 * raw(m, n) / base * tension;
                        let ws = ((m * core::f32::consts::PI * px).sin()
                            * (n * core::f32::consts::PI * py).sin())
                        .abs()
                            + 0.01;
                        let wl = ((m * core::f32::consts::PI * lx).sin()
                            * (n * core::f32::consts::PI * ly).sin())
                        .abs()
                            + 0.01;
                        push(freq, ws * wl, m, n, &mut self.modes);
                    }
                }
            }
        }
        self.n_modes = k;
    }

    /// Strike. Computes the mode table and arms the force pulse.
    pub fn trigger(&mut self, p: &EngineParams) {
        self.p = *p;
        self.mode_table(p);
        let fmin = self
            .modes[..self.n_modes]
            .iter()
            .map(|m| m.freq)
            .fold(f32::MAX, f32::min);
        let split_hz = p.cascade_split * p.f0;
        let coupling = 1.0 - 0.55 * p.brace_coupling;
        // transect gain lane (v0: tilt) with UNITY-ENERGY compensation —
        // tilt reshapes the spectrum, it must not change level (M4 fix
        // round: uncompensated positive tilt clipped immediately)
        if p.out_tilt_db_oct != 0.0 {
            let mut e_before = 0.0f32;
            let mut e_after = 0.0f32;
            for m in self.modes[..self.n_modes].iter_mut() {
                e_before += m.amp * m.amp;
                m.amp *= (m.freq / fmin).powf(p.out_tilt_db_oct / 6.02);
                e_after += m.amp * m.amp;
            }
            if e_after > 0.0 {
                let comp = (e_before / e_after).sqrt();
                for m in self.modes[..self.n_modes].iter_mut() {
                    m.amp *= comp;
                }
            }
        }
        // M11 — Root Weight: fundamental-dominance redistribution (the
        // un-rimshot axis; autopsy F1). Attenuation ramps from 0 at the
        // lowest mode to the full −25·rw dB by HALF an octave above it
        // (a membrane's mode 2 sits at 1.58·f0 → it takes the full
        // margin), flat beyond. Physically: strike/listen focus moving
        // toward dead center, extended past the physical range into the
        // Halo-Feeder produced extreme (25.4 dB measured). Same
        // unity-energy compensation as the transect tilt; 0 = legacy
        // bit-exact.
        if p.root_weight > 0.0 {
            let atten_db = 25.0 * p.root_weight.clamp(0.0, 1.0);
            let mut e_before = 0.0f32;
            let mut e_after = 0.0f32;
            for m in self.modes[..self.n_modes].iter_mut() {
                e_before += m.amp * m.amp;
                let s = (2.0 * (m.freq / fmin).max(1.0).log2()).min(1.0);
                m.amp *= (10.0f32).powf(-atten_db * s / 20.0);
                e_after += m.amp * m.amp;
            }
            if e_after > 0.0 {
                let comp = (e_before / e_after).sqrt();
                for m in self.modes[..self.n_modes].iter_mut() {
                    m.amp *= comp;
                }
            }
        }
        let mut low_amp_sum = 0.0f32;
        let mut low_count = 0usize;
        for m in self.modes[..self.n_modes].iter_mut() {
            m.amp *= coupling;
            // damping law + bracing T60 terms
            let mut t60 = p.t60_base / (1.0 + (m.freq / 900.0).powf(p.tilt));
            t60 *= p.brace_t60;
            m.low = m.freq < split_hz;
            if m.freq < 4.0 * p.f0 {
                t60 *= 1.0 + 0.9 * p.brace_low_bonus;
            }
            m.t60 = t60.max(1e-3);
            if m.low {
                low_amp_sum += m.amp;
                low_count += 1;
            }
            m.u = 0.0;
            m.v = 0.0;
        }
        // cascade injection gains for receivers
        let med_low = if low_count > 0 {
            low_amp_sum / low_count as f32
        } else {
            0.0
        };
        let inj = p.cascade_amt * p.velocity * p.velocity * med_low * 0.6;
        // M8: rattle->cascade shock gain — its OWN path (velocity enters via
        // contact strength; kicks are scaled ONLINE by the bank running peak
        // in the process loop — analytic units cannot reach ring-state scale,
        // the M3.2 normalization lesson)
        let inj_rc = p.rattle_casc.clamp(0.0, 1.0) * 0.5;
        for (k, m) in self.modes[..self.n_modes].iter_mut().enumerate() {
            let ratio_w = (m.freq / p.f0).powf(-0.3);
            m.inj = if m.low { 0.0 } else { inj * ratio_w };
            m.inj_rc = if m.low { 0.0 } else { inj_rc * ratio_w };
            // coherent cascade: arm each shadow ring with a PER-MODE PHASE
            // (golden-ratio salt). Quadrature init alone was insufficient —
            // ~100 phase-ALIGNED rising sines sum to a coherent LF ramp,
            // the DC thump Sam heard twice. Scattered phases make the onset
            // sum incoherent (~sqrt(N) instead of N); the buildup x energy
            // gate covers each ring's own small nonzero start.
            if p.cascade_coherent {
                let a = m.inj * 40.0;
                let phase = 2.0 * core::f32::consts::PI * ((k as f32 * PHI) % 1.0);
                m.u2 = a * phase.sin();
                m.v2 = a * phase.cos();
            } else {
                m.u2 = 0.0;
                m.v2 = 0.0;
            }
            // STEREO geometry: ramp over ~3 octaves above 4·f0, with the
            // protection FLOOR as a knob — floor 0 lets width/decohere reach
            // the sub at full strength (negative correlation is a feature)
            let raw = ((m.freq / (4.0 * p.f0)).max(1e-6).log2() / 3.0).clamp(0.0, 1.0);
            let ramp = raw.max(1.0 - p.stereo_floor.clamp(0.0, 1.0));
            // width acts via the ramp; SUB ROTATE acts via the INVERSE
            // spectral weight (the low region), up to 90° quadrature — the
            // vast-sub knob
            let theta = 0.0
                + core::f32::consts::FRAC_PI_2 * p.sub_rotate.clamp(0.0, 1.0) * (1.0 - raw);
            m.ct = theta.cos();
            m.st = theta.sin();
            let salt = 2.0 * ((k as f32 * PHI) % 1.0) - 1.0;
            // 8 cents at full decohere: ratio offset 2^(8/1200)-1 = 0.00463
            m.eps = 0.00463 * p.decohere.clamp(0.0, 1.0) * salt * ramp;
            // damp asym: ±25% T60 divergence at full, salted + ramped
            m.asym_k = 0.25 * p.damp_asym.clamp(0.0, 1.0) * salt * ramp;
            // mode spread: equal-power pan, gl²+gr² = 2 (unity at spread 0;
            // exact 1.0 at pan 0 to preserve default bit-identity)
            let pan = (p.mode_spread.clamp(0.0, 1.0) * salt * ramp).clamp(-1.0, 1.0);
            if pan == 0.0 {
                m.pgl = 1.0;
                m.pgr = 1.0;
            } else {
                let phi_p = core::f32::consts::FRAC_PI_4 * (1.0 + pan);
                m.pgl = core::f32::consts::SQRT_2 * phi_p.cos();
                m.pgr = core::f32::consts::SQRT_2 * phi_p.sin();
            }
            m.ur = 0.0;
            m.vr = 0.0;
            // M10 — cavity volume-coupling weight: only odd×odd modes
            // displace net volume (even halves cancel); weight 1/(m·n).
            let (mi_o, ni_o) = (m.mi as u32 % 2 == 1, m.ni as u32 % 2 == 1);
            m.cpl = if mi_o && ni_o {
                1.0 / (m.mi * m.ni).max(1.0)
            } else {
                0.0
            };
        }
        self.detuned = p.decohere > 0.0 || p.damp_asym > 0.0;
        self.update_coeffs(1.0);

        // M7 exciter arming. Mallet at ex_color/ex_time = 0.5 is EXACTLY the
        // M3 law (color adds 0.25*(c-0.5) to stiffness = +0.0; time scales
        // tau by 4^(0.5-t) = *1.0) — default bit-identity is load-bearing.
        self.exc_lp1 = 0.0;
        self.exc_lp2 = 0.0;
        self.exc_hp = 0.0;
        self.clicks_fired = 0;
        let sr = self.sr;
        let color = p.ex_color.clamp(0.0, 1.0);
        let ext = p.ex_time.clamp(0.0, 1.0);
        match p.exciter {
            Exciter::Mallet => {
                let stiff = (p.stiffness + 0.30 * p.brace_coupling + 0.25 * (color - 0.5))
                    .clamp(0.0, 1.0);
                let tau = 0.004 * (1.0 - 0.75 * stiff) / (0.35 + 0.65 * p.velocity)
                    * (4.0f32).powf(0.5 - ext);
                self.pulse_len = ((sr * tau) as usize).max(8);
                self.pulse_pos = 0;
                self.clicks_left = 0;
            }
            Exciter::Burst => {
                let ms = 2.0 * (40.0f32).powf(ext); // 2..80 ms, log
                self.pulse_len = ((sr * ms / 1000.0) as usize).max(16);
                self.pulse_pos = 0;
                self.clicks_left = 0;
                let lp_cut = (2000.0 * (4.5f32).powf(color)).min(0.4 * sr);
                let hp_cut = 200.0 * (5.0f32).powf(color);
                self.exc_a_lp = (-2.0 * core::f32::consts::PI * lp_cut / sr).exp();
                self.exc_a_hp = (-2.0 * core::f32::consts::PI * hp_cut / sr).exp();
            }
            Exciter::Buckling => {
                // no pulse: the click train IS the excitation
                self.pulse_len = 0;
                self.pulse_pos = usize::MAX;
                self.buck_rate0 = 30.0 * (30.0f32).powf(ext); // 30..900 /s
                // snap width: 2.0 ms (soft crumple) -> 0.25 ms (crisp snap)
                // across ex_color; the LP cutoff (below) rides color too, so
                // both sharpness mechanisms agree
                let snap_s = 0.002 * (0.125f32).powf(color);
                self.buck_plen = ((sr * snap_s) as usize).max(4);
                self.buck_ppos = usize::MAX;
                self.buck_pamp = 0.0;
                self.next_click = 0;
                self.clicks_left = 600; // runaway backstop, never musical
                let lp_cut = (1500.0 * (8.0f32).powf(color)).min(0.4 * sr);
                self.exc_a_lp = (-2.0 * core::f32::consts::PI * lp_cut / sr).exp();
            }
            Exciter::Raw => {
                self.raw_tail_len = (ext * 0.030 * sr) as usize; // 0..30 ms DC-kick
                self.pulse_len = self.raw_tail_len.max(1);
                self.pulse_pos = 0;
                self.clicks_left = 0;
                let lp_cut = (500.0 * (40.0f32).powf(color)).min(0.45 * sr);
                self.exc_a_lp = (-2.0 * core::f32::consts::PI * lp_cut / sr).exp();
            }
            Exciter::Stick => {
                // light stiff stick: raised-cosine Hertzian contact,
                // 0.8 ms (shoulder, color 0) → 0.08 ms (tip, color 1),
                // × the family 4^(0.5−t) time scale, × v^−0.2 (Hertz:
                // harder hits stiffen the contact slightly). Floor 4
                // samples: band-limited by construction (a 4-sample
                // raised cosine is already ~−32 dB at 0.45·sr, and the
                // spectral peak of any full render sits at the body
                // resonance far above the pulse's top edge — measured
                // at the QC gate like every exciter).
                let tau = 0.0008
                    * (0.1f32).powf(color)
                    * (4.0f32).powf(0.5 - ext)
                    * (p.velocity.max(0.05) / 0.8).powf(-0.2);
                self.pulse_len = ((sr * tau) as usize).max(4);
                self.pulse_pos = 0;
                self.clicks_left = 0;
                // force-integral loudness match vs Mallet (the buckling
                // calibration law: bank peak rides ∫F dt for modes with
                // period ≫ contact time — amp·len ≈ const). n_mallet is
                // the pulse Mallet would arm on this same patch at
                // neutral color/time; ratio capped 40 (tip extremes).
                let stiff_eq =
                    (p.stiffness + 0.30 * p.brace_coupling).clamp(0.0, 1.0);
                let n_mallet = (sr * 0.004 * (1.0 - 0.75 * stiff_eq)
                    / (0.35 + 0.65 * p.velocity))
                    .max(8.0);
                // 0.28: empirical trim on top of the integral law — short
                // pulses drive mid modes more efficiently than the law
                // assumes (the mallet's own spectrum already rolls off
                // there). Calibrated against full-render peaks at color
                // 0.3/0.5/0.9: within ±0.7 dB of Mallet at all three.
                self.stick_amp =
                    p.velocity * (n_mallet / self.pulse_len as f32).min(40.0) * 0.28;
                // contact radiation gain (the tick/crack); rides color —
                // tip contacts click harder than shoulder contacts
                self.stick_dir = 1.6 + 2.8 * color;
                self.stick_bq = [[0.0; 2]; BED_BQ];
            }
        }
        self.glide_r2 = (2.0f32).powf(p.glide_st / 6.0);
        self.e0 = 0.0;
        self.e_smooth = 0.0;
        self.e_norm = 0.0;
        self.ctrl_count = 0;
        self.t = 0.0;

        // NL2 satellites: seat weights + analytic seat-displacement estimate
        // (replaces the lab's offline calibration pre-pass; the estimate is
        // the resonant peak response of each mode to the force pulse, summed
        // at the seat — constant tuned against lab contact counts).
        self.n_sats = (p.sat_count as usize).min(MAX_SATS);
        self.contacts = 0;
        self.entries = 0;
        let nm = self.n_modes;
        // M8 — tune/track: one frequency scale for the whole rattle family
        // (partials AND contact dynamics — the hardware's timescale follows
        // its size). track ref = 36 Hz so defaults keep the f36-kick pitch.
        let tr = p.rattle_track.clamp(0.0, 1.0);
        let fscale = (2.0f32).powf(p.rattle_tune.clamp(-2.0, 2.0))
            * (p.f0 / 36.0).max(1e-3).powf(tr);
        // M8 — gap laws at note-on: static base (0.5 = preset-neutral) x
        // velocity depth (gap_vel 0 = static; 1 = fully velocity-affected,
        // harder = thrown wider). Decay-tightening rides e_norm at ctrl rate.
        let gv = p.gap_vel.clamp(0.0, 1.0);
        let gap_note = (2.0 * p.rattle_gap.clamp(0.0, 1.0))
            * ((1.0 - gv) + gv * 2.0 * p.velocity);
        let b = p.bounce.clamp(0.0, 1.0);
        // gravity for the bounce path (normalized units): tuned so full-
        // velocity throws land first flights ~40-90 ms — musical settle.
        self.sat_grav = 60.0 + 240.0 * b;
        for j in 0..self.n_sats {
            let seat = p.sat_seat[j];
            let sx = 0.08 + 0.42 * seat;
            let sy = 0.06 + 0.38 * seat;
            let mut est = 0.0f32;
            for k in 0..nm {
                let m = &self.modes[k];
                let w = match p.arch {
                    Arch::Bar => (m.mi * core::f32::consts::PI * seat).cos().abs() + 0.05,
                    _ => ((m.mi * core::f32::consts::PI * sx).sin()
                        * (m.ni * core::f32::consts::PI * sy).sin())
                    .abs()
                        + 0.01,
                };
                self.sat_w[j][k] = w;
                self.sat_w_r[j][k] = w; // walk diverges these at ctrl rate
                // peak response estimate: amp x pulse-length x resonant gain
                est += w * self.modes[k].amp;
            }
            // analytic estimate now only SEEDS the online tracker (low, so
            // the true peak takes over within the first oscillation)
            let est_peak = est * p.velocity * self.pulse_len.max(64) as f32 * 0.25;
            self.sat_speak[j] = (est_peak * 0.3).max(1e-9);
            // gap floor 0.05: below ~5% of seat peak the contact duty cycle
            // approaches 100% and the one-sample-delayed unilateral reaction
            // becomes a parametric PUMP that keeps the bank alive forever
            // (the M8 no-drone diagnosis) — the floor keeps that regime
            // unreachable while every preset gap stays untouched
            let rest0 = (p.sat_rest[j] * gap_note).max(0.05);
            self.sat_rest0[j] = rest0;
            self.sat_rest_eff[j] = rest0;
            self.sat_z[j] = rest0;
            // M8 — velocity throw: a hard hit scatters the hardware upward
            // immediately (bounce-scaled); the settle brings it home.
            self.sat_v[j] = b * p.velocity * 14.0 * rest0.max(0.05);
            // symplectic-Euler stability: ω·dt < 2 — clamp to 0.3·2π·sr
            // (fuzzed Tune x Track x f0 extremes can otherwise blow past it
            // and NaN the contact integrator: the M8 validator-hang lesson)
            self.sat_om[j] = (2.0 * core::f32::consts::PI * p.sat_fs[j] * fscale)
                .clamp(20.0, 1.885 * self.sr);
            self.sat_ze[j] = 6.9078 / (p.sat_t60[j].max(1e-3) * self.sat_om[j]);
            // contact kick gain in NORMALIZED units: a full-depth contact
            // (pen ~ rest) rings the satellite at ~1/3 of its gap scale.
            // Unclamped — pen is geometrically bounded (sd_n <= 1), so force
            // is bounded and potential-derived: no clamp, no limit cycle.
            self.sat_kc[j] = 0.05 * self.sat_om[j] * self.sat_om[j];
            self.sat_incontact[j] = false;
            self.sat_incontact_r[j] = false;
            self.sat_sdp[j] = 0.0;
            self.sat_sdp_r[j] = 0.0;
            self.sat_sdvp[j] = 0.0;
            self.sat_sdvp_r[j] = 0.0;
            self.sat_carried[j] = false;
            self.sat_carried_r[j] = false;
            // M8 — multi-modal voice: partial rotors (coupled-form), rung by
            // contact impulses; decay ratio^-0.7 (higher partials die faster)
            for q in 0..SAT_PARTIALS {
                let ratio = p.sat_pr[j][q];
                if ratio > 0.0 {
                    let f = (p.sat_fs[j] * ratio * fscale).min(self.sr * 0.45);
                    let w = 2.0 * core::f32::consts::PI * f / self.sr;
                    self.sat_pcw[j][q] = w.cos();
                    self.sat_psw[j][q] = w.sin();
                    let t60p = p.sat_t60[j].max(1e-3) * ratio.powf(-0.7);
                    self.sat_prr[j][q] = (-6.9078 / (t60p * self.sr)).exp();
                    self.sat_pamp[j][q] = p.sat_pa[j][q];
                } else {
                    self.sat_pamp[j][q] = 0.0;
                    self.sat_pcw[j][q] = 0.0;
                    self.sat_psw[j][q] = 0.0;
                    self.sat_prr[j][q] = 0.0;
                }
                self.sat_pu[j][q] = 0.0;
                self.sat_pv[j][q] = 0.0;
                self.sat_pu_r[j][q] = 0.0;
                self.sat_pv_r[j][q] = 0.0;
            }
        }
        self.walk_on = p.walk > 0.0 && self.n_sats > 0;
        self.rc_shock = 0.0;
        // rattle mix ratio: normalized-online radiation vs bank running peak
        // (the RT equivalent of the lab's offline normalize-then-mix-at-0.5);
        // exposed as Rattle Level (stereo round 2)
        self.sat_gain = p.rattle_level.clamp(0.0, 1.0);
        self.sat_peak = 1e-12;
        self.sat_peak_r = 1e-12;
        self.contacts_r = 0;
        for j in 0..self.n_sats {
            self.sat_z_r[j] = self.sat_z[j];
            self.sat_v_r[j] = 0.0;
            self.sat_speak_r[j] = self.sat_speak[j];
        }
        self.dust_env = 0.0;
        self.dust_peak = 1e-9;
        self.dust_lp1 = 0.0;
        self.dust_lp2 = 0.0;
        self.dust_lp1r = 0.0;
        self.dust_lp2r = 0.0;
        // M9 wire-bed: param-dependent coefficients + state reset
        {
            let sr = self.sr;
            // M11 recalibration (the M10.5 diagnosis): the knob was in
            // exponential time-constant units τ, but the ear hears
            // T60 ≈ 6.91·τ/follow — the M10 remap fixed the SPAN, not
            // the UNITS. Now the knob IS perceived T60: log-mapped
            // 0.15–1.5 s, with dust_follow folded into the coefficient
            // so the knob means the same seconds at any follow setting.
            // (SD-measured dry-snare zone 0.39–0.63 s = mid-throw;
            // inversion/gated-reverb at the top, where it belongs.)
            let t60_knob = 0.15 * (10.0f32).powf(p.bed_release.clamp(0.0, 1.0));
            let rel_t = t60_knob * p.dust_follow.clamp(0.5, 3.0) / 6.9078;
            self.bed_a_rel = (-1.0 / (rel_t * sr)).exp();
            // brightness: 0.5 = the exact legacy band (bit-identity)
            let b = p.bed_bright.clamp(0.0, 1.0);
            if (b - 0.5).abs() < 1e-6 {
                self.bed_a_hp = self.a_hp;
                self.bed_a_lp = self.a_lp;
            } else {
                let hp_cut = 1500.0 * (2.2f32).powf(2.0 * b - 1.0);
                // M10: dark half keeps the M9 law; the top half steepens
                // to reach the biquad-opened octave (b=1 → 15.6 kHz)
                let lp_fac: f32 = if b <= 0.5 { 1.55 } else { 2.4 };
                let lp_cut =
                    (6500.0 * lp_fac.powf(2.0 * b - 1.0)).min(0.42 * sr);
                self.bed_a_hp = (-2.0 * core::f32::consts::PI * hp_cut / sr).exp();
                self.bed_a_lp = (-2.0 * core::f32::consts::PI * lp_cut / sr).exp();
            }
            self.bed_env = 0.0;
            self.bed_src_s1 = 0.0;
            self.bed_src_s2 = 0.0;
            self.comb_buf_l = [0.0; 256];
            self.comb_buf_r = [0.0; 256];
            self.comb_pos = 0;
            self.comb_lp_l = 0.0;
            self.comb_lp_r = 0.0;
            self.bed_bq_l = [[0.0; 2]; BED_BQ];
            self.bed_bq_r = [[0.0; 2]; BED_BQ];
        }
        // M10 — cavity + resonant head arming (R1 ⇄ cavity ⇄ R2).
        // cavity = 0 → topology fully off, zero per-sample cost, output
        // bit-exact vs pre-M10.
        self.cav_on = p.cavity > 0.0;
        if self.cav_on {
            let sr = self.sr;
            let cv = p.cavity.clamp(0.0, 1.0);
            // cavity air: Helmholtz-ish fundamental + inharmonic pipe
            // partials of the shell (slightly detuned from integer so
            // the hollow doesn't read as a pitched tube)
            let cav_ratios = [1.0f32, 2.02, 2.94, 3.83];
            let cav_t60 = [0.30f32, 0.14, 0.09, 0.06];
            let cav_gain = [1.0f32, 0.55, 0.38, 0.26];
            let fc0 = p.cavity_tune.clamp(30.0, 1200.0);
            for j in 0..CAV_MODES {
                let f = (fc0 * cav_ratios[j]).min(0.45 * sr);
                let w = 2.0 * core::f32::consts::PI * f / sr;
                self.cav_cw[j] = w.cos();
                self.cav_sw[j] = w.sin();
                self.cav_r[j] = (-6.9078 / (cav_t60[j] * sr)).exp();
                self.cav_g[j] = cav_gain[j];
                self.cav_u[j] = 0.0;
                self.cav_v[j] = 0.0;
            }
            // resonant head R2: a light 3×4 membrane at f0·2^(st/12),
            // damping knob spans ringing (~0.56 s lows) → choked (60 ms)
            let f02 = (p.f0 * (2.0f32).powf(p.head2_tune.clamp(-24.0, 24.0) / 12.0))
                .clamp(20.0, 2000.0);
            let damp = p.head2_damp.clamp(0.0, 1.0);
            let aspect = 0.94f32;
            let base2 = (1.0 + aspect * aspect).sqrt();
            let mut q = 0usize;
            for mi in 1..=3u32 {
                for ni in 1..=4u32 {
                    if q >= R2_MODES {
                        break;
                    }
                    let (mf, nf) = (mi as f32, ni as f32);
                    let fr = f02
                        * (mf * mf + (aspect * nf) * (aspect * nf)).sqrt()
                        / base2;
                    if fr >= 0.45 * sr {
                        continue;
                    }
                    let w = 2.0 * core::f32::consts::PI * fr / sr;
                    self.r2_cw[q] = w.cos();
                    self.r2_sw[q] = w.sin();
                    let t60_2 = ((0.06 + 0.5 * (1.0 - damp))
                        / (1.0 + (fr / 900.0).powf(1.3)))
                    .max(1e-3);
                    self.r2_r[q] = (-6.9078 / (t60_2 * sr)).exp();
                    // volume coupling: odd×odd only (same law as R1)
                    self.r2_cpl[q] = if mi % 2 == 1 && ni % 2 == 1 {
                        1.0 / (mf * nf)
                    } else {
                        0.0
                    };
                    // listen tap: fixed off-center point (0.35/0.29)
                    self.r2_out[q] = ((mf * core::f32::consts::PI * 0.35).sin()
                        * (nf * core::f32::consts::PI * 0.29).sin())
                    .abs()
                        + 0.01;
                    self.r2_u[q] = 0.0;
                    self.r2_v[q] = 0.0;
                    q += 1;
                }
            }
            self.n_r2 = q;
            // Coupling gains — the M10 margin law, learned the loud way
            // (first constants blew up to 1.8e38): a resonator driven
            // per-sample accumulates with gain 1/(1−r) at resonance
            // (~10³ for these T60s), so raw displacement→force coupling
            // multiplies THOUSANDS into every loop. Every exchange is
            // therefore normalized by the RECEIVER's per-sample
            // dissipation (1−r) — the resonant accumulation cancels
            // exactly, and the worst-case (frequency-coincident) loop
            // gain collapses to the product of the dimensionless
            // constants below: R2⇄cav 0.6·0.6 = 0.36, R1⇄cav
            // 0.6·0.12·cv² ≤ 0.072. Bounded by geometry (mass
            // asymmetry: the batter is the heavy head), no clamps.
            // Side effect, also physical: a longer-T60 cavity rings
            // LONGER, not louder.
            self.cav_kc = 0.6 * cv;
            self.cav_ret = 0.12 * cv;
            self.cav_k2 = 0.6;
            // radiation sits OUTSIDE every loop (pure output mix), so
            // it may be large without any stability cost: it undoes the
            // two (1−r) normalizations the forward chain paid.
            // Calibrated: R2 radiation ≈ −10 dB vs the body on the
            // snare recipe at cavity 1.0.
            self.r2_rad = 4000.0;
            self.r2_x2out = 0.0;
            self.x2_peak = 1e-9;
        }
        // M11 — wire-bank arming (Net1). Fixed-per-patch placement: the
        // SD data shows every real snare carries a stable per-instrument
        // mode fingerprint (Aluminum 743/1184, Coliseum 592/937 Hz, in
        // every hit) — so wire frequencies are a deterministic golden-
        // salted spread, ±1.75 octaves around Wire Tune, identical on
        // every trigger of a patch. wires = 0 → fully off, bit-exact.
        self.wire_on = p.wires > 0.0;
        if self.wire_on {
            let sr = self.sr;
            let fc = p.wire_tune.clamp(300.0, 6000.0);
            let dec = p.wire_decay.clamp(0.05, 2.0);
            let throw = p.wire_throw.clamp(0.0, 1.0);
            for k in 0..N_WIRES {
                let s1 = ((k as f32 + 1.0) * PHI) % 1.0;
                let s2 = ((k as f32 + 1.0) * PHI * 7.0) % 1.0;
                let s3 = ((k as f32 + 1.0) * PHI * 13.0) % 1.0;
                let s4 = ((k as f32 + 1.0) * PHI * 29.0) % 1.0;
                let f = (fc * (2.0f32).powf(3.5 * (s1 - 0.5)))
                    .clamp(250.0, (8000.0f32).min(0.40 * sr));
                let w = 2.0 * core::f32::consts::PI * f / sr;
                self.wire_cw[k] = w.cos();
                self.wire_sw[k] = w.sin();
                // per-wire decay salt ×0.75–1.25 around Wire Decay —
                // the SD target band is a FLAT tail (0.39–0.63 s)
                let t60k = dec * (0.75 + 0.5 * s2);
                self.wire_r[k] = (-6.9078 / (t60k * sr)).exp();
                // tension return spring, 150–450 Hz salted: a thrown
                // wire re-lands in ~1.1–3.3 ms — the crack's 2–4 ms
                // stagger BY MECHANISM (M10.5 target ②: never an
                // impulse). ω·dt ≤ 0.065 ≪ 2: symplectic-stable.
                self.wire_omz[k] = 2.0 * core::f32::consts::PI * (150.0 + 300.0 * s3);
                self.wire_rest[k] = 0.03 + 0.03 * s1;
                self.wire_z[k] = self.wire_rest[k];
                // THE THROW: ejected at note-on ∝ velocity × Wire
                // Throw; the return spring brings each wire home at
                // its own time. Ghost hits barely throw — their ring
                // comes from recurring gentle contact instead (the
                // ghost-must-buzz constraint).
                self.wire_vz[k] =
                    throw * p.velocity * self.wire_omz[k] * 0.9 * (0.5 + s4);
                self.wire_u[k] = 0.0;
                self.wire_v[k] = 0.0;
                self.wire_incontact[k] = false;
            }
            self.wire_speak = 1e-9;
            self.wire_henv = 0.0;
            self.wire_hdp = 0.0;
            self.wire_peak = 1e-12;
            self.wire_react = 0.0;
            self.wire_bq = [[0.0; 2]; BED_BQ];
        }
        self.active = true;
    }

    fn update_coeffs(&mut self, freq_mult: f32) {
        let sr = self.sr;
        let detuned = self.detuned;
        for m in self.modes[..self.n_modes].iter_mut() {
            m.r = (-6.9078 / (m.t60 * (1.0 + m.asym_k) * sr)).exp();
            let base = (m.freq * freq_mult).min(sr * 0.49);
            if detuned {
                // L rotor at ω(1+ε), R rotor at ω(1−ε); glide applies to both.
                // R damping diverges with damp_asym (orientation).
                m.rr = (-6.9078 / (m.t60 * (1.0 - m.asym_k) * sr)).exp();
                let wl = 2.0 * core::f32::consts::PI * (base * (1.0 + m.eps)).min(sr * 0.49) / sr;
                m.cw = wl.cos();
                m.sw = wl.sin();
                let wr = 2.0 * core::f32::consts::PI * (base * (1.0 - m.eps)).min(sr * 0.49) / sr;
                m.cwr = wr.cos();
                m.swr = wr.sin();
            } else {
                m.rr = m.r;
                let w = 2.0 * core::f32::consts::PI * base / sr;
                m.cw = w.cos();
                m.sw = w.sin();
            }
        }
    }

    fn white(&mut self) -> f32 {
        // xorshift32 → [-1, 1]
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Render one block, stereo. With width = decohere = 0 both channels
    /// carry the canonical mono voice bit-identically (parity preserved).
    /// Returns true while the voice is audibly active.
    pub fn process(&mut self, out_l: &mut [f32], out_r: &mut [f32]) -> bool {
        let n = out_l.len().min(out_r.len());
        if !self.active {
            for o in out_l[..n].iter_mut() {
                *o = 0.0;
            }
            for o in out_r[..n].iter_mut() {
                *o = 0.0;
            }
            return false;
        }
        let p = self.p;
        let dt = 1.0 / self.sr;
        let mut peak = 0.0f32;
        let stereo = false
            || p.decohere > 0.0
            || p.mode_spread > 0.0
            || p.damp_asym > 0.0
            || p.sub_rotate > 0.0;

        for i in 0..n {
            // control-rate: NL1 glide + cascade envelopes
            if self.ctrl_count == 0 {
                let mut e = 0.0f32;
                for m in self.modes[..self.n_modes].iter() {
                    if m.low {
                        e += m.u * m.u + m.v * m.v;
                    }
                }
                self.e_smooth = self.a_es * self.e_smooth + (1.0 - self.a_es) * e;
                if self.e_smooth > self.e0 {
                    self.e0 = self.e_smooth; // reference = running max
                }
                self.e_norm = if self.e0 > 0.0 {
                    (self.e_smooth / self.e0).min(1.0)
                } else {
                    0.0
                };
                if p.glide_st > 0.0 && self.e0 > 0.0 {
                    let mult = (1.0
                        + (self.glide_r2 - 1.0) * p.velocity * p.velocity * self.e_norm)
                        .sqrt();
                    self.update_coeffs(mult);
                }
                // M8 — gap decay-tightening (the snare-sizzle bloom: density
                // rises as the body calms, then cuts off), bounce-scaled
                let bt = p.bounce.clamp(0.0, 1.0);
                for j in 0..self.n_sats {
                    self.sat_rest_eff[j] =
                        (self.sat_rest0[j] * (1.0 - 0.6 * bt * (1.0 - self.e_norm))).max(0.05);
                }
                // M8 — walk: seats migrate through the decay; per-channel
                // phase salt => L/R weight sets diverge (free decorrelation)
                if self.walk_on {
                    let wk = p.walk.clamp(0.0, 1.0);
                    let nm_w = self.n_modes;
                    for j in 0..self.n_sats {
                        let fw = 0.9 + 0.9 * ((j as f32 * PHI) % 1.0);
                        let ph0 = 2.0 * core::f32::consts::PI * ((j as f32 * PHI * 3.0) % 1.0);
                        let drift = 0.30 * wk;
                        let om_w = 2.0 * core::f32::consts::PI * fw;
                        let seat_l = (p.sat_seat[j]
                            + drift * (om_w * self.t + ph0).sin())
                        .clamp(0.0, 1.0);
                        let seat_r = (p.sat_seat[j]
                            + drift * (om_w * self.t + ph0 + 2.4).sin())
                        .clamp(0.0, 1.0);
                        for (seat, w_arr) in [(seat_l, 0usize), (seat_r, 1usize)] {
                            let sx = 0.08 + 0.42 * seat;
                            let sy = 0.06 + 0.38 * seat;
                            for k in 0..nm_w {
                                let m = &self.modes[k];
                                let w = match p.arch {
                                    Arch::Bar => {
                                        (m.mi * core::f32::consts::PI * seat).cos().abs() + 0.05
                                    }
                                    _ => ((m.mi * core::f32::consts::PI * sx).sin()
                                        * (m.ni * core::f32::consts::PI * sy).sin())
                                    .abs()
                                        + 0.01,
                                };
                                if w_arr == 0 {
                                    self.sat_w[j][k] = w;
                                } else {
                                    self.sat_w_r[j][k] = w;
                                }
                            }
                        }
                    }
                }
                self.ctrl_count = CTRL_INTERVAL;
            }
            self.ctrl_count -= 1;

            // excitation force (M7: exciter family; Mallet path is the M3
            // original, bit-identical at defaults)
            let f_in = match p.exciter {
                Exciter::Mallet => {
                    if self.pulse_pos < self.pulse_len {
                        let ph = core::f32::consts::PI * 2.0 * self.pulse_pos as f32
                            / self.pulse_len as f32;
                        self.pulse_pos += 1;
                        p.velocity * (0.5 - 0.5 * ph.cos())
                    } else {
                        0.0
                    }
                }
                Exciter::Burst => {
                    if self.pulse_pos < self.pulse_len {
                        let ph = core::f32::consts::PI * 2.0 * self.pulse_pos as f32
                            / self.pulse_len as f32;
                        self.pulse_pos += 1;
                        let env = 0.5 - 0.5 * ph.cos();
                        let n = self.white();
                        // dark<->bright: HP then 2x one-pole LP (band-limited)
                        self.exc_hp =
                            self.exc_a_hp * self.exc_hp + (1.0 - self.exc_a_hp) * n;
                        let hp = n - self.exc_hp;
                        self.exc_lp1 =
                            self.exc_a_lp * self.exc_lp1 + (1.0 - self.exc_a_lp) * hp;
                        self.exc_lp2 = self.exc_a_lp * self.exc_lp2
                            + (1.0 - self.exc_a_lp) * self.exc_lp1;
                        2.2 * p.velocity * env * self.exc_lp2
                    } else {
                        0.0
                    }
                }
                Exciter::Buckling => {
                    // click train: power-law amplitudes, rate AND strength
                    // riding the bank's own energy — crumple dies with the
                    // body (plus a 30 ms warmup window so the first clicks
                    // exist before any energy does). Passivity notes: click
                    // strength scales sqrt(e_norm), so re-injection weakens
                    // as the bank calms — convergent, no limit cycle; the
                    // clicks_left cap is a pure backstop.
                    if self.clicks_left > 0 {
                        if self.next_click == 0 {
                            let u = (0.5 * (self.white() + 1.0)).clamp(1e-4, 1.0);
                            let a = (u.powf(-1.0 / 1.5)).min(3.0) / 3.0; // power law
                            let drive_env =
                                self.e_norm.max((1.0 - self.t / 0.03).max(0.0));
                            let core = 2.0 * p.velocity * a * drive_env.sqrt();
                            // arm a raised-cosine snap, loudness-matched to
                            // M7's delta via the force-integral (bank peak
                            // rides integral of force): amp*plen = const,
                            // calibrated 3.0 against M7 full-render peaks at
                            // color 0.3/0.5/0.9 (within +/-1.6 dB at all).
                            let amp = core * 3.0 / self.buck_plen as f32;
                            // retrigger while active: keep the louder snap
                            if self.buck_ppos >= self.buck_plen || amp > self.buck_pamp
                            {
                                self.buck_pamp = amp;
                                self.buck_ppos = 0;
                            }
                            self.clicks_left -= 1;
                            self.clicks_fired += 1;
                            let u2 = (0.5 * (self.white() + 1.0)).clamp(1e-3, 1.0);
                            let rate = self.buck_rate0 * self.e_norm.max(0.04);
                            let mean = (self.sr / rate).max(2.0);
                            let iv = (mean * -(u2.ln())).clamp(0.25 * mean, 4.0 * mean);
                            self.next_click = iv as usize;
                        } else {
                            self.next_click -= 1;
                        }
                    }
                    let imp = if self.buck_ppos < self.buck_plen {
                        let ph = self.buck_ppos as f32 / self.buck_plen as f32;
                        self.buck_ppos += 1;
                        self.buck_pamp
                            * 0.5
                            * (1.0 - (2.0 * core::f32::consts::PI * ph).cos())
                    } else {
                        0.0
                    };
                    // click sharpness: 3x one-pole LP (band-limited pulse;
                    // 18 dB/oct meets the >=60 dB @ 0.45 sr doctrine spec —
                    // exc_hp is unused by this exciter and serves as stage 3)
                    self.exc_lp1 =
                        self.exc_a_lp * self.exc_lp1 + (1.0 - self.exc_a_lp) * imp;
                    self.exc_lp2 = self.exc_a_lp * self.exc_lp2
                        + (1.0 - self.exc_a_lp) * self.exc_lp1;
                    self.exc_hp = self.exc_a_lp * self.exc_hp
                        + (1.0 - self.exc_a_lp) * self.exc_lp2;
                    3.6 * self.exc_hp
                }
                Exciter::Stick => {
                    // the snap: a sub-ms raised-cosine contact pulse —
                    // broadband through mechanism (short contact), zero
                    // waveshaping, level-matched via the force integral
                    if self.pulse_pos < self.pulse_len {
                        let ph = core::f32::consts::PI * 2.0 * self.pulse_pos as f32
                            / self.pulse_len as f32;
                        self.pulse_pos += 1;
                        self.stick_amp * (0.5 - 0.5 * ph.cos())
                    } else {
                        0.0
                    }
                }
                Exciter::Raw => {
                    let mut f = 0.0f32;
                    if self.pulse_pos == 0 {
                        f = 2.0 * p.velocity; // the impulse
                    }
                    if self.pulse_pos < self.raw_tail_len {
                        // DC-kick: smooth LF bump (band-limited by shape)
                        let ph = core::f32::consts::PI * self.pulse_pos as f32
                            / self.raw_tail_len as f32;
                        f += 0.35 * p.velocity * ph.sin();
                    }
                    if self.pulse_pos < self.pulse_len {
                        self.pulse_pos += 1;
                    }
                    // soften LP (color): 2x one-pole
                    self.exc_lp1 = self.exc_a_lp * self.exc_lp1 + (1.0 - self.exc_a_lp) * f;
                    self.exc_lp2 =
                        self.exc_a_lp * self.exc_lp2 + (1.0 - self.exc_a_lp) * self.exc_lp1;
                    self.exc_lp2
                }
            };

            // cascade drive envelope
            let buildup = if p.cascade_amt > 0.0 {
                p.cascade_attack
                    + (1.0 - p.cascade_attack) * (1.0 - (-self.t / p.cascade_tau).exp())
            } else {
                0.0
            };
            let e_low_norm = self.e_norm;
            let dep = if p.cascade_conserve && p.cascade_amt > 0.0 {
                (1.0 - 0.8 * p.cascade_amt * p.velocity * p.velocity * buildup)
                    .clamp(0.05, 1.0)
                    .sqrt()
            } else {
                1.0
            };
            let casc_noise = if p.cascade_amt > 0.0 && !p.cascade_coherent {
                self.white() * buildup * e_low_norm
            } else {
                0.0
            };
            let casc_gain = buildup * e_low_norm; // coherent output envelope

            // NL2 satellites: seat displacement from previous states, penalty
            // contact, symplectic integration, reaction into this sample.
            // STEREO round 2: per-channel banks — the L bank listens to the
            // L rotors, the R bank to the R signal (detuned rotor, or the
            // width/sub-rotate phase tap), POST-decoherence: each ear gets
            // its own contact events ("different satellites floating
            // around"). Mono path (stereo off) = legacy single bank.
            let nm = self.n_modes;
            let detuned_now = self.detuned;
            let mut sat_react = [0.0f32; MAX_SATS];
            let mut sat_react_r = [0.0f32; MAX_SATS];
            let mut sat_radiate = 0.0f32;
            let mut sat_radiate_r = 0.0f32;
            let b_snc = p.bounce.clamp(0.0, 1.0);
            let restitution = 0.40 + 0.30 * b_snc; // strictly < 1: energy-honest
            let mut shock = 0.0f32; // contact-ENTRY impulses (rattle->cascade)
            for j in 0..self.n_sats {
                let mut sd = 0.0f32;
                let mut sd_r = 0.0f32;
                for k in 0..nm {
                    let m = &self.modes[k];
                    sd += self.sat_w[j][k] * m.u;
                    if stereo {
                        let (ru, rv) = if detuned_now { (m.ur, m.vr) } else { (m.u, m.v) };
                        sd_r += self.sat_w_r[j][k] * (ru * m.ct + rv * m.st);
                    }
                }
                let om = self.sat_om[j];
                let rest = self.sat_rest_eff[j];
                // ---- L bank (M8 dynamics: spring blends out, gravity +
                // restitution blend in as bounce rises; the settle is
                // geometric because flight time ∝ v and v <- e·v per bounce)
                self.sat_speak[j] = self.sat_speak[j].max(sd.abs());
                let sd_n = sd / self.sat_speak[j];
                let pen = (sd_n - self.sat_z[j]).clamp(0.0, 2.0);
                let f_n = if pen > 0.0 {
                    self.contacts += 1;
                    pen.powf(1.5)
                } else {
                    0.0
                };
                let sdv_raw = (sd_n - self.sat_sdp[j]) / dt; // surface velocity
                self.sat_sdp[j] = sd_n;
                let sdv = sdv_raw.clamp(-10.0, 10.0); // capped momentum transfer
                let sacc = (sdv_raw - self.sat_sdvp[j]) / dt; // surface accel
                self.sat_sdvp[j] = sdv_raw;
                if b_snc > 0.0 && self.sat_carried[j] {
                    // CARRIED: riding the surface silently. HOP when the
                    // table drops out faster than gravity (the vibrating-
                    // table condition) — then ballistic with the (capped)
                    // table velocity. Dense chatter while the body is loud,
                    // silence as it calms: the settle arc, physically.
                    self.sat_z[j] = sd_n;
                    self.sat_v[j] = sdv;
                    if sacc < -self.sat_grav {
                        self.sat_carried[j] = false;
                        self.sat_incontact[j] = false;
                    }
                } else {
                    if pen > 0.0 && !self.sat_incontact[j] {
                        // click loudness = APPROACH SPEED (impact velocity),
                        // not instantaneous penetration — the entry sample
                        // has only crossed by a hair; pen^1.5 there is
                        // near-zero and starves both voice and shock.
                        let rel_speed = (self.sat_v[j] - sdv).abs();
                        // impact velocity is physical in EVERY regime — not
                        // bounce-gated (fixed reference scale for stability)
                        let hit_amp = f_n + 0.55 * (rel_speed / 17.3).min(3.0);
                        if b_snc > 0.0 {
                            // entry — restitution off the moving surface
                            let rel = self.sat_v[j] - sdv;
                            if rel < 0.0 {
                                self.sat_v[j] = sdv - restitution * rel;
                                // slow post-bounce relative speed => CAPTURED
                                if (self.sat_v[j] - sdv).abs() < 0.15 * self.sat_grav.sqrt()
                                {
                                    self.sat_carried[j] = true;
                                }
                            }
                        }
                        shock += hit_amp; // rattle->cascade shock (entries only)
                        self.entries += 1;
                        // ring the satellite's partial voice
                        for q in 0..SAT_PARTIALS {
                            self.sat_pu[j][q] += hit_amp * self.sat_pamp[j][q];
                        }
                    }
                    self.sat_incontact[j] = pen > 0.0;
                    // pressed buzz: continuous contact scrape drives the
                    // voice too (bounce mode is pure event-clicks)
                    if pen > 0.0 && b_snc < 1.0 {
                        let scrape = f_n * 0.12 * (1.0 - b_snc);
                        for q in 0..SAT_PARTIALS {
                            self.sat_pu[j][q] += scrape * self.sat_pamp[j][q];
                        }
                    }
                    let acc = -om * om * (self.sat_z[j] - rest) * (1.0 - b_snc)
                        - self.sat_grav * b_snc
                        - 2.0 * self.sat_ze[j] * om * self.sat_v[j] * (1.0 - 0.7 * b_snc)
                        + self.sat_kc[j] * f_n * (1.0 - b_snc);
                    self.sat_v[j] += dt * acc;
                    self.sat_z[j] += dt * self.sat_v[j];
                }
                // finite tripwire: a non-finite satellite resets to rest
                // (passivity insurance under hostile parameter fuzz)
                if !(self.sat_z[j].is_finite() && self.sat_v[j].is_finite()) {
                    self.sat_z[j] = rest;
                    self.sat_v[j] = 0.0;
                    self.sat_carried[j] = false;
                }
                // bounce mode is event physics — continuous push-reaction is
                // pressed-mode physics only (also starves the pump)
                sat_react[j] = 0.15 * f_n * (1.0 - b_snc);
                // multi-modal voice: partial rotors ring; sum is the radiation
                let mut voice = 0.0f32;
                for q in 0..SAT_PARTIALS {
                    if self.sat_pamp[j][q] > 0.0 {
                        let pu = self.sat_prr[j][q]
                            * (self.sat_pu[j][q] * self.sat_pcw[j][q]
                                - self.sat_pv[j][q] * self.sat_psw[j][q]);
                        let pv = self.sat_prr[j][q]
                            * (self.sat_pu[j][q] * self.sat_psw[j][q]
                                + self.sat_pv[j][q] * self.sat_pcw[j][q]);
                        self.sat_pu[j][q] = pu;
                        self.sat_pv[j][q] = pv;
                        voice += pu * self.sat_pamp[j][q];
                    }
                }
                sat_radiate += p.sat_level[j] * voice;
                // ---- R bank (stereo only), same laws, own contact events
                if stereo {
                    self.sat_speak_r[j] = self.sat_speak_r[j].max(sd_r.abs());
                    let sd_nr = sd_r / self.sat_speak_r[j];
                    let pen_r = (sd_nr - self.sat_z_r[j]).clamp(0.0, 2.0);
                    let f_nr = if pen_r > 0.0 {
                        self.contacts_r += 1;
                        pen_r.powf(1.5)
                    } else {
                        0.0
                    };
                    let sdv_raw_r = (sd_nr - self.sat_sdp_r[j]) / dt;
                    self.sat_sdp_r[j] = sd_nr;
                    let sdv_r = sdv_raw_r.clamp(-10.0, 10.0);
                    let sacc_r = (sdv_raw_r - self.sat_sdvp_r[j]) / dt;
                    self.sat_sdvp_r[j] = sdv_raw_r;
                    if b_snc > 0.0 && self.sat_carried_r[j] {
                        self.sat_z_r[j] = sd_nr;
                        self.sat_v_r[j] = sdv_r;
                        if sacc_r < -self.sat_grav {
                            self.sat_carried_r[j] = false;
                            self.sat_incontact_r[j] = false;
                        }
                    } else {
                        if pen_r > 0.0 && !self.sat_incontact_r[j] {
                            let rel_speed = (self.sat_v_r[j] - sdv_r).abs();
                            let hit_amp = f_nr + 0.55 * (rel_speed / 17.3).min(3.0);
                            if b_snc > 0.0 {
                                let rel = self.sat_v_r[j] - sdv_r;
                                if rel < 0.0 {
                                    self.sat_v_r[j] = sdv_r - restitution * rel;
                                    if (self.sat_v_r[j] - sdv_r).abs()
                                        < 0.15 * self.sat_grav.sqrt()
                                    {
                                        self.sat_carried_r[j] = true;
                                    }
                                }
                            }
                            shock += hit_amp;
                            self.entries += 1;
                            for q in 0..SAT_PARTIALS {
                                self.sat_pu_r[j][q] += hit_amp * self.sat_pamp[j][q];
                            }
                        }
                        self.sat_incontact_r[j] = pen_r > 0.0;
                        if pen_r > 0.0 && b_snc < 1.0 {
                            let scrape = f_nr * 0.12 * (1.0 - b_snc);
                            for q in 0..SAT_PARTIALS {
                                self.sat_pu_r[j][q] += scrape * self.sat_pamp[j][q];
                            }
                        }
                        let acc_r = -om * om * (self.sat_z_r[j] - rest) * (1.0 - b_snc)
                            - self.sat_grav * b_snc
                            - 2.0 * self.sat_ze[j] * om * self.sat_v_r[j]
                                * (1.0 - 0.7 * b_snc)
                            + self.sat_kc[j] * f_nr * (1.0 - b_snc);
                        self.sat_v_r[j] += dt * acc_r;
                        self.sat_z_r[j] += dt * self.sat_v_r[j];
                    }
                    if !(self.sat_z_r[j].is_finite() && self.sat_v_r[j].is_finite()) {
                        self.sat_z_r[j] = rest;
                        self.sat_v_r[j] = 0.0;
                        self.sat_carried_r[j] = false;
                    }
                    sat_react_r[j] = 0.15 * f_nr * (1.0 - b_snc);
                    let mut voice_r = 0.0f32;
                    for q in 0..SAT_PARTIALS {
                        if self.sat_pamp[j][q] > 0.0 {
                            let pu = self.sat_prr[j][q]
                                * (self.sat_pu_r[j][q] * self.sat_pcw[j][q]
                                    - self.sat_pv_r[j][q] * self.sat_psw[j][q]);
                            let pv = self.sat_prr[j][q]
                                * (self.sat_pu_r[j][q] * self.sat_psw[j][q]
                                    + self.sat_pv_r[j][q] * self.sat_pcw[j][q]);
                            self.sat_pu_r[j][q] = pu;
                            self.sat_pv_r[j][q] = pv;
                            voice_r += pu * self.sat_pamp[j][q];
                        }
                    }
                    sat_radiate_r += p.sat_level[j] * voice_r;
                }
            }
            // rattle->cascade: entry shocks kick the receiver shadow rings,
            // scaled by sqrt(e_norm) — re-injection weakens as the bank
            // calms (the buckling passivity law): convergent, no limit cycle
            self.rc_shock = if p.rattle_casc > 0.0 {
                shock * e_low_norm.sqrt()
            } else {
                0.0
            };

            // M11 — the wire bank (Net1): 16 wires in unilateral contact
            // with the snare-side head, from PREVIOUS-sample motion (the
            // satellite convention). Head source: R2 listen tap when the
            // cavity is on, batter net-volume displacement otherwise —
            // wires work without the cavity. NORMALIZED frame (running
            // peak of head motion): contact recurs at every velocity,
            // which is why the ring survives ghost hits (M10.5 target ①).
            let mut wire_ring = 0.0f32; // rotor radiation: the tonal peaks
            let mut wire_snap = 0.0f32; // entry pulses: the crack
            self.wire_react = 0.0;
            if self.wire_on {
                let hd = if self.cav_on {
                    self.r2_x2out
                } else {
                    let mut x1 = 0.0f32;
                    for m in self.modes[..nm].iter() {
                        if m.cpl > 0.0 {
                            x1 += m.cpl * m.u;
                        }
                    }
                    x1
                };
                self.wire_speak = self.wire_speak.max(hd.abs());
                let hd_n = hd / self.wire_speak;
                let hdv = ((hd_n - self.wire_hdp) / dt).clamp(-4000.0, 4000.0);
                self.wire_hdp = hd_n;
                // contact-feed loudness follows the head's REAL envelope
                // (~4 ms smoother of |hd_n|, which decays with the body
                // since wire_speak is a frozen max): recurring late
                // contacts stay QUIET — the wire tail is then owned by
                // the wire T60 salt (target ③), while ghost hits keep
                // their full ring (env ≈ 1 near their own onset).
                self.wire_henv =
                    self.a_env * self.wire_henv + (1.0 - self.a_env) * hd_n.abs();
                // squared: radiated click energy rides impact KINETIC
                // energy — the contact-noise floor dies at twice the
                // body's dB rate (real drums: 8–14 k T60 ≈ 0.43 s under
                // an LF body ringing ~0.8 s; displacement-proportional
                // feed measured 0.77 s — the energy law lands it)
                let feed = (self.wire_henv * self.wire_henv).min(1.0);
                // during the strike itself the head CARRIES the wires
                // (the vibrating-table law): they ride the surface with
                // the armed throw velocity and generate no entry events —
                // separation begins when the force pulse ends, so the
                // first impacts are RE-LANDINGS at 1–3 ms (the crack's
                // measured rise), not the head's first surge at t≈0.
                let striking = self.pulse_pos < self.pulse_len && self.t < 0.004;
                let mut react = 0.0f32;
                for k in 0..N_WIRES {
                    if striking {
                        self.wire_z[k] = hd_n.max(self.wire_rest[k]);
                        self.wire_incontact[k] = true;
                        continue;
                    }
                    // unilateral contact: pen geometrically bounded (the
                    // passivity law — bound the geometry, not the force)
                    let pen = (hd_n - self.wire_z[k]).clamp(0.0, 2.0);
                    let f_n = if pen > 0.0 { pen.powf(1.5) } else { 0.0 };
                    if pen > 0.0 && !self.wire_incontact[k] {
                        // entry loudness = approach speed (M8 impact law);
                        // normalized by the wire's own throw-speed scale
                        let rel = (self.wire_vz[k] - hdv).abs();
                        let hit = (f_n
                            + 0.5 * (rel / (0.9 * self.wire_omz[k])).min(3.0))
                            * feed;
                        self.wire_u[k] += hit; // ring the wire
                        wire_snap += hit; // the crack pulse
                    }
                    self.wire_incontact[k] = pen > 0.0;
                    // pressed scrape keeps the ring fed through the decay
                    if pen > 0.0 {
                        self.wire_u[k] += 0.06 * f_n * feed;
                    }
                    // stiff tension return spring + unilateral reaction
                    let omz = self.wire_omz[k];
                    let acc = -omz * omz * (self.wire_z[k] - self.wire_rest[k])
                        - 0.5 * omz * self.wire_vz[k]
                        + 0.05 * omz * omz * f_n;
                    self.wire_vz[k] += dt * acc;
                    self.wire_z[k] += dt * self.wire_vz[k];
                    if !(self.wire_z[k].is_finite() && self.wire_vz[k].is_finite())
                    {
                        self.wire_z[k] = self.wire_rest[k];
                        self.wire_vz[k] = 0.0;
                    }
                    react += f_n;
                    // wire ring rotor (coupled-form, per-wire T60 salt)
                    let u = self.wire_r[k]
                        * (self.wire_u[k] * self.wire_cw[k]
                            - self.wire_v[k] * self.wire_sw[k]);
                    let v = self.wire_r[k]
                        * (self.wire_u[k] * self.wire_sw[k]
                            + self.wire_v[k] * self.wire_cw[k]);
                    self.wire_u[k] = u;
                    self.wire_v[k] = v;
                    wire_ring += u;
                }
                // reaction onto the head, rescaled ONLINE from the
                // normalized frame to head units by wire_speak (the
                // M3.2 unit lesson) — applied ×(1−r) at the receiver
                // (the M10 dissipation law). Bounded: ≤ 0.012·16·2.8 ≈
                // 0.5× head scale worst-case, typical ~0.03.
                self.wire_react = 0.012 * react * self.wire_speak;
            }

            // M10 — cavity taps, from PREVIOUS-sample states (same
            // convention as the satellite seats): x1 = batter net-volume
            // displacement, x2 = resonant-head dito, cav_p = cavity
            // pressure. Physics: adiabatic compression p ∝ (x1 − x2)
            // through the cavity resonators; F_batter = −p, F_R2 = +p —
            // the skew-symmetric exchange (energy moves, none is minted).
            let (cav_p, cav_dx) = if self.cav_on {
                let mut x1 = 0.0f32;
                for m in self.modes[..nm].iter() {
                    if m.cpl > 0.0 {
                        x1 += m.cpl * m.u;
                    }
                }
                let mut x2 = 0.0f32;
                let mut pr = 0.0f32;
                for q in 0..self.n_r2 {
                    x2 += self.r2_cpl[q] * self.r2_u[q];
                }
                for j in 0..CAV_MODES {
                    pr += self.cav_g[j] * self.cav_u[j];
                }
                (pr, x1 - x2)
            } else {
                (0.0, 0.0)
            };

            let mut yl = 0.0f32;
            let mut yr = 0.0f32;
            // rings accumulate SEPARATELY and join after the peak tracker:
            // the rc-kick normalizes against dust_peak, so rings must never
            // feed the very peak that scales their own excitation (self-
            // referential normalization = feedback gain > 1 = inf; the
            // passivity law applies to normalizers too)
            let mut ring_l = 0.0f32;
            let mut ring_r = 0.0f32;
            let coherent = p.cascade_coherent && p.cascade_amt > 0.0;
            let rc_on = p.rattle_casc > 0.0 && self.n_sats > 0;
            let rings_on = coherent || rc_on;
            // contact-entry shock, this sample, scaled to ring-state units
            // by the bank's running peak (online normalization)
            let kick = self.rc_shock * self.dust_peak * 0.5;
            // ring output gate: cascade path (buildup x energy) + rattle
            // path (fixed mix — the rings decay by their own damping)
            let ring_gate = (if coherent { casc_gain } else { 0.0 })
                + 0.35 * p.rattle_casc.clamp(0.0, 1.0);
            let n_sats = self.n_sats;
            let sat_w = &self.sat_w;
            let detuned = self.detuned;
            let cav_ret = self.cav_ret;
            // M11: with no cavity the wires sit directly on the batter —
            // their reaction loads it through the same volume weights
            let wire_react_bank = if self.wire_on && !self.cav_on {
                self.wire_react
            } else {
                0.0
            };
            for (k, m) in self.modes[..nm].iter_mut().enumerate() {
                // M10: cavity pressure reacts on the batter (−p, the
                // heavy-head side of the skew-symmetric pair), ×(1−r):
                // the receiver-dissipation normalization (see trigger)
                let base_drive = m.amp * f_in + m.inj * casc_noise
                    - (cav_ret * cav_p + wire_react_bank) * m.cpl * (1.0 - m.r);
                let mut drive = base_drive;
                for j in 0..n_sats {
                    drive -= sat_react[j] * sat_w[j][k];
                }
                let u = m.r * (m.u * m.cw - m.v * m.sw) + drive;
                let v = m.r * (m.u * m.sw + m.v * m.cw);
                m.u = u;
                m.v = v;
                // R source: detuned rotor if engaged (with its own damping
                // and its own satellite-bank reaction), else the shared rotor
                let (ru, rv) = if detuned {
                    let mut drive_r = base_drive;
                    for j in 0..n_sats {
                        drive_r -= sat_react_r[j] * sat_w[j][k];
                    }
                    let ur = m.rr * (m.ur * m.cwr - m.vr * m.swr) + drive_r;
                    let vr = m.rr * (m.ur * m.swr + m.vr * m.cwr);
                    m.ur = ur;
                    m.vr = vr;
                    (ur, vr)
                } else {
                    (u, v)
                };
                // width/sub-rotate tap: L = u; R = ru·cosθ + rv·sinθ
                let mut cl = if m.low { u * dep } else { u };
                let r_tap = ru * m.ct + rv * m.st;
                let mut cr = if m.low { r_tap * dep } else { r_tap };
                if rings_on && !m.low {
                    let mut u2 = m.r * (m.u2 * m.cw - m.v2 * m.sw);
                    let mut v2 = m.r * (m.u2 * m.sw + m.v2 * m.cw);
                    if rc_on && kick > 0.0 {
                        // phase-salted shock kick (the DC-click lesson:
                        // never kick a hundred rings in phase)
                        let ph = 2.0 * core::f32::consts::PI * ((k as f32 * PHI) % 1.0);
                        u2 += kick * m.inj_rc * ph.sin();
                        v2 += kick * m.inj_rc * ph.cos();
                    }
                    m.u2 = u2;
                    m.v2 = v2;
                    ring_l += u2 * ring_gate * m.pgl;
                    ring_r += (u2 * m.ct + v2 * m.st) * ring_gate * m.pgr;
                }
                // mode spread: equal-power pan (1.0/1.0 at spread 0)
                yl += cl * m.pgl;
                yr += cr * m.pgr;
            }
            // M10 — Stick contact radiation: the band-limited pulse
            // itself joins the output (the tick every real stick makes;
            // the mallet's 3 ms thud has no audible direct term). Runs
            // through its own Cheby-II cascade — a 4-sample raised
            // cosine is only ~−32 dB at 0.45·sr on its own; the cascade
            // takes the gate to spec. Joins before the peak trackers:
            // the wires HEAR the crack (bed attack sharpens, honestly).
            if p.exciter == Exciter::Stick && self.stick_dir > 0.0 {
                let mut d = self.stick_dir * f_in;
                for (c, st) in self.bed_bqc.iter().zip(self.stick_bq.iter_mut()) {
                    let y = c[0] * d + st[0];
                    st[0] = c[1] * d - c[3] * y + st[1];
                    st[1] = c[2] * d - c[4] * y;
                    d = y;
                }
                yl += d;
                yr += d;
            }
            // M10 — cavity + resonant-head update. Cavity rotors are
            // driven by the volume change (x1 − x2); R2 by +p. Radiation
            // joins BEFORE the peak trackers (R2 never consumes a
            // normalizer that includes it — the normalizer law holds).
            if self.cav_on {
                // every injection ×(1−r): the receiver's dissipation
                // normalizes away its own resonant gain (the M10 law)
                let inj = self.cav_kc * cav_dx;
                for j in 0..CAV_MODES {
                    let u = self.cav_r[j]
                        * (self.cav_u[j] * self.cav_cw[j]
                            - self.cav_v[j] * self.cav_sw[j])
                        + inj * self.cav_g[j] * (1.0 - self.cav_r[j]);
                    let v = self.cav_r[j]
                        * (self.cav_u[j] * self.cav_sw[j]
                            + self.cav_v[j] * self.cav_cw[j]);
                    self.cav_u[j] = u;
                    self.cav_v[j] = v;
                }
                // M11: the wires press back on the snare-side head
                // (unilateral reaction, dissipation-normalized)
                let f2 = self.cav_k2 * cav_p - self.wire_react;
                let mut x2o = 0.0f32;
                for q in 0..self.n_r2 {
                    let u = self.r2_r[q]
                        * (self.r2_u[q] * self.r2_cw[q]
                            - self.r2_v[q] * self.r2_sw[q])
                        + f2 * self.r2_cpl[q] * (1.0 - self.r2_r[q]);
                    let v = self.r2_r[q]
                        * (self.r2_u[q] * self.r2_sw[q]
                            + self.r2_v[q] * self.r2_cw[q]);
                    self.r2_u[q] = u;
                    self.r2_v[q] = v;
                    x2o += self.r2_out[q] * u;
                }
                // finite tripwire (passivity insurance under fuzz):
                // a non-finite cavity/head resets silently
                if !x2o.is_finite() {
                    self.cav_u = [0.0; CAV_MODES];
                    self.cav_v = [0.0; CAV_MODES];
                    self.r2_u = [0.0; R2_MODES];
                    self.r2_v = [0.0; R2_MODES];
                    x2o = 0.0;
                }
                self.r2_x2out = x2o;
                self.x2_peak = self.x2_peak.max(x2o.abs());
                // radiation GAIN capped at 0.7× the bank running peak:
                // frequency-coincident patches (body tuned onto the
                // cavity) otherwise radiate ~+20 dB over typical. A gain
                // min, not a waveshaper; x2_peak's input never contains
                // this output, and rad ≤ 0.7·dust_peak can never run the
                // shared tracker away (the satellite-normalizer pattern).
                let g_rad = self
                    .r2_rad
                    .min(0.7 * self.dust_peak / self.x2_peak.max(1e-12));
                let rad = g_rad * x2o;
                yl += rad;
                yr += rad;
            }
            // shared bank-peak tracker (rattle + dust normalizers) — L side,
            // matching the mono lineage. Rings join LAST (after dust): they
            // must not feed the kick normalizer (feedback) NOR the dust
            // envelope (env_n > 1 meets the thr=0 dB pole — see dust block).
            self.dust_peak = self.dust_peak.max(yl.abs());
            // M8 — radiation smoother (2x one-pole @10 kHz): soften click
            // edges before normalization so the band-limit gate holds
            self.sat_rad_lp1 =
                self.a_rad * self.sat_rad_lp1 + (1.0 - self.a_rad) * sat_radiate;
            self.sat_rad_lp2 =
                self.a_rad * self.sat_rad_lp2 + (1.0 - self.a_rad) * self.sat_rad_lp1;
            let sat_radiate = self.sat_rad_lp2;
            self.sat_rad_lp1r =
                self.a_rad * self.sat_rad_lp1r + (1.0 - self.a_rad) * sat_radiate_r;
            self.sat_rad_lp2r =
                self.a_rad * self.sat_rad_lp2r + (1.0 - self.a_rad) * self.sat_rad_lp1r;
            let sat_radiate_r = self.sat_rad_lp2r;
            if self.n_sats > 0 {
                self.sat_peak = self.sat_peak.max(sat_radiate.abs());
                if self.sat_peak > 1e-9 {
                    let rad = sat_radiate / self.sat_peak * self.dust_peak * self.sat_gain;
                    if stereo {
                        // STEREO round 2: per-channel banks radiate into
                        // their own ears (supersedes the round-1 alternate
                        // panning) — each channel hears ITS satellites
                        yl += rad;
                        self.sat_peak_r = self.sat_peak_r.max(sat_radiate_r.abs());
                        if self.sat_peak_r > 1e-9 {
                            yr += sat_radiate_r / self.sat_peak_r
                                * self.dust_peak
                                * self.sat_gain;
                        }
                    } else {
                        yl += rad;
                        yr += rad;
                    }
                }
            }

            // M11 — wire radiation: ring + crack through their own
            // Cheby-II gate (band-limit doctrine), normalized online
            // against their own running peak, mixed vs the bank peak
            // (the satellite pattern — dust_peak was tracked BEFORE
            // this join, so no normalizer contains its consumer). The
            // bed source taps yl AFTER this: the follower hears the
            // wires (physical — the noise floor breathes with them).
            // Single bank, radiated equally L/R (the bed's per-channel
            // noise supplies the stereo texture above it).
            if self.wire_on {
                let mut wr = wire_ring + 2.4 * wire_snap;
                for (c, st) in self.bed_bqc.iter().zip(self.wire_bq.iter_mut()) {
                    let y = c[0] * wr + st[0];
                    st[0] = c[1] * wr - c[3] * y + st[1];
                    st[1] = c[2] * wr - c[4] * y;
                    wr = y;
                }
                self.wire_peak = self.wire_peak.max(wr.abs());
                if self.wire_peak > 1e-9 {
                    let wrad =
                        wr / self.wire_peak * self.dust_peak * p.wires * 0.9;
                    yl += wrad;
                    yr += wrad;
                }
            }

            // dust layer: envelope-gated bandpassed noise, running-peak norm.
            // Stereo engaged → two uncorrelated chains (Microtonic dispersion);
            // otherwise the legacy single chain, duplicated (bit-identity).
            // M9: at bed_* defaults the LEGACY path runs verbatim
            // (bit-identical); any non-default bed param engages the
            // wire-bed pipeline (own release, source region, comb, bright).
            if p.dust_level > 0.0 {
                let legacy_bed = p.bed_release <= 0.0
                    && p.bed_source <= 0.0
                    && p.bed_comb <= 0.0
                    && (p.bed_bright - 0.5).abs() < 1e-6;
                let thr = (10.0f32).powf(p.dust_thr_db / 20.0).min(0.999);
                if legacy_bed {
                    let ay = yl.abs();
                    let a_env = self.a_env;
                    self.dust_env = a_env * self.dust_env + (1.0 - a_env) * ay;
                    let env_n = self.dust_env / self.dust_peak;
                    // thr capped below 1: at dust_thr_db == 0 the (1 - thr)
                    // denominator is a POLE (div-by-zero -> inf output; found
                    // by param fuzz). Latent since M3.2, exposed when env_n
                    // could exceed 1.
                    // knee denominator floored: near thr = 0 dB the exact
                    // (1 - thr) normalization is a gain pole (~1000x just
                    // under the cap) — floor keeps extreme thresholds
                    // selective without the amplification cliff
                    let g = if env_n > thr {
                        ((env_n - thr) / (1.0 - thr).max(0.15)).powf(p.dust_follow)
                    } else {
                        0.0
                    };
                    let a_hp = self.a_hp;
                    let a_lp = self.a_lp;
                    let w = self.white();
                    self.dust_lp1 = a_hp * self.dust_lp1 + (1.0 - a_hp) * w;
                    let hp = w - self.dust_lp1;
                    self.dust_lp2 = a_lp * self.dust_lp2 + (1.0 - a_lp) * hp;
                    let d_l =
                        p.dust_level * self.dust_lp2 * g * self.dust_peak * 0.7;
                    yl += d_l;
                    if stereo {
                        let w2 = self.white();
                        self.dust_lp1r = a_hp * self.dust_lp1r + (1.0 - a_hp) * w2;
                        let hp2 = w2 - self.dust_lp1r;
                        self.dust_lp2r = a_lp * self.dust_lp2r + (1.0 - a_lp) * hp2;
                        yr += p.dust_level * self.dust_lp2r * g * self.dust_peak * 0.7;
                    } else {
                        yr += d_l;
                    }
                } else {
                    // ---- M9 wire-bed pipeline ----
                    // source region: crossfade full-band |y| with the
                    // 150-800 Hz head-motion band (cavity proxy). Taps the
                    // same post-rattle, pre-bed point as legacy — no bed or
                    // ring output feeds it (normalizer law).
                    self.bed_src_s1 =
                        self.a_src_hp * self.bed_src_s1 + (1.0 - self.a_src_hp) * yl;
                    let src_hp = yl - self.bed_src_s1;
                    self.bed_src_s2 =
                        self.a_src_lp * self.bed_src_s2 + (1.0 - self.a_src_lp) * src_hp;
                    // M10 — bed source GRADUATES: with the cavity on, the
                    // source region crossfades from the 150–800 Hz proxy
                    // band toward REAL resonant-head motion (normalized
                    // by its own running peak, rescaled to bank units by
                    // dust_peak — neither normalizer sees bed output).
                    let src_prox = self.bed_src_s2.abs() * 2.0;
                    let src_reg = if self.cav_on {
                        let cvm = p.cavity.clamp(0.0, 1.0);
                        let real = self.r2_x2out.abs() / self.x2_peak.max(1e-9)
                            * self.dust_peak
                            * 1.4;
                        (1.0 - cvm) * src_prox + cvm * real
                    } else {
                        src_prox
                    };
                    let src =
                        (1.0 - p.bed_source) * yl.abs() + p.bed_source * src_reg;
                    // attack/release follower: 1 ms up, bed_release down —
                    // release may exceed body T60 (the decay inversion)
                    if src > self.bed_env {
                        self.bed_env =
                            self.bed_a_atk * self.bed_env + (1.0 - self.bed_a_atk) * src;
                    } else {
                        self.bed_env *= self.bed_a_rel;
                    }
                    let env_n = self.bed_env / self.dust_peak;
                    let g = if env_n > thr {
                        ((env_n - thr) / (1.0 - thr).max(0.15)).powf(p.dust_follow)
                    } else {
                        0.0
                    };
                    let a_hp = self.bed_a_hp;
                    let a_lp = self.bed_a_lp;
                    let fb = 0.88 * p.bed_comb;
                    // M10: plain per-sample white — the Cheby-II cascade
                    // below owns the band-limit gate now; the M9 ZOH core
                    // (and its sinc shelf over the top octave) is gone.
                    // L chain: bright-banded noise -> wire comb -> biquads
                    let w = self.white();
                    self.dust_lp1 = a_hp * self.dust_lp1 + (1.0 - a_hp) * w;
                    let hp = w - self.dust_lp1;
                    self.dust_lp2 = a_lp * self.dust_lp2 + (1.0 - a_lp) * hp;
                    let idx_l = (self.comb_pos + 256 - self.comb_dly_l) & 255;
                    let dl = self.comb_buf_l[idx_l];
                    self.comb_lp_l =
                        self.a_comb_lp * self.comb_lp_l + (1.0 - self.a_comb_lp) * dl;
                    let cw_l = self.dust_lp2 + fb * self.comb_lp_l;
                    self.comb_buf_l[self.comb_pos] = cw_l;
                    let shaped_l = (1.0 - p.bed_comb) * self.dust_lp2 + p.bed_comb * cw_l;
                    let mut sm = shaped_l;
                    for (c, st) in self.bed_bqc.iter().zip(self.bed_bq_l.iter_mut()) {
                        let y = c[0] * sm + st[0];
                        st[0] = c[1] * sm - c[3] * y + st[1];
                        st[1] = c[2] * sm - c[4] * y;
                        sm = y;
                    }
                    let d_l = p.dust_level * sm * g * self.dust_peak * 0.7;
                    yl += d_l;
                    if stereo {
                        let w2 = self.white();
                        self.dust_lp1r = a_hp * self.dust_lp1r + (1.0 - a_hp) * w2;
                        let hp2 = w2 - self.dust_lp1r;
                        self.dust_lp2r = a_lp * self.dust_lp2r + (1.0 - a_lp) * hp2;
                        let idx_r = (self.comb_pos + 256 - self.comb_dly_r) & 255;
                        let dr = self.comb_buf_r[idx_r];
                        self.comb_lp_r =
                            self.a_comb_lp * self.comb_lp_r + (1.0 - self.a_comb_lp) * dr;
                        let cw_r = self.dust_lp2r + fb * self.comb_lp_r;
                        self.comb_buf_r[self.comb_pos] = cw_r;
                        let shaped_r =
                            (1.0 - p.bed_comb) * self.dust_lp2r + p.bed_comb * cw_r;
                        let mut smr = shaped_r;
                        for (c, st) in self.bed_bqc.iter().zip(self.bed_bq_r.iter_mut()) {
                            let y = c[0] * smr + st[0];
                            st[0] = c[1] * smr - c[3] * y + st[1];
                            st[1] = c[2] * smr - c[4] * y;
                            smr = y;
                        }
                        yr += p.dust_level * smr * g * self.dust_peak * 0.7;
                    } else {
                        // mono: R comb buffer still advances coherently
                        self.comb_buf_r[self.comb_pos] = cw_l;
                        yr += d_l;
                    }
                    self.comb_pos = (self.comb_pos + 1) & 255;
                }
            }
            // rings join the output last (kept out of every normalizer path)
            yl += ring_l;
            yr += ring_r;
            // bracing choke: early clamp that releases (both channels)
            if p.brace_choke > 0.0 {
                let ch = (-(self.t) * 14.0 * p.brace_choke * (-self.t / 0.05).exp()).exp();
                yl *= ch;
                yr *= ch;
            }
            out_l[i] = yl;
            out_r[i] = yr;
            peak = peak.max(yl.abs()).max(yr.abs());
            self.t += dt;
        }
        if peak < 1e-6 && self.pulse_pos >= self.pulse_len && self.t > 0.25 {
            self.active = false;
        }
        self.active
    }

    /// Energy-audit gesture (full ledger lands with the plugin): sum of
    /// squared mode states — the panel's honesty lamp will watch this.
    pub fn stored_energy(&self) -> f32 {
        self.modes[..self.n_modes]
            .iter()
            .map(|m| m.u * m.u + m.v * m.v)
            .sum()
    }

    pub fn n_modes(&self) -> usize {
        self.n_modes
    }
}

/// Bracing macro → granular params (the shell-side law, Batch 005).
pub fn apply_brace_macro(p: &mut EngineParams, brace: f32) {
    let b = brace.clamp(0.0, 1.0);
    p.brace_coupling = b;
    p.brace_choke = b;
    p.brace_tension = 0.05 * b;
    p.brace_t60 = 1.0 - 0.45 * b;
    p.brace_low_bonus = 1.0 - b;
}

/// Size macro → granular params (the Batch 004c law, canonized in the
/// listening log: "Yup. That's size, alright."). One scalar co-scales:
///   f0 ∝ 1/size · density ↑ · T60 ×size^0.7 · cascade τ ×size^1.3
///   nonlinear drive d = ceil(velocity² / size^1.5)  — susceptibility
///   falls with size; nonlinear commotion is itself a smallness cue.
/// The soft ceiling d = 0.85·tanh(d_raw/0.85) removes the artificiality
/// Sam flagged at drive ≈ 0.9 (M6 spec).
///
/// Algebra note: the engine multiplies BOTH nonlinear mechanisms by
/// velocity² internally (glide: (2^(g/6)−1)·v²·E; cascade: amt·v² in
/// inj and depletion). The macro therefore rescales the PARAMS so the
/// effective drive equals the law without double-counting velocity:
///   cascade_amt' = amt·d/v²        (amt'·v² = amt·d)
///   glide: (2^(g'/6)−1)·v² = (2^(g/6)−1)·d
///     → g' = 6·log2(1 + (2^(g/6)−1)·d/v²)
/// Call AFTER velocity/f0 are final (note+tune+vel-curve applied).
pub fn apply_size_macro(p: &mut EngineParams, size: f32) {
    let s = size.clamp(0.4, 2.5);
    let v2 = (p.velocity * p.velocity).max(1e-6);
    let d_raw = v2 / s.powf(1.5);
    let d = 0.85 * (d_raw / 0.85).tanh();
    p.f0 /= s;
    p.t60_base *= s.powf(0.7);
    p.cascade_tau *= s.powf(1.3);
    // density rises with size: n_axial(1.0) = user's Mode Density,
    // ~×0.73 at size 0.5, ~×1.37 at size 2.0 (hits 8/10/14 from base 10)
    p.n_axial = ((p.n_axial as f32 * s.powf(0.45)).round() as u32).clamp(4, 14);
    let ratio = d / v2;
    p.cascade_amt *= ratio;
    let r2 = (2.0f32).powf(p.glide_st / 6.0);
    p.glide_st = 6.0 * (1.0 + (r2 - 1.0) * ratio).max(1.0).log2();
}

pub fn db(x: f32) -> f32 {
    db_to_lin(x)
}
