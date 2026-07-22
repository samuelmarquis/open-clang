//! open-clang — physical drum/impact synthesizer (WRAC shell, M4 v0).
//!
//! The DSP lives in the shared `clg-engine` crate (also used by the offline
//! `clg` CLI). This crate is the WRAC/CLAP shell: parameters, state
//! persistence, MIDI-note triggering, and an 8-voice pool over the
//! single-voice engine.
//!
//! M4 v0 is HEADLESS: controls live in the host's generic parameter editor.
//! The panel (M5) will add the board + the modal-transect display; the
//! engine's viz feed is not wired yet (`// M5: viz` markers).
//!
//! File layout follows the WRAC template:
//! - `plugin.rs` : the plugin contract as seen by the host
//! - `state.rs`  : lock-free parameter state (one atomic per param)
//! - `audio.rs`  : the audio-thread processor (voice pool over clg_engine)

mod audio;
mod plugin;
mod state;

// Export the CLAP entry point. The adapter owns the C ABI and calls the safe Rust entry.
wrac_clap_adapter::export_clap_entry! {
    entry: &crate::plugin::PLUGIN_ENTRY,
}
