//! Send effects: Schroeder reverb, tempo-synced delay, tube saturator, RMS glue compressor.
//! Reverb/delay ported from zicbox applyReverb.h with adaptations for Rust.

// ---------------------------------------------------------------------------
// RampedParam: sample-accurate linear ramp for zipper-free parameter changes
// ---------------------------------------------------------------------------

/// Sample-accurate linear parameter ramp to prevent zipper noise.
/// Use for any parameter that changes in real-time (volume, cutoff, etc.).
#[derive(Clone, Copy, Debug)]
pub struct RampedParam {
    current: f32,
    target: f32,
    increment: f32,
    remaining: u32,
}

impl RampedParam {
    pub fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            increment: 0.0,
            remaining: 0,
        }
    }

    /// Set a new target value with a ramp duration in samples.
    /// For 10ms at 48kHz, use ramp_samples = 480.
    pub fn set(&mut self, target: f32, ramp_samples: u32) {
        self.target = target;
        if ramp_samples <= 1 {
            self.current = target;
            self.remaining = 0;
            self.increment = 0.0;
        } else {
            self.increment = (target - self.current) / ramp_samples as f32;
            self.remaining = ramp_samples;
        }
    }

    /// Get the next sample value, advancing the ramp by one step.
    #[inline]
    pub fn next(&mut self) -> f32 {
        if self.remaining > 0 {
            self.remaining -= 1;
            if self.remaining == 0 {
                self.current = self.target;
            } else {
                self.current += self.increment;
            }
        }
        self.current
    }

    /// Get the current value without advancing.
    #[inline]
    pub fn value(&self) -> f32 {
        self.current
    }
}

// Base comb/allpass lengths tuned for 44100 Hz; scaled by sample_rate / 44100.0
const BASE_COMB_LENGTHS: [usize; 4] = [1117, 1301, 1571, 1787];
const BASE_ALLPASS_LENGTHS: [usize; 2] = [557, 443];

/// Schroeder reverb: 4 parallel comb filters feeding 2 series allpass filters.
pub struct ReverbEffect {
    comb_buf: Vec<f32>,
    comb_lengths: [usize; 4],
    comb_offsets: [usize; 4],
    comb_pos: [usize; 4],
    comb_state: [f32; 4], // damping filter state per comb
    allpass_buf: Vec<f32>,
    allpass_lengths: [usize; 2],
    allpass_offsets: [usize; 2],
    allpass_pos: [usize; 2],
    feedback: f32,
    damping: f32,
    wet: f32,
    // Early reflections: 5 fixed taps for spatial definition before diffuse tail
    er_buf: Vec<f32>,
    er_buf_size: usize,
    er_pos: usize,
    er_taps: [usize; 5],
    er_gains: [f32; 5],
    er_wet: f32,
}

impl ReverbEffect {
    pub fn new(sample_rate: f64) -> Self {
        let scale = sample_rate / 44100.0;
        let comb_lengths: [usize; 4] = [
            (BASE_COMB_LENGTHS[0] as f64 * scale) as usize,
            (BASE_COMB_LENGTHS[1] as f64 * scale) as usize,
            (BASE_COMB_LENGTHS[2] as f64 * scale) as usize,
            (BASE_COMB_LENGTHS[3] as f64 * scale) as usize,
        ];
        let mut comb_offsets = [0usize; 4];
        let mut offset = 0;
        for i in 0..4 {
            comb_offsets[i] = offset;
            offset += comb_lengths[i];
        }
        let comb_total = offset;

        let allpass_lengths: [usize; 2] = [
            (BASE_ALLPASS_LENGTHS[0] as f64 * scale) as usize,
            (BASE_ALLPASS_LENGTHS[1] as f64 * scale) as usize,
        ];
        let mut allpass_offsets = [0usize; 2];
        let mut offset = 0;
        for i in 0..2 {
            allpass_offsets[i] = offset;
            offset += allpass_lengths[i];
        }
        let allpass_total = offset;

        // Early reflections: 5 taps at 3ms, 7ms, 11ms, 17ms, 23ms
        let er_delays_ms: [f64; 5] = [3.0, 7.0, 11.0, 17.0, 23.0];
        let er_buf_size = (sample_rate * 0.03) as usize + 1; // max 30ms
        let er_taps: [usize; 5] = [
            (er_delays_ms[0] * 0.001 * sample_rate) as usize,
            (er_delays_ms[1] * 0.001 * sample_rate) as usize,
            (er_delays_ms[2] * 0.001 * sample_rate) as usize,
            (er_delays_ms[3] * 0.001 * sample_rate) as usize,
            (er_delays_ms[4] * 0.001 * sample_rate) as usize,
        ];
        let er_gains: [f32; 5] = [0.35, 0.25, 0.20, 0.15, 0.10];

        Self {
            comb_buf: vec![0.0; comb_total],
            comb_lengths,
            comb_offsets,
            comb_pos: [0; 4],
            comb_state: [0.0; 4],
            allpass_buf: vec![0.0; allpass_total],
            allpass_lengths,
            allpass_offsets,
            allpass_pos: [0; 2],
            feedback: 0.7,
            damping: 0.25,
            wet: 0.3,
            er_buf: vec![0.0; er_buf_size],
            er_buf_size,
            er_pos: 0,
            er_taps,
            er_gains,
            er_wet: 0.3,
        }
    }

