# TextStep

TUI step sequencer / drum machine / synth in Rust. All DSP from scratch.

## Build & Test

```bash
cargo build          # dev build
cargo build --release
cargo test           # 55 tests, runs in <1s
cargo run            # launch TUI
```

## Architecture

Two-thread model: UI thread (ratatui + crossterm) and audio thread (cpal/CoreAudio). Communication via lock-free crossbeam channels (`src/messages.rs`).

### Source Map

#### Core
- `src/main.rs` — entry point, spawns audio thread, runs UI event loop
- `src/app.rs` — `App` struct (all application state), `UiState`, `FocusSection` enum, `DrumControlField`, `SynthDrag`, modal states
- `src/messages.rs` — `UiToAudio` / `AudioToUi` message enums for cross-thread comms
- `src/params.rs` — shared parameter types

#### UI Layer (`src/ui/`)
- `mod.rs` — top-level `render()` dispatch, layout computation (3 branches: expanded/collapsed synth, help overlay), modal rendering (`render_text_input`, `render_file_picker`, `render_preset_browser`, `render_pattern_browser`), activity bar
- `layout.rs` — **single source of truth** for all layout dimension constants (`TRANSPORT_HEIGHT`, `SYNTH_SECTION_HEIGHT`, etc.). Both `mod.rs` and `mouse.rs` import from here.
- `theme.rs` — color palette (hardware/synthwave: amber, cyan, pink, gold), step characters, semantic aliases
- `transport_bar.rs` — transport bar: play state, BPM, beat LEDs, swing, record, synth+drum pattern/kit selectors, loop indicators, master gauges
- `drum_grid.rs` — 8-track x 32-step grid with spaced squares, [M]/[S] per track row
- `knobs.rs` — vertical slider panel for selected drum track (10 params: Tune thru Delay)
- `synth_knobs.rs` — grouped synth params: OSC1+OSC2 (wave selectors + sliders), ENV1+ENV2+FILT (ADSR bars + filter sliders), AMP (vol/reverb/delay/sat)
- `synth_grid.rs` — synth step grid (same format as drum grid)
- `waveform.rs` — spectrum analyzer / VU meter (toggleable)
- `splash.rs` — ASCII logo + matrix rain animation
- `help_overlay.rs` — 3-column key binding popup

#### Input
- `src/keys.rs` — keyboard handler. `PARAM_INCREMENT = 0.02`. Synth control navigation uses 3 visual row groups x N columns. F2 = synth collapse toggle.
- `src/mouse.rs` — mouse handler. Click-to-focus, clickable transport, scroll wheel on params (0.01/tick), drag for value adjustment. **Layout computation must mirror `ui/mod.rs` exactly** — uses shared constants from `layout.rs`.

#### Audio (`src/audio/`)
- `engine.rs` — audio callback, receives messages, triggers voices
- `clock.rs` — beat/step timing, swing
- `drum_voice.rs` — per-track DSP (kick, snare, hats, etc.), params: tune/sweep/color/snap/filter/drive/decay/volume
- `synth_voice.rs` — polyphonic synth DSP, 2 oscillators, 2 envelopes, filter with own ADSR
- `mixer.rs` — channel mixing, send effects
- `effects.rs` — FDN reverb, delay, compressor, saturator, lookahead limiter, sidechain envelope, oversampler
- `display_buffer.rs` — lock-free audio→UI buffer for waveform display
- `fft.rs` — FFT for spectrum analyzer

#### Sequencer (`src/sequencer/`)
- `drum_pattern.rs` — `DrumPattern` (8 tracks x 32 steps + 8 `DrumTrackParams`), `TrackId` enum, `NUM_DRUM_TRACKS=8`, `MAX_STEPS=32`
- `synth_pattern.rs` — `SynthPattern` (32 `SynthStep` with note/velocity/length), `SynthParams` (27+ fields), `SynthControlField` enum
- `transport.rs` — `Transport` struct (BPM, play state, swing)
- `project.rs` — `Project` = 1 kit + 10 patterns. serde JSON serialization, hex-encoded steps. `NUM_PATTERNS=10`, `NUM_KITS=8`

#### Presets (`src/presets/`)
- `mod.rs` — preset browser state machine, categories, merge modes
- `drum_presets.rs` — hand-crafted drum sound presets
- `pattern_presets.rs` — drum pattern presets by genre
- `synth_presets.rs` — synth sound presets
- `synth_pattern_presets.rs` — synth pattern presets

## Key Patterns

### Layout Must Match in Two Places
Render layout (`ui/mod.rs`) and mouse hit-testing (`mouse.rs`) must compute identical layouts. Always use constants from `ui/layout.rs`. If you change a height/width in rendering, update the corresponding mouse hit-test.

### Synth Knobs Grouped Layout
`synth_knobs.rs` uses percentage-based `Layout::split()` for side-by-side sections. `mouse.rs` replicates the exact same splits. If you add/remove a synth param group, update both files.

### Adding a Drum Parameter
1. Add field to `DrumTrackParams` in `drum_pattern.rs`
2. Add variant to `DrumControlField` in `app.rs`
3. Add to `KNOB_FIELDS` array in `app.rs`
4. Add label in `SLIDER_LABELS` in `knobs.rs`
5. Wire into DSP in `drum_voice.rs`

### Adding a Synth Parameter
1. Add field to `SynthParams` in `synth_pattern.rs`
2. Add variant to `SynthControlField` + implement `get()`/`set()`/`full_label()`
3. Add to appropriate group constant in `synth_knobs.rs` (and same group in `mouse.rs`)
4. Wire into DSP in `synth_voice.rs`

### Color Usage
- Amber (`#e8a838`) — active steps, gauge fills, primary data
- Cyan (`#61dafb`) — transport state, beat LEDs, playhead, focused borders
- Pink (`#ff6b9d`) — focus/selection, current track, record
- Gold (`#ffd700`) — queued patterns, warnings
- Use `theme::` constants, never raw `Color::Rgb()`

### Data Directory
`~/Library/Application Support/textstep/` (macOS). Projects saved as `.tsp` (JSON).
