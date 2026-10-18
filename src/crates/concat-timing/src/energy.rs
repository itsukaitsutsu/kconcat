// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The waveform as decibel frames, and the two helpers silence detection uses.
//!
//! [`rms`] and [`linear_from_db`] moved here from `concat-media/src/silence.rs`
//! (decision D4-B) so there is one copy: `concat-media` now depends on this
//! crate for them, and its own tests for them stay where they were, in
//! `concat-media`, because running them needs FFmpeg and this crate must not.
//!
//! [`energy_frames`] is step 1 of Level 1 (`03-manuals/A-word-timing-level1.md`
//! §2): one pass over the buffer, O(n), framing at a 5 ms hop and a 20 ms
//! window - the same window the VAD already uses - into decibels below full
//! scale. Everything above it, the snapper and the Tier 1 onset check alike,
//! reads frames from here and never touches samples again.

use crate::SAMPLE_RATE;

/// Samples in one hop: 5 ms at [`SAMPLE_RATE`].
///
/// A smaller hop resolves onsets finer but admits more noise; 5 ms is the
/// manual's starting value and the one the debounce in [`crate::snap`] counts
/// frames in (2 frames = 10 ms).
pub const HOP_SAMPLES: usize = (SAMPLE_RATE as f64 * 0.005) as usize;

/// Samples in one analysis window: 20 ms at [`SAMPLE_RATE`].
///
/// The VAD's own chunk is 320 samples, so a snapped word and a VAD decision
/// are measured on the same grain of audio.
pub const WIN_SAMPLES: usize = (SAMPLE_RATE as f64 * 0.020) as usize;

/// A decibel level below full scale, as a linear amplitude in 0..=1.
///
/// -6 dB is about 0.5, -45 dB about 0.0056. The inverse of what a meter
/// reads, and the comparison a silence threshold actually wants.
pub fn linear_from_db(db: f64) -> f64 {
    10.0f64.powf(db / 20.0)
}

/// The root-mean-square level of one window of samples, in 0..=1.
///
/// RMS rather than peak, because a single stray sample - a click, a popped
/// plosive - should not make a whole window read as sound. This is also the
/// measure FFmpeg's `silencedetect` uses, so a silence threshold and an onset
/// floor are the same kind of number.
pub fn rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|sample| sample * sample).sum();
    f64::from((sum / samples.len() as f32).sqrt())
}

/// The time, in seconds, at which frame `index` begins.
///
/// Frames are indexed by their hop, so frame 0 starts at 0 and frame `i` at
/// `i * 5 ms`. The snapper writes this time back onto a word; the Tier 1
/// onset check reads a word's time and looks here for a rise.
pub fn frame_time(index: usize) -> f64 {
    index as f64 * HOP_SAMPLES as f64 / f64::from(SAMPLE_RATE)
}

/// The frame index whose time is closest to `seconds`, clamped to `len`.
///
/// The inverse of [`frame_time`], for turning a word's start into the frame
/// window around it. Rounds rather than floors: a search window written as
/// "±0.30 s around the word" should centre on the word, not start at it.
pub fn frame_at(seconds: f64, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let raw = seconds * f64::from(SAMPLE_RATE) / HOP_SAMPLES as f64;
    raw.round().clamp(0.0, (len - 1) as f64) as usize
}

/// Step 1 of Level 1: the whole buffer as decibel frames, in one pass.
///
/// `e[i] = 10 * log10(rms²(window i) + eps)`, windows of [`WIN_SAMPLES`]
/// stepped by [`HOP_SAMPLES`]. The epsilon keeps silence at a finite number
/// instead of -infinity, far below any floor a percentile could pick.
///
/// A buffer shorter than one window yields no frames: there is nothing to
/// measure, and an empty frame list is honest where a padded one would invent
/// energy that is not in the audio.
pub fn energy_frames(samples: &[f32]) -> Vec<f64> {
    if samples.len() < WIN_SAMPLES {
        return Vec::new();
    }
    let count = (samples.len() - WIN_SAMPLES) / HOP_SAMPLES + 1;
    const EPS: f64 = 1e-10;
    (0..count)
        .map(|i| {
            let window = &samples[i * HOP_SAMPLES..i * HOP_SAMPLES + WIN_SAMPLES];
            let level = rms(window);
            10.0 * (level * level + EPS).log10()
        })
        .collect()
}