    /// Update reverb parameters. amount: 0-1, damping: 0-1.
    pub fn set_params(&mut self, amount: f32, damping: f32) {
        // Reduced feedback ceiling (0.85 max) for cleaner decay
        self.feedback = (0.50 + amount * 0.35).min(0.85);
        self.damping = damping;
        self.wet = amount * 0.7;
        self.er_wet = amount * 0.4; // early reflections slightly louder than diffuse tail
    }

    /// Process one sample of reverb input, return wet output.
    pub fn tick(&mut self, input: f32) -> f32 {
        // Early reflections: read tapped delays for spatial definition
        let mut er_sum = 0.0_f32;
        for i in 0..5 {
            let tap_pos = if self.er_pos >= self.er_taps[i] {
                self.er_pos - self.er_taps[i]
            } else {
                self.er_buf_size - (self.er_taps[i] - self.er_pos)
            };
            er_sum += self.er_buf[tap_pos] * self.er_gains[i];
        }
        self.er_buf[self.er_pos] = input;
        self.er_pos = (self.er_pos + 1) % self.er_buf_size;

        let mut comb_sum = 0.0_f32;

        for i in 0..4 {
            let len = self.comb_lengths[i];
            let base = self.comb_offsets[i];
            let pos = self.comb_pos[i];
            let delayed = self.comb_buf[base + pos];

            // Damped feedback: one-pole LP on the delayed signal
            self.comb_state[i] =
                delayed * (1.0 - self.damping) + self.comb_state[i] * self.damping;

            self.comb_buf[base + pos] = input + self.comb_state[i] * self.feedback;
            self.comb_pos[i] = (pos + 1) % len;

            comb_sum += delayed;
        }

        comb_sum *= 0.25;

        // 2 series allpass filters for diffusion
        let mut out = comb_sum;

        for i in 0..2 {
            let len = self.allpass_lengths[i];
            let base = self.allpass_offsets[i];
            let pos = self.allpass_pos[i];
            let buf_out = self.allpass_buf[base + pos];

            self.allpass_buf[base + pos] = out + buf_out * 0.5;
            out = -out + buf_out;

            self.allpass_pos[i] = (pos + 1) % len;
        }

        out * self.wet + er_sum * self.er_wet
    }
}

// ---------------------------------------------------------------------------
// Delay: tempo-synced circular buffer with LP-filtered feedback
// ---------------------------------------------------------------------------

const DELAY_BUF_SIZE: usize = 131072; // ~2.7s at 48kHz

/// Musical subdivision for delay time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DelaySub {
    Sixteenth,         // 1/16
    SixteenthDotted,   // 1/16D
    EighthTriplet,     // 1/8T
    Eighth,            // 1/8
    EighthDotted,      // 1/8D
    QuarterTriplet,    // 1/4T
    Quarter,           // 1/4
    QuarterDotted,     // 1/4D
    HalfTriplet,       // 1/2T
    Half,              // 1/2
}

pub const DELAY_SUBS: [DelaySub; 10] = [
    DelaySub::Sixteenth,
    DelaySub::SixteenthDotted,
    DelaySub::EighthTriplet,
    DelaySub::Eighth,
    DelaySub::EighthDotted,
    DelaySub::QuarterTriplet,
    DelaySub::Quarter,
    DelaySub::QuarterDotted,
    DelaySub::HalfTriplet,
    DelaySub::Half,
];

impl DelaySub {
    /// Select subdivision from a 0.0-1.0 parameter.
    pub fn from_param(v: f32) -> Self {
        let idx = (v * DELAY_SUBS.len() as f32) as usize;
        DELAY_SUBS[idx.min(DELAY_SUBS.len() - 1)]
    }

    /// Delay time in seconds for a given BPM.
    pub fn seconds(&self, bpm: f64) -> f64 {
        let beat = 60.0 / bpm; // quarter note duration
        match self {
            DelaySub::Sixteenth =>       beat * 0.25,
            DelaySub::SixteenthDotted => beat * 0.375,
            DelaySub::EighthTriplet =>   beat * 1.0 / 3.0,
            DelaySub::Eighth =>          beat * 0.5,
            DelaySub::EighthDotted =>    beat * 0.75,
            DelaySub::QuarterTriplet =>  beat * 2.0 / 3.0,
            DelaySub::Quarter =>         beat,
            DelaySub::QuarterDotted =>   beat * 1.5,
            DelaySub::HalfTriplet =>     beat * 4.0 / 3.0,
            DelaySub::Half =>            beat * 2.0,
        }
    }

