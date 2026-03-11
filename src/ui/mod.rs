//! Top-level render dispatch: computes layout, renders all sections, handles modals.

pub mod drum_grid;
pub mod help_overlay;
pub mod knobs;
pub mod layout;
pub mod splash;
pub mod synth_grid;
pub mod synth_knobs;
pub mod theme;
pub mod transport_bar;
pub mod waveform;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use layout::*;

use crate::app::{App, DrumControlField, FocusSection, ModalState, SplashPhase};
use crate::messages::SynthId;
use crate::presets::PresetTarget;
use crate::sequencer::drum_pattern::{NUM_DRUM_TRACKS, TRACK_IDS};

/// Render a collapsed panel bar with [.] indicator showing it can be expanded.
fn render_collapsed_bar(f: &mut Frame, area: Rect, label: &str, focused: bool) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let style = if focused {
        Style::default().fg(theme::CYAN)
    } else {
        Style::default().fg(theme::DIM_TEXT)
    };
    let block = Block::default()
        .borders(Borders::TOP)
        .title(format!("[.] {}", label))
        .title_style(style);
    f.render_widget(block, area);
}

/// Top-level render function.
pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Splash screen: logo phases take over entirely
    if matches!(app.ui.splash.phase, SplashPhase::SlideIn | SplashPhase::Hold) {
        splash::render_splash(f, size, &app.ui.splash);
        return;
    }

    // Matrix reveal: render real UI first, then overlay matrix rain on unrevealed cells
    let matrix_active = app.ui.splash.phase == SplashPhase::MatrixReveal;

    // Compute dual-synth layout from panel visibility
    let ly = compute_dual_layout(size, &app.ui.panel_vis);

    // ── Transport ────────────────────────────────────────────────
    transport_bar::render_transport(f, ly.transport, app);

    // ── Synth A Knobs ────────────────────────────────────────────
    if app.ui.panel_vis.synth_a_knobs {
        synth_knobs::render_synth_knobs(f, ly.synth_a_knobs, app, SynthId::A);
    } else {
        let focused = matches!(app.ui.focus, FocusSection::SynthAControls);
        render_collapsed_bar(f, ly.synth_a_knobs_collapsed, "SYNTH A KNOBS", focused);
    }

    // ── Synth A Grid ─────────────────────────────────────────────
    if app.ui.panel_vis.synth_a_grid {
        synth_grid::render_synth_grid(f, ly.synth_a_grid, app, SynthId::A);
    } else {
        let focused = matches!(app.ui.focus, FocusSection::SynthAGrid);
        render_collapsed_bar(f, ly.synth_a_grid_collapsed, "SYNTH A GRID", focused);
    }

    // ── Synth B Knobs ────────────────────────────────────────────
    if app.ui.panel_vis.synth_b_knobs {
        synth_knobs::render_synth_knobs(f, ly.synth_b_knobs, app, SynthId::B);
    } else {
        let focused = matches!(app.ui.focus, FocusSection::SynthBControls);
        render_collapsed_bar(f, ly.synth_b_knobs_collapsed, "SYNTH B KNOBS", focused);
    }

    // ── Synth B Grid ─────────────────────────────────────────────
    if app.ui.panel_vis.synth_b_grid {
        synth_grid::render_synth_grid(f, ly.synth_b_grid, app, SynthId::B);
    } else {
        let focused = matches!(app.ui.focus, FocusSection::SynthBGrid);
        render_collapsed_bar(f, ly.synth_b_grid_collapsed, "SYNTH B GRID", focused);
    }

    // ── Drum Grid ────────────────────────────────────────────────
    if app.ui.panel_vis.drum_grid {
        drum_grid::render_drum_grid(f, ly.drum_grid, app);
    }

    // ── Drum Knobs ───────────────────────────────────────────────
    if app.ui.panel_vis.drum_knobs {
        knobs::render_knobs(f, ly.drum_knobs, app);
    } else {
        let focused = matches!(app.ui.focus, FocusSection::Knobs);
        render_collapsed_bar(f, ly.drum_knobs_collapsed, "DRUM KNOBS", focused);
    }

    // ── Waveform ─────────────────────────────────────────────────
    if app.ui.panel_vis.waveform {
        let wave_area = ly.waveform;
        let wave_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),  // VU meter
                Constraint::Min(20),   // Oscilloscope
            ])
            .split(wave_area);
        waveform::render_vu_meter(f, wave_chunks[0], &app.display_buf);
        waveform::render_scope_bars(f, wave_chunks[1], &app.ui.scope_bars, &app.ui.scope_intensity);
    } else {
        render_collapsed_bar(f, ly.waveform_collapsed, "WAVEFORM", false);
    }

    // ── Activity bar ─────────────────────────────────────────────
    render_activity_bar(f, ly.activity_bar, app);

    // ── Help overlay (rendered on top, like a modal) ─────────────
    if app.ui.show_help {
        // Render help as a centered overlay
        let help_h = HELP_HEIGHT.min(size.height.saturating_sub(2));
        let help_y = size.y + (size.height.saturating_sub(help_h)) / 2;
        let help_area = Rect::new(size.x, help_y, size.width, help_h);
        f.render_widget(Clear, help_area);
        help_overlay::render_help(f, help_area);
    }

    // Matrix rain overlay (covers unrevealed cells)
    if matrix_active {
        splash::render_splash(f, size, &app.ui.splash);
    }

    // Modal dialogs (always on top)
    match &app.ui.modal {
        ModalState::TextInput { prompt, buffer, .. } => {
            render_text_input(f, size, prompt, buffer);
        }
        ModalState::FilePicker { title, items, selected, .. } => {
            render_file_picker(f, size, title, items, *selected);
        }
        ModalState::PresetBrowser(browser) => {
            render_preset_browser(f, size, browser);
        }
        ModalState::PatternBrowser(pb) => {
            render_pattern_browser(f, size, pb);
        }
        ModalState::None => {}
    }
}

