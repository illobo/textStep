//! Synth pattern data: 32 steps with note/velocity/length, plus full synth parameter set.

use serde::{Serialize, Deserialize};

pub const MAX_STEPS: usize = 32;

/// A single synth sequencer step: MIDI note, velocity (0 = off), and length in steps.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SynthStep {
    pub note: u8,     // MIDI note number (0-127). 60 = C4
    pub velocity: u8, // 0 = step off, 1-127 = on with velocity
    #[serde(default = "default_length")]
    pub length: u8,   // 1-32, number of steps the note spans
}

fn default_length() -> u8 { 1 }
fn default_lfo2_waveform() -> u8 { 0 } // Sine
fn default_lfo2_division() -> f32 { 0.47 } // ~1/4 note
fn default_lfo2_dest() -> u8 { 2 } // Osc1Tune (pitch vibrato)

impl Default for SynthStep {
    fn default() -> Self {
        Self {
            note: 60,
            velocity: 0,
            length: 1,
        }
    }
}

impl SynthStep {
    pub fn is_active(&self) -> bool {
        self.velocity > 0
    }

    pub fn note_name(&self) -> String {
        const NAMES: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F",
            "F#", "G", "G#", "A", "A#", "B",
        ];
        let name = NAMES[(self.note % 12) as usize];
        let octave = (self.note as i8 / 12) - 1;
        format!("{}{}", name, octave)
    }
}

/// Complete synth parameter set: dual oscillators, sub, two ADSR envelopes,
/// filter with its own envelope, LFO routing, and send effect levels.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SynthParams {
    // Oscillator 1
    #[serde(default)]
    pub osc1_waveform: u8, // 0=Square, 1=Saw, 2=Sine, 3=Noise
    #[serde(default)]
    pub osc1_tune: f32,    // 0.0-1.0, maps to -24..+24 semitones
    #[serde(default)]
    pub osc1_pwm: f32,     // 0.0-1.0, pulse width / fold / supersaw detune / noise color
    #[serde(default)]
    pub osc1_level: f32,   // 0.0-1.0

    // Oscillator 2
    #[serde(default)]
    pub osc2_waveform: u8, // 0=Square, 1=Saw, 2=Sine, 3=Noise
    #[serde(default)]
    pub osc2_tune: f32,    // 0.0-1.0, maps to -24..+24 semitones
    #[serde(default)]
    pub osc2_pwm: f32,     // 0.0-1.0
    #[serde(default)]
    pub osc2_level: f32,   // 0.0-1.0
    #[serde(default)]
    pub osc2_detune: f32,  // 0.0-1.0, maps to -50..+50 cents (0.5 = no detune)

    // Sub oscillator (square, 1 oct below osc2)
    #[serde(default)]
    pub sub_level: f32, // 0.0-1.0

    // Envelope 1 (osc1 amplitude)
    #[serde(default)]
    pub env1_attack: f32,
    #[serde(default)]
    pub env1_decay: f32,
    #[serde(default)]
    pub env1_sustain: f32,
    #[serde(default)]
    pub env1_release: f32,

    // Envelope 2 (osc2 + sub amplitude)
    #[serde(default)]
    pub env2_attack: f32,
    #[serde(default)]
    pub env2_decay: f32,
    #[serde(default)]
    pub env2_sustain: f32,
    #[serde(default)]
    pub env2_release: f32,

    // Filter (24dB multimode)
    #[serde(default)]
    pub filter_type: u8,        // 0=LP, 1=HP, 2=BP
    #[serde(default)]
    pub filter_cutoff: f32,
    #[serde(default)]
    pub filter_resonance: f32,
    #[serde(default)]
    pub filter_env_amount: f32, // 0.0-1.0

    // Filter envelope
    #[serde(default)]
    pub filter_env_attack: f32,
    #[serde(default)]
    pub filter_env_decay: f32,
    #[serde(default)]
    pub filter_env_sustain: f32,
    #[serde(default)]
    pub filter_env_release: f32,

    // LFO1
    #[serde(default)]
    pub lfo_waveform: u8,   // 0=Sine, 1=Triangle, 2=SawDn, 3=SawUp, 4=Square, 5=Exp
    #[serde(default)]
    pub lfo_division: f32,  // 0.0-1.0, mapped to beat divisions
    #[serde(default)]
    pub lfo_depth: f32,     // 0.0-1.0
    #[serde(default)]
    pub lfo_dest: u8,       // index into LFO_DEST_FIELDS

    // LFO2
    #[serde(default = "default_lfo2_waveform")]
    pub lfo2_waveform: u8,
    #[serde(default = "default_lfo2_division")]
    pub lfo2_division: f32,
    #[serde(default)]
    pub lfo2_depth: f32,
    #[serde(default = "default_lfo2_dest")]
    pub lfo2_dest: u8,

    // Output
    #[serde(default)]
    pub volume: f32,
    #[serde(default)]
    pub send_reverb: f32,
    #[serde(default)]
    pub send_delay: f32,

    // Runtime (not saved)
    #[serde(skip)]
    pub mute: bool,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            // Osc1: Square, centered tune, mid PWM, audible
            osc1_waveform: 0,
            osc1_tune: 0.5,
            osc1_pwm: 0.5,
            osc1_level: 0.8,

            // Osc2: Saw, off by default
            osc2_waveform: 1,
            osc2_tune: 0.5,
            osc2_pwm: 0.0,
            osc2_level: 0.0,
            osc2_detune: 0.5,

            // Sub: off
            sub_level: 0.0,

            // Env1: nice pluck/pad
            env1_attack: 0.01,
            env1_decay: 0.3,
            env1_sustain: 0.7,
            env1_release: 0.2,

            // Env2: same
            env2_attack: 0.01,
            env2_decay: 0.3,
            env2_sustain: 0.7,
            env2_release: 0.2,

            // Filter: mostly open, LP
            filter_type: 0,
            filter_cutoff: 0.7,
            filter_resonance: 0.0,
            filter_env_amount: 0.0,

            // Filter env
            filter_env_attack: 0.0,
            filter_env_decay: 0.3,
            filter_env_sustain: 0.0,
            filter_env_release: 0.2,

            // LFO1: off by default
            lfo_waveform: 1, // Triangle
            lfo_division: 0.47, // ~1/4 note
            lfo_depth: 0.0,
            lfo_dest: 0, // FilterCutoff

            // LFO2: off by default
            lfo2_waveform: 0, // Sine
            lfo2_division: 0.47,
            lfo2_depth: 0.0,
            lfo2_dest: 2, // Osc1Tune (pitch vibrato)

            // Output
            volume: 0.8,
            send_reverb: 0.2,
            send_delay: 0.0,

            mute: false,
        }
    }
}

