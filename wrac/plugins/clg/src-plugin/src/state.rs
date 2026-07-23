//! Parameter state shared by the audio thread and host.
//!
//! One atomic per parameter, indexed by the spec table in `plugin/params.rs`.
//! The audio thread builds a full [`clg_engine::EngineParams`] snapshot at
//! each note-on without taking any lock.
//!
//! // M5: viz — the engine's per-voice analysis feed (modal transect, energy
//! ledger, contact ticks) gets a queue here when the panel lands.

use atomic_float::AtomicF32;
use std::sync::atomic::Ordering;

use clg_engine::{Exciter, apply_brace_macro, apply_size_macro, Arch, EngineParams, MAX_SATS};

use crate::plugin::{
    PARAM_ARCH_ID, PARAM_BRACE_ID, PARAM_CASC_AMT_ID, PARAM_CASC_ATTACK_ID,
    PARAM_CASC_COHERENT_ID, PARAM_CASC_CONSERVE_ID, PARAM_CASC_TAU_ID, PARAM_DUST_FOLLOW_ID,
    PARAM_DUST_LEVEL_ID, PARAM_DUST_THR_ID, PARAM_GAIN_ID, PARAM_GLIDE_ID, PARAM_LISTEN_ID,
    PARAM_NAXIAL_ID, PARAM_OUTTILT_ID, PARAM_POSITION_ID, PARAM_SATS_ID, PARAM_STIFFNESS_ID,
    PARAM_T60_ID, PARAM_TILT_ID, PARAM_TUNE_ID, PARAM_DECOHERE_ID, PARAM_SFLOOR_ID,
    PARAM_SIZE_ID, PARAM_VELCURVE_ID, PARAM_RATTLE_LEVEL_ID, PARAM_MODE_SPREAD_ID,
    PARAM_DAMP_ASYM_ID, PARAM_SUB_ROTATE_ID, PARAM_EXCITER_ID, PARAM_EX_COLOR_ID,
    PARAM_EX_TIME_ID, PARAM_RATTLE_CASC_ID, PARAM_BOUNCE_ID, PARAM_RATTLE_GAP_ID,
    PARAM_GAP_VEL_ID, PARAM_RATTLE_TUNE_ID, PARAM_RATTLE_TRACK_ID, PARAM_WALK_ID,
    PARAM_BED_RELEASE_ID, PARAM_BED_SOURCE_ID, PARAM_BED_COMB_ID, PARAM_BED_BRIGHT_ID,
    PARAM_CAVITY_ID, PARAM_CAVITY_TUNE_ID, PARAM_HEAD2_TUNE_ID, PARAM_HEAD2_DAMP_ID,
    PARAM_WIRES_ID, PARAM_WIRE_TUNE_ID, PARAM_WIRE_DECAY_ID, PARAM_WIRE_THROW_ID,
    PARAM_ROOT_WEIGHT_ID,
    PARAM_NET_ID, PARAM_NET_DENSITY_ID, PARAM_NET_TENSION_ID, PARAM_NET_TUNE_ID,
    param_clamp,
    param_default, param_exists,
};

// Derived from the param table — never hand-sized again (M7 lesson).
pub(crate) const PARAM_SLOTS: usize = crate::plugin::PARAM_COUNT;

/// Satellite presets, mirroring the `clg` CLI (`--sats`). M8: each
/// satellite is a small modal OBJECT (partial ratio/amp sets) — the
/// round's one deliberate baseline sound change.
type SatPreset = (
    u32,
    [f32; MAX_SATS],
    [f32; MAX_SATS],
    [f32; MAX_SATS],
    [f32; MAX_SATS],
    [f32; MAX_SATS],
    [[f32; 4]; MAX_SATS], // partial freq ratios
    [[f32; 4]; MAX_SATS], // partial amplitudes
);

