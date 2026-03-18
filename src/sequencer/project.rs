//! Project serialization: bundles kit sounds, synth parameters, and 10 patterns
//! as JSON (.tsp files).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::params::EffectParams;
use crate::presets::pattern_presets;
use crate::presets::synth_presets;
use crate::presets::synth_pattern_presets;
use crate::sequencer::drum_pattern::{
    DrumTrackParams, DrumTrackId, MAX_STEPS, NUM_DRUM_TRACKS, TRACK_IDS,
};
use super::synth_pattern::{
    SynthParams, SynthPattern, SynthStep,
    MAX_STEPS as SYNTH_MAX_STEPS,
};

pub const NUM_PATTERNS: usize = 10;
pub const NUM_KITS: usize = 8;
pub const NUM_SCENES: usize = 16;

// ── Serializable sound params (no mute/solo) ───────────────────────────────

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DrumSoundParams {
    #[serde(default = "default_half")]
    pub tune: f32,
    #[serde(default = "default_third")]
    pub sweep: f32,
    #[serde(default = "default_half")]
    pub color: f32,
    #[serde(default = "default_third")]
    pub snap: f32,
    #[serde(default = "default_half")]
    pub filter: f32,
    #[serde(default)]
    pub drive: f32,
    #[serde(default = "default_half")]
    pub decay: f32,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default)]
    pub send_reverb: f32,
    #[serde(default)]
    pub send_delay: f32,
    #[serde(default = "default_half")]
    pub pan: f32,
}

fn default_half() -> f32 { 0.5 }
fn default_third() -> f32 { 0.3 }
fn default_volume() -> f32 { 0.8 }

impl Default for DrumSoundParams {
    fn default() -> Self {
        Self {
            tune: 0.5, sweep: 0.3, color: 0.5, snap: 0.3,
            filter: 0.5, drive: 0.0, decay: 0.5, volume: 0.8,
            send_reverb: 0.0, send_delay: 0.0, pan: 0.5,
        }
    }
}

impl DrumSoundParams {
    pub fn defaults_for(track: DrumTrackId) -> Self {
        let p = DrumTrackParams::defaults_for(track);
        Self {
            tune: p.tune, sweep: p.sweep, color: p.color, snap: p.snap,
            filter: p.filter, drive: p.drive, decay: p.decay, volume: p.volume,
            send_reverb: p.send_reverb, send_delay: p.send_delay, pan: p.pan,
        }
    }

    pub fn to_track_params(self) -> DrumTrackParams {
        DrumTrackParams {
            tune: self.tune, sweep: self.sweep, color: self.color, snap: self.snap,
            filter: self.filter, drive: self.drive, decay: self.decay, volume: self.volume,
            send_reverb: self.send_reverb, send_delay: self.send_delay, pan: self.pan,
            mute: false, solo: false,
        }
    }

    pub fn from_track_params(p: &DrumTrackParams) -> Self {
        Self {
            tune: p.tune, sweep: p.sweep, color: p.color, snap: p.snap,
            filter: p.filter, drive: p.drive, decay: p.decay, volume: p.volume,
            send_reverb: p.send_reverb, send_delay: p.send_delay, pan: p.pan,
        }
    }
}

// ── Genre Kit Presets ────────────────────────────────────────────────────────

/// Helper: build a DrumSoundParams from (tune, sweep, color, snap, filter, drive, decay, volume, send_reverb, send_delay).
fn ds(t: f32, sw: f32, c: f32, sn: f32, f: f32, dr: f32, dc: f32, v: f32, sr: f32, sd: f32) -> DrumSoundParams {
    DrumSoundParams { tune: t, sweep: sw, color: c, snap: sn, filter: f, drive: dr, decay: dc, volume: v, send_reverb: sr, send_delay: sd, pan: 0.5 }
}

/// Build a named kit from 8 DrumSoundParams (one per voice in TRACK_IDS order).
fn make_kit(name: &str, voices: [DrumSoundParams; 8]) -> DrumKit {
    let tracks = TRACK_IDS.iter().zip(voices.iter()).map(|(&id, params)| KitTrack {
        id: id.name().to_lowercase(),
        params: *params,
    }).collect();
    DrumKit { name: name.to_string(), tracks }
}

