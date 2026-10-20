// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Upload-ready platform export choices (roadmap V2): TikTok, Reels,
//! Shorts, YouTube. Specs only — this crate does not encode. Sequence
//! size is not rewritten here; the editor applies these numbers to the
//! export request.

/// One named destination the export sheet can offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformPreset {
    /// Keep the timeline's frame; still mix to the shared loudness target.
    YouTube,
    /// TikTok's short-form video destination.
    TikTok,
    /// Instagram Reels' short-form video destination.
    Reels,
    /// YouTube Shorts' short-form video destination.
    Shorts,
}

impl PlatformPreset {
    /// Sheet row order: Custom/YouTube first so an unchanged project
    /// still exports at timeline size.
    pub const ALL: [PlatformPreset; 4] = [
        PlatformPreset::YouTube,
        PlatformPreset::TikTok,
        PlatformPreset::Reels,
        PlatformPreset::Shorts,
    ];

    /// Returns the preset at the sheet's row index, clamped to the last row.
    pub fn from_index(index: usize) -> Self {
        Self::ALL[index.min(Self::ALL.len() - 1)]
    }

    /// Returns the stable lowercase identifier used in settings and exports.
    pub fn id(self) -> &'static str {
        match self {
            Self::YouTube => "youtube",
            Self::TikTok => "tiktok",
            Self::Reels => "reels",
            Self::Shorts => "shorts",
        }
    }

    /// Vertical 9:16 frame when the destination is a short-form app.
    /// `None` means keep the timeline (YouTube).
    pub fn frame(self) -> Option<(u32, u32)> {
        match self {
            Self::YouTube => None,
            Self::TikTok | Self::Reels | Self::Shorts => Some((1080, 1920)),
        }
    }

    /// Returns the destination's suggested frame rate, if it has one.
    /// `None` means to keep the timeline's frame rate.
    pub fn frame_rate_hint(self) -> Option<(u32, u32)> {
        match self {
            Self::YouTube => None,
            Self::TikTok | Self::Reels | Self::Shorts => Some((30, 1)),
        }
    }

    /// Integrated loudness target, LUFS. Same number for every platform
    /// here; true-peak ceiling is [`TRUE_PEAK_DBTP`].
    pub fn loudness_lufs(self) -> f64 {
        -14.0
    }

    /// Whether a finished mix should be normalised. Always on for these
    /// four destinations; a later "Custom" row can turn it off.
    pub fn normalize_mix(self) -> bool {
        true
    }

    /// CBR target in kbps when the sheet has no user bitrate yet.
    pub fn bitrate_kbps(self) -> u32 {
        match self {
            Self::YouTube => 12_000,
            Self::TikTok | Self::Reels | Self::Shorts => 8_000,
        }
    }
}

/// True-peak ceiling matching the roadmap (±1 dBTP).
pub const TRUE_PEAK_DBTP: f64 = -1.0;

/// Karaoke default after a caption *look* was applied: the preset already
/// chose ON (non-empty highlight) or OFF (empty). Do not force the
/// generated-caption yellow. When no look was applied, keep the old
/// "words and empty highlight → default karaoke" fill.
pub fn apply_generated_karaoke(
    highlight: &mut String,
    has_words: bool,
    look_preset_decided: bool,
    default_highlight: &str,
) {
    if look_preset_decided {
        return;
    }
    if has_words && highlight.is_empty() {
        *highlight = default_highlight.to_owned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_form_platforms_share_a_vertical_frame() {
        for p in [
            PlatformPreset::TikTok,
            PlatformPreset::Reels,
            PlatformPreset::Shorts,
        ] {
            assert_eq!(p.frame(), Some((1080, 1920)));
            assert_eq!(p.frame_rate_hint(), Some((30, 1)));
            assert_eq!(p.loudness_lufs(), -14.0);
            assert!(p.normalize_mix());
        }
        assert_eq!(PlatformPreset::YouTube.frame(), None);
        assert_eq!(TRUE_PEAK_DBTP, -1.0);
    }

    #[test]
    fn from_index_clamps() {
        assert_eq!(PlatformPreset::from_index(0), PlatformPreset::YouTube);
        assert_eq!(PlatformPreset::from_index(3), PlatformPreset::Shorts);
        assert_eq!(PlatformPreset::from_index(99), PlatformPreset::Shorts);
    }

    #[test]
    fn karaoke_fill_only_when_no_look_decided() {
        let def = "#ffd21f";
        let mut empty = String::new();
        apply_generated_karaoke(&mut empty, true, false, def);
        assert_eq!(empty, def);

        let mut off = String::new();
        apply_generated_karaoke(&mut off, true, true, def);
        assert!(off.is_empty(), "boxed-style OFF must stay off");

        let mut on = "#39ff88".to_owned();
        apply_generated_karaoke(&mut on, true, true, def);
        assert_eq!(on, "#39ff88");

        let mut title = String::new();
        apply_generated_karaoke(&mut title, false, false, def);
        assert!(title.is_empty());
    }
}
