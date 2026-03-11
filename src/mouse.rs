//! Mouse event handler. Hit-testing layout MUST mirror ui/mod.rs — uses shared
//! constants from ui/layout.rs.

use std::time::Instant;

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::{App, CompressorDrag, DragState, DrumControlField, FocusSection, KNOB_FIELDS, ModalState, SynthDrag, SynthNoteDrag};
use crate::messages::{SynthId, UiToAudio};
use crate::sequencer::drum_pattern::{NUM_DRUM_TRACKS, TRACK_IDS};
use crate::sequencer::project::{NUM_KITS, NUM_PATTERNS};
use crate::ui::layout::{compute_dual_layout, DualSynthLayout};

/// Threshold for double-click detection.
const DOUBLE_CLICK_MS: u128 = 300;

/// Sensitivity: parameter change per pixel of vertical mouse movement.
/// 0.04 = full range in ~25 rows of drag.
const DRAG_SENSITIVITY: f32 = 0.04;

/// Routes mouse events (click, scroll, drag) to the appropriate UI section.
/// Layout rectangles are recomputed from `term_size` to match rendering.
pub fn handle_mouse(app: &mut App, event: MouseEvent, term_size: Rect) {
    // Ignore mouse during splash or modals
    if app.ui.splash.phase != crate::app::SplashPhase::Done {
        return;
    }
    if app.ui.modal != ModalState::None {
        return;
    }

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            handle_left_down(app, event.column, event.row, term_size);
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            handle_drag(app, event.column, event.row, term_size);
        }
        MouseEventKind::Up(MouseButton::Left) => {
            // If synth note drag ended without movement, toggle the step
            if let Some(ref drag) = app.ui.mouse.synth_note_drag {
                if app.synth_a_pattern.steps[drag.step].length == drag.original_length {
                    // No length change — treat as double-click toggle on second click
                    // (first click just selects; this is handled by last_click logic)
                }
            }
            app.ui.mouse.drag = None;
            app.ui.mouse.compressor_drag = None;
            app.ui.mouse.fader_drag = None;
            app.ui.mouse.synth_drag = None;
            app.ui.mouse.synth_note_drag = None;
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let delta: f32 = if matches!(event.kind, MouseEventKind::ScrollUp) { 0.01 } else { -0.01 };
            handle_scroll(app, event.column, event.row, delta, term_size);
        }
        _ => {}
    }
}

fn handle_scroll(app: &mut App, col: u16, row: u16, delta: f32, term_size: Rect) {
    let ly = compute_dual_layout(term_size, &app.ui.panel_vis);

    if hit_test_area(col, row, ly.drum_knobs) {
        // Scroll over drum knobs: adjust currently selected drum param
        let track = app.ui.drum_ctrl_track;
        let field = app.ui.drum_ctrl_field;
        let current = get_param_value(&app.drum_pattern.params[track], field);
        set_param_value(&mut app.drum_pattern.params[track], field, current + delta);
        app.send_drum_pattern();
        app.dirty = true;
    } else if hit_test_area(col, row, ly.synth_a_knobs) {
        // Scroll over synth A knobs
        let field = app.ui.synth_a.ctrl_field;
        if !field.is_enum() {
            let current = field.get(&app.synth_a_pattern.params);
            field.set(&mut app.synth_a_pattern.params, (current + delta).clamp(0.0, 1.0));
            app.send_synth_pattern(SynthId::A);
            app.dirty = true;
        }
    } else if hit_test_area(col, row, ly.synth_b_knobs) {
        // Scroll over synth B knobs
        let field = app.ui.synth_b.ctrl_field;
        if !field.is_enum() {
            let current = field.get(&app.synth_b_pattern.params);
            field.set(&mut app.synth_b_pattern.params, (current + delta).clamp(0.0, 1.0));
            app.send_synth_pattern(SynthId::B);
            app.dirty = true;
        }
    } else if hit_test_compressor_gauge(col, row, ly.transport) {
        // Scroll over compressor gauge
        app.effect_params.compressor_amount = (app.effect_params.compressor_amount + delta).clamp(0.0, 1.0);
        app.send_effect_params();
        app.dirty = true;
    }
}