// --- SynthPattern ---

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SynthPattern {
    pub steps: [SynthStep; MAX_STEPS],
    pub params: SynthParams,
}

impl Default for SynthPattern {
    fn default() -> Self {
        Self {
            steps: [SynthStep::default(); MAX_STEPS],
            params: SynthParams::default(),
        }
    }
}

// --- SynthControlField ---

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SynthControlField {
    // Osc1
    Osc1Waveform,
    Osc1Tune,
    Osc1Pwm,
    Osc1Level,
    // Osc2
    Osc2Waveform,
    Osc2Tune,
    Osc2Pwm,
    Osc2Level,
    Osc2Detune,
    // Sub
    SubLevel,
    // Env1
    Env1Attack,
    Env1Decay,
    Env1Sustain,
    Env1Release,
    // Env2
    Env2Attack,
    Env2Decay,
    Env2Sustain,
    Env2Release,
    // Filter
    FilterType,
    FilterCutoff,
    FilterResonance,
    FilterEnvAmount,
    // Filter Env
    FilterEnvAttack,
    FilterEnvDecay,
    FilterEnvSustain,
    FilterEnvRelease,
    // LFO1
    LfoWaveform,
    LfoDivision,
    LfoDepth,
    LfoDest,
    // LFO2
    Lfo2Waveform,
    Lfo2Division,
    Lfo2Depth,
    Lfo2Dest,
    // Output
    Volume,
    SendReverb,
    SendDelay,
    // Runtime
    Mute,
}

