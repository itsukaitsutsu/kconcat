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

use concat_project::sectionlist::{Row, clip_sections_with_options, kept_ranges};
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
    /// Re-run detection with the current controls.
    Detect,
    ThresholdChanged(f32),
    MinSilenceChanged(f32),
    MinSectionChanged(f32),
    UseCaptionsChanged(bool),
    SearchChanged(String),
    MinDurationChanged(f32),
    KindToggled(i32),
    SelectVisible,
    SelectNone,
    SelectInvert,
    SelectAllSilence,
    SelectAllCaptions,
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
    noise_db: f64,
    min_silence_ms: f64,
    min_section_ms: u32,
    use_captions: bool,
    search: String,
    min_duration: f64,
    show_caption: bool,
    show_silence: bool,
    show_other: bool,
}

impl SectionsPane {
    /// Applies one message, on the same terms as the captions pane: the
    /// studio's copy of this pane is a blank while this runs.
    pub fn update(&mut self, msg: SectionsMsg, studio: &mut Studio) {
        match msg {
            SectionsMsg::Open => self.open_on(studio),
            SectionsMsg::Detect => self.detect(studio),
            SectionsMsg::ThresholdChanged(value) => {
                self.noise_db = f64::from(value).clamp(-100.0, 0.0)
            }
            SectionsMsg::MinSilenceChanged(value) => {
                self.min_silence_ms = f64::from(value).clamp(0.0, 60_000.0)
            }
            SectionsMsg::MinSectionChanged(value) => {
                self.min_section_ms = value.clamp(0.0, 60_000.0).round() as u32
            }
            SectionsMsg::UseCaptionsChanged(value) => self.use_captions = value,
            SectionsMsg::SearchChanged(value) => self.search = value,
            SectionsMsg::MinDurationChanged(value) => {
                self.min_duration = f64::from(value).clamp(0.0, 3600.0)
            }
            SectionsMsg::KindToggled(kind) => match kind {
                0 => self.show_caption = !self.show_caption,
                1 => self.show_silence = !self.show_silence,
                2 => self.show_other = !self.show_other,
                _ => {}
            },
            SectionsMsg::SelectVisible => self.set_visible(false),
            SectionsMsg::SelectNone => self.rows.iter_mut().for_each(|row| row.keep = true),
            SectionsMsg::SelectInvert => {
                let visible = self.visible_indices();
                for index in visible {
                    self.rows[index].keep = !self.rows[index].keep;
                }
            }
            SectionsMsg::SelectAllSilence => self
                .rows
                .iter_mut()
                .filter(|row| row.section.kind == Kind::Silence)
                .for_each(|row| row.keep = false),
            SectionsMsg::SelectAllCaptions => self
                .rows
                .iter_mut()
                .filter(|row| row.section.kind == Kind::Caption)
                .for_each(|row| row.keep = false),
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
                        match clip_sections_with_options(
                            studio.timeline(),
                            &clip_id,
                            &silences,
                            PAD_SECONDS,
                            self.use_captions,
                            self.min_section_ms,
                        ) {
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
        self.noise_db = concat_media::silence::DEFAULT_NOISE_DB;
        self.min_silence_ms = concat_media::silence::DEFAULT_MIN_DURATION * 1000.0;
        self.min_section_ms = concat_project::sections::MIN_SECTION_MS;
        self.use_captions = true;
        self.show_caption = true;
        self.show_silence = true;
        self.show_other = true;
        self.detect(studio);
    }

    fn row_visible(&self, row: &Row) -> bool {
        let kind_visible = match row.section.kind {
            Kind::Caption => self.show_caption,
            Kind::Silence => self.show_silence,
            Kind::Other => self.show_other,
        };
        let query = self.search.trim().to_lowercase();
        kind_visible
            && row.section.duration() + f64::EPSILON >= self.min_duration
            && (query.is_empty() || row.section.text.to_lowercase().contains(&query))
    }

    fn visible_indices(&self) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| self.row_visible(row).then_some(index))
            .collect()
    }

    fn set_visible(&mut self, keep: bool) {
        let visible = self.visible_indices();
        for index in visible {
            self.rows[index].keep = keep;
        }
    }

    /// Run silence detection using the editable VTS-compatible controls.
    fn detect(&mut self, studio: &Studio) {
        let Some(clip) = self.clip.as_ref().and_then(|id| studio.clip(id)).cloned() else {
            self.message = t("sections.clipGone");
            return;
        };
        let Some(media) = studio.project().media_by_id(&clip.media_id).cloned() else {
            self.message = t("sections.clipHasNoFile");
            return;
        };
        self.running = true;
        self.rows.clear();
        self.message.clear();
        let path = std::path::PathBuf::from(&media.path);
        let stream = clip.audio_stream.and_then(|s| usize::try_from(s).ok());
        let noise_db = self.noise_db;
        let min_duration = self.min_silence_ms / 1000.0;
        spawn_in_project(
            move || {
                concat_media::silence::detect(&path, noise_db, min_duration, stream)
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
            .enumerate()
            .filter(|(_, row)| self.row_visible(row))
            .map(|(index, row)| SectionRowData {
                source_index: index as i32,
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
                start_ratio: ((row.timeline_start - timeline_start) / timeline_span).clamp(0.0, 1.0)
                    as f32,
                end_ratio: ((row.timeline_end - timeline_start) / timeline_span).clamp(0.0, 1.0)
                    as f32,
            })
            .collect();
        SectionsSheetData {
            open: self.open,
            running: self.running,
            noise_db: self.noise_db as f32,
            min_silence_ms: self.min_silence_ms as f32,
            min_section_ms: self.min_section_ms as f32,
            use_captions: self.use_captions,
            search: self.search.as_str().into(),
            min_duration: self.min_duration as f32,
            show_caption: self.show_caption,
            show_silence: self.show_silence,
            show_other: self.show_other,
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
    let milliseconds = (seconds.max(0.0) * 1000.0).round() as u32;
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
