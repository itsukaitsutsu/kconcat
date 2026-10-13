// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The Sections sheet: what a clip is made of, as an aligned timeline and a
//! scrollable table (`ui/dialogs/sections.slint`).
//!
//! Opens on the selected clip with sound (or the first such clip), runs
//! `concat_media::silence::detect` over its file on a worker (when
//! `detect_silence` is enabled), and tiles the clip with
//! [`concat_project::sectionlist::clip_sections_with_cues`].

use std::path::PathBuf;
use std::rc::Rc;

use concat_project::Command;
use concat_project::commands::ClipPatch;
use concat_project::sectionlist::{Row, clip_sections_with_cues, kept_ranges};
use concat_project::sections::Kind;
use concat_project::subtitle::Cue;
use slint::VecModel;

use crate::host::spawn_in_project;
use crate::i18n::t;
use crate::panes::Msg;
use crate::studio::{Studio, sync};
use crate::ui::{SectionRowData, SectionsSheetData};

/// Seconds of air kept at each end of a detected silence, so a cut does
/// not take the start off a word or the end off a sentence.
const PAD_SECONDS: f64 = 0.15;

/// Default minimum silence duration in milliseconds (matches VTS `detect-section.txt`).
pub const DEFAULT_MIN_SILENCE_MS: f64 = 800.0;

