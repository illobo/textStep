//! Synth sound presets: pads, leads, basses, keys, and FX organized by category.

use crate::sequencer::synth_pattern::SynthParams;
use super::SynthSoundPreset;

// Helper: synth preset with all params
const fn sp(
    osc1_waveform: u8, osc1_tune: f32, osc1_pwm: f32, osc1_level: f32,
    osc2_waveform: u8, osc2_tune: f32, osc2_pwm: f32, osc2_level: f32, osc2_detune: f32,
    sub_level: f32,
    env1_a: f32, env1_d: f32, env1_s: f32, env1_r: f32,
    env2_a: f32, env2_d: f32, env2_s: f32, env2_r: f32,
    filter_type: u8, filter_cutoff: f32, filter_resonance: f32, filter_env_amount: f32,
    fenv_a: f32, fenv_d: f32, fenv_s: f32, fenv_r: f32,
    volume: f32,
) -> SynthParams {
    SynthParams {
        osc1_waveform, osc1_tune, osc1_pwm, osc1_level,
        osc2_waveform, osc2_tune, osc2_pwm, osc2_level, osc2_detune,
        sub_level,
        env1_attack: env1_a, env1_decay: env1_d, env1_sustain: env1_s, env1_release: env1_r,
        env2_attack: env2_a, env2_decay: env2_d, env2_sustain: env2_s, env2_release: env2_r,
        filter_type, filter_cutoff, filter_resonance, filter_env_amount,
        filter_env_attack: fenv_a, filter_env_decay: fenv_d, filter_env_sustain: fenv_s, filter_env_release: fenv_r,
        lfo_waveform: 1, lfo_division: 0.47, lfo_depth: 0.0, lfo_dest: 0,
        lfo2_waveform: 0, lfo2_division: 0.47, lfo2_depth: 0.0, lfo2_dest: 2,
        volume,
        send_reverb: 0.2, send_delay: 0.0,
        mute: false,
    }
}

/// Override LFO settings on a preset
const fn with_lfo(mut p: SynthParams, wave: u8, div: f32, depth: f32, dest: u8) -> SynthParams {
    p.lfo_waveform = wave;
    p.lfo_division = div;
    p.lfo_depth = depth;
    p.lfo_dest = dest;
    p
}

