//! Hand-crafted drum sound presets organized by category (808, 909, Acoustic, Lo-Fi, etc.).

use crate::sequencer::drum_pattern::DrumTrackId;
use crate::sequencer::project::DrumSoundParams;
use super::DrumSoundPreset;

// Helper to construct DrumSoundParams concisely
const fn ds(tune: f32, sweep: f32, color: f32, snap: f32, filter: f32, drive: f32, decay: f32, volume: f32) -> DrumSoundParams {
    DrumSoundParams { tune, sweep, color, snap, filter, drive, decay, volume, send_reverb: 0.0, send_delay: 0.0, pan: 0.5 }
}

// ── Kick Presets ─────────────────────────────────────────────────────────────

pub static KICK_PRESETS: &[DrumSoundPreset] = &[
    // 808 — sub-osc adds low-end, so reduce volume/decay vs original
    DrumSoundPreset { name: "Deep 808",      category: "808",        voice: DrumTrackId::Kick, params: ds(0.20, 0.65, 0.15, 0.10, 0.55, 0.10, 0.70, 0.78) },
    DrumSoundPreset { name: "Punchy 808",    category: "808",        voice: DrumTrackId::Kick, params: ds(0.30, 0.55, 0.20, 0.50, 0.70, 0.15, 0.45, 0.75) },
    DrumSoundPreset { name: "Sub 808",       category: "808",        voice: DrumTrackId::Kick, params: ds(0.15, 0.70, 0.10, 0.05, 0.35, 0.05, 0.80, 0.80) },
    DrumSoundPreset { name: "Short 808",     category: "808",        voice: DrumTrackId::Kick, params: ds(0.25, 0.50, 0.20, 0.40, 0.60, 0.15, 0.28, 0.75) },
    // 909 — per-track saturation adds punch, ease drive
    DrumSoundPreset { name: "Hard 909",      category: "909",        voice: DrumTrackId::Kick, params: ds(0.35, 0.45, 0.30, 0.60, 0.80, 0.25, 0.40, 0.80) },
    DrumSoundPreset { name: "Soft 909",      category: "909",        voice: DrumTrackId::Kick, params: ds(0.30, 0.40, 0.25, 0.30, 0.60, 0.08, 0.45, 0.72) },
    DrumSoundPreset { name: "Boom 909",      category: "909",        voice: DrumTrackId::Kick, params: ds(0.25, 0.60, 0.20, 0.45, 0.55, 0.15, 0.55, 0.75) },
    // Acoustic — tighter decay, sub adds natural weight
    DrumSoundPreset { name: "Tight Acoustic", category: "Acoustic",  voice: DrumTrackId::Kick, params: ds(0.40, 0.30, 0.35, 0.70, 0.90, 0.12, 0.30, 0.75) },
    DrumSoundPreset { name: "Jazz Kick",     category: "Acoustic",   voice: DrumTrackId::Kick, params: ds(0.45, 0.20, 0.40, 0.50, 0.70, 0.05, 0.35, 0.68) },
    // Lo-Fi
    DrumSoundPreset { name: "Dusty Kick",    category: "Lo-Fi",      voice: DrumTrackId::Kick, params: ds(0.30, 0.50, 0.45, 0.35, 0.45, 0.35, 0.50, 0.70) },
    DrumSoundPreset { name: "Tape Kick",     category: "Lo-Fi",      voice: DrumTrackId::Kick, params: ds(0.25, 0.55, 0.50, 0.20, 0.40, 0.45, 0.50, 0.68) },
    // Industrial — saturation stacks with per-track, ease drive
    DrumSoundPreset { name: "Distorted Kick", category: "Industrial", voice: DrumTrackId::Kick, params: ds(0.30, 0.65, 0.60, 0.80, 0.90, 0.70, 0.35, 0.80) },
    DrumSoundPreset { name: "Metal Kick",    category: "Industrial",  voice: DrumTrackId::Kick, params: ds(0.40, 0.75, 0.70, 0.90, 1.00, 0.60, 0.30, 0.75) },
    // Minimal — short decay tames sub tail
    DrumSoundPreset { name: "Click Kick",    category: "Minimal",    voice: DrumTrackId::Kick, params: ds(0.35, 0.30, 0.10, 0.80, 0.70, 0.05, 0.18, 0.72) },
    DrumSoundPreset { name: "Micro Kick",    category: "Minimal",    voice: DrumTrackId::Kick, params: ds(0.40, 0.20, 0.15, 0.60, 0.50, 0.00, 0.12, 0.68) },
];

