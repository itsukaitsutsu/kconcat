// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Level 1: snap a word's start to the energy rise nearest it, and Level 1.5:
//! the two cheaper-than-a-model signals that catch what energy alone misses.
//!
//! The algorithm is `03-manuals/A-word-timing-level1.md` §2, steps 1-5, and
//! every number in [`Params`] is that manual's "Starting parameters" table -
//! nothing invented here, and the only fields a later session may tune are
//! the ones the table marks "tune by".
//!
//! Two rules from the roadmap sit above the manual and override it where they
//! disagree (T2):
//!
//! - **Collision rule.** After snapping, if a word's start is earlier than the
//!   previous word's final start, *revert that word to Whisper's value*. Do
//!   not clamp it. Then check the monotonic invariant again.
//! - **Clock rule.** Snapping runs on the continuous buffer's clock, before
//!   any speed division, and never on a VAD-compacted buffer: a spliced buffer
//!   has no onsets where the cuts were.
//!
//! The snapper is pure: samples in, words mutated in place, no file, no
//! decoder, no engine type in any signature.

/// The snapper's tuning, one field per row of the manual's parameter table.
///
/// `Default` is the table's "Start" column exactly. A session tuning these
/// changes one field per run and records the run, per T8 step 3.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    /// Which percentile of the local window is the noise floor. Start: 20.
    /// Tune by: lower (10th) in noisy rooms.
    pub floor_percentile: usize,
    /// How far above the floor a frame must sit to be a rise, in dB.
    /// Start: 8. Tune by: raise to 10-12 if false onsets appear on breaths.
    pub rise_db: f64,
    /// How many frames a rise must survive to count: 2 frames = 10 ms.
    /// Tune by: raise on very noisy audio.
    pub debounce_frames: usize,
    /// How far before the word's reported start to search, seconds: 0.20.
    pub search_back: f64,
    /// How far after it: 0.12. Whisper lags, so forward is the short side.
    /// Tune by: widen if words are consistently early.
    pub search_forward: f64,
    /// The air left before a snapped word, taken from the previous word's
    /// end: 40 ms. Tune by: 30-60 ms is the perceptual sweet spot.
    pub air_gap: f64,
    /// The shortest a word may be after snapping: 20 ms, the existing clamp.
    pub min_duration: f64,
    /// Level 1.5's knobs (T3): ZCR margin, flux thresholds, band edges. Not
    /// rows of the manual's Level 1 table - starting guesses flagged for T8.
    pub wideband: Wideband,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            floor_percentile: 20,
            rise_db: 8.0,
            debounce_frames: 2,
            search_back: 0.20,
            search_forward: 0.12,
            air_gap: 0.04,
            min_duration: 0.02,
            wideband: Wideband::default(),
        }
    }
}

use crate::energy::{frame_at, frame_time};
use crate::metrics::{is_rise, local_floor_pct};
use crate::signals::{Signals, Wideband, is_flux_rise, is_zcr_rise};
use crate::words::{Source, Word};

/// What one snapping pass did: the words on their final timing, and the
/// three counts that say how much of the result is Whisper's and how much
/// is Level 1's.
#[derive(Clone, Debug)]
pub struct SnapReport {
    /// The words after snapping, in input order, each carrying its marker.
    pub words: Vec<Word>,
    /// Words whose start now comes from an energy rise.
    pub snapped: usize,
    /// Words a snap moved, then the collision rule gave back to Whisper.
    pub reverted: usize,
    /// Words Whisper itself listed out of order, tied to the previous word's
    /// final start so the monotonic invariant holds. The start is not
    /// Whisper's and not an onset's - it is the invariant's, and the marker
    /// says `snapped` because Level 1 moved it.
    pub tied: usize,
}