const SAT_PRESETS: [SatPreset; 4] = [
    (
        0,
        [0.0; 4],
        [0.1; 4],
        [0.0; 4],
        [0.0; 4],
        [0.0; 4],
        [[1.0, 0.0, 0.0, 0.0]; 4],
        [[1.0, 0.0, 0.0, 0.0]; 4],
    ),
    (
        2,
        [1900.0, 2700.0, 0.0, 0.0],
        [0.10, 0.08, 0.1, 0.1],
        [0.22, 0.61, 0.0, 0.0],
        [0.15, 0.22, 0.0, 0.0],
        [1.0, 0.8, 0.0, 0.0],
        // bright inharmonic wire sets
        [[1.0, 1.53, 2.31, 0.0], [1.0, 1.71, 2.63, 0.0],
         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
        [[1.0, 0.6, 0.35, 0.0], [1.0, 0.55, 0.3, 0.0],
         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
    ),
    (
        1,
        [900.0, 0.0, 0.0, 0.0],
        [0.15, 0.1, 0.1, 0.1],
        [0.45, 0.0, 0.0, 0.0],
        [0.55, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        // dull knocker + one overtone
        [[1.0, 2.7, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0],
         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
        [[1.0, 0.4, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0],
         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
    ),
    (
        3,
        [1300.0, 2100.0, 3400.0, 0.0],
        [0.12, 0.10, 0.07, 0.1],
        [0.18, 0.52, 0.80, 0.0],
        [0.30, 0.45, 0.20, 0.0],
        [1.0, 0.9, 0.7, 0.0],
        // clattery junk: spread partial sets
        [[1.0, 1.34, 1.83, 2.51], [1.0, 1.47, 2.06, 2.9],
         [1.0, 1.62, 2.24, 0.0], [1.0, 0.0, 0.0, 0.0]],
        [[1.0, 0.7, 0.5, 0.35], [1.0, 0.65, 0.45, 0.3],
         [1.0, 0.6, 0.4, 0.0], [1.0, 0.0, 0.0, 0.0]],
    ),
];

pub(crate) struct SharedState {
    values: [AtomicF32; PARAM_SLOTS],
}

impl SharedState {
    pub(crate) fn new() -> Self {
        let values = std::array::from_fn(|i| AtomicF32::new(param_default(i as u32)));
        Self { values }
    }

    /// Clamp + store. Returns the applied value, or None for unknown ids.
    pub(crate) fn set_parameter_value(&self, id: u32, plain: f64) -> Option<f32> {
        if !param_exists(id) {
            return None;
        }
        let v = param_clamp(id, plain as f32);
        self.values[id as usize].store(v, Ordering::Relaxed);
        Some(v)
    }

    pub(crate) fn parameter_value(&self, id: u32) -> Option<f32> {
        param_exists(id).then(|| self.values[id as usize].load(Ordering::Relaxed))
    }

    fn v(&self, id: u32) -> f32 {
        self.values[id as usize].load(Ordering::Relaxed)
    }

    pub(crate) fn output_gain(&self) -> f32 {
        if self.v(crate::plugin::PARAM_BYPASS_ID) >= 0.5 {
            return 0.0; // instrument bypass = silence; voices keep decaying
        }
        (10.0f32).powf(self.v(PARAM_GAIN_ID) / 20.0)
    }

    /// Snapshot for one voice at note-on. `key`/`velocity` come from the
    /// triggering note: f0 = tune x 2^((key-60)/12), C3 plays Tune as-is.
    pub(crate) fn engine_params_for_note(&self, key: i16, velocity: f32) -> EngineParams {
        let tune = self.v(PARAM_TUNE_ID);
        let f0 = tune * (2.0f32).powf((key as f32 - 60.0) / 12.0);
        // Vel Curve (Batch 002 promise): the exposed velocity-response ladder
        let velocity = velocity.clamp(0.0, 1.0).powf(self.v(PARAM_VELCURVE_ID));
        let (n, fs, t60s, seats, rests, levels, pr, pa) =
            SAT_PRESETS[(self.v(PARAM_SATS_ID).round() as usize).min(3)];
        let mut p = EngineParams {
            arch: match self.v(PARAM_ARCH_ID).round() as usize {
                1 => Arch::Plate,
                2 => Arch::Bar,
                _ => Arch::Membrane,
            },
            f0: f0.clamp(16.0, 4000.0),
            velocity: velocity.clamp(0.02, 1.0),
            position: self.v(PARAM_POSITION_ID),
            listen_pos: self.v(PARAM_LISTEN_ID),
            stiffness: self.v(PARAM_STIFFNESS_ID),
            t60_base: self.v(PARAM_T60_ID),
            tilt: self.v(PARAM_TILT_ID),
            n_axial: self.v(PARAM_NAXIAL_ID).round() as u32,
            glide_st: self.v(PARAM_GLIDE_ID),
            out_tilt_db_oct: self.v(PARAM_OUTTILT_ID),
            cascade_amt: self.v(PARAM_CASC_AMT_ID),
            cascade_tau: self.v(PARAM_CASC_TAU_ID),
            cascade_attack: self.v(PARAM_CASC_ATTACK_ID),
            cascade_conserve: self.v(PARAM_CASC_CONSERVE_ID) >= 0.5,
            cascade_coherent: self.v(PARAM_CASC_COHERENT_ID) >= 0.5,
            sat_count: n,
            sat_fs: fs,
            sat_t60: t60s,
            sat_seat: seats,
            sat_rest: rests,
            sat_level: levels,
            sat_pr: pr,
            sat_pa: pa,
            rattle_casc: self.v(PARAM_RATTLE_CASC_ID),
            bounce: self.v(PARAM_BOUNCE_ID),
            rattle_gap: self.v(PARAM_RATTLE_GAP_ID),
            gap_vel: self.v(PARAM_GAP_VEL_ID),
            rattle_tune: self.v(PARAM_RATTLE_TUNE_ID) / 12.0, // st -> octaves
            rattle_track: self.v(PARAM_RATTLE_TRACK_ID),
            walk: self.v(PARAM_WALK_ID),
            dust_level: self.v(PARAM_DUST_LEVEL_ID),
            decohere: self.v(PARAM_DECOHERE_ID),
            stereo_floor: self.v(PARAM_SFLOOR_ID),
            rattle_level: self.v(PARAM_RATTLE_LEVEL_ID),
            mode_spread: self.v(PARAM_MODE_SPREAD_ID),
            damp_asym: self.v(PARAM_DAMP_ASYM_ID),
            sub_rotate: self.v(PARAM_SUB_ROTATE_ID),
            exciter: match self.v(PARAM_EXCITER_ID).round() as usize {
                1 => Exciter::Burst,
                2 => Exciter::Buckling,
                3 => Exciter::Raw,
                4 => Exciter::Stick,
                _ => Exciter::Mallet,
            },
            ex_color: self.v(PARAM_EX_COLOR_ID),
            ex_time: self.v(PARAM_EX_TIME_ID),
            dust_thr_db: self.v(PARAM_DUST_THR_ID),
            dust_follow: self.v(PARAM_DUST_FOLLOW_ID),
            bed_release: self.v(PARAM_BED_RELEASE_ID),
            bed_source: self.v(PARAM_BED_SOURCE_ID),
            bed_comb: self.v(PARAM_BED_COMB_ID),
            bed_bright: self.v(PARAM_BED_BRIGHT_ID),
            cavity: self.v(PARAM_CAVITY_ID),
            cavity_tune: self.v(PARAM_CAVITY_TUNE_ID),
            head2_tune: self.v(PARAM_HEAD2_TUNE_ID),
            head2_damp: self.v(PARAM_HEAD2_DAMP_ID),
            wires: self.v(PARAM_WIRES_ID),
            wire_tune: self.v(PARAM_WIRE_TUNE_ID),
            wire_decay: self.v(PARAM_WIRE_DECAY_ID),
            wire_throw: self.v(PARAM_WIRE_THROW_ID),
            root_weight: self.v(PARAM_ROOT_WEIGHT_ID),
            net: self.v(PARAM_NET_ID),
            net_density: self.v(PARAM_NET_DENSITY_ID),
            net_tension: self.v(PARAM_NET_TENSION_ID),
            net_tune: self.v(PARAM_NET_TUNE_ID),
            ..EngineParams::default()
        };
        apply_brace_macro(&mut p, self.v(PARAM_BRACE_ID));
        // Size AFTER brace and after note/tune/vel-curve: the macro reads
        // final velocity and scales f0/T60/tau/density/drive per the 004c law
        apply_size_macro(&mut p, self.v(PARAM_SIZE_ID));
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M4 fix round: replicate the host param path exactly (set via the
    /// same entry the flush/process events use), then render through the
    /// engine — dust on/off must differ audibly (Sam: "no difference").
    /// The M7 no-sound bug, made structurally impossible: every param id
    /// must index inside the atomic store, and ids must be dense 0..COUNT
    /// (the store is indexed by id directly).
    #[test]
    fn param_table_matches_store() {
        for id in 0..crate::plugin::PARAM_COUNT as u32 {
            assert!(
                crate::plugin::param_exists(id),
                "param id {id} missing: table not dense"
            );
        }
        assert!(!crate::plugin::param_exists(crate::plugin::PARAM_COUNT as u32));
    }

    #[test]
    /// M8 fuzz-hang reproducer: random params + random keys through the
    /// host path, many retriggers, wall-clock guarded. The clap-validator
    /// param-fuzz-basic wedge must be reproducible here or it's wrapper-side.
    #[test]
    fn param_fuzz_stress_host_path() {
        use std::time::Instant;
        let sr = 44100.0f32;
        let s = SharedState::new();
        let mut rng: u32 = 0x1234_5678;
        let mut next = || {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            (rng as f64) / (u32::MAX as f64)
        };
        let mut engines: Vec<clg_engine::Engine> =
            (0..8).map(|_| clg_engine::Engine::new(sr)).collect();
        let mut last_p: Vec<Option<clg_engine::EngineParams>> = vec![None; 8];
        let t0 = Instant::now();
        let mut l = [0.0f32; 256];
        let mut r = [0.0f32; 256];
        for cycle in 0..300 {
            // fuzz every param to a random in-range value
            for id in 0..crate::plugin::PARAM_COUNT as u32 {
                let v = next();
                // set_parameter_value clamps to the spec range from any input
                let _ = s.set_parameter_value(id, -100.0 + 300.0 * v);
            }
            let key = (next() * 127.0) as i16;
            let vel = next() as f32;
            let p = s.engine_params_for_note(key, vel.max(0.02));
            last_p[cycle % 8] = Some(p);
            engines[cycle % 8].trigger(&p);
            for (ei, e) in engines.iter_mut().enumerate() {
                for blk in 0..4 {
                    e.process(&mut l, &mut r);
                    // the validator's pass condition: EVERY sample finite
                    if !l.iter().chain(r.iter()).all(|x| x.is_finite()) {
                        panic!(
                            "non-finite output: cycle {cycle} voice {ei} blk {blk}\nPARAMS: {:#?}",
                            last_p[ei]
                        );
                    }
                }
            }
            assert!(
                t0.elapsed().as_secs() < 30,
                "WEDGE reproduced at cycle {cycle}"
            );
        }
    }

    /// M10 gate: the cavity topology must (a) be audible through the
    /// host path, and (b) DECAY — coupled bidirectional banks are where
    /// limit cycles live (the satellite-drone lesson). Worst case is
    /// frequency coincidence: body tuned onto the cavity, ringing head.
    #[test]
    fn cavity_reaches_engine_and_decays() {
        let sr = 44100.0f32;
        let render = |cav: f32| -> Vec<f32> {
            let s = SharedState::new();
            s.set_parameter_value(PARAM_CAVITY_ID, cav as f64).unwrap();
            s.set_parameter_value(PARAM_CAVITY_TUNE_ID, 170.0).unwrap();
            s.set_parameter_value(PARAM_HEAD2_TUNE_ID, 0.0).unwrap();
            s.set_parameter_value(PARAM_HEAD2_DAMP_ID, 0.0).unwrap();
            s.set_parameter_value(super::super::plugin::PARAM_TUNE_ID as u32, 170.0)
                .unwrap();
            let p = s.engine_params_for_note(60, 0.95);
            assert!((p.cavity - cav).abs() < 1e-6, "cavity lost: {}", p.cavity);
            let mut e = clg_engine::Engine::new(sr);
            e.trigger(&p);
            let mut out = vec![0.0f32; (sr * 6.0) as usize];
            let mut r = [0.0f32; 256];
            for c in out.chunks_mut(256) {
                let n = c.len();
                e.process(c, &mut r[..n]);
            }
            out
        };
        let off = render(0.0);
        let on = render(1.0);
        let rms = |x: &[f32]| (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt();
        let d: Vec<f32> = on.iter().zip(&off).map(|(a, b)| a - b).collect();
        assert!(
            rms(&d) > rms(&off) * 0.01,
            "cavity 1.0 inaudible through the host path"
        );
        // decay gate: last 0.5 s must sit >=40 dB under the first 0.5 s
        let n5 = (sr * 0.5) as usize;
        let head = rms(&on[..n5]);
        let tail = rms(&on[on.len() - n5..]);
        assert!(
            tail < head * 0.01,
            "cavity limit cycle: head {head} tail {tail}"
        );
        assert!(on.iter().all(|x| x.is_finite()));
    }

    /// M11 gate: the wire bank must (a) be audible through the host
    /// path, and (b) DECAY — the wire⇄head contact loop is new coupled
    /// territory (unilateral reactions rectify into pumps; the M8 law).
    /// Both topologies: cavity ON (wires on R2) and OFF (wires on the
    /// batter), ringing head, full throw.
    #[test]
    fn wires_reach_engine_and_decay() {
        let sr = 44100.0f32;
        let render = |wires: f32, cav: f32| -> Vec<f32> {
            let s = SharedState::new();
            s.set_parameter_value(PARAM_WIRES_ID, wires as f64).unwrap();
            s.set_parameter_value(PARAM_WIRE_THROW_ID, 1.0).unwrap();
            s.set_parameter_value(PARAM_CAVITY_ID, cav as f64).unwrap();
            s.set_parameter_value(PARAM_HEAD2_DAMP_ID, 0.0).unwrap();
            s.set_parameter_value(PARAM_ROOT_WEIGHT_ID, 1.0).unwrap();
            let p = s.engine_params_for_note(60, 0.95);
            assert!((p.wires - wires).abs() < 1e-6, "wires lost: {}", p.wires);
            let mut e = clg_engine::Engine::new(sr);
            e.trigger(&p);
            let mut out = vec![0.0f32; (sr * 6.0) as usize];
            let mut r = [0.0f32; 256];
            for c in out.chunks_mut(256) {
                let n = c.len();
                e.process(c, &mut r[..n]);
            }
            out
        };
        let rms = |x: &[f32]| (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt();
        for cav in [0.0f32, 1.0] {
            let off = render(0.0, cav);
            let on = render(1.0, cav);
            let d: Vec<f32> = on.iter().zip(&off).map(|(a, b)| a - b).collect();
            assert!(
                rms(&d) > rms(&off) * 0.01,
                "wires 1.0 inaudible through the host path (cavity {cav})"
            );
            let n5 = (sr * 0.5) as usize;
            let head = rms(&on[..n5]);
            let tail = rms(&on[on.len() - n5..]);
            assert!(
                tail < head * 0.01,
                "wire limit cycle (cavity {cav}): head {head} tail {tail}"
            );
            assert!(on.iter().all(|x| x.is_finite()));
        }
    }

    /// M13 gate: the fittings network must (a) be audible through the
    /// host path (bar showcase — the whole point of the round), and
    /// (b) DECAY — new contact machinery answers to the decay gate
    /// before anything else (the M12.1 standing lesson).
    #[test]
    fn net_reaches_engine_and_decays() {
        let sr = 44100.0f32;
        let render = |net: f32, tension: f32| -> Vec<f32> {
            let s = SharedState::new();
            s.set_parameter_value(PARAM_ARCH_ID, 2.0).unwrap(); // Bar
            s.set_parameter_value(PARAM_NET_ID, net as f64).unwrap();
            s.set_parameter_value(PARAM_NET_DENSITY_ID, 1.0).unwrap();
            s.set_parameter_value(PARAM_NET_TENSION_ID, tension as f64).unwrap();
            let p = s.engine_params_for_note(60, 0.95);
            assert!((p.net - net).abs() < 1e-6, "net lost: {}", p.net);
            let mut e = clg_engine::Engine::new(sr);
            e.trigger(&p);
            let mut out = vec![0.0f32; (sr * 6.0) as usize];
            let mut r = [0.0f32; 256];
            for c in out.chunks_mut(256) {
                let n = c.len();
                e.process(c, &mut r[..n]);
            }
            assert_eq!(e.airbag_trips(), 0);
            assert_eq!(e.watchdog_kills(), 0, "watchdog on a healthy net patch");
            out
        };
        let rms = |x: &[f32]| (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt();
        for tension in [0.1f32, 0.9] {
            let off = render(0.0, tension);
            let on = render(1.0, tension);
            let d: Vec<f32> = on.iter().zip(&off).map(|(a, b)| a - b).collect();
            assert!(
                rms(&d) > rms(&off) * 0.01,
                "net 1.0 inaudible through the host path (tension {tension})"
            );
            let n5 = (sr * 0.5) as usize;
            let head = rms(&on[..n5]);
            let tail = rms(&on[on.len() - n5..]);
            assert!(
                tail < head * 0.01,
                "net limit cycle (tension {tension}): head {head} tail {tail}"
            );
            assert!(on.iter().all(|x| x.is_finite()));
        }
    }

    #[test]
    fn dust_reaches_engine_via_host_path() {
        let sr = 44100.0f32;
        let render = |dust: f32| -> Vec<f32> {
            let s = SharedState::new();
            s.set_parameter_value(PARAM_DUST_LEVEL_ID, dust as f64).unwrap();
            s.set_parameter_value(PARAM_DUST_THR_ID, -40.0).unwrap();
            s.set_parameter_value(PARAM_DUST_FOLLOW_ID, 1.0).unwrap();
            let p = s.engine_params_for_note(60, 0.95);
            assert!((p.dust_level - dust).abs() < 1e-6, "dust_level lost: {}", p.dust_level);
            let mut e = clg_engine::Engine::new(sr);
            e.trigger(&p);
            let mut out = vec![0.0f32; (sr * 1.0) as usize];
            let mut r = [0.0f32; 256];
            for c in out.chunks_mut(256) {
                let n = c.len();
                e.process(c, &mut r[..n]);
            }
            out
        };
        let a = render(0.0);
        let b = render(1.0);
        // crude HF proxy: first-difference energy in 50-300 ms
        let hf = |x: &[f32]| -> f32 {
            let seg = &x[2205..13230];
            let d: f32 = seg.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
            d.sqrt()
        };
        let (ha, hb) = (hf(&a), hf(&b));
        assert!(
            hb > ha * 2.0,
            "dust on ({hb}) not audibly above dust off ({ha}) through the host path"
        );
    }
}
