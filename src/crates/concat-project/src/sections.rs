// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Spans of time, and what a source is made of.
//!
//! Trimming by content is arithmetic over spans before it is anything else:
//! the silences have to be merged, what is left has to be found, and the
//! whole source has to be tiled into pieces a person can read a label on.
//! That arithmetic is here, and it is pure - no files, no FFmpeg, no engine
//! type - so the same numbers the detector computed are the numbers the
//! document is edited by, and both are testable without either.
//!
//! Three pieces, in the order they are used:
//!
//! 1. [`silence_spans`] reads the lines a `silencedetect` filter prints and
//!    returns the spans. It is text in and spans out: who ran the filter,
//!    and on what, is somebody else's business, which is why this crate can
//!    hold it.
//! 2. [`merge`] and [`complement`] turn spans into the spans beside them -
//!    the kept ranges, which is the whole of a trim.
//! 3. [`build_sections`] tiles the source into [`Section`]s so that what is
//!    about to be deleted can be looked at first.
//!
//! Every time is a float second against the source's own clock, which is
//! the document's terms; rounding to a millisecond happens on the way out,
//! so the arithmetic is never rounding against itself.

use serde::{Deserialize, Serialize};

/// How close two times may be and still count as the same time.
const TOUCH: f64 = 1e-6;

/// The shortest piece of a source worth keeping, in seconds. Anything
/// shorter is a sliver: it is a frame or two of picture and a click of
/// sound, and a timeline made of them is unreadable.
pub const MIN_KEEP: f64 = 0.05;

/// The shortest section the list shows on its own, in milliseconds. A
/// section under it is folded into its neighbour, because a 40 ms row
/// between two rows of the same kind is noise where a cut is meant to be
/// chosen.
pub const MIN_SECTION_MS: u32 = 120;

/// What a piece of the source is.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Speech a cue covers: the words, and the picture they are heard over.
    Caption,
    /// Below the noise floor for long enough to have been detected.
    Silence,
    /// Audible and covered by no cue: music, laughter, room tone, a cough.
    Other,
}

/// One labelled piece of a source, as the list shows it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    /// What this piece is.
    pub kind: Kind,
    /// Seconds into the source.
    pub start: f64,
    /// Seconds into the source; after [`Section::start`].
    pub end: f64,
    /// The words, for a [`Kind::Caption`]; empty for the others.
    pub text: String,
}

impl Section {
    /// How long the piece runs, in seconds.
    pub fn duration(&self) -> f64 {
        round_millis(self.end - self.start)
    }
}

/// What a tiling came to, so a panel can say it in one line.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    /// Seconds across every section, which is the source's own length once
    /// the tiling covers it.
    pub total: f64,
    /// Seconds of [`Kind::Caption`].
    pub caption_seconds: f64,
    /// Seconds of [`Kind::Silence`]: what a "delete the silences" pass
    /// would take out.
    pub silence_seconds: f64,
    /// Seconds of [`Kind::Other`].
    pub other_seconds: f64,
    /// How many sections each kind came to.
    pub captions: usize,
    /// How many sections.
    pub silences: usize,
    /// How many sections.
    pub others: usize,
    /// How many sections in all.
    pub count: usize,
}

/// A cue, as far as a tiling needs to know it: when, and what.
///
/// Taken as a trait rather than as [`crate::subtitle::Cue`] so that a
/// transcription's own segments, a file's cues and a document's caption
/// clips can all be tiled by the same function.
pub trait TimedCue {
    /// Seconds into the source where the words begin.
    fn start(&self) -> f64;
    /// Seconds into the source where the words end.
    fn end(&self) -> f64;
    /// The words.
    fn text(&self) -> &str;
}

impl TimedCue for crate::subtitle::Cue {
    fn start(&self) -> f64 {
        self.start
    }
    fn end(&self) -> f64 {
        self.end
    }
    fn text(&self) -> &str {
        &self.text
    }
}

/// Seconds, rounded to what a subtitle file and a frame count can both
/// hold, so a span read back is the span that was written.
pub fn round_millis(seconds: f64) -> f64 {
    (seconds * 1000.0).round() / 1000.0
}

