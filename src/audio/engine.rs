//! Audio engine: cpal callback, voice management, LFO, message dispatch.

use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

use crate::audio::clock::SequencerClock;
use crate::audio::display_buffer::AudioDisplayBuffer;
use crate::audio::drum_voice::{create_drum_voices, DrumVoiceDsp};
use crate::audio::effects::{DelayEffect, FdnReverb, GlueCompressor, LookaheadLimiter, ReverbEffect, SidechainEnvelope, TubeSaturator};
use crate::audio::mixer::{effective_mute, per_track_saturate};
use crate::audio::synth_voice::SynthVoice;
use crate::messages::{AudioToUi, SynthId, UiToAudio};
use crate::params::EffectParams;
use crate::sequencer::drum_pattern::{DrumPattern, DrumTrackId, NUM_DRUM_TRACKS};
use crate::sequencer::synth_pattern::{SynthPattern, LFO_DEST_FIELDS, lfo_division_multiplier};
use crate::sequencer::transport::{PlayState, Transport};

/// Tempo-synced global LFO shared across synth voices.
/// Supports sine, triangle, saw (up/down), square, and exponential decay waveforms.
pub(crate) struct Lfo {
    phase: f64,
}

impl Lfo {
    fn new() -> Self {
        Self { phase: 0.0 }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Advance LFO and return value in -1.0..1.0.
    fn tick(&mut self, sample_rate: f64, bpm: f64, division_mult: f64, waveform: u8) -> f32 {
        let inc = (bpm / 60.0) * division_mult / sample_rate;
        self.phase += inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let p = self.phase as f32;
        match waveform {
            0 => {
                // Sine
                (p * 2.0 * std::f32::consts::PI).sin()
            }
            1 => {
                // Triangle
                if p < 0.25 {
                    p * 4.0
                } else if p < 0.75 {
                    2.0 - p * 4.0
                } else {
                    -4.0 + p * 4.0
                }
            }
            2 => {
                // Saw down
                1.0 - 2.0 * p
            }
            3 => {
                // Saw up
                -1.0 + 2.0 * p
            }
            4 => {
                // Square
                if p < 0.5 { 1.0 } else { -1.0 }
            }
            5 => {
                // Exponential decay: starts at +1, decays toward -1 each cycle
                (2.0 * (-5.0 * p).exp() - 1.0).clamp(-1.0, 1.0)
            }
            _ => 0.0,
        }
    }
}

/// Bundles all per-synth state (voice, pattern, effects, LFO).
pub struct SynthInstance {
    pub pattern: SynthPattern,
    pub voice: SynthVoice,
    pub gate_samples: u32,
    pub note_end_step: Option<usize>,
    pub lfo: Lfo,
    pub lfo2: Lfo,
    pub saturator: TubeSaturator,
    pub reverb: ReverbEffect,
    pub delay: DelayEffect,
}

impl SynthInstance {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            pattern: SynthPattern::default(),
            voice: SynthVoice::new(sample_rate as f32),
            gate_samples: 0,
            note_end_step: None,
            lfo: Lfo::new(),
            lfo2: Lfo::new(),
            saturator: TubeSaturator::new(sample_rate as f32),
            reverb: ReverbEffect::new(sample_rate),
            delay: {
                let mut d = DelayEffect::new();
                d.set_warm_mode(true); // synth delays: HP + saturation for character
                d
            },
        }
    }
}

/// Core audio engine running on the audio thread.
/// Owns all voices, effects, the sequencer clock, and handles messages from the UI thread.
pub struct AudioEngine {
    sample_rate: f64,
    clock: SequencerClock,

    // Local copies updated from UI messages
    transport: Transport,
    drum_pattern: DrumPattern,
    master_volume: f32,
    crossfader: f32, // 0.0=A, 0.5=center, 1.0=B

    // DSP
    drum_voices: [Box<dyn DrumVoiceDsp>; 8],
    synth_a: SynthInstance,
    synth_b: SynthInstance,

    // Send effects (drum bus)
    drum_reverb: FdnReverb,
    drum_delay: DelayEffect,
    compressor: GlueCompressor,
    crush_compressor: GlueCompressor, // parallel "New York" compression
    drum_saturator: TubeSaturator,
    effect_params: EffectParams,