// ── Activity bar ─────────────────────────────────────────────────────────────

/// Activity bar: trigger pads + param tweak + status message.
fn render_activity_bar(f: &mut Frame, area: Rect, app: &App) {
    // If there's a status message, show it instead
    if let Some(ref msg) = app.ui.status_msg {
        let line = Line::from(Span::styled(
            format!(" {}", msg.text),
            Style::default().fg(theme::GOLD).add_modifier(Modifier::BOLD),
        ));
        f.render_widget(Paragraph::new(line), area);
        return;
    }

    let mut spans: Vec<Span> = vec![Span::raw(" ")];

    // Drum pad indicators
    for track in 0..NUM_DRUM_TRACKS {
        let name = TRACK_IDS[track].name();
        let flashing = app.ui.trigger_flash[track] > 0;

        let style = if flashing {
            Style::default()
                .fg(theme::TEXT)
                .bg(theme::AMBER)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DIM_TEXT)
        };

        spans.push(Span::styled(format!("[{}]", name), style));
        spans.push(Span::raw(" "));
    }

    // Current parameter tweak display
    if app.ui.focus == FocusSection::Knobs {
        let t = app.ui.drum_ctrl_track;
        let params = &app.drum_pattern.params[t];
        let (label, value) = match app.ui.drum_ctrl_field {
            DrumControlField::Tune   => ("Tune",   params.tune),
            DrumControlField::Sweep  => ("Sweep",  params.sweep),
            DrumControlField::Color  => ("Color",  params.color),
            DrumControlField::Snap   => ("Snap",   params.snap),
            DrumControlField::Filter => ("Filter", params.filter),
            DrumControlField::Drive  => ("Drive",  params.drive),
            DrumControlField::Decay  => ("Decay",  params.decay),
            DrumControlField::Volume => ("Volume", params.volume),
            DrumControlField::SendReverb => ("Reverb Send", params.send_reverb),
            DrumControlField::SendDelay  => ("Delay Send",  params.send_delay),
            DrumControlField::Pan    => ("Pan",    params.pan),
            DrumControlField::Mute   => ("Mute",   if params.mute { 1.0 } else { 0.0 }),
            DrumControlField::Solo   => ("Solo",   if params.solo { 1.0 } else { 0.0 }),
        };

        let track_name = TRACK_IDS[t].name();
        spans.push(Span::styled(" \u{2502} ", Style::default().fg(theme::BORDER)));
        spans.push(Span::styled(
            format!("{} ", track_name),
            Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{}: {:.2}", label, value),
            Style::default().fg(theme::AMBER),
        ));
    }

    // Help hint at the end
    spans.push(Span::styled(" \u{2502} ", Style::default().fg(theme::BORDER)));
    spans.push(Span::styled("? Help", Style::default().fg(theme::DIM_TEXT)));

    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}

/// Render a centered text input modal.
fn render_text_input(f: &mut Frame, area: Rect, prompt: &str, buffer: &str) {
    let w = 50u16.min(area.width.saturating_sub(4));
    let h = 5u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect::new(x, y, w, h);

    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(format!(" {} ", prompt))
        .title_style(Style::default().fg(theme::AMBER).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::AMBER));

    let cursor = "\u{2588}"; // block cursor
    let text = format!(" {}{}", buffer, cursor);
    let lines = vec![
        Line::from(Span::styled(text, Style::default().fg(theme::TEXT))),
        Line::from(Span::styled(
            " [Enter] Confirm  [Esc] Cancel",
            Style::default().fg(theme::DIM_TEXT),
        )),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup);
}

