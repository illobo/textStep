//! Synth parameter panel: grouped knobs for OSC, ENV, FILT, LFO, and AMP sections.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, FocusSection};
use crate::messages::SynthId;
use crate::sequencer::synth_pattern::{SynthControlField, lfo_waveform_name, lfo_division_name, lfo_dest_name, SynthParams};
use crate::ui::theme;

/// Height of the vertical slider bars.
const SLIDER_ROWS: usize = 5;

// ── Row group definitions ────────────────────────────────────────────────────
// Each constant below lists the `SynthControlField` variants for one sub-section.
// These arrays drive both the rendering order in `render_synth_knobs` and the
// keyboard/mouse navigation mapping in `mouse.rs`, so both files must stay in sync.

const OSC1_SLIDERS: &[SynthControlField] = &[
    SynthControlField::Osc1Tune,
    SynthControlField::Osc1Pwm,
    SynthControlField::Osc1Level,
    SynthControlField::Glide,
];

const OSC2_SLIDERS: &[SynthControlField] = &[
    SynthControlField::Osc2Tune,
    SynthControlField::Osc2Pwm,
    SynthControlField::Osc2Level,
    SynthControlField::Osc2Detune,
    SynthControlField::SubLevel,
];

const FILT_SLIDERS: &[SynthControlField] = &[
    SynthControlField::FilterCutoff,
    SynthControlField::FilterResonance,
    SynthControlField::FilterEnvAmount,
    SynthControlField::FilterKeyFollow,
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

const FILT_ENV_ADSR: &[SynthControlField] = &[
    SynthControlField::FilterEnvAttack,
    SynthControlField::FilterEnvDecay,
    SynthControlField::FilterEnvSustain,
    SynthControlField::FilterEnvRelease,
];

/// Short labels for ADSR bars.
const ADSR_LABELS: &[&str] = &["A", "D", "S", "R"];

// ── Main render ──────────────────────────────────────────────────────────────

/// Renders the synth parameter panel with grouped slider/ADSR sections
/// laid out in four row groups: OSC, ENV+FILT, LFO, and AMP.
pub fn render_synth_knobs(f: &mut Frame, area: Rect, app: &App, synth_id: SynthId) {
    let (pattern, ui, focus_section) = match synth_id {
        SynthId::A => (&app.synth_a_pattern, &app.ui.synth_a, FocusSection::SynthAControls),
        SynthId::B => (&app.synth_b_pattern, &app.ui.synth_b, FocusSection::SynthBControls),
    };

    let focused = app.ui.focus == focus_section;
    let border_style = theme::focus_border_style(focused);

    let title = match synth_id {
        SynthId::A => format!(" SYNTH A  Oct:{} ", ui.octave),
        SynthId::B => format!(" SYNTH B  Oct:{} ", ui.octave),
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(theme::TITLE_COLOR).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 10 || inner.width < 40 {
        return;
    }

    let params = &pattern.params;
    let sel = ui.ctrl_field;

    // Split inner into 3 row groups: OSC (8), ENV+FILT (8), AMP+LFO (remaining)
    let row_groups = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Row group 1: OSC1 + OSC2
            Constraint::Length(8), // Row group 2: ENV1 + ENV2 + FILT
            Constraint::Min(7),   // Row group 3: AMP (left) + LFO (right)
        ])
        .split(inner);

    // ── Row group 1: OSC1 + OSC2 ────────────────────────────────────────
    {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35), // OSC1
                Constraint::Length(2),      // gap
                Constraint::Percentage(65), // OSC2
            ])
            .split(row_groups[0]);

        render_section_header(f, cols[0], "OSC1");
        render_waveform_selector(
            f,
            Rect::new(cols[0].x, cols[0].y + 1, cols[0].width, 1),
            params.osc1_waveform,
            focused && sel == SynthControlField::Osc1Waveform,
        );
        render_slider_group(
            f,
            Rect::new(cols[0].x, cols[0].y + 2, cols[0].width, 6),
            params,
            OSC1_SLIDERS,
            sel,
            focused,
        );

        render_section_header(f, cols[2], "OSC2");
        // Row 1: Osc2 waveform (left) | Sub waveform (center) | Sync toggle (right)
        {
            let third = cols[2].width / 3;
            render_waveform_selector(
                f,
                Rect::new(cols[2].x, cols[2].y + 1, third, 1),
                params.osc2_waveform,
                focused && sel == SynthControlField::Osc2Waveform,
            );
            render_sub_waveform_selector(
                f,
                Rect::new(cols[2].x + third, cols[2].y + 1, third, 1),
                params.sub_waveform,
                focused && sel == SynthControlField::SubWaveform,
            );
            render_sync_toggle(
                f,
                Rect::new(cols[2].x + third * 2, cols[2].y + 1, cols[2].width - third * 2, 1),
                params.osc_sync > 0,
                focused && sel == SynthControlField::OscSync,
            );
        }
        render_slider_group(
            f,
            Rect::new(cols[2].x, cols[2].y + 2, cols[2].width, 6),
            params,
            OSC2_SLIDERS,
            sel,
            focused,
        );
    }

    // ── Row group 2: ENV1 + ENV2 + FILT ─────────────────────────────────
    {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // ENV1
                Constraint::Length(1),      // gap
                Constraint::Percentage(20), // ENV2
                Constraint::Length(1),      // gap
                Constraint::Percentage(60), // FILT
            ])
            .split(row_groups[1]);

        // ENV1: section header + packed ADSR bars
        render_section_header(f, cols[0], "ENV1");
        let env1_body = Rect::new(cols[0].x, cols[0].y + 1, cols[0].width, cols[0].height.saturating_sub(1));
        render_adsr_bars(f, env1_body, params, ENV1_ADSR, sel, focused);

        // ENV2: section header + packed ADSR bars
        render_section_header(f, cols[2], "ENV2");
        let env2_body = Rect::new(cols[2].x, cols[2].y + 1, cols[2].width, cols[2].height.saturating_sub(1));
        render_adsr_bars(f, env2_body, params, ENV2_ADSR, sel, focused);

        // FILT: section header + filter type selector + sliders + ADSR bars
        render_section_header(f, cols[4], "FILT");
        render_filter_type_selector(
            f,
            Rect::new(cols[4].x, cols[4].y + 1, cols[4].width, 1),
            params.filter_type,
            focused && sel == SynthControlField::FilterType,
        );

        // Split FILT area: left for sliders (Freq, Res, EnvAmt), right for ADSR bars
        let filt_body = Rect::new(
            cols[4].x,
            cols[4].y + 2,
            cols[4].width,
            cols[4].height.saturating_sub(2),
        );
        let filt_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45), // sliders
                Constraint::Percentage(55), // ADSR bars
            ])
            .split(filt_body);

        render_slider_group(f, filt_split[0], params, FILT_SLIDERS, sel, focused);
        render_adsr_bars(f, filt_split[1], params, FILT_ENV_ADSR, sel, focused);
    }

    // ── Row group 3: AMP (left) + LFO1/LFO2 stacked (right) ──────────
    {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // AMP
                Constraint::Percentage(50), // LFO1 + LFO2 stacked
            ])
            .split(row_groups[2]);

        // AMP section
        render_section_header(f, cols[0], "AMP");
        let amp_body = Rect::new(
            cols[0].x,
            cols[0].y + 1,
            cols[0].width,
            cols[0].height.saturating_sub(1),
        );
        render_amp_group(f, amp_body, params, app.effect_params.synth_saturator_drive, sel, focused);

        // LFO1 + LFO2 stacked vertically in the right column
        let lfo_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // LFO1: header + value + label
                Constraint::Min(3),   // LFO2: header + value + label
            ])
            .split(cols[1]);

        // LFO1 section
        render_section_header(f, lfo_rows[0], "LFO1");
        let lfo1_body = Rect::new(
            lfo_rows[0].x,
            lfo_rows[0].y + 1,
            lfo_rows[0].width,
            lfo_rows[0].height.saturating_sub(1),
        );
        render_lfo_row(f, lfo1_body, params, sel, focused, false);

        // LFO2 section
        render_section_header(f, lfo_rows[1], "LFO2");
        let lfo2_body = Rect::new(
            lfo_rows[1].x,
            lfo_rows[1].y + 1,
            lfo_rows[1].width,
            lfo_rows[1].height.saturating_sub(1),
        );
        render_lfo_row(f, lfo2_body, params, sel, focused, true);
    }
}

