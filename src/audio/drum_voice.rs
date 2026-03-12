//! Drum voice DSP implementations (8 tracks).
//! All synthesis is hand-rolled: oscillators, filters, envelopes, noise.
//! No external DSP crate dependencies.

use crate::sequencer::drum_pattern::DrumTrackParams;

/// Trait implemented by every drum voice.
pub trait DrumVoiceDsp: Send {
    fn trigger(&mut self, params: &DrumTrackParams);
    fn choke(&mut self) {}
    fn tick(&mut self) -> f32;
}

/// Soft waveshaping/saturation driven by the drive parameter.
/// drive=0 is clean, drive=1 is heavy saturation.
fn apply_drive(x: f32, drive: f32) -> f32 {
    if drive < 0.001 {
        return x;
    }
    let gain = 1.0 + drive * 8.0;
    (x * gain).tanh() / gain.tanh()
}

// ---------------------------------------------------------------------------
// Tiny xorshift32 noise generator (one per voice that needs it)
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
        // map u32 to -1..1
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

// ---------------------------------------------------------------------------
// Simple 1-pole filters
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct OnePoleHP {
    prev_in: f32,
    prev_out: f32,
    coeff: f32, // 0..1, higher = more highs pass
}

impl OnePoleHP {
    fn new() -> Self {
        Self {
            prev_in: 0.0,
            prev_out: 0.0,
            coeff: 0.5,
        }
    }

    fn set_freq(&mut self, freq: f32, sr: f64) {
        let rc = 1.0 / (2.0 * std::f64::consts::PI * freq as f64);
        let dt = 1.0 / sr;
        self.coeff = (rc / (rc + dt)) as f32;
    }

    fn tick(&mut self, x: f32) -> f32 {
        let y = self.coeff * (self.prev_out + x - self.prev_in);
        self.prev_in = x;
        self.prev_out = y;
        y
    }
}

#[derive(Clone)]
struct OnePoleLP {
    prev_out: f32,
    coeff: f32, // alpha
}

impl OnePoleLP {
    fn new() -> Self {
        Self {
            prev_out: 0.0,
            coeff: 0.5,
        }
    }

    fn set_freq(&mut self, freq: f32, sr: f64) {
        let rc = 1.0 / (2.0 * std::f64::consts::PI * freq as f64);
        let dt = 1.0 / sr;
        self.coeff = (dt / (rc + dt)) as f32;
    }

    fn tick(&mut self, x: f32) -> f32 {
        let y = self.prev_out + self.coeff * (x - self.prev_out);
        self.prev_out = y;
        y
    }
}

// ---------------------------------------------------------------------------
// State-variable filter (LP / HP / BP outputs)
// Ported from zicbox EffectFilterData
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct StateVariableFilter {
    cutoff: f32,
    feedback: f32,
    buf: f32,
    lp_out: f32,
    hp_out: f32,
    bp_out: f32,
}

impl StateVariableFilter {
    fn new() -> Self {
        Self {
            cutoff: 0.5,
            feedback: 0.0,
            buf: 0.0,
            lp_out: 0.0,
            hp_out: 0.0,
            bp_out: 0.0,
        }
    }

    /// Compute cutoff and feedback coefficients from frequency, resonance, and sample rate.
    /// `freq` in Hz, `resonance` in 0.0..1.0, `sr` in Hz.
    fn set_freq(&mut self, freq: f32, resonance: f32, sr: f64) {
        // Map frequency to 0..1 normalised cutoff
        let norm = (freq as f64 / sr).min(0.45) as f32; // Nyquist guard
        // The SVF cutoff coefficient (clamped for stability)
        self.cutoff = (2.0 * norm).clamp(0.0, 0.9);
        // Feedback from resonance (ported from zicbox getFeedback)
        if resonance <= 0.0 || self.cutoff >= 1.0 {
            self.feedback = 0.0;
        } else {
            let ratio = 1.0 - self.cutoff;
            let reso = resonance * 0.99;
            self.feedback = reso + reso / ratio;
        }
    }

    /// Process one sample, updating all three outputs.
    fn tick(&mut self, input: f32) {
        self.hp_out = input - self.buf;
        self.bp_out = self.buf - self.lp_out;
        self.buf += self.cutoff * (self.hp_out + self.feedback * self.bp_out);
        self.lp_out += self.cutoff * (self.buf - self.lp_out);
    }

    fn lp(&self) -> f32 {
        self.lp_out
    }

    #[allow(dead_code)]
    fn hp(&self) -> f32 {
        self.hp_out
    }

    #[allow(dead_code)]
    fn bp(&self) -> f32 {
        self.bp_out
    }

