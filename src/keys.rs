//! Keyboard event handler: maps key events to app state changes.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, DrumControlField, FocusSection, ModalAction, ModalState};
use crate::presets::{PatternMergeMode, PresetTarget};
use crate::messages::{SynthId, UiToAudio};
use crate::sequencer::drum_pattern::{MAX_STEPS, NUM_DRUM_TRACKS, TRACK_IDS};
use crate::sequencer::project::{NUM_KITS, NUM_PATTERNS};
use crate::sequencer::synth_pattern::{SynthControlField, MAX_STEPS as SYNTH_MAX_STEPS};
use crate::sequencer::transport::{PlayState, RecordMode};

/// How much a parameter changes per arrow-key press (0.02 = 50 steps for full range).
const PARAM_INCREMENT: f32 = 0.02;

/// Map drum pad keys (bottom row) to track indices.
fn pad_key_to_track(ch: char) -> Option<usize> {
    match ch {
        'z' => Some(0), // Kick
        'x' => Some(1), // Snare
        'c' => Some(2), // CHH
        'v' => Some(3), // OHH
        'b' => Some(4), // Ride
        'n' => Some(5), // Clap
        'm' => Some(6), // Cowbell
        ',' => Some(7), // Tom
        _ => None,
    }
}

/// Map QWERTYUIOP to pattern indices 0-9.
fn pattern_key_to_index(ch: char) -> Option<usize> {
    match ch {
        'q' | 'Q' => Some(0),
        'w' | 'W' => Some(1),
        'e' | 'E' => Some(2),
        'r' | 'R' => Some(3),
        't' | 'T' => Some(4),
        'y' | 'Y' => Some(5),
        'u' | 'U' => Some(6),
        'i' | 'I' => Some(7),
        'o' | 'O' => Some(8),
        'p' | 'P' => Some(9),
        _ => None,
    }
}

/// Map 1-8 to kit indices 0-7.
fn kit_key_to_index(ch: char) -> Option<usize> {
    match ch {
        '1'..='8' => Some((ch as usize) - ('1' as usize)),
        _ => None,
    }
}

/// Determine which synth (A or B) is targeted by the current focus section.
/// Returns None if focus is on drums or transport.
fn focused_synth(focus: FocusSection) -> Option<SynthId> {
    match focus {
        FocusSection::SynthAGrid | FocusSection::SynthAControls => Some(SynthId::A),
        FocusSection::SynthBGrid | FocusSection::SynthBControls => Some(SynthId::B),
        _ => None,
    }
}

/// Get mutable reference to the SynthUiState for the given synth.
fn synth_ui_mut(app: &mut App, synth_id: SynthId) -> &mut crate::app::SynthUiState {
    match synth_id {
        SynthId::A => &mut app.ui.synth_a,
        SynthId::B => &mut app.ui.synth_b,
    }
}

/// Get mutable references to both the SynthUiState and SynthPattern for the given synth.
fn synth_ui_and_pattern(app: &mut App, synth_id: SynthId) -> (&mut crate::app::SynthUiState, &mut crate::sequencer::synth_pattern::SynthPattern) {
    match synth_id {
        SynthId::A => (&mut app.ui.synth_a, &mut app.synth_a_pattern),
        SynthId::B => (&mut app.ui.synth_b, &mut app.synth_b_pattern),
    }
}

/// Send the appropriate synth pattern to the audio thread.
fn send_synth(app: &App, synth_id: SynthId) {
    app.send_synth_pattern(synth_id);
}