fn handle_left_down(app: &mut App, col: u16, row: u16, term_size: Rect) {
    let ly = compute_dual_layout(term_size, &app.ui.panel_vis);

    // ── Panel toggle clicks ([X] on expanded, anywhere on collapsed) ──
    if check_panel_toggle(col, row, &ly, &mut app.ui.panel_vis) {
        return;
    }

    // ── Click-to-focus on sections ──────────────────────────────────
    if hit_test_area(col, row, ly.transport) {
        app.ui.focus = FocusSection::Transport;
    } else if hit_test_area(col, row, ly.synth_a_grid) {
        app.ui.focus = FocusSection::SynthAGrid;
    } else if hit_test_area(col, row, ly.synth_a_knobs) {
        app.ui.focus = FocusSection::SynthAControls;
    } else if hit_test_area(col, row, ly.synth_b_grid) {
        app.ui.focus = FocusSection::SynthBGrid;
    } else if hit_test_area(col, row, ly.synth_b_knobs) {
        app.ui.focus = FocusSection::SynthBControls;
    } else if hit_test_area(col, row, ly.drum_grid) {
        app.ui.focus = FocusSection::DrumGrid;
    } else if hit_test_area(col, row, ly.drum_knobs) {
        app.ui.focus = FocusSection::Knobs;
    }

    // ── Synth A zones ───────────────────────────────────────────────
    if let Some(step) = hit_test_synth_step(col, row, ly.synth_a_grid) {
        handle_synth_step_click_with_col(app, SynthId::A, step, col);
    } else if let Some(field) = hit_test_synth_knobs(col, row, ly.synth_a_knobs) {
        handle_synth_knobs_click(app, SynthId::A, field, row);
    // ── Synth B zones ───────────────────────────────────────────────
    } else if let Some(step) = hit_test_synth_step(col, row, ly.synth_b_grid) {
        handle_synth_step_click_with_col(app, SynthId::B, step, col);
    } else if let Some(field) = hit_test_synth_knobs(col, row, ly.synth_b_knobs) {
        handle_synth_knobs_click(app, SynthId::B, field, row);
    // ── Drum zones ──────────────────────────────────────────────────
    } else if let Some((track, step)) = hit_test_grid_step(col, row, ly.drum_grid) {
        handle_grid_click(app, track, step);
    } else if let Some((track, is_mute)) = hit_test_mute_solo(col, row, ly.drum_grid) {
        handle_mute_solo_click(app, track, is_mute);
    } else if let Some(field) = hit_test_knobs_panel(col, row, ly.drum_knobs) {
        handle_knobs_click(app, field, row);
    // ── Bottom / transport zones ────────────────────────────────────
    } else if let Some(track) = hit_test_activity_pad(col, row, ly.activity_bar) {
        handle_pad_click(app, track);
    } else if hit_test_compressor_gauge(col, row, ly.transport) {
        handle_compressor_click(app, row);
    } else if let Some((idx, is_synth)) = hit_test_pattern_selector(col, row, ly.transport) {
        if is_synth { app.queue_synth_pattern(idx); } else { app.queue_pattern(idx); }
    } else if let Some((idx, is_synth)) = hit_test_kit_selector(col, row, ly.transport) {
        if is_synth { app.switch_synth_kit(idx); } else { app.switch_kit(idx); }
    } else if hit_test_play_button(col, row, ly.transport) {
        use crate::sequencer::transport::PlayState;
        app.transport.state = match app.transport.state {
            PlayState::Stopped => PlayState::Playing,
            PlayState::Playing => PlayState::Paused,
            PlayState::Paused => PlayState::Playing,
        };
        app.send_transport();
    } else if hit_test_record_button(col, row, ly.transport) {
        use crate::sequencer::transport::RecordMode;
        app.transport.record_mode = match app.transport.record_mode {
            RecordMode::Off => RecordMode::On,
            RecordMode::On => RecordMode::Off,
        };
        app.send_transport();
    }
}

// ── Panel toggle click detection ─────────────────────────────────────────────

/// Check if a click is on a panel toggle control.
/// Clicking [X] (first 4 chars) of an expanded panel's title bar collapses it.
/// Clicking anywhere on a collapsed bar expands that panel.
/// Returns true if a toggle was performed.
fn check_panel_toggle(col: u16, row: u16, ly: &DualSynthLayout, vis: &mut crate::app::PanelVisibility) -> bool {
    // Helper: check collapsed bar (clicking anywhere expands)
    fn check_collapsed(col: u16, row: u16, rect: Rect) -> bool {
        rect.height > 0 && row >= rect.y && row < rect.y + rect.height
            && col >= rect.x && col < rect.x + rect.width
    }
    // Helper: check expanded title bar [X] region (first 4 chars of first row)
    fn check_expanded_toggle(col: u16, row: u16, rect: Rect) -> bool {
        rect.height > 0 && row == rect.y && col >= rect.x && col < rect.x + 4
    }

    // Synth A Knobs
    if vis.synth_a_knobs && check_expanded_toggle(col, row, ly.synth_a_knobs) {
        vis.synth_a_knobs = false;
        return true;
    }
    if !vis.synth_a_knobs && check_collapsed(col, row, ly.synth_a_knobs_collapsed) {
        vis.synth_a_knobs = true;
        return true;
    }

    // Synth A Grid
    if vis.synth_a_grid && check_expanded_toggle(col, row, ly.synth_a_grid) {
        vis.synth_a_grid = false;
        return true;
    }
    if !vis.synth_a_grid && check_collapsed(col, row, ly.synth_a_grid_collapsed) {
        vis.synth_a_grid = true;
        return true;
    }

    // Synth B Knobs
    if vis.synth_b_knobs && check_expanded_toggle(col, row, ly.synth_b_knobs) {
        vis.synth_b_knobs = false;
        return true;
    }
    if !vis.synth_b_knobs && check_collapsed(col, row, ly.synth_b_knobs_collapsed) {
        vis.synth_b_knobs = true;
        return true;
    }

    // Synth B Grid
    if vis.synth_b_grid && check_expanded_toggle(col, row, ly.synth_b_grid) {
        vis.synth_b_grid = false;
        return true;
    }
    if !vis.synth_b_grid && check_collapsed(col, row, ly.synth_b_grid_collapsed) {
        vis.synth_b_grid = true;
        return true;
    }

    // Drum Knobs
    if vis.drum_knobs && check_expanded_toggle(col, row, ly.drum_knobs) {
        vis.drum_knobs = false;
        return true;
    }
    if !vis.drum_knobs && check_collapsed(col, row, ly.drum_knobs_collapsed) {
        vis.drum_knobs = true;
        return true;
    }

    // Waveform
    if vis.waveform && check_expanded_toggle(col, row, ly.waveform) {
        vis.waveform = false;
        return true;
    }
    if !vis.waveform && check_collapsed(col, row, ly.waveform_collapsed) {
        vis.waveform = true;
        return true;
    }

    false
}

