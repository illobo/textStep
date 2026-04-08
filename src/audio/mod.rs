pub mod clock;
pub mod display_buffer;
pub mod drum_voice;
pub mod fft;
pub mod effects;
pub mod engine;
pub mod mixer;
pub mod synth_voice;

use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender};

use crate::messages::{AudioToUi, UiToAudio};
use display_buffer::AudioDisplayBuffer;
use engine::AudioEngine;

/// Start the cpal audio output stream with an AudioEngine running inside the callback.
///
/// The caller must keep the returned `cpal::Stream` alive for audio to continue playing.
pub fn start_audio_stream(
    rx: Receiver<UiToAudio>,
    tx: Sender<AudioToUi>,
    display_buf: Arc<AudioDisplayBuffer>,
) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no default audio output device found")?;

    let config = device
        .default_output_config()
        .map_err(|e| format!("failed to get default output config: {e}"))?;

    let sample_rate = config.sample_rate() as f64;
    let channels = config.channels() as usize;

    let mut engine = AudioEngine::new(sample_rate, rx, tx, display_buf);

    // Pre-allocate fallback stereo buffer for non-stereo configs (3d)
    let mut fallback_buf: Vec<f32> = Vec::new();

    let stream_config: cpal::StreamConfig = config.into();

    let stream = device
        .build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if channels == 2 {
                    engine.process(data);
                } else {
                    let num_frames = data.len() / channels;
                    let stereo_len = num_frames * 2;
                    // Reuse pre-allocated buffer, growing only if needed
                    fallback_buf.resize(stereo_len, 0.0);
                    fallback_buf.fill(0.0);
                    engine.process(&mut fallback_buf);
                    for frame_idx in 0..num_frames {
                        let mono_sample = fallback_buf[frame_idx * 2];
                        for ch in 0..channels {
                            data[frame_idx * channels + ch] = mono_sample;
                        }
                    }
                }
            },
            |err| {
                log::error!("audio stream error: {}", err);
            },
            None,
        )
        .map_err(|e| format!("failed to build audio output stream: {e}"))?;

    stream.play().map_err(|e| format!("failed to start audio stream: {e}"))?;

    Ok(stream)
}