/// Everything that can happen to the Sections sheet.
#[derive(Clone, Debug)]
pub enum SectionsMsg {
    /// The tray's Sections tool.
    Open,
    Close,
    /// A row's keep switch flipped.
    RowToggled(i32),
    /// Jump playhead to a row's timeline start.
    RowSeek(i32),
    /// Edit a caption row's text inline.
    RowTextEdited(i32, String),
    /// Re-run detection with the current controls.
    Detect,
    DetectSilenceChanged(bool),
    ThresholdChanged(f32),
    MinSilenceChanged(f32),
    MinSectionChanged(f32),
    UseCaptionsChanged(bool),
    SubtitlePathEdited(String),
    BrowseSubtitle,
    SubtitlePicked(PathBuf),
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
pub struct SectionsPane {
    pub open: bool,
    pub running: bool,
    /// The clip being listed, kept while the worker runs.
    clip: Option<String>,
    /// The clip's name as the sheet title shows it.
    clip_label: String,
    /// Track name (`V1`, `A1`, etc.) for the table's `track` column.
    track_label: String,
    /// The rows, once measured; their `keep` is the reader's tick.
    rows: Vec<Row>,
    /// Why the last run failed, when it did.
    pub message: String,
    detect_silence: bool,
    noise_db: f64,
    min_silence_ms: f64,
    min_section_ms: u32,
    use_captions: bool,
    subtitle_path: String,
    external_cues: Vec<Cue>,
    sub_info: String,
    search: String,
    min_duration: f64,
    show_caption: bool,
    show_silence: bool,
    show_other: bool,
    rows_model: Rc<VecModel<SectionRowData>>,
}

impl Default for SectionsPane {
    fn default() -> Self {
        Self {
            open: false,
            running: false,
            clip: None,
            clip_label: String::new(),
            track_label: "V1".to_owned(),
            rows: Vec::new(),
            message: String::new(),
            detect_silence: true,
            noise_db: concat_media::silence::DEFAULT_NOISE_DB,
            min_silence_ms: DEFAULT_MIN_SILENCE_MS,
            min_section_ms: concat_project::sections::MIN_SECTION_MS,
            use_captions: true,
            subtitle_path: String::new(),
            external_cues: Vec::new(),
            sub_info: String::new(),
            search: String::new(),
            min_duration: 0.0,
            show_caption: true,
            show_silence: true,
            show_other: true,
            rows_model: Rc::new(VecModel::default()),
        }
    }
}

impl SectionsPane {
    /// Applies one message, on the same terms as the captions pane: the
    /// studio's copy of this pane is a blank while this runs.
    pub fn update(&mut self, msg: SectionsMsg, studio: &mut Studio) {
        match msg {
            SectionsMsg::Open => self.open_on(studio),
            SectionsMsg::Detect => self.detect(studio),
            SectionsMsg::DetectSilenceChanged(on) => {
                self.detect_silence = on;
            }
            SectionsMsg::ThresholdChanged(value) => {
                if value.is_finite() {
                    self.noise_db = f64::from(value).clamp(-100.0, 0.0);
                }
            }
            SectionsMsg::MinSilenceChanged(value) => {
                if value.is_finite() {
                    self.min_silence_ms = f64::from(value).clamp(0.0, 60_000.0);
                }
            }
            SectionsMsg::MinSectionChanged(value) => {
                if value.is_finite() {
                    self.min_section_ms = value.clamp(0.0, 60_000.0).round() as u32;
                }
            }
            SectionsMsg::UseCaptionsChanged(value) => self.use_captions = value,
            SectionsMsg::SubtitlePathEdited(path) => self.set_subtitle_path(path),
            SectionsMsg::BrowseSubtitle => {
                let title = t("sections.uploadSubtitle");
                crate::host::after_dialog(
                    move || {
                        crate::platform::pick_files(&title, Some(("Subtitles", &["srt", "vtt"])))
                            .and_then(|mut paths| paths.drain(..).next())
                    },
                    |studio, _, _, picked| {
                        studio.handle(Msg::Sections(SectionsMsg::SubtitlePicked(picked)));
                    },
                );
            }
            SectionsMsg::SubtitlePicked(path) => {
                self.set_subtitle_path(path.to_string_lossy().into_owned());
            }
            SectionsMsg::SearchChanged(value) => self.search = value,
            SectionsMsg::MinDurationChanged(value) => {
                if value.is_finite() {
                    self.min_duration = f64::from(value).clamp(0.0, 3600.0);
                }
            }
            SectionsMsg::KindToggled(kind) => self.toggle_kind(kind),
            SectionsMsg::SelectVisible => self.set_visible(false),
            SectionsMsg::SelectNone => self.select_none(),
            SectionsMsg::SelectInvert => self.select_invert(),
            SectionsMsg::SelectAllSilence => self.select_all_silence(),
            SectionsMsg::SelectAllCaptions => self.select_all_captions(),
            SectionsMsg::Close => self.open = false,
            SectionsMsg::Apply => {
                let (Some(clip_id), false) = (self.clip.clone(), self.rows.is_empty()) else {
                    return;
                };
                let ticks: Vec<bool> = self.rows.iter().map(|row| row.keep).collect();
                let kept = kept_ranges(&self.rows, &ticks);
                if kept.is_empty() || self.rows.iter().all(|row| row.keep) {
                    return;
                }
                studio.apply(Command::AutotrimClip { clip_id, kept });
                self.open = false;
                self.rows.clear();
                studio.notify(&t("sections.trimmed"), false);
            }
            SectionsMsg::RowToggled(index) => self.toggle_row(index),
            SectionsMsg::RowSeek(index) => {
                if let Some(row) = usize::try_from(index).ok().and_then(|i| self.rows.get(i)) {
                    studio.seek(row.timeline_start as f32);
                }
            }
            SectionsMsg::RowTextEdited(index, text) => {
                if let Ok(idx) = usize::try_from(index) {
                    self.edit_row_text(idx, text, studio);
                }
            }
            SectionsMsg::Finished(result) => {
                self.running = false;
                let Some(clip_id) = self.clip.clone() else {
                    return;
                };
                match result {
                    Ok(silences) => self.populate_sections(studio, &clip_id, &silences),
                    Err(error) => self.message = error,
                }
            }
        }
    }

