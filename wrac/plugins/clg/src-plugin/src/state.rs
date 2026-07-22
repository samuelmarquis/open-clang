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
    PARAM_T60_ID, PARAM_TILT_ID, PARAM_TUNE_ID, PARAM_WIDTH_ID, PARAM_DECOHERE_ID, PARAM_SFLOOR_ID,
    PARAM_SIZE_ID, PARAM_VELCURVE_ID, PARAM_RATTLE_LEVEL_ID, PARAM_MODE_SPREAD_ID,
    PARAM_DAMP_ASYM_ID, PARAM_SUB_ROTATE_ID, PARAM_EXCITER_ID, PARAM_EX_COLOR_ID,
    PARAM_EX_TIME_ID, param_clamp,
    param_default, param_exists,
};

pub(crate) const PARAM_SLOTS: usize = 31;

/// Satellite presets, mirroring the `clg` CLI (`--sats`).
type SatPreset = (
    u32,
    [f32; MAX_SATS],
    [f32; MAX_SATS],
    [f32; MAX_SATS],
    [f32; MAX_SATS],
    [f32; MAX_SATS],
);

const SAT_PRESETS: [SatPreset; 4] = [
    (0, [0.0; 4], [0.1; 4], [0.0; 4], [0.0; 4], [0.0; 4]),
    (
        2,
        [1900.0, 2700.0, 0.0, 0.0],
        [0.10, 0.08, 0.1, 0.1],
        [0.22, 0.61, 0.0, 0.0],
        [0.15, 0.22, 0.0, 0.0],
        [1.0, 0.8, 0.0, 0.0],
    ),
    (
        1,
        [900.0, 0.0, 0.0, 0.0],
        [0.15, 0.1, 0.1, 0.1],
        [0.45, 0.0, 0.0, 0.0],
        [0.55, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ),
    (
        3,
        [1300.0, 2100.0, 3400.0, 0.0],
        [0.12, 0.10, 0.07, 0.1],
        [0.18, 0.52, 0.80, 0.0],
        [0.30, 0.45, 0.20, 0.0],
        [1.0, 0.9, 0.7, 0.0],
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
        let (n, fs, t60s, seats, rests, levels) =
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
            dust_level: self.v(PARAM_DUST_LEVEL_ID),
            width: 0.0, // param 22 deprecated-inert (width killed, PNT 004)
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
                _ => Exciter::Mallet,
            },
            ex_color: self.v(PARAM_EX_COLOR_ID),
            ex_time: self.v(PARAM_EX_TIME_ID),
            dust_thr_db: self.v(PARAM_DUST_THR_ID),
            dust_follow: self.v(PARAM_DUST_FOLLOW_ID),
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