/// 8 genre-specific drum kits with hand-tuned parameters.
pub fn genre_kits() -> Vec<DrumKit> {
    vec![
        // Kit 1: 808 — deep, boomy, analog warmth
        make_kit("808", [
            //       tune  sweep color snap  filt  drive decay vol   rvb   dly
            ds(0.20, 0.70, 0.15, 0.10, 0.50, 0.10, 0.80, 0.85, 0.05, 0.00), // Kick
            ds(0.35, 0.15, 0.50, 0.40, 0.50, 0.10, 0.40, 0.80, 0.15, 0.00), // Snare
            ds(0.60, 0.00, 0.50, 0.40, 0.65, 0.00, 0.08, 0.70, 0.05, 0.00), // CHH
            ds(0.50, 0.00, 0.50, 0.30, 0.55, 0.00, 0.50, 0.65, 0.10, 0.00), // OHH
            ds(0.55, 0.00, 0.45, 0.15, 0.50, 0.00, 0.70, 0.55, 0.10, 0.00), // Ride
            ds(0.50, 0.30, 0.50, 0.50, 0.50, 0.10, 0.40, 0.70, 0.20, 0.00), // Clap
            ds(0.50, 0.30, 0.50, 0.20, 0.50, 0.10, 0.40, 0.70, 0.10, 0.00), // Cowbell
            ds(0.30, 0.60, 0.10, 0.30, 0.70, 0.10, 0.55, 0.80, 0.10, 0.00), // Tom
        ]),
        // Kit 2: 909 — punchy, crisp, harder-hitting
        make_kit("909", [
            ds(0.35, 0.50, 0.30, 0.60, 0.80, 0.30, 0.45, 0.85, 0.05, 0.00), // Kick
            ds(0.45, 0.10, 0.60, 0.60, 0.65, 0.20, 0.35, 0.85, 0.15, 0.00), // Snare
            ds(0.55, 0.00, 0.55, 0.35, 0.70, 0.10, 0.12, 0.70, 0.05, 0.00), // CHH
            ds(0.55, 0.00, 0.55, 0.35, 0.65, 0.10, 0.55, 0.70, 0.10, 0.00), // OHH
            ds(0.60, 0.00, 0.50, 0.25, 0.60, 0.10, 0.65, 0.60, 0.10, 0.00), // Ride
            ds(0.55, 0.25, 0.55, 0.60, 0.60, 0.15, 0.35, 0.75, 0.20, 0.00), // Clap
            ds(0.55, 0.25, 0.45, 0.30, 0.60, 0.10, 0.35, 0.65, 0.10, 0.00), // Cowbell
            ds(0.50, 0.70, 0.20, 0.50, 0.70, 0.20, 0.45, 0.75, 0.10, 0.00), // Tom
        ]),
        // Kit 3: Techno — dark, driving, industrial-leaning
        make_kit("Techno", [
            ds(0.22, 0.75, 0.25, 0.55, 0.60, 0.35, 0.70, 0.90, 0.05, 0.00), // Kick
            ds(0.38, 0.10, 0.70, 0.65, 0.55, 0.35, 0.25, 0.75, 0.15, 0.00), // Snare
            ds(0.50, 0.05, 0.60, 0.50, 0.55, 0.25, 0.06, 0.65, 0.05, 0.00), // CHH
            ds(0.45, 0.10, 0.65, 0.40, 0.50, 0.30, 0.40, 0.60, 0.10, 0.10), // OHH
            ds(0.50, 0.05, 0.60, 0.30, 0.45, 0.30, 0.55, 0.50, 0.10, 0.10), // Ride
            ds(0.48, 0.20, 0.60, 0.70, 0.55, 0.30, 0.30, 0.75, 0.15, 0.00), // Clap
            ds(0.55, 0.35, 0.55, 0.40, 0.50, 0.35, 0.25, 0.60, 0.10, 0.10), // Cowbell
            ds(0.35, 0.75, 0.30, 0.55, 0.55, 0.35, 0.50, 0.75, 0.10, 0.00), // Tom
        ]),
        // Kit 4: House — classic house, warm and bouncy
        make_kit("House", [
            ds(0.28, 0.60, 0.20, 0.45, 0.65, 0.15, 0.60, 0.85, 0.05, 0.00), // Kick
            ds(0.42, 0.10, 0.55, 0.50, 0.60, 0.15, 0.38, 0.78, 0.15, 0.00), // Snare
            ds(0.58, 0.00, 0.45, 0.35, 0.65, 0.05, 0.10, 0.68, 0.05, 0.00), // CHH
            ds(0.52, 0.00, 0.45, 0.30, 0.60, 0.05, 0.50, 0.65, 0.10, 0.00), // OHH
            ds(0.58, 0.00, 0.40, 0.20, 0.55, 0.05, 0.65, 0.55, 0.10, 0.00), // Ride
            ds(0.52, 0.30, 0.50, 0.55, 0.55, 0.10, 0.42, 0.72, 0.25, 0.00), // Clap
            ds(0.55, 0.25, 0.40, 0.25, 0.55, 0.05, 0.35, 0.60, 0.10, 0.00), // Cowbell
            ds(0.45, 0.55, 0.15, 0.40, 0.75, 0.10, 0.50, 0.75, 0.10, 0.00), // Tom
        ]),
        // Kit 5: Minimal — clean, sparse, subtle
        make_kit("Minimal", [
            ds(0.38, 0.30, 0.10, 0.75, 0.65, 0.00, 0.20, 0.75, 0.05, 0.00), // Kick
            ds(0.50, 0.00, 0.35, 0.70, 0.75, 0.00, 0.15, 0.65, 0.10, 0.00), // Snare
            ds(0.72, 0.00, 0.30, 0.25, 0.85, 0.00, 0.04, 0.55, 0.05, 0.10), // CHH
            ds(0.65, 0.00, 0.30, 0.20, 0.60, 0.00, 0.25, 0.50, 0.10, 0.10), // OHH
            ds(0.65, 0.00, 0.25, 0.15, 0.55, 0.00, 0.45, 0.45, 0.10, 0.10), // Ride
            ds(0.58, 0.10, 0.30, 0.80, 0.78, 0.00, 0.10, 0.60, 0.10, 0.00), // Clap
            ds(0.52, 0.10, 0.25, 0.30, 0.50, 0.00, 0.18, 0.50, 0.10, 0.10), // Cowbell
            ds(0.60, 0.20, 0.05, 0.50, 0.60, 0.00, 0.22, 0.60, 0.10, 0.00), // Tom
        ]),
        // Kit 6: Lo-Fi — gritty, crushed, vintage
        make_kit("Lo-Fi", [
            ds(0.28, 0.55, 0.45, 0.30, 0.40, 0.45, 0.55, 0.78, 0.15, 0.00), // Kick
            ds(0.38, 0.12, 0.65, 0.35, 0.40, 0.45, 0.42, 0.72, 0.20, 0.00), // Snare
            ds(0.48, 0.00, 0.60, 0.25, 0.38, 0.35, 0.10, 0.62, 0.10, 0.05), // CHH
            ds(0.42, 0.00, 0.55, 0.20, 0.35, 0.35, 0.50, 0.60, 0.15, 0.10), // OHH
            ds(0.42, 0.00, 0.55, 0.12, 0.35, 0.25, 0.75, 0.52, 0.15, 0.10), // Ride
            ds(0.45, 0.25, 0.60, 0.40, 0.42, 0.40, 0.40, 0.68, 0.20, 0.00), // Clap
            ds(0.48, 0.30, 0.55, 0.20, 0.40, 0.35, 0.38, 0.58, 0.15, 0.10), // Cowbell
            ds(0.38, 0.50, 0.35, 0.25, 0.45, 0.40, 0.52, 0.70, 0.15, 0.00), // Tom
        ]),
        // Kit 7: Electro — bright, aggressive, funk-influenced
        make_kit("Electro", [
            ds(0.30, 0.80, 0.25, 0.70, 0.85, 0.25, 0.50, 0.88, 0.05, 0.00), // Kick
            ds(0.48, 0.15, 0.55, 0.75, 0.80, 0.25, 0.30, 0.82, 0.10, 0.00), // Snare
            ds(0.65, 0.05, 0.45, 0.55, 0.80, 0.15, 0.07, 0.72, 0.05, 0.10), // CHH
            ds(0.60, 0.05, 0.50, 0.45, 0.75, 0.15, 0.45, 0.68, 0.10, 0.10), // OHH
            ds(0.65, 0.05, 0.45, 0.35, 0.70, 0.15, 0.55, 0.58, 0.10, 0.10), // Ride
            ds(0.55, 0.20, 0.50, 0.70, 0.75, 0.20, 0.28, 0.78, 0.10, 0.00), // Clap
            ds(0.60, 0.30, 0.40, 0.35, 0.70, 0.15, 0.30, 0.68, 0.10, 0.10), // Cowbell
            ds(0.50, 0.80, 0.15, 0.60, 0.80, 0.20, 0.40, 0.78, 0.10, 0.00), // Tom
        ]),
        // Kit 8: Ambient — soft, washy, atmospheric
        make_kit("Ambient", [
            ds(0.25, 0.40, 0.20, 0.10, 0.40, 0.05, 0.70, 0.65, 0.30, 0.00), // Kick
            ds(0.35, 0.05, 0.60, 0.12, 0.35, 0.00, 0.55, 0.55, 0.40, 0.00), // Snare
            ds(0.55, 0.00, 0.55, 0.10, 0.40, 0.00, 0.15, 0.45, 0.30, 0.15), // CHH
            ds(0.48, 0.00, 0.50, 0.08, 0.38, 0.00, 0.75, 0.45, 0.40, 0.20), // OHH
            ds(0.50, 0.00, 0.45, 0.08, 0.35, 0.00, 0.90, 0.42, 0.40, 0.20), // Ride
            ds(0.45, 0.15, 0.55, 0.15, 0.38, 0.00, 0.55, 0.50, 0.35, 0.00), // Clap
            ds(0.48, 0.20, 0.40, 0.10, 0.40, 0.00, 0.50, 0.45, 0.30, 0.15), // Cowbell
            ds(0.40, 0.45, 0.15, 0.12, 0.45, 0.00, 0.65, 0.58, 0.35, 0.00), // Tom
        ]),
    ]
}