// ── Synth step hit testing ───────────────────────────────────────────────────

/// Simple area containment check.
fn hit_test_area(col: u16, row: u16, area: Rect) -> bool {
    col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
}

/// Returns step index if the click is on a synth step cell.
/// Updated for new grid format: NAME_WIDTH=9, header+spacer before step row.
fn hit_test_synth_step(col: u16, row: u16, grid_area: Rect) -> Option<usize> {
    let inner_x = grid_area.x + 1;
    let inner_y = grid_area.y + 1;

    // Step row is after header (row 0) + spacer (row 1) = inner_y + 2
    if row != inner_y + 2 {
        return None;
    }

    // Steps start after track name column (NAME_WIDTH = 9)
    let steps_x_start = inner_x + 9;
    let rel_x = col.checked_sub(steps_x_start)?;

    // Each step is 2 chars, with a bar separator ┃ at position 32
    if rel_x < 32 {
        let s = rel_x / 2;
        if s < 16 { Some(s as usize) } else { None }
    } else if rel_x == 32 {
        None // bar separator
    } else {
        let s = (rel_x - 33) / 2;
        if s < 16 { Some(s as usize + 16) } else { None }
    }
}

fn handle_synth_step_click(app: &mut App, synth_id: SynthId, step: usize) {
    let now = Instant::now();

    let is_double = if let Some((prev_time, _prev_track, prev_step)) = app.ui.mouse.last_click {
        now.duration_since(prev_time).as_millis() < DOUBLE_CLICK_MS
            && prev_step == step
    } else {
        false
    };

    // Route to the correct synth pattern and UI state
    let (pattern, ui_state, focus) = match synth_id {
        SynthId::A => (&mut app.synth_a_pattern, &mut app.ui.synth_a, FocusSection::SynthAGrid),
        SynthId::B => (&mut app.synth_b_pattern, &mut app.ui.synth_b, FocusSection::SynthBGrid),
    };

    if is_double {
        // Double-click: toggle step
        use crate::sequencer::synth_pattern::SynthStep;
        if pattern.steps[step].is_active() {
            pattern.steps[step] = SynthStep { note: 0, velocity: 0, length: 1 };
        } else {
            let note = 60 + (ui_state.octave as u8).wrapping_sub(4) * 12;
            pattern.steps[step] = SynthStep { note, velocity: 100, length: 1 };
        }
        match synth_id {
            SynthId::A => app.send_synth_pattern(SynthId::A),
            SynthId::B => app.send_synth_pattern(SynthId::B),
        }
        app.dirty = true;
        app.ui.mouse.last_click = None;
        app.ui.mouse.synth_note_drag = None;
    } else {
        // Single click: move cursor + focus
        app.ui.focus = focus;
        ui_state.cursor_step = step;
        app.ui.mouse.last_click = Some((now, 0, step));

        if pattern.steps[step].is_active() {
            app.ui.mouse.synth_note_drag = Some(SynthNoteDrag {
                synth_id,
                step,
                original_length: pattern.steps[step].length,
                start_col: 0,
            });
        } else {
            use crate::sequencer::synth_pattern::SynthStep;
            let note = 60 + (ui_state.octave as u8).wrapping_sub(4) * 12;
            pattern.steps[step] = SynthStep { note, velocity: 100, length: 1 };
            app.send_synth_pattern(synth_id);
            app.dirty = true;
            app.ui.mouse.synth_note_drag = Some(SynthNoteDrag {
                synth_id,
                step,
                original_length: 1,
                start_col: 0,
            });
        }
    }
}

/// Variant that also records the click column for drag calculation.
fn handle_synth_step_click_with_col(app: &mut App, synth_id: SynthId, step: usize, col: u16) {
    handle_synth_step_click(app, synth_id, step);
    if let Some(ref mut drag) = app.ui.mouse.synth_note_drag {
        drag.start_col = col;
    }
}

// ── Synth knobs hit testing ───────────────────────────────────────────────────

use crate::sequencer::synth_pattern::SynthControlField;

// ── Synth knob field groups (must match synth_knobs.rs) ──────────────────────

const OSC1_SLIDERS: &[SynthControlField] = &[
    SynthControlField::Osc1Tune,
    SynthControlField::Osc1Pwm,
    SynthControlField::Osc1Level,
];

const OSC2_SLIDERS: &[SynthControlField] = &[
    SynthControlField::Osc2Tune,
    SynthControlField::Osc2Pwm,
    SynthControlField::Osc2Level,
    SynthControlField::Osc2Detune,
    SynthControlField::SubLevel,
];

const ENV1_ADSR: &[SynthControlField] = &[
    SynthControlField::Env1Attack,
    SynthControlField::Env1Decay,
    SynthControlField::Env1Sustain,
    SynthControlField::Env1Release,
];