    fn reset(&mut self) {
        self.buf = 0.0;
        self.lp_out = 0.0;
        self.hp_out = 0.0;
        self.bp_out = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Comb filter for shell resonance (used by SnareVoice)
// ---------------------------------------------------------------------------

/// Simple feedback comb filter for adding resonant character to noise.
/// Models the shell resonance of an acoustic snare drum.
struct CombFilter {
    buf: [f32; 512],
    pos: usize,
    delay: usize,
    feedback: f32,
}

impl CombFilter {
    fn new() -> Self {
        Self {
            buf: [0.0; 512],
            pos: 0,
            delay: 100,
            feedback: 0.0,
        }
    }

    fn set(&mut self, freq_hz: f32, sr: f64, fb: f32) {
        self.delay = ((sr as f32 / freq_hz) as usize).clamp(1, 511);
        self.feedback = fb.clamp(0.0, 0.8);
    }

    #[inline]
    fn tick(&mut self, input: f32) -> f32 {
        let read_pos = if self.pos >= self.delay {
            self.pos - self.delay
        } else {
            512 - (self.delay - self.pos)
        };
        let delayed = self.buf[read_pos];
        self.buf[self.pos] = input + delayed * self.feedback;
        self.pos = (self.pos + 1) % 512;
        input + delayed * self.feedback
    }
}

// ===================================================================
// 1. KickVoice  (TR-909 inspired: sine osc + pitch env + click impulse)
//
// Two signal paths summed:
//   Path A: Sine oscillator → pitch envelope → body amp envelope → LP filter
//   Path B: Short impulse (DC pulse) → resonant LP filter (~5kHz) → click amp envelope
//
// Parameters:
//   tune   — fundamental frequency (30-80 Hz)
//   sweep  — pitch envelope depth (how far above fundamental the pitch starts)
//   color  — pitch envelope decay time (fast thump vs slow "zoop")
//   snap   — click/impulse level (the "Attack" knob)
//   filter — body LP cutoff (500-8000 Hz)
//   drive  — saturation
//   decay  — body amplitude decay (80-600ms)
// ===================================================================
pub struct KickVoice {
    sr: f64,
    // Path A: sine oscillator with pitch envelope
    phase: f64,
    freq_base: f32,       // fundamental (tune)
    freq_start: f32,      // initial pitch (fundamental + sweep)
    pitch_env: f32,       // 1→0 pitch envelope
    pitch_decay: f32,     // per-sample pitch env decay coefficient
    body_env: f32,        // body amplitude envelope
    body_decay: f32,      // per-sample body env decay
    // Body LP filter
    body_lp: OnePoleLP,
    // Path B: click impulse → resonant LP filter
    click_env: f32,       // click amplitude envelope (fixed short decay)
    click_decay: f32,
    click_level: f32,     // overall click amplitude (snap)
    click_pulse_phase: f64, // low-freq square wave for impulse
    click_svf: StateVariableFilter, // resonant LP at ~5kHz
    //
    noise: Noise,
    drive: f32,
    active: bool,
    // Sub-oscillator: one octave below for low-end weight
    sub_phase: f64,
    sub_freq: f32,
    sub_env: f32,
    sub_decay: f32,
}

impl KickVoice {
    pub fn new(sr: f64) -> Self {
        Self {
            sr,
            phase: 0.0,
            freq_base: 50.0,
            freq_start: 200.0,
            pitch_env: 0.0,
            pitch_decay: 0.0,
            body_env: 0.0,
            body_decay: 0.0,
            body_lp: OnePoleLP::new(),
            click_env: 0.0,
            click_decay: 0.0,
            click_level: 0.5,
            click_pulse_phase: 0.0,
            click_svf: StateVariableFilter::new(),
            noise: Noise::new(42),
            drive: 0.0,
            active: false,
            sub_phase: 0.0,
            sub_freq: 0.0,
            sub_env: 0.0,
            sub_decay: 0.0,
        }
    }
}

impl DrumVoiceDsp for KickVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        self.phase = 0.0;
        self.click_pulse_phase = 0.0;

        // ── Path A: sine body ────────────────────────────────────────

        // tune: fundamental freq 30-80 Hz
        self.freq_base = 30.0 + p.tune * 50.0;

        // sweep: pitch envelope depth (0-300 Hz above fundamental)
        self.freq_start = self.freq_base + p.sweep * 300.0;

        // Pitch envelope: always starts at 1.0, decays to 0 → pitch settles to fundamental
        self.pitch_env = 1.0;
        // color: pitch envelope decay time (5-80ms)
        // Low color = fast snap (punchy 909), high color = slow pitch glide
        let pitch_time = 0.005 + p.color as f64 * 0.075;
        self.pitch_decay = (-5.0_f64 / (pitch_time * self.sr)).exp() as f32;

        // decay: body amplitude envelope (80-600ms)
        let body_time = 0.08 + p.decay as f64 * 0.52;
        self.body_env = 1.0;
        self.body_decay = (-5.0_f64 / (body_time * self.sr)).exp() as f32;

        // filter: body LP (500-8000 Hz)
        let lp_freq = 500.0 + p.filter * 7500.0;
        self.body_lp.set_freq(lp_freq, self.sr);
        self.body_lp.prev_out = 0.0;

        // ── Path B: click impulse ────────────────────────────────────

        // snap: click level (the 909 "Attack" knob)
        self.click_level = p.snap;

        // Click envelope: fixed short decay (~4ms, like the 909)
        self.click_env = 1.0;
        self.click_decay = (-5.0_f64 / (0.004 * self.sr)).exp() as f32;

        // Resonant LP filter on click (~5kHz, resonance ~18%)
        let click_filter_freq = 4000.0 + p.snap * 2000.0;
        self.click_svf.set_freq(click_filter_freq, 0.18, self.sr);
        self.click_svf.reset();

        // Sub-oscillator: one octave below the body fundamental
        self.sub_freq = self.freq_base * 0.5;
        self.sub_phase = 0.0;
        self.sub_env = 0.35; // subtle reinforcement, not overwhelming
        // Decay tracks body but slightly shorter — support without boom
        self.sub_decay = (-5.0_f64 / ((0.10 + p.decay as f64 * 0.3) * self.sr)).exp() as f32;

        self.drive = p.drive;
        self.active = true;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // ── Path A: pitched sine body ──

        // Pitch envelope: cubed for fast initial drop then slow settle
        let pe = self.pitch_env;
        let pitch_shaped = pe * pe * pe;
        let freq = self.freq_base + (self.freq_start - self.freq_base) * pitch_shaped;
        self.pitch_env *= self.pitch_decay;

        // Sine oscillator
        self.phase += freq as f64 / self.sr;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let sine = (self.phase * std::f64::consts::TAU).sin() as f32;

        // Body LP filter + envelope
        let body = self.body_lp.tick(sine) * self.body_env;
        self.body_env *= self.body_decay;

        // ── Path B: click impulse ──

        // Impulse: low-frequency square wave (~30 Hz) — essentially a DC pulse
        // for the first half-cycle, producing a short positive impulse
        self.click_pulse_phase += 30.0_f64 / self.sr;
        let impulse = if self.click_pulse_phase < 0.5 { 1.0_f32 } else { 0.0 };

        // Add a tiny bit of noise for texture
        let click_raw = impulse + self.noise.next() * 0.15;

        // Resonant LP filter gives the click its "knock" character
        self.click_svf.tick(click_raw);
        let click = self.click_svf.lp() * self.click_env * self.click_level;
        self.click_env *= self.click_decay;

        // ── Path C: sub-oscillator (one octave below for chest-hitting low-end) ──

        self.sub_phase += self.sub_freq as f64 / self.sr;
        if self.sub_phase >= 1.0 {
            self.sub_phase -= 1.0;
        }
        let sub = (self.sub_phase * std::f64::consts::TAU).sin() as f32 * self.sub_env;
        self.sub_env *= self.sub_decay;

        // ── Sum all paths ──

        let raw = body + click + sub;
        let driven = apply_drive(raw, self.drive);

        // Deactivate when both envelopes are spent
        if self.body_env < 1e-6 && self.click_env < 1e-6 {
            self.active = false;
        }

        driven
    }
}

// ===================================================================
// 2. SnareVoice  (impact transient + tightness gate + separate decays)
// ===================================================================
pub struct SnareVoice {
    sr: f64,
    phase: f64,
    freq: f32,
    pitch_env: f32,
    pitch_decay: f32,
    body_env: f32,
    body_decay: f32,
    noise_env: f32,
    noise_decay: f32,
    tone_noise_mix: f32, // 0=all tone, 1=all noise
    impact_remaining: u32,
    impact_samples: u32,
    impact_amp: f32,
    tightness: f32,
    overall_env: f32,
    overall_decay: f32,
    hp: OnePoleHP,
    lp: OnePoleLP,
    noise: Noise,
    drive: f32,
    active: bool,
    comb: CombFilter,
}