// ── Kit ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrumKit {
    #[serde(default = "default_kit_name")]
    pub name: String,
    pub tracks: Vec<KitTrack>,
}

fn default_kit_name() -> String { "Default Kit".to_string() }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KitTrack {
    pub id: String,
    #[serde(flatten)]
    pub params: DrumSoundParams,
}

impl Default for DrumKit {
    fn default() -> Self {
        let tracks = TRACK_IDS.iter().map(|&id| KitTrack {
            id: id.name().to_lowercase(),
            params: DrumSoundParams::defaults_for(id),
        }).collect();
        Self {
            name: "Default Kit".to_string(),
            tracks,
        }
    }
}

// ── Pattern (steps only) ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternData {
    #[serde(default = "default_pattern_name")]
    pub name: String,
    /// Per-pattern BPM override. 0.0 means "use project BPM".
    #[serde(default)]
    pub bpm: f64,
    /// Hex-encoded steps per track (8 hex chars = 32 steps)
    pub steps: Vec<String>,
}

fn default_pattern_name() -> String { "Empty".to_string() }

impl Default for PatternData {
    fn default() -> Self {
        Self {
            name: "Empty".to_string(),
            bpm: 0.0,
            steps: vec!["00000000".to_string(); NUM_DRUM_TRACKS],
        }
    }
}

// ── Synth Kit ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthKitData {
    #[serde(default = "default_kit_name")]
    pub name: String,
    #[serde(default)]
    pub params: SynthParams,
}

impl Default for SynthKitData {
    fn default() -> Self {
        Self {
            name: "Default Kit".to_string(),
            params: SynthParams::default(),
        }
    }
}

impl SynthKitData {
    pub fn apply_to(&self, pattern: &mut SynthPattern) {
        let mute = pattern.params.mute;
        pattern.params = self.params;
        pattern.params.mute = mute;
    }
}

// ── Synth Pattern ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthPatternData {
    #[serde(default = "default_pattern_name")]
    pub name: String,
    pub steps: Vec<SynthStepData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthStepData {
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub note: u8,
    #[serde(default)]
    pub velocity: f32,
    #[serde(default)]
    pub gate: f32,
    #[serde(default = "default_step_length")]
    pub length: u8,
}

fn default_step_length() -> u8 { 1 }

impl Default for SynthStepData {
    fn default() -> Self {
        Self {
            active: false,
            note: 0,
            velocity: 0.0,
            gate: 0.0,
            length: 1,
        }
    }
}

impl Default for SynthPatternData {
    fn default() -> Self {
        Self {
            name: "Empty".to_string(),
            steps: (0..SYNTH_MAX_STEPS).map(|_| SynthStepData::default()).collect(),
        }
    }
}

impl SynthPatternData {
    pub fn from_synth_pattern(pattern: &SynthPattern) -> Self {
        let steps = pattern.steps.iter().map(|s| SynthStepData {
            active: s.is_active(),
            note: s.note,
            velocity: s.velocity as f32 / 127.0,
            gate: 1.0,
            length: s.length,
        }).collect();
        Self {
            name: "Empty".to_string(),
            steps,
        }
    }

    pub fn apply_to(&self, pattern: &mut SynthPattern) {
        for (i, step_data) in self.steps.iter().enumerate() {
            if i >= SYNTH_MAX_STEPS { break; }
            pattern.steps[i] = SynthStep {
                note: step_data.note,
                velocity: if step_data.active {
                    (step_data.velocity * 127.0).round().clamp(1.0, 127.0) as u8
                } else {
                    0
                },
                length: step_data.length,
            };
        }
    }
}

// ── Project ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    #[serde(default)]
    pub drum_pattern: usize,
    #[serde(default)]
    pub drum_kit: usize,
    #[serde(default)]
    pub synth_a_pattern: usize,
    #[serde(default)]
    pub synth_a_kit: usize,
    #[serde(default)]
    pub synth_b_pattern: usize,
    #[serde(default)]
    pub synth_b_kit: usize,
    #[serde(default = "default_bpm")]
    pub bpm: f64,
    #[serde(default = "default_swing")]
    pub swing: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub textstep: FileHeader,
    #[serde(default)]
    pub metadata: ProjectMetadata,
    /// Legacy single kit field — read from old files, not written to new saves.
    #[serde(default, skip_serializing)]
    pub kit: DrumKit,
    /// Kit bank (8 kits). New files use this.
    #[serde(default)]
    pub kits: Vec<DrumKit>,
    #[serde(default)]
    pub active_kit: usize,
    pub patterns: Vec<PatternData>,
    #[serde(default)]
    pub active_pattern: usize,
    #[serde(default = "default_bpm")]
    pub bpm: f64,
    #[serde(default = "default_loop_length")]
    pub loop_length: u8,
    #[serde(default = "default_swing")]
    pub swing: f32,
    #[serde(default)]
    pub effects: EffectParams,
    #[serde(default)]
    pub synth_kits: Vec<SynthKitData>,
    #[serde(default)]
    pub active_synth_kit: usize,
    #[serde(default)]
    pub synth_patterns: Vec<SynthPatternData>,
    #[serde(default)]
    pub active_synth_pattern: usize,
    #[serde(default)]
    pub synth_b_kits: Vec<SynthKitData>,
    #[serde(default)]
    pub active_synth_b_kit: usize,
    #[serde(default)]
    pub synth_b_patterns: Vec<SynthPatternData>,
    #[serde(default)]
    pub active_synth_b_pattern: usize,
    #[serde(default)]
    pub scenes: Vec<Option<Scene>>,
}