/// Reads the lines an FFmpeg `silencedetect` filter prints into the spans
/// they describe, as `(start, end)` seconds.
///
/// The filter writes `silence_start: 1.234` and, when the sound comes back,
/// `silence_end: 5.678 | silence_duration: 4.444`; the two arrive on
/// separate lines and are matched up here. A silence that runs to the end
/// of the file never prints its end, so `total` supplies it - without that
/// the last pause of every recording would be the one that goes undetected.
///
/// Anything on the line besides the two keys is ignored, which is what
/// makes this a function over text: the caller may pass the filter's whole
/// stderr, or the two lines that matter, or a log a user attached.
pub fn silence_spans(text: &str, total: f64) -> Vec<(f64, f64)> {
    let mut spans = Vec::new();
    let mut open: Option<f64> = None;

    for line in text.lines() {
        if let Some(value) = value_after(line, "silence_start:") {
            open = Some(value.max(0.0));
        } else if let Some(end) = value_after(line, "silence_end:") {
            // An end with no start behind it is a line out of a longer log;
            // taking it alone would invent a silence from zero.
            if let Some(start) = open.take() {
                spans.push((start, end));
            }
        }
    }
    if let Some(start) = open {
        spans.push((start, total.max(start)));
    }
    spans
}

/// The float after `key` on a line, or None when the line does not carry it.
fn value_after(line: &str, key: &str) -> Option<f64> {
    let at = line.find(key)?;
    let rest = line[at + key.len()..].trim_start();
    // Up to the first character that cannot be part of a number. The filter
    // shares its line - `silence_end: 5.678 | silence_duration: 4.444` - so
    // the bar ends it and the value reads as 5.678.
    //
    // Letters are allowed through on purpose. Rust parses "nan" and "inf" as
    // floats, exactly as Python's `float()` does, and a value that is not
    // finite is then refused below rather than becoming half a number: a
    // scanner that stopped at the first non-digit would read "inf" as "i",
    // fail, and quietly drop a silence the log really printed.
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '|' || c == '#')
        .unwrap_or(rest.len());
    let value = rest[..end].trim().parse::<f64>().ok()?;
    value.is_finite().then_some(value)
}

/// The same spans, sorted and with every overlap and every adjacency
/// folded away, optionally keeping `pad` seconds of sound at each end.
///
/// Padding is what stops a detected silence from having eaten the breath
/// before a word and the consonant after it: a pad of 0.1 keeps a tenth of
/// a second of air on both sides of every cut, which is the difference
/// between a tight read and a chopped one. The pad is applied before the
/// merge, so two silences a pad apart become one span rather than two with
/// a sliver between them.
pub fn merge(spans: &[(f64, f64)], pad: f64) -> Vec<(f64, f64)> {
    let mut sorted = spans.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));

    let mut out: Vec<(f64, f64)> = Vec::with_capacity(sorted.len());
    for (start, end) in sorted {
        let start = (start - pad).max(0.0);
        let end = end + pad;
        match out.last_mut() {
            // Touching counts as overlapping: two silences with a 0.4 ms
            // gap are one silence, and the gap is not worth a cut.
            Some(last) if start <= last.1 + TOUCH => last.1 = last.1.max(end),
            _ => out.push((start, end)),
        }
    }
    out
}

/// Everything in `[0, total]` that is inside no span: the kept ranges.
///
/// This is the whole of a trim expressed as arithmetic. Overlapping and
/// adjacent spans are folded first, so the ranges that come back are sorted,
/// disjoint, and each at least `min_length` long - a kept range too short to
/// hold is not a kept range, it is a sliver, and an export made of them is
/// a stutter.
pub fn complement(spans: &[(f64, f64)], total: f64, min_length: f64) -> Vec<(f64, f64)> {
    let mut kept = Vec::new();
    let mut position = 0.0;

    for (start, end) in merge(spans, 0.0) {
        let start = start.clamp(0.0, total);
        let end = end.clamp(0.0, total);
        if start - position >= min_length {
            kept.push((position, start));
        }
        position = position.max(end);
    }
    if total - position >= min_length {
        kept.push((position, total));
    }
    kept
}

