# Audio Quality Improvements Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the drum machine from "flat" to "punchy and glued" by improving dynamics processing, per-voice character, and effects quality.

**Architecture:** Seven independent improvements to the audio signal path, each modifying 1-2 files. Changes are additive — no existing parameters or UI controls are removed. New DSP modules (RampedParam, CombFilter, EarlyReflections) are added as small structs in existing files.

**Tech Stack:** Rust, f32 DSP, no external crate dependencies (all hand-rolled).

---

## File Structure

| File | Role | Changes |
|------|------|---------|
| `src/audio/effects.rs` | Send effects | Add `RampedParam`, peak detector to compressor, parallel compression, early reflections to reverb |
| `src/audio/drum_voice.rs` | Per-track DSP | Kick sub-oscillator, snare comb filter, hi-hat transient/sizzle |
| `src/audio/engine.rs` | Audio callback | Per-track soft-clip, parallel compression routing, use RampedParam for master_volume |
| `src/audio/mixer.rs` | Utilities | Add `per_track_saturate()` helper |

---

## Chunk 1: RampedParam + Per-Track Soft Clipping

### Task 1: Add RampedParam struct to effects.rs

**Files:**
- Modify: `src/audio/effects.rs` (add at top, before ReverbEffect)

- [ ] **Step 1: Write the RampedParam test**

