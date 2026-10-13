// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The Sections sheet: what a clip is made of, as an aligned timeline and a
//! scrollable card list.
//!
//! Opens on the selected clip with sound (or the first such clip), runs
//! `concat_media::silence::detect` over its file on a worker, and tiles
//! the clip with [`concat_project::sectionlist::clip_sections`] - whose
//! rows this sheet only displays and lets the reader tick. The numbers
//! (which row is silence, where it sits on the timeline, what the
//! ticked rows keep) all come from that crate and its tests; nothing is
//! computed here but labels.
//!
//! There is deliberately no Apply yet: turning ticks into cuts is the
//! autotrim (Phase 5), and a button that pretends to do it would be
//! worse than none. The sheet ends at "Keeping X of Y", which is the
//! promise the autotrim will be held to.

use concat_project::sectionlist::{Row, clip_sections, kept_ranges};
use concat_project::sections::Kind;

use crate::host::spawn_in_project;
use crate::i18n::t;
use crate::panes::Msg;
use crate::studio::Studio;
use crate::ui::{SectionRowData, SectionsSheetData};

/// Seconds of air kept at each end of a detected silence, so a cut does
/// not take the start off a word or the end off a sentence.
const PAD_SECONDS: f64 = 0.15;

/// Everything that can happen to the Sections sheet.
#[derive(Clone, Debug)]
pub enum SectionsMsg {
    /// The tray's Sections tool.
    Open,
    Close,
    /// A row's keep switch flipped.
    RowToggled(i32),
    /// Apply the ticks as an autotrim (Phase 5).
    Apply,
    /// The detector's worker is done: the file-clock silences, or why not.
    Finished(Result<Vec<(f64, f64)>, String>),
}

/// The Sections sheet's state.
#[derive(Default)]
pub struct SectionsPane {
    pub open: bool,
    pub running: bool,
    /// The clip being listed, kept while the worker runs: the selection
    /// could in principle change under it, and the rows must describe
    /// the clip that was measured.
    clip: Option<String>,
    /// The clip's name as the sheet title shows it.
    clip_label: String,
    /// The rows, once measured; their `keep` is the reader's tick.
    rows: Vec<Row>,
    /// Why the last run failed, when it did.
    pub message: String,
}

impl SectionsPane {
    /// Applies one message, on the same terms as the captions pane: the
    /// studio's copy of this pane is a blank while this runs.
    pub fn update(&mut self, msg: SectionsMsg, studio: &mut Studio) {
        match msg {
            SectionsMsg::Open => self.open_on(studio),
            SectionsMsg::Close => self.open = false,
            SectionsMsg::Apply => {
                let (Some(clip_id), false) = (self.clip.clone(), self.rows.is_empty()) else {
                    return;
                };
                let ticks: Vec<bool> = self.rows.iter().map(|row| row.keep).collect();
                let kept = kept_ranges(&self.rows, &ticks);
                // studio.apply is the editor's door: the autotrim is one
                // command, so the whole cut is one undo step.
                studio.apply(concat_project::Command::AutotrimClip { clip_id, kept });
                self.open = false;
                self.rows.clear();
                studio.notify(&t("sections.trimmed"), false);
            }
            SectionsMsg::RowToggled(index) => {
                if let Some(row) = usize::try_from(index)
                    .ok()
                    .and_then(|i| self.rows.get_mut(i))
                {
                    row.keep = !row.keep;
                }
            }
            SectionsMsg::Finished(result) => {
                self.running = false;
                let Some(clip_id) = self.clip.clone() else {
                    return;
                };
                match result {
                    Ok(silences) => {
                        match clip_sections(studio.timeline(), &clip_id, &silences, PAD_SECONDS) {
                            Some(view) => {
                                self.rows = view.rows;
                                if self.rows.is_empty() {
                                    self.message = t("sections.nothingFound");
                                }
                            }
                            None => self.message = t("sections.clipGone"),
                        }
                    }
                    Err(error) => self.message = error,
                }
            }
        }
    }

    /// The sheet opened on the selected clip with sound, else the first
    /// such clip, else an empty sheet saying there is nothing to list.
    fn open_on(&mut self, studio: &mut Studio) {
        let selected = studio.sole_selection().filter(|id| {
            studio
                .clip(id)
                .is_some_and(|clip| studio.clip_has_sound(clip))
        });
        let clip_id = selected.or_else(|| {
            studio
                .timeline()
                .clips
                .iter()
                .filter(|clip| studio.clip_has_sound(clip))
                .min_by(|a, b| a.start.total_cmp(&b.start))
                .map(|clip| clip.id.clone())
        });
        let Some(clip_id) = clip_id else {
            *self = SectionsPane {
                open: true,
                message: t("sections.noVoiceOnTimeline"),
                ..SectionsPane::default()
            };
            return;
        };
        let Some(clip) = studio.clip(&clip_id).cloned() else {
            return;
        };
        let Some(media) = studio.project().media_by_id(&clip.media_id).cloned() else {
            *self = SectionsPane {
                open: true,
                message: t("sections.clipHasNoFile"),
                ..SectionsPane::default()
            };
            return;
        };
        let clip_label = format!("{} · {}", media.name, clock(clip.start));
        *self = SectionsPane {
            open: true,
            running: true,
            clip: Some(clip_id),
            clip_label,
            ..SectionsPane::default()
        };
        let path = std::path::PathBuf::from(&media.path);
        let stream = clip.audio_stream.and_then(|s| usize::try_from(s).ok());
        spawn_in_project(
            move || {
                concat_media::silence::detect(
                    &path,
                    concat_media::silence::DEFAULT_NOISE_DB,
                    concat_media::silence::DEFAULT_MIN_DURATION,
                    stream,
                )
                .map_err(|error| error.to_string())
            },
            |studio, _, _, result| studio.handle(Msg::Sections(SectionsMsg::Finished(result))),
        );
    }