const ENV2_ADSR: &[SynthControlField] = &[
    SynthControlField::Env2Attack,
    SynthControlField::Env2Decay,
    SynthControlField::Env2Sustain,
    SynthControlField::Env2Release,
];

const FILT_SLIDERS: &[SynthControlField] = &[
    SynthControlField::FilterCutoff,
    SynthControlField::FilterResonance,
    SynthControlField::FilterEnvAmount,
];

const FILT_ENV_ADSR: &[SynthControlField] = &[
    SynthControlField::FilterEnvAttack,
    SynthControlField::FilterEnvDecay,
    SynthControlField::FilterEnvSustain,
    SynthControlField::FilterEnvRelease,
];

/// Returns SynthControlField if the click is inside the synth knobs panel.
/// Layout must mirror synth_knobs.rs: 3 row groups with percentage-based horizontal splits.
fn hit_test_synth_knobs(col: u16, row: u16, knobs_area: Rect) -> Option<SynthControlField> {
    let inner = Rect::new(
        knobs_area.x + 1,
        knobs_area.y + 1,
        knobs_area.width.saturating_sub(2),
        knobs_area.height.saturating_sub(2),
    );

    if !hit_test_area(col, row, inner) {
        return None;
    }

    // Same vertical split as synth_knobs.rs
    let row_groups = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // OSC1 + OSC2
            Constraint::Length(8),  // ENV1 + ENV2 + FILT
            Constraint::Length(3),  // LFO
            Constraint::Min(7),    // AMP
        ])
        .split(inner);

    // ── Row group 0: OSC1 + OSC2 ────────────────────────────────────────
    if hit_test_area(col, row, row_groups[0]) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Length(2),
                Constraint::Percentage(65),
            ])
            .split(row_groups[0]);

        if hit_test_area(col, row, cols[0]) {
            let local_row = row - cols[0].y;
            return match local_row {
                0 | 1 => Some(SynthControlField::Osc1Waveform),
                _ => hit_slider_field(col, cols[0].x, cols[0].width, OSC1_SLIDERS),
            };
        } else if hit_test_area(col, row, cols[2]) {
            let local_row = row - cols[2].y;
            return match local_row {
                0 | 1 => Some(SynthControlField::Osc2Waveform),
                _ => hit_slider_field(col, cols[2].x, cols[2].width, OSC2_SLIDERS),
            };
        }
        return None;
    }

    // ── Row group 1: ENV1 + ENV2 + FILT ──────────────────────────────────
    if hit_test_area(col, row, row_groups[1]) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length(1),
                Constraint::Percentage(20),
                Constraint::Length(1),
                Constraint::Percentage(60),
            ])
            .split(row_groups[1]);

        if hit_test_area(col, row, cols[0]) {
            return hit_adsr_field(col, cols[0], ENV1_ADSR);
        } else if hit_test_area(col, row, cols[2]) {
            return hit_adsr_field(col, cols[2], ENV2_ADSR);
        } else if hit_test_area(col, row, cols[4]) {
            let local_row = row - cols[4].y;
            return match local_row {
                0 | 1 => Some(SynthControlField::FilterType),
                _ => {
                    let filt_body = Rect::new(
                        cols[4].x,
                        cols[4].y + 2,
                        cols[4].width,
                        cols[4].height.saturating_sub(2),
                    );
                    let filt_split = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(45),
                            Constraint::Percentage(55),
                        ])
                        .split(filt_body);

                    if hit_test_area(col, row, filt_split[0]) {
                        hit_slider_field(col, filt_split[0].x, filt_split[0].width, FILT_SLIDERS)
                    } else {
                        hit_adsr_field(col, filt_split[1], FILT_ENV_ADSR)
                    }
                }
            };
        }
        return None;
    }

    // ── Row group 2: LFO ──────────────────────────────────────────────────
    if hit_test_area(col, row, row_groups[2]) {
        let col_width = row_groups[2].width as usize / 4;
        if col_width > 0 {
            let rel_x = (col - row_groups[2].x) as usize;
            let idx = (rel_x / col_width).min(3);
            return match idx {
                0 => Some(SynthControlField::LfoWaveform),
                1 => Some(SynthControlField::LfoDivision),
                2 => Some(SynthControlField::LfoDepth),
                _ => Some(SynthControlField::LfoDest),
            };
        }
        return Some(SynthControlField::LfoWaveform);
    }

    // ── Row group 3: AMP ─────────────────────────────────────────────────
    if hit_test_area(col, row, row_groups[3]) {
        let amp_w = row_groups[3].width.min(40);
        let amp_body = Rect::new(
            row_groups[3].x,
            row_groups[3].y + 1,
            amp_w,
            row_groups[3].height.saturating_sub(1),
        );
        if hit_test_area(col, row, amp_body) {
            let col_width = amp_w as usize / 4;
            if col_width > 0 {
                let rel_x = (col - amp_body.x) as usize;
                let idx = (rel_x / col_width).min(2);
                return match idx {
                    0 => Some(SynthControlField::Volume),
                    1 => Some(SynthControlField::SendReverb),
                    _ => Some(SynthControlField::SendDelay),
                };
            }
        }
        return Some(SynthControlField::Volume);
    }

    None
}