fn default_bpm() -> f64 { 120.0 }
fn default_loop_length() -> u8 { 32 }
fn default_swing() -> f32 { 0.50 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileHeader {
    pub format_version: u32,
    #[serde(default)]
    pub app_version: String,
}

impl Default for FileHeader {
    fn default() -> Self {
        Self {
            format_version: 1,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectMetadata {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub author: String,
}

impl Default for ProjectFile {
    fn default() -> Self {
        let mut patterns = Vec::with_capacity(NUM_PATTERNS);
        for i in 0..NUM_PATTERNS {
            patterns.push(PatternData {
                name: format!("Pattern {}", i + 1),
                ..Default::default()
            });
        }
        let kits = genre_kits();
        let mut synth_patterns = Vec::with_capacity(NUM_PATTERNS);
        for i in 0..NUM_PATTERNS {
            synth_patterns.push(SynthPatternData {
                name: format!("Synth {}", i + 1),
                ..Default::default()
            });
        }
        let mut synth_kits = Vec::with_capacity(NUM_KITS);
        for i in 0..NUM_KITS {
            synth_kits.push(SynthKitData {
                name: format!("Synth Kit {}", i + 1),
                ..Default::default()
            });
        }
        let mut synth_b_patterns = Vec::with_capacity(NUM_PATTERNS);
        for i in 0..NUM_PATTERNS {
            synth_b_patterns.push(SynthPatternData {
                name: format!("Synth B {}", i + 1),
                ..Default::default()
            });
        }
        let mut synth_b_kits = Vec::with_capacity(NUM_KITS);
        for i in 0..NUM_KITS {
            synth_b_kits.push(SynthKitData {
                name: format!("Synth B Kit {}", i + 1),
                ..Default::default()
            });
        }
        Self {
            textstep: FileHeader::default(),
            metadata: ProjectMetadata {
                name: "Untitled".to_string(),
                ..Default::default()
            },
            kit: DrumKit::default(),
            kits,
            active_kit: 0,
            patterns,
            active_pattern: 0,
            bpm: 120.0,
            loop_length: 32,
            swing: 0.50,
            effects: EffectParams::default(),
            synth_kits,
            active_synth_kit: 0,
            synth_patterns,
            active_synth_pattern: 0,
            synth_b_kits,
            active_synth_b_kit: 0,
            synth_b_patterns,
            active_synth_b_pattern: 0,
            scenes: Vec::new(),
        }
    }
}

fn pattern_from_preset(name: &str, display_name: &str, bpm: f64) -> PatternData {
    if let Some(preset) = pattern_presets::preset_by_name(name) {
        // Mirror 16-step patterns to fill 32 steps: repeat first 4 hex chars into last 4
        let steps: Vec<String> = preset.steps.iter().map(|s| {
            if s.len() == 8 && &s[4..] == "0000" {
                // First half has data, second half empty — repeat first half
                format!("{}{}", &s[..4], &s[..4])
            } else {
                s.to_string()
            }
        }).collect();
        PatternData {
            name: display_name.to_string(),
            bpm,
            steps,
        }
    } else {
        PatternData {
            name: display_name.to_string(),
            bpm,
            steps: vec!["00000000".into(); NUM_DRUM_TRACKS],
        }
    }
}

fn synth_kit_from_preset(name: &str, display_name: &str) -> SynthKitData {
    if let Some(preset) = synth_presets::preset_by_name(name) {
        SynthKitData {
            name: display_name.to_string(),
            params: preset.params,
        }
    } else {
        SynthKitData {
            name: display_name.to_string(),
            ..Default::default()
        }
    }
}

fn synth_pattern_from_preset(preset_name: &str, display_name: &str) -> SynthPatternData {
    if let Some(preset) = synth_pattern_presets::preset_by_name(preset_name) {
        SynthPatternData {
            name: display_name.to_string(),
            steps: preset.steps.iter().map(|&(note, vel, len)| {
                SynthStepData {
                    active: vel > 0,
                    note,
                    velocity: vel as f32 / 127.0,
                    gate: 1.0,
                    length: len,
                }
            }).collect(),
        }
    } else {
        SynthPatternData {
            name: display_name.to_string(),
            ..Default::default()
        }
    }
}

/// Create a demo project with 10 pre-filled genre patterns from classic drum programming.
pub fn demo_project() -> ProjectFile {
    let patterns = vec![
        pattern_from_preset("Around the World","Around the World 121", 121.0),
        pattern_from_preset("Acid House",      "Acid Techno 138",  138.0),
        pattern_from_preset("Classic House",   "House 122",        122.0),
        pattern_from_preset("Deep House",      "Deep House 120",   120.0),
        pattern_from_preset("Driving Techno",  "Techno 130",       130.0),
        pattern_from_preset("Lo-Fi Hip Hop",   "Downtempo 85",      85.0),
        pattern_from_preset("Classic Trance",  "Trance 140",       140.0),
        pattern_from_preset("Amen Break",      "Drum & Bass 174",  174.0),
        pattern_from_preset("Electro Funk",    "Electro 128",      128.0),
        pattern_from_preset("Basic Chain",     "Dub Techno 118",   118.0),
    ];

    let kits = genre_kits();

    let synth_patterns = vec![
        synth_pattern_from_preset("Around World Bass",   "Daft Punk Bass"),
        synth_pattern_from_preset("Acid Techno Bass 1",  "Acid Techno Bass"),
        synth_pattern_from_preset("House Bass 1",        "House Bass"),
        synth_pattern_from_preset("House Bass 3",        "Deep House Bass"),
        synth_pattern_from_preset("Techno Bass 1",       "Techno Bass"),
        synth_pattern_from_preset("Downtempo Bass 1",    "Downtempo Bass"),
        synth_pattern_from_preset("Trance Bass 1",       "Trance Bass"),
        synth_pattern_from_preset("Drum & Bass Bass 1",  "DnB Bass"),
        synth_pattern_from_preset("Electro Bass 1",      "Electro Bass"),
        synth_pattern_from_preset("Dub Techno Bass 1",   "Dub Techno Bass"),
    ];
    let synth_kits = vec![
        synth_kit_from_preset("Electric Piano", "Electric Piano"), // Kit 0: Around the World
        synth_kit_from_preset("Wobble Bass",  "Wobble Bass"),    // Kit 1: Acid Techno
        synth_kit_from_preset("Acid Bass",    "Acid Bass"),      // Kit 2: House
        synth_kit_from_preset("Reese Bass",   "Reese Bass"),     // Kit 3: Deep House
        synth_kit_from_preset("Pulse Bass",   "Pulse Bass"),     // Kit 4: Techno
        synth_kit_from_preset("Sub Bass",     "Sub Bass"),       // Kit 5: Downtempo, Dub Techno
        synth_kit_from_preset("FM Bass",      "FM Bass"),        // Kit 6: Trance, Ambient (reuse)
        synth_kit_from_preset("Growl Bass",   "Growl Bass"),     // Kit 7: DnB
    ];
    let synth_b_patterns = vec![
        synth_pattern_from_preset("Around World Lead",  "Daft Punk Lead"),
        synth_pattern_from_preset("Acid Techno 1",  "Acid Techno Lead"),
        synth_pattern_from_preset("House 1",        "House Keys"),
        synth_pattern_from_preset("House 3",        "Deep House Pad"),
        synth_pattern_from_preset("Techno 1",       "Techno Lead"),
        synth_pattern_from_preset("Downtempo 1",    "Downtempo Pad"),
        synth_pattern_from_preset("Trance 1",       "Trance Lead"),
        synth_pattern_from_preset("Drum & Bass 1",  "DnB Pluck"),
        synth_pattern_from_preset("Electro 1",      "Electro Lead"),
        synth_pattern_from_preset("Dub Techno 1",   "Dub Techno Pad"),
    ];
    let synth_b_kits = vec![
        synth_kit_from_preset("Acid Bass",      "Acid Bass"),      // Kit 0: Around the World lead
        synth_kit_from_preset("Screamer",       "Screamer"),       // Kit 1: Acid Techno
        synth_kit_from_preset("Electric Piano", "Electric Piano"), // Kit 2: House
        synth_kit_from_preset("Shimmer Pad",    "Shimmer Pad"),    // Kit 3: Deep House, Dub Techno
        synth_kit_from_preset("Saw Lead",       "Saw Lead"),       // Kit 4: Techno
        synth_kit_from_preset("Warm Pad",       "Warm Pad"),       // Kit 5: Downtempo
        synth_kit_from_preset("Trance Lead",    "Trance Lead"),    // Kit 6: Trance
        synth_kit_from_preset("Basic Pluck",    "Basic Pluck"),    // Kit 7: DnB, Ambient
    ];

    ProjectFile {
        textstep: FileHeader::default(),
        metadata: ProjectMetadata {
            name: "Demo Song".to_string(),
            ..Default::default()
        },
        kit: DrumKit::default(),
        kits,
        active_kit: 0,
        patterns,
        active_pattern: 0,
        bpm: 121.0,
        loop_length: 32,
        swing: 0.50,
        effects: EffectParams::default(),
        synth_kits,
        active_synth_kit: 0,
        synth_patterns,
        active_synth_pattern: 0,
        synth_b_kits,
        active_synth_b_kit: 0,
        synth_b_patterns,
        active_synth_b_pattern: 0,
        scenes: vec![
            Some(Scene { name: "Around the World".into(), drum_pattern: 0, drum_kit: 0, synth_a_pattern: 0, synth_a_kit: 0, synth_b_pattern: 0, synth_b_kit: 0, bpm: 121.0, swing: 0.50 }),
            Some(Scene { name: "Acid Techno".into(),   drum_pattern: 1, drum_kit: 0, synth_a_pattern: 1, synth_a_kit: 1, synth_b_pattern: 1, synth_b_kit: 1, bpm: 138.0, swing: 0.50 }),
            Some(Scene { name: "Classic House".into(), drum_pattern: 2, drum_kit: 3, synth_a_pattern: 2, synth_a_kit: 2, synth_b_pattern: 2, synth_b_kit: 2, bpm: 122.0, swing: 0.50 }),
            Some(Scene { name: "Deep House".into(),    drum_pattern: 3, drum_kit: 3, synth_a_pattern: 3, synth_a_kit: 3, synth_b_pattern: 3, synth_b_kit: 3, bpm: 120.0, swing: 0.50 }),
            Some(Scene { name: "Driving Techno".into(),drum_pattern: 4, drum_kit: 2, synth_a_pattern: 4, synth_a_kit: 4, synth_b_pattern: 4, synth_b_kit: 4, bpm: 130.0, swing: 0.50 }),
            Some(Scene { name: "Lo-Fi Hip Hop".into(), drum_pattern: 5, drum_kit: 5, synth_a_pattern: 5, synth_a_kit: 5, synth_b_pattern: 5, synth_b_kit: 5, bpm: 85.0,  swing: 0.50 }),
            Some(Scene { name: "Trance".into(),        drum_pattern: 6, drum_kit: 1, synth_a_pattern: 6, synth_a_kit: 6, synth_b_pattern: 6, synth_b_kit: 6, bpm: 140.0, swing: 0.50 }),
            Some(Scene { name: "Drum & Bass".into(),   drum_pattern: 7, drum_kit: 6, synth_a_pattern: 7, synth_a_kit: 7, synth_b_pattern: 7, synth_b_kit: 7, bpm: 174.0, swing: 0.50 }),
            Some(Scene { name: "Electro Funk".into(),  drum_pattern: 8, drum_kit: 6, synth_a_pattern: 8, synth_a_kit: 1, synth_b_pattern: 8, synth_b_kit: 2, bpm: 128.0, swing: 0.50 }),
            Some(Scene { name: "Dub Techno".into(),    drum_pattern: 9, drum_kit: 2, synth_a_pattern: 9, synth_a_kit: 5, synth_b_pattern: 9, synth_b_kit: 3, bpm: 118.0, swing: 0.50 }),
        ],
    }
}

// ── Hex step encoding ───────────────────────────────────────────────────────

pub fn steps_to_hex(steps: &[bool; MAX_STEPS]) -> String {
    let mut s = String::with_capacity(8);
    for chunk in steps.chunks(4) {
        let nibble = (chunk[0] as u8) << 3
                   | (chunk[1] as u8) << 2
                   | (chunk[2] as u8) << 1
                   | (chunk[3] as u8);
        s.push(char::from_digit(nibble as u32, 16).unwrap());
    }
    s
}

pub fn hex_to_steps(hex: &str) -> [bool; MAX_STEPS] {
    let mut steps = [false; MAX_STEPS];
    for (i, ch) in hex.chars().enumerate() {
        if i >= 8 { break; }
        let nibble = ch.to_digit(16).unwrap_or(0) as u8;
        steps[i * 4]     = nibble & 0b1000 != 0;
        steps[i * 4 + 1] = nibble & 0b0100 != 0;
        steps[i * 4 + 2] = nibble & 0b0010 != 0;
        steps[i * 4 + 3] = nibble & 0b0001 != 0;
    }
    steps
}

// ── Project <-> App state conversion ────────────────────────────────────────

use crate::sequencer::drum_pattern::DrumPattern;

impl ProjectFile {
    /// Load kit sound params into a DrumPattern's params array.
    pub fn apply_kit_to_pattern(&self, kit_index: usize, pattern: &mut DrumPattern) {
        let kit = self.kits.get(kit_index).unwrap_or(&self.kits[0]);
        for (i, kit_track) in kit.tracks.iter().enumerate() {
            if i < NUM_DRUM_TRACKS {
                let p = kit_track.params.to_track_params();
                // Preserve mute/solo state
                let mute = pattern.params[i].mute;
                let solo = pattern.params[i].solo;
                pattern.params[i] = p;
                pattern.params[i].mute = mute;
                pattern.params[i].solo = solo;
            }
        }
    }

    /// Load step data from a specific pattern index into a DrumPattern.
    pub fn load_pattern_steps(&self, index: usize, pattern: &mut DrumPattern) {
        if let Some(pat_data) = self.patterns.get(index) {
            for (track, hex) in pat_data.steps.iter().enumerate() {
                if track < NUM_DRUM_TRACKS {
                    pattern.steps[track] = hex_to_steps(hex);
                }
            }
        }
    }

    /// Save the current DrumPattern steps into a pattern slot.
    pub fn save_pattern_steps(&mut self, index: usize, pattern: &DrumPattern) {
        if index < self.patterns.len() {
            self.patterns[index].steps = pattern.steps.iter()
                .map(|track_steps| steps_to_hex(track_steps))
                .collect();
        }
    }

    /// Save current kit params from a DrumPattern into a specific kit slot.
    pub fn save_kit_from_pattern(&mut self, kit_index: usize, pattern: &DrumPattern) {
        if kit_index >= self.kits.len() { return; }
        let kit = &mut self.kits[kit_index];
        for (i, track_id) in TRACK_IDS.iter().enumerate() {
            if i < kit.tracks.len() {
                kit.tracks[i].params = DrumSoundParams::from_track_params(&pattern.params[i]);
            } else {
                kit.tracks.push(KitTrack {
                    id: track_id.name().to_lowercase(),
                    params: DrumSoundParams::from_track_params(&pattern.params[i]),
                });
            }
        }
    }

    /// Save synth pattern steps to project.
    pub fn save_synth_pattern(&mut self, index: usize, pattern: &SynthPattern) {
        if index < self.synth_patterns.len() {
            self.synth_patterns[index] = SynthPatternData::from_synth_pattern(pattern);
        }
    }

    /// Load synth pattern steps from project.
    pub fn load_synth_pattern(&self, index: usize, pattern: &mut SynthPattern) {
        if let Some(pat_data) = self.synth_patterns.get(index) {
            pat_data.apply_to(pattern);
        }
    }

    /// Save synth kit params to project.
    pub fn save_synth_kit(&mut self, index: usize, params: &SynthParams) {
        if index < self.synth_kits.len() {
            self.synth_kits[index].params = *params;
        }
    }

    /// Load synth kit params into pattern.
    pub fn load_synth_kit(&self, index: usize, pattern: &mut SynthPattern) {
        if let Some(kit_data) = self.synth_kits.get(index) {
            kit_data.apply_to(pattern);
        }
    }

    /// Save synth B pattern steps to project.
    pub fn save_synth_b_pattern(&mut self, index: usize, pattern: &SynthPattern) {
        if index < self.synth_b_patterns.len() {
            self.synth_b_patterns[index] = SynthPatternData::from_synth_pattern(pattern);
        }
    }

    /// Load synth B pattern steps from project.
    pub fn load_synth_b_pattern(&self, index: usize, pattern: &mut SynthPattern) {
        if let Some(pat_data) = self.synth_b_patterns.get(index) {
            pat_data.apply_to(pattern);
        }
    }

    /// Save synth B kit params to project.
    pub fn save_synth_b_kit(&mut self, index: usize, params: &SynthParams) {
        if index < self.synth_b_kits.len() {
            self.synth_b_kits[index].params = *params;
        }
    }

    /// Load synth B kit params into pattern.
    pub fn load_synth_b_kit(&self, index: usize, pattern: &mut SynthPattern) {
        if let Some(kit_data) = self.synth_b_kits.get(index) {
            kit_data.apply_to(pattern);
        }
    }

    /// Ensure we always have NUM_PATTERNS patterns and NUM_KITS kits.
    pub fn normalize(&mut self) {
        // Migrate old single-kit format: if kits is empty, seed from legacy kit field
        if self.kits.is_empty() && !self.kit.tracks.is_empty() {
            self.kits.push(self.kit.clone());
        }
        while self.kits.len() < NUM_KITS {
            let idx = self.kits.len();
            self.kits.push(DrumKit {
                name: format!("Kit {}", idx + 1),
                ..DrumKit::default()
            });
        }
        if self.active_kit >= self.kits.len() {
            self.active_kit = 0;
        }

        while self.patterns.len() < NUM_PATTERNS {
            let idx = self.patterns.len();
            self.patterns.push(PatternData {
                name: format!("Pattern {}", idx + 1),
                ..Default::default()
            });
        }
        if self.active_pattern >= self.patterns.len() {
            self.active_pattern = 0;
        }

        while self.synth_kits.len() < NUM_KITS {
            let idx = self.synth_kits.len();
            self.synth_kits.push(SynthKitData {
                name: format!("Synth Kit {}", idx + 1),
                ..Default::default()
            });
        }
        if self.active_synth_kit >= self.synth_kits.len() {
            self.active_synth_kit = 0;
        }

        while self.synth_patterns.len() < NUM_PATTERNS {
            let idx = self.synth_patterns.len();
            self.synth_patterns.push(SynthPatternData {
                name: format!("Synth {}", idx + 1),
                ..Default::default()
            });
        }
        if self.active_synth_pattern >= self.synth_patterns.len() {
            self.active_synth_pattern = 0;
        }

        while self.synth_b_kits.len() < NUM_KITS {
            let idx = self.synth_b_kits.len();
            self.synth_b_kits.push(SynthKitData {
                name: format!("Synth B Kit {}", idx + 1),
                ..Default::default()
            });
        }
        if self.active_synth_b_kit >= self.synth_b_kits.len() {
            self.active_synth_b_kit = 0;
        }

        while self.synth_b_patterns.len() < NUM_PATTERNS {
            let idx = self.synth_b_patterns.len();
            self.synth_b_patterns.push(SynthPatternData {
                name: format!("Synth B {}", idx + 1),
                ..Default::default()
            });
        }
        if self.active_synth_b_pattern >= self.synth_b_patterns.len() {
            self.active_synth_b_pattern = 0;
        }

        // Normalize scenes
        for scene in &mut self.scenes {
            if let Some(s) = scene {
                if s.drum_pattern >= NUM_PATTERNS { s.drum_pattern = 0; }
                if s.drum_kit >= NUM_KITS { s.drum_kit = 0; }
                if s.synth_a_pattern >= NUM_PATTERNS { s.synth_a_pattern = 0; }
                if s.synth_a_kit >= NUM_KITS { s.synth_a_kit = 0; }
                if s.synth_b_pattern >= NUM_PATTERNS { s.synth_b_pattern = 0; }
                if s.synth_b_kit >= NUM_KITS { s.synth_b_kit = 0; }
            }
        }
    }
}

// ── File I/O ────────────────────────────────────────────────────────────────

pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Library/Application Support/textstep")
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg).join("textstep")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".local/share/textstep")
        }
    }
}