/// Tiles `[0, duration]` into labelled sections, so what is about to be cut
/// can be read before anything is.
///
/// The boundaries come from both sets - every silence edge and every cue
/// edge - which is what makes a section homogeneous: no piece is half
/// silence and half speech, so ticking one is ticking exactly what it says.
/// Neighbours that agree on kind and words are then merged back, because a
/// cue crossed by a detected silence is one cue and reads as two rows
/// otherwise, and slivers under `min_section_ms` are folded into the section
/// before them.
///
/// The tiling covers the source exactly: the first section starts at zero,
/// the last ends at `duration`, and every end is the next start.
pub fn build_sections(
    duration: f64,
    silences: &[(f64, f64)],
    cues: &[impl TimedCue],
    min_section_ms: u32,
) -> Vec<Section> {
    let duration = duration.max(0.0);

    // Every place a section can begin or end, clamped into the source.
    let mut bounds = vec![0.0, duration];
    for (start, end) in silences {
        bounds.push(start.clamp(0.0, duration));
        bounds.push(end.clamp(0.0, duration));
    }
    for cue in cues {
        bounds.push(cue.start().clamp(0.0, duration));
        bounds.push(cue.end().clamp(0.0, duration));
    }
    bounds.sort_by(|a, b| a.total_cmp(b));
    bounds.dedup();

    // One raw piece per pair of edges, labelled by what sits at its middle.
    let mut raw: Vec<(Kind, f64, f64, String)> = Vec::new();
    for pair in bounds.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        if end - start < 1e-4 {
            continue;
        }
        let middle = (start + end) / 2.0;
        let (kind, text) = if contains(silences, middle) {
            (Kind::Silence, String::new())
        } else {
            match cues
                .iter()
                .find(|cue| cue.start() - TOUCH <= middle && middle <= cue.end() + TOUCH)
            {
                Some(cue) => (Kind::Caption, cue.text().to_owned()),
                None => (Kind::Other, String::new()),
            }
        };
        raw.push((kind, start, end, text));
    }

    // Adjacent pieces that say the same thing are one thing.
    let mut merged: Vec<(Kind, f64, f64, String)> = Vec::new();
    for (kind, start, end, text) in raw {
        match merged.last_mut() {
            Some(last) if last.0 == kind && last.3 == text => last.2 = end,
            _ => merged.push((kind, start, end, text)),
        }
    }

    // A sliver joins the section before it, or the one after when it is
    // first: a row too short to read is not a choice anybody can make.
    let min_length = f64::from(min_section_ms) / 1000.0;
    let mut folded: Vec<(Kind, f64, f64, String)> = Vec::new();
    for item in merged {
        match folded.last_mut() {
            Some(last) if item.2 - item.1 < min_length => last.2 = item.2,
            _ => folded.push(item),
        }
    }
    if folded.len() > 1 && folded[0].2 - folded[0].1 < min_length {
        let first = folded.remove(0);
        folded[0].1 = first.1;
    }

    folded
        .into_iter()
        .map(|(kind, start, end, text)| Section {
            kind,
            start: round_millis(start),
            end: round_millis(end),
            text,
        })
        .collect()
}

/// The caption layer alone, with no silence pass: the gaps between cues
/// become [`Kind::Other`], which is what a transcript-only view wants.
pub fn sections_from_cues(
    duration: f64,
    cues: &[impl TimedCue],
    min_section_ms: u32,
) -> Vec<Section> {
    build_sections(duration, &[], cues, min_section_ms)
}

