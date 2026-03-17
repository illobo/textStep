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
        sub_waveform: 0, // Square (default)
        env1_attack: env1_a, env1_decay: env1_d, env1_sustain: env1_s, env1_release: env1_r,
        env2_attack: env2_a, env2_decay: env2_d, env2_sustain: env2_s, env2_release: env2_r,
        glide: 0.0,
        osc_sync: 0,
        filter_type, filter_cutoff, filter_resonance, filter_env_amount,
        filter_key_follow: 0.0,
        filter_env_attack: fenv_a, filter_env_decay: fenv_d, filter_env_sustain: fenv_s, filter_env_release: fenv_r,
        lfo_waveform: 1, lfo_division: 0.47, lfo_depth: 0.0, lfo_dest: 0,
        lfo2_waveform: 0, lfo2_division: 0.47, lfo2_depth: 0.0, lfo2_dest: 2,
        volume,
        send_reverb: 0.2, send_delay: 0.0,
        mute: false,
    }
}

/// Override glide on a preset
#[allow(dead_code)]
const fn with_glide(mut p: SynthParams, glide: f32) -> SynthParams {
    p.glide = glide;
    p
}

/// Override osc sync on a preset
#[allow(dead_code)]
const fn with_sync(mut p: SynthParams) -> SynthParams {
    p.osc_sync = 1;
    p
}

/// Override sub waveform on a preset (0=Sqr, 1=Sin, 2=Saw)
#[allow(dead_code)]
const fn with_sub_waveform(mut p: SynthParams, wf: u8) -> SynthParams {
    p.sub_waveform = wf;
    p
}

/// Override filter key follow on a preset
#[allow(dead_code)]
const fn with_key_follow(mut p: SynthParams, amount: f32) -> SynthParams {
    p.filter_key_follow = amount;
    p
}

/// Override LFO1 settings on a preset
const fn with_lfo(mut p: SynthParams, wave: u8, div: f32, depth: f32, dest: u8) -> SynthParams {
    p.lfo_waveform = wave;
    p.lfo_division = div;
    p.lfo_depth = depth;
    p.lfo_dest = dest;
    p
}

/// Override LFO2 settings on a preset
const fn with_lfo2(mut p: SynthParams, wave: u8, div: f32, depth: f32, dest: u8) -> SynthParams {
    p.lfo2_waveform = wave;
    p.lfo2_division = div;
    p.lfo2_depth = depth;
    p.lfo2_dest = dest;
    p
}

/// Override both LFO1 and LFO2 settings
const fn with_lfos(
    p: SynthParams,
    w1: u8, d1: f32, dp1: f32, dst1: u8,
    w2: u8, d2: f32, dp2: f32, dst2: u8,
) -> SynthParams {
    with_lfo2(with_lfo(p, w1, d1, dp1, dst1), w2, d2, dp2, dst2)
}