pub fn projects_dir() -> PathBuf {
    data_dir().join("projects")
}

pub fn kits_dir() -> PathBuf {
    data_dir().join("kits")
}

pub fn save_project(project: &ProjectFile, path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(project).map_err(|e| format!("Serialize: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Write: {}", e))?;
    Ok(())
}

pub fn load_project(path: &std::path::Path) -> Result<ProjectFile, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Read: {}", e))?;
    let mut project: ProjectFile = serde_json::from_str(&json).map_err(|e| format!("Parse: {}", e))?;
    project.normalize();
    Ok(project)
}

/// List all .tsp files in the projects directory.
pub fn list_projects() -> Vec<(String, PathBuf)> {
    let dir = projects_dir();
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("tsp") {
                // Try to read just the name from metadata
                let name = if let Ok(proj) = load_project(&path) {
                    if proj.metadata.name.is_empty() {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("?")
                            .to_string()
                    } else {
                        proj.metadata.name
                    }
                } else {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("?")
                        .to_string()
                };
                results.push((name, path));
            }
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

// ── Kit I/O ─────────────────────────────────────────────────────────────────

pub fn save_kit(kit: &DrumKit, path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(kit).map_err(|e| format!("Serialize: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Write: {}", e))?;
    Ok(())
}

pub fn load_kit(path: &std::path::Path) -> Result<DrumKit, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Read: {}", e))?;
    let kit: DrumKit = serde_json::from_str(&json).map_err(|e| format!("Parse: {}", e))?;
    Ok(kit)
}

