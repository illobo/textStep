//! Transport control bar: play/pause state, BPM, beat LEDs, swing, pattern/kit selectors.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, FocusSection};
use crate::sequencer::project::{NUM_KITS, NUM_PATTERNS};
use crate::sequencer::transport::{PlayState, RecordMode};
use crate::ui::theme;

const PATTERN_KEYS: [char; 10] = ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'];
const KIT_KEYS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

/// Draws the transport bar: play state, BPM, beat LEDs, swing, record toggle,
/// pattern/kit selectors, loop indicators, and master level gauges.
pub fn render_transport(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.ui.focus == FocusSection::Transport;
    let border_style = theme::focus_border_style(focused);

    let dirty_mark = if app.dirty { "*" } else { "" };
    let project_name = if app.project.metadata.name.is_empty() {
        "Untitled"
    } else {
        &app.project.metadata.name
    };

    let block = Block::default()
        .title(format!(" TextStep - {}{} ", project_name, dirty_mark))
        .title_style(Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(border_style);

    // ── Line 1: Play state + BPM + Beat LEDs + Swing + Record ────
    let play_icon = match app.transport.state {
        PlayState::Playing => Span::styled(
            "\u{25B6} PLAY ",
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD),
        ),
        PlayState::Paused => Span::styled(
            "\u{23F8} PAUSE",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        PlayState::Stopped => Span::styled(
            "\u{25A0} STOP ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    };

    let bpm_span = Span::styled(
        format!("  BPM: {:.1}", app.transport.bpm),
        Style::default().fg(Color::White),
    );

    // Beat LEDs: filled cyan for active, hollow dim for inactive, bar start = red
    let is_playing = app.transport.state == PlayState::Playing;
    let beat = app.ui.current_beat as usize;
    let beat_spans: Vec<Span> = (0..4)
        .flat_map(|i| {
            let dot = if is_playing && i <= beat {
                if app.ui.is_bar_start {
                    Span::styled("\u{25CF}", Style::default().fg(theme::BAR_START_LED).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("\u{25CF}", Style::default().fg(theme::BEAT_LED_ON).add_modifier(Modifier::BOLD))
                }
            } else {
                Span::styled("\u{25CB}", Style::default().fg(theme::BEAT_LED_OFF))
            };
            vec![Span::raw(" "), dot]
        })
        .collect();

    let swing_pct = (app.transport.swing * 100.0).round() as u8;
    let swing_span = if swing_pct > 50 {
        Span::styled(
            format!("  Swing: {}%", swing_pct),
            Style::default().fg(theme::GOLD),
        )
    } else {
        Span::styled(
            "  Swing: OFF",
            Style::default().fg(theme::DIM_TEXT),
        )
    };

    let rec_span = match app.transport.record_mode {
        RecordMode::On => Span::styled(
            "  \u{25CF} REC",
            Style::default().fg(theme::PINK).add_modifier(Modifier::BOLD),
        ),
        RecordMode::Off => Span::styled(
            "  \u{25CB} REC",
            Style::default().fg(theme::BORDER),
        ),
    };

    let mut top_spans = vec![play_icon, bpm_span];
    top_spans.extend(beat_spans);
    top_spans.extend(vec![swing_span, rec_span]);

    let top_line = Line::from(top_spans);

    // ── Line 2: Synth machine selector + loop indicator ──────────
    let synth_focused = matches!(app.ui.focus, FocusSection::SynthAGrid | FocusSection::SynthAControls);
    let synth_loop_str = if app.transport.loop_config.enabled {
        format!("Loop [ON] S:{}", app.transport.loop_config.synth_a_length)
    } else {
        "Loop [OFF]".to_string()
    };
    let synth_kit_name = app.project.synth_kits.get(app.ui.synth_a.active_kit)
        .map(|k| k.name.as_str()).unwrap_or("");
    let synth_line = machine_selector_line(
        "Synth",
        app.ui.synth_a.active_pattern,
        app.ui.synth_a.queued_pattern,
        app.ui.synth_a.active_kit,
        synth_kit_name,
        synth_focused,
        &synth_loop_str,
    );

    // ── Line 3: Drum machine selector + loop indicator ───────────
    let drum_focused = matches!(app.ui.focus, FocusSection::DrumGrid | FocusSection::Knobs);
    let drum_loop_str = if app.transport.loop_config.enabled {
        format!("Loop [ON] D:{}", app.transport.loop_config.drum_length)
    } else {
        "Loop [OFF]".to_string()
    };
    let drum_kit_name = app.current_kit_name();
    let drum_line = machine_selector_line(
        "Drum ",
        app.ui.active_pattern,
        app.ui.queued_pattern,
        app.ui.active_kit,
        drum_kit_name,
        drum_focused,
        &drum_loop_str,
    );

    // ── Line 4: Master gauges ────────────────────────────────────
    let gauge_label_style = Style::default().fg(theme::DIM_TEXT);
    let gauge_fill_style = Style::default().fg(theme::AMBER);
    let gauge_empty_style = Style::default().fg(theme::SURFACE);

    let vol = app.effect_params.master_volume;
    let comp = app.effect_params.compressor_amount;
    let sat = app.effect_params.drum_saturator_drive;

    let gauge_line = Line::from(vec![
        Span::styled("VOL ", gauge_label_style),
        gauge_spans(vol, 6, gauge_fill_style, gauge_empty_style),
        Span::raw("   "),
        Span::styled("CMP ", gauge_label_style),
        gauge_spans(comp, 4, gauge_fill_style, gauge_empty_style),
        Span::raw("   "),
        Span::styled("SAT ", gauge_label_style),
        gauge_spans(sat, 4, gauge_fill_style, gauge_empty_style),
    ]);

    let paragraph = Paragraph::new(vec![top_line, synth_line, drum_line, gauge_line]).block(block);
    f.render_widget(paragraph, area);
}

/// Build a styled gauge Span with filled/empty chars in different styles.
fn gauge_spans<'a>(value: f32, width: usize, fill_style: Style, empty_style: Style) -> Span<'a> {
    let v = value.clamp(0.0, 1.0);
    let filled = (v * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    let mut s = String::with_capacity(width * 3);
    for _ in 0..filled { s.push_str(theme::GAUGE_FILLED); }
    for _ in 0..empty { s.push_str(theme::GAUGE_EMPTY); }
    // Use fill style if any filled, otherwise empty style
    if filled > 0 {
        Span::styled(s, fill_style)
    } else {
        Span::styled(s, empty_style)
    }
}

/// Build a Line for a machine's pattern + kit selector row with loop indicator.
fn machine_selector_line<'a>(
    label: &str,
    active_pattern: usize,
    queued_pattern: Option<usize>,
    active_kit: usize,
    kit_name: &str,
    is_focused: bool,
    loop_info: &str,
) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = Vec::new();

    // Machine label — highlighted when focused
    let label_style = if is_focused {
        Style::default()
            .fg(theme::CYAN)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::DIM_TEXT)
    };
    spans.push(Span::styled(format!("{}  ", label), label_style));

    // Pattern selector
    spans.push(Span::styled("Pattern: ", Style::default().fg(theme::TEXT)));
    for i in 0..NUM_PATTERNS {
        let is_active = active_pattern == i;
        let is_queued = queued_pattern == Some(i);
        let key = PATTERN_KEYS[i];

        if is_active {
            spans.push(Span::styled(
                format!("[{}]", key),
                Style::default()
                    .fg(theme::BG)
                    .bg(theme::AMBER)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if is_queued {
            spans.push(Span::styled(
                format!("[{}]", key),
                Style::default()
                    .fg(theme::BG)
                    .bg(theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(format!(" {} ", key), Style::default().fg(theme::DIM_TEXT)));
        }
    }

    spans.push(Span::styled("   Kit: ", Style::default().fg(theme::TEXT)));
    for i in 0..NUM_KITS {
        let is_active = active_kit == i;
        let key = KIT_KEYS[i];

        if is_active {
            spans.push(Span::styled(
                format!("[{}]", key),
                Style::default()
                    .fg(theme::BG)
                    .bg(theme::AMBER)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(format!(" {} ", key), Style::default().fg(theme::DIM_TEXT)));
        }
    }

    // Kit name after selector
    if !kit_name.is_empty() {
        spans.push(Span::styled(
            format!(" {}", kit_name),
            Style::default().fg(theme::AMBER),
        ));
    }

    // Loop indicator at end of row
    let loop_style = if loop_info.contains("[ON]") {
        Style::default().fg(theme::CYAN)
    } else {
        Style::default().fg(theme::DIM_TEXT)
    };
    spans.push(Span::styled(format!("   {}", loop_info), loop_style));

    Line::from(spans)
}
