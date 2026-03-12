// Summing mixer: mute/solo logic, soft clip

/// Returns `true` if the given track should be effectively muted.
///
/// Rules:
/// - If **any** track is soloed, only soloed tracks are audible — all others are muted.
/// - Otherwise, the per-track mute flag is used directly.
pub fn effective_mute(track: usize, muted: &[bool; 8], soloed: &[bool; 8]) -> bool {
    let any_solo = soloed.iter().any(|&s| s);
    if any_solo {
        // Track is muted unless it is soloed
        !soloed[track]
    } else {
        muted[track]
    }
}

/// Soft-clipping via `tanh`. Keeps the signal in roughly (-1, 1).
pub fn soft_clip(x: f32) -> f32 {
    x.tanh()
}

/// Gentle per-track saturation using cubic soft-clip.
/// Adds subtle odd harmonics and tames peaks without killing transients.
/// More transparent than tanh — preserves the first ~0.8 of dynamic range linearly.
#[inline]
pub fn per_track_saturate(x: f32) -> f32 {
    if x > 1.0 {
        2.0 / 3.0 + (x - 1.0) / (1.0 + (x - 1.0) * (x - 1.0))
    } else if x < -1.0 {
        -2.0 / 3.0 + (x + 1.0) / (1.0 + (x + 1.0) * (x + 1.0))
    } else {
        x - (x * x * x) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_solo_no_mute() {
        let muted = [false; 8];
        let soloed = [false; 8];
        for i in 0..8 {
            assert!(!effective_mute(i, &muted, &soloed));
        }
    }

    #[test]
    fn test_mute_only() {
        let mut muted = [false; 8];
        muted[2] = true;
        muted[5] = true;
        let soloed = [false; 8];
        assert!(!effective_mute(0, &muted, &soloed));
        assert!(effective_mute(2, &muted, &soloed));
        assert!(effective_mute(5, &muted, &soloed));
        assert!(!effective_mute(7, &muted, &soloed));
    }

    #[test]
    fn test_solo_overrides_mute() {
        let mut muted = [false; 8];
        muted[0] = true; // muted, but also soloed → should play
        let mut soloed = [false; 8];
        soloed[0] = true;
        soloed[3] = true;

        // Soloed tracks are audible
        assert!(!effective_mute(0, &muted, &soloed));
        assert!(!effective_mute(3, &muted, &soloed));
        // Non-soloed tracks are muted (regardless of their mute flag)
        assert!(effective_mute(1, &muted, &soloed));
        assert!(effective_mute(7, &muted, &soloed));
    }

    #[test]
    fn test_soft_clip_passthrough() {
        // Small values pass through almost unchanged
        let x = 0.1_f32;
        let y = soft_clip(x);
        assert!((y - x).abs() < 0.01);
    }

    #[test]
    fn test_soft_clip_limits() {
        // For large inputs, tanh saturates very close to +/-1
        assert!(soft_clip(100.0) >= 0.999);
        assert!(soft_clip(100.0) <= 1.0);
        assert!(soft_clip(-100.0) <= -0.999);
        assert!(soft_clip(-100.0) >= -1.0);
        // For moderate inputs, still bounded
        assert!(soft_clip(3.0) > 0.99);
        assert!(soft_clip(3.0) < 1.0);
    }

    #[test]
    fn test_soft_clip_symmetry() {
        let v = 0.7;
        assert!((soft_clip(v) + soft_clip(-v)).abs() < 1e-6);
    }

    #[test]
    fn test_per_track_saturate_clean() {
        let x = 0.1_f32;
        let y = per_track_saturate(x);
        assert!((y - x).abs() < 0.02);
    }

    #[test]
    fn test_per_track_saturate_limits() {
        let y = per_track_saturate(2.0);
        assert!(y > 0.8);
        assert!(y < 2.0);
    }

    #[test]
    fn test_per_track_saturate_symmetry() {
        let v = 0.8;
        assert!((per_track_saturate(v) + per_track_saturate(-v)).abs() < 1e-6);
    }
}
