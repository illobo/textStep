// src/ui/layout.rs — Single source of truth for all layout dimensions

use ratatui::layout::Rect;

use crate::app::PanelVisibility;

// ── Dimension constants ──────────────────────────────────────────────────────

/// Transport bar height (title border + 5 content lines + bottom border).
/// Line 1: play state + BPM + beat LEDs + swing + record
/// Line 2-4: Synth A / Synth B / Drum status lines (pattern/kit/loop)
/// Line 5: master gauges
pub const TRANSPORT_HEIGHT: u16 = 7;

/// Height of the drum knobs panel (1 label + 5 bars + 1 value + 2 border).
pub const KNOBS_HEIGHT: u16 = 9;
/// Alias used by the new dual-synth layout.
pub const DRUM_KNOBS_HEIGHT: u16 = KNOBS_HEIGHT;

/// Height of the synth knobs panel (OSC 8 + ENV/FILT 8 + LFO 3 + AMP 7 + 2 border = 30).
pub const SYNTH_KNOBS_HEIGHT: u16 = 30;

/// Height of the synth step row (2 border + header + spacer + step row + spacer = 6).
pub const SYNTH_GRID_HEIGHT: u16 = 6;

/// Minimum height for the drum grid (8 tracks + borders + header).
pub const DRUM_GRID_MIN_HEIGHT: u16 = 11;

/// Height of a collapsed panel (1 top-border + 1 content line showing label).
pub const COLLAPSED_PANEL_HEIGHT: u16 = 2;

/// Height of the waveform/oscilloscope panel (including borders).
pub const WAVEFORM_HEIGHT: u16 = 11;

/// Activity bar (bottom status line).
pub const ACTIVITY_BAR_HEIGHT: u16 = 1;

/// Help panel height.
pub const HELP_HEIGHT: u16 = 22;

// ── Dual-synth layout ────────────────────────────────────────────────────────

/// Pre-computed layout rects for the dual-synth panel system.
///
/// Each panel has two rects: the expanded rect (non-empty when visible) and
/// the collapsed rect (non-empty when collapsed). They are mutually exclusive —
/// when one is set the other is `Rect::default()`.
pub struct DualSynthLayout {
    pub transport: Rect,

    // Synth A
    pub synth_a_knobs: Rect,
    pub synth_a_grid: Rect,
    pub synth_a_knobs_collapsed: Rect,
    pub synth_a_grid_collapsed: Rect,

    // Synth B
    pub synth_b_knobs: Rect,
    pub synth_b_grid: Rect,
    pub synth_b_knobs_collapsed: Rect,
    pub synth_b_grid_collapsed: Rect,

    // Drums
    pub drum_grid: Rect,
    pub drum_knobs: Rect,
    pub drum_knobs_collapsed: Rect,

    // Bottom
    pub waveform: Rect,
    pub waveform_collapsed: Rect,
    pub activity_bar: Rect,
}

/// Describes a single panel slot in the vertical stack.
struct PanelSlot {
    expanded_height: u16,
    is_visible: bool,
    /// If true, this panel receives leftover space when other panels collapse.
    growable: bool,
}