static ALL_FIELDS: [SynthControlField; 38] = [
    SynthControlField::Osc1Waveform,
    SynthControlField::Osc1Tune,
    SynthControlField::Osc1Pwm,
    SynthControlField::Osc1Level,
    SynthControlField::Osc2Waveform,
    SynthControlField::Osc2Tune,
    SynthControlField::Osc2Pwm,
    SynthControlField::Osc2Level,
    SynthControlField::Osc2Detune,
    SynthControlField::SubLevel,
    SynthControlField::Env1Attack,
    SynthControlField::Env1Decay,
    SynthControlField::Env1Sustain,
    SynthControlField::Env1Release,
    SynthControlField::Env2Attack,
    SynthControlField::Env2Decay,
    SynthControlField::Env2Sustain,
    SynthControlField::Env2Release,
    SynthControlField::FilterType,
    SynthControlField::FilterCutoff,
    SynthControlField::FilterResonance,
    SynthControlField::FilterEnvAmount,
    SynthControlField::FilterEnvAttack,
    SynthControlField::FilterEnvDecay,
    SynthControlField::FilterEnvSustain,
    SynthControlField::FilterEnvRelease,
    SynthControlField::LfoWaveform,
    SynthControlField::LfoDivision,
    SynthControlField::LfoDepth,
    SynthControlField::LfoDest,
    SynthControlField::Lfo2Waveform,
    SynthControlField::Lfo2Division,
    SynthControlField::Lfo2Depth,
    SynthControlField::Lfo2Dest,
    SynthControlField::Volume,
    SynthControlField::SendReverb,
    SynthControlField::SendDelay,
    SynthControlField::Mute,
];

impl SynthControlField {
    pub fn label(&self) -> &str {
        match self {
            Self::Osc1Waveform => "WV",
            Self::Osc1Tune => "TN",
            Self::Osc1Pwm => "PW",
            Self::Osc1Level => "LV",
            Self::Osc2Waveform => "WV",
            Self::Osc2Tune => "TN",
            Self::Osc2Pwm => "PW",
            Self::Osc2Level => "LV",
            Self::Osc2Detune => "DT",
            Self::SubLevel => "SB",
            Self::Env1Attack => "A",
            Self::Env1Decay => "D",
            Self::Env1Sustain => "S",
            Self::Env1Release => "R",
            Self::Env2Attack => "A",
            Self::Env2Decay => "D",
            Self::Env2Sustain => "S",
            Self::Env2Release => "R",
            Self::FilterType => "FT",
            Self::FilterCutoff => "FR",
            Self::FilterResonance => "RS",
            Self::FilterEnvAmount => "EA",
            Self::FilterEnvAttack => "A",
            Self::FilterEnvDecay => "D",
            Self::FilterEnvSustain => "S",
            Self::FilterEnvRelease => "R",
            Self::LfoWaveform => "WV",
            Self::LfoDivision => "DV",
            Self::LfoDepth => "DP",
            Self::LfoDest => "DS",
            Self::Lfo2Waveform => "WV",
            Self::Lfo2Division => "DV",
            Self::Lfo2Depth => "DP",
            Self::Lfo2Dest => "DS",
            Self::Volume => "VL",
            Self::SendReverb => "RV",
            Self::SendDelay => "DL",
            Self::Mute => "M",
        }
    }

    pub fn full_label(&self) -> &str {
        match self {
            Self::Osc1Waveform => "Wave",
            Self::Osc1Tune => "Tune",
            Self::Osc1Pwm => "PWM",
            Self::Osc1Level => "Level",
            Self::Osc2Waveform => "Wave",
            Self::Osc2Tune => "Tune",
            Self::Osc2Pwm => "PWM",
            Self::Osc2Level => "Level",
            Self::Osc2Detune => "Detune",
            Self::SubLevel => "Sub",
            Self::Env1Attack => "Atk",
            Self::Env1Decay => "Dec",
            Self::Env1Sustain => "Sus",
            Self::Env1Release => "Rel",
            Self::Env2Attack => "Atk",
            Self::Env2Decay => "Dec",
            Self::Env2Sustain => "Sus",
            Self::Env2Release => "Rel",
            Self::FilterType => "Type",
            Self::FilterCutoff => "Freq",
            Self::FilterResonance => "Res",
            Self::FilterEnvAmount => "EnvAmt",
            Self::FilterEnvAttack => "Atk",
            Self::FilterEnvDecay => "Dec",
            Self::FilterEnvSustain => "Sus",
            Self::FilterEnvRelease => "Rel",
            Self::LfoWaveform => "Wave",
            Self::LfoDivision => "Div",
            Self::LfoDepth => "Depth",
            Self::LfoDest => "Dest",
            Self::Lfo2Waveform => "Wave",
            Self::Lfo2Division => "Div",
            Self::Lfo2Depth => "Depth",
            Self::Lfo2Dest => "Dest",
            Self::Volume => "Vol",
            Self::SendReverb => "Reverb",
            Self::SendDelay => "Delay",
            Self::Mute => "Mute",
        }
    }

