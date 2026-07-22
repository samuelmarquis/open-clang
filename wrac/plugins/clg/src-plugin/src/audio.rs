//! The audio-thread processor: an 8-voice pool over `clg_engine::Engine`.
//!
//! Note-on triggers a voice (round-robin over inactive voices; if all are
//! active, steal the quietest by stored energy). Voices are one-shots —
//! note-off is ignored (drums decay on their own; choke arrives with the
//! MPE work). Output is true stereo (STEREO v1: per-mode L/R decoherence
//! in the engine); with Width/Decohere at 0 both channels are the
//! canonical mono voice, bit-identical.

use std::any::Any;
use std::sync::Arc;

use clg_engine::Engine;
use wrac_clap_adapter::{
    AudioPortChannels, InputEvent, PluginResult, ProcessContext, ProcessStatus, Processor,
};

use crate::state::SharedState;

const VOICES: usize = 8;
/// Engine::process takes any block; we render in chunks of this size into
/// the mix scratch.
const CHUNK: usize = 256;
/// Cap on note-ons handled per block (excess notes trigger at the offset of
/// the last accepted one — graceful, not silent).
const MAX_BLOCK_NOTES: usize = 64;

pub(crate) struct ClgAudioProcessor {
    shared: Arc<SharedState>,
    voices: Vec<Engine>,
    active: [bool; VOICES],
    next: usize,
    max_frames: usize,
    mix_l: Vec<f32>,
    mix_r: Vec<f32>,
    chunk_l: [f32; CHUNK],
    chunk_r: [f32; CHUNK],
}

impl ClgAudioProcessor {
    pub(crate) fn new(shared: Arc<SharedState>, sample_rate: f64, max_frames: u32) -> Self {
        let max_frames = max_frames as usize;
        Self {
            shared,
            voices: (0..VOICES).map(|_| Engine::new(sample_rate as f32)).collect(),
            active: [false; VOICES],
            next: 0,
            max_frames,
            mix_l: vec![0.0; max_frames],
            mix_r: vec![0.0; max_frames],
            chunk_l: [0.0; CHUNK],
            chunk_r: [0.0; CHUNK],
        }
    }

