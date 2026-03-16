// Color constants and style helpers — Hardware/Synthwave theme
// Palette constants are intentionally defined for completeness even if not yet referenced.
#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};

// ── Base palette ──────────────────────────────────────────────
pub const BG:          Color = Color::Rgb(26, 26, 26);    // #1a1a1a
pub const SURFACE:     Color = Color::Rgb(42, 42, 42);    // #2a2a2a
pub const BORDER:      Color = Color::Rgb(58, 58, 58);    // #3a3a3a
pub const DIM_TEXT:    Color = Color::Rgb(136, 136, 136);  // #888888
pub const TEXT:        Color = Color::Rgb(212, 212, 212);  // #d4d4d4

// ── Accent colors ─────────────────────────────────────────────
pub const AMBER:       Color = Color::Rgb(232, 168, 56);   // #e8a838
pub const AMBER_BRIGHT:Color = Color::Rgb(255, 200, 80);   // brighter for downbeats
pub const AMBER_DIM:   Color = Color::Rgb(100, 72, 24);    // dim amber outline
pub const CYAN:        Color = Color::Rgb(97, 218, 251);   // #61dafb
pub const PINK:        Color = Color::Rgb(255, 107, 157);  // #ff6b9d
pub const GOLD:        Color = Color::Rgb(255, 215, 0);    // #ffd700

// ── Semantic aliases (preserve old names for compatibility) ───
pub const ACTIVE_STEP:     Color = AMBER;
pub const INACTIVE_STEP:   Color = BORDER;
pub const PLAYHEAD_BG:     Color = AMBER;
pub const PLAYHEAD_FG:     Color = BG;
pub const CURSOR_BG:       Color = CYAN;
pub const CURSOR_FG:       Color = BG;

pub const MUTED_COLOR:     Color = Color::Red;
pub const SOLOED_COLOR:    Color = Color::Green;

pub const BEAT_LED_ON:     Color = CYAN;
pub const BEAT_LED_OFF:    Color = BORDER;
pub const BAR_START_LED:   Color = Color::Red;
pub const BEAT_BG:         Color = Color::Rgb(30, 30, 40);

pub const FOCUS_BORDER:    Color = CYAN;
pub const NORMAL_BORDER:   Color = BORDER;
pub const TITLE_COLOR:     Color = TEXT;

pub const PARAM_HIGHLIGHT_BG: Color = PINK;
pub const PARAM_HIGHLIGHT_FG: Color = BG;

// ── Gauge characters ──────────────────────────────────────────
pub const GAUGE_FILLED: &str = "\u{2588}"; // █
pub const GAUGE_EMPTY:  &str = "\u{2591}"; // ░

// ── Step characters ───────────────────────────────────────────
pub const STEP_ACTIVE:   &str = "\u{25A0}"; // ■
pub const STEP_INACTIVE: &str = "\u{25A1}"; // □

// ── Scene symbols ────────────────────────────────────────────
pub const SCENE_PAT_SYMBOL: &str = "\u{266B}"; // ♫
pub const SCENE_KIT_SYMBOL: &str = "\u{25C8}"; // ◈

/// Style for a focused section border.
pub fn focus_border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(FOCUS_BORDER)
    } else {
        Style::default().fg(NORMAL_BORDER)
    }
}

/// Style for a highlighted/selected parameter field.
pub fn param_highlight_style() -> Style {
    Style::default()
        .bg(PARAM_HIGHLIGHT_BG)
        .fg(PARAM_HIGHLIGHT_FG)
        .add_modifier(Modifier::BOLD)
}

/// Renders a horizontal gauge bar like "████░░░░░░".
pub fn gauge_string(value: f32, width: usize) -> String {
    let v = value.clamp(0.0, 1.0);
    let filled = (v * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    let mut s = String::with_capacity(width * 3);
    for _ in 0..filled { s.push_str(GAUGE_FILLED); }
    for _ in 0..empty { s.push_str(GAUGE_EMPTY); }
    s
}

/// Renders a percentage string like "80%".
pub fn percent_string(value: f32) -> String {
    let pct = (value.clamp(0.0, 1.0) * 100.0).round() as u32;
    format!("{}%", pct)
}