impl SnareVoice {
    pub fn new(sr: f64) -> Self {
        Self {
            sr,
            phase: 0.0,
            freq: 180.0,
            pitch_env: 0.0,
            pitch_decay: 0.0,
            body_env: 0.0,
            body_decay: 0.0,
            noise_env: 0.0,
            noise_decay: 0.0,
            tone_noise_mix: 0.6,
            impact_remaining: 0,
            impact_samples: 0,
            impact_amp: 0.0,
            tightness: 0.0,
            overall_env: 0.0,
            overall_decay: 0.0,
            hp: OnePoleHP::new(),
            lp: OnePoleLP::new(),
            noise: Noise::new(123),
            drive: 0.0,
            active: false,
            comb: CombFilter::new(),
        }
    }
}

impl DrumVoiceDsp for SnareVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        self.phase = 0.0;

        // tune: body sine frequency (120-280 Hz)
        self.freq = 120.0 + p.tune * 160.0;

        // sweep: pitch envelope depth on body
        self.pitch_env = p.sweep;
        self.pitch_decay = (-5.0_f64 / (0.02 * self.sr)).exp() as f32;

        // Body envelope: short, color extends it slightly (0.02-0.08s)
        let body_time = 0.02 + p.color as f64 * 0.06;
        self.body_env = 1.0;
        self.body_decay = (-5.0_f64 / (body_time * self.sr)).exp() as f32;

        // Noise envelope: controlled by decay param (0.05-0.4s)
        let noise_time = 0.05 + p.decay as f64 * 0.35;
        self.noise_env = 1.0;
        self.noise_decay = (-5.0_f64 / (noise_time * self.sr)).exp() as f32;

        // color: tone/noise balance
        self.tone_noise_mix = p.color;

        // snap: impact transient (~3ms noise burst)
        self.impact_samples = (self.sr * 0.003) as u32;
        self.impact_remaining = self.impact_samples;
        self.impact_amp = 0.3 + p.snap * 0.7;

        // filter: tightness/gate
        self.tightness = p.filter;

        // Overall envelope for tightness gating
        let overall_time = 0.05 + p.decay as f64 * 0.45;
        self.overall_env = 1.0;
        self.overall_decay = (-5.0_f64 / (overall_time * self.sr)).exp() as f32;

        // Noise HP cutoff derived from tune (higher tuned = brighter wires)
        let hp_freq = 2000.0 + p.tune * 6000.0;
        self.hp.set_freq(hp_freq, self.sr);
        self.hp.prev_in = 0.0;
        self.hp.prev_out = 0.0;

        // LP on output
        let lp_freq = 6000.0 + p.tune * 12000.0;
        self.lp.set_freq(lp_freq, self.sr);
        self.lp.prev_out = 0.0;

        // Shell resonance: comb filter tuned to 2x snare pitch for metallic ring
        let comb_freq = (120.0 + p.tune * 160.0) * 2.0; // ~240-560 Hz
        let comb_fb = 0.3 + p.color * 0.3; // more color = more resonance
        self.comb.set(comb_freq, self.sr, comb_fb);

        self.drive = p.drive;
        self.active = true;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Sine body with pitch sweep
        let freq = self.freq * (1.0 + self.pitch_env * 0.5);
        self.pitch_env *= self.pitch_decay;

        self.phase += freq as f64 / self.sr;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let sine = (self.phase * 2.0 * std::f64::consts::PI).sin() as f32;
        let body = sine * self.body_env;
        self.body_env *= self.body_decay;

        // Noise through highpass → comb filter for shell resonance
        let raw_noise = self.noise.next();
        let filtered_noise = self.hp.tick(raw_noise);
        let resonated_noise = self.comb.tick(filtered_noise);
        let noise_out = resonated_noise * self.noise_env;
        self.noise_env *= self.noise_decay;

        // Impact transient: raw noise burst in first ~3ms
        let impact = if self.impact_remaining > 0 {
            self.impact_remaining -= 1;
            self.noise.next() * self.impact_amp
        } else {
            0.0
        };

        // Mix: color controls tone/noise balance
        let tone_gain = 1.0 - self.tone_noise_mix;
        let noise_gain = self.tone_noise_mix;
        let raw = body * tone_gain * 0.6 + noise_out * noise_gain + impact;
        let shaped = self.lp.tick(raw);
        let driven = apply_drive(shaped, self.drive);

        // Tightness gate: pow(env, 1 + tightness * 3)
        self.overall_env *= self.overall_decay;
        let tight_env = self.overall_env.powf(1.0 + self.tightness * 3.0);
        let out = driven * tight_env;

        if self.overall_env < 1e-6 {
            self.active = false;
        }

        out
    }
}

// ===================================================================
// 3. ClosedHiHatVoice  (metallic resonator engine)
// ===================================================================
pub struct ClosedHiHatVoice {
    sr: f64,
    sr_recip: f32,
    // 6-oscillator metallic bank (same approach as RideVoice)
    phases: [f32; 6],
    base_freq: f32,
    fm_intensity: f32,
    // Noise layer
    noise: Noise,
    noise_mix: f32,
    // Envelopes
    env: f32,
    env_decay: f32,
    snap_env: f32,
    snap_decay: f32,
    // HP filter to remove low-end rumble
    hp: OnePoleHP,
    // LP filter for brightness
    svf: StateVariableFilter,
    drive: f32,
    active: bool,
    // Bright click transient (~2ms noise burst at high frequency)
    transient_env: f32,
    transient_decay: f32,
    transient_noise: Noise,
    // Sizzle: high-shelf boost state
    sizzle_state: f32,
    sizzle_coeff: f32,
}

impl ClosedHiHatVoice {
    pub fn new(sr: f64) -> Self {
        let sizzle_freq = 10000.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * sizzle_freq);
        let dt = 1.0 / sr as f32;
        Self {
            sr,
            sr_recip: (1.0 / sr) as f32,
            phases: [0.0; 6],
            base_freq: 400.0,
            fm_intensity: 0.0,
            noise: Noise::new(456),
            noise_mix: 0.3,
            env: 0.0,
            env_decay: 0.0,
            snap_env: 0.0,
            snap_decay: 0.0,
            hp: OnePoleHP::new(),
            svf: StateVariableFilter::new(),
            drive: 0.0,
            active: false,
            transient_env: 0.0,
            transient_decay: 0.0,
            transient_noise: Noise::new(789),
            sizzle_state: 0.0,
            sizzle_coeff: dt / (rc + dt),
        }
    }
}

