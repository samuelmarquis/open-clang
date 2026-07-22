//! The host-facing parameter table. Adding a parameter starts here; count,
//! info, conversions, defaults, and persistence all derive from this table.
//!
//! Parameter IDs are host/project ABI. Never renumber an existing id after
//! publishing; add new ids instead.

use std::sync::Arc;

use wrac_clap_adapter::{
    ParamFlags, ParamInfo, ParamInputEvents, PluginError, PluginParamsExtension, PluginResult,
};

use crate::state::SharedState;

pub(crate) const PARAM_ARCH_ID: u32 = 0;
pub(crate) const PARAM_TUNE_ID: u32 = 1;
pub(crate) const PARAM_STIFFNESS_ID: u32 = 2;
pub(crate) const PARAM_T60_ID: u32 = 3;
pub(crate) const PARAM_TILT_ID: u32 = 4;
pub(crate) const PARAM_NAXIAL_ID: u32 = 5;
pub(crate) const PARAM_GLIDE_ID: u32 = 6;
pub(crate) const PARAM_OUTTILT_ID: u32 = 7;
pub(crate) const PARAM_CASC_AMT_ID: u32 = 8;
pub(crate) const PARAM_CASC_TAU_ID: u32 = 9;
pub(crate) const PARAM_CASC_ATTACK_ID: u32 = 10;
pub(crate) const PARAM_CASC_CONSERVE_ID: u32 = 11;
pub(crate) const PARAM_CASC_COHERENT_ID: u32 = 12;
pub(crate) const PARAM_BRACE_ID: u32 = 13;
pub(crate) const PARAM_SATS_ID: u32 = 14;
pub(crate) const PARAM_DUST_LEVEL_ID: u32 = 15;
pub(crate) const PARAM_DUST_THR_ID: u32 = 16;
pub(crate) const PARAM_DUST_FOLLOW_ID: u32 = 17;
pub(crate) const PARAM_POSITION_ID: u32 = 18;
pub(crate) const PARAM_LISTEN_ID: u32 = 19;
pub(crate) const PARAM_GAIN_ID: u32 = 20;
pub(crate) const PARAM_BYPASS_ID: u32 = 21;
pub(crate) const PARAM_WIDTH_ID: u32 = 22;
pub(crate) const PARAM_DECOHERE_ID: u32 = 23;
pub(crate) const PARAM_SFLOOR_ID: u32 = 24;
pub(crate) const PARAM_SIZE_ID: u32 = 25;
pub(crate) const PARAM_VELCURVE_ID: u32 = 26;
pub(crate) const PARAM_RATTLE_LEVEL_ID: u32 = 27;
pub(crate) const PARAM_MODE_SPREAD_ID: u32 = 28;
pub(crate) const PARAM_DAMP_ASYM_ID: u32 = 29;
pub(crate) const PARAM_SUB_ROTATE_ID: u32 = 30;
// M7 — the exciter family (additive ABI, 2026-07-22)
pub(crate) const PARAM_EXCITER_ID: u32 = 31;
pub(crate) const PARAM_EX_COLOR_ID: u32 = 32;
pub(crate) const PARAM_EX_TIME_ID: u32 = 33;

/// How a parameter formats/parses its value text.
#[derive(Debug, Clone, Copy)]
enum Format {
    Percent,
    Seconds,
    Hertz,
    Semitones,
    DbPerOct,
    Db,
    Integer,
    Plain,
    Choice(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy)]
struct ParameterSpec {
    info: ParamInfo,
    format: Format,
}

const fn flags(stepped: bool, is_enum: bool) -> ParamFlags {
    ParamFlags {
        is_stepped: stepped,
        is_periodic: false,
        is_hidden: false,
        is_readonly: false,
        is_bypass: false,
        is_automatable: true,
        is_automatable_per_note_id: false,
        is_automatable_per_key: false,
        is_automatable_per_channel: false,
        is_automatable_per_port: false,
        is_modulatable: false,
        is_modulatable_per_note_id: false,
        is_modulatable_per_key: false,
        is_modulatable_per_channel: false,
        is_modulatable_per_port: false,
        requires_process: false,
        is_enum,
    }
}

const fn continuous(
    id: u32,
    name: &'static str,
    min: f64,
    max: f64,
    default: f64,
    format: Format,
) -> ParameterSpec {
    ParameterSpec {
        info: ParamInfo {
            id,
            name,
            module: "",
            min_value: min,
            max_value: max,
            default_value: default,
            flags: flags(false, false),
        },
        format,
    }
}

const fn stepped(
    id: u32,
    name: &'static str,
    min: f64,
    max: f64,
    default: f64,
    format: Format,
) -> ParameterSpec {
    ParameterSpec {
        info: ParamInfo {
            id,
            name,
            module: "",
            min_value: min,
            max_value: max,
            default_value: default,
            flags: flags(true, false),
        },
        format,
    }
}

