//! Help overlay: 3-column keyboard shortcut reference panel.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

// Each column: 1 space + KEY_W key + DESC_W desc = COL_W total
// Separator: " │ " = 3 chars
const KEY_W: usize = 15;
const DESC_W: usize = 18;
const COL_W: usize = 1 + KEY_W + DESC_W; // 30

fn ks() -> Style { Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD) }
fn ds() -> Style { Style::default().fg(Color::White) }
fn ss() -> Style { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) }
fn sep_style() -> Style { Style::default().fg(Color::DarkGray) }

fn sep<'a>() -> Span<'a> {
    Span::styled(" \u{2502} ", sep_style())
}

/// One column cell: " key            description   "
fn cell<'a>(k: &'a str, d: &'a str) -> Vec<Span<'a>> {
    vec![
        Span::styled(format!(" {:<w$}", k, w = KEY_W), ks()),
        Span::styled(format!("{:<w$}", d, w = DESC_W), ds()),
    ]
}

/// Section header row spanning 3 columns.
fn hdr<'a>(a: &'a str, b: &'a str, c: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!(" {:<w$}", a, w = COL_W - 1), ss()),
        sep(),
        Span::styled(format!(" {:<w$}", b, w = COL_W - 1), ss()),
        sep(),
        Span::styled(format!(" {:<w$}", c, w = COL_W - 1), ss()),
    ])
}

/// Data row with 3 key/description pairs.
fn row3<'a>(
    k1: &'a str, d1: &'a str,
    k2: &'a str, d2: &'a str,
    k3: &'a str, d3: &'a str,
) -> Line<'a> {
    let mut v = cell(k1, d1);
    v.push(sep());
    v.extend(cell(k2, d2));
    v.push(sep());
    v.extend(cell(k3, d3));
    Line::from(v)
}

/// Renders the help panel with three columns of key-binding entries
/// covering transport, navigation, editing, and pattern management shortcuts.
pub fn render_help(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Key Bindings (? to close) ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let lines = vec![
        hdr("Transport",        "Navigation",       "Sound & Editing"),
        row3("Space",       "Play / Pause",
             "Tab",         "Next section",
             "Shift+M",    "Mute track"),
        row3("Esc",         "Stop",
             "Shift+Tab",  "Prev section",
             "Shift+S",    "Solo track"),
        row3("- / =",      "BPM -1 / +1",
             "Arrows",     "Move cursor",
             "Shift+V",    "Master volume"),
        row3("Shift+- / +","BPM -10 / +10",
             "Enter",      "Toggle step",
             "Shift+C",    "Compressor"),
        row3("` (backtick)","Record on/off",
             "Shift+Up/Dn","Adjust value",
             "Shift+T",    "Tube saturator"),
        row3("l",           "Loop on/off",
             "Alt+Up/Dn",  "Adjust+audition",
             "Alt+R",      "Randomize"),
        row3("Shift+L",    "Loop length",
             ";",          "Page SYN/AMP/FX",
             "F2",         "Toggle synths"),
        row3("< / >",      "Swing ±5%",
             "~",          "Spectrum/VU",
             "",            ""),
        Line::from(Span::raw("")),
        hdr("Patterns & Kits",  "Synth A + B Grid", "File / Project"),
        row3("q w e r t ..",    "Pattern 1-10",
             "Up / Down",       "Pitch ±semitone",
             "Ctrl+S",          "Save project"),
        row3("Shift+above",     "Immediate jump",
             "Shift+Up/Dn",     "Pitch ±octave",
             "Ctrl+O",          "Load project"),
        row3("[ ] / { }",       "Prev/Next pat",
             "( / )",           "Octave down/up",
             "Ctrl+N",          "Rename pattern"),
        row3("1 2 3 .. 8",      "Switch kit 1-8",
             "z x c v ..",      "Synth notes",
             "Ctrl+K",          "Save kit"),
        Line::from(Span::raw("")),
        hdr("Drum Pads",        "Synth Knobs",      "Dual Synth"),
        row3("z x c v b n m ,", "Trigger drums",
             "Arrows",          "Navigate grid",
             "Tab to SA/SB",    "Focus synth"),
        row3("(rec + play)",    "Write at play",
             "Shift+Up/Dn",     "Adjust value",
             "Pat/Kit/Loop",    "Per-synth"),
        row3("",                "",
             "Alt+Up/Dn",       "Adjust+audition",
             "Ctrl+P/L/J/Q",   "File ops"),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
