# Audio Quality Improvements Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development to implement this plan.

**Goal:** Add 5 professional audio DSP features: sidechain compression, anti-aliased oscillators, lookahead limiter, 2× oversampling, and FDN reverb.

**Architecture:** Each DSP feature is a self-contained struct in `effects.rs` (or `synth_voice.rs` for PolyBLEP), wired into the audio callback in `engine.rs`. The FDN reverb replaces the drum bus Schroeder reverb only (synth reverbs stay lightweight). The lookahead limiter replaces the `soft_clip(tanh)` on master output.

**Tech Stack:** Rust, all DSP from scratch (no external crates), cpal audio callback at 48kHz.

---

## Task 1: PolyBLEP Anti-Aliased Oscillators

**Files:**
- Modify: `src/audio/synth_voice.rs` — `Oscillator::tick()` method

**What:** Add PolyBLEP (Polynomial Band-Limited Step) correction to saw and square waveforms. Sine and noise don't alias and need no changes.

**Implementation:**

Add a `poly_blep()` function:
```rust
/// PolyBLEP correction: smooths discontinuities in saw/square to reduce aliasing.
/// t = current phase (0..1), dt = phase increment per sample.
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
```

In `Oscillator::tick()`, apply correction after computing raw waveform:
- **Saw:** `raw_saw - poly_blep(phase, inc)` (subtract because saw has a falling edge at wrap)
- **Square:** `raw_square + poly_blep(phase, inc) - poly_blep((phase + 1.0 - pw) % 1.0, inc)`

**Test:** Add test that measures spectral energy above Nyquist/2 for saw wave at high frequency — PolyBLEP version should have less alias energy than naive version.

---

## Task 2: FDN Reverb (Drum Bus)

**Files:**
- Modify: `src/audio/effects.rs` — add `FdnReverb` struct
- Modify: `src/audio/engine.rs` — replace `drum_reverb: ReverbEffect` with `drum_reverb: FdnReverb`

**What:** 8-delay-line Feedback Delay Network with Hadamard mixing matrix. Keeps the existing early reflections tap system. Much denser and richer than the 4-comb Schroeder.

**Delay line lengths** (prime, scaled to sample_rate/48000):
`[1009, 1201, 1427, 1609, 1847, 2087, 2311, 2707]` (~21–56ms range)

**Signal flow per sample:**
1. Read all 8 delay outputs
2. Apply per-line damping (one-pole LP)
3. Mix via Hadamard matrix × feedback coefficient
4. Add input to each line
5. Write back to delay lines
6. Output = sum of delay outputs × (1/√8)

**Hadamard 8×8** (Householder reflection is simpler): `H[i][j] = if i==j { 0.75 } else { -0.25 }` — this is `I - 2/N * ones`.

**Interface:** Same as ReverbEffect — `set_params(amount, damping)` and `tick(input) -> f32`. Also add `tick_stereo(input) -> (f32, f32)` using odd/even delay lines for L/R.

**Test:** Verify decay, verify stereo decorrelation (L ≠ R after impulse).

---

## Task 3: 2× Oversampler

**Files:**
- Modify: `src/audio/effects.rs` — add `Oversampler2x` struct
- Modify: `src/audio/effects.rs` — wrap `TubeSaturator::tick()` with oversampling
- Modify: `src/audio/drum_voice.rs` — wrap `apply_drive()` with oversampling

**What:** Process nonlinear stages at double sample rate to push aliasing products above audible range. Uses linear interpolation for upsampling, averaging for downsampling (simple, effective for 2×).

```rust
pub struct Oversampler2x {
    prev_in: f32,
}

impl Oversampler2x {
    pub fn new() -> Self { Self { prev_in: 0.0 } }

    pub fn tick<F: FnMut(f32) -> f32>(&mut self, input: f32, mut f: F) -> f32 {
        let mid = (self.prev_in + input) * 0.5;
        self.prev_in = input;
        let y0 = f(mid);
        let y1 = f(input);
        (y0 + y1) * 0.5
    }
}
```

**Integration:**
- `TubeSaturator`: add `oversampler: Oversampler2x` field, use in `tick()`
- `apply_drive()` in drum_voice.rs: add `Oversampler2x` field to each voice that uses drive

**Test:** Process a loud high-frequency sine through saturator with/without oversampling, verify alias products are reduced.

---

## Task 4: Lookahead Limiter

**Files:**
- Modify: `src/audio/effects.rs` — add `LookaheadLimiter` struct
- Modify: `src/audio/engine.rs` — replace `soft_clip()` calls with limiter

**What:** True peak limiter with 1ms lookahead delay. Sees peaks before they arrive, applies smooth gain reduction to prevent clipping while preserving transient shape.

```rust
pub struct LookaheadLimiter {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    pos: usize,
    len: usize,         // lookahead in samples (~48 at 48kHz)
    threshold: f32,     // 0.95 (leave headroom for DAC)
    gain: f32,          // current gain (1.0 = no reduction)
    attack_coeff: f32,  // fast attack (~0.1ms)
    release_coeff: f32, // slow release (~50ms)
}
```

**Signal flow:**
1. Write incoming L/R to circular buffer
2. Scan lookahead window for peak (max absolute value)
3. If peak > threshold: target_gain = threshold / peak
4. Smooth gain with attack/release envelope
5. Read delayed L/R from buffer, apply gain
6. Return limited L/R

**Interface:** `tick_stereo(l: f32, r: f32) -> (f32, f32)`

**Test:** Feed a sudden spike — verify output never exceeds threshold, verify quiet signals pass unmodified.

---

## Task 5: Sidechain Compression

**Files:**
- Modify: `src/audio/effects.rs` — add `SidechainEnvelope` struct
- Modify: `src/audio/engine.rs` — wire kick detection → synth ducking

**What:** When kick fires, duck synths by ~6dB with fast attack / medium release. Classic technique for kick/bass separation.

```rust
pub struct SidechainEnvelope {
    level: f32,            // current envelope level (0..1)
    attack_coeff: f32,     // fast attack (~1ms)
    release_coeff: f32,    // medium release (~80ms)
}
```

**Signal flow in engine.rs:**
1. After drum voice tick loop, detect kick amplitude: `let kick_level = self.drum_voices[0].tick()` (kick is track 0, already computed)
2. Feed kick signal into `SidechainEnvelope`
3. Compute duck gain: `1.0 - envelope * depth` (depth ~0.5 = 6dB duck)
4. Apply duck gain to `synth_a_out` and `synth_b_out` before mixing

**Test:** Verify that when envelope is triggered, gain < 1.0; when idle, gain = 1.0.

---

## Wiring Order

1. PolyBLEP — self-contained in synth_voice.rs
2. FDN Reverb — add struct, swap drum_reverb type in engine.rs
3. Oversampler — add struct, integrate into TubeSaturator + drum voices
4. Lookahead Limiter — add struct, replace soft_clip in engine.rs
5. Sidechain — add struct, wire kick→synth ducking in engine.rs

Each feature gets its own commit.