/// Compute layout for the dual-synth panel system.
///
/// Layout order (top to bottom):
///   Transport | Synth A Knobs | Synth A Grid | Synth B Knobs | Synth B Grid
///   | Drum Grid | Drum Knobs | Waveform | Activity Bar
///
/// Collapsed panels get `COLLAPSED_PANEL_HEIGHT` (2 lines).
/// Reclaimed vertical space is given to growable panels (drum_grid).
pub fn compute_dual_layout(total: Rect, vis: &PanelVisibility) -> DualSynthLayout {
    // Fixed sections: transport at top, activity bar at bottom
    let fixed = TRANSPORT_HEIGHT + ACTIVITY_BAR_HEIGHT;

    // Define the 7 collapsible panels in order
    let panels = [
        PanelSlot { expanded_height: SYNTH_KNOBS_HEIGHT,  is_visible: vis.synth_a_knobs, growable: false },
        PanelSlot { expanded_height: SYNTH_GRID_HEIGHT,   is_visible: vis.synth_a_grid,  growable: false },
        PanelSlot { expanded_height: SYNTH_KNOBS_HEIGHT,  is_visible: vis.synth_b_knobs, growable: false },
        PanelSlot { expanded_height: SYNTH_GRID_HEIGHT,   is_visible: vis.synth_b_grid,  growable: false },
        PanelSlot { expanded_height: DRUM_GRID_MIN_HEIGHT, is_visible: vis.drum_grid,    growable: true  },
        PanelSlot { expanded_height: DRUM_KNOBS_HEIGHT,   is_visible: vis.drum_knobs,    growable: false },
        PanelSlot { expanded_height: WAVEFORM_HEIGHT,     is_visible: vis.waveform,      growable: false },
    ];

    // Make a mutable copy of visibility so we can auto-collapse on overflow
    let mut vis_effective: [bool; 7] = [
        panels[0].is_visible, panels[1].is_visible,
        panels[2].is_visible, panels[3].is_visible,
        panels[4].is_visible, panels[5].is_visible,
        panels[6].is_visible,
    ];

    let available = total.height;

    // Auto-collapse panels when terminal is too small, lowest priority first.
    // Priority order for collapsing (first to collapse → last):
    // waveform(6), drum_knobs(5), synth_b_grid(3), synth_b_knobs(2),
    // synth_a_grid(1), synth_a_knobs(0), drum_grid(4)
    let collapse_order: [usize; 7] = [6, 5, 3, 2, 1, 0, 4];
    loop {
        let mut used: u16 = fixed;
        for (i, p) in panels.iter().enumerate() {
            if vis_effective[i] {
                used = used.saturating_add(p.expanded_height);
            } else {
                used = used.saturating_add(COLLAPSED_PANEL_HEIGHT);
            }
        }
        if used <= available {
            break;
        }
        // Find next panel to collapse
        if let Some(&idx) = collapse_order.iter().find(|&&i| vis_effective[i]) {
            vis_effective[idx] = false;
        } else {
            break; // everything collapsed, nothing more we can do
        }
    }

    // Calculate total requested height
    let mut used: u16 = fixed;
    for (i, p) in panels.iter().enumerate() {
        if vis_effective[i] {
            used = used.saturating_add(p.expanded_height);
        } else {
            used = used.saturating_add(COLLAPSED_PANEL_HEIGHT);
        }
    }

    // Compute extra space to distribute to growable panels
    let extra = if available > used { available - used } else { 0 };

    // Count growable visible panels
    let growable_count = panels.iter().enumerate().filter(|(i, p)| vis_effective[*i] && p.growable).count() as u16;
    let extra_per_growable = if growable_count > 0 { extra / growable_count } else { 0 };
    let mut extra_remainder = if growable_count > 0 { extra % growable_count } else { 0 };

    // Assign heights for each panel
    let mut heights: [u16; 7] = [0; 7];
    for (i, p) in panels.iter().enumerate() {
        if vis_effective[i] {
            heights[i] = p.expanded_height;
            if p.growable {
                heights[i] += extra_per_growable;
                if extra_remainder > 0 {
                    heights[i] += 1;
                    extra_remainder -= 1;
                }
            }
        } else {
            heights[i] = COLLAPSED_PANEL_HEIGHT;
        }
    }

    // Build rects by walking y offsets, clamping to terminal bounds
    let x = total.x;
    let w = total.width;
    let y_max = total.y + total.height; // first row outside the buffer
    let mut y = total.y;

    // Transport
    let transport_h = TRANSPORT_HEIGHT.min(y_max.saturating_sub(y));
    let transport = Rect::new(x, y, w, transport_h);
    y += transport_h;

    // Helper: allocate a panel rect and advance y, clamping to bounds
    let mut panel_rects: [(Rect, Rect); 7] = [(Rect::default(), Rect::default()); 7];
    for i in 0..panels.len() {
        let h = heights[i].min(y_max.saturating_sub(y));
        if h == 0 {
            continue;
        }
        let rect = Rect::new(x, y, w, h);
        if vis_effective[i] {
            panel_rects[i] = (rect, Rect::default());
        } else {
            panel_rects[i] = (Rect::default(), rect);
        }
        y += h;
    }

    // Activity bar at the bottom (gets whatever is left, may be 0)
    let activity_h = ACTIVITY_BAR_HEIGHT.min(y_max.saturating_sub(y));
    let activity_bar = Rect::new(x, y, w, activity_h);

    DualSynthLayout {
        transport,

        synth_a_knobs:           panel_rects[0].0,
        synth_a_knobs_collapsed: panel_rects[0].1,
        synth_a_grid:            panel_rects[1].0,
        synth_a_grid_collapsed:  panel_rects[1].1,

        synth_b_knobs:           panel_rects[2].0,
        synth_b_knobs_collapsed: panel_rects[2].1,
        synth_b_grid:            panel_rects[3].0,
        synth_b_grid_collapsed:  panel_rects[3].1,

        drum_grid:               panel_rects[4].0,
        drum_knobs:              panel_rects[5].0,
        drum_knobs_collapsed:    panel_rects[5].1,

        waveform:                panel_rects[6].0,
        waveform_collapsed:      panel_rects[6].1,
        activity_bar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::PanelVisibility;

    fn term(h: u16) -> Rect {
        Rect::new(0, 0, 120, h)
    }

    #[test]
    fn all_expanded_fills_terminal() {
        let vis = PanelVisibility {
            synth_a_knobs: true,
            synth_a_grid: true,
            synth_b_knobs: true,
            synth_b_grid: true,
            drum_grid: true,
            drum_knobs: true,
            waveform: true,
        };
        // Minimum needed: 7 + 30 + 6 + 30 + 6 + 11 + 9 + 11 + 1 = 111
        let ly = compute_dual_layout(term(111), &vis);

        // Transport and activity bar should be at expected positions
        assert_eq!(ly.transport.height, TRANSPORT_HEIGHT);
        assert_eq!(ly.activity_bar.height, ACTIVITY_BAR_HEIGHT);
        assert_eq!(ly.activity_bar.y + ly.activity_bar.height, 111);

        // All expanded rects should be non-empty
        assert!(ly.synth_a_knobs.height > 0);
        assert!(ly.synth_a_grid.height > 0);
        assert!(ly.synth_b_knobs.height > 0);
        assert!(ly.synth_b_grid.height > 0);
        assert!(ly.drum_grid.height > 0);
        assert!(ly.drum_knobs.height > 0);
        assert!(ly.waveform.height > 0);

        // All collapsed rects should be empty
        assert_eq!(ly.synth_a_knobs_collapsed, Rect::default());
        assert_eq!(ly.synth_b_grid_collapsed, Rect::default());
        assert_eq!(ly.waveform_collapsed, Rect::default());
    }

    #[test]
    fn collapsed_panels_give_space_to_drum_grid() {
        let vis_expanded = PanelVisibility {
            synth_a_knobs: true,
            synth_a_grid: true,
            synth_b_knobs: true,
            synth_b_grid: true,
            drum_grid: true,
            drum_knobs: true,
            waveform: true,
        };
        let vis_collapsed = PanelVisibility {
            synth_a_knobs: true,
            synth_a_grid: true,
            synth_b_knobs: false,  // collapsed
            synth_b_grid: false,   // collapsed
            drum_grid: true,
            drum_knobs: true,
            waveform: true,
        };
        let h = 120;
        let ly_exp = compute_dual_layout(term(h), &vis_expanded);
        let ly_col = compute_dual_layout(term(h), &vis_collapsed);

        // Drum grid should be bigger when synth B is collapsed
        assert!(ly_col.drum_grid.height > ly_exp.drum_grid.height);
    }

    #[test]
    fn collapsed_panel_has_collapsed_rect() {
        let vis = PanelVisibility {
            synth_a_knobs: true,
            synth_a_grid: true,
            synth_b_knobs: false,
            synth_b_grid: false,
            drum_grid: true,
            drum_knobs: false,
            waveform: false,
        };
        let ly = compute_dual_layout(term(80), &vis);

        // Synth B knobs: expanded empty, collapsed non-empty
        assert_eq!(ly.synth_b_knobs, Rect::default());
        assert_eq!(ly.synth_b_knobs_collapsed.height, COLLAPSED_PANEL_HEIGHT);

        // Drum knobs: collapsed
        assert_eq!(ly.drum_knobs, Rect::default());
        assert_eq!(ly.drum_knobs_collapsed.height, COLLAPSED_PANEL_HEIGHT);

        // Waveform: collapsed
        assert_eq!(ly.waveform, Rect::default());
        assert_eq!(ly.waveform_collapsed.height, COLLAPSED_PANEL_HEIGHT);
    }

    #[test]
    fn panels_are_contiguous_vertically() {
        let vis = PanelVisibility::default();
        let ly = compute_dual_layout(term(100), &vis);

        // Transport starts at 0
        assert_eq!(ly.transport.y, 0);

        // Each panel starts where the previous one ends
        // synth_a_knobs follows transport
        let after_transport = ly.transport.y + ly.transport.height;
        let sa_knobs_y = if ly.synth_a_knobs.height > 0 {
            ly.synth_a_knobs.y
        } else {
            ly.synth_a_knobs_collapsed.y
        };
        assert_eq!(sa_knobs_y, after_transport);
    }

    #[test]
    fn small_terminal_no_panic() {
        // Terminal too small to fit all panels — must not overflow buffer bounds
        let vis = PanelVisibility::default();
        let ly = compute_dual_layout(term(22), &vis);

        // Every rect must stay within [0, 22)
        let all_rects = [
            ly.transport,
            ly.synth_a_knobs, ly.synth_a_knobs_collapsed,
            ly.synth_a_grid, ly.synth_a_grid_collapsed,
            ly.synth_b_knobs, ly.synth_b_knobs_collapsed,
            ly.synth_b_grid, ly.synth_b_grid_collapsed,
            ly.drum_grid, ly.drum_knobs, ly.drum_knobs_collapsed,
            ly.waveform, ly.waveform_collapsed,
            ly.activity_bar,
        ];
        for r in &all_rects {
            assert!(
                r.y + r.height <= 22,
                "rect {:?} extends past terminal height 22",
                r,
            );
        }
    }

    #[test]
    fn default_visibility_layout() {
        // Default: synth B collapsed, everything else expanded
        let vis = PanelVisibility::default();
        let ly = compute_dual_layout(term(100), &vis);

        assert!(ly.synth_a_knobs.height > 0);
        assert!(ly.synth_a_grid.height > 0);
        assert_eq!(ly.synth_b_knobs, Rect::default());
        assert_eq!(ly.synth_b_grid, Rect::default());
        assert!(ly.synth_b_knobs_collapsed.height > 0);
        assert!(ly.synth_b_grid_collapsed.height > 0);
        assert!(ly.drum_grid.height > 0);
        assert!(ly.drum_knobs.height > 0);
        assert!(ly.waveform.height > 0);
    }
}