    pub fn get(&self, p: &SynthParams) -> f32 {
        match self {
            Self::Osc1Waveform => p.osc1_waveform as f32 / 3.0,
            Self::Osc1Tune => p.osc1_tune,
            Self::Osc1Pwm => p.osc1_pwm,
            Self::Osc1Level => p.osc1_level,
            Self::Osc2Waveform => p.osc2_waveform as f32 / 3.0,
            Self::Osc2Tune => p.osc2_tune,
            Self::Osc2Pwm => p.osc2_pwm,
            Self::Osc2Level => p.osc2_level,
            Self::Osc2Detune => p.osc2_detune,
            Self::SubLevel => p.sub_level,
            Self::Env1Attack => p.env1_attack,
            Self::Env1Decay => p.env1_decay,
            Self::Env1Sustain => p.env1_sustain,
            Self::Env1Release => p.env1_release,
            Self::Env2Attack => p.env2_attack,
            Self::Env2Decay => p.env2_decay,
            Self::Env2Sustain => p.env2_sustain,
            Self::Env2Release => p.env2_release,
            Self::FilterType => p.filter_type as f32 / 2.0,
            Self::FilterCutoff => p.filter_cutoff,
            Self::FilterResonance => p.filter_resonance,
            Self::FilterEnvAmount => p.filter_env_amount,
            Self::FilterEnvAttack => p.filter_env_attack,
            Self::FilterEnvDecay => p.filter_env_decay,
            Self::FilterEnvSustain => p.filter_env_sustain,
            Self::FilterEnvRelease => p.filter_env_release,
            Self::LfoWaveform => p.lfo_waveform as f32 / (NUM_LFO_WAVEFORMS - 1) as f32,
            Self::LfoDivision => p.lfo_division,
            Self::LfoDepth => p.lfo_depth,
            Self::LfoDest => p.lfo_dest as f32 / (LFO_DEST_FIELDS.len() - 1).max(1) as f32,
            Self::Lfo2Waveform => p.lfo2_waveform as f32 / (NUM_LFO_WAVEFORMS - 1) as f32,
            Self::Lfo2Division => p.lfo2_division,
            Self::Lfo2Depth => p.lfo2_depth,
            Self::Lfo2Dest => p.lfo2_dest as f32 / (LFO_DEST_FIELDS.len() - 1).max(1) as f32,
            Self::Volume => p.volume,
            Self::SendReverb => p.send_reverb,
            Self::SendDelay => p.send_delay,
            Self::Mute => if p.mute { 1.0 } else { 0.0 },
        }
    }

    pub fn set(&self, p: &mut SynthParams, v: f32) {
        let v = v.clamp(0.0, 1.0);
        match self {
            Self::Osc1Waveform => p.osc1_waveform = (v * 3.0).round() as u8,
            Self::Osc1Tune => p.osc1_tune = v,
            Self::Osc1Pwm => p.osc1_pwm = v,
            Self::Osc1Level => p.osc1_level = v,
            Self::Osc2Waveform => p.osc2_waveform = (v * 3.0).round() as u8,
            Self::Osc2Tune => p.osc2_tune = v,
            Self::Osc2Pwm => p.osc2_pwm = v,
            Self::Osc2Level => p.osc2_level = v,
            Self::Osc2Detune => p.osc2_detune = v,
            Self::SubLevel => p.sub_level = v,
            Self::Env1Attack => p.env1_attack = v,
            Self::Env1Decay => p.env1_decay = v,
            Self::Env1Sustain => p.env1_sustain = v,
            Self::Env1Release => p.env1_release = v,
            Self::Env2Attack => p.env2_attack = v,
            Self::Env2Decay => p.env2_decay = v,
            Self::Env2Sustain => p.env2_sustain = v,
            Self::Env2Release => p.env2_release = v,
            Self::FilterType => p.filter_type = (v * 2.0).round() as u8,
            Self::FilterCutoff => p.filter_cutoff = v,
            Self::FilterResonance => p.filter_resonance = v,
            Self::FilterEnvAmount => p.filter_env_amount = v,
            Self::FilterEnvAttack => p.filter_env_attack = v,
            Self::FilterEnvDecay => p.filter_env_decay = v,
            Self::FilterEnvSustain => p.filter_env_sustain = v,
            Self::FilterEnvRelease => p.filter_env_release = v,
            Self::LfoWaveform => p.lfo_waveform = (v * (NUM_LFO_WAVEFORMS - 1) as f32).round() as u8,
            Self::LfoDivision => p.lfo_division = v,
            Self::LfoDepth => p.lfo_depth = v,
            Self::LfoDest => p.lfo_dest = (v * (LFO_DEST_FIELDS.len() - 1) as f32).round() as u8,
            Self::Lfo2Waveform => p.lfo2_waveform = (v * (NUM_LFO_WAVEFORMS - 1) as f32).round() as u8,
            Self::Lfo2Division => p.lfo2_division = v,
            Self::Lfo2Depth => p.lfo2_depth = v,
            Self::Lfo2Dest => p.lfo2_dest = (v * (LFO_DEST_FIELDS.len() - 1) as f32).round() as u8,
            Self::Volume => p.volume = v,
            Self::SendReverb => p.send_reverb = v,
            Self::SendDelay => p.send_delay = v,
            Self::Mute => p.mute = v > 0.5,
        }
    }

