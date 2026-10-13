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
    let clip = timeline.clip(clip_id)?;
    let window = clip.duration * clip.speed;

    // This clip's captions, moved onto the source clock.
    let mut cues: Vec<Cue> = timeline
        .captions_of(clip_id)
        .iter()
        .filter_map(|caption_id| timeline.clip(caption_id))
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
        })
        .filter(|cue| cue.end > 0.0 && cue.start < window)
        .collect();
    cues.sort_by(|a, b| a.start.total_cmp(&b.start));
    if !use_captions {
        cues.clear();
    }

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
        .map(|section| Row {
            timeline_start: sections::round_millis(clip.start + section.start / clip.speed),
            timeline_end: sections::round_millis(clip.start + section.end / clip.speed),
            keep: section.kind != Kind::Silence,
            section,
        })
        .collect();
    Some(ClipSections {
        clip_id: clip_id.to_owned(),
        rows,
        summary,
    })
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