/// Main key event handler — dispatches based on modal state first.
pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Modal dialogs take priority
    match &app.ui.modal {
        ModalState::TextInput { .. } => {
            handle_text_input(app, key);
            return;
        }
        ModalState::FilePicker { .. } => {
            handle_file_picker(app, key);
            return;
        }
        ModalState::PresetBrowser(_) => {
            handle_preset_browser(app, key);
            return;
        }
        ModalState::PatternBrowser(_) => {
            handle_pattern_browser(app, key);
            return;
        }
        ModalState::SceneBrowser(_) => {
            handle_scene_browser(app, key);
            return;
        }
        ModalState::None => {}
    }

    // Help overlay intercepts Esc and ?
    if app.ui.show_help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                app.ui.show_help = false;
            }
            _ => {}
        }
        return;
    }

    // ── Global keys (work in any focus) ─────────────────────────────────

    match key.code {
        // Quit
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return;
        }

        // Save
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.save_project();
            return;
        }

        // Load
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.open_load_dialog();
            return;
        }

        // Rename current pattern
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.open_rename_pattern();
            return;
        }

        // Save kit
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.save_kit();
            return;
        }

        // Load kit
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.open_load_kit_dialog();
            return;
        }

        // Scene browser
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.open_scene_browser();
            return;
        }

        // Sound preset browser
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.open_preset_browser();
            return;
        }

        // Pattern preset browser
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.open_pattern_browser();
            return;
        }

        // Help overlay
        KeyCode::Char('?') => {
            app.ui.show_help = !app.ui.show_help;
            if app.ui.show_help { app.ui.show_waveform = false; }
            return;
        }

        // Waveform/oscilloscope toggle
        KeyCode::Char('~') => {
            app.ui.show_waveform = !app.ui.show_waveform;
            if app.ui.show_waveform { app.ui.show_help = false; }
            return;
        }

        // Synth section collapse/expand toggle (bulk toggle all synth panels)
        KeyCode::F(2) => {
            let all_synth_visible = app.ui.panel_vis.synth_a_knobs
                && app.ui.panel_vis.synth_a_grid
                && app.ui.panel_vis.synth_b_knobs
                && app.ui.panel_vis.synth_b_grid;
            let new_state = !all_synth_visible;
            app.ui.panel_vis.synth_a_knobs = new_state;
            app.ui.panel_vis.synth_a_grid = new_state;
            app.ui.panel_vis.synth_b_knobs = new_state;
            app.ui.panel_vis.synth_b_grid = new_state;
            return;
        }

        // Focus navigation
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.ui.focus = app.ui.focus.next(&app.ui.panel_vis);
            return;
        }
        KeyCode::BackTab => {
            app.ui.focus = app.ui.focus.prev(&app.ui.panel_vis);
            return;
        }

        // Transport: play / pause
        KeyCode::Char(' ') => {
            match app.transport.state {
                PlayState::Stopped => app.transport.state = PlayState::Playing,
                PlayState::Playing => app.transport.state = PlayState::Paused,
                PlayState::Paused => app.transport.state = PlayState::Playing,
            }
            app.send_transport();
            return;
        }

        // Transport: stop
        KeyCode::Esc => {
            app.transport.state = PlayState::Stopped;
            app.send_transport();
            return;
        }

        // BPM adjustments: -/= for ±1, _/+ for ±10
        KeyCode::Char('-') => {
            app.transport.bpm = (app.transport.bpm - 1.0).clamp(60.0, 300.0);
            app.send_transport();
            app.dirty = true;
            return;
        }
        KeyCode::Char('=') => {
            app.transport.bpm = (app.transport.bpm + 1.0).clamp(60.0, 300.0);
            app.send_transport();
            app.dirty = true;
            return;
        }
        KeyCode::Char('_') => {
            app.transport.bpm = (app.transport.bpm - 10.0).clamp(60.0, 300.0);
            app.send_transport();
            app.dirty = true;
            return;
        }
        KeyCode::Char('+') => {
            app.transport.bpm = (app.transport.bpm + 10.0).clamp(60.0, 300.0);
            app.send_transport();
            app.dirty = true;
            return;
        }

        // Swing adjustments: < / > for ±5%
        KeyCode::Char('<') => {
            app.transport.swing = (app.transport.swing - PARAM_INCREMENT).clamp(0.50, 0.75);
            app.send_transport();
            app.dirty = true;
            let pct = (app.transport.swing * 100.0).round() as u8;
            app.show_status(format!("Swing: {}%", pct));
            return;
        }
        KeyCode::Char('>') => {
            app.transport.swing = (app.transport.swing + PARAM_INCREMENT).clamp(0.50, 0.75);
            app.send_transport();
            app.dirty = true;
            let pct = (app.transport.swing * 100.0).round() as u8;
            app.show_status(format!("Swing: {}%", pct));
            return;
        }

        // Crossfader: ( / ) to move toward A / B
        KeyCode::Char('(') => {
            app.effect_params.crossfader = (app.effect_params.crossfader - PARAM_INCREMENT).clamp(0.0, 1.0);
            app.send_effect_params();
            app.dirty = true;
            return;
        }
        KeyCode::Char(')') => {
            app.effect_params.crossfader = (app.effect_params.crossfader + PARAM_INCREMENT).clamp(0.0, 1.0);
            app.send_effect_params();
            app.dirty = true;
            return;
        }

        // Record mode toggle: backtick
        KeyCode::Char('`') => {
            app.transport.record_mode = match app.transport.record_mode {
                RecordMode::Off => RecordMode::On,
                RecordMode::On => RecordMode::Off,
            };
            app.send_transport();
            return;
        }

        // Loop toggle
        KeyCode::Char('l') => {
            app.transport.loop_config.enabled = !app.transport.loop_config.enabled;
            app.send_transport();
            return;
        }

        // Loop length cycle (Shift+L) — focus-aware
        KeyCode::Char('L') => {
            if let Some(synth_id) = focused_synth(app.ui.focus) {
                let len_ref = match synth_id {
                    SynthId::A => &mut app.transport.loop_config.synth_a_length,
                    SynthId::B => &mut app.transport.loop_config.synth_b_length,
                };
                *len_ref = match *len_ref { 8 => 16, 16 => 24, 24 => 32, _ => 8 };
                let new_len = *len_ref;
                let label = match synth_id { SynthId::A => "Synth A", SynthId::B => "Synth B" };
                app.show_status(format!("{} loop: {} steps", label, new_len));
            } else {
                app.transport.loop_config.drum_length = match app.transport.loop_config.drum_length {
                    8 => 16,
                    16 => 24,
                    24 => 32,
                    _ => 8,
                };
                app.show_status(format!("Drum loop: {} steps", app.transport.loop_config.drum_length));
            }
            app.send_transport();
            app.dirty = true;
            return;
        }

        // Param page cycle (SYN -> AMP -> FX -> SYN)
        KeyCode::Char(';') => {
            app.ui.param_page = app.ui.param_page.cycle();
            app.ui.drum_ctrl_field = DrumControlField::first_for_page(app.ui.param_page);
            return;
        }

        // Mute current drum track
        KeyCode::Char('M') => {
            let t = app.ui.drum_cursor_track;
            app.drum_pattern.params[t].mute = !app.drum_pattern.params[t].mute;
            app.send_drum_pattern();
            return;
        }

        // Solo current drum track
        KeyCode::Char('S') => {
            let t = app.ui.drum_cursor_track;
            app.drum_pattern.params[t].solo = !app.drum_pattern.params[t].solo;
            app.send_drum_pattern();
            return;
        }

        // Compressor: cycle presets (Off → Light → Medium → Heavy → Off)
        KeyCode::Char('C') => {
            let cur = app.effect_params.compressor_amount;
            app.effect_params.compressor_amount = if cur < 0.01 {
                0.25
            } else if cur < 0.30 {
                0.50
            } else if cur < 0.55 {
                0.75
            } else if cur < 0.80 {
                1.0
            } else {
                0.0
            };
            app.send_effect_params();
            app.dirty = true;
            let label = if app.effect_params.compressor_amount < 0.01 {
                "Off"
            } else if app.effect_params.compressor_amount < 0.30 {
                "Light"
            } else if app.effect_params.compressor_amount < 0.55 {
                "Medium"
            } else if app.effect_params.compressor_amount < 0.80 {
                "Heavy"
            } else {
                "Max"
            };
            app.show_status(format!("Compressor: {}", label));
            return;
        }

        // Master volume: Shift+V cycles presets
        KeyCode::Char('V') => {
            let cur = app.effect_params.master_volume;
            app.effect_params.master_volume = if cur < 0.25 {
                0.4
            } else if cur < 0.45 {
                0.6
            } else if cur < 0.65 {
                0.8
            } else if cur < 0.85 {
                1.0
            } else {
                0.2
            };
            app.send_effect_params();
            app.dirty = true;
            let pct = (app.effect_params.master_volume * 100.0).round() as u32;
            app.show_status(format!("Volume: {}%", pct));
            return;
        }

        // Tube saturator: Shift+T cycles presets (Off → Warm → Hot → Crispy → Off) — focus-aware
        KeyCode::Char('T') => {
            let is_synth = focused_synth(app.ui.focus).is_some();
            let cur = if is_synth {
                app.effect_params.synth_saturator_drive
            } else {
                app.effect_params.drum_saturator_drive
            };
            let new_val = if cur < 0.01 {
                0.25
            } else if cur < 0.30 {
                0.50
            } else if cur < 0.55 {
                0.75
            } else if cur < 0.80 {
                1.0
            } else {
                0.0
            };
            if is_synth {
                app.effect_params.synth_saturator_drive = new_val;
            } else {
                app.effect_params.drum_saturator_drive = new_val;
            }
            app.send_effect_params();
            app.dirty = true;
            let label = if new_val < 0.01 {
                "Off"
            } else if new_val < 0.30 {
                "Warm"
            } else if new_val < 0.55 {
                "Hot"
            } else if new_val < 0.80 {
                "Crispy"
            } else {
                "Fried"
            };
            let target = if is_synth { "Synth Tube" } else { "Drum Tube" };
            app.show_status(format!("{}: {}", target, label));
            return;
        }

        // Sidechain: Shift+D cycles duck depth (Off → Light → Medium → Heavy → Max → Off)
        KeyCode::Char('D') => {
            let cur = app.effect_params.sidechain_amount;
            app.effect_params.sidechain_amount = if cur < 0.01 {
                0.25
            } else if cur < 0.30 {
                0.50
            } else if cur < 0.55 {
                0.75
            } else if cur < 0.80 {
                1.0
            } else {
                0.0
            };
            app.send_effect_params();
            app.dirty = true;
            let label = if app.effect_params.sidechain_amount < 0.01 {
                "Off"
            } else if app.effect_params.sidechain_amount < 0.30 {
                "Light"
            } else if app.effect_params.sidechain_amount < 0.55 {
                "Medium"
            } else if app.effect_params.sidechain_amount < 0.80 {
                "Heavy"
            } else {
                "Max"
            };
            app.show_status(format!("Sidechain: {}", label));
            return;
        }

        // Randomize current page params (Alt+R)
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::ALT) => {
            randomize_page_params(app);
            return;
        }

        // Pattern prev/next: [ ] queued, { } immediate — focus-aware
        KeyCode::Char('[') => {
            if let Some(synth_id) = focused_synth(app.ui.focus) {
                let ui = match synth_id { SynthId::A => &app.ui.synth_a, SynthId::B => &app.ui.synth_b };
                let prev = if ui.active_pattern == 0 { NUM_PATTERNS - 1 } else { ui.active_pattern - 1 };
                app.queue_synth_pattern_for(synth_id, prev);
            } else {
                let cur = app.ui.active_pattern;
                let prev = if cur == 0 { NUM_PATTERNS - 1 } else { cur - 1 };
                app.queue_pattern(prev);
            }
            return;
        }
        KeyCode::Char(']') => {
            if let Some(synth_id) = focused_synth(app.ui.focus) {
                let ui = match synth_id { SynthId::A => &app.ui.synth_a, SynthId::B => &app.ui.synth_b };
                let next = (ui.active_pattern + 1) % NUM_PATTERNS;
                app.queue_synth_pattern_for(synth_id, next);
            } else {
                let next = (app.ui.active_pattern + 1) % NUM_PATTERNS;
                app.queue_pattern(next);
            }
            return;
        }
        KeyCode::Char('{') => {
            if let Some(synth_id) = focused_synth(app.ui.focus) {
                let ui = match synth_id { SynthId::A => &app.ui.synth_a, SynthId::B => &app.ui.synth_b };
                let prev = if ui.active_pattern == 0 { NUM_PATTERNS - 1 } else { ui.active_pattern - 1 };
                app.switch_synth_pattern_for(synth_id, prev);
            } else {
                let cur = app.ui.active_pattern;
                let prev = if cur == 0 { NUM_PATTERNS - 1 } else { cur - 1 };
                app.switch_pattern(prev);
            }
            return;
        }
        KeyCode::Char('}') => {
            if let Some(synth_id) = focused_synth(app.ui.focus) {
                let ui = match synth_id { SynthId::A => &app.ui.synth_a, SynthId::B => &app.ui.synth_b };
                let next = (ui.active_pattern + 1) % NUM_PATTERNS;
                app.switch_synth_pattern_for(synth_id, next);
            } else {
                let next = (app.ui.active_pattern + 1) % NUM_PATTERNS;
                app.switch_pattern(next);
            }
            return;
        }

        _ => {}
    }

    // ── Pattern selection: QWERTYUIOP — focus-aware ─────────────────────
    if let KeyCode::Char(ch) = key.code {
        if let Some(idx) = pattern_key_to_index(ch) {
            let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);
            if let Some(synth_id) = focused_synth(app.ui.focus) {
                if is_shift { app.switch_synth_pattern_for(synth_id, idx); } else { app.queue_synth_pattern_for(synth_id, idx); }
            } else {
                if is_shift { app.switch_pattern(idx); } else { app.queue_pattern(idx); }
            }
            return;
        }
    }

    // ── Kit selection: 1-8 — focus-aware ────────────────────────────────
    if let KeyCode::Char(ch) = key.code {
        if let Some(idx) = kit_key_to_index(ch) {
            if idx < NUM_KITS {
                if let Some(synth_id) = focused_synth(app.ui.focus) {
                    app.switch_synth_kit_for(synth_id, idx);
                } else {
                    app.switch_kit(idx);
                }
            }
            return;
        }
    }

    // ── Drum pad keys / Synth note keys (ZXCVBNM,) ─────────────────────
    if let KeyCode::Char(ch) = key.code {
        // When synth grid/controls is focused, use as chromatic keyboard
        if let Some(synth_id) = focused_synth(app.ui.focus) {
            if let Some(semitone) = synth_key_to_semitone(ch) {
                let (ui, pattern) = match synth_id {
                    SynthId::A => (&mut app.ui.synth_a, &mut app.synth_a_pattern),
                    SynthId::B => (&mut app.ui.synth_b, &mut app.synth_b_pattern),
                };
                let note = (ui.octave * 12 + semitone).min(127);
                // Trigger synth sound
                let _ = app.tx_to_audio.send(UiToAudio::TriggerSynth(synth_id, note));
                ui.flash = 6;

                // If on synth grid, write note at cursor
                let is_grid = matches!(app.ui.focus, FocusSection::SynthAGrid | FocusSection::SynthBGrid);
                if is_grid {
                    let s = ui.cursor_step;
                    pattern.steps[s].note = note;
                    pattern.steps[s].velocity = 100;
                    app.send_synth_pattern(synth_id);
                    app.dirty = true;
                    // Advance cursor
                    // Re-borrow to avoid conflict — we already wrote above
                    let ui2 = match synth_id {
                        SynthId::A => &mut app.ui.synth_a,
                        SynthId::B => &mut app.ui.synth_b,
                    };
                    ui2.cursor_step = (ui2.cursor_step + 1) % SYNTH_MAX_STEPS;
                }

                // If recording + playing, write at playhead
                if app.transport.record_mode == RecordMode::On
                    && app.transport.state == PlayState::Playing
                {
                    let step = app.ui.playback_step;
                    if step < SYNTH_MAX_STEPS {
                        let pattern = match synth_id {
                            SynthId::A => &mut app.synth_a_pattern,
                            SynthId::B => &mut app.synth_b_pattern,
                        };
                        pattern.steps[step].note = note;
                        pattern.steps[step].velocity = 100;
                        app.send_synth_pattern(synth_id);
                        app.dirty = true;
                    }
                }
                return;
            }
        }

        // Otherwise use as drum pads
        if let Some(track) = pad_key_to_track(ch) {
            // Always trigger the sound and flash
            let _ = app.tx_to_audio.send(UiToAudio::TriggerDrum(TRACK_IDS[track]));
            app.ui.flash_track(track);

            // If recording + playing, write step into pattern at playhead
            if app.transport.record_mode == RecordMode::On
                && app.transport.state == PlayState::Playing
            {
                let step = app.ui.playback_step;
                if step < MAX_STEPS {
                    app.drum_pattern.steps[track][step] = true;
                    app.send_drum_pattern();
                    app.dirty = true;
                }
            }
            return;
        }
    }

    // ── Section-specific keys ───────────────────────────────────────────

    match app.ui.focus {
        FocusSection::DrumGrid => handle_drum_grid(app, key),
        FocusSection::Knobs => handle_knobs(app, key),
        FocusSection::SynthAGrid => handle_synth_grid(app, key, SynthId::A),
        FocusSection::SynthAControls => handle_synth_controls(app, key, SynthId::A),
        FocusSection::SynthBGrid => handle_synth_grid(app, key, SynthId::B),
        FocusSection::SynthBControls => handle_synth_controls(app, key, SynthId::B),
        FocusSection::Transport => {} // transport keys are all global
    }
}