/// Render a centered file picker modal.
fn render_file_picker(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &[(String, std::path::PathBuf)],
    selected: usize,
) {
    let item_count = items.len().min(12);
    let w = 50u16.min(area.width.saturating_sub(4));
    let h = (item_count as u16 + 4).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect::new(x, y, w, h);

    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(theme::AMBER).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::AMBER));

    let mut lines: Vec<Line> = Vec::new();

    for (i, (name, _path)) in items.iter().enumerate() {
        if i >= item_count { break; }
        let is_sel = i == selected;
        let prefix = if is_sel { " > " } else { "   " };
        let style = if is_sel {
            Style::default()
                .fg(Color::Black)
                .bg(theme::CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT)
        };
        lines.push(Line::from(Span::styled(format!("{}{}", prefix, name), style)));
    }

    lines.push(Line::from(Span::styled(
        " [Enter] Load  [Esc] Cancel",
        Style::default().fg(theme::DIM_TEXT),
    )));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup);
}

/// Build a coverflow-style category strip: selected item centered, neighbors visible.
fn coverflow_categories<'a>(
    categories: &[&'a str],
    selected: usize,
    avail_width: usize,
    sel_color: Color,
) -> Line<'a> {
    if categories.is_empty() {
        return Line::from(Span::raw(""));
    }

    let sep_str = " \u{2502} ";
    let sep_w = 3; // " │ "

    // Build each label with padding: " Label "
    let labels: Vec<String> = categories.iter().map(|c| format!(" {} ", c)).collect();
    let label_widths: Vec<usize> = labels.iter().map(|l| l.len()).collect();

    // Calculate total width needed for all categories
    let sel_label_w = label_widths[selected];

    // Budget: available width minus the selected label, arrows, and padding
    let arrow_w = 4; // " < " or " > "
    let budget = avail_width.saturating_sub(sel_label_w + arrow_w * 2);

    // Expand outward from selected, alternating left and right
    let mut left_start = selected;
    let mut right_end = selected; // inclusive
    let mut used = 0usize;

    let mut try_left = selected > 0;
    let mut try_right = selected + 1 < categories.len();

    loop {
        let mut progress = false;

        if try_left && left_start > 0 {
            let cost = label_widths[left_start - 1] + sep_w;
            if used + cost <= budget {
                left_start -= 1;
                used += cost;
                progress = true;
            } else {
                try_left = false;
            }
        }

        if try_right && right_end + 1 < categories.len() {
            let cost = label_widths[right_end + 1] + sep_w;
            if used + cost <= budget {
                right_end += 1;
                used += cost;
                progress = true;
            } else {
                try_right = false;
            }
        }

        if !progress { break; }
    }

    let mut spans: Vec<Span> = Vec::new();

    // Left arrow if truncated
    if left_start > 0 {
        spans.push(Span::styled(" \u{25c0} ", Style::default().fg(theme::DIM_TEXT)));
    } else {
        spans.push(Span::raw("   "));
    }

    // Render visible categories
    for i in left_start..=right_end {
        let is_sel = i == selected;
        let dist = if i > selected { i - selected } else { selected - i };

        let style = if is_sel {
            Style::default().fg(Color::Black).bg(sel_color).add_modifier(Modifier::BOLD)
        } else if dist == 1 {
            Style::default().fg(theme::TEXT)
        } else {
            Style::default().fg(theme::DIM_TEXT)
        };

        spans.push(Span::styled(labels[i].clone(), style));

        if i < right_end {
            spans.push(Span::styled(sep_str, Style::default().fg(Color::Rgb(50, 50, 50))));
        }
    }

    // Right arrow if truncated
    if right_end + 1 < categories.len() {
        spans.push(Span::styled(" \u{25b6} ", Style::default().fg(theme::DIM_TEXT)));
    }

    Line::from(spans)
}

