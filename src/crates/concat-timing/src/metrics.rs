// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The measurements that decide whether a timing change helped.
//!
//! Two kinds, and the split is the point (roadmap T1, T8):
//!
//! - **Tier 1 needs no labels.** Onset plausibility, monotonicity, outliers
//!   and the delta against a baseline prove a timing is wrong without anyone
//!   knowing the right one. They run on every change, forever.
//! - **WER needs a corrected transcript.** Word-level, normalised, and only
//!   as good as the reference the owner typed (T8). It measures text, not
//!   timing, which is why it runs beside Tier 1 rather than inside it.
//!
//! The example binary `metric_runner` prints both from
//! `06-corpus/whisper-cache.json` and `audio-16k.f32` in seconds (D5-B).

use crate::energy::{frame_at, frame_time};
use crate::signals::{Signals, is_flux_rise, is_zcr_rise};
use crate::snap;
use crate::words::Word;

/// How far either side of a word's start an energy rise may sit and still
/// make the onset plausible: ±150 ms, the Tier 1 definition.
const ONSET_TOLERANCE: f64 = 0.15;

/// The window a local noise floor is measured in, either side of a word's
/// start: ±0.30 s, step 2 of Level 1.
const FLOOR_WINDOW: f64 = 0.30;

/// One word's share of a cue that is shorter than the word: impossible, so it
/// is an outlier whatever the numbers say.
const MAX_WORD_SECONDS: f64 = 2.0;

/// The Tier 1 table for one run: four numbers, no labels needed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tier1 {
    /// The share of words with an energy rise within ±150 ms of their start.
    /// No target until a baseline exists; then the baseline is the target.
    pub onset_rate: f64,
    /// Reversed or overlapping word pairs. Target: 0.
    pub monotonic_violations: usize,
    /// Words outside their cue, of negative duration, or longer than 2 s.
    /// Target: 0.
    pub outliers: usize,
    /// How many words were measured, so a rate can be read as a count.
    pub words: usize,
}

/// The movement of one run against another, word by word.
///
/// A uniform shift is an offset bug, not an improvement: `mean_shift` says so
/// loudly when every word moved the same way in the same direction, while
/// `moved` staying near zero after a snap means the snapper did nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Delta {
    /// Words whose start moved by more than a centisecond.
    pub moved: usize,
    /// Mean signed movement in seconds: positive means later.
    pub mean_shift: f64,
    /// The largest single movement in seconds, either direction.
    pub max_abs_shift: f64,
}

/// A word-level error rate against a corrected reference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wer {
    /// Right place, wrong word.
    pub substitutions: usize,
    /// In the reference, missing from the hypothesis.
    pub deletions: usize,
    /// In the hypothesis, not in the reference.
    pub insertions: usize,
    /// `substitutions + deletions + insertions`.
    pub errors: usize,
    /// Words in the reference: the denominator.
    pub reference_words: usize,
    /// `errors / reference_words`, or 0 when the reference is empty.
    pub rate: f64,
}

/// The Tier 1 table for `words` measured against the buffer's `signals`.
///
/// `segment` is the cue the words belong to: a word outside it is an outlier
/// however well-behaved its span is, because a caption that reads words from
/// its neighbour's cue is the bug the sync matrix exists to catch.
pub fn tier1(words: &[Word], signals: &Signals, segment: (f64, f64)) -> Tier1 {
    const SLACK: f64 = 0.001;
    let hits = words
        .iter()
        .filter(|w| onset_plausible(signals, w.start))
        .count();
    let monotonic_violations = words
        .windows(2)
        .filter(|pair| {
            // Reversed: the next word starts before this one did. Overlapping:
            // it starts inside this one's span. Either is one violation, and
            // a pair that is both still counts once - it is one broken seam.
            let (cur, next) = (&pair[0], &pair[1]);
            next.start < cur.start - SLACK || next.start < cur.end - SLACK
        })
        .count();
    let outliers = words
        .iter()
        .filter(|w| {
            w.start < segment.0 - SLACK
                || w.end > segment.1 + SLACK
                || w.duration() < 0.0
                || w.duration() > MAX_WORD_SECONDS
        })
        .count();
    Tier1 {
        onset_rate: if words.is_empty() {
            0.0
        } else {
            hits as f64 / words.len() as f64
        },
        monotonic_violations,
        outliers,
        words: words.len(),
    }
}

