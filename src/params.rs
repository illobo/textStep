//! Shared effect parameters used by both UI and audio engine.

use serde::{Deserialize, Serialize};

/// Master effect and mix bus parameters, serialized with each project.
/// All values are normalized to 0.0..1.0.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EffectParams {
    pub reverb_amount: f32,      // 0.0-1.0: feedback/decay
    pub reverb_damping: f32,     // 0.0-1.0: tail brightness (0=bright, 1=dark)
    #[serde(default = "default_delay_time")]
    pub delay_time: f32,         // 0.0-1.0: single-knob delay macro
    pub delay_feedback: f32,     // 0.0-1.0: (legacy, unused by single-knob)
    pub delay_tone: f32,         // 0.0-1.0: (legacy, unused by single-knob)
    #[serde(default)]
    pub compressor_amount: f32,  // 0.0-1.0: master glue compressor (0=off)
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,      // 0.0-1.0: master output volume
    #[serde(default)]
    pub drum_saturator_drive: f32,    // 0.0-1.0: drum tube saturator drive (0=off)
    #[serde(default)]
    pub synth_saturator_drive: f32,   // 0.0-1.0: synth tube saturator drive (0=off)
    #[serde(default = "default_drum_volume")]
    pub drum_volume: f32,        // 0.0-1.0: drum bus output volume
    #[serde(default = "default_crossfader")]
    pub crossfader: f32,         // 0.0-1.0: synth A/B crossfader (0=A, 0.5=center, 1=B)
    #[serde(default = "default_sidechain")]
    pub sidechain_amount: f32,   // 0.0-1.0: kick→synth sidechain duck depth (0=off)
}

fn default_sidechain() -> f32 { 0.5 }

fn default_master_volume() -> f32 { 0.8 }
fn default_drum_volume() -> f32 { 1.0 }
fn default_crossfader() -> f32 { 0.5 }
fn default_delay_time() -> f32 { 0.4 } // maps to ~1/8D (dotted eighth)

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            reverb_amount: 0.3,
            reverb_damping: 0.4,
            delay_time: 0.4,    // dotted eighth (1/8D) — polyrhythmic bounce
            delay_feedback: 0.45,
            delay_tone: 0.5,
            compressor_amount: 0.0,
            master_volume: 0.8,
            drum_saturator_drive: 0.0,
            synth_saturator_drive: 0.0,
            drum_volume: 1.0,
            crossfader: 0.5,
            sidechain_amount: 0.5,
        }
    }
}
