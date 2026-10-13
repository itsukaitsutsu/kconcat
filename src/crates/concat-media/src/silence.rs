// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Silence detection: where a file is quiet enough to cut.
//!
//! The spans this produces are the ones [`concat_project::sections::merge`]
//! and `complement` turn into a trim, so the two halves of an autotrim meet
//! here: this crate decodes and measures, that one does the arithmetic. It
//! is the reason `sections::silence_spans` - which parses the lines an
//! FFmpeg `silencedetect` filter prints - has a sibling that never shells
//! out: Concat spawns nothing at run time, so the measurement is done by
//! decoding through [`AudioDecoder`] and folding energy in Rust, exactly as
//! [`crate::peaks`] folds extremes for the waveform.
//!
//! Two layers, split the way `sections.rs` splits its own work:
//!
//! 1. [`spans_from_windows`] is pure - a slice of per-window RMS levels in,
//!    spans out. Every rule about what counts as silence lives there, so it
//!    is testable with no file, no decoder and no FFmpeg.
//! 2. [`detect`] streams a file's audio into those windows and hands them
//!    over. Samples are folded as they arrive and never accumulate, so an
//!    hour-long recording costs the same memory as a jingle.
//!
//! The semantics are FFmpeg `silencedetect`'s, because that is what the
//! rest of the engine's trimming was written against: a window is silent
//! when its level is at or below the noise floor, and a run of silent
//! windows is reported only once it has lasted at least `min_duration`.
//! The span reported is the run itself, not padded - padding is
//! `sections::merge`'s job, and it already takes a `pad` argument.

use std::path::Path;

use crate::error::Result;
use crate::samples::{AudioDecoder, AudioOptions, SampleFormat};

/// The rate audio is resampled to before measuring, matching [`crate::peaks`]
/// so a file is decoded the same way whichever of the two asks.
const SILENCE_RATE: u32 = 48_000;

/// How long one measured window is, in seconds.
///
/// 10 ms is the granularity FFmpeg's filter effectively works at, and it is
/// fine enough that [`concat_project::sections::MIN_SECTION_MS`] - 120 ms -
/// is still several windows wide, so a span this returns is never below the
/// shortest section the list will show.
pub const WINDOW_SECONDS: f64 = 0.01;

/// Samples in one window at [`SILENCE_RATE`].
pub const WINDOW_SAMPLES: usize = (SILENCE_RATE as f64 * WINDOW_SECONDS) as usize;

/// The default noise floor, in decibels below full scale.
///
/// FFmpeg's own `silencedetect` default is -60 dB, which is quieter than a
/// real room: a lavalier in a treated room sits around -50, and a phone in
/// a kitchen higher still. -45 dB catches the pauses in ordinary speech
/// without treating a breath or a room tone as a cut point.
pub const DEFAULT_NOISE_DB: f64 = -45.0;

/// The default shortest silence worth reporting, in seconds.
///
/// FFmpeg's default is 2 s, which is for finding gaps between scenes. For
/// trimming the pauses inside a talking head, half a second is the useful
/// granularity: it catches the gap after a sentence and leaves the ones
/// inside it, which are where the meaning is.
pub const DEFAULT_MIN_DURATION: f64 = 0.5;

/// A decibel level below full scale, as a linear amplitude in 0..=1.
///
/// -6 dB is about 0.5, -45 dB about 0.0056. The inverse of what a meter
/// reads, and the comparison [`spans_from_windows`] actually wants.
pub fn linear_from_db(db: f64) -> f64 {
    10.0f64.powf(db / 20.0)
}

/// The root-mean-square level of one window of samples, in 0..=1.
///
/// RMS rather than peak, because a single stray sample - a click, a popped
/// plosive - should not make a whole window read as sound. This is also the
/// measure FFmpeg's filter uses.
pub fn rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|sample| sample * sample).sum();
    f64::from((sum / samples.len() as f32).sqrt())
}

/// The spans of silence in a run of per-window levels.
///
/// `levels` is one RMS per [`WINDOW_SECONDS`], window 0 starting at 0. A
/// window is silent when its level is at or below `threshold`, a linear
/// amplitude from [`linear_from_db`]. A run shorter than `min_duration` is
/// dropped, the same way FFmpeg's filter drops one, and the spans that do
/// come back are `[start, end)` in seconds against the file's own clock.
///
/// A run still open when the levels end is reported if it is long enough:
/// a file that fades out and stays out is silent to its end, and trimming
/// that tail is exactly what an autotrim wants.
///
/// Pure, and the whole of the rule - see the module docs for why.
pub fn spans_from_windows(levels: &[f64], threshold: f64, min_duration: f64) -> Vec<(f64, f64)> {
    let mut spans: Vec<(f64, f64)> = Vec::new();
    // The window a current run of silence began at, or None while sound.
    let mut run_start: Option<usize> = None;

    for (index, level) in levels.iter().enumerate() {
        if *level <= threshold {
            if run_start.is_none() {
                run_start = Some(index);
            }
        } else if let Some(start) = run_start.take() {
            push_if_long_enough(&mut spans, start, index, min_duration);
        }
    }
    // A run open at the end of the file still counts, up to the last window.
    if let Some(start) = run_start {
        push_if_long_enough(&mut spans, start, levels.len(), min_duration);
    }
    spans
}