impl DrumVoiceDsp for ClosedHiHatVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        self.phases = [0.0; 6];

        // tune: base frequency for metallic bank (300-900 Hz, higher than ride)
        self.base_freq = 300.0 + p.tune * 600.0;

        // color: cross-FM intensity (0 = clean shimmer, 1 = gritty/harsh)
        self.fm_intensity = p.color * 1.5;

        // sweep: noise layer mix (0 = pure metallic, 1 = noisy/trashy)
        self.noise_mix = 0.3 + p.sweep * 0.7;

        // decay: 10-80ms (short, closed character)
        let time = 0.01 + p.decay * 0.07;
        self.env = 1.0;
        self.env_decay = (-5.0_f64 / (time as f64 * self.sr)).exp() as f32;

        // snap: transient click
        self.snap_env = p.snap * 0.8;
        self.snap_decay = (-5.0_f64 / (0.002 * self.sr)).exp() as f32;

        // HP filter: remove low-end (3-8 kHz range for hi-hat airiness)
        let hp_freq = 3000.0 + p.filter * 5000.0;
        self.hp.set_freq(hp_freq, self.sr);
        self.hp.prev_in = 0.0;
        self.hp.prev_out = 0.0;

        // LP filter: brightness control (6-18 kHz)
        let lp_freq = 6000.0 + p.filter * 12000.0;
        self.svf.set_freq(lp_freq, 0.05, self.sr);
        self.svf.reset();

        // Bright transient: very short noise burst for attack definition
        self.transient_env = 0.5 + p.snap * 0.5;
        let transient_ms = 2.0;
        self.transient_decay = (-5.0_f64 / (transient_ms as f64 * 0.001 * self.sr)).exp() as f32;

        self.drive = p.drive;
        self.active = true;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let sr_recip = self.sr_recip;

        // --- 6-oscillator metallic bank with cross-FM ---
        let mut mix = 0.0_f32;
        let mut last_sig = 0.0_f32;

        for i in 0..6 {
            let freq = self.base_freq * CYMBAL_RATIOS[i];

            // Cross-FM from previous oscillator
            let fm_offset = last_sig * self.fm_intensity;
            self.phases[i] += freq * sr_recip + fm_offset * 0.01;
            if self.phases[i] >= 1.0 {
                self.phases[i] -= 1.0;
            }

            // Square wave (TR-808 style)
            let sq = if self.phases[i] < 0.5 { 1.0_f32 } else { -1.0 };

            // Alternate add/multiply for dense inharmonic spectrum
            if i % 2 == 0 {
                mix += sq;
            } else {
                mix *= 0.5 + sq * 0.5;
            }

            last_sig = sq;
        }

        mix *= 0.5; // normalize

        // Noise layer (filtered white noise for "fizz")
        let noise_sample = self.noise.next() * self.noise_mix * 1.0;

        // Snap transient (noise burst)
        let snap = self.noise.next() * self.snap_env;
        self.snap_env *= self.snap_decay;

        // Mix metallic + noise
        let raw = mix + noise_sample + snap;

        // HP filter for airiness, then LP for brightness
        let hp_out = self.hp.tick(raw);
        self.svf.tick(hp_out);
        let filtered = self.svf.lp();
        let driven = apply_drive(filtered, self.drive);

        // Add bright transient click
        let transient = self.transient_noise.next() * self.transient_env;
        self.transient_env *= self.transient_decay;

        // Sizzle: high-shelf boost (add high-passed version of signal)
        let sizzle_in = driven + transient;
        self.sizzle_state += self.sizzle_coeff * (sizzle_in - self.sizzle_state);
        let hi_content = sizzle_in - self.sizzle_state; // HP = input - LP
        let out = (sizzle_in + hi_content * 0.4) * self.env; // boost highs by ~40%

        self.env *= self.env_decay;
        if self.env < 1e-6 {
            self.active = false;
        }

        out
    }
}

// ===================================================================
// 4. OpenHiHatVoice  (noise × metallic ring modulation, TR-909 style)
//
// Real hihats are primarily NOISE shaped by metallic resonances, not
// metallic oscillators with noise added. This voice uses:
//
//   1. Ring modulation: white noise × 6-oscillator metallic bank
//      This "stamps" the metallic frequency spectrum onto the noise,
//      producing a shhhhh with shimmer — the classic hihat character.
//   2. Additional pure noise layer for air/body.
//   3. Two-band amplitude envelope (HF decays faster → natural darkening).
//   4. HP + sweeping LP filter chain for spectral evolution.
//   5. Smooth choke (~10ms exponential fade) for CHH→OHH interaction.
//
// Parameters:
//   tune   — metallic resonance base frequency (200-800 Hz)
//   sweep  — ring mod depth: 0 = pure noise (trashy), 1 = heavy metallic color
//   color  — phase modulation between oscillators (clean shimmer → gritty)
//   snap   — transient click/stick hit
//   filter — HP/LP cutoff (controls brightness and body)
//   drive  — saturation
//   decay  — sustain time (100-800ms)
// ===================================================================
pub struct OpenHiHatVoice {
    sr: f64,
    sr_recip: f32,
    // 6-oscillator metallic bank (used as ring modulator carrier)
    phases: [f32; 6],
    base_freq: f32,
    pm_depth: f32,        // phase modulation depth between oscillators
    ring_mod_depth: f32,  // how much metallic coloring vs pure noise
    // Noise
    noise: Noise,
    noise_rng: Noise,
    // Two-band envelope
    env_body: f32,
    env_body_decay: f32,
    env_hf: f32,
    env_hf_decay: f32,
    // Attack ramp (~0.5ms)
    attack_env: f32,
    attack_inc: f32,
    // Stick transient
    snap_env: f32,
    snap_decay: f32,
    // Choke
    choke_env: f32,
    choke_decay: f32,
    choking: bool,
    // Filter chain: HP → sweeping LP
    hp: OnePoleHP,
    svf: StateVariableFilter,
    lp_freq: f32,
    lp_target: f32,
    lp_sweep_coeff: f32,
    drive: f32,
    active: bool,
    // Bright click transient (~3ms noise burst)
    transient_env: f32,
    transient_decay: f32,
    transient_noise: Noise,
    // Sizzle: high-shelf boost state
    sizzle_state: f32,
    sizzle_coeff: f32,
}

impl OpenHiHatVoice {
    pub fn new(sr: f64) -> Self {
        let sizzle_freq = 10000.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * sizzle_freq);
        let dt = 1.0 / sr as f32;
        Self {
            sr,
            sr_recip: (1.0 / sr) as f32,
            phases: [0.0; 6],
            base_freq: 400.0,
            pm_depth: 0.0,
            ring_mod_depth: 0.5,
            noise: Noise::new(789),
            noise_rng: Noise::new(0xBEEF),
            env_body: 0.0,
            env_body_decay: 0.0,
            env_hf: 0.0,
            env_hf_decay: 0.0,
            attack_env: 0.0,
            attack_inc: 0.0,
            snap_env: 0.0,
            snap_decay: 0.0,
            choke_env: 1.0,
            choke_decay: 1.0,
            choking: false,
            hp: OnePoleHP::new(),
            svf: StateVariableFilter::new(),
            lp_freq: 16000.0,
            lp_target: 8000.0,
            lp_sweep_coeff: 1.0,
            drive: 0.0,
            active: false,
            transient_env: 0.0,
            transient_decay: 0.0,
            transient_noise: Noise::new(0xCAFE),
            sizzle_state: 0.0,
            sizzle_coeff: dt / (rc + dt),
        }
    }
}

