// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! V3 auto-ducking: the music bed drops under speech and returns after
//! (roadmap V3, owner A6, 16 Oct 2026).
//!
//! One keyframe writer - [`duck_keys`] - and two routes of speech into it,
//! both chosen by the owner:
//!
//! - **Route B, transcript:** the heard words of caption clips are the
//!   speech ([`regions_from_words`]). No extra analysis; the accuracy is
//!   the accuracy of the T2/T3 word timing.
//! - **Route A, audio analysis:** the caller decodes the speech clip's
//!   samples - decoding lives where FFmpeg lives, not in this crate - and
//!   hands them to [`regions_from_samples`], an energy gate with one
//!   sensitivity dial like Premiere's. The gate window is a starting
//!   guess flagged for tuning on the owner's real audio, the way T3's
//!   flux bands were.
//!
//! The output is plain volume keyframes on the music clip (owner A6:
//! "volume keyframes on the music clip, as in Premiere - this keeps the
//! result editable and reversible"). Editable: they are ordinary keys the
//! curve editor already rides, and the user can move any of them.
//! Reversible: the command is one undo, and [`crate::commands::Command::
//! ClearClipKeys`] erases the whole duck without the undo history.

use serde::{Deserialize, Serialize};

use crate::captionmap::Word;
use crate::model::{Clip, ClipKey, KeyEase, KeyProperty};

/// The owner's default duck depth, in dB (A6: kept, editable).
pub const DEFAULT_AMOUNT_DB: f64 = -12.0;
/// The owner's default ramp length on both ends of a speech region (A6).
pub const DEFAULT_FADE_MS: u32 = 500;
/// The deepest duck the writer accepts; past this the music is silence
/// and the dial is lying.
pub const MIN_AMOUNT_DB: f64 = -60.0;
/// The longest ramp the writer accepts, in ms.
pub const MAX_FADE_MS: u32 = 10_000;

/// Route A's frame length: 20 ms, the grid the timing crate already uses.
const FRAME_S: f64 = 0.02;
/// Route A's gate: a frame is speech when its RMS sits within
/// `GATE_WINDOW_DB * (1 - sensitivity)` of the loudest frame. A starting
/// guess, flagged for tuning on the owner's real audio (the T3 precedent);
/// the sensitivity dial itself is the user's.
const GATE_WINDOW_DB: f64 = 40.0;
/// The least the sensitivity dial can be; at zero the gate would sit at
/// the peak and only the single loudest frame would ever count.
const MIN_SENSITIVITY: f64 = 0.05;
/// Route A's absolute floor: a frame quieter than this is not speech
/// whatever the loudest frame says. A starting guess, flagged like the
/// gate window above.
const SPEECH_FLOOR_DB: f64 = -50.0;

/// A span of speech, in absolute timeline seconds - the one shape both
/// routes produce and the writer consumes.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuckRegion {
    /// When the speech starts, on the timeline clock.
    pub start: f64,
    /// When it ends; after `start`.
    pub end: f64,
}

/// A level in dB as the linear factor `Clip::volume` and its keys carry:
/// -12 dB is about a quarter of the amplitude.
pub fn db_to_linear(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}

/// Overlapping regions made one, and regions whose gap is no longer than
/// `max_gap` merged too: a pause shorter than the ramp cannot complete a
/// ramp, so the duck would flap for nothing - it stays down across the
/// pause instead. Unsorted input is sorted; non-finite and empty spans
/// are dropped.
pub fn merge_regions(regions: &[(f64, f64)], max_gap: f64) -> Vec<(f64, f64)> {
    let mut sorted: Vec<(f64, f64)> = regions
        .iter()
        .filter(|(start, end)| start.is_finite() && end.is_finite() && end > start)
        .copied()
        .collect();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<(f64, f64)> = Vec::with_capacity(sorted.len());
    for (start, end) in sorted {
        match out.last_mut() {
            Some(last) if start - last.1 <= max_gap => last.1 = last.1.max(end),
            _ => out.push((start, end)),
        }
    }
    out
}

/// Route B: the heard words are the speech. Every word is a span, merged
/// with the fade as the gap rule.
pub fn regions_from_words(words: &[Word], fade_s: f64) -> Vec<(f64, f64)> {
    let spans: Vec<(f64, f64)> = words.iter().map(|word| (word.start, word.end)).collect();
    merge_regions(&spans, fade_s)
}