pub fn list_kits() -> Vec<(String, PathBuf)> {
    let dir = kits_dir();
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("tsk") {
                let name = if let Ok(kit) = load_kit(&path) {
                    kit.name
                } else {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("?")
                        .to_string()
                };
                results.push((name, path));
            }
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let mut steps = [false; MAX_STEPS];
        steps[0] = true;
        steps[4] = true;
        steps[8] = true;
        steps[12] = true;
        let hex = steps_to_hex(&steps);
        assert_eq!(hex, "88880000");
        let decoded = hex_to_steps(&hex);
        assert_eq!(steps, decoded);
    }

    #[test]
    fn hex_all_on() {
        let steps = [true; MAX_STEPS];
        let hex = steps_to_hex(&steps);
        assert_eq!(hex, "ffffffff");
    }

    #[test]
    fn hex_empty() {
        let steps = [false; MAX_STEPS];
        let hex = steps_to_hex(&steps);
        assert_eq!(hex, "00000000");
        let decoded = hex_to_steps(&hex);
        assert_eq!(steps, decoded);
    }

    #[test]
    fn project_default_has_10_patterns() {
        let proj = ProjectFile::default();
        assert_eq!(proj.patterns.len(), NUM_PATTERNS);
    }

    #[test]
    fn project_serialize_roundtrip() {
        let proj = ProjectFile::default();
        let json = serde_json::to_string(&proj).unwrap();
        let loaded: ProjectFile = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.patterns.len(), proj.patterns.len());
        assert_eq!(loaded.bpm, proj.bpm);
        assert_eq!(loaded.kits.len(), NUM_KITS);
        assert_eq!(loaded.kits[0].tracks.len(), NUM_DRUM_TRACKS);
        assert_eq!(loaded.active_kit, 0);
    }

    #[test]
    fn forward_compat_missing_fields() {
        // Simulate an old file with only legacy "kit" field (no "kits" array)
        let json = r#"{
            "textstep": {"format_version": 1},
            "kit": {
                "name": "Old Kit",
                "tracks": [{"id": "kick", "tune": 0.3, "decay": 0.5, "volume": 0.8}]
            },
            "patterns": [{"name": "P1", "steps": ["80000000"]}],
            "bpm": 130.0
        }"#;
        let mut proj: ProjectFile = serde_json::from_str(json).unwrap();
        proj.normalize();
        assert_eq!(proj.patterns.len(), NUM_PATTERNS);
        // Legacy kit migrated into kits[0]
        assert_eq!(proj.kits.len(), NUM_KITS);
        assert_eq!(proj.kits[0].name, "Old Kit");
        assert_eq!(proj.kits[0].tracks[0].params.tune, 0.3);
        // Missing fields get defaults
        assert_eq!(proj.kits[0].tracks[0].params.sweep, default_third());
        assert_eq!(proj.kits[0].tracks[0].params.filter, default_half());
        // Remaining kits are defaults
        assert_eq!(proj.kits[1].name, "Kit 2");
    }

    #[test]
    fn test_project_roundtrip_dual_synth() {
        // Create a project with synth B data
        let mut project = ProjectFile::default();

        // Set synth B pattern data
        project.synth_b_patterns[0].name = "Test Synth B Pattern".to_string();
        project.synth_b_patterns[1].name = "Custom Pattern".to_string();
        project.synth_b_patterns[1].steps[5] = SynthStepData {
            active: true,
            note: 60,
            velocity: 0.8,
            gate: 0.9,
            length: 4,
        };

        // Set synth B kit data
        project.synth_b_kits[0].name = "Test Synth B Kit".to_string();
        project.synth_b_kits[0].params.osc1_level = 0.75;

        project.active_synth_b_kit = 2;
        project.active_synth_b_pattern = 3;

        // Serialize
        let json = serde_json::to_string(&project).unwrap();

        // Deserialize
        let mut loaded: ProjectFile = serde_json::from_str(&json).unwrap();
        loaded.normalize();

        // Verify synth B data survived
        assert_eq!(loaded.synth_b_patterns[0].name, "Test Synth B Pattern");
        assert_eq!(loaded.synth_b_patterns[1].name, "Custom Pattern");
        assert_eq!(loaded.synth_b_patterns[1].steps[5].active, true);
        assert_eq!(loaded.synth_b_patterns[1].steps[5].note, 60);
        assert_eq!(loaded.synth_b_patterns[1].steps[5].velocity, 0.8);
        assert_eq!(loaded.synth_b_kits[0].name, "Test Synth B Kit");
        assert_eq!(loaded.synth_b_kits[0].params.osc1_level, 0.75);
        assert_eq!(loaded.active_synth_b_kit, 2);
        assert_eq!(loaded.active_synth_b_pattern, 3);

        // Verify arrays are properly sized
        assert_eq!(loaded.synth_b_patterns.len(), NUM_PATTERNS);
        assert_eq!(loaded.synth_b_kits.len(), NUM_KITS);
    }

    #[test]
    fn test_old_project_loads_with_synth_b_defaults() {
        // Simulate an old project file without synth_b fields
        let json = r#"{
            "textstep": {"format_version": 1, "app_version": "0.1.0"},
            "metadata": {"name": "Old Project"},
            "kits": [{"name": "Kit 1", "tracks": []}],
            "active_kit": 0,
            "patterns": [{"name": "P1", "steps": []}],
            "active_pattern": 0,
            "bpm": 120.0,
            "loop_length": 32,
            "swing": 0.5,
            "synth_kits": [{"name": "Synth Kit 1", "params": {}}],
            "active_synth_kit": 0,
            "synth_patterns": [{"name": "Synth 1", "steps": []}],
            "active_synth_pattern": 0
        }"#;

        let mut project: ProjectFile = serde_json::from_str(json).unwrap();
        project.normalize();

        // Verify synth_b fields get defaults
        assert_eq!(project.synth_b_patterns.len(), NUM_PATTERNS);
        assert_eq!(project.synth_b_kits.len(), NUM_KITS);
        assert_eq!(project.active_synth_b_kit, 0);
        assert_eq!(project.active_synth_b_pattern, 0);

        // Default names should be present
        assert_eq!(project.synth_b_patterns[0].name, "Synth B 1");
        assert_eq!(project.synth_b_kits[0].name, "Synth B Kit 1");

        // Old data should be intact
        assert_eq!(project.metadata.name, "Old Project");
        assert_eq!(project.synth_patterns[0].name, "Synth 1");
    }

    #[test]
    fn demo_project_has_scenes() {
        let proj = demo_project();
        assert!(!proj.scenes.is_empty());
        let populated = proj.scenes.iter().filter(|s| s.is_some()).count();
        assert!(populated >= 5, "Expected at least 5 demo scenes, got {}", populated);
    }
}