/// Hit-test within a slider group: fields evenly distributed across the width.
fn hit_slider_field(col: u16, area_x: u16, area_w: u16, fields: &[SynthControlField]) -> Option<SynthControlField> {
    if fields.is_empty() { return None; }
    let col_width = area_w as usize / fields.len().max(1);
    if col_width == 0 { return fields.first().copied(); }
    let rel_x = (col - area_x) as usize;
    let idx = (rel_x / col_width).min(fields.len() - 1);
    Some(fields[idx])
}

/// Hit-test within ADSR bars: 4 bars, 2-char wide, 1-char gap, centered in area.
fn hit_adsr_field(col: u16, area: Rect, fields: &[SynthControlField]) -> Option<SynthControlField> {
    if fields.len() != 4 { return None; }
    let bar_width: usize = 2;
    let gap: usize = 1;
    let group_width = 4 * bar_width + 3 * gap; // 11 chars
    let left_pad = ((area.width as usize).saturating_sub(group_width)) / 2;
    let rel_x = (col - area.x) as usize;
    if rel_x < left_pad { return Some(fields[0]); }
    let adj_x = rel_x - left_pad;
    // Bars at: 0-1, 3-4, 6-7, 9-10 (with gaps at 2, 5, 8)
    let idx = if adj_x >= 9 { 3 } else if adj_x >= 6 { 2 } else if adj_x >= 3 { 1 } else { 0 };
    Some(fields[idx])
}

fn handle_synth_knobs_click(app: &mut App, synth_id: SynthId, field: SynthControlField, start_y: u16) {
    let (pattern, ui_state, focus) = match synth_id {
        SynthId::A => (&mut app.synth_a_pattern, &mut app.ui.synth_a, FocusSection::SynthAControls),
        SynthId::B => (&mut app.synth_b_pattern, &mut app.ui.synth_b, FocusSection::SynthBControls),
    };

    app.ui.focus = focus;
    ui_state.ctrl_field = field;

    // For enum fields, just cycle on click instead of drag
    if field.is_enum() {
        let max_val: u8 = if field == SynthControlField::FilterType { 2 } else { 3 };
        let cur = field.get(&pattern.params);
        let cur_int = (cur * max_val as f32).round() as u8;
        let new_int = (cur_int + 1) % (max_val + 1);
        field.set(&mut pattern.params, new_int as f32 / max_val as f32);
        match synth_id {
            SynthId::A => app.send_synth_pattern(SynthId::A),
            SynthId::B => app.send_synth_pattern(SynthId::B),
        }
        app.dirty = true;
        return;
    }

    // Start drag for continuous params
    let start_value = field.get(&pattern.params);
    app.ui.mouse.synth_drag = Some(SynthDrag {
        synth_id,
        field,
        start_y,
        start_value,
    });
}

// ── Grid step hit testing ────────────────────────────────────────────────────

/// Returns (track, step) if the click is on a drum grid cell.
/// Updated for new spaced-square grid format: NAME_WIDTH=9, header+spacer before tracks.
fn hit_test_grid_step(col: u16, row: u16, grid_area: Rect) -> Option<(usize, usize)> {
    // Inner area (inside Thick block border)
    let inner_x = grid_area.x + 1;
    let inner_y = grid_area.y + 1;

    // Track rows start after header (row 0) + spacer (row 1)
    let track_y_start = inner_y + 2;
    if row < track_y_start || row >= track_y_start + NUM_DRUM_TRACKS as u16 {
        return None;
    }
    let track = (row - track_y_start) as usize;

    // Steps start after track name column (NAME_WIDTH = 9)
    let steps_x_start = inner_x + 9;

    // Steps 0-15: positions steps_x_start + step*2 (each 2 chars: symbol + space)
    // Bar separator ┃ at steps_x_start + 32
    // Steps 16-31: positions steps_x_start + 33 + (step-16)*2

    let rel_x = col.checked_sub(steps_x_start)?;

    let step = if rel_x < 32 {
        // First half (steps 0-15)
        let s = rel_x / 2;
        if s < 16 { Some(s as usize) } else { None }
    } else if rel_x == 32 {
        // Bar separator
        None
    } else {
        // Second half (steps 16-31): starts at rel_x 33
        let s = (rel_x - 33) / 2;
        if s < 16 { Some(s as usize + 16) } else { None }
    };

    step.map(|s| (track, s))
}

fn handle_grid_click(app: &mut App, track: usize, step: usize) {
    let now = Instant::now();

    // Check for double-click
    let is_double = if let Some((prev_time, prev_track, prev_step)) = app.ui.mouse.last_click {
        now.duration_since(prev_time).as_millis() < DOUBLE_CLICK_MS
            && prev_track == track
            && prev_step == step
    } else {
        false
    };

    if is_double {
        // Double-click: toggle step
        app.drum_pattern.steps[track][step] = !app.drum_pattern.steps[track][step];
        app.send_drum_pattern();
        app.dirty = true;
        app.ui.mouse.last_click = None; // reset so triple-click doesn't re-toggle
    } else {
        // Single click: move cursor
        app.ui.focus = FocusSection::DrumGrid;
        app.ui.drum_cursor_track = track;
        app.ui.drum_cursor_step = step;
        app.ui.drum_ctrl_track = track;
        app.ui.mouse.last_click = Some((now, track, step));
    }
}

// ── Inline controls hit testing ──────────────────────────────────────────────