/// Override send effects on a preset
const fn with_sends(mut p: SynthParams, reverb: f32, delay: f32) -> SynthParams {
    p.send_reverb = reverb;
    p.send_delay = delay;
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
    SynthSoundPreset {
        name: "Growl Bass", category: "Bass",
        // Saw + sub, LFO1 wobbles filter, LFO2 pumps osc2 level rhythmically
        params: with_lfos(sp(1, 0.5, 0.0, 0.9,
                   1, 0.5, 0.0, 0.5, 0.54,
                   0.6,
                   0.01, 0.2, 0.9, 0.1,
                   0.01, 0.2, 0.9, 0.1,
                   0, 0.35, 0.45, 0.15,
                   0.0, 0.25, 0.2, 0.1,
                   0.80),
                   1, 0.22, 0.40, 0,         // LFO1: Tri, 1/8, strong → Filter
                   4, 0.35, 0.30, 7),         // LFO2: Square, 1/4T, moderate → Osc2 Level
    },
    SynthSoundPreset {
        name: "Rubber Bass", category: "Bass",
        // Sine + detuned saw, LFO1 slow pitch drift, LFO2 on filter
        params: with_lfos(sp(2, 0.5, 0.0, 0.8,
                   1, 0.5, 0.0, 0.6, 0.53,
                   0.5,
                   0.01, 0.15, 0.85, 0.12,
                   0.01, 0.15, 0.85, 0.12,
                   0, 0.38, 0.25, 0.10,
                   0.0, 0.3, 0.2, 0.15,
                   0.78),
                   0, 0.67, 0.06, 2,         // LFO1: Sine, 1 bar, subtle → Osc1 Pitch
                   1, 0.47, 0.20, 0),         // LFO2: Tri, 1/4, moderate → Filter
    },
    SynthSoundPreset {
        name: "FM Bass", category: "Bass",
        // Square + sine octave up, snappy filter env
        params: sp(0, 0.5, 0.5, 0.7,
                   2, 0.75, 0.0, 0.5, 0.5,   // Sine +12
                   0.4,
                   0.00, 0.12, 0.6, 0.08,
                   0.00, 0.12, 0.6, 0.08,
                   0, 0.30, 0.35, 0.50,       // LP, low cutoff, reso, strong env
                   0.0, 0.10, 0.0, 0.06,
                   0.82),
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
    SynthSoundPreset {
        name: "Trance Lead", category: "Lead",
        // Bright saw, LFO1 on PWM, LFO2 slow filter sweep for movement
        params: with_lfos(sp(1, 0.5, 0.0, 0.9,
                   0, 0.5, 0.4, 0.5, 0.52,
                   0.0,
                   0.03, 0.2, 0.85, 0.35,
                   0.03, 0.2, 0.85, 0.35,
                   0, 0.60, 0.30, 0.20,
                   0.01, 0.3, 0.4, 0.3,
                   0.72),
                   1, 0.47, 0.25, 3,          // LFO1: Tri, 1/4, moderate → Osc1 PWM
                   0, 0.78, 0.15, 0),          // LFO2: Sine, 2 bars, gentle → Filter
    },
    SynthSoundPreset {
        name: "Portamento Lead", category: "Lead",
        // Single saw, no osc2, vibrato + filter motion
        params: with_lfos(sp(1, 0.5, 0.0, 1.0,
                   2, 0.5, 0.0, 0.0, 0.5,   // Osc2 off
                   0.2,
                   0.04, 0.2, 0.9, 0.4,
                   0.04, 0.2, 0.9, 0.4,
                   0, 0.55, 0.20, 0.18,
                   0.02, 0.3, 0.3, 0.3,
                   0.78),
                   0, 0.35, 0.10, 2,          // LFO1: Sine, 1/4T, vibrato → Pitch
                   1, 0.67, 0.12, 0),          // LFO2: Tri, 1 bar, subtle → Filter
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
    SynthSoundPreset {
        name: "Shimmer Pad", category: "Pad",
        // Sines detuned, LFO1 slow filter, LFO2 slow detune drift
        params: with_lfos(with_sends(sp(2, 0.5, 0.0, 0.6,
                   2, 0.5, 0.0, 0.5, 0.53,
                   0.2,
                   0.50, 0.2, 0.85, 0.60,
                   0.50, 0.2, 0.85, 0.60,
                   0, 0.55, 0.20, 0.10,
                   0.4, 0.6, 0.3, 0.5,
                   0.62), 0.35, 0.15),
                   0, 0.88, 0.12, 0,          // LFO1: Sine, 4 bars, subtle → Filter
                   1, 0.78, 0.08, 8),          // LFO2: Tri, 2 bars, gentle → Osc2 Detune
    },
    SynthSoundPreset {
        name: "Evolving Pad", category: "Pad",
        // Squares with PWM, LFO1 on PWM1, LFO2 on PWM2 at different rates
        params: with_lfos(with_sends(sp(0, 0.5, 0.4, 0.6,
                   0, 0.5, 0.6, 0.5, 0.52,
                   0.3,
                   0.45, 0.3, 0.75, 0.55,
                   0.45, 0.3, 0.75, 0.55,
                   0, 0.42, 0.15, 0.08,
                   0.3, 0.5, 0.3, 0.4,
                   0.65), 0.30, 0.10),
                   1, 0.67, 0.30, 3,          // LFO1: Tri, 1 bar, strong → Osc1 PWM
                   1, 0.78, 0.25, 6),          // LFO2: Tri, 2 bars, moderate → Osc2 PWM
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

    SynthSoundPreset {
        name: "Kalimba", category: "Pluck",
        // Sine + sine octave, very short decay, gentle filter env
        params: sp(2, 0.5, 0.0, 0.8,
                   2, 0.75, 0.0, 0.4, 0.5,   // Sine +12
                   0.0,
                   0.00, 0.18, 0.0, 0.08,
                   0.00, 0.10, 0.0, 0.06,
                   0, 0.65, 0.08, 0.35,
                   0.0, 0.12, 0.0, 0.08,
                   0.78),
    },
    SynthSoundPreset {
        name: "Harp", category: "Pluck",
        // Saw + sine, moderate decay, open filter
        params: with_sends(sp(1, 0.5, 0.0, 0.6,
                   2, 0.5, 0.0, 0.5, 0.51,
                   0.0,
                   0.00, 0.35, 0.0, 0.25,
                   0.00, 0.25, 0.0, 0.20,
                   0, 0.60, 0.12, 0.40,
                   0.0, 0.25, 0.0, 0.2,
                   0.72), 0.30, 0.15),
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

    SynthSoundPreset {
        name: "Detroit Stab", category: "Stab",
        // Single saw, fast filter env snap
        params: sp(1, 0.5, 0.0, 0.9,
                   1, 0.5, 0.0, 0.0, 0.5,
                   0.2,
                   0.00, 0.08, 0.0, 0.06,
                   0.00, 0.08, 0.0, 0.06,
                   0, 0.30, 0.20, 0.70,       // LP, low cut, strong env
                   0.0, 0.06, 0.0, 0.04,
                   0.82),
    },
    SynthSoundPreset {
        name: "Brass Stab", category: "Stab",
        // Two saws, medium attack for brass-like onset
        params: sp(1, 0.5, 0.0, 0.8,
                   1, 0.5, 0.0, 0.7, 0.53,
                   0.3,
                   0.06, 0.12, 0.5, 0.12,
                   0.06, 0.12, 0.5, 0.12,
                   0, 0.45, 0.15, 0.35,
                   0.04, 0.10, 0.2, 0.1,
                   0.78),
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

    SynthSoundPreset {
        name: "Wurlitzer", category: "Keys",
        // Sine with tremolo (LFO1→Volume) and gentle filter env
        params: with_lfo(sp(2, 0.5, 0.0, 0.8,
                   2, 0.75, 0.0, 0.2, 0.5,   // Sine +12, subtle
                   0.0,
                   0.01, 0.35, 0.55, 0.25,
                   0.01, 0.20, 0.3, 0.15,
                   0, 0.50, 0.12, 0.20,
                   0.0, 0.25, 0.15, 0.2,
                   0.72),
                   0, 0.47, 0.10, 10),        // LFO: Sine, 1/4, tremolo → Volume
    },
    SynthSoundPreset {
        name: "Toy Piano", category: "Keys",
        // Square, short decay, bright, no filter env
        params: sp(0, 0.5, 0.5, 0.8,
                   2, 0.75, 0.0, 0.3, 0.5,
                   0.0,
                   0.00, 0.20, 0.0, 0.10,
                   0.00, 0.12, 0.0, 0.08,
                   0, 0.70, 0.05, 0.20,
                   0.0, 0.10, 0.0, 0.08,
                   0.75),
    },

    // ── Drums (synth-based percussion) ─────────────────────────────────
    SynthSoundPreset {
        name: "Synth Kick", category: "Drums",
        // Sine with fast pitch drop (sweep), no sustain, punchy
        params: sp(2, 0.25, 0.0, 1.0,         // Sine, low tune
                   2, 0.5, 0.0, 0.0, 0.5,     // Osc2 off
                   0.7,                         // Heavy sub
                   0.00, 0.15, 0.0, 0.02,      // Very short envelope
                   0.00, 0.08, 0.0, 0.02,
                   0, 0.45, 0.0, 0.30,         // LP, moderate, some env
                   0.0, 0.06, 0.0, 0.02,
                   0.90),
    },
    SynthSoundPreset {
        name: "808 Kick", category: "Drums",
        // Long sine sub with pitch sweep
        params: sp(2, 0.20, 0.0, 1.0,
                   2, 0.5, 0.0, 0.0, 0.5,
                   0.8,
                   0.00, 0.40, 0.0, 0.05,     // Medium-long decay
                   0.00, 0.25, 0.0, 0.05,
                   0, 0.35, 0.0, 0.15,
                   0.0, 0.10, 0.0, 0.03,
                   0.88),
    },
    SynthSoundPreset {
        name: "Synth Snare", category: "Drums",
        // Noise + sine body, short decay
        params: sp(2, 0.45, 0.0, 0.6,         // Sine body
                   3, 0.5, 0.5, 0.7, 0.5,     // Noise layer
                   0.0,
                   0.00, 0.12, 0.0, 0.04,
                   0.00, 0.08, 0.0, 0.03,
                   0, 0.55, 0.15, 0.45,        // LP, moderate env
                   0.0, 0.08, 0.0, 0.03,
                   0.82),
    },
    SynthSoundPreset {
        name: "Noise Snare", category: "Drums",
        // Pure noise, tight filter env for snap
        params: sp(3, 0.5, 0.5, 0.9,
                   3, 0.5, 0.3, 0.3, 0.5,
                   0.0,
                   0.00, 0.10, 0.0, 0.03,
                   0.00, 0.06, 0.0, 0.02,
                   0, 0.40, 0.30, 0.60,
                   0.0, 0.06, 0.0, 0.02,
                   0.80),
    },
    SynthSoundPreset {
        name: "Synth Tom", category: "Drums",
        // Sine with pitch sweep, medium decay
        params: sp(2, 0.40, 0.0, 0.9,
                   2, 0.5, 0.0, 0.0, 0.5,
                   0.3,
                   0.00, 0.22, 0.0, 0.05,
                   0.00, 0.15, 0.0, 0.04,
                   0, 0.50, 0.10, 0.25,
                   0.0, 0.10, 0.0, 0.04,
                   0.80),
    },
    SynthSoundPreset {
        name: "Zap", category: "Drums",
        // Fast pitch sweep down, very short — like a 909 zap
        params: sp(2, 0.55, 0.0, 1.0,
                   2, 0.5, 0.0, 0.0, 0.5,
                   0.0,
                   0.00, 0.06, 0.0, 0.02,
                   0.00, 0.04, 0.0, 0.01,
                   0, 0.70, 0.0, 0.80,         // Strong filter env for sweep
                   0.0, 0.04, 0.0, 0.01,
                   0.78),
    },
    SynthSoundPreset {
        name: "Clap", category: "Drums",
        // Noise burst, medium decay
        params: sp(3, 0.5, 0.5, 0.8,
                   3, 0.5, 0.3, 0.4, 0.5,
                   0.0,
                   0.00, 0.15, 0.0, 0.05,
                   0.00, 0.10, 0.0, 0.04,
                   0, 0.50, 0.25, 0.40,
                   0.0, 0.08, 0.0, 0.04,
                   0.75),
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
    SynthSoundPreset {
        name: "Siren", category: "FX",
        // Sine, fast LFO1 on pitch, LFO2 on filter for texture
        params: with_lfos(sp(2, 0.5, 0.0, 0.9,
                   2, 0.5, 0.0, 0.0, 0.5,
                   0.0,
                   0.01, 0.1, 1.0, 0.3,
                   0.01, 0.1, 1.0, 0.3,
                   0, 0.60, 0.10, 0.0,
                   0.0, 0.1, 0.0, 0.1,
                   0.70),
                   0, 0.10, 0.35, 2,          // LFO1: Sine, 1/16, strong → Pitch
                   1, 0.47, 0.20, 0),          // LFO2: Tri, 1/4, moderate → Filter
    },
    SynthSoundPreset {
        name: "Riser", category: "FX",
        // Noise, long attack envelope, building tension
        params: sp(3, 0.5, 0.5, 0.7,
                   3, 0.5, 0.3, 0.4, 0.5,
                   0.0,
                   0.90, 0.1, 0.8, 0.30,     // Very long attack
                   0.90, 0.1, 0.8, 0.30,
                   0, 0.25, 0.35, 0.50,       // LP, low cut, opens with env
                   0.80, 0.2, 0.6, 0.3,       // Filter env also long attack
                   0.65),
    },
    SynthSoundPreset {
        name: "Robot", category: "FX",
        // Square, LFO1 fast on pitch (ring mod effect), LFO2 on volume (tremolo)
        params: with_lfos(sp(0, 0.5, 0.5, 0.8,
                   0, 0.75, 0.5, 0.4, 0.5,
                   0.0,
                   0.01, 0.1, 0.8, 0.1,
                   0.01, 0.1, 0.8, 0.1,
                   0, 0.50, 0.15, 0.0,
                   0.0, 0.1, 0.0, 0.05,
                   0.70),
                   0, 0.05, 0.20, 2,          // LFO1: Sine, 1/16D, fast pitch → ring mod
                   4, 0.22, 0.40, 10),         // LFO2: Square, 1/8, strong tremolo → Volume
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

pub fn preset_by_name(name: &str) -> Option<&'static SynthSoundPreset> {
    SYNTH_PRESETS.iter().find(|p| p.name == name)
}