// ── Snare Presets ────────────────────────────────────────────────────────────

pub static SNARE_PRESETS: &[DrumSoundPreset] = &[
    // 808 — comb filter adds body, reduce color to tame resonance
    DrumSoundPreset { name: "Classic 808",   category: "808",        voice: DrumTrackId::Snare, params: ds(0.35, 0.15, 0.40, 0.40, 0.50, 0.10, 0.40, 0.78) },
    DrumSoundPreset { name: "Rimshot 808",   category: "808",        voice: DrumTrackId::Snare, params: ds(0.50, 0.05, 0.25, 0.70, 0.70, 0.15, 0.25, 0.78) },
    DrumSoundPreset { name: "Noisy 808",     category: "808",        voice: DrumTrackId::Snare, params: ds(0.30, 0.20, 0.55, 0.30, 0.40, 0.20, 0.50, 0.72) },
    // 909
    DrumSoundPreset { name: "Crack 909",     category: "909",        voice: DrumTrackId::Snare, params: ds(0.45, 0.10, 0.50, 0.60, 0.65, 0.18, 0.35, 0.80) },
    DrumSoundPreset { name: "Fat 909",       category: "909",        voice: DrumTrackId::Snare, params: ds(0.40, 0.15, 0.45, 0.45, 0.55, 0.20, 0.45, 0.78) },
    // Acoustic — comb gives natural shell resonance
    DrumSoundPreset { name: "Tight Snare",   category: "Acoustic",   voice: DrumTrackId::Snare, params: ds(0.50, 0.05, 0.38, 0.65, 0.75, 0.10, 0.30, 0.78) },
    DrumSoundPreset { name: "Brush Snare",   category: "Acoustic",   voice: DrumTrackId::Snare, params: ds(0.45, 0.00, 0.55, 0.20, 0.50, 0.00, 0.35, 0.62) },
    // Lo-Fi
    DrumSoundPreset { name: "Crunchy Snare", category: "Lo-Fi",      voice: DrumTrackId::Snare, params: ds(0.40, 0.10, 0.52, 0.50, 0.45, 0.45, 0.40, 0.72) },
    DrumSoundPreset { name: "Vinyl Snare",   category: "Lo-Fi",      voice: DrumTrackId::Snare, params: ds(0.35, 0.15, 0.48, 0.35, 0.40, 0.30, 0.45, 0.68) },
    // Industrial — keep high color for aggressive resonance
    DrumSoundPreset { name: "Noise Blast",   category: "Industrial", voice: DrumTrackId::Snare, params: ds(0.30, 0.20, 0.75, 0.80, 0.80, 0.65, 0.30, 0.80) },
    // Minimal — low color = minimal comb effect
    DrumSoundPreset { name: "Click Snare",   category: "Minimal",    voice: DrumTrackId::Snare, params: ds(0.55, 0.00, 0.25, 0.80, 0.80, 0.05, 0.15, 0.72) },
    DrumSoundPreset { name: "Ghost Snare",   category: "Minimal",    voice: DrumTrackId::Snare, params: ds(0.40, 0.05, 0.40, 0.15, 0.45, 0.00, 0.20, 0.48) },
];

// ── Closed Hi-Hat Presets ────────────────────────────────────────────────────