/// How much `after` moved against `before`, matched by position.
///
/// The two lists must be the same words in the same order - a run and its
/// baseline are the same cache with and without a change - so matching is by
/// index and a length mismatch is reported as every word of the longer list
/// having moved, rather than silently zipping the prefix.
pub fn delta_vs_baseline(before: &[Word], after: &[Word]) -> Delta {
    const CENTISECOND: f64 = 0.01;
    let pairs = before.iter().zip(after.iter());
    let shifts: Vec<f64> = pairs.map(|(b, a)| a.start - b.start).collect();
    let extra = before.len().abs_diff(after.len());
    let moved = shifts.iter().filter(|s| s.abs() > CENTISECOND).count() + extra;
    let sum: f64 = shifts.iter().sum();
    Delta {
        moved,
        mean_shift: if shifts.is_empty() {
            0.0
        } else {
            sum / shifts.len() as f64
        },
        max_abs_shift: shifts.iter().fold(0.0f64, |m, s| m.max(s.abs())),
    }
}

/// One caption line as comparable words: lowercase, punctuation gone.
///
/// The agreed normalisation (T8 step 1), recorded here so a WER number can
/// always say which rule produced it: case folded, everything that is not
/// `char::is_alphanumeric` dropped (the same predicate `captionmap` uses -
/// ASCII-only tests would collapse a Japanese line to nothing), tokens split
/// on whitespace. Digits survive as digits; fillers are **not** removed here,
/// because removing them is E2's job and a WER that hid its own fillers could
/// not measure a model.
pub fn normalize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|token| {
            token
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

/// Word-level WER of `hypothesis` against `reference`, both pre-normalised
/// with [`normalize`].
///
/// Edit distance with unit costs, backtraced into the three kinds so a bad
/// number can be read as "the model deletes endings" rather than as a score.
pub fn wer(reference: &[String], hypothesis: &[String]) -> Wer {
    let rows = reference.len();
    let cols = hypothesis.len();
    // dp[i][j]: the edit distance between the first i reference words and the
    // first j hypothesis words, unit costs, so the backtrace can name kinds.
    let mut dp = vec![vec![0usize; cols + 1]; rows + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=rows {
        for j in 1..=cols {
            let sub = dp[i - 1][j - 1] + usize::from(reference[i - 1] != hypothesis[j - 1]);
            dp[i][j] = sub.min(dp[i - 1][j] + 1).min(dp[i][j - 1] + 1);
        }
    }
    let (mut i, mut j) = (rows, cols);
    let (mut substitutions, mut deletions, mut insertions) = (0usize, 0usize, 0usize);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && reference[i - 1] == hypothesis[j - 1] {
            i -= 1;
            j -= 1;
        } else if i > 0 && j > 0 && dp[i][j] == dp[i - 1][j - 1] + 1 {
            substitutions += 1;
            i -= 1;
            j -= 1;
        } else if i > 0 && dp[i][j] == dp[i - 1][j] + 1 {
            deletions += 1;
            i -= 1;
        } else {
            insertions += 1;
            j -= 1;
        }
    }
    let errors = substitutions + deletions + insertions;
    Wer {
        substitutions,
        deletions,
        insertions,
        errors,
        reference_words: rows,
        rate: if rows == 0 {
            0.0
        } else {
            errors as f64 / rows as f64
        },
    }
}

/// Whether an energy rise begins at frame `index`: step 3's rule, factored
/// out because Tier 1 and the snapper must agree on what a rise is.
///
/// A rise is a frame at least `rise_db` above the local floor, preceded by a
/// frame below it (the rise starts here, not mid-slope) and followed by one
/// still most of the way up (the debounce: it survives 5 more ms).
pub fn is_rise(
    frames: &[f64],
    index: usize,
    floor: f64,
    rise_db: f64,
    debounce_frames: usize,
) -> bool {
    let debounce = rise_db - 2.0;
    let tail = debounce_frames.saturating_sub(1);
    index > 0
        && index + tail < frames.len()
        && frames[index] - floor >= rise_db
        && frames[index - 1] < floor + rise_db
        && (1..=tail).all(|k| frames[index + k] >= debounce + floor)
}

/// The local noise floor around `seconds`: the 20th percentile of the frames
/// in ±0.30 s, step 2 of Level 1.
///
/// A percentile, not a mean: in a window that is mostly speech the mean sits
/// above the floor and no rise would ever clear it. An empty window has no
/// floor to speak of, and reports the quietest frame there is - none.
pub fn local_floor(frames: &[f64], seconds: f64) -> Option<f64> {
    local_floor_pct(frames, seconds, 20)
}

/// The same window at a percentile the caller picks: [`snap::Params`] makes
/// the floor percentile tunable (10th in noisy rooms), and the snapper and
/// Tier 1 must still agree on what the floor is, so both come through here.
pub fn local_floor_pct(frames: &[f64], seconds: f64, percentile: usize) -> Option<f64> {
    let centre = frame_at(seconds, frames.len());
    let half = (FLOOR_WINDOW / (frame_time(1) - frame_time(0))).round() as usize;
    let from = centre.saturating_sub(half);
    let to = (centre + half + 1).min(frames.len());
    if from >= to {
        return None;
    }
    let mut window: Vec<f64> = frames[from..to].to_vec();
    window.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let at = window.len() * percentile.min(100) / 100;
    Some(window[at.min(window.len() - 1)])
}

/// Whether any rise sits within ±150 ms of `seconds`: the Tier 1 onset check.
pub fn onset_plausible(signals: &Signals, seconds: f64) -> bool {
    let frames = &signals.energy;
    let Some(floor) = local_floor(frames, seconds) else {
        return false;
    };
    let centre = frame_at(seconds, frames.len());
    let tol = (ONSET_TOLERANCE / (frame_time(1) - frame_time(0))).round() as usize;
    let from = centre.saturating_sub(tol);
    let to = (centre + tol + 1).min(frames.len());
    let params = snap::Params::default();
    // The same three signals the snapper searches, over the same window:
    // Tier 1 credits exactly the onsets Level 1/1.5 can snap to, so the
    // plausibility rate can never disagree with the snapper's own eyes.
    let zcr_floor = local_floor_pct(&signals.zcr, seconds, params.floor_percentile).unwrap_or(0.0);
    let flux_median = local_floor_pct(&signals.flux, seconds, 50).unwrap_or(0.0);
    (from..to).any(|i| {
        is_rise(frames, i, floor, params.rise_db, params.debounce_frames)
            || is_zcr_rise(&signals.zcr, i, zcr_floor, params.wideband.zcr_margin)
            || is_flux_rise(&signals.flux, frames, i, flux_median, &params.wideband)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SAMPLE_RATE;

    fn word(text: &str, start: f64, end: f64) -> Word {
        Word::whisper(text, start, end)
    }

    #[test]
    fn a_two_word_fixture_reports_zero_violations_and_plausible_onsets() {
        let audio = two_bursts();
        let signals = Signals::from_samples(&audio);
        let words = vec![word("satu", 0.515, 0.70), word("dua", 0.915, 1.10)];
        let t = tier1(&words, &signals, (0.0, 2.0));
        assert_eq!(t.monotonic_violations, 0);
        assert_eq!(t.outliers, 0);
        assert_eq!(t.words, 2);
        assert!(
            t.onset_rate > 0.99,
            "both onsets are in the audio: {}",
            t.onset_rate
        );
    }

    fn two_bursts() -> Vec<f32> {
        let mut audio = vec![0.001; SAMPLE_RATE as usize * 2];
        for (at, loud) in [(0.515, 0.1f32), (0.915, 0.1)] {
            let start = (at * f64::from(SAMPLE_RATE)) as usize;
            let len = (0.06 * f64::from(SAMPLE_RATE)) as usize;
            audio[start..start + len].fill(loud);
        }
        audio
    }

    #[test]
    fn a_deliberately_reversed_pair_is_counted_as_one_violation() {
        let signals = Signals::from_samples(&two_bursts());
        let words = vec![word("dua", 0.90, 1.10), word("satu", 0.50, 0.70)];
        let t = tier1(&words, &signals, (0.0, 2.0));
        assert_eq!(t.monotonic_violations, 1);
    }

    #[test]
    fn an_overlapping_pair_is_counted_as_one_violation() {
        let signals = Signals::from_samples(&two_bursts());
        let words = vec![word("satu", 0.50, 0.95), word("dua", 0.90, 1.10)];
        let t = tier1(&words, &signals, (0.0, 2.0));
        assert_eq!(t.monotonic_violations, 1);
    }

    #[test]
    fn outliers_count_a_negative_span_an_overlong_word_and_one_outside_the_cue() {
        let signals = Signals::from_samples(&two_bursts());
        let words = vec![
            word("reversed", 0.60, 0.50),  // negative duration
            word("greedy", 0.90, 3.50),    // 2.6 s long
            word("elsewhere", 5.00, 5.20), // outside the cue
        ];
        let t = tier1(&words, &signals, (0.0, 2.0));
        assert_eq!(t.outliers, 3);
    }

    #[test]
    fn a_word_left_in_silence_is_not_a_plausible_onset() {
        let signals = Signals::from_samples(&two_bursts());
        let words = vec![word("hantu", 1.50, 1.70)];
        let t = tier1(&words, &signals, (0.0, 2.0));
        assert_eq!(t.onset_rate, 0.0, "1.50 s is flat silence: provably wrong");
    }

    #[test]
    fn wer_counts_one_substitution_one_deletion_one_insertion() {
        let reference = normalize("kita potong kata ini");
        let hypothesis = normalize("kita potong suara ini tambah");
        let w = wer(&reference, &hypothesis);
        // "kata"→"suara" substitutes, "ini tambah" inserts one, and deleting
        // nothing leaves: reference 4 words, hypothesis 5.
        assert_eq!(w.substitutions, 1);
        assert_eq!(w.insertions, 1);
        assert_eq!(w.deletions, 0);
        assert_eq!(w.errors, 2);
        assert_eq!(w.reference_words, 4);
        assert!((w.rate - 0.5).abs() < 1e-9);
    }

    #[test]
    fn a_deleted_word_is_a_deletion() {
        let reference = normalize("satu dua tiga");
        let hypothesis = normalize("satu tiga");
        let w = wer(&reference, &hypothesis);
        assert_eq!((w.substitutions, w.deletions, w.insertions), (0, 1, 0));
        assert!((w.rate - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn normalisation_gives_the_same_result_for_ya_comma_and_ya() {
        assert_eq!(normalize("Ya,"), normalize("ya"));
        assert_eq!(normalize("  Ya , "), vec!["ya".to_string()]);
    }

    #[test]
    fn normalisation_keeps_digits_and_splits_without_ascii_bias() {
        assert_eq!(
            normalize("jam 12:30"),
            vec!["jam".to_string(), "1230".to_string()]
        );
        // No spaces between words: the tokens must not collapse into one.
        assert_eq!(normalize("わたしは"), vec!["わたしは".to_string()]);
    }

    #[test]
    fn a_uniform_shift_reads_as_an_offset_not_as_movement() {
        let before = vec![word("a", 1.00, 1.20), word("b", 1.30, 1.50)];
        let after = vec![word("a", 1.25, 1.45), word("b", 1.55, 1.75)];
        let d = delta_vs_baseline(&before, &after);
        assert_eq!(d.moved, 2);
        assert!((d.mean_shift - 0.25).abs() < 1e-9, "uniform: an offset bug");
        assert!((d.max_abs_shift - 0.25).abs() < 1e-9);
    }

    #[test]
    fn a_fricative_onset_counts_as_a_plausible_onset() {
        // Tier 1 and the snapper must agree on what an onset is (the same
        // reason `is_rise` is shared): a word starting on a fricative that
        // Level 1.5 snaps to is plausible, even though energy cannot see it.
        let n = SAMPLE_RATE as usize;
        let mut audio = vec![0.001f32; n];
        let from = n / 2;
        for (i, sample) in audio.iter_mut().enumerate().take(n * 8 / 10).skip(from) {
            *sample = 0.002
                * if (i.wrapping_mul(2_654_435_761) >> 16) & 1 == 0 {
                    1.0
                } else {
                    -1.0
                };
        }
        let signals = Signals::from_samples(&audio);
        assert!(
            onset_plausible(&signals, 0.52),
            "the fricative at 0.5 s is an onset Tier 1 must credit"
        );
        assert!(
            !onset_plausible(&signals, 0.10),
            "silence stays implausible"
        );
    }

    #[test]
    fn a_run_that_changed_nothing_moved_nothing() {
        let before = vec![word("a", 1.00, 1.20)];
        let after = vec![word("a", 1.00, 1.20)];
        let d = delta_vs_baseline(&before, &after);
        assert_eq!(d.moved, 0);
        assert_eq!(d.mean_shift, 0.0);
    }
}