impl DrumVoiceDsp for OpenHiHatVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        // Randomize initial phases — each hit has unique character
        for phase in &mut self.phases {
            *phase = (self.noise_rng.next() + 1.0) * 0.5;
        }

        // tune: metallic resonance base frequency (200-800 Hz)
        self.base_freq = 200.0 + p.tune * 600.0;

        // color: phase modulation depth (clean shimmer → gritty/harsh)
        self.pm_depth = p.color * 1.5;

        // sweep: ring modulation depth (pure noise ↔ metallic coloring)
        // 0.0 = mostly noise (trashy/white), 1.0 = strong metallic ring
        self.ring_mod_depth = p.sweep;

        // decay: two-band envelope (100-800ms body, HF decays 3x faster)
        let body_time = 0.1 + p.decay * 0.7;
        self.env_body = 1.0;
        self.env_body_decay = (-5.0_f64 / (body_time as f64 * self.sr)).exp() as f32;
        let hf_time = body_time / 3.0;
        self.env_hf = 1.0;
        self.env_hf_decay = (-5.0_f64 / (hf_time as f64 * self.sr)).exp() as f32;

        // Attack ramp (~0.5ms)
        self.attack_env = 0.0;
        self.attack_inc = 1.0 / (0.0005 * self.sr as f32);

        // snap: stick transient (~3ms noise burst)
        self.snap_env = p.snap * 1.0;
        self.snap_decay = (-5.0_f64 / (0.003 * self.sr)).exp() as f32;

        // Reset choke
        self.choke_env = 1.0;
        self.choking = false;

        // HP filter (4-10 kHz range — higher than before for more "air")
        let hp_freq = 4000.0 + p.filter * 6000.0;
        self.hp.set_freq(hp_freq, self.sr);
        self.hp.prev_in = 0.0;
        self.hp.prev_out = 0.0;

        // Sweeping LP: starts bright, darkens over time (like a real cymbal)
        let lp_start = 10000.0 + p.filter * 8000.0;
        let lp_end = 5000.0 + p.filter * 5000.0;
        self.lp_freq = lp_start;
        self.lp_target = lp_end;
        self.lp_sweep_coeff = (-5.0_f64 / (body_time as f64 * 0.6 * self.sr)).exp() as f32;
        self.svf.set_freq(lp_start, 0.1, self.sr);
        self.svf.reset();

        // Bright transient: slightly longer than closed hat (3ms) for diffuse attack
        self.transient_env = 0.4 + p.snap * 0.6;
        let transient_ms = 3.0;
        self.transient_decay = (-5.0_f64 / (transient_ms as f64 * 0.001 * self.sr)).exp() as f32;

        self.drive = p.drive;
        self.active = true;
    }

    fn choke(&mut self) {
        self.choking = true;
        self.choke_decay = (-5.0_f64 / (0.01 * self.sr)).exp() as f32;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let sr_recip = self.sr_recip;

        // Attack ramp
        if self.attack_env < 1.0 {
            self.attack_env = (self.attack_env + self.attack_inc).min(1.0);
        }

        // ── Metallic oscillator bank (used as ring modulator carrier) ──
        let mut metallic = 0.0_f32;
        let mut prev_osc = 0.0_f32;

        for i in 0..6 {
            let freq = self.base_freq * CYMBAL_RATIOS[i];
            let pm = prev_osc * self.pm_depth * 0.15;
            self.phases[i] += freq * sr_recip;
            if self.phases[i] >= 1.0 {
                self.phases[i] -= 1.0;
            }
            let osc = ((self.phases[i] + pm) * std::f32::consts::TAU).sin();
            metallic += osc;
            prev_osc = osc;
        }
        metallic /= 6.0; // normalize to roughly -1..1

        // ── Noise source ──
        let noise = self.noise.next();

        // ── Ring modulation: noise × metallic ──
        // This stamps the metallic frequency spectrum onto the noise,
        // producing "shhhhh" with metallic shimmer rather than pure tones.
        let ring = noise * metallic;

        // Blend: ring-modulated signal vs pure noise
        // ring_mod_depth=0 → pure noise (trashy), =1 → full ring mod (metallic shhh)
        let blend = (ring * self.ring_mod_depth + noise * (1.0 - self.ring_mod_depth) * 0.6) * 2.5;

        // ── Stick transient ──
        let snap = self.noise.next() * self.snap_env;
        self.snap_env *= self.snap_decay;

        // ── Two-band envelope ──
        // Body envelope on the blended signal, HF envelope adds extra brightness at start
        let extra_hf = self.noise.next() * 0.3 * self.env_hf;
        let raw = blend * self.env_body + extra_hf + snap;

        // ── Filter chain: HP → sweeping LP ──
        let hp_out = self.hp.tick(raw);

        self.lp_freq = self.lp_target + (self.lp_freq - self.lp_target) * self.lp_sweep_coeff;
        self.svf.set_freq(self.lp_freq, 0.1, self.sr);
        self.svf.tick(hp_out);
        let filtered = self.svf.lp();

        let driven = apply_drive(filtered, self.drive);

        // Add bright transient click
        let transient = self.transient_noise.next() * self.transient_env;
        self.transient_env *= self.transient_decay;

        // Sizzle: high-shelf boost
        let sizzle_in = driven + transient;
        self.sizzle_state += self.sizzle_coeff * (sizzle_in - self.sizzle_state);
        let hi_content = sizzle_in - self.sizzle_state;
        let out = (sizzle_in + hi_content * 0.4) * self.attack_env;

        // Advance envelopes
        self.env_body *= self.env_body_decay;
        self.env_hf *= self.env_hf_decay;

        // Choke
        if self.choking {
            self.choke_env *= self.choke_decay;
            if self.choke_env < 1e-5 {
                self.active = false;
                return 0.0;
            }
            return out * self.choke_env;
        }

        if self.env_body < 1e-6 && self.env_hf < 1e-6 {
            self.active = false;
        }

        out
    }
}

// ===================================================================
// 5. RideVoice  (6-oscillator metallic bank + ping transient)
//    Inspired by TR-808 / Mutable Instruments Plaits / zicbox FreakHat
// ===================================================================

/// Inharmonic frequency ratios for metallic cymbal spectrum.
/// Based on Mutable Instruments Plaits hi-hat engine.
const CYMBAL_RATIOS: [f32; 6] = [1.0, 1.304, 1.466, 1.787, 1.932, 2.536];

