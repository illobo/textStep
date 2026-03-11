// App struct, UiState, handle_key dispatch

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};

use crate::audio::display_buffer::AudioDisplayBuffer;

use crate::messages::{AudioToUi, SynthId, UiToAudio};
use crate::params::EffectParams;
use crate::sequencer::drum_pattern::{DrumPattern, NUM_DRUM_TRACKS};
use crate::sequencer::project::{self, ProjectFile, NUM_KITS, NUM_PATTERNS};
use crate::sequencer::synth_pattern::{SynthControlField, SynthPattern};
use crate::sequencer::transport::Transport;

// ── Focus & field enums ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    DrumGrid,
    Knobs,
    SynthAGrid,      // was: SynthGrid
    SynthAControls,  // was: SynthControls
    SynthBGrid,      // new
    SynthBControls,  // new
    Transport,
}

impl FocusSection {
    pub fn next(&self, vis: &PanelVisibility) -> Self {
        use FocusSection::*;
        let order = [
            Transport, SynthAControls, SynthAGrid,
            SynthBControls, SynthBGrid, DrumGrid, Knobs,
        ];
        let cur = order.iter().position(|s| s == self).unwrap_or(0);
        for i in 1..=order.len() {
            let candidate = order[(cur + i) % order.len()];
            if candidate.is_visible(vis) {
                return candidate;
            }
        }
        Transport
    }

    pub fn prev(&self, vis: &PanelVisibility) -> Self {
        use FocusSection::*;
        let order = [
            Transport, SynthAControls, SynthAGrid,
            SynthBControls, SynthBGrid, DrumGrid, Knobs,
        ];
        let cur = order.iter().position(|s| s == self).unwrap_or(0);
        for i in 1..=order.len() {
            let candidate = order[(cur + order.len() - i) % order.len()];
            if candidate.is_visible(vis) {
                return candidate;
            }
        }
        Transport
    }

