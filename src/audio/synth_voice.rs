//! Polyphonic synth voice: dual oscillators, sub, noise, 2 ADSR envelopes,
//! SVF filter, tempo-synced LFO. Hand-rolled DSP, no external crate dependencies.

use crate::sequencer::synth_pattern::SynthParams;

// ---------------------------------------------------------------------------
// Waveform enum
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Waveform {
    Square,
    Saw,
    Sine,
    Noise,
}

impl Waveform {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Waveform::Square,
            1 => Waveform::Saw,
            2 => Waveform::Sine,
            3 => Waveform::Noise,
            _ => Waveform::Square,
        }
    }
}

// ---------------------------------------------------------------------------
// Tiny xorshift32 noise generator
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct Noise {
    state: u32,
}

impl Noise {
    fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Returns a sample in -1.0 ..= 1.0
    fn next(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

// ---------------------------------------------------------------------------
// 1-pole lowpass filter (for noise waveform filtering)
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct OnePoleLP {
    prev_out: f32,
    coeff: f32,
}

impl OnePoleLP {
    fn new() -> Self {
        Self {
            prev_out: 0.0,
            coeff: 0.5,
        }
    }

    fn set_freq(&mut self, freq: f32, sr: f32) {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * freq);
        let dt = 1.0 / sr;
        self.coeff = dt / (rc + dt);
    }

    fn tick(&mut self, x: f32) -> f32 {
        let y = self.prev_out + self.coeff * (x - self.prev_out);
        self.prev_out = y;
        y
    }
}

// ---------------------------------------------------------------------------
// PolyBLEP anti-aliasing correction for discontinuities
// ---------------------------------------------------------------------------

/// PolyBLEP correction: smooths discontinuities in saw/square to reduce aliasing.
/// t = current phase (0..1), dt = phase increment per sample.
#[inline]
fn poly_blep(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Oscillator — phase accumulator based
// ---------------------------------------------------------------------------
#[derive(Clone)]
pub struct Oscillator {
    phase: f32,
    phase2: f32, // second phase for supersaw detune
    sample_rate: f32,
    noise_lp: OnePoleLP,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            phase2: 0.0,
            sample_rate,
            noise_lp: OnePoleLP::new(),
        }
    }

    pub fn tick(
        &mut self,
        freq_hz: f32,
        waveform: Waveform,
        param: f32,
        noise: &mut Noise,
    ) -> f32 {
        let inc = freq_hz / self.sample_rate;

        match waveform {
            Waveform::Square => {
                // param = pulse width (0.0-1.0 mapped to 0.05..0.95)
                let pw = 0.05 + param * 0.9;
                let mut out = if self.phase < pw { 1.0 } else { -1.0 };
                // PolyBLEP correction at rising edge (phase=0) and falling edge (phase=pw)
                out += poly_blep(self.phase as f64, inc as f64) as f32;
                let mut t = self.phase - pw;
                if t < 0.0 { t += 1.0; }
                out -= poly_blep(t as f64, inc as f64) as f32;
                self.phase += inc;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                out
            }
            Waveform::Saw => {
                // Primary saw with PolyBLEP anti-aliasing
                let mut saw1 = 2.0 * self.phase - 1.0;
                saw1 -= poly_blep(self.phase as f64, inc as f64) as f32;
                self.phase += inc;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }

                if param < 0.001 {
                    saw1
                } else {
                    // Supersaw: blend with a detuned second saw (also PolyBLEP'd)
                    let detune = param * 0.02; // up to ~1 semitone
                    let inc2 = freq_hz * (1.0 + detune) / self.sample_rate;
                    let mut saw2 = 2.0 * self.phase2 - 1.0;
                    saw2 -= poly_blep(self.phase2 as f64, inc2 as f64) as f32;
                    self.phase2 += inc2;
                    if self.phase2 >= 1.0 {
                        self.phase2 -= 1.0;
                    }
                    (saw1 + saw2) * 0.5
                }
            }
            Waveform::Sine => {
                // param = fold amount
                let x = self.phase * 2.0 * std::f32::consts::PI;
                let fold = 1.0 + param * 4.0;
                let out = (x * fold).sin();
                self.phase += inc;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                out
            }
            Waveform::Noise => {
                // param controls LP cutoff on noise (200Hz..20kHz)
                let cutoff = 200.0 * (100.0_f32).powf(param); // 200 * 100^param => 200..20000
                self.noise_lp.set_freq(cutoff, self.sample_rate);
                let raw = noise.next();
                // Advance phase so frequency tracking still works (unused for noise)
                self.phase += inc;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.noise_lp.tick(raw)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ADSR Envelope
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
enum AdsrState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone)]
pub struct AdsrEnvelope {
    state: AdsrState,
    level: f32,
    attack: f32,  // seconds
    decay: f32,   // seconds
    sustain: f32, // level 0-1
    release: f32, // seconds
    sample_rate: f32,
}

impl AdsrEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            state: AdsrState::Idle,
            level: 0.0,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.1,
            sample_rate,
        }
    }

    /// Set ADSR parameters from normalized 0.0-1.0 values.
    pub fn set_params(&mut self, a: f32, d: f32, s: f32, r: f32) {
        // Exponential mapping for time params
        self.attack = 0.001 * (2000.0_f32).powf(a);  // 0.001s .. 2.0s
        self.decay = 0.01 * (500.0_f32).powf(d);     // 0.01s .. 5.0s
        self.sustain = s;                              // 0.0 .. 1.0 linear
        self.release = 0.01 * (500.0_f32).powf(r);   // 0.01s .. 5.0s
    }

    pub fn trigger(&mut self) {
        self.state = AdsrState::Attack;
        // Start from current level (for re-triggers)
    }

    pub fn release(&mut self) {
        if self.state != AdsrState::Idle {
            self.state = AdsrState::Release;
        }
    }

    pub fn is_idle(&self) -> bool {
        self.state == AdsrState::Idle
    }

    pub fn tick(&mut self) -> f32 {
        match self.state {
            AdsrState::Idle => {
                self.level = 0.0;
            }
            AdsrState::Attack => {
                // Linear ramp from current level to 1.0
                let rate = 1.0 / (self.attack * self.sample_rate).max(1.0);
                self.level += rate;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = AdsrState::Decay;
                }
            }
            AdsrState::Decay => {
                // Exponential fall from 1.0 to sustain
                let rate = 1.0 / (self.decay * self.sample_rate).max(1.0);
                self.level += (self.sustain - self.level) * rate * 6.0;
                // Check if close enough to sustain
                if (self.level - self.sustain).abs() < 0.001 {
                    self.level = self.sustain;
                    self.state = AdsrState::Sustain;
                }
            }
            AdsrState::Sustain => {
                self.level = self.sustain;
            }
            AdsrState::Release => {
                // Exponential fall from current level to 0.0
                let rate = 1.0 / (self.release * self.sample_rate).max(1.0);
                self.level -= self.level * rate * 6.0;
                if self.level < 0.0001 {
                    self.level = 0.0;
                    self.state = AdsrState::Idle;
                }
            }
        }
        self.level
    }
}