/// Map ZXCVBNM, to chromatic semitones (C, C#, D, D#, E, F, F#, G).
fn synth_key_to_semitone(ch: char) -> Option<u8> {
    match ch {
        'z' => Some(0),  // C
        'x' => Some(2),  // D
        'c' => Some(4),  // E
        'v' => Some(5),  // F
        'b' => Some(7),  // G
        'n' => Some(9),  // A
        'm' => Some(11), // B
        ',' => Some(12), // C (next octave)
        _ => None,
    }
}

// ── Modal: Text Input ───────────────────────────────────────────────────────

fn handle_text_input(app: &mut App, key: KeyEvent) {
    // Extract the current state
    let (buffer, on_confirm) = if let ModalState::TextInput { buffer, on_confirm, .. } = &mut app.ui.modal {
        (buffer, on_confirm.clone())
    } else {
        return;
    };

    match key.code {
        KeyCode::Enter => {
            let name = buffer.clone();
            app.ui.modal = ModalState::None;
            match on_confirm {
                ModalAction::SaveProject => {
                    if !name.is_empty() {
                        app.save_project_with_name(&name);
                    }
                }
                ModalAction::RenamePattern => {
                    if !name.is_empty() {
                        app.rename_current_pattern(&name);
                        app.show_status(format!("Pattern renamed: {}", name));
                    }
                }
                ModalAction::RenameScene(slot) => {
                    if !name.is_empty() {
                        app.rename_scene(slot, &name);
                        app.show_status(format!("Scene renamed: {}", name));
                    }
                }
                ModalAction::SaveKit => {
                    if !name.is_empty() {
                        app.save_kit_with_name(&name);
                    }
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            app.ui.modal = ModalState::None;
        }
        KeyCode::Backspace => {
            buffer.pop();
        }
        KeyCode::Char(ch) => {
            if buffer.len() < 40 {
                buffer.push(ch);
            }
        }
        _ => {}
    }
}

// ── Modal: File Picker ──────────────────────────────────────────────────────

fn handle_file_picker(app: &mut App, key: KeyEvent) {
    let (items, selected, on_confirm) = if let ModalState::FilePicker { items, selected, on_confirm, .. } = &mut app.ui.modal {
        (items.clone(), selected, on_confirm.clone())
    } else {
        return;
    };

    match key.code {
        KeyCode::Up => {
            if *selected > 0 {
                *selected -= 1;
            }
        }
        KeyCode::Down => {
            if *selected + 1 < items.len() {
                *selected += 1;
            }
        }
        KeyCode::Enter => {
            let idx = *selected;
            if let Some((_name, path)) = items.get(idx) {
                let path = path.clone();
                app.ui.modal = ModalState::None;
                match on_confirm {
                    ModalAction::LoadProject => app.load_project_from_path(&path),
                    ModalAction::LoadKit => app.load_kit_from_path(&path),
                    _ => {}
                }
            }
        }
        KeyCode::Esc => {
            app.ui.modal = ModalState::None;
        }
        _ => {}
    }
}

// ── Drum Grid ───────────────────────────────────────────────────────────────

fn handle_drum_grid(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Left => {
            app.ui.drum_cursor_step = if app.ui.drum_cursor_step == 0 {
                MAX_STEPS - 1
            } else {
                app.ui.drum_cursor_step - 1
            };
        }
        KeyCode::Right => {
            if app.ui.drum_cursor_step == MAX_STEPS - 1 {
                // Move into knobs panel
                app.ui.focus = FocusSection::Knobs;
                app.ui.drum_ctrl_field = crate::app::KNOB_FIELDS[0];
            } else {
                app.ui.drum_cursor_step += 1;
            }
        }
        KeyCode::Up => {
            app.ui.drum_cursor_track = if app.ui.drum_cursor_track == 0 {
                NUM_DRUM_TRACKS - 1
            } else {
                app.ui.drum_cursor_track - 1
            };
            app.ui.drum_ctrl_track = app.ui.drum_cursor_track;
        }
        KeyCode::Down => {
            app.ui.drum_cursor_track = (app.ui.drum_cursor_track + 1) % NUM_DRUM_TRACKS;
            app.ui.drum_ctrl_track = app.ui.drum_cursor_track;
        }
        KeyCode::Enter => {
            let t = app.ui.drum_cursor_track;
            let s = app.ui.drum_cursor_step;
            app.drum_pattern.steps[t][s] = !app.drum_pattern.steps[t][s];
            app.send_drum_pattern();
            app.dirty = true;
            // Advance cursor so holding Enter fills consecutive steps
            app.ui.drum_cursor_step = (app.ui.drum_cursor_step + 1) % MAX_STEPS;
        }
        _ => {}
    }
}

// ── Knobs Panel ─────────────────────────────────────────────────────────────

fn handle_knobs(app: &mut App, key: KeyEvent) {
    let t = app.ui.drum_ctrl_track;
    let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let has_alt = key.modifiers.contains(KeyModifiers::ALT);

    match key.code {
        KeyCode::Left => {
            app.ui.drum_ctrl_field = app.ui.drum_ctrl_field.prev_knob();
        }
        KeyCode::Right => {
            app.ui.drum_ctrl_field = app.ui.drum_ctrl_field.next_knob();
        }
        KeyCode::Up if has_shift || has_alt => {
            adjust_drum_field(app, t, PARAM_INCREMENT);
            if has_alt {
                let _ = app.tx_to_audio.send(UiToAudio::TriggerDrum(TRACK_IDS[t]));
                app.ui.flash_track(t);
            }
        }
        KeyCode::Down if has_shift || has_alt => {
            adjust_drum_field(app, t, -PARAM_INCREMENT);
            if has_alt {
                let _ = app.tx_to_audio.send(UiToAudio::TriggerDrum(TRACK_IDS[t]));
                app.ui.flash_track(t);
            }
        }
        KeyCode::Up => {
            // Move between row 1 (0-4) and row 2 (5-9), or change track at edge
            if let Some(idx) = app.ui.drum_ctrl_field.knob_index() {
                if idx >= 5 {
                    // Row 2 -> Row 1 (same column)
                    app.ui.drum_ctrl_field = crate::app::KNOB_FIELDS[idx - 5];
                } else {
                    // Row 1 -> change track up
                    app.ui.drum_ctrl_track = if t == 0 { NUM_DRUM_TRACKS - 1 } else { t - 1 };
                    app.ui.drum_cursor_track = app.ui.drum_ctrl_track;
                }
            }
        }
        KeyCode::Down => {
            if let Some(idx) = app.ui.drum_ctrl_field.knob_index() {
                if idx < 5 {
                    // Row 1 -> Row 2 (same column)
                    app.ui.drum_ctrl_field = crate::app::KNOB_FIELDS[idx + 5];
                } else {
                    // Row 2 -> change track down
                    app.ui.drum_ctrl_track = (t + 1) % NUM_DRUM_TRACKS;
                    app.ui.drum_cursor_track = app.ui.drum_ctrl_track;
                }
            }
        }
        _ => {}
    }
}

// ── Synth Grid ─────────────────────────────────────────────────────────────

fn handle_synth_grid(app: &mut App, key: KeyEvent, synth_id: SynthId) {
    // Helper macro-like closures aren't ideal; use direct match for each synth
    let controls_focus = match synth_id {
        SynthId::A => FocusSection::SynthAControls,
        SynthId::B => FocusSection::SynthBControls,
    };
    let label = match synth_id { SynthId::A => "Synth A", SynthId::B => "Synth B" };

    match key.code {
        // Shift+Left: decrease note length
        KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
            let (ui, pattern) = synth_ui_and_pattern(app, synth_id);
            let s = ui.cursor_step;
            if pattern.steps[s].is_active() && pattern.steps[s].length > 1 {
                pattern.steps[s].length -= 1;
                send_synth(app, synth_id);
                app.dirty = true;
            }
        }
        // Shift+Right: increase note length
        KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
            let loop_len = match synth_id {
                SynthId::A => app.transport.loop_config.synth_a_length as usize,
                SynthId::B => app.transport.loop_config.synth_b_length as usize,
            };
            let (ui, pattern) = synth_ui_and_pattern(app, synth_id);
            let s = ui.cursor_step;
            if pattern.steps[s].is_active() {
                let max_length = (loop_len - s).min(32) as u8;
                if pattern.steps[s].length < max_length {
                    pattern.steps[s].length += 1;
                    send_synth(app, synth_id);
                    app.dirty = true;
                }
            }
        }
        KeyCode::Left => {
            let ui = synth_ui_mut(app, synth_id);
            ui.cursor_step = if ui.cursor_step == 0 {
                SYNTH_MAX_STEPS - 1
            } else {
                ui.cursor_step - 1
            };
        }
        KeyCode::Right => {
            let ui = synth_ui_mut(app, synth_id);
            if ui.cursor_step == SYNTH_MAX_STEPS - 1 {
                // Move into synth controls
                app.ui.focus = controls_focus;
            } else {
                let ui = synth_ui_mut(app, synth_id);
                ui.cursor_step += 1;
            }
        }
        KeyCode::Up => {
            // Change note pitch up (semitone), or Shift for octave
            let (ui, pattern) = synth_ui_and_pattern(app, synth_id);
            let s = ui.cursor_step;
            if pattern.steps[s].is_active() {
                let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { 12 } else { 1 };
                pattern.steps[s].note = (pattern.steps[s].note + delta).min(127);
                let note = pattern.steps[s].note;
                ui.flash = 6;
                send_synth(app, synth_id);
                app.dirty = true;
                let _ = app.tx_to_audio.send(UiToAudio::TriggerSynth(synth_id, note));
            }
        }
        KeyCode::Down => {
            let (ui, pattern) = synth_ui_and_pattern(app, synth_id);
            let s = ui.cursor_step;
            if pattern.steps[s].is_active() {
                let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { 12 } else { 1 };
                pattern.steps[s].note = pattern.steps[s].note.saturating_sub(delta).max(12);
                let note = pattern.steps[s].note;
                ui.flash = 6;
                send_synth(app, synth_id);
                app.dirty = true;
                let _ = app.tx_to_audio.send(UiToAudio::TriggerSynth(synth_id, note));
            }
        }
        KeyCode::Enter => {
            let (ui, pattern) = synth_ui_and_pattern(app, synth_id);
            let s = ui.cursor_step;
            let octave = ui.octave;
            let step = &mut pattern.steps[s];
            if step.is_active() {
                // Toggle off — reset length
                step.velocity = 0;
                step.length = 1;
            } else {
                // Toggle on with default note at current octave
                step.note = octave * 12 + 12; // C at current octave
                step.velocity = 100;
                step.length = 1;
            }
            send_synth(app, synth_id);
            app.dirty = true;
            // Advance cursor
            let ui = synth_ui_mut(app, synth_id);
            ui.cursor_step = (ui.cursor_step + 1) % SYNTH_MAX_STEPS;
        }
        KeyCode::Char('(') => {
            // Octave down
            let ui = synth_ui_mut(app, synth_id);
            if ui.octave > 0 {
                ui.octave -= 1;
                let oct = ui.octave;
                app.show_status(format!("{} octave: {}", label, oct));
            }
        }
        KeyCode::Char(')') => {
            // Octave up
            let ui = synth_ui_mut(app, synth_id);
            if ui.octave < 8 {
                ui.octave += 1;
                let oct = ui.octave;
                app.show_status(format!("{} octave: {}", label, oct));
            }
        }
        _ => {}
    }
}