    /// Short label for display.
    pub fn label(&self) -> &'static str {
        match self {
            DelaySub::Sixteenth =>       "1/16",
            DelaySub::SixteenthDotted => "1/16D",
            DelaySub::EighthTriplet =>   "1/8T",
            DelaySub::Eighth =>          "1/8",
            DelaySub::EighthDotted =>    "1/8D",
            DelaySub::QuarterTriplet =>  "1/4T",
            DelaySub::Quarter =>         "1/4",
            DelaySub::QuarterDotted =>   "1/4D",
            DelaySub::HalfTriplet =>     "1/2T",
            DelaySub::Half =>            "1/2",
        }
    }
}

pub struct DelayEffect {
    buf: Box<[f32; DELAY_BUF_SIZE]>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f32,
    // One-pole LP in feedback loop
    lp_state: f32,
    lp_coeff: f32, // alpha
    wet: f32,
}

impl DelayEffect {
    pub fn new() -> Self {
        Self {
            buf: Box::new([0.0; DELAY_BUF_SIZE]),
            write_pos: 0,
            delay_samples: 22050, // ~0.5s default
            feedback: 0.4,
            lp_state: 0.0,
            lp_coeff: 0.5,
            wet: 0.3,
        }
    }

    /// Update delay parameters.
    /// time: 0-1 (selects subdivision), feedback: 0-1, tone: 0-1 (LP cutoff in feedback).
    pub fn set_params(&mut self, time: f32, feedback: f32, tone: f32, bpm: f64, sample_rate: f64) {
        let sub = DelaySub::from_param(time);
        let delay_sec = sub.seconds(bpm);
        self.delay_samples = ((delay_sec * sample_rate) as usize).min(DELAY_BUF_SIZE - 1).max(1);
        self.feedback = feedback.min(0.95);
        self.wet = 0.4 + feedback * 0.4; // wet scales with feedback

        // LP cutoff: 1000-12000 Hz mapped from tone
        let freq = 1000.0 + tone as f64 * 11000.0;
        let rc = 1.0 / (2.0 * std::f64::consts::PI * freq);
        let dt = 1.0 / sample_rate;
        self.lp_coeff = (dt / (rc + dt)) as f32;
    }

    /// Process one sample of delay input, return wet output.
    pub fn tick(&mut self, input: f32) -> f32 {
        // Read from delay line
        let read_pos = if self.write_pos >= self.delay_samples {
            self.write_pos - self.delay_samples
        } else {
            DELAY_BUF_SIZE - (self.delay_samples - self.write_pos)
        };

        let delayed = self.buf[read_pos];

        // LP filter on feedback
        self.lp_state += self.lp_coeff * (delayed - self.lp_state);

        // Write input + filtered feedback
        self.buf[self.write_pos] = input + self.lp_state * self.feedback;
        self.write_pos = (self.write_pos + 1) % DELAY_BUF_SIZE;

        delayed * self.wet
    }
}

// ---------------------------------------------------------------------------
// Glue Compressor: feedforward RMS compressor with soft knee (SSL G-bus style)
//
// Single "amount" knob (0.0-1.0) controls:
//   - Threshold: 0dB (bypass) → -24dB (heavy compression)
//   - Ratio: 2:1 → 4:1
//   - Attack: 10ms → 3ms (lets transients through at low settings)
//   - Release: 200ms → 100ms
//   - Makeup gain: automatic
//
// Signal flow:
//   input → RMS detector → gain computer (soft knee) → gain smoother → VCA → output
// ---------------------------------------------------------------------------

pub struct GlueCompressor {
    // RMS level detection (exponential moving average of input²)
    rms_sq: f32,
    rms_coeff: f32,
    // Peak detection (fast attack for transients)
    peak_level: f32,
    peak_attack_coeff: f32,
    peak_release_coeff: f32,
    // Gain smoothing (separate attack/release)
    gain_smooth: f32, // current smoothed gain (linear, not dB)
    attack_coeff: f32,
    release_coeff: f32,
    // Compression parameters (derived from amount)
    threshold_db: f32,
    ratio: f32,
    knee_db: f32,
    makeup_linear: f32,
    // State
    amount: f32,
}

impl GlueCompressor {
    pub fn new(sample_rate: f64) -> Self {
        let mut comp = Self {
            rms_sq: 0.0,
            rms_coeff: 0.0,
            peak_level: 0.0,
            peak_attack_coeff: 0.0,
            peak_release_coeff: 0.0,
            gain_smooth: 1.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            threshold_db: 0.0,
            ratio: 2.0,
            knee_db: 6.0,
            makeup_linear: 1.0,
            amount: 0.0,
        };
        comp.set_amount(0.0, sample_rate);
        comp
    }

