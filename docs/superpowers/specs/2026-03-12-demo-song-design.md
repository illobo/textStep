# Demo Song & Per-Pattern BPM

**Date:** 2026-03-12
**Status:** Implemented

## Overview

Add per-pattern BPM to drum patterns and ship a 10-scene "demo song" as the factory default project. Each scene is a cohesive musical idea with matching drum pattern, drum kit sounds, Synth A (bass), and Synth B (lead/pad) at a genre-appropriate tempo.

## Feature 1: Per-Pattern BPM

### Data Change

Add `bpm: Option<f64>` to `PatternData` in `project.rs`:

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct PatternData {
    pub name: String,
    pub steps: Vec<String>,  // hex-encoded
    #[serde(default)]
    pub bpm: Option<f64>,
}
```

`#[serde(default)]` ensures backward compatibility — old saves load as `None`.

### Behavior

- When the active drum pattern changes and the new pattern has `Some(bpm)`, update `app.transport.bpm` and send `UiToAudio::SetBpm(bpm)` to the audio thread.
- If `None`, keep current BPM unchanged.
- For **queued** pattern switches (QWERTYUIOP, `[`/`]`): BPM changes when the pattern actually activates at loop boundary, not when queued.
- For **immediate** switches (Shift+QWERTYUIOP, `{`/`}`): BPM changes immediately.

### Where to Wire

BPM application happens entirely on the **UI thread** — no audio-thread changes needed.

1. **`app.rs` — `switch_pattern()`** — This method is called for both immediate switches and queued activations. Add BPM application here: if the new pattern has `Some(bpm)`, set `self.transport.bpm` and send `UiToAudio::SetBpm(bpm)`. This single point covers all cases:
   - Immediate switches (Shift+QWERTYUIOP, `{`/`}`) call `switch_pattern()` directly from `keys.rs`
   - Queued switches activate at loop boundary in `app.rs:745` which also calls `switch_pattern()`
2. **`mouse.rs`** — If pattern switching is clickable, it should also call `switch_pattern()`, so BPM is handled automatically.

### UI

No new UI needed. The transport bar already displays `app.transport.bpm`. BPM updates are visible immediately.

### No per-pattern BPM editing UI yet

Users set BPM globally with `-`/`=`. A pattern's stored BPM is set only by the demo song (or by editing the .tsp file). A "save current BPM to pattern" feature can be added later.

## Feature 2: Demo Song

### Where It Lives

Enhance the existing `ProjectFile::demo_project()` in `project.rs`. This is the factory default when no save file exists.

### Scene Lineup

| Slot | Key | Genre | BPM | Drum Pattern Preset | Drum Kit Style | Synth A (bass) | Synth B (lead/pad) |
|------|-----|-------|-----|---------------------|----------------|----------------|---------------------|
| 1 | Q | Acid Techno | 138 | Acid House | 808 | Wobble Bass | Screamer |
| 2 | W | House | 122 | Classic House | 909 | Acid Bass | Electric Piano |
| 3 | E | Deep House | 120 | Deep House | 909 | Reese Bass | Shimmer Pad |
| 4 | R | Techno | 130 | Driving Techno | 909 | Pulse Bass | Saw Lead |
| 5 | T | Downtempo / Lo-Fi | 85 | Lo-Fi Hip Hop | Lo-Fi | Sub Bass | Warm Pad |
| 6 | Y | Trance | 140 | Classic Trance | 909 | FM Bass | Trance Lead |
| 7 | U | Drum & Bass | 174 | Amen Break | Acoustic | Growl Bass | Basic Pluck |
| 8 | I | Electro | 128 | Electro Funk | 808 | Rubber Bass | Square Lead |
| 9 | O | Dub Techno | 118 | Basic Chain | 808 | Sub Bass | Ethereal |
| 10 | P | Ambient | 90 | Sparse Pulse | Minimal | Dark Pad | Bell Pluck |

### Preset Lookup

All presets are looked up by **exact name**, not by genre filtering. The implementer calls a lookup function (e.g., `find_preset_by_name()`) that searches the full preset array.

### Per Scene, We Set

- **Drum pattern**: Load steps from matching `pattern_presets.rs` preset (by name)
- **Drum kit**: Apply per-track drum sound presets from `drum_presets.rs` matching the genre (kick, snare, closed hat, open hat, clap, ride, perc, tom)
- **Synth A pattern**: Load from `synth_pattern_presets.rs` using the **"[Genre] Bass"** genre variant (e.g., "Acid Techno Bass 1")
- **Synth A kit**: Apply synth preset params from `synth_presets.rs` by name (e.g., "Wobble Bass")
- **Synth B pattern**: Load from `synth_pattern_presets.rs` using the **melodic** genre variant (e.g., "Acid Techno 1")
- **Synth B kit**: Apply synth preset params from `synth_presets.rs` by name (e.g., "Screamer")
- **BPM**: Set `Some(bpm)` on the drum pattern

### Available Synth Pattern Genres

Melodic (for Synth B): Techno, Acid Techno, Trance, Dub Techno, IDM, EDM, Drum & Bass, House, Breakbeat, Jungle, Garage, Ambient, Glitch, Electro, Downtempo