#[cfg(test)]
mod demo_tests {
    use super::*;

    #[test]
    fn demo_project_has_10_patterns() {
        let proj = demo_project();
        assert_eq!(proj.patterns.len(), 10);
    }

    #[test]
    fn demo_patterns_have_bpm() {
        let proj = demo_project();
        assert!((proj.patterns[0].bpm - 121.0).abs() < 0.01); // Around the World
        assert!((proj.patterns[7].bpm - 174.0).abs() < 0.01); // D&B
        assert!((proj.patterns[9].bpm - 118.0).abs() < 0.01); // Dub Techno
    }

    #[test]
    fn demo_patterns_have_nonempty_steps() {
        let proj = demo_project();
        for (i, pat) in proj.patterns.iter().enumerate() {
            let has_notes = pat.steps.iter().any(|s| s != "00000000");
            assert!(has_notes, "Pattern {} ({}) has no steps", i, pat.name);
        }
    }

    #[test]
    fn demo_synth_kits_have_presets() {
        let proj = demo_project();
        assert_eq!(proj.synth_kits.len(), NUM_KITS);
        assert_eq!(proj.synth_b_kits.len(), NUM_KITS);
        // Verify kits have named presets (not default names)
        assert_eq!(proj.synth_kits[0].name, "Electric Piano");
        assert_eq!(proj.synth_b_kits[0].name, "Acid Bass");
    }