    /// Set compression amount (0.0 = off, 1.0 = heavy glue).
    pub fn set_amount(&mut self, amount: f32, sample_rate: f64) {
        self.amount = amount.clamp(0.0, 1.0);

        if self.amount < 0.001 {
            // Fully bypassed
            self.threshold_db = 0.0;
            self.ratio = 1.0;
            self.makeup_linear = 1.0;
            return;
        }

        // Threshold: -2dB at amount=0.01 → -24dB at amount=1.0
        self.threshold_db = -2.0 - self.amount * 22.0;

        // Ratio: 2:1 at low amount → 4:1 at max
        self.ratio = 2.0 + self.amount * 2.0;

        // Soft knee width (constant 6dB for transparency)
        self.knee_db = 6.0;

        // Attack: 10ms at amount=0 → 3ms at amount=1
        let attack_ms = 10.0 - self.amount as f64 * 7.0;
        self.attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp() as f32;

        // Release: 200ms at amount=0 → 100ms at amount=1
        let release_ms = 200.0 - self.amount as f64 * 100.0;
        self.release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp() as f32;

        // RMS window: ~10ms
        let rms_ms = 10.0;
        self.rms_coeff = (-1.0 / (rms_ms * 0.001 * sample_rate)).exp() as f32;

        // Peak detector: very fast attack (0.1ms), moderate release (5ms)
        // Catches transients that RMS misses
        let peak_attack_ms = 0.1;
        self.peak_attack_coeff = (-1.0 / (peak_attack_ms * 0.001 * sample_rate)).exp() as f32;
        let peak_release_ms = 5.0;
        self.peak_release_coeff = (-1.0 / (peak_release_ms * 0.001 * sample_rate)).exp() as f32;

        // Auto makeup gain: approximate the average gain reduction
        // At threshold with the given ratio, max GR ≈ threshold * (1 - 1/ratio)
        // We compensate for roughly half of that (sounds natural)
        let max_gr_db = self.threshold_db * (1.0 - 1.0 / self.ratio);
        let makeup_db = -max_gr_db * 0.5;
        self.makeup_linear = db_to_linear(makeup_db);
    }

    /// Process one sample. Returns compressed output.
    pub fn tick(&mut self, input: f32) -> f32 {
        if self.amount < 0.001 {
            return input;
        }

        // RMS detection: exponential moving average of squared input
        self.rms_sq = self.rms_coeff * self.rms_sq + (1.0 - self.rms_coeff) * input * input;

        // Convert RMS to dB (with floor to avoid log(0))
        let rms_db = linear_to_db(self.rms_sq.sqrt().max(1e-10));

        // Peak detection (envelope follower)
        let abs_input = input.abs();
        if abs_input > self.peak_level {
            self.peak_level = self.peak_attack_coeff * self.peak_level
                + (1.0 - self.peak_attack_coeff) * abs_input;
        } else {
            self.peak_level = self.peak_release_coeff * self.peak_level
                + (1.0 - self.peak_release_coeff) * abs_input;
        }
        let peak_db = linear_to_db(self.peak_level.max(1e-10));

        // Use the louder of RMS and peak for detection
        // Preserves transient response while still compressing sustained signals
        let detect_db = rms_db.max(peak_db);

        // Gain computation with soft knee
        let gain_db = compute_gain_db(detect_db, self.threshold_db, self.ratio, self.knee_db);

        // Convert to linear gain
        let target_gain = db_to_linear(gain_db);

        // Smooth gain with separate attack/release
        if target_gain < self.gain_smooth {
            // Attacking (gain decreasing = more compression)
            self.gain_smooth =
                self.attack_coeff * self.gain_smooth + (1.0 - self.attack_coeff) * target_gain;
        } else {
            // Releasing (gain increasing = less compression)
            self.gain_smooth =
                self.release_coeff * self.gain_smooth + (1.0 - self.release_coeff) * target_gain;
        }

        // Apply gain + makeup
        input * self.gain_smooth * self.makeup_linear
    }
}

// ---------------------------------------------------------------------------
// Tube Saturator: gentle asymmetric waveshaping for analog warmth
//
// Models a tube preamp stage with output transformer:
//   - Gentle asymmetric soft clipping produces even harmonics (warmth)
//   - Low input gain range (1x-2.5x) to stay in the "sweet spot"
//   - DC bias pushes into the nonlinear zone for asymmetry
//   - One-pole LP output filter (~8kHz) simulates transformer rolloff,
//     taming harsh upper harmonics for that smooth vintage sound
//   - Dry/wet mix so it's always musical — never harsh
//
// Drive 0.0 = bypass, 1.0 = rich tube warmth
// ---------------------------------------------------------------------------

pub struct TubeSaturator {
    drive: f32,       // 0.0-1.0
    bias: f32,        // DC offset target
    bias_smooth: f32, // smoothed bias (avoids clicks)
    lp_state: f32,    // output transformer LP filter state
    lp_coeff: f32,    // LP filter coefficient
    dc_block: f32,    // DC blocker state
    sample_rate: f32,
    oversampler: Oversampler2x, // 2× oversampling for clean harmonics
}

