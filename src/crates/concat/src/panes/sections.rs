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
use concat_project::sectionlist::{Row, carry_ticks, clip_sections_with_cues, kept_ranges};
use concat_project::sections::Kind;
use concat_project::subtitle::Cue;
use slint::VecModel;

use crate::host::spawn_in_project;
use crate::i18n::{t, tf};
use crate::panes::Msg;
use crate::studio::{Studio, sync};
use crate::ui::{SectionRowData, SectionsSheetData, TimelineSectionData};

/// Seconds of air kept at each end of a detected silence, so a cut does
/// not take the start off a word or the end off a sentence.
const PAD_SECONDS: f64 = 0.15;

/// Default minimum silence duration in milliseconds (matches VTS `detect-section.txt`).
pub const DEFAULT_MIN_SILENCE_MS: f64 = 800.0;

/// Everything that can happen to the Sections sheet.
#[derive(Clone, Debug)]
pub enum SectionsMsg {
    /// The tray's Sections tool (modal).
    Open,
    /// Populate the dockable Sections workspace panel without opening the modal.
    OpenPanel,
    Close,
    /// A row's keep switch flipped.
    RowToggled(i32),
    /// Shift-click range selection across visible rows.
    RowShiftSelect(i32),
    /// Hold-and-drag checkbox paint started on a source row index.
    RowDragBegin(i32),
    /// Hold-and-drag checkbox paint moved over a visible row index.
    RowDragOver(i32),
    /// Jump playhead to a row's timeline start.
    RowSeek(i32),
    /// Edit a caption row's text inline.
    RowTextEdited(i32, String),
    /// User scrolled the Sections rows table directly (passes top/focus visible row index).
    ListScrolled(i32),
    /// User clicked a section bar in the Timeline Ruler section strip.
    TimelineSectionToggled(i32),
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
    /// Put the kept caption sections (or loaded .srt/.vtt cues) onto the Concat timeline, replacing existing captions.
    AddCaptionsToTimeline,
    /// Add the kept caption sections (or loaded .srt/.vtt cues) as a new dedicated caption track above existing tracks.
    AddCaptionsNewTrack,
    /// Delete the timeline caption clips matching checked rows without trimming the video clip.
    DeleteSelectedCaptions,
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
    /// The video/audio clip ID on the timeline that each row belongs to (for multi-clip tracks).
    row_clip_ids: Vec<String>,
    /// Cached silence spans from the last FFmpeg pass so caption deletions can re-tile immediately.
    silences_cache: Vec<(f64, f64)>,
    /// Last source row index clicked in the table, used for Shift+click range selection.
    last_clicked_row: Option<usize>,
    /// Visible row index where the current hold-and-drag checkbox gesture started.
    drag_anchor_visible: Option<usize>,
    /// The `keep` value painted during the current hold-and-drag checkbox gesture.
    drag_target_keep: bool,
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
    pub show_caption: bool,
    pub show_silence: bool,
    pub show_other: bool,
    pub scroll_to_row: i32,
    pub scroll_sync_seq: i32,
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
            row_clip_ids: Vec::new(),
            silences_cache: Vec::new(),
            last_clicked_row: None,
            drag_anchor_visible: None,
            drag_target_keep: false,
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
            scroll_to_row: -1,
            scroll_sync_seq: 0,
            rows_model: Rc::new(VecModel::default()),
        }
    }
}

