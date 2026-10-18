// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! A word, plainly: text, a span, and where the span came from.
//!
//! Plain on purpose (D4-B). The snapper takes `&[f32]` and `&mut [Word]` and
//! nothing else, so nothing about the project model, the transcript or the
//! engine leaks into the arithmetic. `concat-speech` maps its `HeardWord` to
//! this on the way in and back on the way out.
//!
//! The [`Source`] marker is step 5 of Level 1: without it, "is snapping
//! better?" is a feeling, and a bad snap is mysterious instead of diagnosable.

/// Where a word's span came from.
///
/// Recorded per word, never inferred: a word Whisper placed and a word the
/// snapper moved are different facts, and an A/B of the feature - or a debug
/// export of `word, whisper_time, snapped_time` - needs to tell them apart
/// after the fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The span is exactly what the transcriber reported.
    Whisper,
    /// The span was moved to an acoustic onset by [`crate::snap`].
    Snapped,
}

/// One heard word: its text and its span in seconds on the media clock.
///
/// The clock is the continuous 16 kHz buffer's, not the timeline's: snapping
/// runs before any speed division, on the true time axis of the audio
/// (`03-manuals/A-word-timing-level1.md` §2). Spans are `[start, end)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    /// The word as transcribed. The snapper never edits it.
    pub text: String,
    /// Start, seconds. The only field the snapper is allowed to move.
    pub start: f64,
    /// End, seconds. Moved only to keep air before the next word's onset.
    pub end: f64,
    /// Where the current span came from.
    pub source: Source,
}

impl Word {
    /// A word on the media clock, marked as the transcriber reported it.
    pub fn whisper(text: &str, start: f64, end: f64) -> Self {
        Self {
            text: text.to_string(),
            start,
            end,
            source: Source::Whisper,
        }
    }

    /// The word's duration in seconds; negative for a reversed span, which is
    /// exactly what the Tier 1 outlier count looks for.
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}