// ── Section header: ╶── LABEL ──╴ ───────────────────────────────────────────

fn render_section_header(f: &mut Frame, area: Rect, label: &str) {
    if area.width < 6 {
        return;
    }
    let dash_count = (area.width as usize).saturating_sub(label.len() + 6);
    let left_dashes = dash_count / 2;
    let right_dashes = dash_count - left_dashes;

    let dim = Style::default().fg(theme::DIM_TEXT);
    let bright = Style::default().fg(theme::AMBER).add_modifier(Modifier::BOLD);

    let spans = vec![
        Span::styled(format!("\u{2576}\u{2500}{}", "\u{2500}".repeat(left_dashes)), dim),
        Span::styled(format!(" {} ", label), bright),
        Span::styled(format!("{}\u{2500}\u{2574}", "\u{2500}".repeat(right_dashes)), dim),
    ];
    f.render_widget(
        Paragraph::new(Line::from(spans)),
        Rect::new(area.x, area.y, area.width, 1),
    );
}

// ── Waveform selector: [Sqr] Saw Sin Nse ────────────────────────────────────

fn render_waveform_selector(f: &mut Frame, area: Rect, current: u8, is_selected: bool) {
    let names = ["Sqr", "Saw", "Sin", "Nse"];
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw(" "));

    for (i, name) in names.iter().enumerate() {
        let is_active = i as u8 == current;
        if is_active {
            let color = if is_selected { theme::PINK } else { theme::AMBER };
            spans.push(Span::styled("[", Style::default().fg(color)));
            spans.push(Span::styled(
                *name,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("]", Style::default().fg(color)));
        } else {
            spans.push(Span::styled(
                *name,
                Style::default().fg(theme::DIM_TEXT),
            ));
        }
        spans.push(Span::raw(" "));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Sync toggle: [X]Sync or [ ]Sync ─────────────────────────────────────────

fn render_sync_toggle(f: &mut Frame, area: Rect, enabled: bool, is_selected: bool) {
    let color = if is_selected { theme::PINK } else if enabled { theme::AMBER } else { theme::DIM_TEXT };
    let check = if enabled { "X" } else { " " };
    let spans = vec![
        Span::styled("[", Style::default().fg(color)),
        Span::styled(check, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled("]", Style::default().fg(color)),
        Span::styled("Sync", Style::default().fg(color)),
    ];
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Sub waveform selector: [Sqr] Sin Saw ────────────────────────────────────

fn render_sub_waveform_selector(f: &mut Frame, area: Rect, current: u8, is_selected: bool) {
    let names = ["Sqr", "Sin", "Saw"];
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled("Sub:", Style::default().fg(theme::DIM_TEXT)));

    for (i, name) in names.iter().enumerate() {
        let is_active = i as u8 == current;
        if is_active {
            let color = if is_selected { theme::PINK } else { theme::AMBER };
            spans.push(Span::styled("[", Style::default().fg(color)));
            spans.push(Span::styled(
                *name,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("]", Style::default().fg(color)));
        } else {
            spans.push(Span::styled(
                *name,
                Style::default().fg(theme::DIM_TEXT),
            ));
        }
        spans.push(Span::raw(" "));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Filter type selector: [LP] HP BP ─────────────────────────────────────────

fn render_filter_type_selector(f: &mut Frame, area: Rect, current: u8, is_selected: bool) {
    let names = ["LP", "HP", "BP"];
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw(" "));

    for (i, name) in names.iter().enumerate() {
        let is_active = i as u8 == current;
        if is_active {
            let color = if is_selected { theme::PINK } else { theme::AMBER };
            spans.push(Span::styled("[", Style::default().fg(color)));
            spans.push(Span::styled(
                *name,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("]", Style::default().fg(color)));
        } else {
            spans.push(Span::styled(
                *name,
                Style::default().fg(theme::DIM_TEXT),
            ));
        }
        spans.push(Span::raw(" "));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Vertical slider group ────────────────────────────────────────────────────

fn render_slider_group(
    f: &mut Frame,
    area: Rect,
    params: &crate::sequencer::synth_pattern::SynthParams,
    fields: &[SynthControlField],
    selected: SynthControlField,
    focused: bool,
) {
    if fields.is_empty() || area.height < 3 || area.width < 4 {
        return;
    }

    let col_width = (area.width as usize) / fields.len().max(1);
    let bar_rows = SLIDER_ROWS.min(area.height.saturating_sub(2) as usize); // reserve label + value

    // Row 0: labels
    {
        let mut spans: Vec<Span> = Vec::new();
        for field in fields {
            let is_sel = focused && *field == selected;
            let label = field.full_label();
            let style = if is_sel {
                Style::default().fg(theme::PINK).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DIM_TEXT)
            };
            spans.push(Span::styled(
                format!("{:^width$}", label, width = col_width),
                style,
            ));
        }
        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(area.x, area.y, area.width, 1),
        );
    }

    // Rows 1..bar_rows: vertical bars (top = full, bottom = empty)
    for row in 0..bar_rows {
        let mut spans: Vec<Span> = Vec::new();
        for field in fields {
            let val = field.get(params);
            let filled = (val * bar_rows as f32).round() as usize;
            // Row 0 = top of bar, so filled if (bar_rows - row) <= filled
            let is_filled = (bar_rows - row) <= filled;
            let is_sel = focused && *field == selected;

            let ch = if is_filled { "\u{2588}" } else { "\u{2591}" }; // filled or empty
            let color = if is_sel { theme::PINK } else { theme::AMBER };

            // Center the bar character in the column
            let left_pad = (col_width.saturating_sub(1)) / 2;
            let right_pad = col_width.saturating_sub(1).saturating_sub(left_pad);

            spans.push(Span::raw(" ".repeat(left_pad)));
            spans.push(Span::styled(ch, Style::default().fg(color)));
            spans.push(Span::raw(" ".repeat(right_pad)));
        }
        let y = area.y + 1 + row as u16;
        if y < area.y + area.height {
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(area.x, y, area.width, 1),
            );
        }
    }

    // Bottom row: values (.XX format)
    {
        let val_y = area.y + 1 + bar_rows as u16;
        if val_y < area.y + area.height {
            let mut spans: Vec<Span> = Vec::new();
            for field in fields {
                let val = field.get(params);
                let is_sel = focused && *field == selected;
                let style = if is_sel {
                    Style::default().fg(theme::PINK)
                } else {
                    Style::default().fg(theme::DIM_TEXT)
                };
                spans.push(Span::styled(
                    format!("{:^width$.2}", val, width = col_width),
                    style,
                ));
            }
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(area.x, val_y, area.width, 1),
            );
        }
    }
}

// ── Packed ADSR vertical bars ────────────────────────────────────────────────
//
// Renders 4 tightly-packed vertical bar sliders labelled A, D, S, R.
// Each bar is 2 chars wide, with 1 char gap between bars.
// Total bar content: 4*2 + 3*1 = 11 chars, centered in the area.

fn render_adsr_bars(
    f: &mut Frame,
    area: Rect,
    params: &crate::sequencer::synth_pattern::SynthParams,
    fields: &[SynthControlField],
    selected: SynthControlField,
    focused: bool,
) {
    if fields.len() != 4 || area.height < 3 || area.width < 11 {
        return;
    }

    let bar_width: usize = 2;  // each bar is 2 chars wide
    let gap: usize = 1;        // 1 char gap between bars
    let group_width = 4 * bar_width + 3 * gap; // 11 chars total
    let left_pad = ((area.width as usize).saturating_sub(group_width)) / 2;

    let bar_rows = SLIDER_ROWS.min(area.height.saturating_sub(2) as usize); // reserve label + value

    // Row 0: labels (A D S R)
    {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" ".repeat(left_pad)));
        for (i, &label) in ADSR_LABELS.iter().enumerate() {
            let is_sel = focused && fields[i] == selected;
            let style = if is_sel {
                Style::default().fg(theme::PINK).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DIM_TEXT)
            };
            spans.push(Span::styled(format!("{:^width$}", label, width = bar_width), style));
            if i < 3 {
                spans.push(Span::raw(" ".repeat(gap)));
            }
        }
        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(area.x, area.y, area.width, 1),
        );
    }

    // Rows 1..bar_rows: vertical bars
    for row in 0..bar_rows {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" ".repeat(left_pad)));
        for (i, field) in fields.iter().enumerate() {
            let val = field.get(params);
            let filled = (val * bar_rows as f32).round() as usize;
            let is_filled = (bar_rows - row) <= filled;
            let is_sel = focused && *field == selected;

            let ch = if is_filled { "\u{2588}\u{2588}" } else { "\u{2591}\u{2591}" };
            let color = if is_sel { theme::PINK } else { theme::AMBER };

            spans.push(Span::styled(ch, Style::default().fg(color)));
            if i < 3 {
                spans.push(Span::raw(" ".repeat(gap)));
            }
        }
        let y = area.y + 1 + row as u16;
        if y < area.y + area.height {
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(area.x, y, area.width, 1),
            );
        }
    }

    // Bottom row: values as 2-digit integers (0.30 -> "30")
    {
        let val_y = area.y + 1 + bar_rows as u16;
        if val_y < area.y + area.height {
            let mut spans: Vec<Span> = Vec::new();
            spans.push(Span::raw(" ".repeat(left_pad)));
            for (i, field) in fields.iter().enumerate() {
                let val = field.get(params);
                let display = (val.clamp(0.0, 1.0) * 100.0).round() as u32;
                let display = display.min(99);
                let is_sel = focused && *field == selected;
                let style = if is_sel {
                    Style::default().fg(theme::PINK)
                } else {
                    Style::default().fg(theme::AMBER)
                };
                spans.push(Span::styled(format!("{:02}", display), style));
                if i < 3 {
                    spans.push(Span::raw(" ".repeat(gap)));
                }
            }
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(area.x, val_y, area.width, 1),
            );
        }
    }
}