impl TubeSaturator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            drive: 0.0,
            bias: 0.0,
            bias_smooth: 0.0,
            lp_state: 0.0,
            lp_coeff: 0.3,
            dc_block: 0.0,
            sample_rate,
            oversampler: Oversampler2x::new(),
        }
    }

    /// Set drive amount (0.0 = clean, 1.0 = rich saturation).
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 1.0);
        // Gentle bias — just enough to create asymmetry
        self.bias = self.drive * 0.15;
        // Output transformer LP: rolls off more as drive increases
        // At drive=0: ~12kHz (barely noticeable), at drive=1: ~6kHz (warm)
        let cutoff_hz = 12000.0 - self.drive * 6000.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / self.sample_rate;
        self.lp_coeff = dt / (rc + dt);
    }

    /// Process one sample through tube saturation.
    pub fn tick(&mut self, input: f32) -> f32 {
        if self.drive < 0.001 {
            return input;
        }

        // Smooth the bias
        self.bias_smooth += 0.0005 * (self.bias - self.bias_smooth);

        // Gentle input gain (1x at drive=0, 2.5x at drive=1)
        let gain = 1.0 + self.drive * 1.5;
        let x = (input + self.bias_smooth) * gain;

        // Asymmetric waveshaping at 2× sample rate for clean harmonics
        let ref_level = if gain > 1.0 { gain / (1.0 + gain) } else { 1.0 };
        let compensated = self.oversampler.tick(x, |s| {
            let shaped = if s >= 0.0 {
                s / (1.0 + s.abs())
            } else {
                (s * 0.8).tanh() / (0.8_f32).tanh()
            };
            shaped / ref_level
        });

        // Output transformer: one-pole LP to roll off harsh upper harmonics
        self.lp_state += self.lp_coeff * (compensated - self.lp_state);

        // DC blocker (one-pole HP at ~5Hz) to remove any DC offset from asymmetry
        let output = self.lp_state - self.dc_block;
        let dc_coeff = 5.0 * 2.0 * std::f32::consts::PI / self.sample_rate;
        self.dc_block += dc_coeff * output;

        // Dry/wet mix: even at full drive, blend 70% wet / 30% dry
        // This keeps the transients alive and prevents mushiness
        let wet_amount = self.drive * 0.7;
        input * (1.0 - wet_amount) + output * wet_amount
    }
}

/// Soft-knee gain computation. Returns gain reduction in dB (negative or zero).
fn compute_gain_db(input_db: f32, threshold: f32, ratio: f32, knee: f32) -> f32 {
    let half_knee = knee * 0.5;

    if input_db < threshold - half_knee {
        // Below knee: no compression
        0.0
    } else if input_db > threshold + half_knee {
        // Above knee: full compression
        // gain_reduction = input - threshold - (input - threshold) / ratio
        //                = (input - threshold) * (1 - 1/ratio)
        // We want output = threshold + (input - threshold) / ratio
        // So gain = output - input = threshold + (input-threshold)/ratio - input
        //         = -(input - threshold) * (1 - 1/ratio)
        -(input_db - threshold) * (1.0 - 1.0 / ratio)
    } else {
        // In the knee: quadratic interpolation for smooth transition
        let x = input_db - threshold + half_knee;
        let slope = 1.0 - 1.0 / ratio;
        -(slope * x * x) / (2.0 * knee)
    }
}

#[inline]
fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

#[inline]
fn linear_to_db(linear: f32) -> f32 {
    20.0 * linear.log10()
}

// ---------------------------------------------------------------------------
// FdnReverb: 8-delay-line Feedback Delay Network
// ---------------------------------------------------------------------------

/// 8-delay-line Feedback Delay Network reverb.
/// Much denser and richer than Schroeder: Householder feedback matrix preserves energy,
/// prime-length delays prevent flutter echoes, per-line damping controls brightness.
pub struct FdnReverb {
    delays: Vec<f32>,       // single flat buffer for all 8 delay lines
    lengths: [usize; 8],
    offsets: [usize; 8],
    positions: [usize; 8],
    damping_state: [f32; 8],
    damping: f32,
    feedback: f32,
    wet: f32,
    // Early reflections (same as before)
    er_buf: Vec<f32>,
    er_buf_size: usize,
    er_pos: usize,
    er_taps: [usize; 5],
    er_gains: [f32; 5],
    er_wet: f32,
}

// Prime delay lengths tuned for 48kHz (~21-56ms range)
const FDN_BASE_LENGTHS: [usize; 8] = [1009, 1201, 1427, 1609, 1847, 2087, 2311, 2707];