pub struct RideVoice {
    sr: f64,
    sr_recip: f32, // 1.0 / sr, cached
    // 6-oscillator bank
    phases: [f32; 6],
    base_freq: f32,
    inharmonicity: f32, // extra spread between oscillators
    fm_intensity: f32,  // cross-modulation amount
    // Ping transient (bandpass-filtered sine burst for bell attack)
    ping_phase: f64,
    ping_freq: f32,
    ping_env: f32,
    ping_decay: f32,
    // Envelopes: two-band (HF decays faster than body)
    env_body: f32,
    env_body_decay: f32,
    env_hf: f32,
    env_hf_decay: f32,
    // HP filter for airiness
    hp: OnePoleHP,
    // LP filter for brightness
    svf: StateVariableFilter,
    drive: f32,
    active: bool,
}

impl RideVoice {
    pub fn new(sr: f64) -> Self {
        Self {
            sr,
            sr_recip: (1.0 / sr) as f32,
            phases: [0.0; 6],
            base_freq: 205.0,
            inharmonicity: 0.0,
            fm_intensity: 0.0,
            ping_phase: 0.0,
            ping_freq: 1000.0,
            ping_env: 0.0,
            ping_decay: 0.0,
            env_body: 0.0,
            env_body_decay: 0.0,
            env_hf: 0.0,
            env_hf_decay: 0.0,
            hp: OnePoleHP::new(),
            svf: StateVariableFilter::new(),
            drive: 0.0,
            active: false,
        }
    }
}

impl DrumVoiceDsp for RideVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        // Reset oscillator phases
        self.phases = [0.0; 6];

        // tune: base frequency for the oscillator bank (150-600 Hz)
        self.base_freq = 150.0 + p.tune * 450.0;

        // sweep: inharmonicity spread (0 = tight cluster, 1 = wide spread)
        self.inharmonicity = p.sweep * 400.0;

        // color: FM cross-modulation intensity (0 = clean shimmer, 1 = screaming)
        self.fm_intensity = p.color * 2.0;

        // snap: ping transient strength
        // The "ping" is a bandpass-filtered sine burst at ~2x base freq
        self.ping_phase = 0.0;
        self.ping_freq = self.base_freq * 2.5;
        self.ping_env = 0.3 + p.snap * 0.7;
        self.ping_decay = (-5.0_f64 / (0.008 * self.sr)).exp() as f32; // ~8ms ping

        // decay: two-band envelope
        // Body (mids) decays at the user rate (200ms-2s)
        let body_time = 0.2 + p.decay * 1.8;
        self.env_body = 1.0;
        self.env_body_decay = (-5.0_f64 / (body_time as f64 * self.sr)).exp() as f32;
        // HF decays 3x faster (spectral evolution: highs die first like real cymbal)
        let hf_time = body_time / 3.0;
        self.env_hf = 1.0;
        self.env_hf_decay = (-5.0_f64 / (hf_time as f64 * self.sr)).exp() as f32;

        // filter: HP cutoff for airiness (500-4000 Hz)
        let hp_freq = 500.0 + p.filter * 3500.0;
        self.hp.set_freq(hp_freq, self.sr);
        self.hp.prev_in = 0.0;
        self.hp.prev_out = 0.0;

        // SVF LP for brightness (4-18 kHz)
        let lp_freq = 4000.0 + p.filter * 14000.0;
        self.svf.set_freq(lp_freq, 0.1, self.sr);
        self.svf.reset();

        self.drive = p.drive;
        self.active = true;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let sr_recip = self.sr_recip;

        // --- 6-oscillator metallic bank with cross-FM ---
        let mut mix = 0.0_f32;
        let mut last_sig = 0.0_f32;

        for i in 0..6 {
            let freq = self.base_freq * CYMBAL_RATIOS[i]
                + (i as f32) * self.inharmonicity;

            // Cross-FM: previous oscillator modulates this one's phase
            let fm_offset = last_sig * self.fm_intensity;
            self.phases[i] += freq * sr_recip + fm_offset * 0.01;
            if self.phases[i] >= 1.0 {
                self.phases[i] -= 1.0;
            }

            // Square wave: just sign of phase (TR-808 style, very cheap)
            let sig = if self.phases[i] > 0.5 { 1.0_f32 } else { -1.0_f32 };
            last_sig = sig;

            // Alternate add/multiply for ring-mod-like density (FreakHat approach)
            if i % 2 == 0 {
                mix += sig;
            } else {
                // Ring mod: multiply only if mix is non-zero, else just add
                if mix.abs() > 0.001 {
                    mix *= sig;
                } else {
                    mix += sig;
                }
            }
        }
        // Scale down the oscillator bank
        mix *= 0.15;

        // --- Ping transient (bell attack) ---
        let ping = if self.ping_env > 1e-5 {
            self.ping_phase += self.ping_freq as f64 / self.sr;
            if self.ping_phase >= 1.0 {
                self.ping_phase -= 1.0;
            }
            let p = (self.ping_phase * 2.0 * std::f64::consts::PI).sin() as f32;
            let out = p * self.ping_env;
            self.ping_env *= self.ping_decay;
            out
        } else {
            0.0
        };

        // --- Two-band envelope: body (mids) + HF ---
        // Split: oscillator bank carries both, apply weighted envelope
        // HF content is the raw mix, body is a LP-smoothed version
        let hf_part = mix * self.env_hf;
        let body_part = mix * self.env_body;
        // Blend: body carries the fundamental shimmer, hf adds brightness
        let metallic = body_part * 0.6 + hf_part * 0.4 + ping;

        self.env_body *= self.env_body_decay;
        self.env_hf *= self.env_hf_decay;

        // --- Filter chain: HP (airiness) then LP (brightness) ---
        let hp_out = self.hp.tick(metallic);
        self.svf.tick(hp_out);
        let filtered = self.svf.lp();

        let driven = apply_drive(filtered, self.drive);

        // Use body envelope for overall gate (it's the longest)
        if self.env_body < 1e-6 {
            self.active = false;
        }

        driven
    }
}

// ===================================================================
// 6. ClapVoice  (resonant SVF bandpass + noise color + punch)
// ===================================================================
pub struct ClapVoice {
    sr: f64,
    env: f32,
    env_decay: f32,
    burst_index: u32,
    burst_count: u32,
    burst_sample: u32,
    burst_on_samples: u32,
    burst_off_samples: u32,
    burst_on: bool,
    in_burst_phase: bool,
    // Inline SVF state for resonant bandpass
    svf_buf: f32,
    svf_lp: f32,
    svf_cutoff: f32,
    svf_feedback: f32,
    // Pink noise approximation
    pink_state: f32,
    pink_lp_coeff: f32,
    noise_color: f32,
    // Punch transient
    punch: f32,
    punch_samples: u32,
    punch_remaining: u32,
    noise: Noise,
    drive: f32,
    active: bool,
}