// ── Synth Controls ──────────────────────────────────────────────────────────

/// Visual rows for synth knob navigation.
/// These match the 3 visual row groups in synth_knobs.rs:
///   Row 0: OSC1 (left) + OSC2 (right)
///   Row 1: ENV1 (left) + ENV2 (middle) + FILT (right)
///   Row 2: AMP
const SYNTH_CTRL_ROWS: [&[SynthControlField]; 3] = [
    // Row 0: OSC1 | OSC2 (side by side visually)
    &[
        SynthControlField::Osc1Waveform, SynthControlField::Osc1Tune, SynthControlField::Osc1Pwm, SynthControlField::Osc1Level, SynthControlField::Glide,
        SynthControlField::Osc2Waveform, SynthControlField::Osc2Tune, SynthControlField::Osc2Pwm, SynthControlField::Osc2Level, SynthControlField::Osc2Detune, SynthControlField::SubLevel, SynthControlField::SubWaveform, SynthControlField::OscSync,
    ],
    // Row 1: ENV1 | ENV2 | FILT (side by side visually)
    &[
        SynthControlField::Env1Attack, SynthControlField::Env1Decay, SynthControlField::Env1Sustain, SynthControlField::Env1Release,
        SynthControlField::Env2Attack, SynthControlField::Env2Decay, SynthControlField::Env2Sustain, SynthControlField::Env2Release,
        SynthControlField::FilterType, SynthControlField::FilterCutoff, SynthControlField::FilterResonance, SynthControlField::FilterEnvAmount, SynthControlField::FilterKeyFollow, SynthControlField::FilterEnvAttack, SynthControlField::FilterEnvDecay, SynthControlField::FilterEnvSustain, SynthControlField::FilterEnvRelease,
    ],
    // Row 2: AMP | LFO1 | LFO2
    &[
        SynthControlField::Volume, SynthControlField::SendReverb, SynthControlField::SendDelay,
        SynthControlField::LfoWaveform, SynthControlField::LfoDivision, SynthControlField::LfoDepth, SynthControlField::LfoDest,
        SynthControlField::Lfo2Waveform, SynthControlField::Lfo2Division, SynthControlField::Lfo2Depth, SynthControlField::Lfo2Dest,
    ],
];