impl FdnReverb {
    pub fn new(sample_rate: f64) -> Self {
        let scale = sample_rate / 48000.0;
        let mut lengths = [0usize; 8];
        let mut offsets = [0usize; 8];
        let mut offset = 0;
        for i in 0..8 {
            lengths[i] = (FDN_BASE_LENGTHS[i] as f64 * scale) as usize;
            offsets[i] = offset;
            offset += lengths[i];
        }
        let total = offset;

        // Early reflections: 5 taps at 3ms, 7ms, 11ms, 17ms, 23ms
        let er_delays_ms: [f64; 5] = [3.0, 7.0, 11.0, 17.0, 23.0];
        let er_buf_size = (sample_rate * 0.03) as usize + 1;
        let er_taps: [usize; 5] = [
            (er_delays_ms[0] * 0.001 * sample_rate) as usize,
            (er_delays_ms[1] * 0.001 * sample_rate) as usize,
            (er_delays_ms[2] * 0.001 * sample_rate) as usize,
            (er_delays_ms[3] * 0.001 * sample_rate) as usize,
            (er_delays_ms[4] * 0.001 * sample_rate) as usize,
        ];
        let er_gains: [f32; 5] = [0.35, 0.25, 0.20, 0.15, 0.10];

        Self {
            delays: vec![0.0; total],
            lengths,
            offsets,
            positions: [0; 8],
            damping_state: [0.0; 8],
            damping: 0.25,
            feedback: 0.7,
            wet: 0.3,
            er_buf: vec![0.0; er_buf_size],
            er_buf_size,
            er_pos: 0,
            er_taps,
            er_gains,
            er_wet: 0.3,
        }
    }

    pub fn set_params(&mut self, amount: f32, damping: f32) {
        self.feedback = (0.40 + amount * 0.45).min(0.85);
        self.damping = damping;
        self.wet = amount * 0.6;
        self.er_wet = amount * 0.4;
    }

    /// Process one mono sample, return mono wet output.
    pub fn tick(&mut self, input: f32) -> f32 {
        // Early reflections
        let mut er_sum = 0.0_f32;
        for i in 0..5 {
            let tap_pos = if self.er_pos >= self.er_taps[i] {
                self.er_pos - self.er_taps[i]
            } else {
                self.er_buf_size - (self.er_taps[i] - self.er_pos)
            };
            er_sum += self.er_buf[tap_pos] * self.er_gains[i];
        }
        self.er_buf[self.er_pos] = input;
        self.er_pos = (self.er_pos + 1) % self.er_buf_size;

        // Read all 8 delay lines and apply damping
        let mut read_vals = [0.0_f32; 8];
        for i in 0..8 {
            let pos = self.positions[i];
            let raw = self.delays[self.offsets[i] + pos];
            // One-pole LP damping
            self.damping_state[i] = raw * (1.0 - self.damping) + self.damping_state[i] * self.damping;
            read_vals[i] = self.damping_state[i];
        }

        // Householder feedback matrix: H = I - (2/N) * ones
        // For N=8: diagonal = 0.75, off-diagonal = -0.25
        // mixed[i] = 0.75 * read[i] - 0.25 * sum(read[j!=i])
        // Equivalently: mixed[i] = read[i] - 0.25 * sum(all read)
        let sum: f32 = read_vals.iter().sum();
        let mut mixed = [0.0_f32; 8];
        for i in 0..8 {
            mixed[i] = read_vals[i] - 0.25 * sum;
        }

        // Write back: input + feedback * mixed
        for i in 0..8 {
            let pos = self.positions[i];
            self.delays[self.offsets[i] + pos] = input + mixed[i] * self.feedback;
            self.positions[i] = (pos + 1) % self.lengths[i];
        }

        // Output: sum of delay reads normalized
        let out = sum * 0.125; // 1/8
        out * self.wet + er_sum * self.er_wet
    }

    /// Stereo output: odd delay lines -> left, even -> right for decorrelation.
    pub fn tick_stereo(&mut self, input: f32) -> (f32, f32) {
        // Early reflections (shared mono)
        let mut er_sum = 0.0_f32;
        for i in 0..5 {
            let tap_pos = if self.er_pos >= self.er_taps[i] {
                self.er_pos - self.er_taps[i]
            } else {
                self.er_buf_size - (self.er_taps[i] - self.er_pos)
            };
            er_sum += self.er_buf[tap_pos] * self.er_gains[i];
        }
        self.er_buf[self.er_pos] = input;
        self.er_pos = (self.er_pos + 1) % self.er_buf_size;

        let mut read_vals = [0.0_f32; 8];
        for i in 0..8 {
            let pos = self.positions[i];
            let raw = self.delays[self.offsets[i] + pos];
            self.damping_state[i] = raw * (1.0 - self.damping) + self.damping_state[i] * self.damping;
            read_vals[i] = self.damping_state[i];
        }

        let sum: f32 = read_vals.iter().sum();
        let mut mixed = [0.0_f32; 8];
        for i in 0..8 {
            mixed[i] = read_vals[i] - 0.25 * sum;
        }

        for i in 0..8 {
            let pos = self.positions[i];
            self.delays[self.offsets[i] + pos] = input + mixed[i] * self.feedback;
            self.positions[i] = (pos + 1) % self.lengths[i];
        }

        // Split odd/even for stereo
        let left = (read_vals[0] + read_vals[2] + read_vals[4] + read_vals[6]) * 0.25;
        let right = (read_vals[1] + read_vals[3] + read_vals[5] + read_vals[7]) * 0.25;
        let er_l = er_sum * 0.6;
        let er_r = er_sum * 0.4;

        (left * self.wet + er_l * self.er_wet, right * self.wet + er_r * self.er_wet)
    }
}