    // Master bus
    limiter: LookaheadLimiter,
    sidechain: SidechainEnvelope,

    // Display buffer (shared with UI)
    display_buf: Arc<AudioDisplayBuffer>,
    peak_tracker: f32,

    // Channels
    rx: Receiver<UiToAudio>,
    tx: Sender<AudioToUi>,
}

impl AudioEngine {
    pub fn new(sample_rate: f64, rx: Receiver<UiToAudio>, tx: Sender<AudioToUi>, display_buf: Arc<AudioDisplayBuffer>) -> Self {
        let effect_params = EffectParams::default();
        let mut drum_reverb = FdnReverb::new(sample_rate);
        let mut drum_delay = DelayEffect::new();
        drum_reverb.set_params(effect_params.reverb_amount, effect_params.reverb_damping);
        drum_delay.set_single_knob(effect_params.delay_time, 120.0, sample_rate);

        let compressor = GlueCompressor::new(sample_rate);
        let mut crush_compressor = GlueCompressor::new(sample_rate);
        crush_compressor.set_amount(1.0, sample_rate); // always heavy
        let drum_saturator = TubeSaturator::new(sample_rate as f32);

        Self {
            sample_rate,
            clock: SequencerClock::new(),
            transport: Transport::default(),
            drum_pattern: DrumPattern::default(),
            master_volume: 0.8,
            crossfader: 0.5,
            drum_voices: create_drum_voices(sample_rate),
            synth_a: SynthInstance::new(sample_rate),
            synth_b: SynthInstance::new(sample_rate),
            drum_reverb,
            drum_delay,
            compressor,
            crush_compressor,
            drum_saturator,
            effect_params,
            limiter: LookaheadLimiter::new(sample_rate),
            sidechain: SidechainEnvelope::new(sample_rate),
            display_buf,
            peak_tracker: 0.0,
            rx,
            tx,
        }
    }

