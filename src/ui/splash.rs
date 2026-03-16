//! Splash screen animation: typewriter logo, matrix rain reveal, boot sequence.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{SplashPhase, SplashState};
use super::theme;

// ── Block letter definitions (5 wide x 5 tall each) ─────────────────
const LETTER_T: [&str; 5] = ["█████", "  █  ", "  █  ", "  █  ", "  █  "];
const LETTER_E: [&str; 5] = ["█████", "█    ", "████ ", "█    ", "█████"];
const LETTER_X: [&str; 5] = ["█   █", " █ █ ", "  █  ", " █ █ ", "█   █"];
const LETTER_S: [&str; 5] = ["█████", "█    ", "█████", "    █", "█████"];
const LETTER_P: [&str; 5] = ["█████", "█   █", "█████", "█    ", "█    "];

const LOGO_LETTERS: [&[&str; 5]; 8] = [
    &LETTER_T, &LETTER_E, &LETTER_X, &LETTER_T,
    &LETTER_S, &LETTER_T, &LETTER_E, &LETTER_P,
];

const LETTER_WIDTH: usize = 5;
const LETTER_GAP: usize = 2;
const LOGO_ROWS: usize = 5;
const LOGO_WIDTH: u16 = (LETTER_WIDTH * 8 + LETTER_GAP * 7) as u16; // 54
const LOGO_HEIGHT: u16 = LOGO_ROWS as u16;

const SUBTITLE: &str = "step sequencer + synthesizer";
const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

// Step pattern displayed below the logo (16 steps)
const STEP_PATTERN: [bool; 16] = [
    true, false, true, false, true, true, false, true,
    false, false, true, false, true, true, false, true,
];

// Amber-themed reveal characters
const REVEAL_CHARS: &[u8] = b".:;=+*#%@&$~<>{}[]|/\\^";

/// Build one row of the block-letter logo
fn build_logo_line(row: usize) -> String {
    let gap: String = (0..LETTER_GAP).map(|_| ' ').collect();
    LOGO_LETTERS.iter()
        .map(|letter| letter[row])
        .collect::<Vec<_>>()
        .join(&gap)
}

/// Total chars in the logo (for typewriter effect)
fn logo_total_chars() -> usize {
    let line = build_logo_line(0);
    line.chars().count() * LOGO_ROWS
}

/// Draws the splash screen: animated block-letter logo with typewriter effect,
/// matrix rain columns, and a simulated boot log sequence.
pub fn render_splash(f: &mut Frame, area: Rect, splash: &SplashState) {
    match splash.phase {
        SplashPhase::SlideIn | SplashPhase::Hold => {
            render_boot(f, area, splash);
        }
        SplashPhase::MatrixReveal => {
            render_scanline_reveal(f, area, splash);
        }
        SplashPhase::Done => {}
    }
}