pub static CHH_PRESETS: &[DrumSoundPreset] = &[
    // Transient burst + sizzle add attack and brightness — reduce snap/volume accordingly
    DrumSoundPreset { name: "Tight 808",     category: "808",        voice: DrumTrackId::ClosedHiHat, params: ds(0.60, 0.00, 0.50, 0.30, 0.65, 0.00, 0.08, 0.65) },
    DrumSoundPreset { name: "Sizzle 909",    category: "909",        voice: DrumTrackId::ClosedHiHat, params: ds(0.55, 0.00, 0.55, 0.28, 0.70, 0.10, 0.12, 0.65) },
    DrumSoundPreset { name: "Crisp Hat",     category: "Acoustic",   voice: DrumTrackId::ClosedHiHat, params: ds(0.70, 0.00, 0.40, 0.40, 0.80, 0.05, 0.06, 0.60) },
    DrumSoundPreset { name: "Dark Hat",      category: "Lo-Fi",      voice: DrumTrackId::ClosedHiHat, params: ds(0.45, 0.00, 0.60, 0.20, 0.40, 0.30, 0.10, 0.62) },
    DrumSoundPreset { name: "Gritty Hat",    category: "Industrial", voice: DrumTrackId::ClosedHiHat, params: ds(0.50, 0.10, 0.70, 0.45, 0.55, 0.45, 0.08, 0.65) },
    DrumSoundPreset { name: "Thin Hat",      category: "Minimal",    voice: DrumTrackId::ClosedHiHat, params: ds(0.75, 0.00, 0.30, 0.15, 0.90, 0.00, 0.05, 0.52) },
    DrumSoundPreset { name: "Shaker",        category: "Acoustic",   voice: DrumTrackId::ClosedHiHat, params: ds(0.65, 0.00, 0.80, 0.10, 0.70, 0.00, 0.04, 0.55) },
    DrumSoundPreset { name: "Noisy Click",   category: "Lo-Fi",      voice: DrumTrackId::ClosedHiHat, params: ds(0.50, 0.05, 0.75, 0.40, 0.35, 0.35, 0.06, 0.60) },
];

// ── Open Hi-Hat Presets ──────────────────────────────────────────────────────

pub static OHH_PRESETS: &[DrumSoundPreset] = &[
    // Transient burst + sizzle add brightness — reduce snap/volume accordingly
    DrumSoundPreset { name: "Classic 808",   category: "808",        voice: DrumTrackId::OpenHiHat, params: ds(0.50, 0.60, 0.30, 0.25, 0.50, 0.00, 0.50, 0.65) },
    DrumSoundPreset { name: "Sizzle 909",    category: "909",        voice: DrumTrackId::OpenHiHat, params: ds(0.55, 0.50, 0.40, 0.28, 0.60, 0.10, 0.55, 0.65) },
    DrumSoundPreset { name: "Washy",         category: "Acoustic",   voice: DrumTrackId::OpenHiHat, params: ds(0.45, 0.70, 0.35, 0.15, 0.45, 0.00, 0.70, 0.60) },
    DrumSoundPreset { name: "Trash Open",    category: "Industrial", voice: DrumTrackId::OpenHiHat, params: ds(0.40, 0.80, 0.60, 0.40, 0.70, 0.45, 0.45, 0.65) },
    DrumSoundPreset { name: "Short Open",    category: "Minimal",    voice: DrumTrackId::OpenHiHat, params: ds(0.55, 0.40, 0.30, 0.20, 0.55, 0.00, 0.30, 0.55) },
    DrumSoundPreset { name: "Lo-Fi Open",    category: "Lo-Fi",      voice: DrumTrackId::OpenHiHat, params: ds(0.45, 0.55, 0.50, 0.15, 0.35, 0.30, 0.55, 0.60) },
];

// ── Ride Presets ─────────────────────────────────────────────────────────────

