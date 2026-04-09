//! Drum parameter panel: vertical slider bars for the selected track's sound parameters.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, FocusSection, KNOB_FIELDS};
use crate::sequencer::drum_pattern::TRACK_IDS;
use crate::ui::theme;

/// Full labels for the 13 slider columns.
const SLIDER_LABELS: [&str; 13] = [
    "Tune", "Sweep", "Color", "Snap", "Shape", "Attack", "Filter", "Drive", "Decay", "Volume",
    "Reverb", "Delay", "Pan",
];

/// Number of vertical bar rows in each slider.
const BAR_ROWS: usize = 5;

/// Renders vertical slider columns for the currently selected drum track's
/// parameters (Tune, Sweep, Color, Snap, Filter, Drive, Decay, Volume, etc.).
pub fn render_knobs(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.ui.focus == FocusSection::Knobs;
    let border_style = theme::focus_border_style(focused);

    let track = app.ui.drum_ctrl_track;
    let params = &app.drum_pattern.params[track];
    let track_name = TRACK_IDS[track].name();

    let title = format!(" {} ", track_name);

    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(theme::PINK)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Need at least 7 rows (1 label + 5 bars + 1 value) and some width
    if inner.height < 7 || inner.width < 20 {
        return;
    }

    let num_knobs = KNOB_FIELDS.len();
    let values: [f32; 13] = [
        params.tune,
        params.sweep,
        params.color,
        params.snap,
        params.shape,
        params.attack,
        params.filter,
        params.drive,
        params.decay,
        params.volume,
        params.send_reverb,
        params.send_delay,
        params.pan,
    ];

    let col_width = (inner.width as usize) / num_knobs;

    // ── Row 0: Labels ──────────────────────────────────────────────
    {
        let mut spans: Vec<Span> = Vec::new();
        for idx in 0..num_knobs {
            let label = SLIDER_LABELS[idx];
            let padded = format!("{:^width$}", label, width = col_width);
            spans.push(Span::styled(padded, Style::default().fg(theme::DIM_TEXT)));
        }
        // Pad remaining space for button area
        let used: usize = col_width * num_knobs;
        if used < inner.width as usize {
            spans.push(Span::raw(" ".repeat(inner.width as usize - used)));
        }
        let line_area = Rect::new(inner.x, inner.y, inner.width, 1);
        f.render_widget(Paragraph::new(Line::from(spans)), line_area);
    }

    // ── Rows 1..=5: Bar segments (top = high value, bottom = low) ──
    for bar_row in 0..BAR_ROWS {
        let mut spans: Vec<Span> = Vec::new();
        // Threshold: bar_row 0 = top (needs value > 0.8), bar_row 4 = bottom (needs value > 0.0)
        let threshold = 1.0 - ((bar_row as f32 + 1.0) / BAR_ROWS as f32);

        for idx in 0..num_knobs {
            let field = KNOB_FIELDS[idx];
            let value = values[idx];
            let is_selected = focused && app.ui.drum_ctrl_field == field;

            let (ch, fg, bg) = if value > threshold {
                // Filled segment
                let fill_color = if is_selected {
                    theme::PINK
                } else {
                    theme::AMBER
                };
                (theme::GAUGE_FILLED, fill_color, theme::BG)
            } else {
                // Empty segment
                (theme::GAUGE_EMPTY, theme::SURFACE, theme::BG)
            };

            // Center the bar character in the column
            let left_pad = (col_width.saturating_sub(1)) / 2;
            let right_pad = col_width.saturating_sub(1).saturating_sub(left_pad);

            spans.push(Span::raw(" ".repeat(left_pad)));
            spans.push(Span::styled(ch, Style::default().fg(fg).bg(bg)));
            spans.push(Span::raw(" ".repeat(right_pad)));
        }

        let y = inner.y + 1 + bar_row as u16;
        if y < inner.y + inner.height {
            let line_area = Rect::new(inner.x, y, inner.width, 1);
            f.render_widget(Paragraph::new(Line::from(spans)), line_area);
        }
    }

    // ── Row 6: Values ──────────────────────────────────────────────
    {
        let val_y = inner.y + 1 + BAR_ROWS as u16;
        if val_y < inner.y + inner.height {
            let mut spans: Vec<Span> = Vec::new();
            for idx in 0..num_knobs {
                let v = (values[idx].clamp(0.0, 1.0) * 100.0).round() as u32;
                let val_str = format!(".{:02}", v % 100);
                let padded = format!("{:^width$}", val_str, width = col_width);
                spans.push(Span::styled(padded, Style::default().fg(theme::AMBER)));
            }
            let used: usize = col_width * num_knobs;
            if used < inner.width as usize {
                spans.push(Span::raw(" ".repeat(inner.width as usize - used)));
            }
            let line_area = Rect::new(inner.x, val_y, inner.width, 1);
            f.render_widget(Paragraph::new(Line::from(spans)), line_area);
        }
    }
}