// ── LFO row: [Wave] Div Depth [Dest] ─────────────────────────────────────────

fn render_lfo_row(
    f: &mut Frame,
    area: Rect,
    params: &SynthParams,
    selected: SynthControlField,
    focused: bool,
    is_lfo2: bool,
) {
    if area.height < 2 || area.width < 16 {
        return;
    }

    let (wave_field, div_field, depth_field, dest_field) = if is_lfo2 {
        (SynthControlField::Lfo2Waveform, SynthControlField::Lfo2Division,
         SynthControlField::Lfo2Depth, SynthControlField::Lfo2Dest)
    } else {
        (SynthControlField::LfoWaveform, SynthControlField::LfoDivision,
         SynthControlField::LfoDepth, SynthControlField::LfoDest)
    };
    let (waveform, division, depth, dest) = if is_lfo2 {
        (params.lfo2_waveform, params.lfo2_division, params.lfo2_depth, params.lfo2_dest)
    } else {
        (params.lfo_waveform, params.lfo_division, params.lfo_depth, params.lfo_dest)
    };

    let col_width = (area.width as usize) / 4;

    // Row 0: values — [Exp] 1/4 ████░░░░ [FilterCutoff]
    {
        let mut spans: Vec<Span> = Vec::new();

        // Wave selector
        let wave_sel = focused && selected == wave_field;
        let wave_name = lfo_waveform_name(waveform);
        let wave_color = if wave_sel { theme::PINK } else { theme::AMBER };
        let wave_str = format!("[{}]", wave_name);
        spans.push(Span::styled(
            format!("{:^width$}", wave_str, width = col_width),
            Style::default().fg(wave_color).add_modifier(Modifier::BOLD),
        ));

        // Division selector
        let div_sel = focused && selected == div_field;
        let div_name = lfo_division_name(division);
        let div_color = if div_sel { theme::PINK } else { theme::AMBER };
        spans.push(Span::styled(
            format!("{:^width$}", div_name, width = col_width),
            Style::default().fg(div_color).add_modifier(Modifier::BOLD),
        ));

        // Depth: horizontal bar
        let depth_sel = focused && selected == depth_field;
        let depth_color = if depth_sel { theme::PINK } else { theme::AMBER };
        let bar_width = col_width.saturating_sub(2);
        let filled = (depth * bar_width as f32).round() as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(bar, Style::default().fg(depth_color)));
        spans.push(Span::raw(" ".repeat(col_width.saturating_sub(bar_width + 1))));

        // Dest selector
        let dest_sel = focused && selected == dest_field;
        let dest_name = lfo_dest_name(dest);
        let dest_color = if dest_sel { theme::PINK } else { theme::AMBER };
        let dest_str = format!("[{}]", dest_name);
        spans.push(Span::styled(
            format!("{:^width$}", dest_str, width = col_width),
            Style::default().fg(dest_color).add_modifier(Modifier::BOLD),
        ));

        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(area.x, area.y, area.width, 1),
        );
    }

    // Row 1: labels
    if area.height >= 2 {
        let labels = ["Wave", "Div", "Depth", "Dest"];
        let mut spans: Vec<Span> = Vec::new();
        for (i, label) in labels.iter().enumerate() {
            let is_sel = focused && match i {
                0 => selected == wave_field,
                1 => selected == div_field,
                2 => selected == depth_field,
                3 => selected == dest_field,
                _ => false,
            };
            let style = if is_sel {
                Style::default().fg(theme::PINK).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DIM_TEXT)
            };
            spans.push(Span::styled(
                format!("{:^width$}", label, width = col_width),
                style,
            ));
        }
        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(area.x, area.y + 1, area.width, 1),
        );
    }
}