    pub fn all() -> &'static [SynthControlField] {
        &ALL_FIELDS
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Osc1Waveform | Self::Osc2Waveform | Self::FilterType
            | Self::LfoWaveform | Self::LfoDivision | Self::LfoDest
            | Self::Lfo2Waveform | Self::Lfo2Division | Self::Lfo2Dest)
    }
}

// --- Helper ---

pub fn filter_type_name(t: u8) -> &'static str {
    match t {
        0 => "LP",
        1 => "HP",
        2 => "BP",
        _ => "??",
    }
}

pub fn waveform_name(w: u8) -> &'static str {
    match w {
        0 => "Sqr",
        1 => "Saw",
        2 => "Sin",
        3 => "Nse",
        _ => "???",
    }
}

// --- LFO helpers ---

/// Assignable LFO destination fields (continuous params only).
pub const LFO_DEST_FIELDS: [SynthControlField; 11] = [
    SynthControlField::FilterCutoff,
    SynthControlField::FilterResonance,
    SynthControlField::Osc1Tune,
    SynthControlField::Osc1Pwm,
    SynthControlField::Osc1Level,
    SynthControlField::Osc2Tune,
    SynthControlField::Osc2Pwm,
    SynthControlField::Osc2Level,
    SynthControlField::Osc2Detune,
    SynthControlField::SubLevel,
    SynthControlField::Volume,
];

pub const NUM_LFO_WAVEFORMS: u8 = 6;

pub fn lfo_waveform_name(w: u8) -> &'static str {
    match w {
        0 => "Sin",
        1 => "Tri",
        2 => "Saw\u{2193}",
        3 => "Saw\u{2191}",
        4 => "Sqr",
        5 => "Exp",
        _ => "???",
    }
}

/// Beat division multipliers (cycles per beat) and display names.
pub const LFO_DIVISIONS: [(f64, &str); 18] = [
    (8.0,       "1/32"),
    (6.0,       "1/16T"),
    (4.0,       "1/16"),
    (2.666667,  "1/16D"),
    (3.0,       "1/8T"),
    (2.0,       "1/8"),
    (1.333333,  "1/8D"),
    (1.5,       "1/4T"),
    (1.0,       "1/4"),
    (0.666667,  "1/4D"),
    (0.75,      "1/2T"),
    (0.5,       "1/2"),
    (0.333333,  "1/2D"),
    (0.25,      "1bar"),
    (0.125,     "2bar"),
    (0.0625,    "4bar"),
    (0.03125,   "8bar"),
    (0.015625,  "16b"),
];

/// Map normalized 0.0-1.0 to a division index (0-9).
pub fn lfo_division_index(v: f32) -> usize {
    (v * (LFO_DIVISIONS.len() - 1) as f32).round() as usize
}

/// Get the cycles-per-beat multiplier for a division parameter.
pub fn lfo_division_multiplier(v: f32) -> f64 {
    LFO_DIVISIONS[lfo_division_index(v)].0
}

/// Get the display name for a division parameter.
pub fn lfo_division_name(v: f32) -> &'static str {
    LFO_DIVISIONS[lfo_division_index(v)].1
}

/// Unique display names for LFO destination targets.
const LFO_DEST_NAMES: [&str; 11] = [
    "Freq",    // FilterCutoff
    "Res",     // FilterResonance
    "O1Tune",  // Osc1Tune
    "O1PWM",   // Osc1Pwm
    "O1Lvl",   // Osc1Level
    "O2Tune",  // Osc2Tune
    "O2PWM",   // Osc2Pwm
    "O2Lvl",   // Osc2Level
    "O2Det",   // Osc2Detune
    "Sub",     // SubLevel
    "Vol",     // Volume
];

/// Get display name for an LFO destination index.
pub fn lfo_dest_name(idx: u8) -> &'static str {
    if (idx as usize) < LFO_DEST_NAMES.len() {
        LFO_DEST_NAMES[idx as usize]
    } else {
        "???"
    }
}
