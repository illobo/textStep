//! Preset browser: categories, merge modes, and state machine for browsing
//! drum/synth sound and pattern presets.

pub mod drum_presets;
pub mod pattern_presets;
pub mod synth_pattern_presets;
pub mod synth_presets;

use crate::sequencer::drum_pattern::DrumTrackId;
use crate::sequencer::project::DrumSoundParams;
use crate::sequencer::synth_pattern::SynthParams;

// ── Drum Sound Preset ────────────────────────────────────────────────────────

pub struct DrumSoundPreset {
    pub name: &'static str,
    pub category: &'static str,
    pub voice: DrumTrackId,
    pub params: DrumSoundParams,
}

// ── Synth Sound Preset ───────────────────────────────────────────────────────

pub struct SynthSoundPreset {
    pub name: &'static str,
    pub category: &'static str,
    pub params: SynthParams,
}

/// Which kind of preset the browser is targeting.
#[derive(Clone, Debug, PartialEq)]
pub enum PresetTarget {
    DrumSound(usize), // track index
    SynthSound,
    Pattern,
    SynthPattern,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PatternMergeMode {
    Replace,
    Layer,
}

/// State machine for the preset browser modal: tracks target, category, and selection index.
#[derive(Clone, Debug, PartialEq)]
pub struct PresetBrowserState {
    pub target: PresetTarget,
    pub target_synth: crate::messages::SynthId, // which synth to apply synth presets to
    pub categories: Vec<&'static str>,
    pub category_idx: usize,
    pub preset_names: Vec<&'static str>,
    pub preset_idx: usize,
}

impl PresetBrowserState {
    pub fn for_drum_track(track: usize) -> Self {
        let voice = crate::sequencer::drum_pattern::TRACK_IDS[track];
        let all = drum_presets::presets_for_voice(voice);
        let categories = drum_presets::categories_for_voice(voice);
        let cat = categories.first().copied().unwrap_or("All");
        let names: Vec<&'static str> = all.iter()
            .filter(|p| p.category == cat)
            .map(|p| p.name)
            .collect();
        Self {
            target: PresetTarget::DrumSound(track),
            target_synth: crate::messages::SynthId::A, // default, not used for drum presets
            categories,
            category_idx: 0,
            preset_names: names,
            preset_idx: 0,
        }
    }

    pub fn for_synth() -> Self {
        let categories = synth_presets::categories();
        let cat = categories.first().copied().unwrap_or("All");
        let names: Vec<&'static str> = synth_presets::SYNTH_PRESETS.iter()
            .filter(|p| p.category == cat)
            .map(|p| p.name)
            .collect();
        Self {
            target: PresetTarget::SynthSound,
            target_synth: crate::messages::SynthId::A, // default, will be set when browser opens
            categories,
            category_idx: 0,
            preset_names: names,
            preset_idx: 0,
        }
    }

    pub fn refresh_presets(&mut self) {
        let cat = self.categories[self.category_idx];
        self.preset_names = match &self.target {
            PresetTarget::DrumSound(track) => {
                let voice = crate::sequencer::drum_pattern::TRACK_IDS[*track];
                drum_presets::presets_for_voice(voice).iter()
                    .filter(|p| p.category == cat)
                    .map(|p| p.name)
                    .collect()
            }
            PresetTarget::SynthSound => {
                synth_presets::SYNTH_PRESETS.iter()
                    .filter(|p| p.category == cat)
                    .map(|p| p.name)
                    .collect()
            }
            PresetTarget::Pattern => {
                pattern_presets::presets_for_genre(cat).iter()
                    .map(|p| p.name)
                    .collect()
            }
            PresetTarget::SynthPattern => {
                synth_pattern_presets::presets_for_genre(cat).iter()
                    .map(|p| p.name)
                    .collect()
            }
        };
        self.preset_idx = 0;
    }

    pub fn selected_drum_params(&self) -> Option<DrumSoundParams> {
        if let PresetTarget::DrumSound(track) = &self.target {
            let voice = crate::sequencer::drum_pattern::TRACK_IDS[*track];
            let cat = self.categories[self.category_idx];
            let filtered: Vec<_> = drum_presets::presets_for_voice(voice).iter()
                .filter(|p| p.category == cat)
                .collect();
            filtered.get(self.preset_idx).map(|p| p.params)
        } else {
            None
        }
    }

    pub fn selected_synth_params(&self) -> Option<SynthParams> {
        if self.target == PresetTarget::SynthSound {
            let cat = self.categories[self.category_idx];
            let filtered: Vec<_> = synth_presets::SYNTH_PRESETS.iter()
                .filter(|p| p.category == cat)
                .collect();
            filtered.get(self.preset_idx).map(|p| p.params)
        } else {
            None
        }
    }

    pub fn for_pattern() -> Self {
        let categories = pattern_presets::genres();
        let cat = categories.first().copied().unwrap_or("All");
        let names: Vec<&'static str> = pattern_presets::presets_for_genre(cat).iter()
            .map(|p| p.name)
            .collect();
        Self {
            target: PresetTarget::Pattern,
            target_synth: crate::messages::SynthId::A, // default, not used for drum patterns
            categories,
            category_idx: 0,
            preset_names: names,
            preset_idx: 0,
        }
    }

    pub fn selected_pattern(&self) -> Option<&'static pattern_presets::PatternPreset> {
        if self.target == PresetTarget::Pattern {
            let cat = self.categories[self.category_idx];
            let filtered = pattern_presets::presets_for_genre(cat);
            filtered.get(self.preset_idx).copied()
        } else {
            None
        }
    }

    pub fn for_synth_pattern() -> Self {
        let categories = synth_pattern_presets::genres();
        let cat = categories.first().copied().unwrap_or("All");
        let names: Vec<&'static str> = synth_pattern_presets::presets_for_genre(cat).iter()
            .map(|p| p.name)
            .collect();
        Self {
            target: PresetTarget::SynthPattern,
            target_synth: crate::messages::SynthId::A, // default, will be set when browser opens
            categories,
            category_idx: 0,
            preset_names: names,
            preset_idx: 0,
        }
    }

    pub fn selected_synth_pattern(&self) -> Option<&'static synth_pattern_presets::SynthPatternPreset> {
        if self.target == PresetTarget::SynthPattern {
            let cat = self.categories[self.category_idx];
            let filtered = synth_pattern_presets::presets_for_genre(cat);
            filtered.get(self.preset_idx).copied()
        } else {
            None
        }
    }
}

// ── Pattern Browser State (extends PresetBrowserState) ───────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct PatternBrowserState {
    pub browser: PresetBrowserState,
    pub merge_mode: PatternMergeMode,
}

impl PatternBrowserState {
    pub fn new() -> Self {
        Self {
            browser: PresetBrowserState::for_pattern(),
            merge_mode: PatternMergeMode::Replace,
        }
    }

    pub fn new_synth() -> Self {
        Self {
            browser: PresetBrowserState::for_synth_pattern(),
            merge_mode: PatternMergeMode::Replace,
        }
    }

    pub fn toggle_merge_mode(&mut self) {
        self.merge_mode = match self.merge_mode {
            PatternMergeMode::Replace => PatternMergeMode::Layer,
            PatternMergeMode::Layer => PatternMergeMode::Replace,
        };
    }
}
