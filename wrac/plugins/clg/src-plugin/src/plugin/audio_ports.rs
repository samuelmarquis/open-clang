//! Instrument audio layout: no inputs, one fixed stereo main output.

use wrac_clap_adapter::{AudioPortFlags, AudioPortInfo, AudioPortType, PluginAudioPortsExtension};

pub(super) struct ClgAudioPorts;

impl PluginAudioPortsExtension for ClgAudioPorts {
    fn audio_port_count(&self, is_input: bool) -> u32 {
        if is_input { 0 } else { 1 }
    }

    fn audio_port_info(&self, index: u32, is_input: bool) -> Option<AudioPortInfo> {
        (!is_input && index == 0).then_some(AudioPortInfo {
            id: 1,
            name: "Main Out",
            flags: AudioPortFlags {
                is_main: true,
                ..AudioPortFlags::default()
            },
            channel_count: 2,
            port_type: AudioPortType::Stereo,
            in_place_pair: None,
        })
    }
}