    #[test]
    fn demo_synth_patterns_have_notes() {
        let proj = demo_project();
        assert_eq!(proj.synth_patterns.len(), NUM_PATTERNS);
        assert_eq!(proj.synth_b_patterns.len(), NUM_PATTERNS);
        for (i, pat) in proj.synth_patterns.iter().enumerate() {
            let has_notes = pat.steps.iter().any(|s| s.active);
            assert!(has_notes, "Synth A pattern {} ({}) has no notes", i, pat.name);
        }
        for (i, pat) in proj.synth_b_patterns.iter().enumerate() {
            let has_notes = pat.steps.iter().any(|s| s.active);
            assert!(has_notes, "Synth B pattern {} ({}) has no notes", i, pat.name);
        }
    }

    #[test]
    fn demo_serialize_roundtrip() {
        let proj = demo_project();
        let json = serde_json::to_string(&proj).unwrap();
        let loaded: ProjectFile = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.patterns.len(), 10);
        assert_eq!(loaded.patterns[0].name, "Around the World 121");
        assert!((loaded.patterns[0].bpm - 121.0).abs() < 0.01);
        assert_eq!(loaded.synth_kits[0].name, "Electric Piano");
    }

    /// Render all genre kit voices to WAV files for auditioning.
    /// Run with: cargo test render_kits -- --ignored --nocapture
    /// Output: kit_renders/<kit_name>/<voice>.wav
    #[test]
    #[ignore]
    fn render_kits_to_wav() {
        use crate::audio::drum_voice::create_drum_voices;
        use crate::sequencer::drum_pattern::TRACK_IDS;
        use std::fs;
        use std::io::Write;
        use std::path::PathBuf;

        const SAMPLE_RATE: f64 = 44100.0;
        const RENDER_SAMPLES: usize = (44100.0 * 1.5) as usize; // 1.5s per hit

        fn write_wav(path: &PathBuf, samples: &[f32], sample_rate: u32) {
            let num_samples = samples.len() as u32;
            let data_size = num_samples * 2; // 16-bit mono
            let file_size = 36 + data_size;
            let mut f = fs::File::create(path).expect("Failed to create WAV");
            // RIFF header
            f.write_all(b"RIFF").unwrap();
            f.write_all(&file_size.to_le_bytes()).unwrap();
            f.write_all(b"WAVE").unwrap();
            // fmt chunk
            f.write_all(b"fmt ").unwrap();
            f.write_all(&16u32.to_le_bytes()).unwrap();
            f.write_all(&1u16.to_le_bytes()).unwrap(); // PCM
            f.write_all(&1u16.to_le_bytes()).unwrap(); // mono
            f.write_all(&sample_rate.to_le_bytes()).unwrap();
            f.write_all(&(sample_rate * 2).to_le_bytes()).unwrap(); // byte rate
            f.write_all(&2u16.to_le_bytes()).unwrap(); // block align
            f.write_all(&16u16.to_le_bytes()).unwrap(); // bits per sample
            // data chunk
            f.write_all(b"data").unwrap();
            f.write_all(&data_size.to_le_bytes()).unwrap();
            for &s in samples {
                let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                f.write_all(&v.to_le_bytes()).unwrap();
            }
        }

        let kits = genre_kits();
        let voice_names: Vec<&str> = TRACK_IDS.iter().map(|id| id.name()).collect();
        let output_dir = PathBuf::from("kit_renders");
        fs::create_dir_all(&output_dir).unwrap();

        println!("\nRendering {} kits x {} voices...\n", kits.len(), voice_names.len());

        for kit in &kits {
            let kit_dir = output_dir.join(&kit.name);
            fs::create_dir_all(&kit_dir).unwrap();
            let mut voices = create_drum_voices(SAMPLE_RATE);

            for (i, track) in kit.tracks.iter().enumerate() {
                let params = track.params.to_track_params();
                voices[i].trigger(&params);

                let mut samples = Vec::with_capacity(RENDER_SAMPLES);
                let mut peak: f32 = 0.0;
                for _ in 0..RENDER_SAMPLES {
                    let s = voices[i].tick() * params.volume;
                    peak = peak.max(s.abs());
                    samples.push(s);
                }

                // Normalize to -1dB headroom
                if peak > 0.001 {
                    let gain = 0.89 / peak;
                    for s in &mut samples { *s *= gain; }
                }

                let filename = format!("{}_{}.wav", i + 1, voice_names[i].to_lowercase());
                let path = kit_dir.join(&filename);
                write_wav(&path, &samples, SAMPLE_RATE as u32);
                println!("  {:>8} / {:<12} peak={:.3}  -> {}/{}",
                    kit.name, voice_names[i], peak, kit.name, filename);
            }
            println!();
        }
        println!("Done! Files in: {}/", output_dir.display());
    }

    #[test]
    fn scene_serializes_roundtrip() {
        let scene = Scene {
            name: "Test Scene".to_string(),
            drum_pattern: 2,
            drum_kit: 3,
            synth_a_pattern: 4,
            synth_a_kit: 1,
            synth_b_pattern: 5,
            synth_b_kit: 6,
            bpm: 130.0,
            swing: 0.55,
        };
        let json = serde_json::to_string(&scene).unwrap();
        let loaded: Scene = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "Test Scene");
        assert_eq!(loaded.drum_pattern, 2);
        assert_eq!(loaded.drum_kit, 3);
        assert_eq!(loaded.synth_a_pattern, 4);
        assert_eq!(loaded.synth_a_kit, 1);
        assert_eq!(loaded.synth_b_pattern, 5);
        assert_eq!(loaded.synth_b_kit, 6);
        assert!((loaded.bpm - 130.0).abs() < 0.01);
        assert!((loaded.swing - 0.55).abs() < 0.01);
    }

    #[test]
    fn old_project_without_scenes_loads() {
        let json = r#"{
            "textstep": {"format_version": 1},
            "kit": {"tracks": []},
            "kits": [],
            "active_kit": 0,
            "patterns": [],
            "active_pattern": 0,
            "active_synth_kit": 0,
            "active_synth_pattern": 0
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(project.scenes.is_empty());
    }

    #[test]
    fn normalize_clamps_scene_indices() {
        let mut project = ProjectFile::default();
        project.scenes = vec![Some(Scene {
            name: "Bad".to_string(),
            drum_pattern: 99,
            drum_kit: 99,
            synth_a_pattern: 99,
            synth_a_kit: 99,
            synth_b_pattern: 99,
            synth_b_kit: 99,
            bpm: 130.0,
            swing: 0.5,
        })];
        project.normalize();
        let s = project.scenes[0].as_ref().unwrap();
        assert!(s.drum_pattern < NUM_PATTERNS);
        assert!(s.drum_kit < NUM_KITS);
        assert!(s.synth_a_pattern < NUM_PATTERNS);
        assert!(s.synth_a_kit < NUM_KITS);
        assert!(s.synth_b_pattern < NUM_PATTERNS);
        assert!(s.synth_b_kit < NUM_KITS);
    }
}
