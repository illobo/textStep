//! Drum pattern presets: genre-specific step patterns (House, Techno, DnB, etc.).

use crate::sequencer::drum_pattern::NUM_DRUM_TRACKS;

pub struct PatternPreset {
    pub name: &'static str,
    pub genre: &'static str,
    /// Hex-encoded steps per track (8 hex chars = 32 steps each)
    pub steps: [&'static str; NUM_DRUM_TRACKS],
}

// Track order: Kick, Snare, CHH, OHH, Ride, Clap, Cowbell, Tom

pub static PATTERN_PRESETS: &[PatternPreset] = &[
    // ── Daft Punk ────────────────────────────────────────────────────────
    // Signature disco-house-electro patterns inspired by Daft Punk productions
    PatternPreset { name: "Around the World", genre: "Daft Punk",
        steps: ["88880000", "08080000", "a0a00000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Da Funk", genre: "Daft Punk",
        steps: ["c8880000", "08080000", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Revolution 909", genre: "Daft Punk",
        steps: ["88880000", "00000000", "ffff0000", "00020000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "One More Time", genre: "Daft Punk",
        steps: ["88880000", "00000000", "aaaa0000", "01010000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Robot Rock", genre: "Daft Punk",
        steps: ["c8c80000", "08080000", "00000000", "00000000", "aaaa0000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Get Lucky", genre: "Daft Punk",
        steps: ["82820000", "08080000", "aaaa0000", "04040000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Harder Better", genre: "Daft Punk",
        steps: ["88880000", "00000000", "ffff0000", "00000000", "00000000", "08080000", "22220000", "00000000"] },
    PatternPreset { name: "Giorgio", genre: "Daft Punk",
        steps: ["88880000", "08080000", "ffff0000", "22220000", "00000000", "00000000", "00000000", "00000000"] },

    // ── Basics ──────────────────────────────────────────────────────────
    // Progressive building blocks: offbeat hats (3,7,11,15) for proper techno feel
    PatternPreset { name: "Kick Only", genre: "Basics",
        steps: ["88880000", "00000000", "00000000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Kick + Hat", genre: "Basics",
        steps: ["88880000", "00000000", "22220000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Kick + Snare", genre: "Basics",
        steps: ["88880000", "08080000", "00000000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Kick + Clap", genre: "Basics",
        steps: ["88880000", "00000000", "00000000", "00000000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "K + S + HH", genre: "Basics",
        steps: ["88880000", "08080000", "22220000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "K + Clap + HH", genre: "Basics",
        steps: ["88880000", "00000000", "22220000", "00000000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "K + S + HH + OH", genre: "Basics",
        steps: ["88880000", "08080000", "22220000", "04040000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Full Kit", genre: "Basics",
        steps: ["88880000", "08080000", "22220000", "04040000", "00000000", "08080000", "00000000", "00000000"] },

    // ── Techno II ──────────────────────────────────────────────────────
    // Clean, functional patterns — offbeat hats for proper techno
    PatternPreset { name: "Offbeat Hats", genre: "Techno II",
        steps: ["88880000", "00000000", "00000000", "22220000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Ride Driver", genre: "Techno II",
        steps: ["88880000", "00000000", "00000000", "00000000", "22220000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Half Time", genre: "Techno II",
        steps: ["80800000", "00800000", "22220000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Double Kick", genre: "Techno II",
        steps: ["c8c80000", "08080000", "22220000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Syncopated K", genre: "Techno II",
        steps: ["a0a00000", "08080000", "22220000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Offbeat Kick", genre: "Techno II",
        steps: ["82820000", "08080000", "22220000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "808 Cowbell", genre: "Techno II",
        steps: ["88880000", "00000000", "22220000", "00000000", "00000000", "00000000", "22220000", "00000000"] },
    PatternPreset { name: "Tom Groove", genre: "Techno II",
        steps: ["88880000", "00000000", "22220000", "00000000", "00000000", "08080000", "00000000", "22220000"] },
    PatternPreset { name: "Stripped 16ths", genre: "Techno II",
        steps: ["88880000", "00000000", "ffff0000", "00000000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Perc Stack", genre: "Techno II",
        steps: ["88880000", "00000000", "22220000", "04040000", "00000000", "08080000", "20200000", "00200020"] },

    // ── Techno ───────────────────────────────────────────────────────────
    PatternPreset { name: "Four on the Floor", genre: "Techno",
        steps: ["88880000", "00000000", "aaaa0000", "00000000", "22220000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Driving Techno", genre: "Techno",
        steps: ["88880000", "00000000", "eeee0000", "11110000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Hard Techno", genre: "Techno",
        steps: ["8a8a0000", "00800080", "aaaa0000", "00000000", "00000000", "04040000", "00000000", "00200000"] },
    PatternPreset { name: "Minimal Techno", genre: "Techno",
        steps: ["80880000", "00000000", "a2a20000", "04040000", "00000000", "00800000", "00000000", "00000000"] },
    PatternPreset { name: "Industrial Beat", genre: "Techno",
        steps: ["88880000", "20200000", "aaaa0000", "00000000", "00000000", "08080000", "22000000", "00020000"] },
    PatternPreset { name: "Warehouse", genre: "Techno",
        steps: ["88880000", "00000000", "ffff0000", "00000000", "55550000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Pounding", genre: "Techno",
        steps: ["aaaa0000", "00000000", "55550000", "00000000", "08080000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Hypnotic", genre: "Techno",
        steps: ["88840000", "00000000", "aaaa0000", "11110000", "00000000", "00800080", "00000000", "00000000"] },
    PatternPreset { name: "Tool Room", genre: "Techno",
        steps: ["88880000", "08080000", "aaaa0000", "00000000", "00000000", "00000000", "22220000", "00000000"] },
    PatternPreset { name: "Berghain", genre: "Techno",
        steps: ["88880000", "00080000", "a2a20000", "00000000", "00000000", "08000000", "00000000", "00200020"] },

    // ── House ────────────────────────────────────────────────────────────
    PatternPreset { name: "Classic House", genre: "House",
        steps: ["88880000", "00000000", "0a0a0000", "00000000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Deep House", genre: "House",
        steps: ["80880000", "00000000", "0a0a0000", "04040000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Funky House", genre: "House",
        steps: ["a08a0000", "00000000", "aaaa0000", "00000000", "22220000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Chicago Jack", genre: "House",
        steps: ["88880000", "08080000", "aaaa0000", "11110000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Disco House", genre: "House",
        steps: ["88880000", "00000000", "eeee0000", "11110000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Acid House", genre: "House",
        steps: ["88880000", "00000000", "aaaa0000", "00000000", "00000000", "08080000", "22220000", "00000000"] },
    PatternPreset { name: "Soulful", genre: "House",
        steps: ["80840000", "08080000", "a2a20000", "04040000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Garage Bump", genre: "House",
        steps: ["a0880000", "00000000", "0a0a0000", "04000000", "00000000", "08080000", "00000000", "02000000"] },

    // ── Drum & Bass ──────────────────────────────────────────────────────
    PatternPreset { name: "Amen Break", genre: "Drum & Bass",
        steps: ["80200080", "04080408", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Two-Step DnB", genre: "Drum & Bass",
        steps: ["80008000", "00800080", "aaaa0000", "00010000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Liquid Funk", genre: "Drum & Bass",
        steps: ["80200000", "00800080", "a2a20000", "04040000", "00000000", "00000000", "00000000", "00200000"] },
    PatternPreset { name: "Neurofunk", genre: "Drum & Bass",
        steps: ["82008200", "04080408", "a0a00000", "00000000", "00000000", "00000000", "00000000", "00020000"] },
    PatternPreset { name: "Jump Up", genre: "Drum & Bass",
        steps: ["80008020", "00800080", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Choppy Breaks", genre: "Drum & Bass",
        steps: ["80420040", "04080008", "a2a20000", "01010000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Roller", genre: "Drum & Bass",
        steps: ["80200820", "04080408", "eeee0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Halftime", genre: "Drum & Bass",
        steps: ["80000000", "00800000", "a2a20000", "00000000", "00000000", "00000000", "00000000", "00200000"] },

    // ── Trance ───────────────────────────────────────────────────────────
    PatternPreset { name: "Classic Trance", genre: "Trance",
        steps: ["88880000", "08080000", "aaaa0000", "00010000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Uplifting", genre: "Trance",
        steps: ["88880000", "00000000", "eeee0000", "11110000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Psytrance", genre: "Trance",
        steps: ["88880000", "00000000", "aaaa0000", "55550000", "00000000", "08080000", "00000000", "00000000"] },
    PatternPreset { name: "Goa", genre: "Trance",
        steps: ["88880000", "08000800", "aaaa0000", "00000000", "55550000", "00080000", "00000000", "00000000"] },
    PatternPreset { name: "Progressive", genre: "Trance",
        steps: ["80880000", "08080000", "a2a20000", "00000000", "00000000", "00000000", "00000000", "00200000"] },
    PatternPreset { name: "Tech Trance", genre: "Trance",
        steps: ["88880000", "00000000", "aaaa0000", "00000000", "22220000", "08080000", "00000000", "00020000"] },
    PatternPreset { name: "Hard Trance", genre: "Trance",
        steps: ["aaaa0000", "08080000", "55550000", "00000000", "00000000", "00000000", "22220000", "00000000"] },
    PatternPreset { name: "Euphoric Build", genre: "Trance",
        steps: ["88880000", "08080000", "ffff0000", "00000000", "00000000", "08080000", "00000000", "00000000"] },

    // ── Trap ─────────────────────────────────────────────────────────────
    PatternPreset { name: "Atlanta Trap", genre: "Trap",
        steps: ["80000080", "00080000", "aa6a0000", "00000000", "00000000", "08000000", "00000000", "00000000"] },
    PatternPreset { name: "Hard Trap", genre: "Trap",
        steps: ["80200000", "00080000", "aaaa0000", "00010000", "00000000", "08000000", "00000000", "00000000"] },
    PatternPreset { name: "Festival Trap", genre: "Trap",
        steps: ["80000820", "00080008", "aaae0000", "00000000", "00000000", "08000000", "00000000", "00020000"] },
    PatternPreset { name: "Lo-Fi Trap", genre: "Trap",
        steps: ["80000080", "00080000", "a0a00000", "02020000", "00000000", "08000000", "00000000", "00000000"] },
    PatternPreset { name: "Drill", genre: "Trap",
        steps: ["82008000", "00080408", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Sparse 808", genre: "Trap",
        steps: ["80000000", "00080000", "a0a00000", "00000000", "00000000", "08000000", "00000000", "00000000"] },
    PatternPreset { name: "Triple Hi-Hat", genre: "Trap",
        steps: ["80000080", "00080000", "eeee0000", "00010000", "00000000", "08000000", "00000000", "00000000"] },
    PatternPreset { name: "Bouncy", genre: "Trap",
        steps: ["a0080020", "00080000", "aaaa0000", "00000000", "00000000", "08000000", "00000000", "00020000"] },

    // ── Breakbeat ────────────────────────────────────────────────────────
    PatternPreset { name: "Classic Break", genre: "Breakbeat",
        steps: ["80200820", "04080008", "aaaa0000", "00010000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Nu Breaks", genre: "Breakbeat",
        steps: ["80080020", "04080408", "a2a20000", "00000000", "00000000", "00000000", "00000000", "00020000"] },
    PatternPreset { name: "Big Beat", genre: "Breakbeat",
        steps: ["88840000", "08080000", "aaaa0000", "00010000", "00000000", "00000000", "00000000", "00200000"] },
    PatternPreset { name: "Funky Breaks", genre: "Breakbeat",
        steps: ["80240000", "04080000", "aaaa0000", "00000000", "00000000", "00080000", "00000000", "00000000"] },
    PatternPreset { name: "Shuffle Break", genre: "Breakbeat",
        steps: ["80200820", "00080408", "a4a40000", "02020000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Progressive Break", genre: "Breakbeat",
        steps: ["88840000", "00080000", "a2a20000", "04040000", "00000000", "08000000", "00000000", "00000000"] },
    PatternPreset { name: "Breakstep", genre: "Breakbeat",
        steps: ["80008020", "04080008", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00020000"] },
    PatternPreset { name: "Amen Chop", genre: "Breakbeat",
        steps: ["80420840", "04080008", "a0a00000", "00000000", "00000000", "00000000", "00000000", "02000000"] },

    // ── Electro ──────────────────────────────────────────────────────────
    PatternPreset { name: "Electro Funk", genre: "Electro",
        steps: ["80080000", "08080000", "aaaa0000", "00000000", "00000000", "00000000", "22220000", "00000000"] },
    PatternPreset { name: "Robot Dance", genre: "Electro",
        steps: ["a0880000", "08080000", "55550000", "00000000", "00000000", "00000000", "aaaa0000", "00000000"] },
    PatternPreset { name: "Kraftwerk", genre: "Electro",
        steps: ["88880000", "08080000", "00000000", "00000000", "00000000", "00000000", "aaaa0000", "22220000"] },
    PatternPreset { name: "Miami Bass", genre: "Electro",
        steps: ["a08a0000", "08080000", "aaaa0000", "00010000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Synth Pop", genre: "Electro",
        steps: ["80880000", "08080000", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00200020"] },
    PatternPreset { name: "EBM", genre: "Electro",
        steps: ["aaaa0000", "08080000", "00000000", "00000000", "00000000", "00000000", "55550000", "00000000"] },
    PatternPreset { name: "Electro Clash", genre: "Electro",
        steps: ["88880000", "00080008", "a2a20000", "00000000", "00000000", "08000000", "00000000", "00000000"] },
    PatternPreset { name: "Detroit Electro", genre: "Electro",
        steps: ["80880000", "08080000", "aaaa0000", "00000000", "22220000", "00000000", "00000000", "00000000"] },

    // ── Ambient ──────────────────────────────────────────────────────────
    PatternPreset { name: "Sparse Pulse", genre: "Ambient",
        steps: ["80000000", "00000000", "00000000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Breathing", genre: "Ambient",
        steps: ["80000000", "00800000", "00000000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Shimmering", genre: "Ambient",
        steps: ["00000000", "00000000", "00000000", "00000000", "a2a20000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Soft Tick", genre: "Ambient",
        steps: ["00000000", "00000000", "80800000", "00000000", "00000000", "00000000", "20200000", "00000000"] },
    PatternPreset { name: "Deep Space", genre: "Ambient",
        steps: ["80000000", "00000000", "00000000", "00000000", "00000000", "00800000", "00000000", "00200000"] },
    PatternPreset { name: "Rain", genre: "Ambient",
        steps: ["00000000", "00000000", "a4a40000", "02020000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Windchime", genre: "Ambient",
        steps: ["00000000", "00000000", "00000000", "00000000", "80200000", "00000000", "00800000", "00000000"] },
    PatternPreset { name: "Glacial", genre: "Ambient",
        steps: ["80000000", "00000000", "00000000", "00000000", "00000000", "00000000", "00000000", "00800000"] },

    // ── Downtempo ────────────────────────────────────────────────────────
    PatternPreset { name: "Trip Hop", genre: "Downtempo",
        steps: ["80000000", "00800000", "a2a20000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Chillout", genre: "Downtempo",
        steps: ["80040000", "00800000", "a0a00000", "02020000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Lo-Fi Hip Hop", genre: "Downtempo",
        steps: ["80000820", "00800000", "a2a20000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Lazy Groove", genre: "Downtempo",
        steps: ["80040000", "08080000", "a0a20000", "00000000", "00000000", "00000000", "00000000", "00200000"] },
    PatternPreset { name: "Dub Chill", genre: "Downtempo",
        steps: ["80800000", "00080000", "a0a00000", "00000000", "00000000", "00000000", "20200000", "00000000"] },
    PatternPreset { name: "Mellow Beat", genre: "Downtempo",
        steps: ["80000000", "00800000", "00000000", "02020000", "a0a00000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Sunset", genre: "Downtempo",
        steps: ["80040000", "00800000", "a2a20000", "04000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Headnod", genre: "Downtempo",
        steps: ["a0080000", "00800000", "a2a20000", "00000000", "00000000", "00080000", "00000000", "00000000"] },

    // ── IDM ──────────────────────────────────────────────────────────────
    PatternPreset { name: "Glitchy Groove", genre: "IDM",
        steps: ["82040208", "04020801", "a4520000", "01000000", "00000000", "00000000", "00000000", "00200000"] },
    PatternPreset { name: "Wonky", genre: "IDM",
        steps: ["80200400", "02080010", "a0a20000", "00000000", "00000000", "04000000", "00000000", "00020000"] },
    PatternPreset { name: "Autechre", genre: "IDM",
        steps: ["80040200", "00800010", "a0240000", "02000000", "00000000", "00000000", "20000000", "00000000"] },
    PatternPreset { name: "Aphex Style", genre: "IDM",
        steps: ["82008200", "04020402", "a2a20000", "00010000", "00000000", "00000000", "00000000", "00200020"] },
    PatternPreset { name: "Broken Grid", genre: "IDM",
        steps: ["80400020", "04000208", "a0820000", "00000000", "00000000", "00080000", "00200000", "00000000"] },
    PatternPreset { name: "Polymetric", genre: "IDM",
        steps: ["88480000", "00080400", "a2a20000", "00000000", "00000000", "00000000", "48000000", "00000000"] },
    PatternPreset { name: "Drill n Bass", genre: "IDM",
        steps: ["80420840", "04280428", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Warp Records", genre: "IDM",
        steps: ["a0040200", "00820008", "a0a00000", "02020000", "00000000", "00000000", "00000000", "00200000"] },

    // ── Reggaeton ────────────────────────────────────────────────────────
    PatternPreset { name: "Dembow", genre: "Reggaeton",
        steps: ["80800000", "02820000", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Perreo", genre: "Reggaeton",
        steps: ["82820000", "00800080", "aaaa0000", "00000000", "00000000", "04040000", "00000000", "00000000"] },
    PatternPreset { name: "Reggaeton Pop", genre: "Reggaeton",
        steps: ["80800000", "02820000", "a2a20000", "04040000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Latin Bounce", genre: "Reggaeton",
        steps: ["a0a00000", "02820000", "55550000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Moombahton", genre: "Reggaeton",
        steps: ["80800000", "02820000", "aaaa0000", "00000000", "00000000", "00000000", "22220000", "00000000"] },
    PatternPreset { name: "Slow Dembow", genre: "Reggaeton",
        steps: ["80000080", "02820000", "a0a00000", "00000000", "00000000", "00000000", "00000000", "00200000"] },

    // ── Dub Techno ───────────────────────────────────────────────────────
    PatternPreset { name: "Basic Chain", genre: "Dub Techno",
        steps: ["88880000", "00000000", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Deep Dub", genre: "Dub Techno",
        steps: ["80880000", "00000000", "a2a20000", "04040000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Hazy", genre: "Dub Techno",
        steps: ["88880000", "00000000", "00000000", "00000000", "a2a20000", "00000000", "22220000", "00000000"] },
    PatternPreset { name: "Echoing", genre: "Dub Techno",
        steps: ["80800000", "00800000", "a0a00000", "00000000", "00000000", "00000000", "20200000", "00000000"] },
    PatternPreset { name: "Foggy", genre: "Dub Techno",
        steps: ["80880000", "00080000", "a0a00000", "02020000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Chord Stab", genre: "Dub Techno",
        steps: ["88880000", "00000000", "aaaa0000", "00000000", "00000000", "08000000", "00000000", "00000000"] },

    // ── Garage ───────────────────────────────────────────────────────────
    PatternPreset { name: "2-Step", genre: "Garage",
        steps: ["80080000", "08080000", "0a0a0000", "04000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "UK Garage", genre: "Garage",
        steps: ["80840000", "08080000", "aaaa0000", "00010000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Speed Garage", genre: "Garage",
        steps: ["a0880000", "08080000", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00200000"] },
    PatternPreset { name: "Bassline", genre: "Garage",
        steps: ["80880000", "08080000", "aaaa0000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Future Garage", genre: "Garage",
        steps: ["80040000", "08080000", "a2a20000", "04000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Shuffled", genre: "Garage",
        steps: ["80080020", "08080000", "a4a40000", "02020000", "00000000", "00000000", "00000000", "00000000"] },

    // ── Glitch ───────────────────────────────────────────────────────────
    PatternPreset { name: "Stuttered", genre: "Glitch",
        steps: ["e0200000", "00e00000", "a2a20000", "00000000", "00000000", "00000000", "00000000", "00000000"] },
    PatternPreset { name: "Scattered", genre: "Glitch",
        steps: ["82040102", "04010204", "a0820000", "00000000", "00000000", "00000000", "20010000", "00000000"] },
    PatternPreset { name: "Micro Edits", genre: "Glitch",
        steps: ["c0600000", "00c00000", "a2a20000", "00000000", "00000000", "00060000", "00000000", "00000000"] },
    PatternPreset { name: "Error Code", genre: "Glitch",
        steps: ["80020400", "04000208", "00000000", "00000000", "a0a00000", "00000000", "20400000", "00800000"] },
    PatternPreset { name: "Buffer Overflow", genre: "Glitch",
        steps: ["a0240802", "02080140", "a0240000", "00000000", "00000000", "00000000", "00000000", "00020000"] },
    PatternPreset { name: "Digital Rain", genre: "Glitch",
        steps: ["80400020", "00200040", "82420000", "00000000", "00000000", "04000000", "00000000", "20000000"] },
];

pub fn genres() -> Vec<&'static str> {
    let mut g: Vec<&'static str> = Vec::new();
    for p in PATTERN_PRESETS {
        if !g.contains(&p.genre) {
            g.push(p.genre);
        }
    }
    g
}

pub fn presets_for_genre(genre: &str) -> Vec<&'static PatternPreset> {
    PATTERN_PRESETS.iter().filter(|p| p.genre == genre).collect()
}

pub fn preset_by_name(name: &str) -> Option<&'static PatternPreset> {
    PATTERN_PRESETS.iter().find(|p| p.name == name)
}