    pub fn set_subtitle_path(&mut self, path: String) {
        let trimmed = path.trim().to_owned();
        self.subtitle_path = path;
        if trimmed.is_empty() {
            self.external_cues.clear();
            self.sub_info.clear();
            return;
        }
        let path_ref = std::path::Path::new(&trimmed);
        let file_name = path_ref
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&trimmed);
        match std::fs::read_to_string(path_ref) {
            Ok(raw) => {
                let cues = concat_project::subtitle::parse(&raw);
                self.sub_info = format!("{} cues ({file_name})", cues.len());
                self.external_cues = cues;
                self.use_captions = true;
            }
            Err(err) => {
                self.external_cues.clear();
                self.sub_info = format!("{file_name}: {err}");
            }
        }
    }

    fn populate_sections(&mut self, studio: &Studio, clip_id: &str, silences: &[(f64, f64)]) {
        let linked_count = studio.timeline().captions_of(clip_id).len();
        if self.external_cues.is_empty() && self.subtitle_path.trim().is_empty() {
            if linked_count > 0 {
                self.sub_info = format!("{linked_count} {}", t("sections.captions"));
            } else {
                self.sub_info.clear();
            }
        }
        let ext = (!self.external_cues.is_empty()).then_some(self.external_cues.as_slice());
        match clip_sections_with_cues(
            studio.timeline(),
            clip_id,
            silences,
            PAD_SECONDS,
            self.use_captions,
            self.min_section_ms,
            ext,
        ) {
            Some(view) => {
                self.rows = view.rows;
                if self.rows.is_empty() {
                    self.message = t("sections.nothingFound");
                } else {
                    self.message.clear();
                }
            }
            None => self.message = t("sections.clipGone"),
        }
    }

    fn edit_row_text(&mut self, index: usize, text: String, studio: &mut Studio) {
        let Some(row) = self.rows.get_mut(index) else {
            return;
        };
        if row.section.kind != Kind::Caption {
            return;
        }
        row.section.text = text.clone();
        let sec_start = row.section.start;
        let sec_end = row.section.end;
        let tl_start = row.timeline_start;
        let tl_end = row.timeline_end;

        for cue in &mut self.external_cues {
            if (cue.start - sec_start).abs() < 0.05 && (cue.end - sec_end).abs() < 0.05 {
                cue.text = text.clone();
            }
        }

        let Some(clip_id) = self.clip.clone() else {
            return;
        };
        let matching_captions: Vec<(String, concat_project::model::TextStyle)> = studio
            .timeline()
            .captions_of(&clip_id)
            .into_iter()
            .filter_map(|cap_id| studio.timeline().clip(&cap_id))
            .filter(|cap| cap.start < tl_end - 1e-3 && cap.start + cap.duration > tl_start + 1e-3)
            .filter_map(|cap| cap.text.clone().map(|style| (cap.id.clone(), style)))
            .collect();

        for (cap_id, mut style) in matching_captions {
            if style.content != text {
                style.content = text.clone();
                studio.apply(Command::UpdateClip {
                    clip_id: cap_id,
                    patch: ClipPatch {
                        name: None,
                        text: Some(Some(style)),
                        ..Default::default()
                    },
                });
            }
        }
    }

    fn toggle_kind(&mut self, kind: i32) {
        match kind {
            0 => self.show_caption = !self.show_caption,
            1 => self.show_silence = !self.show_silence,
            2 => self.show_other = !self.show_other,
            _ => {}
        }
    }

    fn toggle_row(&mut self, index: i32) {
        if let Some(row) = usize::try_from(index)
            .ok()
            .and_then(|i| self.rows.get_mut(i))
        {
            row.keep = !row.keep;
        }
    }

    fn select_none(&mut self) {
        self.rows.iter_mut().for_each(|row| row.keep = true);
    }

    fn select_invert(&mut self) {
        let visible = self.visible_indices();
        for index in visible {
            self.rows[index].keep = !self.rows[index].keep;
        }
    }