    fn note_on(&mut self, key: i16, velocity: f32) {
        if !(0..128).contains(&key) {
            return;
        }
        // round-robin over free voices; else steal the quietest
        let slot = (0..VOICES)
            .map(|i| (self.next + i) % VOICES)
            .find(|&i| !self.active[i])
            .unwrap_or_else(|| {
                (0..VOICES)
                    .min_by(|&a, &b| {
                        self.voices[a]
                            .stored_energy()
                            .partial_cmp(&self.voices[b].stored_energy())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(0)
            });
        let params = self.shared.engine_params_for_note(key, velocity);
        self.voices[slot].trigger(&params);
        self.active[slot] = true;
        self.next = (slot + 1) % VOICES;
    }

    /// Mix all active voices into mix_l/mix_r over [from, to).
    fn render_span(&mut self, from: usize, to: usize) {
        for vi in 0..VOICES {
            if !self.active[vi] {
                continue;
            }
            let mut live = false;
            let mut off = from;
            while off < to {
                let n = CHUNK.min(to - off);
                live = self.voices[vi]
                    .process(&mut self.chunk_l[..n], &mut self.chunk_r[..n]);
                for (m, c) in self.mix_l[off..off + n].iter_mut().zip(&self.chunk_l[..n]) {
                    *m += c;
                }
                for (m, c) in self.mix_r[off..off + n].iter_mut().zip(&self.chunk_r[..n]) {
                    *m += c;
                }
                off += n;
            }
            self.active[vi] = live;
        }
    }
}

impl Processor for ClgAudioProcessor {
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }

    fn process(&mut self, mut context: ProcessContext<'_>) -> PluginResult<ProcessStatus> {
        let frames = (context.frames_count as usize).min(self.max_frames);

        // 1) Events. Param events apply immediately (block-granular is fine
        //    for knobs); note-ons are collected WITH their sample offsets —
        //    M6: hits land sample-accurately via segmented rendering below.
        let mut notes: [(usize, i16, f32); MAX_BLOCK_NOTES] = [(0, 0, 0.0); MAX_BLOCK_NOTES];
        let mut n_notes = 0usize;
        for event in context.events.input.iter() {
            match event {
                InputEvent::NoteOn(e) => {
                    if n_notes < MAX_BLOCK_NOTES {
                        notes[n_notes] =
                            ((e.time as usize).min(frames), e.key, e.velocity as f32);
                        n_notes += 1;
                    }
                }
                InputEvent::Midi(e) => {
                    let status = e.data[0] & 0xF0;
                    if status == 0x90 && e.data[2] > 0 && n_notes < MAX_BLOCK_NOTES {
                        notes[n_notes] = (
                            (e.time as usize).min(frames),
                            e.data[1] as i16,
                            e.data[2] as f32 / 127.0,
                        );
                        n_notes += 1;
                    }
                }
                InputEvent::ParamValue(e) => {
                    let _ = self.shared.set_parameter_value(e.param_id, e.value);
                }
                _ => {}
            }
        }
        // hosts deliver sorted events; insertion-sort the small array anyway
        for i in 1..n_notes {
            let mut j = i;
            while j > 0 && notes[j - 1].0 > notes[j].0 {
                notes.swap(j - 1, j);
                j -= 1;
            }
        }

        let gain = self.shared.output_gain();

        // 2) Segmented render: [seg start .. next note offset) for all
        //    voices, then trigger, then continue — sample-accurate onsets.
        self.mix_l[..frames].fill(0.0);
        self.mix_r[..frames].fill(0.0);
        let mut seg = 0usize;
        let mut ni = 0usize;
        while seg < frames || ni < n_notes {
            while ni < n_notes && notes[ni].0 <= seg {
                let (_, key, vel) = notes[ni];
                self.note_on(key, vel);
                ni += 1;
            }
            let seg_end = if ni < n_notes {
                notes[ni].0.min(frames)
            } else {
                frames
            };
            if seg_end <= seg {
                if ni >= n_notes {
                    break;
                }
                continue;
            }
            self.render_span(seg, seg_end);
            seg = seg_end;
        }
        // Engine output is at internal amplitude scale; normalize toward
        // sensible host level (calibrated: single full-velocity kick peaks
        // ~0.5 at unity Output).
        let norm = gain * 0.005;

        // M5: viz — publish per-voice VizFrames here once the panel exists.

        // 3) Write stereo: even channels take L, odd take R.
        let Some(mut port) = context.audio.port_pair(0) else {
            return Ok(ProcessStatus::Continue);
        };
        match port.channels()? {
            AudioPortChannels::F32(mut chans) => {
                for ci in 0..chans.channel_pair_count() {
                    if let Some(mut pair) = chans.channel_pair(ci) {
                        if let Some(output) = pair.output_mut() {
                            let src = if ci % 2 == 0 { &self.mix_l } else { &self.mix_r };
                            for (o, m) in output[..frames].iter_mut().zip(&src[..frames]) {
                                *o = *m * norm;
                            }
                        }
                    }
                }
            }
            AudioPortChannels::F64(mut chans) => {
                for ci in 0..chans.channel_pair_count() {
                    if let Some(mut pair) = chans.channel_pair(ci) {
                        if let Some(output) = pair.output_mut() {
                            let src = if ci % 2 == 0 { &self.mix_l } else { &self.mix_r };
                            for (o, m) in output[..frames].iter_mut().zip(&src[..frames]) {
                                *o = f64::from(*m * norm);
                            }
                        }
                    }
                }
            }
        }

        Ok(ProcessStatus::Continue)
    }
}