/// Returns (track, is_mute) if the click is on a [M] or [S] button in the drum grid.
/// Layout: name(9) + 16×2 steps + bar_sep(1) + 16×2 steps + " [M] [S]"
fn hit_test_mute_solo(
    col: u16,
    row: u16,
    grid_area: Rect,
) -> Option<(usize, bool)> {
    let inner_x = grid_area.x + 1;
    let inner_y = grid_area.y + 1;

    // Track rows start after header (row 0) + spacer (row 1)
    let track_y_start = inner_y + 2;
    if row < track_y_start || row >= track_y_start + NUM_DRUM_TRACKS as u16 {
        return None;
    }
    let track = (row - track_y_start) as usize;

    // [M] starts at: name(9) + 32 steps + bar(1) + 32 steps + space(1) = 75
    let ms_start = inner_x + 75;
    if col < ms_start {
        return None;
    }
    let rel_x = (col - ms_start) as usize;

    // "[M] [S]" = positions 0..3 = [M], 3 = space, 4..7 = [S]
    match rel_x {
        0..3 => Some((track, true)),   // Mute
        4..7 => Some((track, false)),  // Solo
        _ => None,
    }
}

/// Handle click on [M] or [S] button.
fn handle_mute_solo_click(app: &mut App, track: usize, is_mute: bool) {
    if is_mute {
        app.drum_pattern.params[track].mute = !app.drum_pattern.params[track].mute;
    } else {
        app.drum_pattern.params[track].solo = !app.drum_pattern.params[track].solo;
    }
    app.send_drum_pattern();
}

// ── Knobs panel hit testing ──────────────────────────────────────────────────

/// Returns the DrumControlField if the click is inside the knobs panel.
/// The knobs panel renders 10 slider columns side by side (label + 5 bars + value).
/// Clicking anywhere within a column selects that parameter.
fn hit_test_knobs_panel(col: u16, row: u16, knobs_area: Rect) -> Option<DrumControlField> {
    // Inner area (inside Thick block border)
    let inner_x = knobs_area.x + 1;
    let inner_y = knobs_area.y + 1;
    let inner_w = knobs_area.width.saturating_sub(2);
    let inner_h = knobs_area.height.saturating_sub(2);

    if col < inner_x || col >= inner_x + inner_w {
        return None;
    }
    if row < inner_y || row >= inner_y + inner_h {
        return None;
    }

    let rel_col = (col - inner_x) as usize;

    let num_knobs = KNOB_FIELDS.len();
    let col_width = if inner_w as usize >= num_knobs { (inner_w as usize) / num_knobs } else { return None };
    let slider_area_width = col_width * num_knobs;

    if rel_col >= slider_area_width {
        return None;
    }

    let col_idx = (rel_col / col_width).min(num_knobs - 1);
    KNOB_FIELDS.get(col_idx).copied()
}

fn handle_knobs_click(app: &mut App, field: DrumControlField, start_y: u16) {
    let track = app.ui.drum_ctrl_track;
    app.ui.focus = FocusSection::Knobs;
    app.ui.drum_ctrl_field = field;

    // Start drag
    let start_value = get_param_value(&app.drum_pattern.params[track], field);
    app.ui.mouse.drag = Some(DragState {
        track,
        field,
        start_y,
        start_value,
    });
}

fn get_param_value(params: &crate::sequencer::drum_pattern::DrumTrackParams, field: DrumControlField) -> f32 {
    match field {
        DrumControlField::Tune => params.tune,
        DrumControlField::Sweep => params.sweep,
        DrumControlField::Color => params.color,
        DrumControlField::Snap => params.snap,
        DrumControlField::Filter => params.filter,
        DrumControlField::Drive => params.drive,
        DrumControlField::Decay => params.decay,
        DrumControlField::Volume => params.volume,
        DrumControlField::SendReverb => params.send_reverb,
        DrumControlField::SendDelay => params.send_delay,
        DrumControlField::Pan => params.pan,
        DrumControlField::Mute | DrumControlField::Solo => 0.0,
    }
}

fn set_param_value(params: &mut crate::sequencer::drum_pattern::DrumTrackParams, field: DrumControlField, value: f32) {
    let v = value.clamp(0.0, 1.0);
    match field {
        DrumControlField::Tune => params.tune = v,
        DrumControlField::Sweep => params.sweep = v,
        DrumControlField::Color => params.color = v,
        DrumControlField::Snap => params.snap = v,
        DrumControlField::Filter => params.filter = v,
        DrumControlField::Drive => params.drive = v,
        DrumControlField::Decay => params.decay = v,
        DrumControlField::Volume => params.volume = v,
        DrumControlField::SendReverb => params.send_reverb = v,
        DrumControlField::SendDelay => params.send_delay = v,
        DrumControlField::Pan => params.pan = v,
        DrumControlField::Mute | DrumControlField::Solo => {}
    }
}

// ── Drag handling ────────────────────────────────────────────────────────────