    fn select_all_silence(&mut self) {
        self.rows
            .iter_mut()
            .filter(|row| row.section.kind == Kind::Silence)
            .for_each(|row| row.keep = false);
    }

    fn select_all_captions(&mut self) {
        self.rows
            .iter_mut()
            .filter(|row| row.section.kind == Kind::Caption)
            .for_each(|row| row.keep = false);
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
                detect_silence: self.detect_silence,
                noise_db: self.noise_db,
                min_silence_ms: self.min_silence_ms,
                min_section_ms: self.min_section_ms,
                use_captions: self.use_captions,
                subtitle_path: self.subtitle_path.clone(),
                external_cues: self.external_cues.clone(),
                sub_info: self.sub_info.clone(),
                rows_model: Rc::clone(&self.rows_model),
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
                detect_silence: self.detect_silence,
                noise_db: self.noise_db,
                min_silence_ms: self.min_silence_ms,
                min_section_ms: self.min_section_ms,
                use_captions: self.use_captions,
                subtitle_path: self.subtitle_path.clone(),
                external_cues: self.external_cues.clone(),
                sub_info: self.sub_info.clone(),
                rows_model: Rc::clone(&self.rows_model),
                ..SectionsPane::default()
            };
            return;
        };
        let track_label = studio
            .timeline()
            .tracks
            .iter()
            .position(|tr| tr.id == clip.track_id)
            .map_or_else(|| "V1".to_owned(), |idx| format!("V{}", idx + 1));
        let clip_label = format!("{} · {}", media.name, clock(clip.start));
        *self = SectionsPane {
            open: true,
            running: self.detect_silence,
            clip: Some(clip_id),
            clip_label,
            track_label,
            detect_silence: self.detect_silence,
            noise_db: self.noise_db,
            min_silence_ms: self.min_silence_ms,
            min_section_ms: self.min_section_ms,
            use_captions: self.use_captions,
            subtitle_path: self.subtitle_path.clone(),
            external_cues: self.external_cues.clone(),
            sub_info: self.sub_info.clone(),
            rows_model: Rc::clone(&self.rows_model),
            ..SectionsPane::default()
        };
        self.detect(studio);
    }

    fn row_visible(&self, row: &Row) -> bool {
        let kind_visible = match row.section.kind {
            Kind::Caption => self.show_caption,
            Kind::Silence => self.show_silence,
            Kind::Other => self.show_other,
        };
        let query = self.search.trim().to_lowercase();
        let kind_name = match row.section.kind {
            Kind::Caption => "caption",
            Kind::Silence => "silence",
            Kind::Other => "other",
        };
        kind_visible
            && row.section.duration() + f64::EPSILON >= self.min_duration
            && (query.is_empty()
                || row.section.text.to_lowercase().contains(&query)
                || kind_name.contains(&query))
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
        if !self.detect_silence {
            self.running = false;
            self.populate_sections(studio, &clip.id, &[]);
            return;
        }
        let Some(media) = studio.project().media_by_id(&clip.media_id).cloned() else {
            self.message = t("sections.clipHasNoFile");
            return;
        };
        let path = PathBuf::from(&media.path);
        let stream = clip.audio_stream.and_then(|s| usize::try_from(s).ok());
        let noise_db = self.noise_db;
        let min_duration = (self.min_silence_ms / 1000.0).max(0.01);
        self.running = true;
        self.rows.clear();
        self.message.clear();
        spawn_in_project(
            move || {
                concat_media::silence::detect(&path, noise_db, min_duration, stream)
                    .map_err(|error| error.to_string())
            },
            |studio, _, _, result| studio.handle(Msg::Sections(SectionsMsg::Finished(result))),
        );
    }

    /// The sheet as Slint shows it.
    pub fn data(&self, studio: &Studio) -> SectionsSheetData {
        let playhead = f64::from(studio.playhead);
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
                track: self.track_label.clone().into(),
                start: precise_clock(row.timeline_start).into(),
                end: precise_clock(row.timeline_end).into(),
                duration: precise_clock(row.section.duration()).into(),
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
                at_playhead: playhead >= row.timeline_start && playhead < row.timeline_end,
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
        sync(&self.rows_model, rows);
        SectionsSheetData {
            open: self.open,
            running: self.running,
            detect_silence: self.detect_silence,
            noise_db: self.noise_db as f32,
            min_silence_ms: self.min_silence_ms as f32,
            min_section_ms: self.min_section_ms as f32,
            use_captions: self.use_captions,
            subtitle_path: self.subtitle_path.as_str().into(),
            sub_info: self.sub_info.as_str().into(),
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
            can_apply: !self.running && !kept.is_empty() && self.rows.iter().any(|row| !row.keep),
            message: self.message.clone().into(),
            rows: slint::ModelRc::from(Rc::clone(&self.rows_model)),
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
    use concat_project::sectionlist::Row;
    use concat_project::sections::{Kind, Section};

    use super::{DEFAULT_MIN_SILENCE_MS, SectionsPane, clock, precise_clock};

    fn sample_rows() -> Vec<Row> {
        vec![
            Row {
                section: Section {
                    kind: Kind::Caption,
                    start: 0.0,
                    end: 2.0,
                    text: "Hello world".to_owned(),
                },
                timeline_start: 0.0,
                timeline_end: 2.0,
                keep: true,
            },
            Row {
                section: Section {
                    kind: Kind::Silence,
                    start: 2.0,
                    end: 3.5,
                    text: String::new(),
                },
                timeline_start: 2.0,
                timeline_end: 3.5,
                keep: false,
            },
            Row {
                section: Section {
                    kind: Kind::Other,
                    start: 3.5,
                    end: 4.0,
                    text: String::new(),
                },
                timeline_start: 3.5,
                timeline_end: 4.0,
                keep: true,
            },
        ]
    }

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

    #[test]
    fn sections_pane_filters_and_bulk_selection_actions_work() {
        let mut pane = SectionsPane {
            rows: sample_rows(),
            ..SectionsPane::default()
        };
        assert!(pane.detect_silence);
        assert_eq!(pane.noise_db, concat_media::silence::DEFAULT_NOISE_DB);
        assert_eq!(pane.min_silence_ms, DEFAULT_MIN_SILENCE_MS);
        assert_eq!(
            pane.min_section_ms,
            concat_project::sections::MIN_SECTION_MS
        );
        assert!(pane.use_captions);
        assert_eq!(pane.visible_indices(), vec![0, 1, 2]);

        // Filter by minimum duration (1.0 s drops the 0.5 s Other row).
        pane.min_duration = 1.0;
        assert_eq!(pane.visible_indices(), vec![0, 1]);

        // Filter by search text / kind.
        pane.min_duration = 0.0;
        pane.search = "WORLD".to_owned();
        assert_eq!(pane.visible_indices(), vec![0]);
        pane.search = "silence".to_owned();
        assert_eq!(pane.visible_indices(), vec![1]);
        pane.search.clear();

        // Filter by kind toggle.
        pane.toggle_kind(1); // hide Silence
        assert_eq!(pane.visible_indices(), vec![0, 2]);

        // SelectVisible marks visible rows for removal (keep = false).
        pane.set_visible(false);
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![false, false, false]
        );

        // SelectNone restores all rows to kept (keep = true).
        pane.select_none();
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![true, true, true]
        );

        // SelectInvert flips only the visible rows ([0, 2]).
        pane.select_invert();
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![false, true, false]
        );

        // SelectAllSilence and SelectAllCaptions target their respective kinds.
        pane.select_none();
        pane.select_all_silence();
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![true, false, true]
        );
        pane.select_none();
        pane.select_all_captions();
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![false, true, true]
        );
    }
}
