//! Transport state: play/pause/stop, BPM, loop configuration, swing amount.

use serde::{Deserialize, Serialize};

/// Sequencer playback state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordMode {
    Off,
    On,
}

/// Per-section loop length settings (8/16/24/32 steps for drum and synth independently).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LoopConfig {
    pub enabled: bool,
    pub drum_length: u8,  // 8, 16, 24, or 32
    #[serde(alias = "synth_length")]
    pub synth_a_length: u8, // 8, 16, 24, or 32 (was: synth_length)
    #[serde(default = "default_synth_b_length")]
    pub synth_b_length: u8, // 8, 16, 24, or 32
}

fn default_synth_b_length() -> u8 { 16 }

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            drum_length: 32,
            synth_a_length: 32,
            synth_b_length: 16,
        }
    }
}

/// Master transport: play state, tempo (BPM), record mode, loop config, and swing.
#[derive(Clone, Copy, Debug)]
pub struct Transport {
    pub state: PlayState,
    pub bpm: f64, // 60.0..=300.0
    pub record_mode: RecordMode,
    pub loop_config: LoopConfig,
    pub swing: f32, // 0.50 (straight) .. 0.75 (heavy shuffle)
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            state: PlayState::Stopped,
            bpm: 120.0,
            record_mode: RecordMode::Off,
            loop_config: LoopConfig::default(),
            swing: 0.50,
        }
    }
}