    pub fn is_visible(&self, vis: &PanelVisibility) -> bool {
        use FocusSection::*;
        match self {
            Transport => true,
            SynthAControls => vis.synth_a_knobs,
            SynthAGrid => vis.synth_a_grid,
            SynthBControls => vis.synth_b_knobs,
            SynthBGrid => vis.synth_b_grid,
            DrumGrid => vis.drum_grid,
            Knobs => vis.drum_knobs,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamPage {
    Synth, // tune, sweep, color, snap
    Amp,   // filter, drive, decay, volume
    Fx,    // send_reverb, send_delay
}

impl ParamPage {
    pub fn cycle(self) -> Self {
        match self {
            ParamPage::Synth => ParamPage::Amp,
            ParamPage::Amp => ParamPage::Fx,
            ParamPage::Fx => ParamPage::Synth,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ParamPage::Synth => "SYN",
            ParamPage::Amp => "AMP",
            ParamPage::Fx => "FX",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrumControlField {
    // Synth page
    Tune,
    Sweep,
    Color,
    Snap,
    // Amp page
    Filter,
    Drive,
    Decay,
    Volume,
    // FX page
    SendReverb,
    SendDelay,
    Pan,
    // Always visible
    Mute,
    Solo,
}

/// All 11 continuous params in knobs panel order.
pub const KNOB_FIELDS: [DrumControlField; 11] = [
    DrumControlField::Tune,
    DrumControlField::Sweep,
    DrumControlField::Color,
    DrumControlField::Snap,
    DrumControlField::Filter,
    DrumControlField::Drive,
    DrumControlField::Decay,
    DrumControlField::Volume,
    DrumControlField::SendReverb,
    DrumControlField::SendDelay,
    DrumControlField::Pan,
];

impl DrumControlField {
    /// Index (0-9) within the KNOB_FIELDS array, or None for Mute/Solo.
    pub fn knob_index(self) -> Option<usize> {
        KNOB_FIELDS.iter().position(|&f| f == self)
    }

    /// Next field in the knobs panel (wraps around all 10).
    pub fn next_knob(self) -> Self {
        match self.knob_index() {
            Some(i) => KNOB_FIELDS[(i + 1) % KNOB_FIELDS.len()],
            None => KNOB_FIELDS[0],
        }
    }

    /// Previous field in the knobs panel (wraps around all 10).
    pub fn prev_knob(self) -> Self {
        match self.knob_index() {
            Some(i) => KNOB_FIELDS[(i + KNOB_FIELDS.len() - 1) % KNOB_FIELDS.len()],
            None => KNOB_FIELDS[0],
        }
    }

    #[allow(dead_code)]
    pub fn page(self) -> Option<ParamPage> {
        match self {
            Self::Tune | Self::Sweep | Self::Color | Self::Snap => Some(ParamPage::Synth),
            Self::Filter | Self::Drive | Self::Decay | Self::Volume => Some(ParamPage::Amp),
            Self::SendReverb | Self::SendDelay | Self::Pan => Some(ParamPage::Fx),
            Self::Mute | Self::Solo => None,
        }
    }

    /// Fields visible on the given page, in order.
    pub fn fields_for_page(page: ParamPage) -> &'static [DrumControlField] {
        match page {
            ParamPage::Synth => &[Self::Tune, Self::Sweep, Self::Color, Self::Snap, Self::Mute, Self::Solo],
            ParamPage::Amp => &[Self::Filter, Self::Drive, Self::Decay, Self::Volume, Self::Mute, Self::Solo],
            ParamPage::Fx => &[Self::SendReverb, Self::SendDelay, Self::Pan, Self::Mute, Self::Solo],
        }
    }

    /// Next field within the visible set for the current page.
    pub fn next(self, page: ParamPage) -> Self {
        let fields = Self::fields_for_page(page);
        let idx = fields.iter().position(|&f| f == self).unwrap_or(0);
        fields[(idx + 1) % fields.len()]
    }

    /// Previous field within the visible set for the current page.
    pub fn prev(self, page: ParamPage) -> Self {
        let fields = Self::fields_for_page(page);
        let idx = fields.iter().position(|&f| f == self).unwrap_or(0);
        fields[(idx + fields.len() - 1) % fields.len()]
    }

    /// First field for a given page.
    pub fn first_for_page(page: ParamPage) -> Self {
        Self::fields_for_page(page)[0]
    }

    /// Last field for a given page.
    pub fn last_for_page(page: ParamPage) -> Self {
        let fields = Self::fields_for_page(page);
        fields[fields.len() - 1]
    }
}

// ── UI State ────────────────────────────────────────────────────────────────

/// Frames a trigger flash lasts (~100ms at 60fps)
const FLASH_FRAMES: u8 = 6;

/// Splash screen animation phase
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplashPhase {
    /// Logo sliding in from the left
    SlideIn,
    /// Logo resting at center for a beat
    Hold,
    /// Matrix rain effect revealing the UI
    MatrixReveal,
    /// Splash done, show main UI
    Done,
}

pub struct SplashState {
    pub phase: SplashPhase,
    /// Current animation frame counter
    pub frame: u16,
    /// Matrix rain column states: (current_row, speed, char)
    pub matrix_columns: Vec<(f32, f32, u8)>,
    /// Which cells have been "revealed" (row-major: row * width + col)
    pub revealed: Vec<bool>,
    pub matrix_width: u16,
    pub matrix_height: u16,
}

impl SplashState {
    pub fn new() -> Self {
        Self {
            phase: SplashPhase::SlideIn,
            frame: 0,
            matrix_columns: Vec::new(),
            revealed: Vec::new(),
            matrix_width: 0,
            matrix_height: 0,
        }
    }

    /// Initialize the matrix effect for the given terminal size.
    fn init_matrix(&mut self, width: u16, height: u16) {
        self.matrix_width = width;
        self.matrix_height = height;
        self.revealed = vec![false; (width as usize) * (height as usize)];

        // Seed from frame counter + a constant for variety
        let mut rng: u32 = 0xDEAD_BEEF;
        let mut rand_next = || -> u32 {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            rng
        };

        self.matrix_columns.clear();
        for _ in 0..width {
            let speed = 0.8 + (rand_next() % 100) as f32 / 100.0 * 2.0; // 0.8-2.8 rows/frame (fast)
            let start = -((rand_next() % (height as u32 / 2 + 5)) as f32); // tight stagger
            let ch = (rand_next() % 94 + 33) as u8; // printable ASCII
            self.matrix_columns.push((start, speed, ch));
        }
    }

    /// Advance the splash animation by one frame. Returns true while still active.
    pub fn tick(&mut self, term_width: u16, term_height: u16) -> bool {
        match self.phase {
            SplashPhase::SlideIn => {
                self.frame += 1;
                if self.frame >= 60 {
                    self.phase = SplashPhase::Hold;
                    self.frame = 0;
                }
                true
            }
            SplashPhase::Hold => {
                self.frame += 1;
                if self.frame >= 60 {
                    self.phase = SplashPhase::MatrixReveal;
                    self.frame = 0;
                    self.init_matrix(term_width, term_height);
                }
                true
            }
            SplashPhase::MatrixReveal => {
                self.frame += 1;
                // Advance each column's rain drop
                let w = self.matrix_width as usize;
                let h = self.matrix_height as usize;
                for (col, (row, speed, ch)) in self.matrix_columns.iter_mut().enumerate() {
                    *row += *speed;
                    // Cycle the character
                    *ch = (*ch).wrapping_add(1);
                    if *ch < 33 || *ch > 126 { *ch = 33; }
                    // Mark cells as revealed up to current row
                    let cur_row = *row as usize;
                    if cur_row < h {
                        for r in 0..=cur_row {
                            if r < h {
                                self.revealed[r * w + col] = true;
                            }
                        }
                    } else {
                        // Column fully done, reveal all
                        for r in 0..h {
                            self.revealed[r * w + col] = true;
                        }
                    }
                }
                // Done when all cells revealed (or after timeout)
                let all_revealed = self.revealed.iter().all(|&r| r);
                if all_revealed || self.frame >= 60 {
                    self.phase = SplashPhase::Done;
                }
                true
            }
            SplashPhase::Done => false,
        }
    }

    /// Skip straight to done (any key pressed).
    pub fn skip(&mut self) {
        self.phase = SplashPhase::Done;
    }
}

/// Modal overlay state for save/load dialogs.
#[derive(Clone, Debug, PartialEq)]
pub enum ModalState {
    None,
    /// Text input prompt (e.g., "Save project as:")
    TextInput {
        prompt: String,
        buffer: String,
        on_confirm: ModalAction,
    },
    /// File picker list
    FilePicker {
        title: String,
        items: Vec<(String, PathBuf)>,
        selected: usize,
        on_confirm: ModalAction,
    },
    /// Sound preset browser
    PresetBrowser(crate::presets::PresetBrowserState),
    /// Pattern preset browser
    PatternBrowser(crate::presets::PatternBrowserState),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModalAction {
    SaveProject,
    RenamePattern,
    SaveKit,
    LoadProject,
    LoadKit,
}

// ── Mouse State ──────────────────────────────────────────────────────────────

/// State for drag-to-adjust-parameter interaction.
#[derive(Clone, Debug)]
pub struct DragState {
    pub track: usize,
    pub field: DrumControlField,
    pub start_y: u16,
    pub start_value: f32,
}

/// State for compressor knob drag.
#[derive(Clone, Debug)]
pub struct CompressorDrag {
    pub start_y: u16,
    pub start_value: f32,
}

/// State for volume fader drag.
#[derive(Clone, Debug)]
pub struct FaderDrag {
    pub kind: FaderKind,
    pub start_y: u16,
    pub start_value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FaderKind {
    Drum,
    Synth,
}

/// State for synth knob drag.
#[derive(Clone, Debug)]
pub struct SynthDrag {
    pub synth_id: crate::messages::SynthId,
    pub field: crate::sequencer::synth_pattern::SynthControlField,
    pub start_y: u16,
    pub start_value: f32,
}

/// State for synth note length drag (horizontal resize).
#[derive(Clone, Debug)]
pub struct SynthNoteDrag {
    pub synth_id: crate::messages::SynthId,
    pub step: usize,
    pub original_length: u8,
    pub start_col: u16,
}

/// Mouse interaction state (double-click detection, drag).
pub struct MouseState {
    /// Last left-click: (time, track, step) for double-click detection on the grid.
    pub last_click: Option<(Instant, usize, usize)>,
    /// Active drum parameter drag.
    pub drag: Option<DragState>,
    /// Active compressor knob drag.
    pub compressor_drag: Option<CompressorDrag>,
    /// Active volume fader drag.
    pub fader_drag: Option<FaderDrag>,
    /// Active synth knob drag.
    pub synth_drag: Option<SynthDrag>,
    /// Active synth note length drag.
    pub synth_note_drag: Option<SynthNoteDrag>,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            last_click: None,
            drag: None,
            fader_drag: None,
            compressor_drag: None,
            synth_drag: None,
            synth_note_drag: None,
        }
    }
}

/// A brief status message shown at the bottom.
pub struct StatusMessage {
    pub text: String,
    pub frames_remaining: u16,
}

#[derive(Clone, Debug)]
pub struct PanelVisibility {
    pub synth_a_knobs: bool,
    pub synth_a_grid: bool,
    pub synth_b_knobs: bool,
    pub synth_b_grid: bool,
    pub drum_grid: bool,
    pub drum_knobs: bool,
    pub waveform: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            synth_a_knobs: true,
            synth_a_grid: true,
            synth_b_knobs: false,  // Synth B collapsed by default
            synth_b_grid: false,
            drum_grid: true,
            drum_knobs: true,
            waveform: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SynthUiState {
    pub playback_step: usize,
    pub cursor_step: usize,
    pub ctrl_field: SynthControlField,
    pub flash: u8,
    pub octave: u8,
    pub active_pattern: usize,
    pub queued_pattern: Option<usize>,
    pub active_kit: usize,
}

impl Default for SynthUiState {
    fn default() -> Self {
        Self {
            playback_step: 0,
            cursor_step: 0,
            ctrl_field: SynthControlField::Osc1Waveform,
            flash: 0,
            octave: 4,
            active_pattern: 0,
            queued_pattern: None,
            active_kit: 0,
        }
    }
}

pub struct UiState {
    pub splash: SplashState,
    pub focus: FocusSection,
    pub drum_cursor_track: usize,
    pub drum_cursor_step: usize,
    pub drum_ctrl_track: usize,
    pub drum_ctrl_field: DrumControlField,
    pub param_page: ParamPage,
    pub playback_step: usize,
    pub current_beat: u8,
    pub is_bar_start: bool,
    pub show_help: bool,
    pub show_waveform: bool,
    pub panel_vis: PanelVisibility,
    /// Per-track trigger flash countdown (> 0 means flashing)
    pub trigger_flash: [u8; NUM_DRUM_TRACKS],
    /// Current active pattern index (0-9)
    pub active_pattern: usize,
    /// Queued pattern to switch to at end of loop (None = no change pending)
    pub queued_pattern: Option<usize>,
    /// Current active kit index (0-7)
    pub active_kit: usize,
    /// Modal overlay
    pub modal: ModalState,
    /// Brief status message
    pub status_msg: Option<StatusMessage>,
    /// Mouse interaction state
    pub mouse: MouseState,
    /// Scope bar heights with peak hold (0.0-1.0), updated each tick
    pub scope_bars: Vec<f32>,
    /// Scope bar intensity/brightness (0.0-1.0), decays faster than bars for glow effect
    pub scope_intensity: Vec<f32>,
    // ── Synth state ──
    pub synth_a: SynthUiState,
    pub synth_b: SynthUiState,
}

impl UiState {
    /// Set a trigger flash for a specific track.
    pub fn flash_track(&mut self, track: usize) {
        if track < NUM_DRUM_TRACKS {
            self.trigger_flash[track] = FLASH_FRAMES;
        }
    }

    /// Decay all flash counters (call once per frame).
    pub fn decay_flashes(&mut self) {
        for f in &mut self.trigger_flash {
            *f = f.saturating_sub(1);
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            splash: SplashState::new(),
            focus: FocusSection::DrumGrid,
            drum_cursor_track: 0,
            drum_cursor_step: 0,
            drum_ctrl_track: 0,
            drum_ctrl_field: DrumControlField::Tune,
            param_page: ParamPage::Synth,
            playback_step: 0,
            current_beat: 0,
            is_bar_start: false,
            show_help: false,
            show_waveform: true,
            panel_vis: PanelVisibility::default(),
            trigger_flash: [0; NUM_DRUM_TRACKS],
            active_pattern: 0,
            queued_pattern: None,
            active_kit: 0,
            modal: ModalState::None,
            status_msg: None,
            mouse: MouseState::default(),
            scope_bars: Vec::new(),
            scope_intensity: Vec::new(),
            synth_a: SynthUiState::default(),
            synth_b: SynthUiState::default(),
        }
    }
}

// ── App ─────────────────────────────────────────────────────────────────────

pub struct App {
    pub ui: UiState,
    pub transport: Transport,
    pub drum_pattern: DrumPattern,
    pub synth_a_pattern: SynthPattern,
    pub synth_b_pattern: SynthPattern,
    pub effect_params: EffectParams,
    pub project: ProjectFile,
    pub project_path: Option<PathBuf>,
    pub dirty: bool,
    pub display_buf: Arc<AudioDisplayBuffer>,
    pub tx_to_audio: Sender<UiToAudio>,
    pub rx_from_audio: Receiver<AudioToUi>,
    pub should_quit: bool,
}

impl App {
    pub fn new(tx: Sender<UiToAudio>, rx: Receiver<AudioToUi>, display_buf: Arc<AudioDisplayBuffer>) -> Self {
        let project = project::demo_project();
        let mut drum_pattern = DrumPattern::default();
        project.apply_kit_to_pattern(0, &mut drum_pattern);
        project.load_pattern_steps(0, &mut drum_pattern);

        let mut transport = Transport::default();
        transport.bpm = project.bpm;
        transport.loop_config.drum_length = project.loop_length;
        transport.swing = project.swing;
        // Apply per-pattern BPM if set
        if let Some(pat) = project.patterns.first() {
            if pat.bpm > 0.0 {
                transport.bpm = pat.bpm;
            }
        }

        let mut synth_a_pattern = SynthPattern::default();

        // Load startup presets: "Four on the Floor" drum + "Techno 2" synth
        {
            use crate::presets::pattern_presets::PATTERN_PRESETS;
            use crate::presets::synth_pattern_presets::SYNTH_PATTERN_PRESETS;
            use crate::sequencer::project::hex_to_steps;

            if let Some(preset) = PATTERN_PRESETS.iter().find(|p| p.name == "Four on the Floor") {
                for (track, hex) in preset.steps.iter().enumerate() {
                    let src = hex_to_steps(hex);
                    // Find effective source length, rounded to 8-step boundary
                    let src_len = src.iter().rposition(|&s| s).map_or(16, |i| {
                        ((i / 8) + 1) * 8
                    }).min(crate::sequencer::drum_pattern::MAX_STEPS);
                    // Tile to fill all 32 steps
                    for s in 0..crate::sequencer::drum_pattern::MAX_STEPS {
                        drum_pattern.steps[track][s] = src[s % src_len];
                    }
                }
            }

            if let Some(preset) = SYNTH_PATTERN_PRESETS.iter().find(|p| p.name == "Techno 2") {
                for (i, &(note, vel, len)) in preset.steps.iter().enumerate() {
                    synth_a_pattern.steps[i].note = note;
                    synth_a_pattern.steps[i].velocity = vel;
                    synth_a_pattern.steps[i].length = len;
                }
            }
        }

        // Send initial state to audio thread so it has the pattern from the start
        let _ = tx.send(UiToAudio::SetTransport(transport));
        let _ = tx.send(UiToAudio::SetDrumPattern(drum_pattern.clone()));
        let _ = tx.send(UiToAudio::SetSynthPattern(SynthId::A, synth_a_pattern.clone()));

        Self {
            ui: UiState::default(),
            transport,
            drum_pattern,
            synth_a_pattern,
            synth_b_pattern: SynthPattern::default(),
            effect_params: EffectParams::default(),
            project,
            project_path: None,
            dirty: false,
            display_buf,
            tx_to_audio: tx,
            rx_from_audio: rx,
            should_quit: false,
        }
    }

    /// Number of frequency bands in the spectrum display.
    const SCOPE_NUM_BARS: usize = 64;
    /// FFT size (must be power of 2). 4096 gives ~11.7Hz resolution at 48kHz.
    const FFT_SIZE: usize = 4096;
    /// Spectrum frequency range.
    const FREQ_LO: f32 = 40.0;
    const FREQ_HI: f32 = 20000.0;
    /// Assumed sample rate for FFT bin→frequency mapping.
    const SAMPLE_RATE: f32 = 48000.0;
    /// Decay factor per frame for bar height (~60fps). 0.92 = bars hold ~0.5s.
    const SCOPE_DECAY: f32 = 0.92;
    /// Faster decay for brightness/intensity. 0.80 = glow fades in ~0.2s.
    const SCOPE_INTENSITY_DECAY: f32 = 0.80;

    /// Drain incoming messages from the audio thread and update UI state.
    pub fn tick(&mut self) {
        // Decay trigger flashes each frame
        self.ui.decay_flashes();
        self.ui.synth_a.flash = self.ui.synth_a.flash.saturating_sub(1);

        // Update scope bars (only when visible to avoid unnecessary work)
        if self.ui.show_waveform {
            self.update_scope_bars();
        }

        // Decay status message
        if let Some(ref mut msg) = self.ui.status_msg {
            msg.frames_remaining = msg.frames_remaining.saturating_sub(1);
            if msg.frames_remaining == 0 {
                self.ui.status_msg = None;
            }
        }

        while let Ok(msg) = self.rx_from_audio.try_recv() {
            match msg {
                AudioToUi::PlaybackPosition {
                    global_step,
                    beat,
                    is_bar_start,
                    triggered,
                    synth_a_triggered,
                    drum_step,
                    synth_a_step,
                    synth_b_step,
                    synth_b_triggered,
                } => {
                    self.ui.playback_step = drum_step;
                    self.ui.synth_a.playback_step = synth_a_step;
                    self.ui.synth_b.playback_step = synth_b_step;
                    self.ui.current_beat = beat;
                    self.ui.is_bar_start = is_bar_start;

                    // Check for queued drum pattern switch at loop wrap (step 0)
                    if drum_step == 0 && global_step > 0 {
                        if let Some(next) = self.ui.queued_pattern.take() {
                            self.switch_pattern(next);
                        }
                    }

                    // Check for queued synth A pattern switch at loop wrap (step 0)
                    if synth_a_step == 0 && global_step > 0 {
                        if let Some(next) = self.ui.synth_a.queued_pattern.take() {
                            self.switch_synth_pattern(next);
                        }
                    }

                    // Flash triggered tracks
                    for track in 0..NUM_DRUM_TRACKS {
                        if triggered & (1 << track) != 0 {
                            self.ui.flash_track(track);
                        }
                    }

                    // Flash synths
                    if synth_a_triggered {
                        self.ui.synth_a.flash = FLASH_FRAMES;
                    }
                    if synth_b_triggered {
                        self.ui.synth_b.flash = FLASH_FRAMES;
                    }
                }
            }
        }
    }

    /// Update scope bars using FFT spectrum analysis with peak hold and decay.
    fn update_scope_bars(&mut self) {
        use crate::audio::fft;

        let num_bars = Self::SCOPE_NUM_BARS;

        // Ensure vecs are the right size
        if self.ui.scope_bars.len() != num_bars {
            self.ui.scope_bars.resize(num_bars, 0.0);
            self.ui.scope_intensity.resize(num_bars, 0.0);
        }

        // Read samples and run FFT
        let mut re = vec![0.0f32; Self::FFT_SIZE];
        self.display_buf.read_waveform(&mut re);
        fft::hann_window(&mut re);

        let mut im = vec![0.0f32; Self::FFT_SIZE];
        fft::fft(&mut re, &mut im);

        // Get magnitude spectrum (first half = positive frequencies)
        let half = Self::FFT_SIZE / 2;
        let mut mag_db = vec![0.0f32; half];
        fft::magnitude_db(&re, &im, &mut mag_db);

        // Map to logarithmic frequency bands
        let bands = fft::bins_to_log_bands(
            &mag_db,
            num_bars,
            Self::SAMPLE_RATE,
            Self::FREQ_LO,
            Self::FREQ_HI,
            Self::FFT_SIZE,
        );

        // Convert dB to 0.0-1.0 display range (-80dB = 0.0, 0dB = 1.0)
        for bar in 0..num_bars {
            let normalized = ((bands[bar] + 80.0) / 80.0).clamp(0.0, 1.0);

            // Peak hold: keep the higher of current level or decayed previous
            let held = self.ui.scope_bars[bar] * Self::SCOPE_DECAY;
            let new_bar = normalized.max(held);

            // Intensity: jump to 1.0 on fresh peak, otherwise decay faster
            if normalized > self.ui.scope_bars[bar] * 0.9 && normalized > 0.02 {
                self.ui.scope_intensity[bar] = 1.0;
            } else {
                self.ui.scope_intensity[bar] *= Self::SCOPE_INTENSITY_DECAY;
            }

            self.ui.scope_bars[bar] = new_bar;
        }
    }

    /// Send current transport state to the audio thread.
    pub fn send_transport(&self) {
        let _ = self.tx_to_audio.send(UiToAudio::SetTransport(self.transport));
    }

    /// Send the full drum pattern to the audio thread.
    pub fn send_drum_pattern(&self) {
        let _ = self
            .tx_to_audio
            .send(UiToAudio::SetDrumPattern(self.drum_pattern.clone()));
    }

    /// Send the synth pattern to the audio thread for the specified synth.
    pub fn send_synth_pattern(&self, synth_id: SynthId) {
        let pattern = match synth_id {
            SynthId::A => &self.synth_a_pattern,
            SynthId::B => &self.synth_b_pattern,
        };
        let _ = self
            .tx_to_audio
            .send(UiToAudio::SetSynthPattern(synth_id, pattern.clone()));
    }

    /// Send effect params to the audio thread.
    pub fn send_effect_params(&self) {
        let _ = self
            .tx_to_audio
            .send(UiToAudio::SetEffectParams(self.effect_params));
    }

    /// Get the name of the current active pattern.
    pub fn current_pattern_name(&self) -> &str {
        self.project.patterns.get(self.ui.active_pattern)
            .map(|p| p.name.as_str())
            .unwrap_or("?")
    }

    /// Open rename dialog for the current pattern.
    pub fn open_rename_pattern(&mut self) {
        let current_name = self.current_pattern_name().to_string();
        self.ui.modal = ModalState::TextInput {
            prompt: format!("Rename pattern {}:", self.ui.active_pattern + 1),
            buffer: current_name,
            on_confirm: ModalAction::RenamePattern,
        };
    }

    /// Apply a new name to the current pattern.
    pub fn rename_current_pattern(&mut self, name: &str) {
        let idx = self.ui.active_pattern;
        if let Some(pat) = self.project.patterns.get_mut(idx) {
            pat.name = name.to_string();
            self.dirty = true;
        }
    }

    // ── Pattern management ──────────────────────────────────────────────

    /// Save current pattern steps and kit params into the project.
    fn store_current_to_project(&mut self) {
        let idx = self.ui.active_pattern;
        self.project.save_pattern_steps(idx, &self.drum_pattern);
        self.project.save_kit_from_pattern(self.ui.active_kit, &self.drum_pattern);
        self.project.active_kit = self.ui.active_kit;
        self.project.bpm = self.transport.bpm;
        self.project.loop_length = self.transport.loop_config.drum_length;
        self.project.swing = self.transport.swing;
        self.project.effects = self.effect_params;
        // Save per-pattern BPM
        if let Some(pat) = self.project.patterns.get_mut(idx) {
            pat.bpm = self.transport.bpm;
        }
        // Save synth pattern and kit
        self.project.save_synth_pattern(self.ui.synth_a.active_pattern, &self.synth_a_pattern);
        self.project.save_synth_kit(self.ui.synth_a.active_kit, &self.synth_a_pattern.params);
        self.project.active_synth_pattern = self.ui.synth_a.active_pattern;
        self.project.active_synth_kit = self.ui.synth_a.active_kit;
    }

    /// Switch to a different pattern immediately.
    pub fn switch_pattern(&mut self, index: usize) {
        if index >= NUM_PATTERNS { return; }
        // Save current pattern first
        self.store_current_to_project();
        // Load new pattern
        self.ui.active_pattern = index;
        self.project.active_pattern = index;
        // Clear steps and load from project
        self.drum_pattern.steps = [[false; 32]; NUM_DRUM_TRACKS];
        self.project.load_pattern_steps(index, &mut self.drum_pattern);
        self.send_drum_pattern();
        // Apply per-pattern BPM if set
        if let Some(pat) = self.project.patterns.get(index) {
            if pat.bpm > 0.0 {
                self.transport.bpm = pat.bpm;
                self.send_transport();
            }
        }
    }

    /// Queue a pattern to switch at end of current loop.
    pub fn queue_pattern(&mut self, index: usize) {
        if index >= NUM_PATTERNS { return; }
        if index == self.ui.active_pattern {
            // Pressing the same pattern cancels the queue
            self.ui.queued_pattern = None;
        } else {
            self.ui.queued_pattern = Some(index);
        }
    }

    // ── Synth pattern management ────────────────────────────────────────

    /// Switch to a different synth pattern immediately.
    pub fn switch_synth_pattern(&mut self, index: usize) {
        if index >= NUM_PATTERNS { return; }
        // Save current synth pattern first
        self.project.save_synth_pattern(self.ui.synth_a.active_pattern, &self.synth_a_pattern);
        // Load new synth pattern
        self.ui.synth_a.active_pattern = index;
        self.project.active_synth_pattern = index;
        self.synth_a_pattern = SynthPattern::default();
        self.project.load_synth_pattern(index, &mut self.synth_a_pattern);
        self.send_synth_pattern(SynthId::A);
    }

    /// Queue a synth pattern to switch at end of current loop.
    pub fn queue_synth_pattern(&mut self, index: usize) {
        if index >= NUM_PATTERNS { return; }
        if index == self.ui.synth_a.active_pattern {
            // Pressing the same pattern cancels the queue
            self.ui.synth_a.queued_pattern = None;
        } else {
            self.ui.synth_a.queued_pattern = Some(index);
        }
    }

    /// Switch to a different synth kit immediately.
    pub fn switch_synth_kit(&mut self, index: usize) {
        if index >= NUM_KITS { return; }
        // Save current synth kit params back
        self.project.save_synth_kit(self.ui.synth_a.active_kit, &self.synth_a_pattern.params);
        // Load new synth kit
        self.ui.synth_a.active_kit = index;
        self.project.active_synth_kit = index;
        self.project.load_synth_kit(index, &mut self.synth_a_pattern);
        self.send_synth_pattern(SynthId::A);
    }

    // ── Focus-aware synth helpers (dual synth) ─────────────────────────

    /// Switch to a different synth pattern for the specified synth.
    pub fn switch_synth_pattern_for(&mut self, synth_id: SynthId, index: usize) {
        if index >= NUM_PATTERNS { return; }
        match synth_id {
            SynthId::A => {
                self.project.save_synth_pattern(self.ui.synth_a.active_pattern, &self.synth_a_pattern);
                self.ui.synth_a.active_pattern = index;
                self.project.active_synth_pattern = index;
                self.synth_a_pattern = SynthPattern::default();
                self.project.load_synth_pattern(index, &mut self.synth_a_pattern);
                self.send_synth_pattern(SynthId::A);
            }
            SynthId::B => {
                // For synth B, use synth_b_pattern (project B storage is a future task)
                self.ui.synth_b.active_pattern = index;
                self.synth_b_pattern = SynthPattern::default();
                self.send_synth_pattern(SynthId::B);
            }
        }
    }

    /// Queue a synth pattern for the specified synth.
    pub fn queue_synth_pattern_for(&mut self, synth_id: SynthId, index: usize) {
        if index >= NUM_PATTERNS { return; }
        let ui = match synth_id {
            SynthId::A => &mut self.ui.synth_a,
            SynthId::B => &mut self.ui.synth_b,
        };
        if index == ui.active_pattern {
            ui.queued_pattern = None;
        } else {
            ui.queued_pattern = Some(index);
        }
    }

    /// Switch to a different synth kit for the specified synth.
    pub fn switch_synth_kit_for(&mut self, synth_id: SynthId, index: usize) {
        if index >= NUM_KITS { return; }
        match synth_id {
            SynthId::A => {
                self.project.save_synth_kit(self.ui.synth_a.active_kit, &self.synth_a_pattern.params);
                self.ui.synth_a.active_kit = index;
                self.project.active_synth_kit = index;
                self.project.load_synth_kit(index, &mut self.synth_a_pattern);
                self.send_synth_pattern(SynthId::A);
            }
            SynthId::B => {
                // For synth B, just update UI state (project B storage is a future task)
                self.ui.synth_b.active_kit = index;
                self.send_synth_pattern(SynthId::B);
            }
        }
    }

    /// Show a brief status message.
    pub fn show_status(&mut self, text: String) {
        self.ui.status_msg = Some(StatusMessage {
            text,
            frames_remaining: 120, // ~2 seconds at 60fps
        });
    }

    // ── Save / Load ─────────────────────────────────────────────────────

    pub fn save_project(&mut self) {
        self.store_current_to_project();

        if let Some(ref path) = self.project_path {
            match project::save_project(&self.project, path) {
                Ok(()) => {
                    self.dirty = false;
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("project")
                        .to_string();
                    self.show_status(format!("Saved: {}.tsp", name));
                }
                Err(e) => self.show_status(format!("Save error: {}", e)),
            }
        } else {
            // No path yet — open text input for name
            self.ui.modal = ModalState::TextInput {
                prompt: "Save project as:".to_string(),
                buffer: self.project.metadata.name.clone(),
                on_confirm: ModalAction::SaveProject,
            };
        }
    }

    pub fn save_project_with_name(&mut self, name: &str) {
        self.project.metadata.name = name.to_string();
        let filename = slugify(name);
        let path = project::projects_dir().join(format!("{}.tsp", filename));
        self.project_path = Some(path.clone());
        match project::save_project(&self.project, &path) {
            Ok(()) => {
                self.dirty = false;
                self.show_status(format!("Saved: {}.tsp", filename));
            }
            Err(e) => self.show_status(format!("Save error: {}", e)),
        }
    }

    pub fn open_load_dialog(&mut self) {
        let items = project::list_projects();
        if items.is_empty() {
            self.show_status("No projects found".to_string());
            return;
        }
        self.ui.modal = ModalState::FilePicker {
            title: "Load Project".to_string(),
            items,
            selected: 0,
            on_confirm: ModalAction::LoadProject,
        };
    }

    // ── Kit management ─────────────────────────────────────────────────

    /// Switch to a different kit immediately.
    pub fn switch_kit(&mut self, index: usize) {
        if index >= NUM_KITS { return; }
        // Save current kit params back
        self.project.save_kit_from_pattern(self.ui.active_kit, &self.drum_pattern);
        // Load new kit
        self.ui.active_kit = index;
        self.project.active_kit = index;
        self.project.apply_kit_to_pattern(index, &mut self.drum_pattern);
        self.send_drum_pattern();
    }

    /// Get the name of the current active kit.
    pub fn current_kit_name(&self) -> &str {
        self.project.kits.get(self.ui.active_kit)
            .map(|k| k.name.as_str())
            .unwrap_or("?")
    }

    // ── Kit Save / Load ───────────────────────────────────────────────

    pub fn save_kit(&mut self) {
        self.store_current_to_project();
        let kit = &self.project.kits[self.ui.active_kit];
        let kit_name = &kit.name;
        if kit_name.is_empty() || kit_name.starts_with("Kit ") {
            self.ui.modal = ModalState::TextInput {
                prompt: format!("Save kit {} as:", self.ui.active_kit + 1),
                buffer: kit_name.clone(),
                on_confirm: ModalAction::SaveKit,
            };
        } else {
            let filename = slugify(kit_name);
            let path = project::kits_dir().join(format!("{}.tsk", filename));
            match project::save_kit(kit, &path) {
                Ok(()) => self.show_status(format!("Kit saved: {}.tsk", filename)),
                Err(e) => self.show_status(format!("Kit save error: {}", e)),
            }
        }
    }

    pub fn save_kit_with_name(&mut self, name: &str) {
        self.store_current_to_project();
        self.project.kits[self.ui.active_kit].name = name.to_string();
        let filename = slugify(name);
        let path = project::kits_dir().join(format!("{}.tsk", filename));
        match project::save_kit(&self.project.kits[self.ui.active_kit], &path) {
            Ok(()) => self.show_status(format!("Kit saved: {}.tsk", filename)),
            Err(e) => self.show_status(format!("Kit save error: {}", e)),
        }
    }

    pub fn open_load_kit_dialog(&mut self) {
        let items = project::list_kits();
        if items.is_empty() {
            self.show_status("No kits found".to_string());
            return;
        }
        self.ui.modal = ModalState::FilePicker {
            title: format!("Load Kit into slot {}", self.ui.active_kit + 1),
            items,
            selected: 0,
            on_confirm: ModalAction::LoadKit,
        };
    }

    pub fn load_kit_from_path(&mut self, path: &PathBuf) {
        match project::load_kit(path) {
            Ok(kit) => {
                let name = kit.name.clone();
                let idx = self.ui.active_kit;
                self.project.kits[idx] = kit;
                self.project.apply_kit_to_pattern(idx, &mut self.drum_pattern);
                self.send_drum_pattern();
                self.dirty = true;
                self.show_status(format!("Kit loaded: {}", name));
            }
            Err(e) => self.show_status(format!("Kit load error: {}", e)),
        }
    }

    // ── Preset Browser ─────────────────────────────────────────────────

    pub fn open_preset_browser(&mut self) {
        let is_synth = matches!(self.ui.focus, FocusSection::SynthAGrid | FocusSection::SynthAControls | FocusSection::SynthBGrid | FocusSection::SynthBControls);
        let mut browser = if is_synth {
            crate::presets::PresetBrowserState::for_synth()
        } else {
            crate::presets::PresetBrowserState::for_drum_track(self.ui.drum_cursor_track)
        };
        // Set target synth based on current focus
        if is_synth {
            browser.target_synth = match self.ui.focus {
                FocusSection::SynthBGrid | FocusSection::SynthBControls => SynthId::B,
                _ => SynthId::A,
            };
        }
        self.ui.modal = ModalState::PresetBrowser(browser);
    }

    pub fn apply_drum_preset(&mut self, track: usize, params: crate::sequencer::project::DrumSoundParams) {
        let tp = params.to_track_params();
        let mute = self.drum_pattern.params[track].mute;
        let solo = self.drum_pattern.params[track].solo;
        self.drum_pattern.params[track] = tp;
        self.drum_pattern.params[track].mute = mute;
        self.drum_pattern.params[track].solo = solo;
        self.send_drum_pattern();
        self.dirty = true;
    }

    pub fn open_pattern_browser(&mut self) {
        let is_synth = matches!(self.ui.focus, FocusSection::SynthAGrid | FocusSection::SynthAControls | FocusSection::SynthBGrid | FocusSection::SynthBControls);
        let mut pb = if is_synth {
            crate::presets::PatternBrowserState::new_synth()
        } else {
            crate::presets::PatternBrowserState::new()
        };
        // Set target synth based on current focus
        if is_synth {
            pb.browser.target_synth = match self.ui.focus {
                FocusSection::SynthBGrid | FocusSection::SynthBControls => SynthId::B,
                _ => SynthId::A,
            };
        }
        self.ui.modal = ModalState::PatternBrowser(pb);
    }

    pub fn apply_pattern_preset(
        &mut self,
        steps: &[&str; crate::sequencer::drum_pattern::NUM_DRUM_TRACKS],
        merge: crate::presets::PatternMergeMode,
    ) {
        use crate::sequencer::drum_pattern::MAX_STEPS;
        use crate::sequencer::project::hex_to_steps;

        // Fill up to loop length if enabled, otherwise full 32 steps
        let fill_len = if self.transport.loop_config.enabled {
            self.transport.loop_config.drum_length as usize
        } else {
            MAX_STEPS
        };

        for (track, hex) in steps.iter().enumerate() {
            if track < NUM_DRUM_TRACKS {
                let src = hex_to_steps(hex);

                // Find the effective length of the source pattern (last active step)
                let src_len = src.iter().rposition(|&s| s).map_or(16, |i| {
                    // Round up to nearest 8-step boundary
                    ((i / 8) + 1) * 8
                }).min(MAX_STEPS);

                // Build the tiled pattern
                let mut tiled = [false; MAX_STEPS];
                for s in 0..fill_len {
                    tiled[s] = src[s % src_len];
                }

                match merge {
                    crate::presets::PatternMergeMode::Replace => {
                        self.drum_pattern.steps[track] = tiled;
                    }
                    crate::presets::PatternMergeMode::Layer => {
                        for s in 0..fill_len {
                            if tiled[s] {
                                self.drum_pattern.steps[track][s] = true;
                            }
                        }
                    }
                }
            }
        }
        self.send_drum_pattern();
        self.dirty = true;
    }

    pub fn apply_synth_pattern_preset(
        &mut self,
        synth_id: SynthId,
        preset_steps: &[(u8, u8, u8); crate::sequencer::synth_pattern::MAX_STEPS],
        merge: crate::presets::PatternMergeMode,
    ) {
        use crate::sequencer::synth_pattern::{MAX_STEPS, SynthStep};

        let pattern = match synth_id {
            SynthId::A => &mut self.synth_a_pattern,
            SynthId::B => &mut self.synth_b_pattern,
        };

        let fill_len = if self.transport.loop_config.enabled {
            self.transport.loop_config.synth_a_length as usize
        } else {
            MAX_STEPS
        };

        // Find effective source length (last non-rest step, rounded to 8)
        let src_len = preset_steps.iter().rposition(|s| s.1 > 0).map_or(16, |i| {
            ((i / 8) + 1) * 8
        }).min(MAX_STEPS);

        for s in 0..fill_len {
            let (note, vel, len) = preset_steps[s % src_len];
            if vel > 0 {
                match merge {
                    crate::presets::PatternMergeMode::Replace => {
                        pattern.steps[s] = SynthStep { note, velocity: vel, length: len };
                    }
                    crate::presets::PatternMergeMode::Layer => {
                        if !pattern.steps[s].is_active() {
                            pattern.steps[s] = SynthStep { note, velocity: vel, length: len };
                        }
                    }
                }
            } else if matches!(merge, crate::presets::PatternMergeMode::Replace) {
                pattern.steps[s] = SynthStep::default();
            }
        }
        self.send_synth_pattern(synth_id);
        self.dirty = true;
    }

    pub fn apply_synth_preset(&mut self, synth_id: SynthId, params: crate::sequencer::synth_pattern::SynthParams) {
        let pattern = match synth_id {
            SynthId::A => &mut self.synth_a_pattern,
            SynthId::B => &mut self.synth_b_pattern,
        };
        let mute = pattern.params.mute;
        pattern.params = params;
        pattern.params.mute = mute;
        self.send_synth_pattern(synth_id);
        self.dirty = true;
    }

    // ── Project Load ────────────────────────────────────────────────────

    pub fn load_project_from_path(&mut self, path: &PathBuf) {
        match project::load_project(path) {
            Ok(proj) => {
                let name = proj.metadata.name.clone();
                self.project = proj;
                self.project_path = Some(path.clone());

                // Apply to app state
                self.transport.bpm = self.project.bpm;
                self.transport.loop_config.drum_length = self.project.loop_length;
                self.send_transport();

                let kit_idx = self.project.active_kit;
                self.ui.active_kit = kit_idx;

                let idx = self.project.active_pattern;
                self.ui.active_pattern = idx;
                self.drum_pattern = DrumPattern::default();
                self.project.apply_kit_to_pattern(kit_idx, &mut self.drum_pattern);
                self.project.load_pattern_steps(idx, &mut self.drum_pattern);
                self.send_drum_pattern();

                self.effect_params = self.project.effects;
                self.send_effect_params();

                // Load synth state from project
                self.ui.synth_a.active_pattern = self.project.active_synth_pattern;
                self.ui.synth_a.active_kit = self.project.active_synth_kit;
                self.synth_a_pattern = SynthPattern::default();
                self.project.load_synth_pattern(self.ui.synth_a.active_pattern, &mut self.synth_a_pattern);
                self.project.load_synth_kit(self.ui.synth_a.active_kit, &mut self.synth_a_pattern);
                self.send_synth_pattern(SynthId::A);

                self.dirty = false;
                self.show_status(format!("Loaded: {}", name));
            }
            Err(e) => self.show_status(format!("Load error: {}", e)),
        }
    }
}

/// Convert a project name to a filename-safe slug.
fn slugify(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse multiple dashes
    let mut result = String::new();
    let mut prev_dash = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash && !result.is_empty() {
                result.push(c);
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    result.trim_end_matches('-').to_string()
}