Add at the bottom of `src/audio/effects.rs` inside a new `#[cfg(test)] mod tests {}`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramped_param_instant_set() {
        let mut p = RampedParam::new(0.0);
        p.set(1.0, 1); // 1-sample ramp = instant
        assert!((p.next() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ramped_param_smooth_ramp() {
        let mut p = RampedParam::new(0.0);
        p.set(1.0, 4); // 4-sample ramp
        let v1 = p.next();
        let v2 = p.next();
        let v3 = p.next();
        let v4 = p.next();
        assert!(v1 > 0.0 && v1 < 0.5); // ramping up
        assert!(v2 > v1);
        assert!(v3 > v2);
        assert!((v4 - 1.0).abs() < 1e-6); // reached target
    }

    #[test]
    fn test_ramped_param_stays_at_target() {
        let mut p = RampedParam::new(0.5);
        // No set() called — should stay at 0.5
        assert!((p.next() - 0.5).abs() < 1e-6);
        assert!((p.next() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ramped_param_retarget_mid_ramp() {
        let mut p = RampedParam::new(0.0);
        p.set(1.0, 100);
        let _ = p.next(); // start ramping
        p.set(0.0, 100);  // retarget mid-ramp
        // Should now be heading toward 0.0
        let v1 = p.next();
        let v2 = p.next();
        assert!(v2 < v1); // decreasing toward 0
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib effects::tests -- --nocapture 2>&1 | head -20`
Expected: FAIL — `RampedParam` not defined.

- [ ] **Step 3: Implement RampedParam**

Add after the `use` statements at the top of `src/audio/effects.rs`, before the reverb constants:

```rust
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib effects::tests -- --nocapture`
Expected: All 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/audio/effects.rs
git commit -m "feat(dsp): add RampedParam for zipper-free parameter changes"
```

---

### Task 2: Add per-track soft saturation to the drum bus

**Files:**
- Modify: `src/audio/mixer.rs` (add helper)
- Modify: `src/audio/engine.rs:432-445` (apply per-track saturation before summing)

- [ ] **Step 1: Write the per-track saturate test**

Add to the existing `#[cfg(test)] mod tests` in `src/audio/mixer.rs`:

```rust
#[test]
fn test_per_track_saturate_clean() {
    // Low-level signal passes through mostly unchanged
    let x = 0.1_f32;
    let y = per_track_saturate(x);
    assert!((y - x).abs() < 0.02);
}

#[test]
fn test_per_track_saturate_limits() {
    // High signal gets tamed but not hard-clipped
    let y = per_track_saturate(2.0);
    assert!(y > 0.8);
    assert!(y < 2.0); // reduced from input
}

#[test]
fn test_per_track_saturate_symmetry() {
    let v = 0.8;
    assert!((per_track_saturate(v) + per_track_saturate(-v)).abs() < 1e-6);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib mixer::tests -- --nocapture 2>&1 | head -10`
Expected: FAIL — `per_track_saturate` not found.

- [ ] **Step 3: Implement per_track_saturate**

Add to `src/audio/mixer.rs` after the `soft_clip` function:

```rust
/// Gentle per-track saturation using cubic soft-clip.
/// Adds subtle odd harmonics and tames peaks without killing transients.
/// More transparent than tanh — preserves the first ~0.8 of dynamic range linearly.
#[inline]
pub fn per_track_saturate(x: f32) -> f32 {
    if x > 1.0 {
        2.0 / 3.0 + (x - 1.0) / (1.0 + (x - 1.0) * (x - 1.0))
    } else if x < -1.0 {
        -2.0 / 3.0 + (x + 1.0) / (1.0 + (x + 1.0) * (x + 1.0))
    } else {
        x - (x * x * x) / 3.0
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib mixer::tests -- --nocapture`
Expected: All mixer tests PASS.

- [ ] **Step 5: Apply per-track saturation in engine.rs**

In `src/audio/engine.rs`, modify the drum voice summing loop (around line 432). Change the inner loop body from:

```rust
let voiced = sample * p.volume;
```

to:

```rust
let voiced = per_track_saturate(sample) * p.volume;
```

Also add the import at the top of `engine.rs`:

```rust
use crate::audio::mixer::{effective_mute, per_track_saturate, soft_clip};
```

- [ ] **Step 6: Run full test suite**

Run: `cargo test`
Expected: All tests PASS. No compile errors.

- [ ] **Step 7: Commit**

```bash
git add src/audio/mixer.rs src/audio/engine.rs
git commit -m "feat(dsp): add per-track soft saturation before drum bus summing"
```

---

## Chunk 2: Compressor Overhaul — Peak Detection + Parallel Compression

### Task 3: Add peak detection to GlueCompressor

**Files:**
- Modify: `src/audio/effects.rs` (GlueCompressor struct and tick method)

- [ ] **Step 1: Write compressor peak detection test**

Add to the tests module in `src/audio/effects.rs`:

```rust
#[test]
fn test_compressor_tames_peaks() {
    let sr = 48000.0;
    let mut comp = GlueCompressor::new(sr);
    comp.set_amount(0.7, sr);
    // Feed a loud transient
    let loud = 0.9_f32;
    let mut out = 0.0;
    for _ in 0..100 {
        out = comp.tick(loud);
    }
    // Compressed output should be lower than input
    assert!(out < loud);
    assert!(out > 0.0);
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
fn test_compressor_preserves_transient() {
    let sr = 48000.0;
    let mut comp = GlueCompressor::new(sr);
    comp.set_amount(0.5, sr);
    // First sample of a transient should pass through mostly intact
    // (peak detector hasn't reacted yet, but the transient window should let it through)
    let first = comp.tick(0.8);
    assert!(first > 0.5, "Transient should not be immediately squashed: {}", first);
}
```

- [ ] **Step 2: Run test to verify current behavior**

Run: `cargo test --lib effects::tests -- --nocapture`
Expected: Tests should compile. The `preserves_transient` test may fail initially — that's what we're fixing.

- [ ] **Step 3: Add peak detector alongside RMS in GlueCompressor**

Modify the `GlueCompressor` struct in `src/audio/effects.rs` to add a peak detector field:

Add after `rms_sq: f32,` / `rms_coeff: f32,`:

```rust
    // Peak detection (fast attack for transients)
    peak_level: f32,
    peak_attack_coeff: f32,  // ~0.1ms attack
    peak_release_coeff: f32, // ~5ms release — fast enough to not hold between hits
```

In `new()`, initialize these:

```rust
            peak_level: 0.0,
            peak_attack_coeff: 0.0,
            peak_release_coeff: 0.0,
```

In `set_amount()`, after the RMS window calculation, add:

```rust
        // Peak detector: very fast attack (0.1ms), moderate release (5ms)
        // This catches transients that RMS misses
        let peak_attack_ms = 0.1;
        self.peak_attack_coeff = (-1.0 / (peak_attack_ms * 0.001 * sample_rate)).exp() as f32;
        let peak_release_ms = 5.0;
        self.peak_release_coeff = (-1.0 / (peak_release_ms * 0.001 * sample_rate)).exp() as f32;
```

In `tick()`, replace the single RMS detection with a dual-mode detector. After the RMS calculation (`let rms_db = ...`), add:

```rust
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
        // This preserves transient response while still compressing sustained signals
        let detect_db = rms_db.max(peak_db);
```

Then change the gain computation line from:

```rust
        let gain_db = compute_gain_db(rms_db, self.threshold_db, self.ratio, self.knee_db);
```

to:

```rust
        let gain_db = compute_gain_db(detect_db, self.threshold_db, self.ratio, self.knee_db);
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib effects::tests -- --nocapture`
Expected: All compressor tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/audio/effects.rs
git commit -m "feat(dsp): add peak detection to compressor for better transient handling"
```

---

### Task 4: Add parallel compression in engine.rs

**Files:**
- Modify: `src/audio/effects.rs` (add a second "crush" compressor)
- Modify: `src/audio/engine.rs:556-576` (parallel compression routing)

- [ ] **Step 1: Write parallel compression test**

Add to `src/audio/effects.rs` tests:

```rust
#[test]
fn test_parallel_compression_louder_than_single() {
    let sr = 48000.0;
    let mut comp_main = GlueCompressor::new(sr);
    let mut comp_crush = GlueCompressor::new(sr);
    comp_main.set_amount(0.5, sr);
    comp_crush.set_amount(1.0, sr); // heavy compression

    let input = 0.3_f32; // moderate signal
    let mut main_out = 0.0;
    let mut crush_out = 0.0;
    for _ in 0..1000 {
        main_out = comp_main.tick(input);
        crush_out = comp_crush.tick(input);
    }
    // Parallel blend should be louder than main alone (crush brings up quiet parts)
    let parallel = main_out + crush_out * 0.3;
    assert!(parallel > main_out);
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test --lib effects::tests::test_parallel_compression -- --nocapture`
Expected: PASS (this test validates the concept, not new code).

- [ ] **Step 3: Add crush compressor to AudioEngine**

In `src/audio/engine.rs`, add a `crush_compressor` field to `AudioEngine`:

After `compressor: GlueCompressor,`:
```rust
    crush_compressor: GlueCompressor,
```

In `new()`, after `let compressor = GlueCompressor::new(sample_rate);`:
```rust
        let mut crush_compressor = GlueCompressor::new(sample_rate);
        crush_compressor.set_amount(1.0, sample_rate); // always heavy
```

Add it to the struct initialization:
```rust
            crush_compressor,
```

- [ ] **Step 4: Wire parallel compression into the mix bus**

In `engine.rs`, replace the linked stereo compression section (around lines 570-575):

Replace:
```rust
            // Linked stereo compression: detect from mono sum, apply gain to both channels
            let mono = (mixed_l + mixed_r) * 0.5;
            let compressed = self.compressor.tick(mono);
            let comp_gain = if mono.abs() > 1e-10 { compressed / mono } else { 1.0 };
            let out_l = soft_clip(mixed_l * comp_gain);
            let out_r = soft_clip(mixed_r * comp_gain);
```

With:
```rust
            // Linked stereo compression with parallel "crush" bus
            let mono = (mixed_l + mixed_r) * 0.5;
            let compressed = self.compressor.tick(mono);
            let comp_gain = if mono.abs() > 1e-10 { compressed / mono } else { 1.0 };

            // Parallel "crush" compression: heavily compressed copy blended in at 30%
            // This adds body and sustain without killing transients (New York compression)
            let crush = self.crush_compressor.tick(mono);
            let crush_gain = if mono.abs() > 1e-10 { crush / mono } else { 1.0 };
            let parallel_gain = comp_gain + crush_gain * 0.3;

            let out_l = soft_clip(mixed_l * parallel_gain);
            let out_r = soft_clip(mixed_r * parallel_gain);
```

- [ ] **Step 5: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/audio/effects.rs src/audio/engine.rs
git commit -m "feat(dsp): add parallel (New York) compression for punch + body"
```

---

## Chunk 3: Kick Sub-Oscillator

### Task 5: Add sub-oscillator layer to KickVoice

**Files:**
- Modify: `src/audio/drum_voice.rs` (KickVoice struct, trigger, tick — around lines 200-350)

- [ ] **Step 1: Add sub-oscillator fields to KickVoice**

In `src/audio/drum_voice.rs`, find the KickVoice struct (line ~200) and add these fields:

```rust
    // Sub-oscillator: one octave below for low-end weight
    sub_phase: f64,
    sub_freq: f32,
    sub_env: f32,
    sub_decay: f32,
```

- [ ] **Step 2: Initialize sub fields in KickVoice::new() or equivalent**

In the constructor/initialization, add:

```rust
            sub_phase: 0.0,
            sub_freq: 0.0,
            sub_env: 0.0,
            sub_decay: 0.0,
```

- [ ] **Step 3: Set sub-oscillator params in trigger()**

In `KickVoice::trigger()` (line ~249), after the main freq_base is set, add:

```rust
        // Sub-oscillator: one octave below the body fundamental
        self.sub_freq = self.freq_base * 0.5;
        self.sub_phase = 0.0;
        self.sub_env = 0.7; // slightly quieter than body
        // Longer decay than body — sub lingers
        self.sub_decay = (-1.0 / (self.sr as f32 * (0.15 + params.decay * 0.5))).exp();
```

- [ ] **Step 4: Mix sub-oscillator into tick()**

In `KickVoice::tick()` (line ~296), after the main body sample is computed but before the final output, add the sub:

```rust
        // Sub-oscillator: pure sine one octave below for chest-hitting low-end
        self.sub_phase += self.sub_freq as f64 / self.sr;
        let sub = (self.sub_phase * std::f64::consts::TAU).sin() as f32 * self.sub_env;
        self.sub_env *= self.sub_decay;
```

Then mix it into the final output. Find where `body` and `click` are combined (something like `let out = body + click;`) and change to:

```rust
        let out = body + click + sub;
```

- [ ] **Step 5: Build and test**

Run: `cargo build && cargo test`
Expected: Compiles and all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/audio/drum_voice.rs
git commit -m "feat(dsp): add sub-oscillator to kick for low-end weight"
```

---

## Chunk 4: Snare Comb Filter + Adjustable Tone/Noise Mix

### Task 6: Add comb filter resonance to SnareVoice

**Files:**
- Modify: `src/audio/drum_voice.rs` (SnareVoice — around lines 352-554)

- [ ] **Step 1: Add comb filter struct to drum_voice.rs**

Add a simple comb filter struct near the top of the file (after the filter structs, around line 180):

```rust
/// Simple feedforward comb filter for adding resonant character to noise.
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

    /// Set comb frequency and feedback.
    /// freq_hz: resonant frequency, sr: sample rate, fb: feedback 0.0-0.8
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
```

- [ ] **Step 2: Add comb filter to SnareVoice struct**

In the SnareVoice struct (line ~352), add:

```rust
    comb: CombFilter,
```

Initialize it in the constructor:

```rust
            comb: CombFilter::new(),
```

- [ ] **Step 3: Configure comb in trigger()**

In `SnareVoice::trigger()` (line ~405), after existing setup, add:

```rust
        // Shell resonance: comb filter tuned to 2x snare pitch for metallic ring
        let comb_freq = (120.0 + params.tune * 160.0) * 2.0; // ~240-560 Hz
        let comb_fb = 0.3 + params.color * 0.3; // more color = more resonance
        self.comb.set(comb_freq, self.sr, comb_fb);
```

- [ ] **Step 4: Route noise through comb filter in tick()**

In `SnareVoice::tick()` (line ~456), find where the noise component is computed and route it through the comb filter before mixing with tone:

After computing the filtered noise but before mixing tone + noise:

```rust
        let noise_resonated = self.comb.tick(noise_filtered);
```

Then replace the noise component in the mix with `noise_resonated`.

- [ ] **Step 5: Build and test**

Run: `cargo build && cargo test`
Expected: Compiles and all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/audio/drum_voice.rs
git commit -m "feat(dsp): add comb filter resonance to snare for shell character"
```

---

## Chunk 5: Hi-Hat Transient + Sizzle

### Task 7: Add bright transient and sizzle EQ to hi-hat voices

**Files:**
- Modify: `src/audio/drum_voice.rs` (ClosedHiHatVoice ~510, OpenHiHatVoice ~677)

- [ ] **Step 1: Add transient burst fields to ClosedHiHatVoice**

In the ClosedHiHatVoice struct (line ~510), add:

```rust
    // Bright click transient (~2ms noise burst at 4-10kHz)
    transient_env: f32,
    transient_decay: f32,
    transient_noise: Noise,
    // Sizzle: one-pole shelf boost around 10kHz
    sizzle_state: f32,
    sizzle_coeff: f32,
```

Initialize in constructor:

```rust
            transient_env: 0.0,
            transient_decay: 0.0,
            transient_noise: Noise::new(789),
            sizzle_state: 0.0,
            sizzle_coeff: 0.0,
```

- [ ] **Step 2: Configure transient in trigger()**

In `ClosedHiHatVoice::trigger()` (line ~556):

```rust
        // Bright transient: very short noise burst for attack definition
        self.transient_env = 0.5 + params.snap * 0.5; // snap controls transient intensity
        let transient_ms = 2.0; // 2ms burst
        self.transient_decay = (-1.0 / (self.sr as f32 * transient_ms * 0.001)).exp();

        // Sizzle: subtle high-shelf at ~10kHz
        let sizzle_freq = 10000.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * sizzle_freq);
        let dt = 1.0 / self.sr as f32;
        self.sizzle_coeff = dt / (rc + dt);
```

- [ ] **Step 3: Mix transient and sizzle into tick()**

In `ClosedHiHatVoice::tick()` (line ~592), after the main metallic signal is computed but before final output:

```rust
        // Add bright transient click
        let transient = self.transient_noise.next() * self.transient_env;
        self.transient_env *= self.transient_decay;

        // Sizzle: high-shelf boost (add high-passed version of signal)
        let sizzle_in = out + transient;
        self.sizzle_state += self.sizzle_coeff * (sizzle_in - self.sizzle_state);
        let hi_content = sizzle_in - self.sizzle_state; // HP = input - LP
        let out = sizzle_in + hi_content * 0.4; // boost highs by ~40%
```

- [ ] **Step 4: Apply same treatment to OpenHiHatVoice**

Repeat the same pattern for OpenHiHatVoice (line ~677): add the same transient/sizzle fields, initialize them, configure in trigger(), and mix into tick(). The transient decay can be slightly longer (3ms) for the open hat's more diffuse attack.

- [ ] **Step 5: Build and test**

Run: `cargo build && cargo test`
Expected: Compiles and all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/audio/drum_voice.rs
git commit -m "feat(dsp): add bright transient + sizzle EQ to hi-hat voices"
```

---

## Chunk 6: Reverb Early Reflections

### Task 8: Add early reflections to ReverbEffect

**Files:**
- Modify: `src/audio/effects.rs` (ReverbEffect struct and tick)

- [ ] **Step 1: Write early reflections test**

Add to the tests module in `src/audio/effects.rs`:

```rust
#[test]
fn test_reverb_early_reflections() {
    let mut reverb = ReverbEffect::new(48000.0);
    reverb.set_params(0.5, 0.3);
    // Feed an impulse
    let first = reverb.tick(1.0);
    // Feed silence — early reflections should produce output within first ~20ms (960 samples)
    let mut found_reflection = false;
    for i in 0..960 {
        let out = reverb.tick(0.0);
        if out.abs() > 0.01 {
            found_reflection = true;
            break;
        }
    }
    assert!(found_reflection, "Should hear early reflections within 20ms");
}

#[test]
fn test_reverb_reduced_feedback_ceiling() {
    let mut reverb = ReverbEffect::new(48000.0);
    reverb.set_params(1.0, 0.5); // max amount
    // Feed impulse then silence — should decay, not ring forever
    reverb.tick(1.0);
    let mut last = 1.0_f32;
    for _ in 0..48000 { // 1 second of silence
        last = reverb.tick(0.0);
    }
    assert!(last.abs() < 0.1, "Reverb should decay after 1s, got {}", last);
}
```

- [ ] **Step 2: Run tests to see current behavior**

Run: `cargo test --lib effects::tests -- --nocapture`
Expected: `test_reverb_early_reflections` may FAIL (no early reflections exist yet).

- [ ] **Step 3: Add early reflection tap delays to ReverbEffect**

In `ReverbEffect` struct, add:

```rust
    // Early reflections: 5 fixed taps for spatial definition before diffuse tail
    er_buf: Vec<f32>,
    er_buf_size: usize,
    er_pos: usize,
    er_taps: [usize; 5],   // tap positions in samples
    er_gains: [f32; 5],     // tap gain levels
    er_wet: f32,
```

In `new()`, compute the early reflection taps:

```rust
        // Early reflections: 5 taps at 3ms, 7ms, 11ms, 17ms, 23ms
        let er_delays_ms = [3.0, 7.0, 11.0, 17.0, 23.0];
        let er_buf_size = (sample_rate * 0.03) as usize + 1; // max 30ms
        let er_taps: [usize; 5] = [
            (er_delays_ms[0] * 0.001 * sample_rate) as usize,
            (er_delays_ms[1] * 0.001 * sample_rate) as usize,
            (er_delays_ms[2] * 0.001 * sample_rate) as usize,
            (er_delays_ms[3] * 0.001 * sample_rate) as usize,
            (er_delays_ms[4] * 0.001 * sample_rate) as usize,
        ];
        // Gains: first reflection loudest, decaying with distance
        let er_gains = [0.35, 0.25, 0.20, 0.15, 0.10];
```

Initialize in the struct:

```rust
            er_buf: vec![0.0; er_buf_size],
            er_buf_size,
            er_pos: 0,
            er_taps,
            er_gains,
            er_wet: 0.3,
```

In `set_params()`, scale ER wet with amount:

```rust
        self.er_wet = amount * 0.4; // early reflections slightly louder than diffuse tail
```

- [ ] **Step 4: Process early reflections in tick()**

In `ReverbEffect::tick()`, before the comb filter loop, add early reflection processing:

```rust
        // Early reflections: read tapped delays
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
```

Then at the end of `tick()`, change the return from:

```rust
        out * self.wet
```

to:

```rust
        out * self.wet + er_sum * self.er_wet
```

- [ ] **Step 5: Reduce feedback ceiling**

In `set_params()`, change:

```rust
        self.feedback = (0.50 + amount * 0.42).min(0.92);
```

to:

```rust
        // Reduced feedback ceiling (0.85 max) for cleaner decay
        self.feedback = (0.50 + amount * 0.35).min(0.85);
```

- [ ] **Step 6: Run tests**

Run: `cargo test --lib effects::tests -- --nocapture`
Expected: All reverb tests PASS.

- [ ] **Step 7: Run full test suite**

Run: `cargo test`
Expected: All 23+ tests PASS.

- [ ] **Step 8: Commit**

```bash
git add src/audio/effects.rs
git commit -m "feat(dsp): add early reflections to reverb + reduce feedback ceiling"
```

---

## Final Verification

- [ ] **Run full build + test**

```bash
cargo build --release && cargo test
```

- [ ] **Smoke test: launch the TUI and listen**

```bash
cargo run
```

Play a pattern and listen for:
1. Punchier kick (sub-oscillator adds weight)
2. Snare with more body/character (comb resonance)
3. Crisper hi-hats (transient click + sizzle)
4. Better dynamics (parallel compression = glue without flatness)
5. Clearer reverb (early reflections add space without mud)
6. No zipper noise when adjusting parameters (RampedParam)

---

## Summary of Changes

| File | Lines Changed (approx) | What |
|------|----------------------|------|
| `src/audio/effects.rs` | +120 | RampedParam, peak detector, early reflections, reduced reverb feedback |
| `src/audio/drum_voice.rs` | +80 | Kick sub-osc, snare comb filter, hi-hat transient+sizzle |
| `src/audio/engine.rs` | +15 | Per-track saturation, parallel compression routing |
| `src/audio/mixer.rs` | +20 | `per_track_saturate()` helper |

Total: ~235 new lines of DSP code across 4 files, 8 tasks, 6 chunks.