fn render_boot(f: &mut Frame, area: Rect, splash: &SplashState) {
    // Black background
    let blank = Paragraph::new("");
    f.render_widget(blank, area);

    let total_height = LOGO_HEIGHT + 7; // logo + line + steps + gap + subtitle + version
    let center_x = area.x + area.width.saturating_sub(LOGO_WIDTH) / 2;
    let center_y = area.y + area.height.saturating_sub(total_height) / 2;

    match splash.phase {
        SplashPhase::SlideIn => {
            let t = (splash.frame as f32 / 60.0).min(1.0);

            if t < 0.25 {
                // CRT power-on: horizontal amber line expands from center
                let line_t = t / 0.25;
                let eased = 1.0 - (1.0 - line_t).powi(2);
                let half_w = (eased * (area.width as f32 / 2.0)) as u16;
                let cx = area.x + area.width / 2;
                let cy = area.y + area.height / 2;
                let start = cx.saturating_sub(half_w);
                let width = (half_w * 2).min(area.width);
                if width > 0 {
                    // Bright center, fading edges
                    let mut spans: Vec<Span> = Vec::new();
                    for i in 0..width {
                        let dist = ((i as f32 / width as f32) - 0.5).abs() * 2.0;
                        let brightness = (1.0 - dist * 0.6).max(0.3);
                        let r = (232.0 * brightness) as u8;
                        let g = (168.0 * brightness) as u8;
                        let b = (56.0 * brightness) as u8;
                        spans.push(Span::styled("━", Style::default().fg(Color::Rgb(r, g, b))));
                    }
                    let p = Paragraph::new(Line::from(spans));
                    f.render_widget(p, Rect::new(start, cy, width, 1));
                }
            } else {
                // Logo characters type in one by one
                let type_t = (t - 0.25) / 0.75;
                let total_chars = logo_total_chars();
                let chars_shown = ((type_t * total_chars as f32) as usize).min(total_chars);

                // Amber brightness fades in
                let brightness = (type_t.min(0.5) * 2.0).min(1.0);
                let amber = amber_at_brightness(brightness);

                let mut shown = 0;
                for row in 0..LOGO_ROWS {
                    let y = center_y + row as u16;
                    if y >= area.y + area.height { break; }

                    let line = build_logo_line(row);
                    let line_chars: Vec<char> = line.chars().collect();
                    let to_show = (chars_shown.saturating_sub(shown)).min(line_chars.len());
                    if to_show > 0 {
                        let visible: String = line_chars[..to_show].iter().collect();
                        // Cursor char at the typing edge
                        let mut spans = vec![
                            Span::styled(visible, Style::default().fg(amber).add_modifier(Modifier::BOLD)),
                        ];
                        if to_show < line_chars.len() && shown + to_show == chars_shown {
                            spans.push(Span::styled("▌", Style::default().fg(theme::CYAN)));
                        }
                        let p = Paragraph::new(Line::from(spans));
                        f.render_widget(p, Rect::new(center_x, y, LOGO_WIDTH + 1, 1));
                    }
                    shown += line_chars.len();
                    if shown >= chars_shown { break; }
                }
            }
        }
        SplashPhase::Hold => {
            // Gentle amber pulse on the logo
            let pulse = ((splash.frame as f32 * 0.08).sin() * 0.12 + 0.88).clamp(0.76, 1.0);
            let amber = amber_at_brightness(pulse);

            // Render logo
            for row in 0..LOGO_ROWS {
                let y = center_y + row as u16;
                if y >= area.y + area.height { break; }
                let line = build_logo_line(row);
                let span = Span::styled(line, Style::default().fg(amber).add_modifier(Modifier::BOLD));
                let p = Paragraph::new(Line::from(span));
                f.render_widget(p, Rect::new(center_x, y, LOGO_WIDTH, 1));
            }

            // Decorative line below logo
            let line_y = center_y + LOGO_HEIGHT + 1;
            if line_y < area.y + area.height {
                let deco_width = LOGO_WIDTH as usize;
                let mut deco = String::with_capacity(deco_width * 3);
                for i in 0..deco_width {
                    // Fade from border to amber at center
                    let center_dist = ((i as f32 / deco_width as f32) - 0.5).abs() * 2.0;
                    if center_dist < 0.6 {
                        deco.push('═');
                    } else {
                        deco.push('─');
                    }
                }
                let span = Span::styled(deco, Style::default().fg(theme::BORDER));
                let p = Paragraph::new(Line::from(span));
                f.render_widget(p, Rect::new(center_x, line_y, LOGO_WIDTH, 1));
            }

            // Animated step sequencer pattern
            let steps_y = center_y + LOGO_HEIGHT + 3;
            if steps_y < area.y + area.height {
                let playhead = (splash.frame as usize / 4) % 16;
                let step_str_width = 16 * 2 - 1; // "■ □ ■ ..." with spaces between
                let steps_x = area.x + area.width.saturating_sub(step_str_width as u16) / 2;

                let mut spans: Vec<Span> = Vec::new();
                for (i, &active) in STEP_PATTERN.iter().enumerate() {
                    let is_playhead = i == playhead;
                    let (ch, style) = if is_playhead {
                        (theme::STEP_ACTIVE, Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD))
                    } else if active {
                        (theme::STEP_ACTIVE, Style::default().fg(amber))
                    } else {
                        (theme::STEP_INACTIVE, Style::default().fg(theme::BORDER))
                    };
                    spans.push(Span::styled(ch, style));
                    if i < 15 { spans.push(Span::raw(" ")); }
                }
                let p = Paragraph::new(Line::from(spans));
                f.render_widget(p, Rect::new(steps_x, steps_y, step_str_width as u16, 1));
            }

            // Subtitle
            let sub_y = center_y + LOGO_HEIGHT + 5;
            if sub_y < area.y + area.height {
                let sub_x = area.x + area.width.saturating_sub(SUBTITLE.len() as u16) / 2;
                let span = Span::styled(SUBTITLE, Style::default().fg(theme::DIM_TEXT));
                let p = Paragraph::new(Line::from(span));
                f.render_widget(p, Rect::new(sub_x, sub_y, SUBTITLE.len() as u16, 1));
            }

            // Version in pink
            let ver_y = center_y + LOGO_HEIGHT + 6;
            if ver_y < area.y + area.height {
                let ver_x = area.x + area.width.saturating_sub(VERSION.len() as u16) / 2;
                let span = Span::styled(VERSION, Style::default().fg(theme::PINK));
                let p = Paragraph::new(Line::from(span));
                f.render_widget(p, Rect::new(ver_x, ver_y, VERSION.len() as u16, 1));
            }
        }
        _ => {}
    }
}