    /// The sheet as Slint shows it.
    pub fn data(&self, _studio: &Studio) -> SectionsSheetData {
        let ticks: Vec<bool> = self.rows.iter().map(|row| row.keep).collect();
        let kept = kept_ranges(&self.rows, &ticks);
        let kept_seconds: f64 = kept.iter().map(|(start, end)| end - start).sum();
        let total_seconds: f64 = self.rows.iter().map(|row| row.section.duration()).sum();
        let timeline_start = self.rows.first().map_or(0.0, |row| row.timeline_start);
        let timeline_end = self
            .rows
            .last()
            .map_or(timeline_start, |row| row.timeline_end);
        let timeline_span = (timeline_end - timeline_start).max(f64::EPSILON);
        let rows: Vec<SectionRowData> = self
            .rows
            .iter()
            .map(|row| SectionRowData {
                kind: kind_label(row.section.kind).into(),
                label: if row.section.kind == Kind::Caption && !row.section.text.is_empty() {
                    row.section.text.clone().into()
                } else {
                    kind_label(row.section.kind).into()
                },
                range: format!(
                    "{} – {}",
                    precise_clock(row.timeline_start),
                    precise_clock(row.timeline_end)
                )
                .into(),
                keep: row.keep,
                kind_code: match row.section.kind {
                    Kind::Caption => 0,
                    Kind::Silence => 1,
                    Kind::Other => 2,
                },
                start_ratio: ((row.timeline_start - timeline_start) / timeline_span)
                    .clamp(0.0, 1.0) as f32,
                end_ratio: ((row.timeline_end - timeline_start) / timeline_span)
                    .clamp(0.0, 1.0) as f32,
            })
            .collect();
        SectionsSheetData {
            open: self.open,
            running: self.running,
            clip_label: self.clip_label.clone().into(),
            summary: if self.rows.is_empty() {
                String::new().into()
            } else {
                format!(
                    "{} {} · {} {} · {} {}",
                    self.rows
                        .iter()
                        .filter(|r| r.section.kind == Kind::Caption)
                        .count(),
                    t("sections.captions"),
                    self.rows
                        .iter()
                        .filter(|r| r.section.kind == Kind::Silence)
                        .count(),
                    t("sections.silences"),
                    self.rows
                        .iter()
                        .filter(|r| r.section.kind == Kind::Other)
                        .count(),
                    t("sections.other"),
                )
                .into()
            },
            kept_label: if self.rows.is_empty() {
                String::new().into()
            } else {
                format!(
                    "{} {} / {}",
                    t("sections.keeping"),
                    clock(kept_seconds),
                    clock(total_seconds)
                )
                .into()
            },
            timeline_start: clock(timeline_start).into(),
            timeline_end: clock(timeline_end).into(),
            can_apply: !self.running && !self.rows.is_empty(),
            message: self.message.clone().into(),
            rows: std::rc::Rc::new(slint::VecModel::from(rows)).into(),
        }
    }
}

/// A section kind as the row's first word.
fn kind_label(kind: Kind) -> String {
    match kind {
        Kind::Caption => t("sections.caption"),
        Kind::Silence => t("sections.silence"),
        Kind::Other => t("sections.other"),
    }
}

/// Seconds as `m:ss` (or `h:mm:ss`), the way the sheets tell time.
fn clock(seconds: f64) -> String {
    let seconds = seconds.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// A section boundary with millisecond precision. Whole-second boundaries
/// stay compact; fractional boundaries cannot both look like `0:00` anymore.
fn precise_clock(seconds: f64) -> String {
    let milliseconds = (seconds.max(0.0) * 1000.0).round() as u64;
    let whole_seconds = milliseconds / 1000;
    let fraction = milliseconds % 1000;
    if fraction == 0 {
        return clock(whole_seconds as f64);
    }
    let (hours, minutes, seconds) = (
        whole_seconds / 3600,
        whole_seconds / 60 % 60,
        whole_seconds % 60,
    );
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}.{fraction:03}")
    } else {
        format!("{minutes}:{seconds:02}.{fraction:03}")
    }
}

#[cfg(test)]
mod tests {
    use super::{clock, precise_clock};

    #[test]
    fn a_clock_reads_like_the_sheets_tell_time() {
        assert_eq!(clock(0.0), "0:00");
        assert_eq!(clock(72.9), "1:12");
        assert_eq!(clock(3_725.0), "1:02:05");
        assert_eq!(clock(-1.0), "0:00");
    }

    #[test]
    fn section_boundaries_keep_fractional_seconds_visible() {
        assert_eq!(precise_clock(0.0), "0:00");
        assert_eq!(precise_clock(0.25), "0:00.250");
        assert_eq!(precise_clock(72.9), "1:12.900");
        assert_eq!(precise_clock(3_725.125), "1:02:05.125");
    }
}