const fn choice(
    id: u32,
    name: &'static str,
    names: &'static [&'static str],
    default: f64,
) -> ParameterSpec {
    ParameterSpec {
        info: ParamInfo {
            id,
            name,
            module: "",
            min_value: 0.0,
            max_value: (names.len() - 1) as f64,
            default_value: default,
            flags: flags(true, true),
        },
        format: Format::Choice(names),
    }
}

const fn bypass(id: u32) -> ParameterSpec {
    let mut f = flags(true, true);
    f.is_bypass = true;
    ParameterSpec {
        info: ParamInfo {
            id,
            name: "Bypass",
            module: "",
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
            flags: f,
        },
        format: Format::Choice(OFF_ON),
    }
}

pub(crate) const ARCH_NAMES: &[&str] = &["Membrane", "Plate", "Bar"];
pub(crate) const SATS_NAMES: &[&str] = &["None", "Wires", "Loose", "Trash"];
pub(crate) const EXCITER_NAMES: &[&str] = &["Mallet", "Burst", "Buckling", "Raw"];
const OFF_ON: &[&str] = &["Off", "On"];

const PARAM_SPECS: &[ParameterSpec] = &[
    choice(PARAM_ARCH_ID, "Material", ARCH_NAMES, 0.0),
    continuous(PARAM_TUNE_ID, "Tune", 20.0, 500.0, 36.0, Format::Hertz),
    continuous(PARAM_STIFFNESS_ID, "Strike Stiffness", 0.0, 1.0, 0.55, Format::Percent),
    continuous(PARAM_T60_ID, "Decay", 0.1, 4.0, 1.5, Format::Seconds),
    continuous(PARAM_TILT_ID, "Damping Tilt", 0.3, 3.0, 2.0, Format::Plain),
    stepped(PARAM_NAXIAL_ID, "Mode Density", 4.0, 14.0, 8.0, Format::Integer),
    continuous(PARAM_GLIDE_ID, "Glide", 0.0, 12.0, 8.0, Format::Semitones),
    continuous(PARAM_OUTTILT_ID, "Transect Tilt", -12.0, 6.0, -7.0, Format::DbPerOct),
    continuous(PARAM_CASC_AMT_ID, "Cascade", 0.0, 1.0, 0.0, Format::Percent),
    continuous(PARAM_CASC_TAU_ID, "Cascade Time", 0.01, 0.15, 0.05, Format::Seconds),
    continuous(PARAM_CASC_ATTACK_ID, "Cascade Attack", 0.0, 1.0, 0.0, Format::Percent),
    choice(PARAM_CASC_CONSERVE_ID, "Cascade Conserve", OFF_ON, 1.0),
    choice(PARAM_CASC_COHERENT_ID, "Cascade Coherent", OFF_ON, 1.0), // M4 fix: coherent default (stochastic washes at high tunes)
    continuous(PARAM_BRACE_ID, "Bracing", 0.0, 1.0, 0.0, Format::Percent),
    choice(PARAM_SATS_ID, "Rattle", SATS_NAMES, 0.0),
    continuous(PARAM_DUST_LEVEL_ID, "Dust", 0.0, 1.0, 0.0, Format::Percent),
    continuous(PARAM_DUST_THR_ID, "Dust Threshold", -70.0, 0.0, -40.0, Format::Db),
    continuous(PARAM_DUST_FOLLOW_ID, "Dust Follow", 0.5, 3.0, 1.0, Format::Plain),
    continuous(PARAM_POSITION_ID, "Strike Position", 0.0, 1.0, 0.35, Format::Percent),
    continuous(PARAM_LISTEN_ID, "Listen Position", 0.0, 1.0, 0.31, Format::Percent),
    continuous(PARAM_GAIN_ID, "Output", -24.0, 12.0, 0.0, Format::Db),
    bypass(PARAM_BYPASS_ID),
    // STEREO v1 (additive ABI, 2026-07-22). Width KILLED by verdict
    // 2026-07-22 ("kill width ... all these other controls are far more
    // rewarding") — slot 22 retained for ABI, param inert & renamed; the
    // phase-divergence math lives on inside Sub Rotate. PATHS-NOT-TAKEN 004.
    continuous(PARAM_WIDTH_ID, "(deprecated)", 0.0, 1.0, 0.0, Format::Percent),
    continuous(PARAM_DECOHERE_ID, "Decohere", 0.0, 1.0, 0.0, Format::Percent),
    continuous(PARAM_SFLOOR_ID, "Stereo Floor", 0.0, 1.0, 0.3, Format::Percent),
    // M6 (additive ABI, 2026-07-22): the Size macro (Batch 004c law) +
    // the velocity-response curve (Batch 002 promise)
    continuous(PARAM_SIZE_ID, "Size", 0.4, 2.5, 1.0, Format::Plain),
    continuous(PARAM_VELCURVE_ID, "Vel Curve", 0.25, 4.0, 1.0, Format::Plain),
    // Stereo round 2 (additive ABI, 2026-07-22): per-ear satellites ride
    // the stereo engagement automatically; these are the new prototypes
    continuous(PARAM_RATTLE_LEVEL_ID, "Rattle Level", 0.0, 1.0, 0.5, Format::Percent),
    continuous(PARAM_MODE_SPREAD_ID, "Mode Spread", 0.0, 1.0, 0.0, Format::Percent),
    continuous(PARAM_DAMP_ASYM_ID, "Damp Asym", 0.0, 1.0, 0.0, Format::Percent),
    continuous(PARAM_SUB_ROTATE_ID, "Sub Rotate", 0.0, 1.0, 0.0, Format::Percent),
    // M7 — the exciter family ("acoustic shader" slot; clean-fucked-fidelity
    // doctrine: band-limited force signals, no waveshaping)
    choice(PARAM_EXCITER_ID, "Exciter", EXCITER_NAMES, 0.0),
    continuous(PARAM_EX_COLOR_ID, "Ex Color", 0.0, 1.0, 0.5, Format::Percent),
    continuous(PARAM_EX_TIME_ID, "Ex Time", 0.0, 1.0, 0.5, Format::Percent),
];

