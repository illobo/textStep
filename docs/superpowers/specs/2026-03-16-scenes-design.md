# Scenes Feature Design

## Overview

A scene is a named snapshot of the current instrument state: which pattern and kit is active for drums, synth A, and synth B, plus BPM. Scenes allow quick recall of full arrangements from a single modal dialog.

## Data Model

```rust
pub const NUM_SCENES: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub drum_pattern: usize,
    pub drum_kit: usize,
    pub synth_a_pattern: usize,
    pub synth_a_kit: usize,
    pub synth_b_pattern: usize,
    pub synth_b_kit: usize,
    pub bpm: f64,
    pub swing: f32,
}
```

- `NUM_SCENES = 16` constant, consistent with `NUM_PATTERNS` / `NUM_KITS` style.
- Stored as `scenes: Vec<Option<Scene>>` in `ProjectFile`.
- `#[serde(default)]` for backward compatibility with existing .tsp files.
- `ProjectFile::normalize()` must clamp `scenes` vec length to `NUM_SCENES` and validate all indices within each scene (clamp `drum_pattern` to `NUM_PATTERNS`, `drum_kit` to `NUM_KITS`, etc.).
- Demo project pre-populates scenes matching its genre patterns.

## Modal UI

Opened with **Ctrl+E**. Rendered as a new `ModalState::SceneBrowser` variant.

```
+-- Scenes -------------------------------------------+
|                                                     |
|  1. Acid Intro    DR: music-1 diamond-1  SA: music-1 diamond-1  SB: music-1 diamond-1 |
|  2. House Build   DR: music-2 diamond-4  SA: music-2 diamond-2  SB: music-2 diamond-2 |
| >3. Techno Drop   DR: music-4 diamond-3  SA: music-4 diamond-4  SB: music-4 diamond-3 |
|  4. (empty)                                         |
|  ...                                                |
|                                                     |
|  Enter: queue  Shift+Enter: now                     |
|  S: save here  R: rename  D: delete  Esc: close    |
+-----------------------------------------------------+
```

(In actual rendering: music = unicode char, diamond = unicode char, with dash separators for spacing.)

Display format per scene line:
`DR: <pat_symbol>-<n> <kit_symbol>-<n>  SA: <pat_symbol>-<n> <kit_symbol>-<n>  SB: <pat_symbol>-<n> <kit_symbol>-<n>`

Where pat_symbol and kit_symbol are unicode characters defined in `theme.rs`.

## Keybindings (within modal)

| Key           | Action                                          |
|---------------|-------------------------------------------------|
| Up / Down     | Navigate scene list                             |
| Enter         | Queue scene switch (applies at pattern end)     |
| Shift+Enter   | Immediate scene switch                          |
| S             | Save current state into selected slot           |
| R             | Open text input to rename selected scene        |
| D             | Delete (clear) selected scene                   |
| Esc           | Close modal                                     |

## Scene Switching Behavior

### Queued (Enter)
- Stores target scene index in a `queued_scene: Option<usize>` field on `UiState`.
- The scene switch triggers when the **drum pattern** loops (step wraps to 0). The drum pattern is the rhythmic backbone and the most natural loop boundary. Synth patterns reset to step 0 at the same moment.
- Queuing a scene **clears any individually queued pattern/kit changes** (and vice versa: queuing an individual pattern/kit clears any queued scene). Only one type of queued change is active at a time.
- The queued scene indicator can be shown in the transport bar.

### Immediate (Shift+Enter)
- Switches all 6 indices + BPM + swing immediately.
- Reuses existing `switch_drum_pattern`, `switch_synth_kit_for`, etc. methods on `App`.

### Saving (S key)
- Snapshots the current 6 active indices + BPM + swing from `UiState` / `App` state. Scenes reference indices only, not pattern/kit data, so no `store_current_to_project()` call is needed.

## Implementation Scope

### Files to modify
- `src/sequencer/project.rs` — Add `Scene` struct, `scenes` field to `ProjectFile`, populate in `demo_project()`
- `src/app.rs` — Add `ModalState::SceneBrowser`, `SceneBrowserState` struct, `queued_scene` field, scene recall/save methods
- `src/keys.rs` — Handle Ctrl+E to open modal, handle keys within `SceneBrowser` modal
- `src/ui/mod.rs` — Render `SceneBrowser` modal (closed box, proper alignment)
- `src/ui/theme.rs` — Add `SCENE_PAT_SYMBOL` and `SCENE_KIT_SYMBOL` unicode constants
- `src/ui/transport_bar.rs` — Optional: show queued scene indicator

### Files NOT modified
- Audio engine, DSP — scenes only change indices, no new audio messages needed.
- `src/mouse.rs` — mouse interaction for the scene modal (click-to-select rows) is a future enhancement. For now, keyboard-only within the modal.

## Persistence

- Scenes serialize/deserialize as part of the .tsp project file.
- Empty slots stored as `null` in JSON array.
- Old project files without `scenes` field default to an empty vec (no scenes).

## Testing

- Unit test: scene save/recall round-trip (save current indices, recall, verify indices match)
- Unit test: serialization round-trip (scene data survives JSON encode/decode)
- Unit test: backward compat — old .tsp without `scenes` field loads with empty scenes vec
- Unit test: `normalize()` clamps out-of-bounds scene indices
- Unit test: demo project has expected pre-populated scenes