/// Route A: the energy gate over decoded samples, as Premiere's
/// analysis with its sensitivity slider. `sensitivity` runs 0..=1 (the
/// dial; clamped at [`MIN_SENSITIVITY`] below): at 1 the gate sits at the
/// peak and only the loudest frame counts; lower values widen the gate
/// down from the peak. Speech frames are gathered into spans and merged
/// with the fade, exactly like Route B's words.
pub fn regions_from_samples(
    samples: &[f32],
    sample_rate: f32,
    sensitivity: f64,
    fade_s: f64,
) -> Vec<(f64, f64)> {
    if samples.is_empty() || !sample_rate.is_finite() || sample_rate <= 0.0 {
        return Vec::new();
    }
    let frame_len = (FRAME_S * f64::from(sample_rate)).round().max(1.0) as usize;
    let mut dbs: Vec<f64> = Vec::new();
    for chunk in samples.chunks(frame_len) {
        let sum: f64 = chunk
            .iter()
            .map(|sample| f64::from(*sample) * f64::from(*sample))
            .sum();
        let rms = (sum / chunk.len() as f64).sqrt();
        // The floor keeps a digital zero from becoming -infinity; nothing
        // audible lives that deep.
        dbs.push(20.0 * rms.max(1e-10).log10());
    }
    let peak = dbs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let sensitivity = sensitivity.clamp(MIN_SENSITIVITY, 1.0);
    // The gate is the louder of the relative one (the dial, down from the
    // peak) and an absolute floor: a clip that is silence all the way
    // through has no peak to hang a relative gate on, and ducking
    // "everything" there would be the exact flapping the merge rule
    // exists to prevent. -50 dBFS is a starting guess with the T3 flag on
    // it: tune on the owner's real audio, not here.
    let gate = (peak - GATE_WINDOW_DB * (1.0 - sensitivity)).max(SPEECH_FLOOR_DB);
    let mut spans: Vec<(f64, f64)> = Vec::new();
    let mut run: Option<(usize, usize)> = None;
    for (index, db) in dbs.iter().enumerate() {
        if *db >= gate {
            run.get_or_insert((index, index)).1 = index;
        } else if let Some((first, last)) = run.take() {
            spans.push(frame_span(
                first,
                last,
                frame_len,
                samples.len(),
                sample_rate,
            ));
        }
    }
    if let Some((first, last)) = run {
        spans.push(frame_span(
            first,
            last,
            frame_len,
            samples.len(),
            sample_rate,
        ));
    }
    merge_regions(&spans, fade_s)
}

/// The seconds a run of speech frames covers, the last frame cut to the
/// samples that actually exist.
fn frame_span(
    first: usize,
    last: usize,
    frame_len: usize,
    total: usize,
    sample_rate: f32,
) -> (f64, f64) {
    let rate = f64::from(sample_rate);
    let start = (first * frame_len) as f64 / rate;
    let end = ((last + 1) * frame_len).min(total) as f64 / rate;
    (start, end)
}

/// The one keyframe writer: volume keys on `music` that drop it under
/// every region and bring it back after. Each region contributes four
/// keys - base at `start - fade`, ducked at `start`, ducked at `end`,
/// base at `end + fade` - clamped into the clip's `0..=1`, so a region
/// hanging over the clip's edge ducks from the edge itself. A fade of
/// zero makes the four keys two instants: a step down and a step up,
/// because the ducked key replaces the base key at the same `at`. The
/// keys come back sorted by `at`, linear ease, ready for
/// [`Clip::set_key`], which replaces anything already within
/// [`crate::model::KEY_EPSILON`] of each `at`.
pub fn duck_keys(
    music: &Clip,
    regions: &[(f64, f64)],
    amount_db: f64,
    fade_s: f64,
) -> Vec<ClipKey> {
    let duration = music.duration;
    if !duration.is_finite() || duration <= 0.0 || !music.start.is_finite() {
        return Vec::new();
    }
    let amount_db = if amount_db.is_finite() {
        amount_db.clamp(MIN_AMOUNT_DB, 0.0)
    } else {
        DEFAULT_AMOUNT_DB
    };
    let fade_s = if fade_s.is_finite() {
        fade_s.max(0.0)
    } else {
        0.5
    };
    let base = music.volume;
    let ducked = base * db_to_linear(amount_db);
    let window_end = music.start + duration;
    let mut keys: Vec<ClipKey> = Vec::with_capacity(regions.len() * 4);
    for &(start, end) in regions {
        // Only the part inside the music clip's window can carry keys; a
        // region wholly outside writes nothing.
        if end.min(window_end) <= start.max(music.start) {
            continue;
        }
        // The clamps make a region hanging over the clip's edge duck from
        // the edge itself: the ramp key lands on 0 (or 1) and the base it
        // carries is the level the clip already had.
        for (time, value) in [
            (start - fade_s, base),
            (start, ducked),
            (end, ducked),
            (end + fade_s, base),
        ] {
            keys.push(ClipKey {
                property: KeyProperty::Volume,
                at: ((time - music.start) / duration).clamp(0.0, 1.0),
                value,
                ease: KeyEase::LINEAR,
            });
        }
    }
    // Sorted by `at`, stably: with a fade of zero the entry's base key
    // comes before its ducked key and the exit's ducked key before its
    // base key, which is what makes `set_key` (last at an `at` wins) land
    // a step down and a step up.
    keys.sort_by(|a, b| a.at.total_cmp(&b.at));
    keys
}
