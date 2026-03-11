//! Lock-free message types for cross-thread communication between UI and audio.

use crate::params::EffectParams;
use crate::sequencer::drum_pattern::{DrumPattern, DrumTrackId};
use crate::sequencer::synth_pattern::SynthPattern;
use crate::sequencer::transport::Transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthId {
    A,
    B,
}

/// Commands sent from the UI thread to the audio thread.
/// Sent via a bounded crossbeam channel (capacity 64).
pub enum UiToAudio {
    SetTransport(Transport),
    SetDrumPattern(DrumPattern),
    SetSynthPattern(SynthId, SynthPattern),
    SetEffectParams(EffectParams),
    TriggerDrum(DrumTrackId),  // fire the voice immediately
    TriggerSynth(SynthId, u8),         // MIDI note number — fire synth immediately
    ReleaseSynth(SynthId),             // release synth envelopes
}

/// Notifications sent from the audio thread back to the UI.
/// Sent via a bounded crossbeam channel (capacity 16).
pub enum AudioToUi {
    PlaybackPosition {
        global_step: usize,
        beat: u8,
        is_bar_start: bool,
        triggered: u8,       // bitmask: which drum tracks triggered on this step
        synth_a_triggered: bool, // whether synth A was triggered on this step
        drum_step: usize,    // drum pattern step (global_step % drum_length)
        synth_a_step: usize,   // synth A pattern step (global_step % synth_length)
        synth_b_step: usize,   // synth B pattern step (global_step % synth_length)
        synth_b_triggered: bool, // whether synth B was triggered on this step
    },
}