/// Snaps every word's start to the nearest energy rise, then repairs the
/// invariants (roadmap T2, manual §2 steps 1-5).
///
/// `signals` are [`Signals::from_samples`] of the **continuous** window
/// buffer, on the window's own clock: before any speed division, never a
/// VAD-compacted buffer (see the module docs - two of the tests below fail
/// loudly if either rule is broken at the call site). `segment` is the
/// word's cue on that same clock; no word ever leaves it.
///
/// The collision rule, exactly as the roadmap fixes it: a snapped start
/// earlier than the previous word's final start is *reverted to Whisper's
/// value*, never clamped - and if Whisper's value is itself out of order,
/// the monotonic invariant wins and the starts are tied (counted in
/// [`SnapReport::tied`], so the repair is never silent).
pub fn snap_words(
    words: &[Word],
    signals: &Signals,
    segment: (f64, f64),
    params: &Params,
) -> SnapReport {
    let mut out: Vec<Word> = Vec::with_capacity(words.len());
    let (mut snapped, mut reverted, mut tied) = (0usize, 0usize, 0usize);
    // The latest a start may sit and still leave room for the minimum
    // duration inside the segment; for a segment shorter than that minimum,
    // the segment's own start.
    let max_start = (segment.1 - params.min_duration).max(segment.0);
    for word in words {
        let mut current = in_segment(word, segment, max_start, params);
        // Steps 1-3: the rise, if there is one. Step 4's clamps apply to the
        // snapped value too - a rise outside the segment is not a licence to
        // leave it.
        let moved = match snap_start(word.start, signals, params) {
            Some(onset) => {
                let onset = onset.clamp(segment.0, max_start);
                if (onset - current.start).abs() > 1e-9 {
                    current.start = onset;
                    current.end = fit_end(current.end, onset, segment.1, params);
                    current.source = Source::Snapped;
                    true
                } else {
                    false
                }
            }
            None => false,
        };
        if moved {
            snapped += 1;
        }
        if let Some(prev) = out.last_mut() {
            if current.start < prev.start - 1e-9 {
                // Collision rule: give the word back to Whisper, do not clamp.
                if moved {
                    snapped -= 1;
                    reverted += 1;
                }
                current = in_segment(word, segment, max_start, params);
                if current.start < prev.start - 1e-9 {
                    // Whisper itself listed the words out of order. The
                    // monotonic invariant wins: tie the starts, and count the
                    // tie so the repair is never silent.
                    current.start = prev.start;
                    current.end = fit_end(current.end, prev.start, segment.1, params);
                    current.source = Source::Snapped;
                    tied += 1;
                }
            }
            // The air gap: the previous word's end yields to the new start,
            // but never below its own minimum duration - a word is not
            // crushed to make its neighbour fit.
            let limit = current.start - params.air_gap;
            if prev.end > limit {
                prev.end = limit.max(prev.start + params.min_duration);
            }
        }
        out.push(current);
    }
    SnapReport {
        words: out,
        snapped,
        reverted,
        tied,
    }
}

/// Whisper's value, clamped into the segment (step 5's clip, applied to the
/// input so a cache word outside its cue cannot stay outside it).
fn in_segment(word: &Word, segment: (f64, f64), max_start: f64, params: &Params) -> Word {
    let mut fixed = word.clone();
    fixed.start = word.start.clamp(segment.0, max_start);
    fixed.end = fit_end(word.end, fixed.start, segment.1, params);
    fixed
}

/// The end after a start moved: at least the minimum duration long, never
/// past the segment, never before the start.
fn fit_end(end: f64, start: f64, segment_end: f64, params: &Params) -> f64 {
    end.max(start + params.min_duration)
        .min(segment_end)
        .max(start)
}