fn handle_drag(app: &mut App, col: u16, row: u16, _term_size: Rect) {
    // Check if dragging a synth note (horizontal resize)
    if let Some(ref drag) = app.ui.mouse.synth_note_drag {
        let drag = drag.clone();
        let col_delta = col as i32 - drag.start_col as i32;
        let step_delta = col_delta / 2;
        let new_length = (drag.original_length as i32 + step_delta).clamp(1, 32) as u8;
        let loop_len = match drag.synth_id {
            SynthId::A => app.transport.loop_config.synth_a_length as usize,
            SynthId::B => app.transport.loop_config.synth_b_length as usize,
        };
        let max_length = (loop_len - drag.step).min(32) as u8;
        let clamped = new_length.min(max_length).max(1);
        let pattern = match drag.synth_id {
            SynthId::A => &mut app.synth_a_pattern,
            SynthId::B => &mut app.synth_b_pattern,
        };
        if pattern.steps[drag.step].length != clamped {
            pattern.steps[drag.step].length = clamped;
            app.send_synth_pattern(drag.synth_id);
            app.dirty = true;
        }
        return;
    }

    // Check if dragging the compressor knob
    if app.ui.mouse.compressor_drag.is_some() {
        handle_compressor_drag(app, row);
        return;
    }

    // Check if dragging a volume fader
    if app.ui.mouse.fader_drag.is_some() {
        handle_fader_drag(app, row);
        return;
    }

    // Check if dragging a synth knob
    if let Some(ref d) = app.ui.mouse.synth_drag {
        let d = d.clone();
        let delta_y = d.start_y as f32 - row as f32;
        let new_value = (d.start_value + delta_y * DRAG_SENSITIVITY).clamp(0.0, 1.0);
        let pattern = match d.synth_id {
            SynthId::A => &mut app.synth_a_pattern,
            SynthId::B => &mut app.synth_b_pattern,
        };
        d.field.set(&mut pattern.params, new_value);
        match d.synth_id {
            SynthId::A => app.send_synth_pattern(SynthId::A),
            SynthId::B => app.send_synth_pattern(SynthId::B),
        }
        app.dirty = true;
        return;
    }

    let drag = match app.ui.mouse.drag {
        Some(ref d) => d.clone(),
        None => return,
    };

    // Up = increase (row decreases), down = decrease (row increases)
    let delta_y = drag.start_y as f32 - row as f32;
    let new_value = (drag.start_value + delta_y * DRAG_SENSITIVITY).clamp(0.0, 1.0);

    set_param_value(&mut app.drum_pattern.params[drag.track], drag.field, new_value);
    app.send_drum_pattern();
    app.dirty = true;
}

// ── Activity bar pad hit testing ─────────────────────────────────────────────

/// Returns track index if the click is on an activity bar pad.
fn hit_test_activity_pad(col: u16, row: u16, activity_area: Rect) -> Option<usize> {
    if row != activity_area.y {
        return None;
    }

    // Layout: " [Kick] [Snare] [CHH] [OHH] [Ride] [Clap] [Cowbell] [Tom] "
    // Start at x+1 (leading space)
    let mut x = activity_area.x + 1;
    for track in 0..NUM_DRUM_TRACKS {
        let name = TRACK_IDS[track].name();
        let pad_width = name.len() as u16 + 2; // brackets: [Name]
        if col >= x && col < x + pad_width {
            return Some(track);
        }
        x += pad_width + 1; // +1 for trailing space
    }
    None
}

fn handle_pad_click(app: &mut App, track: usize) {
    let _ = app.tx_to_audio.send(UiToAudio::TriggerDrum(TRACK_IDS[track]));
    app.ui.flash_track(track);
}

// ── Compressor gauge hit testing ──────────────────────────────────────────────

/// Check if click is on the compressor gauge in transport bar line 1.
/// Layout: "▶ PLAY   BPM: 120.0  CMP:████  Loop: ..."
/// Inside border (area.x+1), line 1 is at area.y + 1.
fn hit_test_compressor_gauge(col: u16, row: u16, transport_area: Rect) -> bool {
    let line_y = transport_area.y + 1;
    if row != line_y {
        return false;
    }
    let inner_x = transport_area.x + 1;
    // "▶ PLAY   BPM: 120.0  CMP:" = play(8) + bpm(~14) + "  CMP:" = varies
    // The CMP: label starts after play_icon + bpm_span
    // play_icon: "▶ PLAY " = 8 chars (with trailing space from format)
    // bpm_span: "  BPM: XXX.X" = ~12 chars
    // comp_span: "  CMP:XXXX" = 10 chars
    // Approximate: search for the CMP region
    // play(8) + bpm(12) + "  CMP:" (6) = offset 26, gauge is 4 chars
    let cmp_start = inner_x + 20; // "▶ PLAY " (8) + "  BPM: 120.0" (12) = 20
    let cmp_end = cmp_start + 10; // "  CMP:████" = 10 chars
    col >= cmp_start && col < cmp_end
}

fn handle_compressor_click(app: &mut App, start_y: u16) {
    app.ui.mouse.compressor_drag = Some(CompressorDrag {
        start_y,
        start_value: app.effect_params.compressor_amount,
    });
}

fn handle_compressor_drag(app: &mut App, row: u16) {
    let drag = match app.ui.mouse.compressor_drag {
        Some(ref d) => d.clone(),
        None => return,
    };
    let delta_y = drag.start_y as f32 - row as f32;
    let new_value = (drag.start_value + delta_y * DRAG_SENSITIVITY).clamp(0.0, 1.0);
    app.effect_params.compressor_amount = new_value;
    app.send_effect_params();
    app.dirty = true;
}

// ── Transport bar hit testing ────────────────────────────────────────────────