// ── AMP group: Vol, Reverb, Delay + Sat ──────────────────────────────────────

fn render_amp_group(
    f: &mut Frame,
    area: Rect,
    params: &crate::sequencer::synth_pattern::SynthParams,
    sat: f32,
    selected: SynthControlField,
    focused: bool,
) {

    // We render 4 columns: Vol, Reverb, Delay, Sat
    let fields_with_sat: &[(&str, f32, Option<SynthControlField>)] = &[
        ("Vol", params.volume, Some(SynthControlField::Volume)),
        ("Reverb", params.send_reverb, Some(SynthControlField::SendReverb)),
        ("Delay", params.send_delay, Some(SynthControlField::SendDelay)),
        ("Sat", sat, None), // Sat is from effect_params, not a SynthControlField
    ];

    if area.height < 3 || area.width < 8 {
        return;
    }

    let col_width = (area.width as usize) / fields_with_sat.len().max(1);
    let bar_rows = SLIDER_ROWS.min(area.height.saturating_sub(2) as usize);

    // Row 0: labels
    {
        let mut spans: Vec<Span> = Vec::new();
        for &(label, _, ctrl) in fields_with_sat {
            let is_sel = focused && ctrl.map_or(false, |c| c == selected);
            let style = if is_sel {
                Style::default().fg(theme::PINK).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DIM_TEXT)
            };
            spans.push(Span::styled(
                format!("{:^width$}", label, width = col_width),
                style,
            ));
        }
        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(area.x, area.y, area.width, 1),
        );
    }

    // Vertical bars
    for row in 0..bar_rows {
        let mut spans: Vec<Span> = Vec::new();
        for &(_, val, ctrl) in fields_with_sat {
            let filled = (val * bar_rows as f32).round() as usize;
            let is_filled = (bar_rows - row) <= filled;
            let is_sel = focused && ctrl.map_or(false, |c| c == selected);

            let ch = if is_filled { "\u{2588}" } else { "\u{2591}" };
            let color = if is_sel {
                theme::PINK
            } else if ctrl.is_none() && val > 0.01 {
                // Sat uses orange when active
                ratatui::style::Color::Rgb(255, 140, 0)
            } else {
                theme::AMBER
            };

            let left_pad = (col_width.saturating_sub(1)) / 2;
            let right_pad = col_width.saturating_sub(1).saturating_sub(left_pad);

            spans.push(Span::raw(" ".repeat(left_pad)));
            spans.push(Span::styled(ch, Style::default().fg(color)));
            spans.push(Span::raw(" ".repeat(right_pad)));
        }
        let y = area.y + 1 + row as u16;
        if y < area.y + area.height {
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(area.x, y, area.width, 1),
            );
        }
    }

    // Bottom row: values
    {
        let val_y = area.y + 1 + bar_rows as u16;
        if val_y < area.y + area.height {
            let mut spans: Vec<Span> = Vec::new();
            for &(_, val, ctrl) in fields_with_sat {
                let is_sel = focused && ctrl.map_or(false, |c| c == selected);
                let style = if is_sel {
                    Style::default().fg(theme::PINK)
                } else {
                    Style::default().fg(theme::DIM_TEXT)
                };
                spans.push(Span::styled(
                    format!("{:^width$.2}", val, width = col_width),
                    style,
                ));
            }
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(area.x, val_y, area.width, 1),
            );
        }
    }
}