impl ClapVoice {
    pub fn new(sr: f64) -> Self {
        let s4ms = (sr * 0.004) as u32;
        // LP coeff for pink noise approximation (~1kHz)
        let rc = 1.0 / (2.0 * std::f64::consts::PI * 1000.0);
        let dt = 1.0 / sr;
        let pink_coeff = (dt / (rc + dt)) as f32;
        Self {
            sr,
            env: 0.0,
            env_decay: 0.0,
            burst_index: 0,
            burst_count: 4,
            burst_sample: 0,
            burst_on_samples: s4ms,
            burst_off_samples: s4ms,
            burst_on: true,
            in_burst_phase: false,
            svf_buf: 0.0,
            svf_lp: 0.0,
            svf_cutoff: 0.1,
            svf_feedback: 0.0,
            pink_state: 0.0,
            pink_lp_coeff: pink_coeff,
            noise_color: 0.0,
            punch: 0.0,
            punch_samples: 0,
            punch_remaining: 0,
            noise: Noise::new(654),
            drive: 0.0,
            active: false,
        }
    }
}

impl DrumVoiceDsp for ClapVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        // tune: BPF center frequency (800-4000 Hz)
        let center = 800.0 + p.tune * 3200.0;
        self.svf_cutoff =
            (2.0 * (std::f64::consts::PI * center as f64 / self.sr).sin()) as f32;
        // filter: resonance amount (0 = gentle, 1 = nasal/resonant)
        let reso = p.filter * 0.95;
        self.svf_feedback = reso + reso / (1.0 - self.svf_cutoff).max(0.01);
        self.svf_buf = 0.0;
        self.svf_lp = 0.0;

        // color: noise color (0 = white/bright, 1 = pink/warm)
        self.noise_color = p.color;
        self.pink_state = 0.0;

        // decay: tail length (0.05-0.3s)
        let tail_time = 0.05 + p.decay * 0.25;
        self.env = 1.0;
        self.env_decay = (-5.0_f64 / (tail_time as f64 * self.sr)).exp() as f32;

        // snap: burst count (3-6)
        self.burst_count = 3 + (p.snap * 3.0).round() as u32;
        // Fixed burst spacing ~4ms
        let spacing = 0.004;
        self.burst_on_samples = (self.sr * spacing) as u32;
        self.burst_off_samples = (self.sr * spacing) as u32;

        self.burst_index = 0;
        self.burst_sample = 0;
        self.burst_on = true;
        self.in_burst_phase = true;

        // sweep: punch / transient boost (0 = compress, 0.5 = neutral, 1 = boost)
        self.punch = p.sweep;
        self.punch_samples = (self.sr * 0.01) as u32; // 10ms window
        self.punch_remaining = self.punch_samples;

        self.drive = p.drive;
        self.active = true;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Noise generation with color blend
        let white = self.noise.next();
        self.pink_state += self.pink_lp_coeff * (white - self.pink_state);
        let mixed_noise =
            white * (1.0 - self.noise_color) + self.pink_state * self.noise_color;

        // Inline SVF bandpass
        let hp = mixed_noise - self.svf_buf;
        let bp = self.svf_buf - self.svf_lp;
        self.svf_buf += self.svf_cutoff * (hp + self.svf_feedback * bp);
        self.svf_lp += self.svf_cutoff * (self.svf_buf - self.svf_lp);
        let filtered = self.svf_buf;

        // Burst gate
        let burst_gate = if self.in_burst_phase {
            self.burst_sample += 1;
            if self.burst_on {
                if self.burst_sample >= self.burst_on_samples {
                    self.burst_sample = 0;
                    self.burst_on = false;
                }
                1.0
            } else {
                if self.burst_sample >= self.burst_off_samples {
                    self.burst_sample = 0;
                    self.burst_on = true;
                    self.burst_index += 1;
                    if self.burst_index >= self.burst_count {
                        self.in_burst_phase = false;
                    }
                }
                0.0
            }
        } else {
            1.0
        };

        // Punch: transient boost/compress in first ~10ms
        let punch_gain = if self.punch_remaining > 0 {
            self.punch_remaining -= 1;
            if self.punch > 0.5 {
                1.0 + (self.punch - 0.5) * 4.0
            } else {
                0.2 + self.punch * 1.6
            }
        } else {
            1.0
        };

        let driven = apply_drive(filtered * punch_gain, self.drive);
        let out = driven * self.env * burst_gate;

        if !self.in_burst_phase {
            self.env *= self.env_decay;
        }

        if !self.in_burst_phase && self.env < 1e-6 {
            self.active = false;
        }

        out
    }
}

// ===================================================================
// 7. CowbellVoice
// ===================================================================
pub struct CowbellVoice {
    sr: f64,
    phase1: f64,
    phase2: f64,
    freq1: f32,
    freq2: f32,
    pulse_width: f32,
    snap_env: f32,
    snap_decay: f32,
    env: f32,
    env_decay: f32,
    hp: OnePoleHP,
    lp: OnePoleLP,
    noise: Noise,
    drive: f32,
    active: bool,
}

impl CowbellVoice {
    pub fn new(sr: f64) -> Self {
        Self {
            sr,
            phase1: 0.0,
            phase2: 0.0,
            freq1: 545.0,
            freq2: 820.0,
            pulse_width: 0.5,
            snap_env: 0.0,
            snap_decay: 0.0,
            env: 0.0,
            env_decay: 0.0,
            hp: OnePoleHP::new(),
            lp: OnePoleLP::new(),
            noise: Noise::new(987),
            drive: 0.0,
            active: false,
        }
    }
}

impl DrumVoiceDsp for CowbellVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        self.phase1 = 0.0;
        self.phase2 = 0.0;
        // tune: base frequencies
        self.freq1 = 545.0 + p.tune * 100.0;
        // sweep: detune amount between oscillators
        self.freq2 = self.freq1 * (1.3 + p.sweep * 0.4);

        // color: pulse width of square waves
        self.pulse_width = 0.3 + p.color * 0.4;

        self.env = 1.0;
        let time = 0.05 + p.decay * 0.25;
        self.env_decay = (-5.0_f64 / (time as f64 * self.sr)).exp() as f32;

        // snap: attack pop
        self.snap_env = p.snap;
        self.snap_decay = (-5.0_f64 / (0.002 * self.sr)).exp() as f32;

        // filter: bandpass width
        let center = (self.freq1 + self.freq2) / 2.0;
        self.hp.set_freq(center * (0.3 + p.filter * 0.4), self.sr);
        self.lp.set_freq(center * (1.5 + p.filter * 1.5), self.sr);
        self.hp.prev_in = 0.0;
        self.hp.prev_out = 0.0;
        self.lp.prev_out = 0.0;

        self.drive = p.drive;
        self.active = true;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        self.phase1 += self.freq1 as f64 / self.sr;
        if self.phase1 >= 1.0 { self.phase1 -= 1.0; }
        let sq1: f32 = if self.phase1 < self.pulse_width as f64 { 1.0 } else { -1.0 };

        self.phase2 += self.freq2 as f64 / self.sr;
        if self.phase2 >= 1.0 { self.phase2 -= 1.0; }
        let sq2: f32 = if self.phase2 < self.pulse_width as f64 { 1.0 } else { -1.0 };

        let snap = self.noise.next() * self.snap_env;
        self.snap_env *= self.snap_decay;

        let sum = (sq1 + sq2) * 0.5 + snap;
        let filtered = self.lp.tick(self.hp.tick(sum));
        let driven = apply_drive(filtered, self.drive);
        let out = driven * self.env;

        self.env *= self.env_decay;
        if self.env < 1e-6 {
            self.active = false;
        }

        out
    }
}