/// Find (row, col) of a field in the 2D layout.
fn find_synth_field_pos(field: SynthControlField) -> (usize, usize) {
    for (r, row) in SYNTH_CTRL_ROWS.iter().enumerate() {
        for (c, f) in row.iter().enumerate() {
            if *f == field {
                return (r, c);
            }
        }
    }
    (0, 0)
}

fn handle_synth_controls(app: &mut App, key: KeyEvent, synth_id: SynthId) {
    let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let has_alt = key.modifiers.contains(KeyModifiers::ALT);

    match key.code {
        KeyCode::Left => {
            let ui = synth_ui_mut(app, synth_id);
            let (r, c) = find_synth_field_pos(ui.ctrl_field);
            if c > 0 {
                ui.ctrl_field = SYNTH_CTRL_ROWS[r][c - 1];
            }
        }
        KeyCode::Right => {
            let ui = synth_ui_mut(app, synth_id);
            let (r, c) = find_synth_field_pos(ui.ctrl_field);
            if c + 1 < SYNTH_CTRL_ROWS[r].len() {
                ui.ctrl_field = SYNTH_CTRL_ROWS[r][c + 1];
            }
        }
        KeyCode::Up if has_shift || has_alt => {
            adjust_synth_field(app, synth_id, PARAM_INCREMENT);
            if has_alt {
                let ui = synth_ui_mut(app, synth_id);
                let note = ui.octave * 12 + 12;
                ui.flash = 6;
                let _ = app.tx_to_audio.send(UiToAudio::TriggerSynth(synth_id, note));
            }
        }
        KeyCode::Down if has_shift || has_alt => {
            adjust_synth_field(app, synth_id, -PARAM_INCREMENT);
            if has_alt {
                let ui = synth_ui_mut(app, synth_id);
                let note = ui.octave * 12 + 12;
                ui.flash = 6;
                let _ = app.tx_to_audio.send(UiToAudio::TriggerSynth(synth_id, note));
            }
        }
        KeyCode::Up => {
            let ui = synth_ui_mut(app, synth_id);
            let (r, c) = find_synth_field_pos(ui.ctrl_field);
            if r > 0 {
                let new_row = &SYNTH_CTRL_ROWS[r - 1];
                let new_c = c.min(new_row.len() - 1);
                ui.ctrl_field = new_row[new_c];
            }
        }
        KeyCode::Down => {
            let ui = synth_ui_mut(app, synth_id);
            let (r, c) = find_synth_field_pos(ui.ctrl_field);
            if r + 1 < SYNTH_CTRL_ROWS.len() {
                let new_row = &SYNTH_CTRL_ROWS[r + 1];
                let new_c = c.min(new_row.len() - 1);
                ui.ctrl_field = new_row[new_c];
            }
        }
        _ => {}
    }
}