// ---------------------------------------------------------------------------
// 24dB (4-pole) lowpass filter — cascaded Cytomic SVF pair
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct Svf {
    ic1eq: f32,
    ic2eq: f32,
}

impl Svf {
    fn new() -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    /// Process one sample, returning (LP, HP, BP) outputs.
    /// g = tan(pi * cutoff / sr), k = 2.0 - 2.0 * resonance
    fn tick(&mut self, input: f32, g: f32, k: f32) -> (f32, f32, f32) {
        let v3 = input - self.ic2eq;
        let v1 = self.ic1eq * (1.0 / (1.0 + g * (g + k)))
            + v3 * (g / (1.0 + g * (g + k)));
        let v2 = self.ic2eq + g * v1;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        let lp = v2;
        let bp = v1;
        let hp = input - k * v1 - v2;
        (lp, hp, bp)
    }
}

#[derive(Clone)]
pub struct Filter24dB {
    svf1: Svf,
    svf2: Svf,
    sample_rate: f32,
}

impl Filter24dB {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            svf1: Svf::new(),
            svf2: Svf::new(),
            sample_rate,
        }
    }

    /// filter_type: 0=LP, 1=HP, 2=BP
    pub fn tick(&mut self, input: f32, cutoff_hz: f32, resonance: f32, filter_type: u8) -> f32 {
        let cutoff = cutoff_hz.clamp(5.0, 20000.0);
        let reso = resonance.clamp(0.0, 0.99);

        let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
        let k = 2.0 - 2.0 * reso;

        let (lp1, hp1, bp1) = self.svf1.tick(input, g, k);
        let stage1 = match filter_type {
            1 => hp1,
            2 => bp1,
            _ => lp1,
        };
        let (lp2, hp2, bp2) = self.svf2.tick(stage1, g, k);
        match filter_type {
            1 => hp2,
            2 => bp2,
            _ => lp2,
        }
    }
}

/// Paraphonic synthesizer voice with dual oscillators, sub oscillator,
/// two ADSR envelopes (amplitude + filter), and a state-variable filter.
pub struct SynthVoice {
    sample_rate: f32,
    osc1: Oscillator,
    osc2: Oscillator,
    sub_osc: Oscillator, // always square, 1 oct below osc2
    noise1: Noise,
    noise2: Noise,
    noise_sub: Noise,
    env1: AdsrEnvelope,       // osc1 amplitude
    env2: AdsrEnvelope,       // osc2 + sub amplitude
    filter_env: AdsrEnvelope, // filter modulation
    filter: Filter24dB,
    note_freq: f32, // base frequency from MIDI note
}

