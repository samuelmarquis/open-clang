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
    /// 0..1: noise spectral center, dark (~0.7-4 kHz) to bright
    /// (~3.3-10 kHz); 0.5 = the legacy 1.5-6.5 kHz band.
    pub bed_bright: f32,
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
    a_comb_lp: f32,     // ~8 kHz (new())
    bed_sm: [f32; 6],   // bed radiation smoother 6x one-pole @9.5 kHz
    bed_smr: [f32; 6],  // (new-path only; with ZOH noise, keeps >=60 dB gate)
    a_bsm: f32,
    bed_hold_l: f32,    // zero-order-hold noise (2-sample): sinc rolloff
    bed_hold_r: f32,    // buys ~19 dB at 0.45 sr for ~3 dB at the top of
    bed_hold_ph: bool,  // the bright band — finite wire bandwidth, honestly
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
}

fn db_to_lin(db: f32) -> f32 {
    (10.0f32).powf(db / 20.0)
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
        let a_comb_lp = (-2.0 * core::f32::consts::PI * 8000.0 / sr).exp();
        let a_bsm = (-2.0 * core::f32::consts::PI * 9_500.0 / sr).exp();
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
            bed_sm: [0.0; 6],
            bed_smr: [0.0; 6],
            a_bsm,
            bed_hold_l: 0.0,
            bed_hold_r: 0.0,
            bed_hold_ph: false,
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
            // release 30 ms -> 2.5 s, log across bed_release
            let rel_t = 0.030 * (83.333f32).powf(p.bed_release.clamp(0.0, 1.0));
            self.bed_a_rel = (-1.0 / (rel_t * sr)).exp();
            // brightness: 0.5 = the exact legacy band (bit-identity)
            let b = p.bed_bright.clamp(0.0, 1.0);
            if (b - 0.5).abs() < 1e-6 {
                self.bed_a_hp = self.a_hp;
                self.bed_a_lp = self.a_lp;
            } else {
                let hp_cut = 1500.0 * (2.2f32).powf(2.0 * b - 1.0);
                let lp_cut = 6500.0 * (1.55f32).powf(2.0 * b - 1.0);
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
            self.bed_sm = [0.0; 6];
            self.bed_smr = [0.0; 6];
            self.bed_hold_l = 0.0;
            self.bed_hold_r = 0.0;
            self.bed_hold_ph = false;
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
            for (k, m) in self.modes[..nm].iter_mut().enumerate() {
                let base_drive = m.amp * f_in + m.inj * casc_noise;
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
                    let src = (1.0 - p.bed_source) * yl.abs()
                        + p.bed_source * self.bed_src_s2.abs() * 2.0;
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
                    // ZOH noise: hold white for 2 samples (sinc rolloff —
                    // the cheap 19 dB toward the band-limit gate)
                    if !self.bed_hold_ph {
                        self.bed_hold_l = self.white();
                        if stereo {
                            self.bed_hold_r = self.white();
                        }
                    }
                    self.bed_hold_ph = !self.bed_hold_ph;
                    // L chain: bright-banded noise -> wire comb -> smoother
                    let w = self.bed_hold_l;
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
                    for s in self.bed_sm.iter_mut() {
                        *s = self.a_bsm * *s + (1.0 - self.a_bsm) * sm;
                        sm = *s;
                    }
                    let d_l = p.dust_level * sm * g * self.dust_peak * 0.7;
                    yl += d_l;
                    if stereo {
                        let w2 = self.bed_hold_r;
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
                        for s in self.bed_smr.iter_mut() {
                            *s = self.a_bsm * *s + (1.0 - self.a_bsm) * smr;
                            smr = *s;
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