fn adjust_synth_field(app: &mut App, synth_id: SynthId, delta: f32) {
    let ui = synth_ui_mut(app, synth_id);
    let field = ui.ctrl_field;
    let pattern = match synth_id {
        SynthId::A => &mut app.synth_a_pattern,
        SynthId::B => &mut app.synth_b_pattern,
    };
    if field == SynthControlField::Mute {
        pattern.params.mute = !pattern.params.mute;
    } else if field.is_enum() {
        let max_val: u8 = match field {
            SynthControlField::OscSync => 1,   // toggle: 0 or 1
            SynthControlField::SubWaveform => 2, // Sqr/Sin/Saw
            SynthControlField::FilterType => 2,
            SynthControlField::LfoWaveform | SynthControlField::Lfo2Waveform => 2,
            SynthControlField::LfoDivision | SynthControlField::Lfo2Division => 9,
            SynthControlField::LfoDest | SynthControlField::Lfo2Dest => (crate::sequencer::synth_pattern::LFO_DEST_FIELDS.len() - 1) as u8,
            _ => 3, // Osc1/Osc2 waveforms
        };
        let cur = field.get(&pattern.params);
        let cur_int = (cur * max_val as f32).round() as u8;
        let new_int = if delta > 0.0 {
            (cur_int + 1).min(max_val)
        } else {
            cur_int.saturating_sub(1)
        };
        field.set(&mut pattern.params, new_int as f32 / max_val as f32);
    } else {
        let cur = field.get(&pattern.params);
        field.set(&mut pattern.params, cur + delta);
    }
    send_synth(app, synth_id);
    app.dirty = true;
}