pub static RIDE_PRESETS: &[DrumSoundPreset] = &[
    DrumSoundPreset { name: "Bell Ride",     category: "Acoustic",   voice: DrumTrackId::Ride, params: ds(0.60, 0.00, 0.40, 0.20, 0.50, 0.00, 0.75, 0.60) },
    DrumSoundPreset { name: "Ping Ride",     category: "Acoustic",   voice: DrumTrackId::Ride, params: ds(0.70, 0.00, 0.30, 0.40, 0.65, 0.00, 0.60, 0.55) },
    DrumSoundPreset { name: "Dark Ride",     category: "Lo-Fi",      voice: DrumTrackId::Ride, params: ds(0.40, 0.00, 0.55, 0.10, 0.35, 0.20, 0.80, 0.55) },
    DrumSoundPreset { name: "Crash",         category: "Acoustic",   voice: DrumTrackId::Ride, params: ds(0.45, 0.10, 0.60, 0.30, 0.55, 0.10, 0.90, 0.65) },
    DrumSoundPreset { name: "Metal Ride",    category: "Industrial", voice: DrumTrackId::Ride, params: ds(0.55, 0.05, 0.70, 0.50, 0.75, 0.40, 0.65, 0.60) },
    DrumSoundPreset { name: "Thin Ride",     category: "Minimal",    voice: DrumTrackId::Ride, params: ds(0.65, 0.00, 0.25, 0.15, 0.50, 0.00, 0.50, 0.50) },
];

// ── Clap Presets ─────────────────────────────────────────────────────────────

pub static CLAP_PRESETS: &[DrumSoundPreset] = &[
    DrumSoundPreset { name: "Classic 808",   category: "808",        voice: DrumTrackId::Clap, params: ds(0.50, 0.30, 0.50, 0.50, 0.50, 0.10, 0.40, 0.70) },
    DrumSoundPreset { name: "Tight 909",     category: "909",        voice: DrumTrackId::Clap, params: ds(0.55, 0.25, 0.55, 0.60, 0.60, 0.15, 0.35, 0.75) },
    DrumSoundPreset { name: "Big Clap",      category: "808",        voice: DrumTrackId::Clap, params: ds(0.45, 0.40, 0.60, 0.40, 0.45, 0.20, 0.55, 0.75) },
    DrumSoundPreset { name: "Room Clap",     category: "Acoustic",   voice: DrumTrackId::Clap, params: ds(0.50, 0.35, 0.45, 0.55, 0.55, 0.05, 0.50, 0.70) },
    DrumSoundPreset { name: "Crushed Clap",  category: "Industrial", voice: DrumTrackId::Clap, params: ds(0.45, 0.30, 0.70, 0.70, 0.70, 0.60, 0.35, 0.75) },
    DrumSoundPreset { name: "Snap",          category: "Minimal",    voice: DrumTrackId::Clap, params: ds(0.60, 0.10, 0.30, 0.80, 0.80, 0.00, 0.10, 0.65) },
    DrumSoundPreset { name: "Finger Snap",   category: "Acoustic",   voice: DrumTrackId::Clap, params: ds(0.65, 0.05, 0.25, 0.90, 0.85, 0.00, 0.08, 0.60) },
];

// ── Cowbell Presets ──────────────────────────────────────────────────────────