fn param_spec(id: u32) -> PluginResult<&'static ParameterSpec> {
    PARAM_SPECS
        .iter()
        .find(|spec| spec.info.id == id)
        .ok_or(PluginError::InvalidParameter)
}

pub(crate) fn param_exists(id: u32) -> bool {
    PARAM_SPECS.iter().any(|spec| spec.info.id == id)
}

pub(crate) fn param_clamp(id: u32, value: f32) -> f32 {
    match param_spec(id) {
        Ok(spec) => value.clamp(spec.info.min_value as f32, spec.info.max_value as f32),
        Err(_) => value,
    }
}

pub(crate) fn param_default(id: u32) -> f32 {
    param_spec(id)
        .map(|s| s.info.default_value as f32)
        .unwrap_or(0.0)
}

pub(crate) fn parameter_infos() -> impl Iterator<Item = ParamInfo> {
    PARAM_SPECS.iter().map(|spec| spec.info)
}

fn value_to_text(spec: &ParameterSpec, value: f64) -> String {
    match spec.format {
        Format::Percent => format!("{:.0} %", value * 100.0),
        Format::Seconds => {
            if value < 1.0 {
                format!("{:.0} ms", value * 1000.0)
            } else {
                format!("{value:.2} s")
            }
        }
        Format::Hertz => format!("{value:.1} Hz"),
        Format::Semitones => format!("{value:.1} st"),
        Format::DbPerOct => format!("{value:.1} dB/oct"),
        Format::Db => format!("{value:.1} dB"),
        Format::Integer => format!("{value:.0}"),
        Format::Plain => format!("{value:.2}"),
        Format::Choice(names) => {
            let idx = (value.round() as usize).min(names.len() - 1);
            names[idx].to_string()
        }
    }
}

fn text_to_plain(spec: &ParameterSpec, text: &str) -> PluginResult<f64> {
    let text = text.trim();
    if let Format::Choice(names) = spec.format {
        if let Some(idx) = names.iter().position(|n| n.eq_ignore_ascii_case(text)) {
            return Ok(idx as f64);
        }
    }
    let numeric: String = text
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
        .collect();
    let mut v: f64 = numeric.parse().map_err(|_| PluginError::InvalidParameter)?;
    match spec.format {
        Format::Percent => v /= 100.0,
        Format::Seconds => {
            if text.contains("ms") {
                v /= 1000.0;
            }
        }
        _ => {}
    }
    Ok(v.clamp(spec.info.min_value, spec.info.max_value))
}

pub(crate) struct ClgParamsExtension {
    shared: Arc<SharedState>,
}

impl ClgParamsExtension {
    pub(crate) fn new(shared: Arc<SharedState>) -> Self {
        Self { shared }
    }
}

impl PluginParamsExtension for ClgParamsExtension {
    fn param_count(&self) -> u32 {
        PARAM_SPECS.len() as u32
    }

    fn param_info(&self, index: u32) -> Option<ParamInfo> {
        PARAM_SPECS.get(index as usize).map(|spec| spec.info)
    }

    fn param_value(&self, param_id: u32) -> PluginResult<f64> {
        param_spec(param_id)?;
        self.shared
            .parameter_value(param_id)
            .map(f64::from)
            .ok_or(PluginError::InvalidParameter)
    }

    fn apply_param_events(&self, events: ParamInputEvents<'_>) -> PluginResult<()> {
        for event in events.values() {
            if self
                .shared
                .set_parameter_value(event.param_id, event.value)
                .is_none()
            {
                wrac_log::rtwarn!(
                    "params.flush: ignoring unknown parameter id={} value={}",
                    event.param_id,
                    event.value
                );
            }
        }
        Ok(())
    }

    fn value_to_text(&self, param_id: u32, value: f64) -> PluginResult<String> {
        Ok(value_to_text(param_spec(param_id)?, value))
    }

    fn text_to_value(&self, param_id: u32, text: &str) -> PluginResult<f64> {
        text_to_plain(param_spec(param_id)?, text)
    }
}
