// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Phase 4b-1: the data a Sections panel shows, with no panel yet.
//!
//! [`crate::sections`] tiles a *source* into caption / silence / other
//! pieces, and [`crate::sections::summarize`] says what the tiling came
//! to. Neither knows what a timeline is. A panel, though, starts from a
//! clip *on* a timeline: its words are the text clips Phase 1 linked to
//! it (`caption_of`), its silences come from `concat-media`'s detector
//! against the file's own clock, and its rows have to be shown at
//! timeline times, because that is where a click has to land.
//!
//! This module is that translation, and it is deliberately all of
//! Phase 4b that lives in this crate:
//!
//! * [`clip_sections`] - one clip + file-clock silences in, rows out.
//!   Each row carries its [`Section`] (source clock), the same span on
//!   the timeline, and whether it starts ticked. A row starts ticked
//!   unless it is silence: "delete the silences" is the default edit,
//!   and unticking a row is how a person says "keep that pause".
//! * [`kept_ranges`] - the ticked rows as timeline ranges, neighbours
//!   merged. This is the input an autotrim (Phase 5) will turn into
//!   edits, and the equality a test can hold it to: ticking every
//!   non-silence row must keep exactly the complement of the silences.
//!
//! Two clocks, kept apart on purpose. A clip covers source
//! `[source_start, source_start + duration * speed)`; a caption at
//! timeline time `t` sits at source `source_start + (t - start) * speed`.
//! Silence spans arrive on the file's clock, so they are shifted by
//! `-source_start` into the clip's window before tiling. Rows report
//! both clocks so the panel never has to do this arithmetic itself -
//! arithmetic done twice, in two crates, is arithmetic that disagrees.

use std::collections::HashMap;

use crate::model::Timeline;
use crate::sections::{self, Kind, Section, Summary, TimedCue};

/// One row of the Sections list: a section, where it is on the timeline,
/// and whether it starts ticked.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// The section, on the source's clock (seconds into the clip's
    /// source window, so the first row starts at 0).
    pub section: Section,
    /// Where the row begins on the timeline.
    pub timeline_start: f64,
    /// Where the row ends on the timeline.
    pub timeline_end: f64,
    /// Ticked to begin with: everything except silence.
    pub keep: bool,
    /// The timeline text clip that produced this caption row, when the row
    /// came from a Concat caption rather than an external subtitle file.
    ///
    /// A section row is not just a time range: two caption tracks can cover
    /// the same range in different languages. Keeping the source clip id
    /// lets the Sections UI select, edit and delete *that* caption without
    /// touching the translation or a hand-made title underneath it.
    pub caption_id: Option<String>,
}

/// Everything the panel needs for one clip: the rows, and the one-line
/// summary above them.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipSections {
    /// The clip these rows describe.
    pub clip_id: String,
    /// The rows, in timeline order, covering the clip exactly.
    pub rows: Vec<Row>,
    /// What the rows came to, from [`sections::summarize`].
    pub summary: Summary,
}

/// A linked caption as a cue on the source clock, for
/// [`sections::build_sections`].
struct Cue {
    start: f64,
    end: f64,
    text: String,
    /// `None` for a cue read from an external subtitle file. Timeline
    /// captions keep their identity all the way to [`Row::caption_id`].
    clip_id: Option<String>,
}