/// What a tiling came to, for the line a panel says above its list.
pub fn summarize(sections: &[Section]) -> Summary {
    let mut summary = Summary::default();
    for section in sections {
        let seconds = section.end - section.start;
        match section.kind {
            Kind::Caption => {
                summary.caption_seconds += seconds;
                summary.captions += 1;
            }
            Kind::Silence => {
                summary.silence_seconds += seconds;
                summary.silences += 1;
            }
            Kind::Other => {
                summary.other_seconds += seconds;
                summary.others += 1;
            }
        }
    }
    summary.total =
        round_millis(summary.caption_seconds + summary.silence_seconds + summary.other_seconds);
    summary.caption_seconds = round_millis(summary.caption_seconds);
    summary.silence_seconds = round_millis(summary.silence_seconds);
    summary.other_seconds = round_millis(summary.other_seconds);
    summary.count = sections.len();
    summary
}

/// Whether `at` sits inside any of these spans, edges included.
fn contains(spans: &[(f64, f64)], at: f64) -> bool {
    spans
        .iter()
        .any(|(start, end)| start - TOUCH <= at && at <= end + TOUCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subtitle::Cue;

    fn cue(start: f64, end: f64, text: &str) -> Cue {
        Cue {
            start,
            end,
            text: text.to_owned(),
        }
    }

    /// No cues, typed: an empty slice cannot say which cue it is empty of.
    fn no_cues() -> Vec<Cue> {
        Vec::new()
    }

    /// The filter's own output, as it prints it: two keys on two lines, the
    /// end sharing its line with the duration.
    const DETECTED: &str = "\
[silencedetect @ 0x1] silence_start: 1.5
[silencedetect @ 0x1] silence_end: 3.25 | silence_duration: 1.75
[silencedetect @ 0x1] silence_start: 8
";

    #[test]
    fn the_lines_a_silence_filter_prints_read_as_the_spans_they_describe() {
        assert_eq!(
            silence_spans(DETECTED, 10.0),
            vec![(1.5, 3.25), (8.0, 10.0)]
        );
    }

    #[test]
    fn a_silence_running_to_the_end_of_the_file_ends_at_the_file() {
        // The filter never prints silence_end for the last pause of a
        // recording, which is the one an editor most wants gone.
        assert_eq!(
            silence_spans("[silencedetect] silence_start: 12.5", 20.0),
            vec![(12.5, 20.0)]
        );
    }

    #[test]
    fn an_end_with_no_start_behind_it_invents_no_silence() {
        assert_eq!(
            silence_spans("silence_end: 3.0 | silence_duration: 3", 10.0),
            vec![]
        );
    }

    #[test]
    fn a_negative_start_reads_as_zero() {
        // The filter prints -0.001 for a recording that opens on silence.
        assert_eq!(
            silence_spans("silence_start: -0.001\nsilence_end: 2", 10.0),
            vec![(0.0, 2.0)]
        );
    }

    #[test]
    fn a_log_with_no_silence_in_it_reads_as_no_spans() {
        assert_eq!(
            silence_spans("Input #0, mov,mp4\n  Duration: 00:00:10.00\n", 10.0),
            vec![]
        );
        assert_eq!(silence_spans("", 10.0), vec![]);
    }

    #[test]
    fn overlapping_spans_become_one() {
        assert_eq!(
            merge(&[(0.0, 5.0), (3.0, 8.0), (20.0, 21.0)], 0.0),
            vec![(0.0, 8.0), (20.0, 21.0)]
        );
    }

    #[test]
    fn spans_that_only_touch_become_one() {
        // Two detected pauses with a hair between them are one pause, and
        // the hair is not worth a cut in the picture.
        assert_eq!(merge(&[(0.0, 5.0), (5.0, 8.0)], 0.0), vec![(0.0, 8.0)]);
    }

    #[test]
    fn unsorted_spans_come_back_sorted() {
        assert_eq!(
            merge(&[(20.0, 21.0), (0.0, 5.0), (9.0, 10.0)], 0.0),
            vec![(0.0, 5.0), (9.0, 10.0), (20.0, 21.0)]
        );
    }

    #[test]
    fn a_pad_keeps_air_at_both_ends_of_every_cut_and_merges_what_it_joins() {
        // Two pauses a fifth of a second apart, padded by a tenth on each
        // side, meet: one span, not two with a sliver between.
        assert_eq!(merge(&[(1.0, 2.0), (2.2, 3.0)], 0.1), vec![(0.9, 3.1)]);
        assert_eq!(merge(&[(1.0, 2.0)], 0.1), vec![(0.9, 2.1)]);
    }

    #[test]
    fn a_pad_never_takes_a_span_before_the_start_of_the_source() {
        assert_eq!(merge(&[(0.05, 1.0)], 0.5), vec![(0.0, 1.5)]);
    }

    #[test]
    fn the_complement_of_the_silences_is_what_survives_the_trim() {
        let silences = vec![(1.0, 2.0), (5.0, 6.0)];
        assert_eq!(
            complement(&silences, 10.0, MIN_KEEP),
            vec![(0.0, 1.0), (2.0, 5.0), (6.0, 10.0)]
        );
    }

    #[test]
    fn a_source_that_is_all_silence_keeps_nothing() {
        assert_eq!(complement(&[(0.0, 10.0)], 10.0, MIN_KEEP), vec![]);
    }

    #[test]
    fn a_source_with_no_silence_keeps_all_of_itself() {
        assert_eq!(complement(&[], 10.0, MIN_KEEP), vec![(0.0, 10.0)]);
    }

    #[test]
    fn a_kept_range_too_short_to_hold_is_not_kept() {
        // Two silences 30 ms apart: the piece between them is shorter than
        // MIN_KEEP, so it goes with them rather than becoming a stutter.
        assert_eq!(
            complement(&[(1.0, 2.0), (2.03, 4.0)], 10.0, MIN_KEEP),
            vec![(0.0, 1.0), (4.0, 10.0)]
        );
    }

    #[test]
    fn a_silence_past_the_end_of_the_source_is_clipped_to_it() {
        assert_eq!(complement(&[(8.0, 99.0)], 10.0, MIN_KEEP), vec![(0.0, 8.0)]);
    }

    #[test]
    fn overlapping_silences_are_folded_before_the_complement_is_taken() {
        assert_eq!(
            complement(&[(1.0, 5.0), (3.0, 8.0)], 10.0, MIN_KEEP),
            vec![(0.0, 1.0), (8.0, 10.0)]
        );
    }

    #[test]
    fn the_kept_ranges_add_up_to_the_source_minus_the_silence() {
        let silences = silence_spans(DETECTED, 10.0);
        let kept = complement(&silences, 10.0, MIN_KEEP);
        let silence: f64 = merge(&silences, 0.0).iter().map(|(s, e)| e - s).sum();
        let kept_total: f64 = kept.iter().map(|(s, e)| e - s).sum();
        assert!(
            (kept_total + silence - 10.0).abs() < 1e-9,
            "{kept_total} + {silence}"
        );
    }

    #[test]
    fn a_tiling_covers_the_source_exactly() {
        let cues = vec![cue(0.5, 2.0, "hello"), cue(4.0, 6.0, "world")];
        let silences = vec![(2.5, 3.5)];
        let sections = build_sections(10.0, &silences, &cues, MIN_SECTION_MS);

        assert_eq!(sections.first().unwrap().start, 0.0, "starts at zero");
        assert_eq!(sections.last().unwrap().end, 10.0, "ends at the source");
        for pair in sections.windows(2) {
            assert_eq!(pair[0].end, pair[1].start, "no gap and no overlap");
        }
        let total: f64 = sections.iter().map(|s| s.end - s.start).sum();
        assert!((total - 10.0).abs() < 1e-9, "{total}");
    }

    #[test]
    fn every_piece_is_one_kind_because_the_edges_come_from_both_sets() {
        // A cue from 0.5 to 2.0 with a silence from 1.0 to 3.0 would be one
        // row of two meanings if the boundaries came from one set only.
        let sections = build_sections(4.0, &[(1.0, 3.0)], &[cue(0.5, 2.0, "hi")], MIN_SECTION_MS);
        let kinds: Vec<_> = sections.iter().map(|s| s.kind).collect();
        assert!(kinds.contains(&Kind::Caption), "{kinds:?}");
        assert!(kinds.contains(&Kind::Silence), "{kinds:?}");
        assert!(kinds.contains(&Kind::Other), "{kinds:?}");
        for section in &sections {
            let middle = (section.start + section.end) / 2.0;
            assert_eq!(
                section.kind,
                kind_at(middle, &[(1.0, 3.0)], &[cue(0.5, 2.0, "hi")]),
                "{section:?} is labelled by its own middle"
            );
        }
    }

    /// What a moment is, by the same rule the tiling uses.
    fn kind_at(at: f64, silences: &[(f64, f64)], cues: &[Cue]) -> Kind {
        if contains(silences, at) {
            Kind::Silence
        } else if cues
            .iter()
            .any(|c| c.start - TOUCH <= at && at <= c.end + TOUCH)
        {
            Kind::Caption
        } else {
            Kind::Other
        }
    }

    #[test]
    fn a_caption_section_carries_the_words_of_the_cue_it_covers() {
        let sections = build_sections(5.0, &[], &[cue(1.0, 3.0, "the words")], MIN_SECTION_MS);
        let caption = sections.iter().find(|s| s.kind == Kind::Caption).unwrap();
        assert_eq!(caption.text, "the words");
        assert_eq!((caption.start, caption.end), (1.0, 3.0));
    }

    #[test]
    fn a_silence_and_an_other_section_carry_no_words() {
        let sections = build_sections(5.0, &[(2.0, 3.0)], &no_cues(), MIN_SECTION_MS);
        assert!(sections.iter().all(|s| s.text.is_empty()), "{sections:?}");
    }

    #[test]
    fn a_cue_crossed_by_a_silence_reads_as_one_row_on_each_side_of_it() {
        // One cue, one silence inside it: three sections, and the two
        // caption ones are separate because a silence sits between them.
        let sections = build_sections(
            6.0,
            &[(2.0, 3.0)],
            &[cue(1.0, 4.0, "one two three")],
            MIN_SECTION_MS,
        );
        let captions: Vec<_> = sections
            .iter()
            .filter(|s| s.kind == Kind::Caption)
            .collect();
        assert_eq!(captions.len(), 2, "{sections:?}");
        assert!(captions.iter().all(|s| s.text == "one two three"));
    }

    #[test]
    fn two_cues_that_touch_are_not_merged_into_one_row() {
        // Merging is by kind *and* words: adjacent captions of different
        // sentences stay two rows, or the list cannot choose between them.
        let sections = build_sections(
            4.0,
            &[],
            &[cue(0.0, 1.0, "first"), cue(1.0, 2.0, "second")],
            MIN_SECTION_MS,
        );
        let captions: Vec<_> = sections
            .iter()
            .filter(|s| s.kind == Kind::Caption)
            .collect();
        assert_eq!(captions.len(), 2, "{sections:?}");
        assert_eq!(captions[0].text, "first");
        assert_eq!(captions[1].text, "second");
    }

    #[test]
    fn a_sliver_folds_into_the_section_before_it() {
        // 20 ms of other between two captions is under MIN_SECTION_MS, so
        // it joins the first caption rather than becoming an unreadable row.
        let cues = vec![cue(0.0, 1.0, "a"), cue(1.02, 2.0, "b")];
        let sections = build_sections(2.0, &[], &cues, MIN_SECTION_MS);
        assert!(
            sections
                .iter()
                .all(|s| s.end - s.start >= 0.12 || s.kind == Kind::Caption),
            "{sections:?}"
        );
        assert!(
            !sections
                .iter()
                .any(|s| s.kind == Kind::Other && s.end - s.start < 0.12),
            "no 20 ms row survives: {sections:?}"
        );
    }

    #[test]
    fn a_sliver_first_folds_forward_into_the_section_after_it() {
        // The first section has nothing before it, so it joins the next.
        let sections = build_sections(2.0, &[], &[cue(0.02, 2.0, "late start")], MIN_SECTION_MS);
        assert_eq!(sections.first().unwrap().start, 0.0);
        assert_eq!(sections.len(), 1, "{sections:?}");
        assert_eq!(sections[0].kind, Kind::Caption);
    }

    #[test]
    fn a_source_with_nothing_detected_is_one_section() {
        let sections = build_sections(10.0, &[], &no_cues(), MIN_SECTION_MS);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].kind, Kind::Other);
        assert_eq!((sections[0].start, sections[0].end), (0.0, 10.0));
    }

    #[test]
    fn a_zero_length_source_tiles_to_nothing() {
        assert_eq!(build_sections(0.0, &[], &no_cues(), MIN_SECTION_MS), vec![]);
    }

    #[test]
    fn cues_and_silences_outside_the_source_are_clipped_into_it() {
        let sections = build_sections(
            5.0,
            &[(-1.0, 1.0), (4.0, 99.0)],
            &[cue(-2.0, 0.5, "early"), cue(4.5, 20.0, "late")],
            MIN_SECTION_MS,
        );
        assert_eq!(sections.first().unwrap().start, 0.0);
        assert_eq!(sections.last().unwrap().end, 5.0);
        assert!(
            sections.iter().all(|s| s.start >= 0.0 && s.end <= 5.0),
            "{sections:?}"
        );
    }

    #[test]
    fn the_caption_layer_without_a_silence_pass_makes_the_gaps_other() {
        let sections = sections_from_cues(
            6.0,
            &[cue(1.0, 2.0, "hi"), cue(4.0, 5.0, "again")],
            MIN_SECTION_MS,
        );
        let kinds: Vec<_> = sections.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            vec![
                Kind::Other,
                Kind::Caption,
                Kind::Other,
                Kind::Caption,
                Kind::Other
            ]
        );
        assert!(
            !kinds.contains(&Kind::Silence),
            "nothing was detected as silence"
        );
    }

    #[test]
    fn a_summary_counts_the_seconds_and_the_rows_of_each_kind() {
        let sections = build_sections(
            10.0,
            &[(3.0, 5.0)],
            &[cue(0.0, 2.0, "a"), cue(6.0, 8.0, "b")],
            MIN_SECTION_MS,
        );
        let summary = summarize(&sections);

        assert_eq!(summary.count, sections.len());
        assert_eq!(
            summary.captions + summary.silences + summary.others,
            summary.count
        );
        assert_eq!(summary.silences, 1);
        assert_eq!(summary.silence_seconds, 2.0);
        assert_eq!(summary.captions, 2);
        assert_eq!(summary.caption_seconds, 4.0);
        assert_eq!(summary.total, 10.0);
        assert!(
            (summary.caption_seconds + summary.silence_seconds + summary.other_seconds
                - summary.total)
                .abs()
                < 1e-9,
            "{summary:?}"
        );
    }

    #[test]
    fn a_summary_of_nothing_is_zeroes() {
        assert_eq!(summarize(&[]), Summary::default());
    }

    #[test]
    fn a_sections_duration_is_what_the_row_shows() {
        let section = Section {
            kind: Kind::Caption,
            start: 1.0,
            end: 2.5,
            text: "x".to_owned(),
        };
        assert_eq!(section.duration(), 1.5);
    }

    #[test]
    fn deleting_every_silence_section_is_the_same_trim_as_the_complement() {
        // The two routes to a trim have to agree, or the list a user ticks
        // and the ranges an export keeps are different edits.
        let silences = silence_spans(DETECTED, 10.0);
        let sections = build_sections(10.0, &silences, &no_cues(), MIN_SECTION_MS);

        let mut ticked: Vec<(f64, f64)> = sections
            .iter()
            .filter(|s| s.kind != Kind::Silence)
            .map(|s| (s.start, s.end))
            .collect();
        ticked.sort_by(|a, b| a.0.total_cmp(&b.0));

        let kept = complement(&silences, 10.0, MIN_KEEP);
        let ticked_total: f64 = ticked.iter().map(|(s, e)| e - s).sum();
        let kept_total: f64 = kept.iter().map(|(s, e)| e - s).sum();
        assert!(
            (ticked_total - kept_total).abs() < 1e-9,
            "ticked {ticked:?} vs kept {kept:?}"
        );
    }
}