/// Render a centered preset browser modal.
fn render_preset_browser(
    f: &mut Frame,
    area: Rect,
    browser: &crate::presets::PresetBrowserState,
) {
    let title = match &browser.target {
        PresetTarget::DrumSound(track) => {
            let name = TRACK_IDS[*track].name();
            format!(" {} Presets ", name)
        }
        PresetTarget::SynthSound => " Synth Presets ".to_string(),
        PresetTarget::Pattern => " Pattern Presets ".to_string(),
        PresetTarget::SynthPattern => " Synth Patterns ".to_string(),
    };

    let max_items = 14usize;
    let visible_presets = browser.preset_names.len().min(max_items);
    let w = 70u16.min(area.width.saturating_sub(4));
    let h = (visible_presets as u16 + 6).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect::new(x, y, w, h);

    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(theme::PINK).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PINK));

    let mut lines: Vec<Line> = Vec::new();

    // Coverflow category strip
    lines.push(coverflow_categories(
        &browser.categories, browser.category_idx, (w as usize).saturating_sub(2), theme::PINK,
    ));
    lines.push(Line::from(Span::styled(
        " \u{2500}".to_string() + &"\u{2500}".repeat((w as usize).saturating_sub(4)),
        Style::default().fg(Color::Rgb(50, 50, 50)),
    )));

    // Scrolling window around the selected preset
    let scroll_offset = if browser.preset_idx >= max_items {
        browser.preset_idx - max_items + 1
    } else {
        0
    };

    // Preset list
    if browser.preset_names.is_empty() {
        lines.push(Line::from(Span::styled(
            "   (no presets)",
            Style::default().fg(theme::DIM_TEXT),
        )));
    } else {
        for (i, name) in browser.preset_names.iter().enumerate().skip(scroll_offset).take(max_items) {
            let is_sel = i == browser.preset_idx;
            let prefix = if is_sel { " \u{25b6} " } else { "   " };
            let style = if is_sel {
                Style::default().fg(Color::Black).bg(theme::CYAN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };
            lines.push(Line::from(Span::styled(format!("{}{}", prefix, name), style)));
        }
    }

    // Footer
    lines.push(Line::from(Span::styled(
        " \u{2500}".to_string() + &"\u{2500}".repeat((w as usize).saturating_sub(4)),
        Style::default().fg(Color::Rgb(50, 50, 50)),
    )));
    lines.push(Line::from(vec![
        Span::styled(" \u{2191}\u{2193}", Style::default().fg(theme::CYAN)),
        Span::styled(" Browse ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("\u{2190}\u{2192}", Style::default().fg(theme::PINK)),
        Span::styled(" Category ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("Spc", Style::default().fg(theme::AMBER)),
        Span::styled(" Preview ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("\u{23ce}", Style::default().fg(Color::Green)),
        Span::styled(" Load ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::styled(" Close", Style::default().fg(theme::DIM_TEXT)),
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup);
}

/// Render a centered pattern browser modal.
fn render_pattern_browser(
    f: &mut Frame,
    area: Rect,
    pb: &crate::presets::PatternBrowserState,
) {
    use crate::presets::PatternMergeMode;

    let browser = &pb.browser;
    let mode_label = match pb.merge_mode {
        PatternMergeMode::Replace => "REPLACE",
        PatternMergeMode::Layer => "LAYER",
    };
    let title = format!(" Pattern Presets [{}] ", mode_label);

    let max_items = 14usize;
    let visible_presets = browser.preset_names.len().min(max_items);
    let w = 70u16.min(area.width.saturating_sub(4));
    let h = (visible_presets as u16 + 6).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect::new(x, y, w, h);

    f.render_widget(Clear, popup);

    let mode_color = match pb.merge_mode {
        PatternMergeMode::Replace => Color::Yellow,
        PatternMergeMode::Layer => Color::Green,
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(mode_color));

    let mut lines: Vec<Line> = Vec::new();

    // Coverflow genre strip
    lines.push(coverflow_categories(
        &browser.categories, browser.category_idx, (w as usize).saturating_sub(2), mode_color,
    ));
    lines.push(Line::from(Span::styled(
        " \u{2500}".to_string() + &"\u{2500}".repeat((w as usize).saturating_sub(4)),
        Style::default().fg(Color::Rgb(50, 50, 50)),
    )));

    // Scrolling
    let scroll_offset = if browser.preset_idx >= max_items {
        browser.preset_idx - max_items + 1
    } else {
        0
    };

    // Preset list
    if browser.preset_names.is_empty() {
        lines.push(Line::from(Span::styled(
            "   (no patterns)",
            Style::default().fg(theme::DIM_TEXT),
        )));
    } else {
        for (i, name) in browser.preset_names.iter().enumerate().skip(scroll_offset).take(max_items) {
            let is_sel = i == browser.preset_idx;
            let prefix = if is_sel { " \u{25b6} " } else { "   " };
            let style = if is_sel {
                Style::default().fg(Color::Black).bg(theme::CYAN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };
            lines.push(Line::from(Span::styled(format!("{}{}", prefix, name), style)));
        }
    }

    // Footer
    lines.push(Line::from(Span::styled(
        " \u{2500}".to_string() + &"\u{2500}".repeat((w as usize).saturating_sub(4)),
        Style::default().fg(Color::Rgb(50, 50, 50)),
    )));
    lines.push(Line::from(vec![
        Span::styled(" \u{2191}\u{2193}", Style::default().fg(theme::CYAN)),
        Span::styled(" Browse ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("\u{2190}\u{2192}", Style::default().fg(mode_color)),
        Span::styled(" Genre ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("Tab", Style::default().fg(theme::PINK)),
        Span::styled(" Mode ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("\u{23ce}", Style::default().fg(Color::Green)),
        Span::styled(" Load ", Style::default().fg(theme::DIM_TEXT)),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::styled(" Close", Style::default().fg(theme::DIM_TEXT)),
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup);
}