fn randomize_page_params(app: &mut App) {
    use std::time::SystemTime;
    // Seed from system time
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let mut rng = seed;

    let mut rand_f32 = || -> f32 {
        // xorshift32
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        (rng as f32) / (u32::MAX as f32)
    };

    let page = app.ui.param_page;
    for track in 0..NUM_DRUM_TRACKS {
        let p = &mut app.drum_pattern.params[track];
        match page {
            crate::app::ParamPage::Synth => {
                // Skip tune — randomize sweep, color, snap
                p.sweep = rand_f32();
                p.color = rand_f32();
                p.snap = rand_f32();
            }
            crate::app::ParamPage::Amp => {
                // Randomize filter, drive, decay — keep volume untouched
                p.filter = rand_f32();
                p.drive = rand_f32();
                p.decay = rand_f32();
            }
            crate::app::ParamPage::Fx => {
                p.send_reverb = rand_f32() * 0.5; // keep sends moderate
                p.send_delay = rand_f32() * 0.5;
            }
        }
    }
    app.send_drum_pattern();
    app.dirty = true;
    let label = page.label();
    app.show_status(format!("Randomized {} params", label));
}

// ── Modal: Preset Browser ────────────────────────────────────────────────────

fn handle_preset_browser(app: &mut App, key: KeyEvent) {
    let browser = if let ModalState::PresetBrowser(ref mut b) = app.ui.modal {
        b
    } else {
        return;
    };

    match key.code {
        KeyCode::Up => {
            if browser.preset_idx > 0 {
                browser.preset_idx -= 1;
            }
            // Preview: trigger the sound with new params
            preview_preset(app);
        }
        KeyCode::Down => {
            if browser.preset_idx + 1 < browser.preset_names.len() {
                browser.preset_idx += 1;
            }
            preview_preset(app);
        }
        KeyCode::Left => {
            if browser.category_idx > 0 {
                browser.category_idx -= 1;
                browser.refresh_presets();
            }
        }
        KeyCode::Right => {
            if browser.category_idx + 1 < browser.categories.len() {
                browser.category_idx += 1;
                browser.refresh_presets();
            }
        }
        KeyCode::Char(' ') => {
            // Audition: trigger sound with current preset
            preview_preset(app);
        }
        KeyCode::Enter => {
            // Apply and close
            let browser = if let ModalState::PresetBrowser(ref b) = app.ui.modal { b.clone() } else { return; };
            match &browser.target {
                PresetTarget::DrumSound(track) => {
                    if let Some(params) = browser.selected_drum_params() {
                        let track = *track;
                        let name = browser.preset_names.get(browser.preset_idx).copied().unwrap_or("?");
                        app.apply_drum_preset(track, params);
                        app.ui.modal = ModalState::None;
                        app.show_status(format!("Loaded: {}", name));
                    }
                }
                PresetTarget::SynthSound => {
                    if let Some(params) = browser.selected_synth_params() {
                        let name = browser.preset_names.get(browser.preset_idx).copied().unwrap_or("?");
                        let synth_id = browser.target_synth;
                        app.apply_synth_preset(synth_id, params);
                        app.ui.modal = ModalState::None;
                        app.show_status(format!("Loaded: {}", name));
                    }
                }
                PresetTarget::Pattern | PresetTarget::SynthPattern => {} // handled by PatternBrowser modal
            }
        }
        KeyCode::Esc => {
            app.ui.modal = ModalState::None;
        }
        _ => {}
    }
}

// ── Modal: Pattern Browser ───────────────────────────────────────────────────