/// Records the windows `start..end` as a span when they last long enough.
fn push_if_long_enough(spans: &mut Vec<(f64, f64)>, start: usize, end: usize, min_duration: f64) {
    let length = (end - start) as f64 * WINDOW_SECONDS;
    if length >= min_duration {
        spans.push((start as f64 * WINDOW_SECONDS, end as f64 * WINDOW_SECONDS));
    }
}

/// Detects the silences in one file's audio.
///
/// The decode is mono 32-bit float at [`SILENCE_RATE`], of the audio stream
/// `stream` names or the file's first - the same choice [`crate::peaks`]
/// makes, so the waveform drawn and the silences found describe one track.
/// A file with no audio stream is an error that says so.
///
/// `noise_db` and `min_duration` are the two knobs FFmpeg's filter has, and
/// default to [`DEFAULT_NOISE_DB`] and [`DEFAULT_MIN_DURATION`].
///
/// The spans come back sorted and non-overlapping, ready for
/// `sections::merge` - which is what adds the padding a cut wants, since
/// cutting exactly at the last syllable is how a sentence loses its end.
pub fn detect(
    path: &Path,
    noise_db: f64,
    min_duration: f64,
    stream: Option<usize>,
) -> Result<Vec<(f64, f64)>> {
    let mut decoder = AudioDecoder::open(
        path,
        &AudioOptions {
            rate: SILENCE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            stream,
            ..AudioOptions::default()
        },
    )?;

    let threshold = linear_from_db(noise_db);
    let mut levels: Vec<f64> = Vec::new();
    let mut window: Vec<f32> = Vec::with_capacity(WINDOW_SAMPLES);

    while let Some(chunk) = decoder.next_f32()? {
        for sample in chunk {
            window.push(sample);
            if window.len() == WINDOW_SAMPLES {
                levels.push(rms(&window));
                window.clear();
            }
        }
    }
    // The trailing partial window still counts: dropping it would shave up
    // to 10 ms off the end of every file, and a file that ends in silence
    // would look like one that does not.
    if !window.is_empty() {
        levels.push(rms(&window));
    }

    Ok(spans_from_windows(&levels, threshold, min_duration))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `windows` windows of silence, then `windows` of sound, repeated.
    /// The levels are 0.0 and 0.5, either side of any sane threshold.
    fn alternating(silent: usize, loud: usize, pairs: usize) -> Vec<f64> {
        let mut levels = Vec::new();
        for _ in 0..pairs {
            levels.extend(std::iter::repeat_n(0.0, silent));
            levels.extend(std::iter::repeat_n(0.5, loud));
        }
        levels
    }

    #[test]
    fn a_decibel_level_becomes_the_amplitude_to_compare_against() {
        assert!(
            (linear_from_db(0.0) - 1.0).abs() < 1e-9,
            "0 dB is full scale"
        );
        assert!((linear_from_db(-6.0) - 0.5012).abs() < 1e-3);
        assert!((linear_from_db(-45.0) - 0.005623).abs() < 1e-6);
        assert!(
            linear_from_db(-45.0) < linear_from_db(-6.0),
            "quieter is smaller"
        );
    }

    #[test]
    fn rms_is_zero_for_silence_and_unity_for_a_full_square() {
        assert_eq!(rms(&[]), 0.0);
        assert_eq!(rms(&[0.0; 480]), 0.0);
        assert!((rms(&[1.0; 480]) - 1.0).abs() < 1e-6);
        assert!(
            (rms(&[-1.0; 480]) - 1.0).abs() < 1e-6,
            "sign does not matter"
        );
    }

    #[test]
    fn rms_reads_a_click_far_quieter_than_a_peak_would() {
        // One full-scale sample in an otherwise silent 10 ms window. A peak
        // meter reads it as 0 dB and calls the window as loud as the file
        // ever gets; RMS reads it at about -26.8 dB, which is 26.8 dB of
        // forgiveness. It is still above the -45 dB floor, and that is
        // right - a full-scale click is a sound, not a pause.
        let mut window = vec![0.0; WINDOW_SAMPLES];
        window[7] = 1.0;
        let level = rms(&window);
        assert_eq!(
            window.iter().cloned().fold(0.0f32, f32::max),
            1.0,
            "peak is unity"
        );
        assert!((level - 0.045644).abs() < 1e-5, "RMS is sqrt(1/480)");
        assert!(
            level < linear_from_db(-26.0),
            "at least 26 dB below the peak"
        );
        assert!(
            level > linear_from_db(DEFAULT_NOISE_DB),
            "but still a sound"
        );
    }

    #[test]
    fn noise_under_the_floor_stays_silent_where_a_peak_meter_would_not() {
        // The case RMS is for. Real noise has a crest factor - the loudest
        // sample sits well above the average - so a peak meter and an RMS
        // meter disagree about the same window. Every 16th sample at -38 dB
        // and the rest at zero averages to about -50 dB: under a -45 dB
        // floor on RMS, over it on peak.
        let peak = linear_from_db(-38.0) as f32;
        let mut window = vec![0.0; WINDOW_SAMPLES];
        for sample in window.iter_mut().step_by(16) {
            *sample = peak;
        }
        assert!(
            rms(&window) < linear_from_db(DEFAULT_NOISE_DB),
            "RMS reads the window as silence"
        );
        assert!(
            window.iter().cloned().fold(0.0f32, f32::max) > linear_from_db(DEFAULT_NOISE_DB) as f32,
            "a peak meter would have called the same window sound"
        );
    }

    #[test]
    fn no_silence_means_no_spans() {
        assert!(spans_from_windows(&[0.5; 200], linear_from_db(-45.0), 0.5).is_empty());
        assert!(spans_from_windows(&[], linear_from_db(-45.0), 0.5).is_empty());
    }

    #[test]
    fn a_run_shorter_than_the_minimum_is_dropped() {
        // 40 windows is 0.4 s, under the 0.5 s minimum.
        let levels = alternating(40, 100, 1);
        assert!(spans_from_windows(&levels, linear_from_db(-45.0), 0.5).is_empty());
        // 50 windows is exactly 0.5 s, and the comparison is inclusive.
        let levels = alternating(50, 100, 1);
        assert_eq!(
            spans_from_windows(&levels, linear_from_db(-45.0), 0.5),
            vec![(0.0, 0.5)]
        );
    }

    #[test]
    fn the_spans_are_in_seconds_against_the_files_own_clock() {
        // One second of sound, two of silence, one of sound.
        let mut levels = vec![0.5; 100];
        levels.extend(std::iter::repeat_n(0.0, 200));
        levels.extend(std::iter::repeat_n(0.5, 100));
        assert_eq!(
            spans_from_windows(&levels, linear_from_db(-45.0), 0.5),
            vec![(1.0, 3.0)]
        );
    }

    #[test]
    fn several_pauses_come_back_sorted_and_apart() {
        let levels = alternating(100, 100, 3);
        let spans = spans_from_windows(&levels, linear_from_db(-45.0), 0.5);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0], (0.0, 1.0));
        assert_eq!(spans[1], (2.0, 3.0));
        assert_eq!(spans[2], (4.0, 5.0));
        for pair in spans.windows(2) {
            assert!(pair[0].1 <= pair[1].0, "sorted and not overlapping");
        }
    }

    #[test]
    fn silence_running_to_the_end_of_the_file_is_reported() {
        // A fade out that never comes back: an autotrim wants this tail.
        let mut levels = vec![0.5; 100];
        levels.extend(std::iter::repeat_n(0.0, 150));
        assert_eq!(
            spans_from_windows(&levels, linear_from_db(-45.0), 0.5),
            vec![(1.0, 2.5)]
        );
    }

    #[test]
    fn a_wholly_silent_file_is_one_span_over_all_of_it() {
        let spans = spans_from_windows(&[0.0; 300], linear_from_db(-45.0), 0.5);
        assert_eq!(spans, vec![(0.0, 3.0)]);
    }

    #[test]
    fn the_threshold_decides_what_counts_as_quiet() {
        // A room tone at -40 dB: silent against a -35 dB floor, sound
        // against a -45 dB one.
        let level = linear_from_db(-40.0);
        let levels = [level; 200];
        assert_eq!(
            spans_from_windows(&levels, linear_from_db(-35.0), 0.5).len(),
            1
        );
        assert!(spans_from_windows(&levels, linear_from_db(-45.0), 0.5).is_empty());
    }

    #[test]
    fn a_trailing_partial_window_is_not_dropped() {
        // 250 windows and a half: the half is still a window, so 2.5 s of
        // silence reads as 2.5 s and not 2.49.
        let mut levels = vec![0.5; 50];
        levels.extend(std::iter::repeat_n(0.0, 200));
        let spans = spans_from_windows(&levels, linear_from_db(-45.0), 0.5);
        assert_eq!(spans, vec![(0.5, 2.5)]);
    }

    #[test]
    fn the_spans_are_what_sections_merge_takes() {
        // The contract between the two crates, stated as a test: what comes
        // back is a Vec<(f64, f64)> of [start, end), which is the exact
        // signature of concat_project::sections::merge's first argument.
        let spans: Vec<(f64, f64)> =
            spans_from_windows(&alternating(100, 100, 2), linear_from_db(-45.0), 0.5);
        let padded: Vec<(f64, f64)> = spans
            .iter()
            .map(|&(start, end)| ((start + 0.1).min(end), (end - 0.1).max(start)))
            .collect();
        assert_eq!(padded, vec![(0.1, 0.9), (2.1, 2.9)]);
    }
}