/// Step 1-3 for one word: the local floor around its reported start, then
/// the first energy rise in the search window `[start - back, start + fwd]`.
/// `None` is "no rise here" - the word keeps Whisper's value.
fn snap_start(reported: f64, signals: &Signals, params: &Params) -> Option<f64> {
    // A rise needs a predecessor and its debounce tail: three frames short,
    // there is nothing to detect.
    if signals.len() < 3 {
        return None;
    }
    let frames = &signals.energy;
    let floor = local_floor_pct(frames, reported, params.floor_percentile)?;
    // One local floor per signal, all on the manual's percentile idea: the
    // quiet majority of the window around the word defines "no onset here".
    // For flux the percentile is the 50th - flux is already a change, and
    // its steady state is near zero, so the median is the honest floor.
    let zcr_floor = local_floor_pct(&signals.zcr, reported, params.floor_percentile).unwrap_or(0.0);
    let flux_median = local_floor_pct(&signals.flux, reported, 50).unwrap_or(0.0);
    let from = frame_at(reported - params.search_back, frames.len());
    let to = frame_at(reported + params.search_forward, frames.len());
    // One scan, three signals: the first frame any of them calls a rise is
    // the onset, so a fricative before a vowel wins the vowel's own rise.
    (from..=to)
        .find(|&index| {
            is_rise(frames, index, floor, params.rise_db, params.debounce_frames)
                || is_zcr_rise(&signals.zcr, index, zcr_floor, params.wideband.zcr_margin)
                || is_flux_rise(&signals.flux, frames, index, flux_median, &params.wideband)
        })
        .map(frame_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::tier1;
    use crate::signals::Signals;

    /// A burst start aligned the way the T1 fixtures are: sample ≡ 240
    /// (mod 80), so one 20 ms frame sees the onset from 25 % loud and its
    /// predecessor is fully quiet - the rise rule fires on a known frame.
    fn aligned(t: f64) -> usize {
        let sample = (t * 16_000.0) as usize;
        (sample / 80) * 80 + 240
    }

    /// Quiet (-60 dB) audio with 0.1-amplitude bursts: `(start, duration)`.
    fn audio_with(bursts: &[(f64, f64)], seconds: f64) -> Vec<f32> {
        let n = (seconds * 16_000.0) as usize;
        let mut audio = vec![0.001f32; n];
        for &(start, duration) in bursts {
            let from = aligned(start);
            let to = ((start + duration) * 16_000.0) as usize;
            for sample in audio.iter_mut().take(to.min(n)).skip(from) {
                *sample = 0.1;
            }
        }
        audio
    }

    fn signals_of(bursts: &[(f64, f64)], seconds: f64) -> Signals {
        Signals::from_samples(&audio_with(bursts, seconds))
    }

    /// The time the snapper must land on for a burst aligned at `t`: the
    /// frame time of the frame where the rise rule fires. That frame's
    /// window `[k*hop, k*hop+win)` is the first to hold the burst's leading
    /// 80 samples - a quarter of it loud, which clears the floor by more
    /// than the 8 dB threshold - so it starts `win - hop - grid` (up to
    /// 15 ms + one hop) *before* the physical onset. The lookahead is the
    /// framing's, not a bug: `t(i)` is the window start the manual's `e[i]`
    /// definition anchors to, and the acceptance bands (median < 60 ms)
    /// budget for it. If T8's real-corpus runs show the early bias hurts
    /// the ±15 ms signed-error band, the fix is a tuning-session decision
    /// (recorded per T8 step 3), not a silent constant here.
    fn rise_frame_time(t: f64) -> f64 {
        frame_time((aligned(t) - 240) / 80)
    }

    const HOP: f64 = 0.005;

    // 1. A word after a clear energy rise moves to the rise (within one hop).
    #[test]
    fn a_word_after_a_clear_rise_moves_to_it() {
        let signals = signals_of(&[(1.015, 0.20)], 3.0);
        let words = [Word::whisper("kata", 1.10, 1.40)];
        let report = snap_words(&words, &signals, (0.5, 2.0), &Params::default());
        assert_eq!(report.snapped, 1);
        assert_eq!(report.reverted, 0);
        let rise = rise_frame_time(1.015);
        assert!(
            (report.words[0].start - rise).abs() <= HOP,
            "start {} did not land on the rise frame at {}",
            report.words[0].start,
            rise
        );
        assert_eq!(report.words[0].source, Source::Snapped);
    }

    // 2. A word with no rise in the search window keeps Whisper's value.
    #[test]
    fn a_word_with_no_rise_keeps_whispers_value() {
        let signals = signals_of(&[(1.015, 0.20)], 3.0);
        let words = [Word::whisper("sunyi", 2.50, 2.80)];
        let report = snap_words(&words, &signals, (2.0, 3.0), &Params::default());
        assert_eq!(report.snapped, 0);
        assert_eq!(report.words[0].start, 2.50);
        assert_eq!(report.words[0].source, Source::Whisper);
    }

    // 3. A snap that would reorder words is reverted, and the result is monotonic.
    //    Whisper listed these two out of order (real caches do); the second
    //    word's snap lands before the first word's final start, so the
    //    collision rule gives it back to Whisper - and since Whisper's own
    //    value is still out of order, the invariant ties the two starts.
    #[test]
    fn a_snap_that_would_reorder_words_is_reverted() {
        let signals = signals_of(&[(0.815, 0.20)], 3.0);
        let words = [
            Word::whisper("satu", 1.05, 1.35),
            Word::whisper("dua", 1.00, 1.30),
        ];
        let report = snap_words(&words, &signals, (0.5, 2.0), &Params::default());
        assert_eq!(
            report.reverted, 1,
            "the reorder-inducing snap was given back"
        );
        assert_ne!(
            report.words[1].start,
            rise_frame_time(0.815),
            "the snap did not survive"
        );
        assert!(
            report.words[1].start >= report.words[0].start - 1e-9,
            "result is monotonic: {:?} then {:?}",
            report.words[0].start,
            report.words[1].start
        );
        assert_eq!(report.tied, 1, "the tie is counted, never silent");
    }

    // 4. A snap never moves a word outside its segment.
    #[test]
    fn a_snap_never_moves_a_word_outside_its_segment() {
        // The rise sits at 2.57 s, past the segment's end at 2.50 s: the
        // word may follow it only as far as the segment allows, and its end
        // must still leave room for the minimum duration.
        let signals = signals_of(&[(2.555, 0.20)], 3.0);
        let words = [Word::whisper("ujung", 2.45, 2.50)];
        let segment = (2.0, 2.5);
        let report = snap_words(&words, &signals, segment, &Params::default());
        let word = &report.words[0];
        assert!(
            word.start >= segment.0 && word.start <= segment.1,
            "start {} escaped",
            word.start
        );
        assert!(
            word.end >= segment.0 && word.end <= segment.1 + 1e-9,
            "end {} escaped",
            word.end
        );
        assert!(word.end >= word.start);
        let t = tier1(&report.words, &signals, segment);
        assert_eq!(t.outliers, 0);
        assert_eq!(t.monotonic_violations, 0);
    }

    // 5. Every word carries a marker: whisper or snapped.
    #[test]
    fn every_word_carries_its_marker() {
        let signals = signals_of(&[(1.015, 0.20)], 4.0);
        let words = [
            Word::whisper("kena", 1.10, 1.40),  // snaps
            Word::whisper("sunyi", 3.00, 3.30), // nothing to snap to
        ];
        let report = snap_words(&words, &signals, (0.5, 4.0), &Params::default());
        assert_eq!(report.words[0].source, Source::Snapped);
        assert_eq!(report.words[1].source, Source::Whisper);
        assert_eq!(
            report.snapped
                + report
                    .words
                    .iter()
                    .filter(|w| w.source == Source::Whisper)
                    .count(),
            2
        );
    }

    // 6. On a speed-changed clip, snapping runs on the window clock before
    //    the speed division: the same frames, words on the window clock,
    //    snap; the same words after a premature division by the speed, do
    //    not - their search windows no longer contain any onset.
    #[test]
    fn snapping_runs_on_the_window_clock_before_the_speed_division() {
        let signals = signals_of(&[(1.015, 0.20)], 4.0);
        let speed = 2.0;

        // Correct order: window clock in, snapping happens.
        let on_window_clock = [Word::whisper("kata", 1.10, 1.40)];
        let report = snap_words(&on_window_clock, &signals, (0.5, 2.0), &Params::default());
        assert_eq!(report.snapped, 1, "the window clock snaps");

        // Wrong order: words already divided by the speed, frames not.
        let divided: Vec<Word> = on_window_clock
            .iter()
            .map(|w| Word::whisper(&w.text, w.start / speed, w.end / speed))
            .collect();
        let report = snap_words(&divided, &signals, (0.25, 1.0), &Params::default());
        assert_eq!(
            report.snapped, 0,
            "the divided clock finds no onset where the words are"
        );
        assert_eq!(report.words[0].start, divided[0].start);
    }

    // 7. Snapping never runs on a VAD-compacted buffer: if it does, this
    //    test is the alarm - the spliced buffer's frames end long before the
    //    words' search windows, and nothing snaps at all.
    #[test]
    fn a_compacted_buffer_is_audibly_wrong_and_snaps_nothing() {
        let full = audio_with(&[(1.015, 0.20)], 3.0);
        // What a VAD-compacted buffer looks like: only the voiced span,
        // re-indexed from zero. The onset that was at 1.03 s now sits at
        // 0.13 s, but the words still carry the original clock.
        let from = (0.9 * 16_000.0) as usize;
        let to = (1.3 * 16_000.0) as usize;
        let compacted = Signals::from_samples(&full[from..to]);

        let words = [Word::whisper("kata", 1.10, 1.40)];
        let report = snap_words(&words, &compacted, (0.5, 2.0), &Params::default());
        assert_eq!(
            report.snapped, 0,
            "on a compacted buffer the snapper is blind - which is the point: \
             the call site must pass the continuous buffer"
        );
        assert_eq!(report.words[0].start, 1.10);

        // The same word on the continuous buffer does snap, proving the
        // blindness above is the buffer's fault, not the word's.
        let report = snap_words(
            &words,
            &Signals::from_samples(&full),
            (0.5, 2.0),
            &Params::default(),
        );
        assert_eq!(report.snapped, 1);
    }

    // T3, prescribed: on a synthetic fricative-onset fixture, the snap moves
    // to the fricative. Whisper reports the vowel (the loud part); the word
    // really starts on the /s/ before it, which energy cannot see and ZCR
    // can. The snapper must land on the fricative's first frames, not the
    // vowel's energy rise later.
    #[test]
    fn a_fricative_onset_snaps_to_the_fricative_not_the_vowel() {
        // Quiet, then a low-amplitude noise burst (the fricative: -54 dB,
        // below the energy rise threshold), then a loud vowel.
        let n = 3 * crate::SAMPLE_RATE as usize;
        let mut audio = vec![0.001f32; n];
        let fricative = aligned(1.015);
        let vowel = fricative + 1_600; // 100 ms of /s/, then the vowel
        for (i, sample) in audio.iter_mut().enumerate().take(vowel).skip(fricative) {
            *sample = 0.002
                * if (i.wrapping_mul(2_654_435_761) >> 16) & 1 == 0 {
                    1.0
                } else {
                    -1.0
                };
        }
        for sample in audio.iter_mut().take(vowel + 3_200).skip(vowel) {
            *sample = 0.1;
        }
        let signals = Signals::from_samples(&audio);

        // Whisper-style: the word starts at the vowel, well after the /s/.
        let words = [Word::whisper("satu", 1.20, 1.45)];
        let report = snap_words(&words, &signals, (0.5, 2.0), &Params::default());
        assert_eq!(report.snapped, 1);
        let fricative_frame = frame_time((fricative - 240) / 80);
        let vowel_frame = frame_time((vowel - 240) / 80); // the vowel's energy rise
        assert!(
            (report.words[0].start - fricative_frame).abs() <= HOP,
            "start {} must land on the fricative at {}, not the vowel at {}",
            report.words[0].start,
            fricative_frame,
            vowel_frame
        );
        assert!(
            report.words[0].start < vowel_frame - HOP,
            "landing on the vowel means ZCR was not consulted"
        );
    }
}