pub static SYNTH_PRESETS: &[SynthSoundPreset] = &[
    // ── Bass ─────────────────────────────────────────────────────────────
    SynthSoundPreset {
        name: "Sub Bass", category: "Bass",
        params: sp(2, 0.5, 0.5, 0.8,  // Osc1: Sine, center, -, full
                   2, 0.5, 0.0, 0.0, 0.5,  // Osc2: off
                   0.5,  // Sub: half
                   0.01, 0.1, 1.0, 0.15,   // Env1: instant attack, full sustain
                   0.01, 0.1, 1.0, 0.15,   // Env2
                   0, 0.3, 0.0, 0.0,       // Filter: LP, low cutoff, no env
                   0.0, 0.3, 0.0, 0.2,     // Filter env
                   0.85),
    },
    SynthSoundPreset {
        name: "Acid Bass", category: "Bass",
        params: sp(0, 0.5, 0.5, 0.9,  // Osc1: Square
                   0, 0.5, 0.5, 0.0, 0.5,
                   0.3,
                   0.01, 0.64, 0.36, 0.05,  // Env1: snappy attack, medium decay, some sustain
                   0.01, 0.24, 0.44, 0.05,  // Env2
                   0, 0.73, 0.70, 0.16,     // Filter: LP, open cutoff, high reso, subtle env
                   0.40, 0.25, 0.52, 0.10,  // Filter env: slow attack, medium sustain
                   0.80),
    },
    SynthSoundPreset {
        name: "Reese Bass", category: "Bass",
        // Two detuned saws, slow LFO on detune for phasing movement
        params: with_lfo(sp(1, 0.5, 0.0, 0.7,  // Osc1: Saw
                   1, 0.5, 0.0, 0.7, 0.52,  // Osc2: Saw, slight detune
                   0.4,
                   0.01, 0.1, 1.0, 0.2,
                   0.01, 0.1, 1.0, 0.2,
                   0, 0.35, 0.20, 0.15,     // LP, warm filter, subtle env
                   0.0, 0.5, 0.2, 0.2,
                   0.80),
                   0, 0.67, 0.15, 8),       // LFO: Sine, 1 bar, subtle, → Osc2 Detune
    },
    SynthSoundPreset {
        name: "Wobble Bass", category: "Bass",
        // Two detuned saws + sub, LFO wobbles the filter
        params: with_lfo(sp(1, 0.5, 0.0, 0.8,
                   1, 0.5, 0.0, 0.6, 0.55,
                   0.5,
                   0.01, 0.3, 0.8, 0.2,     // Sustained envelope
                   0.01, 0.3, 0.8, 0.2,
                   0, 0.45, 0.40, 0.10,      // Filter: moderate cutoff, some reso, light env
                   0.0, 0.3, 0.3, 0.2,
                   0.80),
                   1, 0.22, 0.5, 0),         // LFO: Tri, 1/8 note, strong, → Filter cutoff
    },
    SynthSoundPreset {
        name: "Pulse Bass", category: "Bass",
        // Square with PWM modulation via LFO for moving pulse width
        params: with_lfo(sp(0, 0.5, 0.3, 0.9,  // Square with narrow PWM
                   0, 0.5, 0.7, 0.4, 0.5,
                   0.6,
                   0.01, 0.15, 0.9, 0.1,
                   0.01, 0.15, 0.9, 0.1,
                   0, 0.40, 0.30, 0.12,
                   0.0, 0.2, 0.3, 0.1,
                   0.80),
                   1, 0.47, 0.3, 3),         // LFO: Tri, 1/4, moderate, → Osc1 PWM
    },

    // ── Lead ─────────────────────────────────────────────────────────────
    SynthSoundPreset {
        name: "Saw Lead", category: "Lead",
        // Detuned saws, subtle vibrato via LFO on pitch
        params: with_lfo(sp(1, 0.5, 0.0, 0.8,  // Osc1: Saw
                   1, 0.5, 0.0, 0.6, 0.53,  // Osc2: Saw, detuned
                   0.0,
                   0.02, 0.3, 0.75, 0.3,    // Env1: quick attack, good sustain
                   0.02, 0.3, 0.75, 0.3,
                   0, 0.65, 0.15, 0.15,      // Filter: open, subtle env
                   0.01, 0.25, 0.3, 0.3,
                   0.75),
                   0, 0.35, 0.08, 2),        // LFO: Sine, 1/4 triplet, subtle vibrato, → Osc1 Tune
    },
    SynthSoundPreset {
        name: "Square Lead", category: "Lead",
        params: sp(0, 0.5, 0.5, 0.8,
                   0, 0.5, 0.5, 0.0, 0.5,
                   0.0,
                   0.02, 0.2, 0.7, 0.25,    // Quick attack, sustained
                   0.02, 0.2, 0.7, 0.25,
                   0, 0.60, 0.10, 0.12,      // Filter: open, light env
                   0.01, 0.2, 0.3, 0.2,
                   0.75),
    },
    SynthSoundPreset {
        name: "Screamer", category: "Lead",
        // Aggressive detuned saws, vibrato for expression
        params: with_lfo(sp(1, 0.5, 0.0, 0.9,
                   1, 0.5, 0.0, 0.7, 0.55,
                   0.0,
                   0.05, 0.15, 1.0, 0.3,    // Full sustain
                   0.05, 0.15, 1.0, 0.3,
                   0, 0.70, 0.50, 0.25,      // High cutoff, high reso, moderate env
                   0.01, 0.2, 0.5, 0.3,
                   0.70),
                   0, 0.35, 0.12, 2),        // LFO: Sine, moderate rate, vibrato, → Osc1 Tune
    },

    // ── Pad ──────────────────────────────────────────────────────────────
    SynthSoundPreset {
        name: "Warm Pad", category: "Pad",
        // Slow filter LFO for gentle movement
        params: with_lfo(sp(1, 0.5, 0.0, 0.6,
                   1, 0.5, 0.0, 0.6, 0.52,
                   0.3,
                   0.40, 0.3, 0.8, 0.50,
                   0.40, 0.3, 0.8, 0.50,
                   0, 0.40, 0.10, 0.10,
                   0.3, 0.5, 0.3, 0.4,
                   0.70),
                   1, 0.78, 0.15, 0),        // LFO: Tri, 2 bars, gentle, → Filter
    },
    SynthSoundPreset {
        name: "Bright Pad", category: "Pad",
        // Slow PWM modulation for shimmer
        params: with_lfo(sp(1, 0.5, 0.0, 0.7,
                   1, 0.5, 0.0, 0.5, 0.54,
                   0.0,
                   0.35, 0.2, 0.9, 0.45,
                   0.35, 0.2, 0.9, 0.45,
                   0, 0.65, 0.20, 0.15,
                   0.2, 0.4, 0.4, 0.4,
                   0.65),
                   1, 0.67, 0.12, 0),        // LFO: Tri, 1 bar, subtle, → Filter
    },
    SynthSoundPreset {
        name: "Dark Pad", category: "Pad",
        // Very slow filter drift
        params: with_lfo(sp(0, 0.5, 0.5, 0.6,
                   0, 0.5, 0.5, 0.5, 0.52,
                   0.4,
                   0.50, 0.4, 0.7, 0.60,
                   0.50, 0.4, 0.7, 0.60,
                   0, 0.25, 0.15, 0.08,
                   0.4, 0.6, 0.2, 0.5,
                   0.70),
                   0, 0.88, 0.10, 0),        // LFO: Sine, 4 bars, very subtle, → Filter
    },
    SynthSoundPreset {
        name: "Ethereal", category: "Pad",
        // Sine + Saw, slow filter + detune drift
        params: with_lfo(sp(2, 0.5, 0.0, 0.5,
                   1, 0.5, 0.0, 0.4, 0.53,
                   0.2,
                   0.60, 0.3, 0.6, 0.70,
                   0.60, 0.3, 0.6, 0.70,
                   0, 0.50, 0.25, 0.12,
                   0.5, 0.7, 0.3, 0.6,
                   0.60),
                   0, 0.88, 0.18, 0),        // LFO: Sine, 4 bars, gentle, → Filter
    },

    // ── Pluck ────────────────────────────────────────────────────────────
    SynthSoundPreset {
        name: "Basic Pluck", category: "Pluck",
        params: sp(1, 0.5, 0.0, 0.8,
                   1, 0.5, 0.0, 0.5, 0.52,
                   0.0,
                   0.00, 0.20, 0.0, 0.10,
                   0.00, 0.20, 0.0, 0.10,
                   0, 0.50, 0.10, 0.50,
                   0.0, 0.15, 0.0, 0.1,
                   0.80),
    },
    SynthSoundPreset {
        name: "Bell Pluck", category: "Pluck",
        params: sp(2, 0.5, 0.0, 0.7,
                   0, 0.75, 0.5, 0.4, 0.5,  // Osc2: Square, +12 semi
                   0.0,
                   0.00, 0.30, 0.0, 0.15,
                   0.00, 0.30, 0.0, 0.15,
                   0, 0.55, 0.20, 0.40,
                   0.0, 0.20, 0.0, 0.15,
                   0.75),
    },
    SynthSoundPreset {
        name: "Marimba", category: "Pluck",
        params: sp(2, 0.5, 0.0, 0.9,  // Sine
                   2, 0.75, 0.0, 0.3, 0.5,  // Sine +12
                   0.0,
                   0.00, 0.25, 0.0, 0.10,
                   0.00, 0.15, 0.0, 0.08,
                   0, 0.60, 0.05, 0.30,
                   0.0, 0.10, 0.0, 0.1,
                   0.80),
    },

    // ── Stab ─────────────────────────────────────────────────────────────
    SynthSoundPreset {
        name: "Chord Stab", category: "Stab",
        params: sp(1, 0.5, 0.0, 0.8,
                   1, 0.5, 0.0, 0.7, 0.54,
                   0.3,
                   0.01, 0.10, 0.0, 0.08,
                   0.01, 0.10, 0.0, 0.08,
                   0, 0.55, 0.15, 0.50,
                   0.0, 0.08, 0.0, 0.05,
                   0.80),
    },
    SynthSoundPreset {
        name: "Hoover", category: "Stab",
        params: sp(1, 0.5, 0.0, 0.9,
                   1, 0.5, 0.0, 0.8, 0.58,
                   0.5,
                   0.02, 0.15, 0.5, 0.15,
                   0.02, 0.15, 0.5, 0.15,
                   0, 0.50, 0.30, 0.40,
                   0.0, 0.12, 0.2, 0.1,
                   0.75),
    },

    // ── Keys ─────────────────────────────────────────────────────────────
    SynthSoundPreset {
        name: "Electric Piano", category: "Keys",
        // Sine harmonics, gentle tremolo via LFO on volume
        params: with_lfo(sp(2, 0.5, 0.0, 0.7,  // Sine
                   2, 0.75, 0.0, 0.3, 0.5,  // Sine +12
                   0.0,
                   0.01, 0.40, 0.5, 0.30,
                   0.01, 0.25, 0.3, 0.20,
                   0, 0.55, 0.10, 0.15,
                   0.0, 0.30, 0.2, 0.25,
                   0.75),
                   0, 0.47, 0.08, 10),       // LFO: Sine, 1/4, subtle tremolo, → Volume
    },
    SynthSoundPreset {
        name: "Organ", category: "Keys",
        params: sp(0, 0.5, 0.5, 0.6,
                   0, 0.75, 0.5, 0.4, 0.5,  // +12 semitones
                   0.5,
                   0.01, 0.05, 1.0, 0.05,
                   0.01, 0.05, 1.0, 0.05,
                   0, 0.55, 0.05, 0.0,
                   0.0, 0.1, 0.0, 0.05,
                   0.70),
    },
    SynthSoundPreset {
        name: "Clav", category: "Keys",
        params: sp(0, 0.5, 0.3, 0.8,
                   0, 0.5, 0.7, 0.0, 0.5,
                   0.0,
                   0.00, 0.15, 0.0, 0.08,
                   0.00, 0.15, 0.0, 0.08,
                   0, 0.65, 0.20, 0.45,
                   0.0, 0.10, 0.0, 0.08,
                   0.80),
    },

    // ── FX ───────────────────────────────────────────────────────────────
    SynthSoundPreset {
        name: "Noise Sweep", category: "FX",
        params: sp(3, 0.5, 0.5, 0.8,  // Noise
                   3, 0.5, 0.5, 0.0, 0.5,
                   0.0,
                   0.30, 0.5, 0.0, 0.40,
                   0.30, 0.5, 0.0, 0.40,
                   0, 0.20, 0.40, 0.70,
                   0.0, 0.80, 0.0, 0.3,
                   0.65),
    },
    SynthSoundPreset {
        name: "Laser", category: "FX",
        params: sp(1, 0.5, 0.0, 0.9,
                   2, 0.5, 0.0, 0.5, 0.5,
                   0.0,
                   0.00, 0.10, 0.0, 0.05,
                   0.00, 0.10, 0.0, 0.05,
                   0, 0.80, 0.60, 0.90,
                   0.0, 0.08, 0.0, 0.05,
                   0.70),
    },
    SynthSoundPreset {
        name: "Wind", category: "FX",
        // Two noise sources, slow filter sweep via LFO
        params: with_lfo(sp(3, 0.5, 0.3, 0.6,
                   3, 0.5, 0.7, 0.4, 0.5,
                   0.0,
                   0.80, 0.3, 0.5, 0.80,
                   0.80, 0.3, 0.5, 0.80,
                   0, 0.35, 0.30, 0.10,
                   0.5, 0.8, 0.3, 0.6,
                   0.55),
                   0, 0.78, 0.25, 0),        // LFO: Sine, 2 bars, moderate, → Filter
    },
];

pub fn categories() -> Vec<&'static str> {
    let mut cats: Vec<&'static str> = Vec::new();
    for p in SYNTH_PRESETS {
        if !cats.contains(&p.category) {
            cats.push(p.category);
        }
    }
    cats
}
