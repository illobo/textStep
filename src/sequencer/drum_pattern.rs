//! Drum pattern data: 8 tracks x 32 steps with per-track synthesis parameters.

pub const NUM_DRUM_TRACKS: usize = 8;
pub const MAX_STEPS: usize = 32;

/// Identifies one of the 8 drum tracks (Kick, Snare, CHH, OHH, Ride, Clap, Cowbell, Tom).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrumTrackId {
    Kick = 0,
    Snare = 1,
    ClosedHiHat = 2,
    OpenHiHat = 3,
    Ride = 4,
    Clap = 5,
    Cowbell = 6,
    Tom = 7,
}

impl DrumTrackId {
    pub fn name(&self) -> &str {
        match self {
            DrumTrackId::Kick => "Kick",
            DrumTrackId::Snare => "Snare",
            DrumTrackId::ClosedHiHat => "CHH",
            DrumTrackId::OpenHiHat => "OHH",
            DrumTrackId::Ride => "Ride",
            DrumTrackId::Clap => "Clap",
            DrumTrackId::Cowbell => "Cowbell",
            DrumTrackId::Tom => "Tom",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DrumTrackParams {
    // Synthesis
    pub tune: f32,    // 0.0..=1.0  Pitch / frequency center
    pub sweep: f32,   // 0.0..=1.0  Pitch envelope depth
    pub color: f32,   // 0.0..=1.0  Timbre: noise/tone balance, waveform shape
    pub snap: f32,    // 0.0..=1.0  Transient click/attack character

    // Filter / Shape
    pub filter: f32,  // 0.0..=1.0  Filter cutoff frequency
    pub drive: f32,   // 0.0..=1.0  Saturation / overdrive

    // Amplitude
    pub decay: f32,   // 0.0..=1.0  Amplitude envelope decay time
    pub volume: f32,  // 0.0..=1.0  Track output level

    // Send effects (FX page)
    pub send_reverb: f32, // 0.0..=1.0  Send level to reverb
    pub send_delay: f32,  // 0.0..=1.0  Send level to delay
    pub pan: f32,         // 0.0..=1.0  Stereo pan (0.0=L, 0.5=center, 1.0=R)

    // Runtime state (not saved in kits)
    pub mute: bool,
    pub solo: bool,
}

impl DrumTrackParams {
    /// Per-voice defaults tuned to sound good out of the box.
    pub fn defaults_for(track: DrumTrackId) -> Self {
        match track {
            DrumTrackId::Kick => Self {
                tune: 0.3, sweep: 0.6, color: 0.2, snap: 0.45,
                filter: 0.7, drive: 0.20, decay: 0.45, volume: 0.75,
                send_reverb: 0.05, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
            DrumTrackId::Snare => Self {
                tune: 0.4, sweep: 0.1, color: 0.5, snap: 0.4,
                filter: 0.5, drive: 0.1, decay: 0.4, volume: 0.75,
                send_reverb: 0.15, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
            DrumTrackId::ClosedHiHat => Self {
                tune: 0.6, sweep: 0.0, color: 0.5, snap: 0.25,
                filter: 0.6, drive: 0.0, decay: 0.1, volume: 0.65,
                send_reverb: 0.05, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
            DrumTrackId::OpenHiHat => Self {
                tune: 0.5, sweep: 0.6, color: 0.3, snap: 0.25,
                filter: 0.5, drive: 0.0, decay: 0.5, volume: 0.65,
                send_reverb: 0.1, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
            DrumTrackId::Ride => Self {
                tune: 0.5, sweep: 0.0, color: 0.5, snap: 0.1,
                filter: 0.4, drive: 0.0, decay: 0.7, volume: 0.6,
                send_reverb: 0.1, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
            DrumTrackId::Clap => Self {
                tune: 0.5, sweep: 0.3, color: 0.5, snap: 0.5,
                filter: 0.5, drive: 0.1, decay: 0.4, volume: 0.7,
                send_reverb: 0.2, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
            DrumTrackId::Cowbell => Self {
                tune: 0.5, sweep: 0.3, color: 0.5, snap: 0.2,
                filter: 0.5, drive: 0.1, decay: 0.4, volume: 0.7,
                send_reverb: 0.1, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
            DrumTrackId::Tom => Self {
                tune: 0.5, sweep: 0.5, color: 0.1, snap: 0.3,
                filter: 0.8, drive: 0.1, decay: 0.5, volume: 0.8,
                send_reverb: 0.1, send_delay: 0.0, pan: 0.5, mute: false, solo: false,
            },
        }
    }
}

impl Default for DrumTrackParams {
    fn default() -> Self {
        Self {
            tune: 0.5,
            sweep: 0.3,
            color: 0.5,
            snap: 0.3,
            filter: 0.5,
            drive: 0.0,
            decay: 0.5,
            volume: 0.8,
            send_reverb: 0.0,
            send_delay: 0.0,
            pan: 0.5,
            mute: false,
            solo: false,
        }
    }
}

pub const TRACK_IDS: [DrumTrackId; NUM_DRUM_TRACKS] = [
    DrumTrackId::Kick,
    DrumTrackId::Snare,
    DrumTrackId::ClosedHiHat,
    DrumTrackId::OpenHiHat,
    DrumTrackId::Ride,
    DrumTrackId::Clap,
    DrumTrackId::Cowbell,
    DrumTrackId::Tom,
];

/// A drum pattern: 8 tracks of boolean step data plus per-track synthesis parameters.
#[derive(Clone, Copy, Debug)]
pub struct DrumPattern {
    pub steps: [[bool; MAX_STEPS]; NUM_DRUM_TRACKS],
    pub params: [DrumTrackParams; NUM_DRUM_TRACKS],
}

impl Default for DrumPattern {
    fn default() -> Self {
        let mut params = [DrumTrackParams::default(); NUM_DRUM_TRACKS];
        for (i, id) in TRACK_IDS.iter().enumerate() {
            params[i] = DrumTrackParams::defaults_for(*id);
        }
        Self {
            steps: [[false; MAX_STEPS]; NUM_DRUM_TRACKS],
            params,
        }
    }
}