// SynthVoice is Send because all fields are plain data (no Rc, no raw pointers).
unsafe impl Send for SynthVoice {}

impl SynthVoice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            osc1: Oscillator::new(sample_rate),
            osc2: Oscillator::new(sample_rate),
            sub_osc: Oscillator::new(sample_rate),
            noise1: Noise::new(0xDEAD_BEEF),
            noise2: Noise::new(0xCAFE_BABE),
            noise_sub: Noise::new(0xBAAD_F00D),
            env1: AdsrEnvelope::new(sample_rate),
            env2: AdsrEnvelope::new(sample_rate),
            filter_env: AdsrEnvelope::new(sample_rate),
            filter: Filter24dB::new(sample_rate),
            note_freq: 440.0,
        }
    }

    /// Trigger the voice with given params and MIDI note number.
    pub fn trigger(&mut self, params: &SynthParams, note: u8) {
        // Convert MIDI note to frequency
        self.note_freq = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);

        // Set envelope parameters
        self.env1.set_params(
            params.env1_attack,
            params.env1_decay,
            params.env1_sustain,
            params.env1_release,
        );
        self.env2.set_params(
            params.env2_attack,
            params.env2_decay,
            params.env2_sustain,
            params.env2_release,
        );
        self.filter_env.set_params(
            params.filter_env_attack,
            params.filter_env_decay,
            params.filter_env_sustain,
            params.filter_env_release,
        );

        // Trigger all envelopes
        self.env1.trigger();
        self.env2.trigger();
        self.filter_env.trigger();
    }

    /// Release all envelopes (note-off).
    pub fn release(&mut self) {
        self.env1.release();
        self.env2.release();
        self.filter_env.release();
    }

    /// Returns true when all envelopes have finished.
    pub fn is_idle(&self) -> bool {
        self.env1.is_idle() && self.env2.is_idle() && self.filter_env.is_idle()
    }

    /// Generate one sample of audio output.
    pub fn tick(&mut self, params: &SynthParams) -> f32 {
        // --- Oscillator frequencies ---
        // osc1_tune: 0-1 mapped to -24..+24 semitones
        let osc1_tune_semitones = params.osc1_tune * 48.0 - 24.0;
        let freq1 = self.note_freq * 2.0_f32.powf(osc1_tune_semitones / 12.0);

        // osc2_tune: 0-1 mapped to -24..+24 semitones
        let osc2_tune_semitones = params.osc2_tune * 48.0 - 24.0;
        // osc2_detune: 0-1 mapped to -50..+50 cents
        let detune_cents = params.osc2_detune * 100.0 - 50.0;
        let freq2 = self.note_freq
            * 2.0_f32.powf(osc2_tune_semitones / 12.0)
            * 2.0_f32.powf(detune_cents / 1200.0);

        // Sub oscillator: 1 octave below osc2
        let sub_freq = freq2 * 0.5;

        // --- Waveform selection ---
        let wf1 = Waveform::from_u8(params.osc1_waveform);
        let wf2 = Waveform::from_u8(params.osc2_waveform);

        // --- Envelope ticks ---
        let env1_val = self.env1.tick();
        let env2_val = self.env2.tick();
        let filter_env_val = self.filter_env.tick();

        // --- Oscillator generation ---
        let osc1_out = self.osc1.tick(freq1, wf1, params.osc1_pwm, &mut self.noise1)
            * env1_val
            * params.osc1_level;

        let osc2_out = self.osc2.tick(freq2, wf2, params.osc2_pwm, &mut self.noise2)
            * env2_val
            * params.osc2_level;

        // Sub oscillator: always square, pw=0.5, sub of osc2 (scales with osc2_level)
        let sub_out = self.sub_osc.tick(sub_freq, Waveform::Square, 0.5, &mut self.noise_sub)
            * env2_val
            * params.osc2_level
            * params.sub_level;

        // --- Mix ---
        let mix = osc1_out + osc2_out + sub_out;

        // --- Filter ---
        // filter_cutoff 0-1 mapped to 5..20000 Hz (exponential)
        let base_cutoff = 5.0 * (4000.0_f32).powf(params.filter_cutoff);
        let cutoff_mod = (base_cutoff + filter_env_val * params.filter_env_amount * 10000.0).max(5.0);
        let output = self.filter.tick(mix, cutoff_mod, params.filter_resonance, params.filter_type);

        output * params.volume
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poly_blep_reduces_aliasing() {
        // A naive saw at high frequency has more energy in upper harmonics (aliasing)
        // PolyBLEP should reduce this
        let phase = 0.001; // just after transition
        let dt = 0.01; // ~4800 Hz at 48kHz
        let correction = poly_blep(phase, dt);
        assert!(correction.abs() > 0.0, "PolyBLEP should correct near transitions");

        // Far from transition, no correction
        let correction_far = poly_blep(0.5, dt);
        assert!(correction_far.abs() < 1e-10, "PolyBLEP should be zero far from transitions");
    }
}