Bass (for Synth A): Techno Bass, Acid Techno Bass, Trance Bass, Dub Techno Bass, IDM Bass, EDM Bass, Drum & Bass Bass, House Bass, Breakbeat Bass, Jungle Bass, Garage Bass, Ambient Bass, Glitch Bass, Electro Bass, Downtempo Bass

### Genre Mapping for Scenes Without Exact Match

| Scene | Synth A Pattern Genre | Synth B Pattern Genre |
|-------|-----------------------|-----------------------|
| Acid Techno | Acid Techno Bass | Acid Techno |
| House | House Bass | House |
| Deep House | House Bass | House |
| Techno | Techno Bass | Techno |
| Downtempo | Downtempo Bass | Downtempo |
| Trance | Trance Bass | Trance |
| Drum & Bass | Drum & Bass Bass | Drum & Bass |
| Electro | Electro Bass | Electro |
| Dub Techno | Dub Techno Bass | Dub Techno |
| Ambient | Ambient Bass | Ambient |

Note: "Deep House" has no dedicated synth pattern genre — use "House" / "House Bass" with different preset numbers (e.g., pick preset 3-4 instead of 1 for variety).

### Pattern Naming

| Slot | Pattern Name |
|------|-------------|
| 1 | Acid Techno 138 |
| 2 | House 122 |
| 3 | Deep House 120 |
| 4 | Techno 130 |
| 5 | Downtempo 85 |
| 6 | Trance 140 |
| 7 | Drum & Bass 174 |
| 8 | Electro 128 |
| 9 | Dub Techno 118 |
| 10 | Ambient 90 |

### Kit Sharing

8 kits available, 10 patterns. Group by genre family:
- Kit 0: 808 sounds (Acid Techno, Electro, Dub Techno)
- Kit 1: 909 sounds (House, Deep House, Techno, Trance)
- Kit 2: Lo-Fi sounds (Downtempo)
- Kit 3: Acoustic sounds (DnB)
- Kit 4: Minimal sounds (Ambient)
- Kits 5-7: Duplicates or variations for future use

Same approach for synth kits — 8 slots, shared across 10 patterns by timbre family.

### Synth Kit Mapping

**Synth A kits (bass-focused):**
- Kit 0: Wobble Bass (Acid Techno)
- Kit 1: Acid Bass (House)
- Kit 2: Reese Bass (Deep House)
- Kit 3: Pulse Bass (Techno)
- Kit 4: Sub Bass (Downtempo, Dub Techno)
- Kit 5: FM Bass (Trance)
- Kit 6: Growl Bass (DnB)
- Kit 7: Rubber Bass (Electro)

**Synth B kits (lead/pad-focused):**
- Kit 0: Screamer (Acid Techno)
- Kit 1: Electric Piano (House)
- Kit 2: Shimmer Pad (Deep House)
- Kit 3: Saw Lead (Techno)
- Kit 4: Warm Pad (Downtempo)
- Kit 5: Trance Lead (Trance)
- Kit 6: Basic Pluck (DnB)
- Kit 7: Square Lead (Electro)

**Kit sharing for patterns 9-10:** Patterns 1-8 map 1:1 to kits 0-7. Patterns 9 and 10 reuse existing kits:
- Pattern 9 (Dub Techno): Synth A reuses Kit 4 (Sub Bass — same as Downtempo), Synth B reuses Kit 2 (Shimmer Pad is close to Ethereal)
- Pattern 10 (Ambient): Synth A reuses Kit 5 (FM Bass — works as a dark texture), Synth B reuses Kit 6 (Basic Pluck — Bell Pluck is similar)

The `active_synth_kit` / `active_synth_b_kit` index is set per-pattern in `demo_project()` so each scene points to the right kit. Since users don't switch patterns simultaneously, kit sharing has no audible impact.

## Edge Cases

- **Old saves**: `bpm: None` on all patterns — no behavior change, fully backward compatible.
- **User edits BPM manually**: Works as before. The transport BPM changes, but the pattern's stored `bpm` is not updated (read-only for now).
- **Queued switch timing**: BPM must change at loop boundary, not at queue time. The queued pattern index is already tracked — when it activates, apply BPM.
- **Demo project override**: If the user has a saved project, demo_project() is never called. No data loss risk.

## Files to Modify

1. **`src/sequencer/project.rs`** — Add `bpm: Option<f64>` to `PatternData`, update `demo_project()` with full scene data
2. **`src/app.rs`** — In `switch_pattern()`, apply BPM from pattern if `Some`. This covers both immediate and queued switches.
3. **`src/sequencer/transport.rs`** — No change needed, transport already has `bpm: f64`

## Testing

1. **Serialization roundtrip**: Extend `project_serialize_roundtrip` test to verify `bpm: Some(130.0)` survives save/load.
2. **Backward compat**: Add test that JSON without `bpm` field deserializes to `bpm: None`.
3. **Demo project validity**: Add test that `demo_project()` produces 10 patterns each with `Some(bpm)`, 8 kits, valid synth kit indices, and non-empty step data.

## Future Work (Not In Scope)

- **Scene abstraction (Approach B)**: Bundle drum + synth A + synth B + BPM into a `Scene` struct for linked switching
- **Save BPM to pattern**: UI action to store current transport BPM into the active pattern's `bpm` field
- **Per-pattern swing**: Similar `Option<f32>` for swing per pattern