// ===================================================================
// 8. TomVoice  (FM synthesis + waveshaping)
// ===================================================================
pub struct TomVoice {
    sr: f64,
    // Carrier oscillator
    phase: f64,
    // FM modulator
    mod_phase: f64,
    mod_feedback_state: f32,
    fm_depth: f32,
    fm_feedback_amt: f32,
    mod_freq_ratio: f32,
    // Waveshaping
    shape_amt: f32,
    // Envelopes
    freq_base: f32,
    pitch_env: f32,
    pitch_decay: f32,
    fm_env: f32,
    fm_env_decay: f32,
    env: f32,
    env_decay: f32,
    snap_env: f32,
    snap_decay: f32,
    // Noise (reduced mix)
    noise_mix: f32,
    lp: OnePoleLP,
    noise: Noise,
    drive: f32,
    active: bool,
}

impl TomVoice {
    pub fn new(sr: f64) -> Self {
        Self {
            sr,
            phase: 0.0,
            mod_phase: 0.0,
            mod_feedback_state: 0.0,
            fm_depth: 0.0,
            fm_feedback_amt: 0.0,
            mod_freq_ratio: 2.5,
            shape_amt: 0.0,
            freq_base: 150.0,
            pitch_env: 0.0,
            pitch_decay: 0.0,
            fm_env: 0.0,
            fm_env_decay: 0.0,
            env: 0.0,
            env_decay: 0.0,
            snap_env: 0.0,
            snap_decay: 0.0,
            noise_mix: 0.0,
            lp: OnePoleLP::new(),
            noise: Noise::new(555),
            drive: 0.0,
            active: false,
        }
    }
}

impl DrumVoiceDsp for TomVoice {
    fn trigger(&mut self, p: &DrumTrackParams) {
        self.phase = 0.0;
        self.mod_phase = 0.0;
        self.mod_feedback_state = 0.0;

        // tune: 60-350 Hz (wider range)
        self.freq_base = 60.0 + p.tune * 290.0;

        // sweep: pitch envelope depth
        self.pitch_env = p.sweep;
        let sweep_time = 0.01 + (1.0 - p.sweep) * 0.04;
        self.pitch_decay = (-5.0_f64 / (sweep_time as f64 * self.sr)).exp() as f32;

        self.env = 1.0;
        let time = 0.08 + p.decay * 0.42;
        self.env_decay = (-5.0_f64 / (time as f64 * self.sr)).exp() as f32;

        // color: FM depth and grit
        self.fm_depth = p.color * 1.2;
        self.fm_feedback_amt = p.color * 0.25;
        self.mod_freq_ratio = 2.0 + p.color * 1.5;
        // FM envelope for attack character
        self.fm_env = 1.0;
        let fm_env_time = 0.008 + p.color as f64 * 0.06;
        self.fm_env_decay = (-5.0_f64 / (fm_env_time * self.sr)).exp() as f32;

        // snap: waveshape amount + click transient
        self.shape_amt = p.snap * 1.5;
        self.snap_env = p.snap;
        self.snap_decay = (-5.0_f64 / (0.002 * self.sr)).exp() as f32;

        // Reduced noise mix — FM adds enough texture
        self.noise_mix = p.color * 0.15;

        // filter: LP on output
        let lp_freq = 200.0 + p.filter * 8000.0;
        self.lp.set_freq(lp_freq, self.sr);
        self.lp.prev_out = 0.0;

        self.drive = p.drive;
        self.active = true;
    }

    fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Pitch envelope: cubed for punchier drop
        let pitch_env_shaped = self.pitch_env * self.pitch_env * self.pitch_env;
        let freq = self.freq_base * (1.0 + 0.8 * pitch_env_shaped);
        self.pitch_env *= self.pitch_decay;

        // FM envelope
        let fm_env_shaped = self.fm_env * self.fm_env;
        self.fm_env *= self.fm_env_decay;

        // FM modulator with self-feedback
        let mod_freq = freq * self.mod_freq_ratio;
        self.mod_phase += mod_freq as f64 / self.sr;
        if self.mod_phase >= 1.0 {
            self.mod_phase -= 1.0;
        }
        let mod_lookup =
            self.mod_phase + (self.mod_feedback_state * self.fm_feedback_amt) as f64;
        let modulator = (mod_lookup * 2.0 * std::f64::consts::PI).sin() as f32;
        self.mod_feedback_state = modulator;

        // Carrier with FM
        let fm_amount = modulator * self.fm_depth * fm_env_shaped;
        self.phase += (freq + fm_amount * freq * 0.1) as f64 / self.sr;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let sine = (self.phase * 2.0 * std::f64::consts::PI).sin() as f32;

        // Cubic waveshaping
        let shaped = if self.shape_amt > 0.001 {
            (sine + self.shape_amt * sine * sine * sine) / (1.0 + self.shape_amt)
        } else {
            sine
        };

        // Reduced noise + click transient
        let noise = self.noise.next() * self.noise_mix;
        let snap = self.noise.next() * self.snap_env;
        self.snap_env *= self.snap_decay;

        let raw = shaped + noise + snap;
        let filtered = self.lp.tick(raw);
        let driven = apply_drive(filtered, self.drive);
        let out = driven * self.env;

        self.env *= self.env_decay;
        if self.env < 1e-6 {
            self.active = false;
        }

        out
    }
}

// ===================================================================
// Factory
// ===================================================================

/// Creates all 8 drum voices in order matching `DrumTrackId`:
/// Kick, Snare, ClosedHiHat, OpenHiHat, Ride, Clap, Cowbell, Tom
pub fn create_drum_voices(sample_rate: f64) -> [Box<dyn DrumVoiceDsp>; 8] {
    [
        Box::new(KickVoice::new(sample_rate)),
        Box::new(SnareVoice::new(sample_rate)),
        Box::new(ClosedHiHatVoice::new(sample_rate)),
        Box::new(OpenHiHatVoice::new(sample_rate)),
        Box::new(RideVoice::new(sample_rate)),
        Box::new(ClapVoice::new(sample_rate)),
        Box::new(CowbellVoice::new(sample_rate)),
        Box::new(TomVoice::new(sample_rate)),
    ]
}