impl TimedCue for Cue {
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

/// The sections of one clip, as a panel lists them.
///
/// `silences` are `(start, end)` seconds on the *file's* clock, as
/// `concat_media::silence::detect` returns them; `pad` is seconds of air
/// kept at each end of a silence (the silence is shrunk by it, not
/// grown - cutting exactly at the last syllable is how a sentence loses
/// its end, and padding here is what a reader in Resolve would call
/// Pre Head / Post Tail kept). Unknown clip id returns None; a clip
/// with no linked captions and no silences is one `other` row, which
/// is the truth and reads fine.
pub fn clip_sections(
    timeline: &Timeline,
    clip_id: &str,
    silences: &[(f64, f64)],
    pad: f64,
) -> Option<ClipSections> {
    clip_sections_with_options(
        timeline,
        clip_id,
        silences,
        pad,
        true,
        sections::MIN_SECTION_MS,
    )
}

/// The configurable form used by the Sections UI.
///
/// `use_captions` controls whether linked caption boundaries classify rows;
/// `min_section_ms` folds smaller fragments into their neighbours. Keeping the
/// original [`clip_sections`] wrapper preserves the stable project API and its
/// VTS-compatible defaults.
pub fn clip_sections_with_options(
    timeline: &Timeline,
    clip_id: &str,
    silences: &[(f64, f64)],
    pad: f64,
    use_captions: bool,
    min_section_ms: u32,
) -> Option<ClipSections> {
    clip_sections_with_cues(
        timeline,
        clip_id,
        silences,
        pad,
        use_captions,
        min_section_ms,
        None,
    )
}

/// Builds clip sections using either external subtitle cues (on the source
/// file's clock, when `external_cues` is `Some` and non-empty) or the clip's
/// linked timeline captions.
pub fn clip_sections_with_cues(
    timeline: &Timeline,
    clip_id: &str,
    silences: &[(f64, f64)],
    pad: f64,
    use_captions: bool,
    min_section_ms: u32,
    external_cues: Option<&[crate::subtitle::Cue]>,
) -> Option<ClipSections> {
    let clip = timeline.clip(clip_id)?;
    let window = clip.duration * clip.speed;

    let mut cues: Vec<Cue> = if !use_captions {
        Vec::new()
    } else if let Some(ext) = external_cues.filter(|c| !c.is_empty()) {
        ext.iter()
            .map(|cue| Cue {
                start: cue.start - clip.source_start,
                end: cue.end - clip.source_start,
                text: cue.text.clone(),
                clip_id: None,
            })
            .filter(|cue| cue.end > 0.0 && cue.start < window)
            .collect()
    } else {
        let linked_ids: std::collections::HashSet<String> =
            timeline.captions_of(clip_id).into_iter().collect();
        timeline
            .clips
            .iter()
            .filter(|c| {
                c.kind == crate::model::ClipKind::Text
                    // A caption tied to this source is unambiguous. An
                    // imported/project-wide caption has no source link but
                    // does carry word timing. A hand-typed title has neither
                    // and must not turn into a removable "caption" section.
                    && (linked_ids.contains(&c.id)
                        || (c.caption_of.is_none() && c.words.is_some()))
            })
            .map(|caption| Cue {
                // Timeline -> the clip's source window: how far into the
                // clip the caption sits, at the clip's speed.
                start: (caption.start - clip.start) * clip.speed,
                end: (caption.start + caption.duration - clip.start) * clip.speed,
                text: caption
                    .text
                    .as_ref()
                    .map(|style| style.content.clone())
                    .unwrap_or_default(),
                clip_id: Some(caption.id.clone()),
            })
            .filter(|cue| cue.end > 0.0 && cue.start < window)
            .collect()
    };
    cues.sort_by(|a, b| a.start.total_cmp(&b.start));

    // The file's silences, into the window, shrunk by the pad, and
    // dropped when the pad swallows them whole.
    let silences: Vec<(f64, f64)> = silences
        .iter()
        .map(|(start, end)| {
            (
                start - clip.source_start + pad,
                end - clip.source_start - pad,
            )
        })
        .filter(|(start, end)| end > &0.0 && start < &window && end > start)
        .collect();

    let sections = sections::build_sections(window, &silences, &cues, min_section_ms);
    let summary = sections::summarize(&sections);
    let rows = sections
        .into_iter()
        .map(|section| {
            // `build_sections` labels a caption span with the first cue that
            // contains its midpoint. Use that same identity rule here so a
            // row targets one caption track even when translations overlap.
            let middle = (section.start + section.end) / 2.0;
            let caption_id = (section.kind == Kind::Caption)
                .then(|| {
                    cues.iter()
                        .find(|cue| {
                            cue.clip_id.is_some()
                                && cue.text == section.text
                                && cue.start - 1e-6 <= middle
                                && middle <= cue.end + 1e-6
                        })
                        .and_then(|cue| cue.clip_id.clone())
                })
                .flatten();
            Row {
                timeline_start: sections::round_millis(clip.start + section.start / clip.speed),
                timeline_end: sections::round_millis(clip.start + section.end / clip.speed),
                keep: section.kind != Kind::Silence,
                caption_id,
                section,
            }
        })
        .collect();
    Some(ClipSections {
        clip_id: clip_id.to_owned(),
        rows,
        summary,
    })
}

/// Carries the ticks a reader has already set over to rows rebuilt from
/// the same timeline.
///
/// The panel rebuilds its rows whenever the document changes, and every
/// fresh row starts ticked by its default. Without this, any edit made
/// anywhere - a caption's words, a volume knob - would put back the
/// silences the reader had chosen to keep. A new row takes the tick of
/// the old row with the same kind and the same span, to the millisecond;
/// a row that matches nothing keeps its default.
pub fn carry_ticks(previous: &[Row], rows: &mut [Row]) {
    if previous.is_empty() {
        return;
    }
    let ticks: HashMap<(Kind, i64, i64), bool> = previous
        .iter()
        .map(|row| {
            (
                span_key(row.section.kind, row.timeline_start, row.timeline_end),
                row.keep,
            )
        })
        .collect();
    for row in rows.iter_mut() {
        if let Some(&keep) = ticks.get(&span_key(
            row.section.kind,
            row.timeline_start,
            row.timeline_end,
        )) {
            row.keep = keep;
        }
    }
}

/// A row's identity for [`carry_ticks`]: its kind and its span, in whole
/// milliseconds, so that float noise cannot make two equal rows differ.
fn span_key(kind: Kind, start: f64, end: f64) -> (Kind, i64, i64) {
    (
        kind,
        (start * 1000.0).round() as i64,
        (end * 1000.0).round() as i64,
    )
}

/// The timeline ranges a set of ticks keeps, neighbours merged.
///
/// `ticks` pairs with `rows` by index; a slice of the wrong length is a
/// programming error in the panel, and the answer to it is the rows'
/// own defaults rather than a panic on a click. This is the function
/// the autotrim will consume, and the invariant a test holds: with the
/// default ticks, the kept ranges total the clip's length minus the
/// silence the summary reports.
pub fn kept_ranges(rows: &[Row], ticks: &[bool]) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let keep = ticks.get(index).copied().unwrap_or(row.keep);
        if !keep {
            continue;
        }
        match out.last_mut() {
            Some(last) if (row.timeline_start - last.1).abs() < 1e-6 => {
                last.1 = row.timeline_end;
            }
            _ => out.push((row.timeline_start, row.timeline_end)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(kind: Kind, start: f64, end: f64, keep: bool) -> Row {
        Row {
            section: Section {
                kind,
                start,
                end,
                text: String::new(),
            },
            timeline_start: start,
            timeline_end: end,
            keep,
            caption_id: None,
        }
    }

    #[test]
    fn a_rebuilt_row_takes_the_tick_its_twin_had() {
        // The reader chose to keep a pause (a silence starts not kept, so
        // flipping it on is a choice) and then edited something elsewhere,
        // which rebuilds every row with the silence back at its default.
        let before = vec![
            row(Kind::Caption, 0.0, 2.0, true),
            row(Kind::Silence, 2.0, 3.0, true),
        ];
        let mut after = vec![
            row(Kind::Caption, 0.0, 2.0, true),
            row(Kind::Silence, 2.0, 3.0, false),
        ];
        carry_ticks(&before, &mut after);
        assert!(after[1].keep, "the pause the reader kept stays kept");
    }

    #[test]
    fn an_unmatched_row_keeps_its_default() {
        let before = vec![row(Kind::Silence, 2.0, 3.0, true)];
        // The silence moved: a different span is a different row.
        let mut after = vec![row(Kind::Silence, 2.5, 3.5, false)];
        carry_ticks(&before, &mut after);
        assert!(!after[0].keep, "the default for a silence is ticked off");
    }

    #[test]
    fn kind_is_part_of_the_identity() {
        let before = vec![row(Kind::Other, 1.0, 2.0, false)];
        let mut after = vec![row(Kind::Caption, 1.0, 2.0, true)];
        carry_ticks(&before, &mut after);
        assert!(after[0].keep);
    }

    #[test]
    fn float_noise_below_a_millisecond_still_matches() {
        let before = vec![row(Kind::Silence, 2.0, 3.0, true)];
        let mut after = vec![row(Kind::Silence, 2.0 + 1e-9, 3.0 - 1e-9, false)];
        carry_ticks(&before, &mut after);
        assert!(after[0].keep);
    }

    #[test]
    fn nothing_to_carry_changes_nothing() {
        let mut after = vec![row(Kind::Silence, 2.0, 3.0, false)];
        carry_ticks(&[], &mut after);
        assert!(!after[0].keep);
    }
}