impl SectionsPane {
    /// Applies one message, on the same terms as the captions pane: the
    /// studio's copy of this pane is a blank while this runs.
    pub fn update(&mut self, msg: SectionsMsg, studio: &mut Studio) {
        match msg {
            SectionsMsg::Open => self.open_on(studio, true),
            SectionsMsg::OpenPanel => self.open_on(studio, false),
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
            SectionsMsg::SearchChanged(value) => {
                self.search = value;
                self.sync_scroll_to_time(f64::from(studio.playhead), true);
            }
            SectionsMsg::MinDurationChanged(value) => {
                if value.is_finite() {
                    self.min_duration = f64::from(value).clamp(0.0, 3600.0);
                    self.sync_scroll_to_time(f64::from(studio.playhead), true);
                }
            }
            SectionsMsg::KindToggled(kind) => {
                self.toggle_kind(kind);
                self.ensure_populated(studio);
                self.sync_timeline_selection(studio);
                self.sync_scroll_to_time(f64::from(studio.playhead), true);
            }
            SectionsMsg::SelectVisible => {
                self.ensure_populated(studio);
                self.set_visible(false);
                self.sync_timeline_selection(studio);
            }
            SectionsMsg::SelectNone => {
                self.select_none();
                self.sync_timeline_selection(studio);
            }
            SectionsMsg::SelectInvert => {
                self.ensure_populated(studio);
                self.select_invert();
                self.sync_timeline_selection(studio);
            }
            SectionsMsg::SelectAllSilence => {
                self.ensure_populated(studio);
                self.select_all_silence();
                self.sync_timeline_selection(studio);
            }
            SectionsMsg::SelectAllCaptions => {
                self.ensure_populated(studio);
                self.select_all_captions();
                self.sync_timeline_selection(studio);
            }
            SectionsMsg::AddCaptionsToTimeline => self.insert_captions_into_timeline(studio, false),
            SectionsMsg::AddCaptionsNewTrack => self.insert_captions_into_timeline(studio, false),
            SectionsMsg::DeleteSelectedCaptions => self.delete_selected_captions(studio),
            SectionsMsg::Close => self.open = false,
            SectionsMsg::Apply => {
                self.apply_trim(studio);
            }
            SectionsMsg::RowToggled(index) => {
                self.toggle_row(index);
                self.sync_timeline_selection(studio);
                if let Some(row) = usize::try_from(index).ok().and_then(|i| self.rows.get(i)) {
                    studio
                        .lanes
                        .ensure_span_visible(row.timeline_start as f32, row.timeline_end as f32);
                }
            }
            SectionsMsg::TimelineSectionToggled(index) => {
                self.toggle_row(index);
                self.sync_timeline_selection(studio);
                self.sync_scroll_to_source_row(index);
            }
            SectionsMsg::RowShiftSelect(index) => {
                self.shift_select_row(index);
                self.sync_timeline_selection(studio);
                if let Some(row) = usize::try_from(index).ok().and_then(|i| self.rows.get(i)) {
                    studio
                        .lanes
                        .ensure_span_visible(row.timeline_start as f32, row.timeline_end as f32);
                }
            }
            SectionsMsg::RowDragBegin(index) => {
                self.drag_begin_row(index);
                self.sync_timeline_selection(studio);
            }
            SectionsMsg::RowDragOver(visible_index) => {
                self.drag_over_visible(visible_index);
                self.sync_timeline_selection(studio);
            }
            SectionsMsg::RowSeek(index) => {
                if let Some(row) = usize::try_from(index)
                    .ok()
                    .and_then(|i| self.rows.get(i))
                    .cloned()
                {
                    studio.seek(row.timeline_start as f32);
                    studio
                        .lanes
                        .ensure_span_visible(row.timeline_start as f32, row.timeline_end as f32);
                    if row.section.kind == Kind::Caption {
                        // A row carries its originating caption id. Falling
                        // back to text + time keeps older/external rows
                        // usable, but never selects a hand-made title merely
                        // because it happens to overlap a subtitle.
                        let matching: Vec<String> = match row.caption_id {
                            Some(caption_id) if studio.timeline().clip(&caption_id).is_some() => {
                                vec![caption_id]
                            }
                            _ => studio
                                .timeline()
                                .clips
                                .iter()
                                .filter(|c| {
                                    is_caption_clip(c)
                                        && c.start < row.timeline_end - 1e-3
                                        && c.start + c.duration > row.timeline_start + 1e-3
                                        && c.text
                                            .as_ref()
                                            .is_some_and(|style| style.content == row.section.text)
                                })
                                .map(|c| c.id.clone())
                                .collect(),
                        };
                        if !matching.is_empty() {
                            studio.selection = matching;
                        }
                    }
                }
            }
            SectionsMsg::RowTextEdited(index, text) => {
                if let Ok(idx) = usize::try_from(index) {
                    if let Some(row) = self.rows.get(idx) {
                        studio.lanes.ensure_span_visible(
                            row.timeline_start as f32,
                            row.timeline_end as f32,
                        );
                    }
                    self.edit_row_text(idx, text, studio);
                }
            }
            SectionsMsg::ListScrolled(visible_index) => {
                if visible_index <= 0 {
                    studio.lanes.scroll_to_time(0.0);
                } else {
                    let visible = self.visible_indices();
                    if let Some(&src_idx) = usize::try_from(visible_index)
                        .ok()
                        .and_then(|vi| visible.get(vi))
                        && let Some(row) = self.rows.get(src_idx)
                    {
                        studio.lanes.scroll_to_time(row.timeline_start as f32);
                    }
                }
            }
            SectionsMsg::Finished(result) => {
                self.running = false;
                let Some(clip_id) = self.clip.clone() else {
                    return;
                };
                match result {
                    Ok(silences) => {
                        self.silences_cache = silences.clone();
                        self.populate_sections(studio, &clip_id, &silences);
                        // A finished detection brings the list to the playhead.
                        // Only that: a rebuild after an edit must leave the list
                        // where the reader is typing or scrolling it.
                        self.sync_scroll_to_time(f64::from(studio.playhead), true);
                    }
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
        let Some(anchor) = studio.timeline().clip(clip_id) else {
            self.rows.clear();
            self.row_clip_ids.clear();
            self.message = t("sections.clipGone");
            return;
        };
        let track_id = anchor.track_id.clone();
        let media_id = anchor.media_id.clone();
        let mut track_clips: Vec<String> = studio
            .timeline()
            .clips
            .iter()
            .filter(|c| c.track_id == track_id && c.media_id == media_id)
            .map(|c| c.id.clone())
            .collect();
        track_clips.sort_by(|a, b| {
            let sa = studio.timeline().clip(a).map_or(0.0, |c| c.start);
            let sb = studio.timeline().clip(b).map_or(0.0, |c| c.start);
            sa.total_cmp(&sb)
        });
        if track_clips.is_empty() {
            track_clips.push(clip_id.to_owned());
        }

        let total_captions = studio
            .timeline()
            .clips
            .iter()
            .filter(|c| is_caption_clip(c))
            .count();
        if self.external_cues.is_empty() && self.subtitle_path.trim().is_empty() {
            if total_captions > 0 {
                self.sub_info = format!("{total_captions} {}", t("sections.captions"));
            } else {
                self.sub_info.clear();
            }
        }

        // Carve silences around active caption cues on the source clock so no
        // detected silence span ever overlaps or chops into a spoken caption.
        let carved_silences: Vec<(f64, f64)> = if self.use_captions {
            let mut cue_spans: Vec<(f64, f64)> = if !self.external_cues.is_empty() {
                self.external_cues
                    .iter()
                    .map(|c| (c.start, c.end))
                    .collect()
            } else {
                let mut spans = Vec::new();
                for cid in &track_clips {
                    if let Some(c) = studio.timeline().clip(cid) {
                        for cap in studio.timeline().clips.iter().filter(|cap| {
                            is_caption_clip(cap)
                                && cap.start < c.start + c.duration - 1e-3
                                && cap.start + cap.duration > c.start + 1e-3
                        }) {
                            let rel_s = (cap.start - c.start).max(0.0) * c.speed;
                            let rel_e =
                                ((cap.start + cap.duration - c.start).min(c.duration)) * c.speed;
                            if rel_e > rel_s {
                                spans.push((c.source_start + rel_s, c.source_start + rel_e));
                            }
                        }
                    }
                }
                spans
            };
            carve_silences_around_cues(silences, &mut cue_spans)
        } else {
            silences.to_vec()
        };

        let ext = (!self.external_cues.is_empty()).then_some(self.external_cues.as_slice());
        let mut all_rows = Vec::new();
        let mut all_clip_ids = Vec::new();
        for cid in &track_clips {
            if let Some(view) = clip_sections_with_cues(
                studio.timeline(),
                cid,
                &carved_silences,
                PAD_SECONDS,
                self.use_captions,
                self.min_section_ms,
                ext,
            ) {
                for row in view.rows {
                    all_rows.push(row);
                    all_clip_ids.push(cid.clone());
                }
            }
        }
        // The reader's ticks survive the rebuild: an edit anywhere in the
        // document must not put back the silences they chose to keep.
        carry_ticks(&self.rows, &mut all_rows);
        self.rows = all_rows;
        self.row_clip_ids = all_clip_ids;
        if self.rows.is_empty() {
            self.message = t("sections.nothingFound");
        } else {
            self.message.clear();
        }
    }

    /// Ensures `self.rows` is populated from the current timeline and cached
    /// silences (and kicks off background silence detection if needed) when the
    /// user interacts with the synchronized Timeline Filter bar.
    pub fn ensure_populated(&mut self, studio: &mut Studio) {
        if self.clip.is_none()
            || self
                .clip
                .as_deref()
                .is_some_and(|id| studio.timeline().clip(id).is_none())
        {
            let candidate = studio
                .selection
                .iter()
                .find_map(|id| studio.timeline().clip(id))
                .filter(|c| c.kind.is_visual() || studio.clip_has_sound(c))
                .or_else(|| {
                    studio
                        .timeline()
                        .clips
                        .iter()
                        .filter(|c| c.kind.is_visual() || studio.clip_has_sound(c))
                        .min_by(|a, b| a.start.total_cmp(&b.start))
                        .map(|v| &**v)
                });
            if let Some(clip) = candidate {
                self.clip = Some(clip.id.clone());
                self.clip_label = clip.name.clone();
            }
        }
        if self.rows.is_empty()
            && let Some(clip_id) = self.clip.clone()
        {
            let silences = self.silences_cache.clone();
            self.populate_sections(studio, &clip_id, &silences);
            if self.detect_silence && self.silences_cache.is_empty() && !self.running {
                self.detect(studio);
            }
        }
    }

    /// Re-tiles `self.rows` against the current timeline after an edit or caption
    /// generation/import so the Timeline Section Strip and Sections sheet stay
    /// continuously in sync.
    pub fn refresh_from_timeline(&mut self, studio: &Studio) {
        let resolved_clip = self
            .clip
            .as_deref()
            .filter(|id| studio.timeline().clip(id).is_some())
            .map(str::to_owned)
            .or_else(|| {
                studio
                    .timeline()
                    .clips
                    .iter()
                    .filter(|c| c.kind.is_visual() || studio.clip_has_sound(c))
                    .min_by(|a, b| a.start.total_cmp(&b.start))
                    .map(|c| c.id.clone())
            });
        let Some(clip_id) = resolved_clip else {
            self.rows.clear();
            self.row_clip_ids.clear();
            return;
        };
        self.clip = Some(clip_id.clone());
        let silences = self.silences_cache.clone();
        self.populate_sections(studio, &clip_id, &silences);
    }

    pub fn can_remove_selected(&self) -> bool {
        !self.running
            && !self.rows.is_empty()
            && self.rows.iter().any(|row| row.keep)
            && self.rows.iter().any(|row| !row.keep)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn timeline_sections(&self) -> Vec<TimelineSectionData> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.row_visible(row))
            .map(|(index, row)| TimelineSectionData {
                source_index: index as i32,
                start: row.timeline_start as f32,
                duration: row.section.duration().max(0.02) as f32,
                kind_code: match row.section.kind {
                    Kind::Caption => 0,
                    Kind::Silence => 1,
                    Kind::Other => 2,
                },
                selected: !row.keep,
                label: if row.section.kind == Kind::Caption && !row.section.text.is_empty() {
                    row.section.text.clone().into()
                } else {
                    kind_label(row.section.kind).into()
                },
            })
            .collect()
    }

    fn apply_trim(&mut self, studio: &mut Studio) {
        let (Some(clip_id), false) = (self.clip.clone(), self.rows.is_empty()) else {
            return;
        };
        let all_ticks: Vec<bool> = self.rows.iter().map(|row| row.keep).collect();
        if kept_ranges(&self.rows, &all_ticks).is_empty() || self.rows.iter().all(|row| row.keep) {
            return;
        }
        let mut commands = Vec::new();
        let has_timeline_captions = studio.timeline().clips.iter().any(|c| is_caption_clip(c));
        if self.use_captions && (!self.external_cues.is_empty() || !has_timeline_captions) {
            let caption_cmds = self.build_caption_commands(studio, &clip_id, false);
            if !caption_cmds.is_empty() {
                commands.push(Command::AddTrack { bottom: false });
                commands.extend(caption_cmds);
            }
        }

        // Group rows by their owning video/audio clip on the track and apply
        // trims from right to left (descending clip start time) so earlier
        // ripple shifts never invalidate later clip ranges.
        let mut groups: Vec<(String, Vec<Row>)> = Vec::new();
        for (idx, row) in self.rows.iter().enumerate() {
            let owner = self
                .row_clip_ids
                .get(idx)
                .cloned()
                .unwrap_or_else(|| clip_id.clone());
            if let Some(last) = groups.last_mut().filter(|(id, _)| id == &owner) {
                last.1.push(row.clone());
            } else {
                groups.push((owner, vec![row.clone()]));
            }
        }
        groups.sort_by(|(id_a, rows_a), (id_b, rows_b)| {
            let start_a = studio
                .timeline()
                .clip(id_a)
                .map(|c| c.start)
                .or_else(|| rows_a.first().map(|r| r.timeline_start))
                .unwrap_or(0.0);
            let start_b = studio
                .timeline()
                .clip(id_b)
                .map(|c| c.start)
                .or_else(|| rows_b.first().map(|r| r.timeline_start))
                .unwrap_or(0.0);
            start_b.total_cmp(&start_a)
        });

        for (owner_id, group_rows) in groups {
            if group_rows.iter().all(|r| r.keep) {
                continue;
            }
            let ticks: Vec<bool> = group_rows.iter().map(|r| r.keep).collect();
            let kept = kept_ranges(&group_rows, &ticks);
            // An empty `kept` is a whole-clip removal: the clip, its sound and
            // the captions over it go, and the gap closes on every lane, the
            // same way a partial cut closes it. (RemoveClips' ripple closed
            // only the clip's own lane, which left its sound behind.)
            commands.push(Command::AutotrimClip {
                clip_id: owner_id,
                kept,
            });
        }

        studio.apply(Command::Batch { commands });
        self.external_cues.clear();
        // The rows describe the trim's old timeline; none of their ticks
        // mean anything against the new one, so they are not carried over.
        self.rows.clear();
        self.refresh_from_timeline(studio);
        studio.notify(&t("sections.trimmed"), false);
    }

    fn edit_row_text(&mut self, index: usize, text: String, studio: &mut Studio) {
        let Some(row) = self.rows.get_mut(index) else {
            return;
        };
        if row.section.kind != Kind::Caption {
            return;
        }
        // The caption this row shows is the timeline clip whose words are
        // the row's old words. Matching on the words, not only on the time,
        // keeps a translation on another caption track (same span, other
        // language) from being overwritten by an edit to this one.
        let previous_text = std::mem::replace(&mut row.section.text, text.clone());
        let caption_id = row.caption_id.clone();
        let sec_start = row.section.start;
        let sec_end = row.section.end;
        let tl_start = row.timeline_start;
        let tl_end = row.timeline_end;

        for cue in &mut self.external_cues {
            if (cue.start - sec_start).abs() < 0.05 && (cue.end - sec_end).abs() < 0.05 {
                cue.text = text.clone();
            }
        }

        let matching_captions: Vec<(String, concat_project::model::TextStyle)> =
            if let Some(caption_id) = caption_id {
                studio
                    .timeline()
                    .clip(&caption_id)
                    .filter(|cap| is_caption_clip(cap))
                    .and_then(|cap| cap.text.clone().map(|style| (cap.id.clone(), style)))
                    .into_iter()
                    .collect()
            } else {
                // External/old rows did not carry an id. Match their words
                // as well as their range, and exclude manual title clips.
                studio
                    .timeline()
                    .clips
                    .iter()
                    .filter(|cap| {
                        is_caption_clip(cap)
                            && cap.start < tl_end - 1e-3
                            && cap.start + cap.duration > tl_start + 1e-3
                    })
                    .filter_map(|cap| cap.text.clone().map(|style| (cap.id.clone(), style)))
                    .filter(|(_, style)| style.content == previous_text)
                    .collect()
            };

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
        if let Some(idx) = usize::try_from(index).ok()
            && let Some(row) = self.rows.get_mut(idx)
        {
            row.keep = !row.keep;
            self.last_clicked_row = Some(idx);
        }
    }

    fn shift_select_row(&mut self, index: i32) {
        let Some(target_idx) = usize::try_from(index).ok() else {
            return;
        };
        if target_idx >= self.rows.len() {
            return;
        }
        let target_keep = !self.rows[target_idx].keep;
        let visible = self.visible_indices();
        if let Some(anchor_idx) = self.last_clicked_row
            && let (Some(a_pos), Some(b_pos)) = (
                visible.iter().position(|&i| i == anchor_idx),
                visible.iter().position(|&i| i == target_idx),
            )
        {
            let (lo, hi) = if a_pos <= b_pos {
                (a_pos, b_pos)
            } else {
                (b_pos, a_pos)
            };
            for &row_idx in &visible[lo..=hi] {
                self.rows[row_idx].keep = target_keep;
            }
        } else {
            self.rows[target_idx].keep = target_keep;
        }
        self.last_clicked_row = Some(target_idx);
    }

    fn drag_begin_row(&mut self, index: i32) {
        let Some(idx) = usize::try_from(index).ok() else {
            return;
        };
        if idx >= self.rows.len() {
            return;
        }
        let next_keep = !self.rows[idx].keep;
        self.rows[idx].keep = next_keep;
        self.drag_target_keep = next_keep;
        self.last_clicked_row = Some(idx);
        let visible = self.visible_indices();
        self.drag_anchor_visible = visible.iter().position(|&i| i == idx);
    }

    fn drag_over_visible(&mut self, visible_index: i32) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        let target_vis = visible_index.clamp(0, visible.len().saturating_sub(1) as i32) as usize;
        let anchor_vis = self.drag_anchor_visible.unwrap_or(target_vis);
        let (lo, hi) = if anchor_vis <= target_vis {
            (anchor_vis, target_vis)
        } else {
            (target_vis, anchor_vis)
        };
        for &row_idx in &visible[lo..=hi.min(visible.len() - 1)] {
            self.rows[row_idx].keep = self.drag_target_keep;
        }
        if let Some(&row_idx) = visible.get(target_vis) {
            self.last_clicked_row = Some(row_idx);
        }
    }

    /// Returns precisely the caption clips selected through the Sections
    /// rows. A caption row normally names its source clip; the time/text
    /// fallback is only for an old document or an external subtitle row.
    fn matching_selected_caption_ids(&self, studio: &Studio) -> Vec<String> {
        let selected: Vec<&Row> = self
            .rows
            .iter()
            .filter(|row| !row.keep && row.section.kind == Kind::Caption)
            .collect();
        if selected.is_empty() {
            return Vec::new();
        }
        let direct: std::collections::HashSet<&str> = selected
            .iter()
            .filter_map(|row| row.caption_id.as_deref())
            .collect();
        let fallback: Vec<&Row> = selected
            .iter()
            .copied()
            .filter(|row| row.caption_id.is_none())
            .collect();
        studio
            .timeline()
            .clips
            .iter()
            .filter(|clip| {
                is_caption_clip(clip)
                    && (direct.contains(clip.id.as_str())
                        || fallback.iter().any(|row| {
                            clip.start < row.timeline_end - 1e-3
                                && clip.start + clip.duration > row.timeline_start + 1e-3
                                && clip
                                    .text
                                    .as_ref()
                                    .is_some_and(|style| style.content == row.section.text)
                        }))
            })
            .map(|clip| clip.id.clone())
            .collect()
    }

    /// Synchronizes `studio.selection` with the timeline caption clips that
    /// overlap the currently checked rows in `Sections`.
    fn sync_timeline_selection(&self, studio: &mut Studio) {
        let ids = self.matching_selected_caption_ids(studio);
        if !ids.is_empty() {
            studio.selection = ids;
        } else if let Some(clip_id) = self.clip.clone() {
            studio.selection = vec![clip_id];
        }
    }

    fn can_delete_captions(&self, studio: &Studio) -> bool {
        if self.running || self.clip.is_none() {
            return false;
        }
        !self.matching_selected_caption_ids(studio).is_empty()
            || self
                .rows
                .iter()
                .any(|r| !r.keep && r.section.kind == Kind::Caption)
    }

    /// Deletes timeline caption clips overlapping the checked rows without
    /// trimming the underlying video clip (`Delete selected captions (keep video)`).
    fn delete_selected_captions(&mut self, studio: &mut Studio) {
        let Some(clip_id) = self.clip.clone() else {
            return;
        };
        let cap_ids = self.matching_selected_caption_ids(studio);
        let checked_sec_ranges: Vec<(f64, f64)> = self
            .rows
            .iter()
            .filter(|r| !r.keep && r.section.kind == Kind::Caption && r.caption_id.is_none())
            .map(|r| (r.section.start, r.section.end))
            .collect();
        if !checked_sec_ranges.is_empty() && !self.external_cues.is_empty() {
            self.external_cues.retain(|cue| {
                !checked_sec_ranges
                    .iter()
                    .any(|&(s, e)| cue.start < e - 1e-3 && cue.end > s + 1e-3)
            });
        }
        let removed_count = cap_ids.len().max(checked_sec_ranges.len());
        if !cap_ids.is_empty() {
            studio.apply(Command::RemoveClips {
                clip_ids: cap_ids,
                ripple: false,
            });
        }
        let silences = self.silences_cache.clone();
        self.populate_sections(studio, &clip_id, &silences);
        studio.selection = vec![clip_id];
        if removed_count > 0 {
            studio.notify(&format!("Deleted {removed_count} caption(s)"), false);
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

    fn can_add_captions(&self) -> bool {
        if self.running || self.clip.is_none() || !self.use_captions {
            return false;
        }
        self.rows.iter().any(|row| {
            row.keep && row.section.kind == Kind::Caption && !row.section.text.trim().is_empty()
        }) || !self.external_cues.is_empty()
    }

    /// Builds `AddTextClip` commands for the kept caption rows (or external
    /// `.srt`/`.vtt` cues) so they become real Concat timeline caption clips
    /// linked to `clip_id`.
    fn build_caption_commands(
        &self,
        studio: &Studio,
        clip_id: &str,
        replace_existing: bool,
    ) -> Vec<Command> {
        let base = studio.prefs.title_style.clone();
        let default_look = crate::panes::captions::default_caption_look();
        let has_existing_text = studio.timeline().clips.iter().any(|c| is_caption_clip(c));
        let look = if !replace_existing && has_existing_text {
            crate::panes::captions::non_overlapping_caption_look(studio, clip_id, default_look)
        } else {
            default_look
        };
        let target_track = Some("__top__".to_owned());
        let kept_rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| {
                row.keep
                    && row.section.kind == Kind::Caption
                    && !row.section.text.trim().is_empty()
                    && row.timeline_end > row.timeline_start
            })
            .collect();
        let row_commands: Vec<Command> = kept_rows
            .iter()
            .map(|row| {
                let start = row.timeline_start;
                // Caption sections are allowed to overlap. Preserve their
                // complete ranges rather than cutting this row at the next
                // caption row's start.
                let end = row.timeline_end;
                let text = row.section.text.trim().to_owned();
                let words = concat_project::captionmap::estimate_words(&text, start, end);
                crate::panes::captions::caption_clip_on_track(
                    text,
                    start,
                    (end - start).max(0.05),
                    look,
                    base.as_ref(),
                    Some(clip_id),
                    Some(words),
                    target_track.clone(),
                )
            })
            .collect();
        if !row_commands.is_empty() {
            return row_commands;
        }
        let Some(clip) = studio.clip(clip_id) else {
            return Vec::new();
        };
        let src_start = clip.source_start;
        let src_end = src_start + clip.duration * clip.speed;
        let speed = clip.speed.max(1e-6);
        let valid_ext: Vec<_> = self
            .external_cues
            .iter()
            .filter(|cue| !cue.text.trim().is_empty() && cue.end > src_start && cue.start < src_end)
            .collect();
        valid_ext
            .iter()
            .filter_map(|cue| {
                let c_start = cue.start.max(src_start);
                let c_end = cue.end.min(src_end);
                if c_end <= c_start {
                    return None;
                }
                let tl_start = clip.start + (c_start - src_start) / speed;
                // Keep an external SRT/VTT cue's full clipped range. Files
                // can contain concurrent lines, and captions now coexist on
                // the same caption track instead of forcing one to end when
                // another begins.
                let tl_end = clip.start + (c_end - src_start) / speed;
                let text = cue.text.trim().to_owned();
                let words = concat_project::captionmap::estimate_words(&text, tl_start, tl_end);
                Some(crate::panes::captions::caption_clip_on_track(
                    text,
                    tl_start,
                    (tl_end - tl_start).max(0.05),
                    look,
                    base.as_ref(),
                    Some(clip_id),
                    Some(words),
                    target_track.clone(),
                ))
            })
            .collect()
    }

    fn insert_captions_into_timeline(&mut self, studio: &mut Studio, replace_existing: bool) {
        let Some(clip_id) = self.clip.clone() else {
            return;
        };
        let caption_cmds = self.build_caption_commands(studio, &clip_id, replace_existing);
        let count = caption_cmds.len();
        if count == 0 {
            return;
        }
        let mut commands = Vec::new();
        if replace_existing {
            let existing = studio.timeline().captions_of(&clip_id);
            if !existing.is_empty() {
                commands.push(Command::RemoveClips {
                    clip_ids: existing,
                    ripple: false,
                });
            }
        }
        let top_track_busy = studio
            .timeline()
            .tracks
            .last()
            .is_none_or(|top| studio.timeline().clips.iter().any(|c| c.track_id == top.id));
        if !replace_existing || top_track_busy {
            commands.push(Command::AddTrack { bottom: false });
        }
        commands.extend(caption_cmds);
        studio.apply(Command::Batch { commands });
        self.external_cues.clear();
        let total = studio.timeline().captions_of(&clip_id).len();
        self.sub_info = format!("{total} {}", t("sections.captions"));
        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
    }

    /// The sheet opened on the selected clip with sound, else the first
    /// such clip, else an empty sheet saying there is nothing to list.
    fn open_on(&mut self, studio: &mut Studio, modal: bool) {
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
                open: modal,
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
                open: modal,
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
            open: modal,
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
            scroll_sync_seq: self.scroll_sync_seq,
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

    /// Scrolls the Sections table to the visible row corresponding to `source_index`.
    pub fn sync_scroll_to_source_row(&mut self, source_index: i32) {
        let Ok(src_idx) = usize::try_from(source_index) else {
            return;
        };
        let visible = self.visible_indices();
        if let Some(vis_pos) = visible.iter().position(|&idx| idx == src_idx) {
            self.scroll_to_row = vis_pos as i32;
            self.scroll_sync_seq = self.scroll_sync_seq.wrapping_add(1);
        }
    }

    /// Scrolls the Sections table to the visible row covering `time` (in timeline seconds),
    /// or the nearest visible row if `time` falls between visible rows.
    /// When `force` is false, only increments `scroll_sync_seq` if the target visible row changed.
    pub fn sync_scroll_to_time(&mut self, time: f64, force: bool) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        let vis_pos = visible
            .iter()
            .position(|&idx| {
                let r = &self.rows[idx];
                time >= r.timeline_start - 1e-3 && time <= r.timeline_end + 1e-3
            })
            .or_else(|| {
                visible
                    .iter()
                    .position(|&idx| self.rows[idx].timeline_end >= time)
            })
            .unwrap_or(visible.len() - 1) as i32;
        if force || vis_pos != self.scroll_to_row {
            self.scroll_to_row = vis_pos;
            self.scroll_sync_seq = self.scroll_sync_seq.wrapping_add(1);
        }
    }

    /// Run silence detection using the editable VTS-compatible controls.
    fn detect(&mut self, studio: &mut Studio) {
        let Some(clip) = self.clip.as_ref().and_then(|id| studio.clip(id)).cloned() else {
            self.open_on(studio, self.open);
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
        let selected_caption_span: Option<(f64, f64)> = if studio.selection.len() == 1 {
            studio
                .clip(&studio.selection[0])
                .filter(|c| is_caption_clip(c))
                .map(|c| (c.start, c.start + c.duration))
        } else {
            None
        };
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
            .map(|(index, row)| {
                let matches_selected_caption = selected_caption_span.is_some_and(|(s, e)| {
                    row.section.kind == Kind::Caption
                        && row.timeline_start < e - 1e-3
                        && row.timeline_end > s + 1e-3
                });
                SectionRowData {
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
                    at_playhead: (playhead >= row.timeline_start && playhead < row.timeline_end)
                        || matches_selected_caption,
                    kind_code: match row.section.kind {
                        Kind::Caption => 0,
                        Kind::Silence => 1,
                        Kind::Other => 2,
                    },
                    start_ratio: ((row.timeline_start - timeline_start) / timeline_span)
                        .clamp(0.0, 1.0) as f32,
                    end_ratio: ((row.timeline_end - timeline_start) / timeline_span).clamp(0.0, 1.0)
                        as f32,
                }
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
            can_add_captions: self.can_add_captions(),
            can_delete_captions: self.can_delete_captions(studio),
            can_apply: !self.running && !kept.is_empty() && self.rows.iter().any(|row| !row.keep),
            scroll_to_row: self.scroll_to_row,
            scroll_sync_seq: self.scroll_sync_seq,
            message: self.message.clone().into(),
            rows: slint::ModelRc::from(Rc::clone(&self.rows_model)),
        }
    }
}

/// Carves silence intervals `(s_start, s_end)` around caption spans `(c_start, c_end)`
/// so that no detected silence span ever overlaps or cuts inside an active caption cue.
fn carve_silences_around_cues(silences: &[(f64, f64)], cues: &mut [(f64, f64)]) -> Vec<(f64, f64)> {
    if cues.is_empty() || silences.is_empty() {
        return silences.to_vec();
    }
    cues.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut carved = Vec::new();
    for &(s_start, s_end) in silences {
        if s_end <= s_start {
            continue;
        }
        let mut fragments = vec![(s_start, s_end)];
        for &(c_start, c_end) in cues.iter() {
            if c_end <= c_start {
                continue;
            }
            let mut next_fragments = Vec::new();
            for (f_start, f_end) in fragments {
                if c_end <= f_start || c_start >= f_end {
                    next_fragments.push((f_start, f_end));
                } else {
                    if c_start > f_start + 0.02 {
                        next_fragments.push((f_start, c_start));
                    }
                    if c_end < f_end - 0.02 {
                        next_fragments.push((c_end, f_end));
                    }
                }
            }
            fragments = next_fragments;
            if fragments.is_empty() {
                break;
            }
        }
        carved.extend(fragments);
    }
    carved
}

/// A section kind as the row's first word.
/// Whether a text clip is a timeline caption rather than a hand-made title.
/// Imported captions carry word timing; transcribed captions additionally have
/// `caption_of`. Old untagged title clips are deliberately not inferred from
/// timing alone, because deleting a title to remove a subtitle is destructive.
fn is_caption_clip(clip: &concat_project::model::Clip) -> bool {
    clip.kind == concat_project::model::ClipKind::Text
        && (clip.caption_of.is_some() || clip.words.is_some())
}

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
                caption_id: Some("caption-1".to_owned()),
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
                caption_id: None,
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
                caption_id: None,
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
        pane.clip = Some("clip-1".to_owned());
        assert!(pane.can_add_captions());
        pane.select_none();
        pane.select_all_captions();
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![false, true, true]
        );
        assert!(!pane.can_add_captions());

        // Shift+click range selection and hold-and-drag checkbox painting across visible rows.
        pane.show_silence = true;
        pane.select_none();
        pane.toggle_row(0); // marks row 0 (keep = false)
        pane.shift_select_row(2); // marks rows 0..=2 (keep = false)
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![false, false, false]
        );
        pane.drag_begin_row(0); // unchecks row 0 (keep = true, drag_target_keep = true)
        pane.drag_over_visible(2); // paints visible rows 0..=2 back to keep = true
        assert_eq!(
            pane.rows.iter().map(|r| r.keep).collect::<Vec<_>>(),
            vec![true, true, true]
        );

        // Carve silences around active caption cues so no silence overlaps a caption.
        let silences = vec![(0.0, 5.0)];
        let mut cues = vec![(1.0, 2.0), (3.0, 4.0)];
        let carved = super::carve_silences_around_cues(&silences, &mut cues);
        assert_eq!(carved, vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)]);
    }
}