    /// Main audio processing function called from the cpal callback.
    pub fn process(&mut self, buffer: &mut [f32]) {
        // 1. Drain all pending UI messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                UiToAudio::SetTransport(t) => {
                    let prev_state = self.transport.state;
                    let bpm_changed = (self.transport.bpm - t.bpm).abs() > 0.01;
                    self.transport = t;
                    if self.transport.state == PlayState::Stopped
                        && prev_state != PlayState::Stopped
                    {
                        self.clock.reset();
                        self.synth_a.lfo.reset();
                        self.synth_b.lfo.reset();
                        self.synth_a.note_end_step = None;
                        self.synth_b.note_end_step = None;
                    }
                    // Update delay time when BPM changes
                    if bpm_changed {
                        let dt = self.effect_params.delay_time;
                        self.drum_delay.set_single_knob(dt, self.transport.bpm, self.sample_rate);
                        self.synth_a.delay.set_single_knob(dt, self.transport.bpm, self.sample_rate);
                        self.synth_b.delay.set_single_knob(dt, self.transport.bpm, self.sample_rate);
                    }
                }
                UiToAudio::SetDrumPattern(p) => {
                    self.drum_pattern = p;
                }
                UiToAudio::SetSynthPattern(synth_id, p) => {
                    match synth_id {
                        SynthId::A => {
                            self.synth_a.pattern = p;
                            self.synth_a.note_end_step = None;
                        }
                        SynthId::B => {
                            self.synth_b.pattern = p;
                            self.synth_b.note_end_step = None;
                        }
                    }
                }
                UiToAudio::SetEffectParams(ep) => {
                    self.effect_params = ep;
                    self.drum_reverb
                        .set_params(ep.reverb_amount, ep.reverb_damping);
                    self.drum_delay.set_single_knob(
                        ep.delay_time, self.transport.bpm, self.sample_rate,
                    );
                    self.compressor
                        .set_amount(ep.compressor_amount, self.sample_rate);
                    self.master_volume = ep.master_volume;
                    self.crossfader = ep.crossfader;
                    self.drum_saturator.set_drive(ep.drum_saturator_drive);
                    self.synth_a.saturator.set_drive(ep.synth_saturator_drive);
                    self.synth_b.saturator.set_drive(ep.synth_saturator_drive);
                    self.synth_a.delay.set_single_knob(
                        ep.delay_time, self.transport.bpm, self.sample_rate,
                    );
                    self.synth_b.delay.set_single_knob(
                        ep.delay_time, self.transport.bpm, self.sample_rate,
                    );
                }
                UiToAudio::TriggerDrum(track_id) => {
                    let track = track_id as usize;
                    let p = &self.drum_pattern.params[track];
                    self.drum_voices[track].trigger(p);
                    if track_id == DrumTrackId::ClosedHiHat {
                        self.drum_voices[DrumTrackId::OpenHiHat as usize].choke();
                    }
                }
                UiToAudio::TriggerSynth(synth_id, note) => {
                    let inst = match synth_id {
                        SynthId::A => &mut self.synth_a,
                        SynthId::B => &mut self.synth_b,
                    };
                    inst.voice.trigger(&inst.pattern.params, note);
                    // Gate for ~half a step (will be released when gate runs out)
                    let samples_per_step = (self.sample_rate * 60.0 / self.transport.bpm / 4.0) as u32;
                    inst.gate_samples = samples_per_step * 3 / 4;
                }
            }
        }

        // Build mute/solo arrays for effective_mute
        let mut muted = [false; 8];
        let mut soloed = [false; 8];
        for i in 0..NUM_DRUM_TRACKS {
            muted[i] = self.drum_pattern.params[i].mute;
            soloed[i] = self.drum_pattern.params[i].solo;
        }

        let drum_loop_len = if self.transport.loop_config.enabled {
            self.transport.loop_config.drum_length as usize
        } else {
            crate::sequencer::drum_pattern::MAX_STEPS
        };
        let synth_a_loop_len = if self.transport.loop_config.enabled {
            self.transport.loop_config.synth_a_length as usize
        } else {
            crate::sequencer::synth_pattern::MAX_STEPS
        };
        let synth_b_loop_len = if self.transport.loop_config.enabled {
            self.transport.loop_config.synth_b_length as usize
        } else {
            crate::sequencer::synth_pattern::MAX_STEPS
        };

        // 2. Process each sample frame (stereo interleaved)
        for frame in buffer.chunks_mut(2) {
            // Advance clock if playing
            if self.transport.state == PlayState::Playing {
                if let Some(event) = self.clock.advance(
                    self.transport.bpm,
                    self.sample_rate,
                    self.transport.swing,
                ) {
                    // Map free-running global_step into per-instrument pattern positions
                    let drum_step = event.global_step % drum_loop_len.max(1);
                    let synth_a_step = event.global_step % synth_a_loop_len.max(1);
                    let synth_b_step = event.global_step % synth_b_loop_len.max(1);
                    let pattern_step = drum_step;

                    // Trigger drum voices for active steps
                    let mut triggered: u8 = 0;
                    for track in 0..NUM_DRUM_TRACKS {
                        if self.drum_pattern.steps[track][pattern_step] {
                            if !effective_mute(track, &muted, &soloed) {
                                let p = &self.drum_pattern.params[track];
                                self.drum_voices[track].trigger(p);
                                triggered |= 1 << track;

                                // Hihat choke: closed hihat chokes open hihat
                                if track == DrumTrackId::ClosedHiHat as usize {
                                    self.drum_voices[DrumTrackId::OpenHiHat as usize].choke();
                                }
                            }
                        }
                    }

                    let samples_per_step = (self.sample_rate * 60.0 / self.transport.bpm / 4.0) as u32;

                    // --- Synth A: trigger voice for active steps (with multi-step note length) ---
                    let mut synth_a_triggered = false;
                    {
                        let step_data = &self.synth_a.pattern.steps[synth_a_step];
                        if step_data.is_active() && !self.synth_a.pattern.params.mute {
                            self.synth_a.voice.trigger(&self.synth_a.pattern.params, step_data.note);
                            synth_a_triggered = true;

                            let length = (step_data.length as usize).max(1);
                            if length <= 1 {
                                self.synth_a.gate_samples = samples_per_step * 3 / 4;
                                self.synth_a.note_end_step = None;
                            } else {
                                let end_step = (synth_a_step + length - 1).min(synth_a_loop_len.max(1) - 1);
                                self.synth_a.gate_samples = samples_per_step * length as u32 - samples_per_step / 4;
                                self.synth_a.note_end_step = Some(end_step);
                            }
                        } else if let Some(end) = self.synth_a.note_end_step {
                            if synth_a_step == end {
                                self.synth_a.gate_samples = samples_per_step * 3 / 4;
                                self.synth_a.note_end_step = None;
                            }
                        } else if !step_data.is_active() && self.synth_a.gate_samples > 0 {
                            self.synth_a.voice.release();
                            self.synth_a.gate_samples = 0;
                        }
                    }

                    // --- Synth B: trigger voice for active steps (with multi-step note length) ---
                    let mut synth_b_triggered = false;
                    {
                        let step_data = &self.synth_b.pattern.steps[synth_b_step];
                        if step_data.is_active() && !self.synth_b.pattern.params.mute {
                            self.synth_b.voice.trigger(&self.synth_b.pattern.params, step_data.note);
                            synth_b_triggered = true;

                            let length = (step_data.length as usize).max(1);
                            if length <= 1 {
                                self.synth_b.gate_samples = samples_per_step * 3 / 4;
                                self.synth_b.note_end_step = None;
                            } else {
                                let end_step = (synth_b_step + length - 1).min(synth_b_loop_len.max(1) - 1);
                                self.synth_b.gate_samples = samples_per_step * length as u32 - samples_per_step / 4;
                                self.synth_b.note_end_step = Some(end_step);
                            }
                        } else if let Some(end) = self.synth_b.note_end_step {
                            if synth_b_step == end {
                                self.synth_b.gate_samples = samples_per_step * 3 / 4;
                                self.synth_b.note_end_step = None;
                            }
                        } else if !step_data.is_active() && self.synth_b.gate_samples > 0 {
                            self.synth_b.voice.release();
                            self.synth_b.gate_samples = 0;
                        }
                    }

                    // Send playback position to UI (drop if channel is full)
                    let _ = self.tx.try_send(AudioToUi::PlaybackPosition {
                        global_step: event.global_step,
                        beat: event.beat,
                        is_bar_start: event.is_bar_start,
                        triggered,
                        synth_a_triggered,
                        drum_step,
                        synth_a_step,
                        synth_b_step,
                        synth_b_triggered,
                    });
                }
            }

            // Decrement synth gate counters and release when expired
            if self.synth_a.gate_samples > 0 {
                self.synth_a.gate_samples -= 1;
                if self.synth_a.gate_samples == 0 {
                    self.synth_a.voice.release();
                }
            }
            if self.synth_b.gate_samples > 0 {
                self.synth_b.gate_samples -= 1;
                if self.synth_b.gate_samples == 0 {
                    self.synth_b.voice.release();
                }
            }

            // Generate drum audio: sum all voices with per-track stereo panning
            let mut drum_dry_l: f32 = 0.0;
            let mut drum_dry_r: f32 = 0.0;
            let mut drum_dry_mono: f32 = 0.0;
            let mut reverb_send: f32 = 0.0;
            let mut delay_send: f32 = 0.0;
            let mut kick_sample: f32 = 0.0;

            for track in 0..NUM_DRUM_TRACKS {
                let sample = self.drum_voices[track].tick();
                // Capture kick for sidechain (track 0, before mute check)
                if track == 0 {
                    kick_sample = sample;
                }
                if !effective_mute(track, &muted, &soloed) {
                    let p = &self.drum_pattern.params[track];
                    let voiced = per_track_saturate(sample) * p.volume;
                    // Equal-power pan law
                    let pan_angle = p.pan * std::f32::consts::FRAC_PI_2;
                    drum_dry_l += voiced * pan_angle.cos();
                    drum_dry_r += voiced * pan_angle.sin();
                    drum_dry_mono += voiced;
                    reverb_send += voiced * p.send_reverb;
                    delay_send += voiced * p.send_delay;
                }
            }

            // Apply drum bus volume + saturator on mono sum, then distribute to stereo
            let drum_vol = self.effect_params.drum_volume;
            let mono_scaled = drum_dry_mono * drum_vol;
            let drum_sat_mono = self.drum_saturator.tick(mono_scaled);
            // Preserve stereo balance: apply saturator's gain change to both channels
            let sat_gain = if mono_scaled.abs() > 1e-10 { drum_sat_mono / mono_scaled } else { 1.0 };
            let drum_sat_l = drum_dry_l * drum_vol * sat_gain;
            let drum_sat_r = drum_dry_r * drum_vol * sat_gain;
            reverb_send *= drum_vol;
            delay_send *= drum_vol;

            // --- Generate synth A audio with LFO modulation + per-instrument FX ---
            let synth_a_out = {
                let synth_params = &self.synth_a.pattern.params;
                let mut modulated_params = *synth_params;
                if synth_params.lfo_depth > 0.001 {
                    let div_mult = lfo_division_multiplier(synth_params.lfo_division);
                    let lfo_val = self.synth_a.lfo.tick(
                        self.sample_rate,
                        self.transport.bpm,
                        div_mult,
                        synth_params.lfo_waveform,
                    );
                    let mod_amount = lfo_val * synth_params.lfo_depth;
                    let dest_idx = synth_params.lfo_dest as usize;
                    if dest_idx < LFO_DEST_FIELDS.len() {
                        let field = LFO_DEST_FIELDS[dest_idx];
                        let current = field.get(&modulated_params);
                        field.set(&mut modulated_params, current + mod_amount);
                    }
                }
                if synth_params.lfo2_depth > 0.001 {
                    let div_mult = lfo_division_multiplier(synth_params.lfo2_division);
                    let lfo_val = self.synth_a.lfo2.tick(
                        self.sample_rate,
                        self.transport.bpm,
                        div_mult,
                        synth_params.lfo2_waveform,
                    );
                    let mod_amount = lfo_val * synth_params.lfo2_depth;
                    let dest_idx = synth_params.lfo2_dest as usize;
                    if dest_idx < LFO_DEST_FIELDS.len() {
                        let field = LFO_DEST_FIELDS[dest_idx];
                        let current = field.get(&modulated_params);
                        field.set(&mut modulated_params, current + mod_amount);
                    }
                }
                let synth_sample = self.synth_a.voice.tick(&modulated_params);
                let mut synth_dry: f32 = 0.0;
                if !self.synth_a.pattern.params.mute {
                    synth_dry = synth_sample;
                    // Synth uses its own reverb/delay instances — don't also feed drum bus
                }
                let sa_sat = self.synth_a.saturator.tick(synth_dry);
                let sa_reverb = self.synth_a.reverb.tick(sa_sat * self.synth_a.pattern.params.send_reverb);
                let sa_delay = self.synth_a.delay.tick(sa_sat * self.synth_a.pattern.params.send_delay);
                sa_sat + sa_reverb + sa_delay
            };

            // --- Generate synth B audio with LFO modulation + per-instrument FX ---
            let synth_b_out = {
                let synth_params = &self.synth_b.pattern.params;
                let mut modulated_params = *synth_params;
                if synth_params.lfo_depth > 0.001 {
                    let div_mult = lfo_division_multiplier(synth_params.lfo_division);
                    let lfo_val = self.synth_b.lfo.tick(
                        self.sample_rate,
                        self.transport.bpm,
                        div_mult,
                        synth_params.lfo_waveform,
                    );
                    let mod_amount = lfo_val * synth_params.lfo_depth;
                    let dest_idx = synth_params.lfo_dest as usize;
                    if dest_idx < LFO_DEST_FIELDS.len() {
                        let field = LFO_DEST_FIELDS[dest_idx];
                        let current = field.get(&modulated_params);
                        field.set(&mut modulated_params, current + mod_amount);
                    }
                }
                if synth_params.lfo2_depth > 0.001 {
                    let div_mult = lfo_division_multiplier(synth_params.lfo2_division);
                    let lfo_val = self.synth_b.lfo2.tick(
                        self.sample_rate,
                        self.transport.bpm,
                        div_mult,
                        synth_params.lfo2_waveform,
                    );
                    let mod_amount = lfo_val * synth_params.lfo2_depth;
                    let dest_idx = synth_params.lfo2_dest as usize;
                    if dest_idx < LFO_DEST_FIELDS.len() {
                        let field = LFO_DEST_FIELDS[dest_idx];
                        let current = field.get(&modulated_params);
                        field.set(&mut modulated_params, current + mod_amount);
                    }
                }
                let synth_sample = self.synth_b.voice.tick(&modulated_params);
                let mut synth_dry: f32 = 0.0;
                if !self.synth_b.pattern.params.mute {
                    synth_dry = synth_sample;
                    // Synth uses its own reverb/delay instances — don't also feed drum bus
                }
                let sb_sat = self.synth_b.saturator.tick(synth_dry);
                let sb_reverb = self.synth_b.reverb.tick(sb_sat * self.synth_b.pattern.params.send_reverb);
                let sb_delay = self.synth_b.delay.tick(sb_sat * self.synth_b.pattern.params.send_delay);
                sb_sat + sb_reverb + sb_delay
            };

            // Process send effects (drum bus) — FDN reverb with native stereo output
            let (reverb_l, reverb_r) = self.drum_reverb.tick_stereo(reverb_send);
            let delay_out = self.drum_delay.tick(delay_send);

            // Sidechain: kick ducks synths for separation (depth from effect params)
            self.sidechain.tick(kick_sample);
            let duck = if self.effect_params.sidechain_amount > 0.001 {
                self.sidechain.duck_gain(self.effect_params.sidechain_amount)
            } else {
                1.0
            };

            // Apply crossfader gain: constant-power (Octatrack-style) cosine curve.
            // Center (0.5) = both at ~0.707 (−3dB), extremes = one full / one silent.
            // Total power stays constant across the full range — no volume dip.
            let xf = self.crossfader;
            let angle = xf * std::f32::consts::FRAC_PI_2;
            let gain_a = angle.cos();
            let gain_b = angle.sin();

            // Stereo mix: synths panned slightly to opposite sides, ducked by kick
            // Synth A slightly left (pan ~0.38), Synth B slightly right (pan ~0.62)
            // Precomputed equal-power gains: cos/sin(0.38 * π/2) ≈ 0.826/0.564
            let sa_scaled = synth_a_out * gain_a * duck;
            let sb_scaled = synth_b_out * gain_b * duck;
            let synth_l = sa_scaled * 0.826 + sb_scaled * 0.564;
            let synth_r = sa_scaled * 0.564 + sb_scaled * 0.826;

            // Stereo wet bus: FDN reverb is natively stereo, delay decorrelated via reverb spread
            let wet_l = synth_l + reverb_l + delay_out;
            let wet_r = synth_r + reverb_r + delay_out;

            let mixed_l = (drum_sat_l + wet_l) * 0.7 * self.master_volume;
            let mixed_r = (drum_sat_r + wet_r) * 0.7 * self.master_volume;
            // Linked stereo compression with parallel "crush" bus
            let mono = (mixed_l + mixed_r) * 0.5;
            let compressed = self.compressor.tick(mono);
            let comp_gain = if mono.abs() > 1e-10 { compressed / mono } else { 1.0 };

            // Parallel "crush" compression: only active when compressor knob is engaged.
            // Adds body and sustain without killing transients (New York compression)
            let crush = self.crush_compressor.tick(mono);
            let crush_blend = if self.effect_params.compressor_amount > 0.001 { 0.3 } else { 0.0 };
            let crush_gain = if mono.abs() > 1e-10 { crush / mono } else { 1.0 };
            let parallel_gain = comp_gain + crush_gain * crush_blend;

            // Lookahead limiter replaces crude tanh soft_clip — preserves transients
            let (out_l, out_r) = self.limiter.tick_stereo(
                mixed_l * parallel_gain,
                mixed_r * parallel_gain,
            );

            frame[0] = out_l;
            if frame.len() > 1 {
                frame[1] = out_r;
            }

            // Feed display buffer (mono mix for waveform display)
            let display_sample = (out_l + out_r) * 0.5;
            self.display_buf.push_sample(display_sample);
            let abs = display_sample.abs();
            if abs > self.peak_tracker {
                self.peak_tracker = abs;
            }
        }

        // Update peak for VU meter (once per buffer, then decay)
        self.display_buf.set_peak(self.peak_tracker);
        // Buffer-independent exponential decay (~60ms time constant)
        let buffer_frames = buffer.len() as f64 / 2.0;
        let decay = (-buffer_frames / (0.06 * self.sample_rate)).exp() as f32;
        self.peak_tracker *= decay;
    }
}