fn render_scanline_reveal(f: &mut Frame, area: Rect, splash: &SplashState) {
    let w = splash.matrix_width as usize;
    let h = splash.matrix_height as usize;

    // Amber-themed scanline reveal
    for row in 0..area.height.min(h as u16) {
        let mut col: u16 = 0;
        let cols = area.width.min(w as u16);
        while col < cols {
            let idx = (row as usize) * w + (col as usize);
            let revealed = idx < splash.revealed.len() && splash.revealed[idx];

            if revealed {
                col += 1;
                continue;
            }

            let run_start = col;
            let mut spans: Vec<Span> = Vec::new();
            while col < cols {
                let idx2 = (row as usize) * w + (col as usize);
                if idx2 < splash.revealed.len() && splash.revealed[idx2] {
                    break;
                }

                let col_data = &splash.matrix_columns[col as usize];
                let cur_row = col_data.0 as i32;
                let row_i = row as i32;

                if row_i == cur_row {
                    // Lead character: bright amber/white
                    let ch = reveal_char(col_data.2);
                    spans.push(Span::styled(
                        String::from(ch),
                        Style::default().fg(theme::AMBER_BRIGHT).bg(Color::Black),
                    ));
                } else if row_i > cur_row.saturating_sub(4) && row_i < cur_row {
                    // Trail: fading amber
                    let dist = (cur_row - row_i) as u8;
                    let fade = 1.0 - (dist as f32 * 0.22);
                    let amber = amber_at_brightness(fade.max(0.15));
                    let ch = reveal_char(col_data.2.wrapping_add(dist * 7));
                    spans.push(Span::styled(
                        String::from(ch),
                        Style::default().fg(amber).bg(Color::Black),
                    ));
                } else {
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(Color::Black),
                    ));
                }

                col += 1;
            }

            if !spans.is_empty() {
                let run_len = col - run_start;
                let line = Paragraph::new(Line::from(spans));
                let cell_area = Rect::new(area.x + run_start, area.y + row, run_len, 1);
                f.render_widget(line, cell_area);
            }
        }
    }
}

/// Amber color at a given brightness (0.0-1.0)
fn amber_at_brightness(b: f32) -> Color {
    Color::Rgb(
        (232.0 * b) as u8,
        (168.0 * b) as u8,
        (56.0 * b) as u8,
    )
}

fn reveal_char(seed: u8) -> char {
    REVEAL_CHARS[(seed as usize) % REVEAL_CHARS.len()] as char
}
