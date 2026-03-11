//! Synth step grid: 32-step note sequencer with multi-step note visualization.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, FocusSection};
use crate::messages::SynthId;
use crate::sequencer::synth_pattern::MAX_STEPS;
use crate::sequencer::transport::PlayState;
use crate::ui::theme;

/// Track name column width, matching drum grid (padded to longest name "Cowbell")
const NAME_WIDTH: usize = 9;

/// Renders the synth step row with note names, velocity shading,
/// multi-step continuation bars, and playhead/cursor highlights.
pub fn render_synth_grid(f: &mut Frame, area: Rect, app: &App, synth_id: SynthId) {
    let (pattern, ui_state, focus_section, loop_len) = match synth_id {
        SynthId::A => (
            &app.synth_a_pattern,
            &app.ui.synth_a,
            FocusSection::SynthAGrid,
            app.transport.loop_config.synth_a_length,
        ),
        SynthId::B => (
            &app.synth_b_pattern,
            &app.ui.synth_b,
            FocusSection::SynthBGrid,
            app.transport.loop_config.synth_b_length,
        ),
    };

    let focused = app.ui.focus == focus_section;
    let border_style = theme::focus_border_style(focused);

    let muted = pattern.params.mute;

    let title = match synth_id {
        SynthId::A => format!(" SYNTH A STEPS [{} steps] ", loop_len),
        SynthId::B => format!(" SYNTH B STEPS [{} steps] ", loop_len),
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(theme::TITLE_COLOR).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(border_style);

    // ── Header row: beat numbers ──
    let mut header_spans: Vec<Span> = Vec::new();
    // Pad to align with track name column (NAME_WIDTH = 6)
    header_spans.push(Span::styled(
        " ".repeat(NAME_WIDTH),
        Style::default().fg(theme::DIM_TEXT),
    ));

    for step in 0..MAX_STEPS {
        if step == 16 {
            header_spans.push(Span::styled(
                "\u{2503}",
                Style::default().fg(theme::BORDER),
            ));
        }
        let beat_in_bar = step % 16;
        let beat_num = beat_in_bar / 4 + 1;
        let sub = beat_in_bar % 4;
        if sub == 0 {
            header_spans.push(Span::styled(
                format!("{} ", beat_num),
                Style::default().fg(theme::DIM_TEXT),
            ));
        } else {
            header_spans.push(Span::styled(
                "\u{00B7} ",
                Style::default().fg(theme::BORDER),
            ));
        }
    }

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(header_spans));

    // ── Spacer row between header and step row ──
    lines.push(Line::from(""));

    // ── Step row ──
    let mut step_spans: Vec<Span> = Vec::new();
    // Track name with mute indicator, matching drum grid NAME_WIDTH
    if muted {
        let label = format!("Syn {}", "M");
        step_spans.push(Span::styled(
            format!("{:<width$}", label, width = NAME_WIDTH),
            Style::default().fg(theme::MUTED_COLOR).add_modifier(Modifier::BOLD),
        ));
    } else {
        step_spans.push(Span::styled(
            format!(" {:<width$}", "Synth", width = NAME_WIDTH - 1),
            Style::default().fg(theme::DIM_TEXT),
        ));
    }

    let playback_step = ui_state.playback_step;
    let is_playing = app.transport.state == PlayState::Playing;

    // Multi-step note tracking: `covered_until` holds the last step index covered by
    // the current note's length. Steps within that range render as continuation bars
    // (`is_continuation`) instead of new note heads, with an end-cap on the final step.
    let mut covered_until: Option<usize> = None;
    let mut cover_bg: Option<Color> = None; // background color for continuation cells

    for s in 0..MAX_STEPS {
        if s == 16 {
            step_spans.push(Span::styled(
                "\u{2503}",
                Style::default().fg(theme::BORDER),
            ));
        }

        let step = &pattern.steps[s];
        let is_cursor = focused && s == ui_state.cursor_step;
        let is_playhead = is_playing && s == playback_step;
        let out_of_loop = s >= loop_len as usize;
        let is_downbeat = s % 4 == 0;

        // Check if this step is a continuation of a previous multi-step note
        let is_continuation = covered_until.map_or(false, |end| s <= end);

        let (text, is_active) = if is_continuation {
            // Render continuation bar; use end-cap on the last step
            let end = covered_until.unwrap();
            let display = if s == end { "─╴" } else { "──" };
            (display.to_string(), true)
        } else if step.is_active() {
            let length = step.length.max(1) as usize;
            if length > 1 {
                covered_until = Some((s + length - 1).min(MAX_STEPS - 1));
            } else {
                covered_until = None;
            }
            let name = step.note_name();
            let display: String = if name.len() >= 2 {
                name[..2].to_string()
            } else {
                format!("{} ", name)
            };
            (display, true)
        } else {
            covered_until = None;
            (format!("{} ", theme::STEP_INACTIVE), false)
        };

        // When starting a new multi-step note, remember its bg for continuations
        if step.is_active() && !is_continuation && step.length.max(1) > 1 {
            cover_bg = Some(theme::SURFACE);
        }

        // Determine base foreground color based on active/downbeat
        let base_fg = if out_of_loop {
            theme::BORDER
        } else if is_active && is_downbeat {
            theme::AMBER_BRIGHT
        } else if is_active {
            theme::AMBER
        } else if is_downbeat {
            theme::AMBER_DIM
        } else {
            theme::BORDER
        };

        let style = if is_cursor && is_playhead {
            Style::default()
                .fg(theme::CURSOR_FG)
                .bg(theme::PLAYHEAD_BG)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_cursor {
            Style::default()
                .fg(theme::CURSOR_FG)
                .bg(theme::CURSOR_BG)
                .add_modifier(Modifier::BOLD)
        } else if is_playhead {
            Style::default()
                .fg(theme::PLAYHEAD_FG)
                .bg(theme::PLAYHEAD_BG)
        } else if is_continuation && !out_of_loop {
            let bg = cover_bg.unwrap_or(Color::Reset);
            Style::default().fg(base_fg).bg(bg)
        } else {
            Style::default().fg(base_fg)
        };

        // Clear cover state when we pass the end
        if covered_until.map_or(false, |end| s >= end) {
            covered_until = None;
            cover_bg = None;
        }

        step_spans.push(Span::styled(text, style));
    }

    lines.push(Line::from(step_spans));

    // ── Spacer row after step row ──
    lines.push(Line::from(""));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