// ---------------------------------------------------------------------------
// Oversampler2x: 2x oversampler for clean nonlinear processing
// ---------------------------------------------------------------------------

/// 2x oversampler for clean nonlinear processing.
/// Processes saturation/distortion at double sample rate to reduce aliasing.
pub struct Oversampler2x {
    prev_in: f32,
}

impl Oversampler2x {
    pub fn new() -> Self {
        Self { prev_in: 0.0 }
    }

    /// Process one sample through a nonlinear function at 2x rate.
    /// Uses linear interpolation for upsampling, averaging for downsampling.
    #[inline]
    pub fn tick<F: FnMut(f32) -> f32>(&mut self, input: f32, mut f: F) -> f32 {
        let mid = (self.prev_in + input) * 0.5;
        self.prev_in = input;
        let y0 = f(mid);
        let y1 = f(input);
        (y0 + y1) * 0.5
    }
}

// ---------------------------------------------------------------------------
// LookaheadLimiter: true peak limiter with lookahead delay
// ---------------------------------------------------------------------------

/// True peak limiter with lookahead delay.
/// Sees peaks before they arrive and applies smooth gain reduction,
/// preserving transient shape while preventing clipping.
pub struct LookaheadLimiter {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    pos: usize,
    len: usize,
    threshold: f32,
    gain: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl LookaheadLimiter {
    pub fn new(sample_rate: f64) -> Self {
        let len = (sample_rate * 0.001) as usize; // 1ms lookahead
        let len = len.max(1);

        // Attack: ~0.1ms (catches transients)
        let attack_coeff = (-1.0 / (0.0001 * sample_rate)).exp() as f32;
        // Release: ~50ms (smooth recovery)
        let release_coeff = (-1.0 / (0.05 * sample_rate)).exp() as f32;

        Self {
            buf_l: vec![0.0; len],
            buf_r: vec![0.0; len],
            pos: 0,
            len,
            threshold: 0.95,
            gain: 1.0,
            attack_coeff,
            release_coeff,
        }
    }

    /// Process one stereo frame. Returns limited (l, r).
    pub fn tick_stereo(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        // Find peak in lookahead window (scan full buffer)
        let mut peak = 0.0_f32;
        for i in 0..self.len {
            peak = peak.max(self.buf_l[i].abs()).max(self.buf_r[i].abs());
        }
        // Also consider incoming samples
        peak = peak.max(in_l.abs()).max(in_r.abs());

        // Compute target gain
        let target_gain = if peak > self.threshold {
            self.threshold / peak
        } else {
            1.0
        };

        // Smooth gain with separate attack/release
        if target_gain < self.gain {
            self.gain = self.attack_coeff * self.gain + (1.0 - self.attack_coeff) * target_gain;
        } else {
            self.gain = self.release_coeff * self.gain + (1.0 - self.release_coeff) * target_gain;
        }

        // Read delayed output
        let out_l = self.buf_l[self.pos] * self.gain;
        let out_r = self.buf_r[self.pos] * self.gain;

        // Write new input
        self.buf_l[self.pos] = in_l;
        self.buf_r[self.pos] = in_r;
        self.pos = (self.pos + 1) % self.len;

        (out_l, out_r)
    }
}

// ---------------------------------------------------------------------------
// SidechainEnvelope: envelope follower for sidechain compression
// ---------------------------------------------------------------------------

/// Envelope follower for sidechain compression.
/// Tracks the amplitude of a trigger signal (e.g. kick drum) and produces
/// a gain reduction signal for ducking other instruments.
pub struct SidechainEnvelope {
    level: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl SidechainEnvelope {
    pub fn new(sample_rate: f64) -> Self {
        // Attack: ~1ms (fast catch of kick transient)
        let attack_coeff = (-1.0 / (0.001 * sample_rate)).exp() as f32;
        // Release: ~80ms (natural duck and recovery)
        let release_coeff = (-1.0 / (0.08 * sample_rate)).exp() as f32;

        Self {
            level: 0.0,
            attack_coeff,
            release_coeff,
        }
    }

    /// Feed a sidechain signal sample, returns current envelope level (0..~1).
    #[inline]
    pub fn tick(&mut self, input: f32) -> f32 {
        let abs = input.abs();
        if abs > self.level {
            self.level = self.attack_coeff * self.level + (1.0 - self.attack_coeff) * abs;
        } else {
            self.level = self.release_coeff * self.level + (1.0 - self.release_coeff) * abs;
        }
        self.level
    }

    /// Compute the gain multiplier for ducking. depth = how much to duck (0.5 = ~6dB).
    #[inline]
    pub fn duck_gain(&self, depth: f32) -> f32 {
        (1.0 - self.level * depth).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramped_param_instant_set() {
        let mut p = RampedParam::new(0.0);
        p.set(1.0, 1);
        assert!((p.next() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ramped_param_smooth_ramp() {
        let mut p = RampedParam::new(0.0);
        p.set(1.0, 4);
        let v1 = p.next();
        let v2 = p.next();
        let v3 = p.next();
        let v4 = p.next();
        assert!(v1 > 0.0 && v1 < 0.5);
        assert!(v2 > v1);
        assert!(v3 > v2);
        assert!((v4 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ramped_param_stays_at_target() {
        let mut p = RampedParam::new(0.5);
        assert!((p.next() - 0.5).abs() < 1e-6);
        assert!((p.next() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compressor_tames_peaks() {
        let sr = 48000.0;
        let mut comp = GlueCompressor::new(sr);
        comp.set_amount(0.7, sr);
        let loud = 0.9_f32;
        let mut out = 0.0;
        // Need enough samples for RMS + peak detector to respond
        for _ in 0..2000 {
            out = comp.tick(loud);
        }
        assert!(out < loud, "Compressed output {} should be less than input {}", out, loud);
        assert!(out > 0.0);
    }

    #[test]
    fn test_reverb_early_reflections() {
        let mut reverb = ReverbEffect::new(48000.0);
        reverb.set_params(0.5, 0.3);
        let _first = reverb.tick(1.0);
        let mut found_reflection = false;
        for _i in 0..960 {
            let out = reverb.tick(0.0);
            if out.abs() > 0.01 {
                found_reflection = true;
                break;
            }
        }
        assert!(found_reflection, "Should hear early reflections within 20ms");
    }

    #[test]
    fn test_reverb_decays() {
        let mut reverb = ReverbEffect::new(48000.0);
        reverb.set_params(1.0, 0.5);
        reverb.tick(1.0);
        let mut last = 1.0_f32;
        for _ in 0..48000 {
            last = reverb.tick(0.0);
        }
        assert!(last.abs() < 0.1, "Reverb should decay after 1s, got {}", last);
    }

    #[test]
    fn test_compressor_bypass() {
        let sr = 48000.0;
        let mut comp = GlueCompressor::new(sr);
        comp.set_amount(0.0, sr);
        let input = 0.5;
        let out = comp.tick(input);
        assert!((out - input).abs() < 1e-6);
    }

    #[test]
    fn test_ramped_param_retarget_mid_ramp() {
        let mut p = RampedParam::new(0.0);
        p.set(1.0, 100);
        let _ = p.next();
        p.set(0.0, 100);
        let v1 = p.next();
        let v2 = p.next();
        assert!(v2 < v1);
    }

    #[test]
    fn test_fdn_reverb_decays() {
        let mut reverb = FdnReverb::new(48000.0);
        reverb.set_params(1.0, 0.5);
        reverb.tick(1.0);
        let mut last = 1.0_f32;
        for _ in 0..48000 {
            last = reverb.tick(0.0);
        }
        assert!(last.abs() < 0.1, "FDN reverb should decay after 1s, got {}", last);
    }

    #[test]
    fn test_fdn_reverb_stereo_differs() {
        let mut reverb = FdnReverb::new(48000.0);
        reverb.set_params(0.5, 0.3);
        reverb.tick_stereo(1.0);
        // After some samples, L and R should differ (decorrelation)
        let mut l_differs_r = false;
        for _ in 0..2000 {
            let (l, r) = reverb.tick_stereo(0.0);
            if (l - r).abs() > 0.001 {
                l_differs_r = true;
                break;
            }
        }
        assert!(l_differs_r, "Stereo FDN should produce different L/R outputs");
    }

    #[test]
    fn test_lookahead_limiter_clamps_peaks() {
        let mut limiter = LookaheadLimiter::new(48000.0);
        // Feed silence to fill buffer, then a loud spike
        for _ in 0..100 {
            limiter.tick_stereo(0.0, 0.0);
        }
        // Feed a spike
        for _ in 0..50 {
            limiter.tick_stereo(2.0, 2.0);
        }
        // Read output — should be limited
        let mut max_out = 0.0_f32;
        for _ in 0..200 {
            let (l, r) = limiter.tick_stereo(0.0, 0.0);
            max_out = max_out.max(l.abs()).max(r.abs());
        }
        assert!(max_out < 1.0, "Limiter should prevent output > threshold, got {}", max_out);
    }

    #[test]
    fn test_sidechain_envelope_follows() {
        let mut env = SidechainEnvelope::new(48000.0);
        // Feed silence — level should be ~0
        for _ in 0..1000 {
            env.tick(0.0);
        }
        assert!(env.level < 0.01);
        // Feed loud signal — level should rise
        for _ in 0..100 {
            env.tick(0.8);
        }
        assert!(env.level > 0.3, "Envelope should follow loud input, got {}", env.level);
        // Duck gain should reduce
        assert!(env.duck_gain(0.5) < 0.9);
    }

    #[test]
    fn test_oversampler_passthrough() {
        let mut os = Oversampler2x::new();
        // Linear function should pass through unchanged
        let out = os.tick(1.0, |x| x);
        // First sample: mid = (0.0 + 1.0) * 0.5 = 0.5, output = (0.5 + 1.0) * 0.5 = 0.75
        // (slight latency artifact from interpolation — expected)
        assert!(out > 0.5 && out < 1.0);
    }

}