pub static COWBELL_PRESETS: &[DrumSoundPreset] = &[
    DrumSoundPreset { name: "Classic 808",   category: "808",        voice: DrumTrackId::Cowbell, params: ds(0.50, 0.30, 0.50, 0.20, 0.50, 0.10, 0.40, 0.70) },
    DrumSoundPreset { name: "High Bell",     category: "Acoustic",   voice: DrumTrackId::Cowbell, params: ds(0.70, 0.20, 0.40, 0.30, 0.60, 0.05, 0.45, 0.65) },
    DrumSoundPreset { name: "Low Bell",      category: "Acoustic",   voice: DrumTrackId::Cowbell, params: ds(0.30, 0.40, 0.55, 0.15, 0.40, 0.10, 0.50, 0.65) },
    DrumSoundPreset { name: "Agogo",         category: "Acoustic",   voice: DrumTrackId::Cowbell, params: ds(0.60, 0.15, 0.35, 0.40, 0.70, 0.00, 0.35, 0.60) },
    DrumSoundPreset { name: "Clanky",        category: "Industrial", voice: DrumTrackId::Cowbell, params: ds(0.55, 0.35, 0.65, 0.50, 0.55, 0.40, 0.30, 0.70) },
    DrumSoundPreset { name: "Muted Bell",    category: "Minimal",    voice: DrumTrackId::Cowbell, params: ds(0.50, 0.10, 0.30, 0.25, 0.45, 0.00, 0.20, 0.55) },
];

// ── Tom Presets ──────────────────────────────────────────────────────────────

pub static TOM_PRESETS: &[DrumSoundPreset] = &[
    DrumSoundPreset { name: "Deep 808 Tom",  category: "808",        voice: DrumTrackId::Tom, params: ds(0.30, 0.60, 0.10, 0.30, 0.70, 0.10, 0.55, 0.80) },
    DrumSoundPreset { name: "High 808 Tom",  category: "808",        voice: DrumTrackId::Tom, params: ds(0.60, 0.50, 0.10, 0.35, 0.75, 0.10, 0.45, 0.75) },
    DrumSoundPreset { name: "Floor Tom",     category: "Acoustic",   voice: DrumTrackId::Tom, params: ds(0.35, 0.45, 0.15, 0.40, 0.80, 0.05, 0.50, 0.80) },
    DrumSoundPreset { name: "Rack Tom",      category: "Acoustic",   voice: DrumTrackId::Tom, params: ds(0.55, 0.40, 0.15, 0.45, 0.85, 0.05, 0.40, 0.75) },
    DrumSoundPreset { name: "Roto Tom",      category: "909",        voice: DrumTrackId::Tom, params: ds(0.50, 0.70, 0.20, 0.50, 0.70, 0.20, 0.45, 0.75) },
    DrumSoundPreset { name: "Dirty Tom",     category: "Lo-Fi",      voice: DrumTrackId::Tom, params: ds(0.40, 0.55, 0.30, 0.25, 0.50, 0.40, 0.55, 0.70) },
    DrumSoundPreset { name: "Pipe Tom",      category: "Industrial", voice: DrumTrackId::Tom, params: ds(0.45, 0.80, 0.50, 0.60, 0.60, 0.60, 0.40, 0.75) },
    DrumSoundPreset { name: "Bleep",         category: "Minimal",    voice: DrumTrackId::Tom, params: ds(0.65, 0.20, 0.05, 0.50, 0.60, 0.00, 0.25, 0.60) },
];

// ── Lookup functions ─────────────────────────────────────────────────────────

pub fn presets_for_voice(voice: DrumTrackId) -> &'static [DrumSoundPreset] {
    match voice {
        DrumTrackId::Kick => KICK_PRESETS,
        DrumTrackId::Snare => SNARE_PRESETS,
        DrumTrackId::ClosedHiHat => CHH_PRESETS,
        DrumTrackId::OpenHiHat => OHH_PRESETS,
        DrumTrackId::Ride => RIDE_PRESETS,
        DrumTrackId::Clap => CLAP_PRESETS,
        DrumTrackId::Cowbell => COWBELL_PRESETS,
        DrumTrackId::Tom => TOM_PRESETS,
    }
}

pub fn categories_for_voice(voice: DrumTrackId) -> Vec<&'static str> {
    let presets = presets_for_voice(voice);
    let mut cats: Vec<&'static str> = Vec::new();
    for p in presets {
        if !cats.contains(&p.category) {
            cats.push(p.category);
        }
    }
    cats
}