/// Task 12: Check if click is on the play/stop button area (first ~10 columns of transport content row 0).
/// Transport has Thick borders, so content row 0 is at area.y + 1, starting at area.x + 1.
fn hit_test_play_button(col: u16, row: u16, transport_area: Rect) -> bool {
    let content_y = transport_area.y + 1;
    let inner_x = transport_area.x + 1;
    row == content_y && col >= inner_x && col < inner_x + 10
}

/// Task 12: Check if click is on the record button area (approx. columns near "REC" label on content row 0).
/// The REC indicator is rendered near the end of the first line, roughly last ~5 columns before border.
fn hit_test_record_button(col: u16, row: u16, transport_area: Rect) -> bool {
    let content_y = transport_area.y + 1;
    let right_edge = transport_area.x + transport_area.width - 1; // inside right border
    // REC label is about 5 chars from the right edge
    row == content_y && col >= right_edge.saturating_sub(5) && col < right_edge
}

/// Returns (pattern_index, is_synth_row) if the click is on a pattern selector key.
/// Transport lines 2-3 have two rows of machine selectors:
///   Line 2 (area.y + 2): Synth | Pattern: q w e ... | Kit: 1 2 ...
///   Line 3 (area.y + 3): Drum  | Pattern: q w e ... | Kit: 1 2 ...
fn hit_test_pattern_selector(col: u16, row: u16, transport_area: Rect) -> Option<(usize, bool)> {
    let synth_line_y = transport_area.y + 2;
    let drum_line_y = transport_area.y + 3;

    let is_synth = if row == synth_line_y {
        true
    } else if row == drum_line_y {
        false
    } else {
        return None;
    };

    // Row layout: "Synth │  Pat: q w e r t y u i o p  │  Kit: 1 2 ..."
    // "Synth " (6) + "│" (1) + "  Pat: " (7) = prefix 14
    let inner_x = transport_area.x + 1;
    let pat_start = inner_x + 14;

    // Each pattern key: 1 char key + 1 space = 2 chars
    if col >= pat_start && col < pat_start + (NUM_PATTERNS as u16) * 2 {
        let idx = ((col - pat_start) / 2) as usize;
        if idx < NUM_PATTERNS {
            return Some((idx, is_synth));
        }
    }
    None
}

// ── Volume fader hit testing ─────────────────────────────────────────────────

/// Check if click is within a fader area.
/// (Currently unused — faders not in DualSynthLayout yet, kept for future re-use.)
#[allow(dead_code)]
fn hit_test_fader(col: u16, row: u16, fader_area: Rect) -> bool {
    col >= fader_area.x
        && col < fader_area.x + fader_area.width
        && row >= fader_area.y
        && row < fader_area.y + fader_area.height
}

/// Convert a click row to a volume value (0.0 at bottom, 1.0 at top).
/// (Currently unused — faders not in DualSynthLayout yet, kept for future re-use.)
#[allow(dead_code)]
fn fader_value_from_click(row: u16, fader_area: Rect) -> f32 {
    // Inner area (inside border)
    let inner_top = fader_area.y + 1;
    let inner_height = fader_area.height.saturating_sub(2);
    if inner_height == 0 {
        return 0.5;
    }
    let rel = row.saturating_sub(inner_top) as f32;
    // Invert: top = 1.0, bottom = 0.0
    (1.0 - rel / inner_height as f32).clamp(0.0, 1.0)
}

fn handle_fader_drag(app: &mut App, row: u16) {
    use crate::app::FaderKind;

    let drag = match app.ui.mouse.fader_drag {
        Some(ref d) => d.clone(),
        None => return,
    };

    let delta_y = drag.start_y as f32 - row as f32;
    let new_value = (drag.start_value + delta_y * DRAG_SENSITIVITY).clamp(0.0, 1.0);

    match drag.kind {
        FaderKind::Drum => {
            app.effect_params.drum_volume = new_value;
            app.send_effect_params();
        }
        FaderKind::Synth => {
            app.synth_a_pattern.params.volume = new_value;
            app.send_synth_pattern(SynthId::A);
        }
    }
    app.dirty = true;
}

/// Returns (kit_index, is_synth_row) if the click is on a kit selector number.
/// Transport lines 2-3 have two rows of machine selectors:
///   Line 2 (area.y + 2): Synth | Pattern: q w e ... | Kit: 1 2 ...
///   Line 3 (area.y + 3): Drum  | Pattern: q w e ... | Kit: 1 2 ...
fn hit_test_kit_selector(col: u16, row: u16, transport_area: Rect) -> Option<(usize, bool)> {
    let synth_line_y = transport_area.y + 2;
    let drum_line_y = transport_area.y + 3;

    let is_synth = if row == synth_line_y {
        true
    } else if row == drum_line_y {
        false
    } else {
        return None;
    };

    let inner_x = transport_area.x + 1;
    // "Synth " (6) + "│" (1) + "  Pat: " (7) + 10 patterns * 2 (20) + " │" (2) + "  Kit: " (7) = 43
    let kit_start = inner_x + 14 + 20 + 9;

    // Each kit: 1 char + 1 space = 2 chars
    if col >= kit_start && col < kit_start + (NUM_KITS as u16) * 2 {
        let idx = ((col - kit_start) / 2) as usize;
        if idx < NUM_KITS {
            return Some((idx, is_synth));
        }
    }
    None
}
