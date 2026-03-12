//! Entry point: spawns audio thread, runs UI event loop with ~60fps rendering.

mod app;
mod audio;
mod keys;
mod messages;
mod mouse;
mod params;
mod presets;
mod sequencer;
mod ui;

use std::env;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

/// Sets up cross-thread channels, starts the audio stream, initializes the
/// terminal, and enters the main UI loop. The audio stream is kept alive
/// until this function returns.
fn main() -> io::Result<()> {
    // Handle --version / -V flag
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("textstep {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Create channels for UI <-> Audio communication
    let (tx_to_audio, rx_from_ui) = crossbeam_channel::bounded(64);
    let (tx_to_ui, rx_from_audio) = crossbeam_channel::bounded(16);

    // Shared display buffer for waveform/VU meter visualization
    let display_buf = Arc::new(audio::display_buffer::AudioDisplayBuffer::new());

    // Start the audio stream (must keep _stream alive)
    let _stream = match audio::start_audio_stream(rx_from_ui, tx_to_ui, Arc::clone(&display_buf)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Audio error: {e}");
            return Ok(());
        }
    };

    // Initialize terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the app
    let mut app = app::App::new(tx_to_audio, rx_from_audio, display_buf);

    // Main event loop
    loop {
        // Advance splash animation or normal tick
        let term_size = terminal.size().unwrap_or_default();
        if app.ui.splash.tick(term_size.width, term_size.height) {
            // Splash is active — render splash and check for skip key
            terminal.draw(|f| ui::render(f, &app))?;

            while event::poll(Duration::from_millis(0))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        app.ui.splash.skip();
                    }
                    Event::Mouse(mouse_event) => {
                        if matches!(mouse_event.kind, crossterm::event::MouseEventKind::Down(_)) {
                            app.ui.splash.skip();
                        }
                    }
                    _ => {}
                }
            }
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // Drain audio messages
        app.tick();

        // Render
        terminal.draw(|f| ui::render(f, &app))?;

        // Drain all pending input events (keeps key repeat responsive)
        let term_size = terminal.size().unwrap_or_default();
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    // Only handle Press and Repeat, ignore Release
                    if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat {
                        keys::handle_key(&mut app, key);
                    }
                }
                Event::Mouse(mouse_event) => {
                    let size = ratatui::layout::Rect::new(0, 0, term_size.width, term_size.height);
                    mouse::handle_mouse(&mut app, mouse_event, size);
                }
                _ => {}
            }
        }
        // Sleep to target ~60fps if no events pending
        std::thread::sleep(Duration::from_millis(16));

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