fn handle_pattern_browser(app: &mut App, key: KeyEvent) {
    let pb = if let ModalState::PatternBrowser(ref mut pb) = app.ui.modal {
        pb
    } else {
        return;
    };

    match key.code {
        KeyCode::Up => {
            if pb.browser.preset_idx > 0 {
                pb.browser.preset_idx -= 1;
            }
        }
        KeyCode::Down => {
            if pb.browser.preset_idx + 1 < pb.browser.preset_names.len() {
                pb.browser.preset_idx += 1;
            }
        }
        KeyCode::Left => {
            if pb.browser.category_idx > 0 {
                pb.browser.category_idx -= 1;
                pb.browser.refresh_presets();
            }
        }
        KeyCode::Right => {
            if pb.browser.category_idx + 1 < pb.browser.categories.len() {
                pb.browser.category_idx += 1;
                pb.browser.refresh_presets();
            }
        }
        // Toggle merge mode with Tab
        KeyCode::Tab => {
            pb.toggle_merge_mode();
        }
        KeyCode::Enter => {
            let pb = if let ModalState::PatternBrowser(ref pb) = app.ui.modal { pb.clone() } else { return; };
            let merge = pb.merge_mode;
            let mode_label = match merge {
                PatternMergeMode::Replace => "Replaced",
                PatternMergeMode::Layer => "Layered",
            };
            if let Some(preset) = pb.browser.selected_pattern() {
                let name = preset.name;
                app.apply_pattern_preset(&preset.steps, merge);
                app.show_status(format!("{}: {}", mode_label, name));
            } else if let Some(preset) = pb.browser.selected_synth_pattern() {
                let name = preset.name;
                let synth_id = pb.browser.target_synth;
                app.apply_synth_pattern_preset(synth_id, &preset.steps, merge);
                app.show_status(format!("{}: {}", mode_label, name));
            }
        }
        KeyCode::Esc => {
            app.ui.modal = ModalState::None;
        }
        _ => {}
    }
}

fn handle_scene_browser(app: &mut App, key: KeyEvent) {
    let browser = if let ModalState::SceneBrowser(ref mut b) = app.ui.modal {
        b
    } else {
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.ui.modal = ModalState::None;
        }
        KeyCode::Up => {
            if browser.selected > 0 {
                browser.selected -= 1;
            }
        }
        KeyCode::Down => {
            let max = crate::sequencer::project::NUM_SCENES.min(14) - 1;
            if browser.selected < max {
                browser.selected += 1;
            }
        }
        KeyCode::Enter => {
            let idx = browser.selected;
            app.queue_scene(idx);
            app.show_status(format!("Scene {} queued", idx + 1));
        }
        KeyCode::Char('!') => {
            let idx = browser.selected;
            app.apply_scene_immediate(idx);
            app.show_status(format!("Scene {} applied", idx + 1));
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            let idx = browser.selected;
            app.save_scene(idx);
            app.show_status(format!("Scene {} saved", idx + 1));
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            let idx = browser.selected;
            app.delete_scene(idx);
            app.show_status(format!("Scene {} deleted", idx + 1));
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            let idx = browser.selected;
            let current_name = app.project.scenes.get(idx)
                .and_then(|s| s.as_ref())
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("Scene {}", idx + 1));
            app.ui.modal = ModalState::TextInput {
                prompt: format!("Rename scene {}:", idx + 1),
                buffer: current_name,
                on_confirm: ModalAction::RenameScene(idx),
            };
        }
        _ => {}
    }
}

fn preview_preset(app: &mut App) {
    use crate::sequencer::transport::PlayState;

    // Only audition when sequencer is stopped — don't interrupt live playback
    if app.transport.state != PlayState::Stopped {
        return;
    }

    let browser = if let ModalState::PresetBrowser(ref b) = app.ui.modal { b } else { return; };
    match &browser.target {
        PresetTarget::DrumSound(track) => {
            if let Some(params) = browser.selected_drum_params() {
                let track = *track;
                app.apply_drum_preset(track, params);
                let _ = app.tx_to_audio.send(UiToAudio::TriggerDrum(TRACK_IDS[track]));
                app.ui.flash_track(track);
            }
        }
        PresetTarget::SynthSound => {
            if let Some(params) = browser.selected_synth_params() {
                let synth_id = browser.target_synth;
                app.apply_synth_preset(synth_id, params);
                let (octave, flash) = match synth_id {
                    SynthId::A => (app.ui.synth_a.octave, &mut app.ui.synth_a.flash),
                    SynthId::B => (app.ui.synth_b.octave, &mut app.ui.synth_b.flash),
                };
                let note = octave * 12 + 12;
                let _ = app.tx_to_audio.send(UiToAudio::TriggerSynth(synth_id, note));
                *flash = 6;
            }
        }
        PresetTarget::Pattern | PresetTarget::SynthPattern => {} // no preview for patterns
    }
}

fn adjust_drum_field(app: &mut App, track: usize, delta: f32) {
    let params = &mut app.drum_pattern.params[track];
    match app.ui.drum_ctrl_field {
        DrumControlField::Tune => params.tune = (params.tune + delta).clamp(0.0, 1.0),
        DrumControlField::Sweep => params.sweep = (params.sweep + delta).clamp(0.0, 1.0),
        DrumControlField::Color => params.color = (params.color + delta).clamp(0.0, 1.0),
        DrumControlField::Snap => params.snap = (params.snap + delta).clamp(0.0, 1.0),
        DrumControlField::Filter => params.filter = (params.filter + delta).clamp(0.0, 1.0),
        DrumControlField::Drive => params.drive = (params.drive + delta).clamp(0.0, 1.0),
        DrumControlField::Decay => params.decay = (params.decay + delta).clamp(0.0, 1.0),
        DrumControlField::Volume => params.volume = (params.volume + delta).clamp(0.0, 1.0),
        DrumControlField::SendReverb => params.send_reverb = (params.send_reverb + delta).clamp(0.0, 1.0),
        DrumControlField::SendDelay => params.send_delay = (params.send_delay + delta).clamp(0.0, 1.0),
        DrumControlField::Pan => params.pan = (params.pan + delta).clamp(0.0, 1.0),
        DrumControlField::Mute => params.mute = !params.mute,
        DrumControlField::Solo => params.solo = !params.solo,
    }
    app.send_drum_pattern();
    app.dirty = true;
}
