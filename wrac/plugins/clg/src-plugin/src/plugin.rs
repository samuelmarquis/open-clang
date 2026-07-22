//! The plugin contract as seen by the host. M4 v0 is headless: hosts present
//! their generic parameter editor; the panel (M5) adds the board later.

use std::sync::Arc;

mod audio_ports;
mod params;
mod state;

pub(crate) use params::{
    PARAM_ARCH_ID, PARAM_BRACE_ID, PARAM_BYPASS_ID, PARAM_CASC_AMT_ID, PARAM_CASC_ATTACK_ID,
    PARAM_CASC_COHERENT_ID, PARAM_CASC_CONSERVE_ID, PARAM_CASC_TAU_ID, PARAM_DUST_FOLLOW_ID,
    PARAM_DUST_LEVEL_ID, PARAM_DUST_THR_ID, PARAM_GAIN_ID, PARAM_GLIDE_ID, PARAM_LISTEN_ID,
    PARAM_NAXIAL_ID, PARAM_OUTTILT_ID, PARAM_POSITION_ID, PARAM_SATS_ID, PARAM_STIFFNESS_ID,
    PARAM_T60_ID, PARAM_TILT_ID, PARAM_TUNE_ID, PARAM_WIDTH_ID, PARAM_DECOHERE_ID, PARAM_SFLOOR_ID,
    PARAM_SIZE_ID, PARAM_VELCURVE_ID, PARAM_RATTLE_LEVEL_ID, PARAM_MODE_SPREAD_ID,
    PARAM_DAMP_ASYM_ID, PARAM_SUB_ROTATE_ID, PARAM_EXCITER_ID, PARAM_EX_COLOR_ID,
    PARAM_EX_TIME_ID, param_clamp,
    param_default, param_exists,
    parameter_infos,
};

use audio_ports::ClgAudioPorts;
use params::ClgParamsExtension;
use state::ClgStateExtension;
use wrac_clap_adapter::{
    AaxDescriptor, AaxStemConfig, ActivateContext, Auv2Descriptor, NoteDialects, NotePortInfo,
    PluginAudioPortsExtension, PluginCore, PluginCoreContext, PluginDescriptor, PluginEntry,
    PluginFactory, PluginFeature, PluginLatencyExtension, PluginNotePortsExtension,
    PluginParamsExtension, PluginResult, PluginStateExtension, Processor, Vst3Descriptor,
};

use crate::audio::ClgAudioProcessor;
use crate::state::SharedState;

// Generated from [package.metadata.wrac] in src-plugin/Cargo.toml.
include!(concat!(env!("OUT_DIR"), "/wrac_plugin_products.rs"));

pub(crate) static PLUGIN_ENTRY: ClgEntry = ClgEntry;

pub(crate) struct ClgEntry;

impl PluginEntry for ClgEntry {
    fn plugin_factory(&self) -> Option<&dyn PluginFactory> {
        Some(&CLG_FACTORY)
    }
}

static CLG_FACTORY: ClgFactory = ClgFactory;

struct ClgFactory;

impl PluginFactory for ClgFactory {
    fn plugin_count(&self) -> u32 {
        PLUGIN_DESCRIPTORS.len() as u32
    }

    fn plugin_descriptor(&self, index: u32) -> Option<PluginDescriptor> {
        PLUGIN_DESCRIPTORS.get(index as usize).copied()
    }

    fn create_plugin(
        &self,
        plugin_id: &str,
        context: PluginCoreContext,
    ) -> Option<Box<dyn PluginCore>> {
        PLUGIN_DESCRIPTORS
            .iter()
            .find(|descriptor| descriptor.id == plugin_id)
            .map(|descriptor| create_plugin_core(context, *descriptor))
    }
}

/// One MIDI/CLAP note input: the trigger port of an instrument.
struct ClgNotePorts;

impl PluginNotePortsExtension for ClgNotePorts {
    fn note_port_count(&self, is_input: bool) -> u32 {
        if is_input { 1 } else { 0 }
    }

    fn note_port_info(&self, index: u32, is_input: bool) -> Option<NotePortInfo> {
        (is_input && index == 0).then(|| NotePortInfo {
            id: 0,
            // CLAP dialect ONLY (M4 fix round): declaring the MIDI dialect
            // makes clap-wrapper's VST3 publish IMidiMapping CC proxy
            // parameters (ParamID 0xb00000+, one set per channel) — the
            // stray "MIDI controller" slider Sam saw in Ableton. We consume
            // note events, not CCs; notes still arrive as CLAP NoteOn.
            supported_dialects: NoteDialects::CLAP,
            preferred_dialect: NoteDialects::CLAP,
            name: "Trigger",
        })
    }
}

/// Zero-latency instrument: nothing in the graph needs lookahead.
struct ClgLatency;

impl PluginLatencyExtension for ClgLatency {
    fn latency_frames(&self) -> u32 {
        0
    }
}

pub(crate) struct ClgPlugin {
    descriptor: PluginDescriptor,
    shared: Arc<SharedState>,
    audio_ports: Arc<ClgAudioPorts>,
    params: Arc<ClgParamsExtension>,
    state_extension: Arc<ClgStateExtension>,
    note_ports: Arc<ClgNotePorts>,
    latency: Arc<ClgLatency>,
}

impl ClgPlugin {
    pub(crate) fn new(_context: PluginCoreContext, descriptor: PluginDescriptor) -> Self {
        let shared = Arc::new(SharedState::new());
        let params = Arc::new(ClgParamsExtension::new(shared.clone()));
        let state_extension = Arc::new(ClgStateExtension::new(shared.clone()));

        Self {
            descriptor,
            shared,
            audio_ports: Arc::new(ClgAudioPorts),
            params,
            state_extension,
            note_ports: Arc::new(ClgNotePorts),
            latency: Arc::new(ClgLatency),
        }
    }
}

pub(crate) fn create_plugin_core(
    context: PluginCoreContext,
    descriptor: PluginDescriptor,
) -> Box<dyn PluginCore> {
    wrac_log::init!(descriptor.name);
    log::debug!(
        "creating plugin core: id={}, name={}",
        descriptor.id,
        descriptor.name
    );
    Box::new(ClgPlugin::new(context, descriptor))
}

impl PluginCore for ClgPlugin {
    fn activate(&mut self, context: ActivateContext) -> PluginResult<Box<dyn Processor>> {
        log::debug!(
            "activating: plugin_id={}, sample_rate={}, max_frames={}",
            self.descriptor.id,
            context.sample_rate,
            context.max_frames_count,
        );
        Ok(Box::new(ClgAudioProcessor::new(
            self.shared.clone(),
            context.sample_rate,
            context.max_frames_count,
        )))
    }

    fn deactivate(&mut self, _processor: Box<dyn Processor>) -> PluginResult<()> {
        Ok(())
    }

    fn audio_ports(&self) -> Option<Arc<dyn PluginAudioPortsExtension>> {
        Some(self.audio_ports.clone())
    }

    fn note_ports(&self) -> Option<Arc<dyn PluginNotePortsExtension>> {
        Some(self.note_ports.clone())
    }

    fn params(&self) -> Option<Arc<dyn PluginParamsExtension>> {
        Some(self.params.clone())
    }

    fn state(&self) -> Option<Arc<dyn PluginStateExtension>> {
        Some(self.state_extension.clone())
    }

    fn latency(&self) -> Option<Arc<dyn PluginLatencyExtension>> {
        Some(self.latency.clone())
    }
}
