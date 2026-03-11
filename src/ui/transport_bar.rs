//! Transport control bar: play/pause state, BPM, beat LEDs, swing, pattern/kit selectors.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, FocusSection};
use crate::sequencer::transport::{PlayState, RecordMode};
use crate::ui::theme;

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

    // ── Line 2: Synth A status line ───────────────────────────────
    let synth_a_focused = matches!(app.ui.focus, FocusSection::SynthAGrid | FocusSection::SynthAControls);
    let synth_a_loop_str = if app.transport.loop_config.enabled {
        format!("Loop[{}]", app.transport.loop_config.synth_a_length)
    } else {
        "Loop[--]".to_string()
    };
    let synth_a_kit_name = app.project.synth_kits.get(app.ui.synth_a.active_kit)
        .map(|k| k.name.as_str()).unwrap_or("");
    let synth_a_line = status_line(
        "SA",
        app.ui.synth_a.active_pattern,
        app.ui.synth_a.queued_pattern,
        app.ui.synth_a.active_kit,
        synth_a_kit_name,
        synth_a_focused,
        &synth_a_loop_str,
        app.synth_a_pattern.params.volume,
    );

    // ── Line 3: Synth B status line ───────────────────────────────
    let synth_b_focused = matches!(app.ui.focus, FocusSection::SynthBGrid | FocusSection::SynthBControls);
    let synth_b_loop_str = if app.transport.loop_config.enabled {
        format!("Loop[{}]", app.transport.loop_config.synth_b_length)
    } else {
        "Loop[--]".to_string()
    };
    let synth_b_kit_name = app.project.synth_kits.get(app.ui.synth_b.active_kit)
        .map(|k| k.name.as_str()).unwrap_or("");
    let synth_b_line = status_line(
        "SB",
        app.ui.synth_b.active_pattern,
        app.ui.synth_b.queued_pattern,
        app.ui.synth_b.active_kit,
        synth_b_kit_name,
        synth_b_focused,
        &synth_b_loop_str,
        app.synth_b_pattern.params.volume,
    );

    // ── Line 4: Drum status line ──────────────────────────────────
    let drum_focused = matches!(app.ui.focus, FocusSection::DrumGrid | FocusSection::Knobs);
    let drum_loop_str = if app.transport.loop_config.enabled {
        format!("Loop[{}]", app.transport.loop_config.drum_length)
    } else {
        "Loop[--]".to_string()
    };
    let drum_kit_name = app.current_kit_name();
    let drum_line = status_line(
        "DR",
        app.ui.active_pattern,
        app.ui.queued_pattern,
        app.ui.active_kit,
        drum_kit_name,
        drum_focused,
        &drum_loop_str,
        app.effect_params.drum_volume,
    );

    // ── Line 5: Master gauges ────────────────────────────────────
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

    let paragraph = Paragraph::new(vec![
        top_line,
        synth_a_line,
        synth_b_line,
        drum_line,
        gauge_line,
    ]).block(block);
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

/// Pattern slot key labels (QWERTYUIOP = patterns 1-10).
const PATTERN_KEYS: [&str; 10] = ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"];

/// Width of the volume slider in the status line (in characters).
pub const STATUS_VOL_WIDTH: usize = 8;

// ── Status line column offsets (from inner_x) for mouse hit-testing ──────────
// Format: "SA  Pattern: q w e r t y u i o p │ Kit: 1 2 3 4 5 6 7 8  Loop[NN]  Vol:████░░░░"
//          2   11       19                    8     15               10         6   8
/// Column offset of pattern keys from inner_x.
pub const STATUS_PAT_OFFSET: u16 = 13; // label(2) + "  Pattern: "(11)
/// Column offset of kit numbers from inner_x.
pub const STATUS_KIT_OFFSET: u16 = 40; // PAT(13) + keys(19) + " │ Kit: "(8)
/// Column offset of volume slider bar from inner_x.
pub const STATUS_VOL_OFFSET: u16 = 71; // KIT(40) + nums(15) + "  Loop[NN]"(10=2+8pad) + "  Vol:"(6)

/// Build a status line with pattern/kit selectors and volume slider:
///   SA  Pattern: q w e r t y u i o p │ Kit: 1 2 3 4 5 6 7 8  Loop[32]  Vol:████░░░░
fn status_line<'a>(
    label: &str,
    active_pattern: usize,
    queued_pattern: Option<usize>,
    active_kit: usize,
    _kit_name: &str,
    is_focused: bool,
    loop_info: &str,
    volume: f32,
) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = Vec::new();

    // Section label
    let label_style = if is_focused {
        Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::DIM_TEXT)
    };
    spans.push(Span::styled(format!("{}", label), label_style));

    spans.push(Span::styled("  Pattern: ", Style::default().fg(theme::DIM_TEXT)));

    // Pattern selector: q w e r t y u i o p
    for (i, key) in PATTERN_KEYS.iter().enumerate() {
        let style = if i == active_pattern {
            Style::default().fg(Color::Black).bg(theme::CYAN).add_modifier(Modifier::BOLD)
        } else if queued_pattern == Some(i) {
            Style::default().fg(Color::Black).bg(theme::GOLD).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DIM_TEXT)
        };
        spans.push(Span::styled(*key, style));
        if i < 9 { spans.push(Span::raw(" ")); }
    }

    spans.push(Span::styled(" \u{2502} Kit: ", Style::default().fg(theme::DIM_TEXT)));

    // Kit selector: 1 2 3 4 5 6 7 8
    for i in 0..8u8 {
        let style = if i as usize == active_kit {
            Style::default().fg(Color::Black).bg(theme::CYAN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DIM_TEXT)
        };
        spans.push(Span::styled(format!("{}", i + 1), style));
        if i < 7 { spans.push(Span::raw(" ")); }
    }

    // Loop indicator
    let loop_style = if loop_info.contains("--") {
        Style::default().fg(theme::DIM_TEXT)
    } else if is_focused {
        Style::default().fg(theme::CYAN)
    } else {
        Style::default().fg(theme::TEXT)
    };
    spans.push(Span::styled(format!("  {:<8}", loop_info), loop_style));

    // Volume slider: amber fill (same theme as master gauges) + numeric value
    spans.push(Span::styled("  Vol:", Style::default().fg(theme::DIM_TEXT)));
    spans.push(gauge_spans(
        volume, STATUS_VOL_WIDTH,
        Style::default().fg(theme::AMBER),
        Style::default().fg(theme::SURFACE),
    ));
    let pct = (volume.clamp(0.0, 1.0) * 100.0).round() as u8;
    spans.push(Span::styled(
        format!("{:>3}", pct),
        Style::default().fg(theme::TEXT),
    ));

    Line::from(spans)
}
