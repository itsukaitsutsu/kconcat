// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The captions sheet: the tray's Captions tool, as a form, an interactive
//! caption editor, and a progress report.
//!
//! Matches VTS's `2 · Auto-caption` (`autocaption.txt`):
//! - Model selector + inline `Get model` downloader with progress bar
//! - 99 Whisper languages (`code — Name`) + `translate to English` with
//!   automatic fallback away from transcription-only `large-v3-turbo`
//! - Collapsible `Advanced options` (`Beam size`, `Hint / prompt`,
//!   `Output folder`, `word timestamps`, `temperature fallback`,
//!   `normalize audio`, `keep 16 kHz wav`)
//! - Manual entry buttons (`New blank captions`, `Open .srt / .vtt…`)
//! - Interactive `Caption editor` (`+ Add cue`, `Undo changes`, `Save captions`,
//!   4-mode layout re-splitter `Engine` / `Readable lines` / `Burst` / `Word`,
//!   inline cue table, `Use these captions`, and
//!   `Use these captions → Detect sections`).

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use concat_project::Command;
use concat_project::captionmap::{CaptionLayoutPreset, Cue, WordSource};
use concat_project::model::TextStyle;
use concat_project::platform_export::apply_generated_karaoke;
use concat_speech::transcribe::Segment;
use slint::{SharedString, VecModel};

use crate::host::{on_ui_in_project, spawn_in_project};
use crate::i18n::{t, tf};
use crate::panes::Msg;
use crate::panes::settings::{SettingsMsg, installed};
use crate::studio::{Studio, sync};
use crate::ui::{CaptionDraftData, CaptionsSheetData};

/// Where a caption sits, by the sheet's row: a frame-height fraction from
/// the centre, positive down. Bottom, centre, top.
const CAPTION_OFFSETS: [f64; 3] = [0.35, 0.0, -0.35];
/// A caption's cap height by the sheet's row, as a fraction of the frame.
const CAPTION_SIZES: [f64; 3] = [0.04, 0.05, 0.065];
/// The eight caption looks (C2). Index 0 on the sheet is "Default".
pub const CAPTION_LOOK_IDS: [&str; 8] = [
    "kconcat.pop-bold",
    "kconcat.boxed",
    "kconcat.karaoke",
    "kconcat.minimal",
    "kconcat.hard-shadow",
    "kconcat.upper-tight",
    "kconcat.low-third-cap",
    "kconcat.clean-bilingual",
];
/// A rough speaking rate, for a script's timing and the speech sheet's
/// estimate.
pub const CHARS_PER_SECOND: f32 = 14.0;

/// Everything that can happen to the captions sheet.
#[derive(Clone, Debug)]
pub enum CaptionsMsg {
    /// The tray's Captions tool: the selected clip's voice when it has
    /// sound, the script otherwise.
    Open,
    /// A clip's menu: the sheet on that clip's voice.
    OpenOnClip(String),
    Close,
    /// The "Use voice from timeline" switch.
    VoiceToggled(bool),
    /// Row in the list of clips with sound.
    SourceChanged(i32),
    TextEdited(String),
    ModelChanged(i32),
    DownloadModelChanged(i32),
    DownloadModelClicked,
    EngineChanged(i32),
    DeviceChanged(i32),
    ComputeTypeChanged(i32),
    VadFilterChanged(bool),
    /// A row of the language list, as the search leaves it.
    LanguageChanged(i32),
    /// The search field's text.
    LanguageQueryChanged(String),
    TranslateChanged(bool),
    BeamSizeChanged(f32),
    PromptChanged(String),
    OutputDirChanged(String),
    BrowseOutputDir,
    OutputDirPicked(PathBuf),
    WordTimestampsChanged(bool),
    TemperatureFallbackChanged(bool),
    NormalizeAudioChanged(bool),
    KeepWavChanged(bool),
    PlacementChanged(i32),
    SizeChanged(i32),
    /// Row of the eight caption looks, plus 0 = title-style default.
    LookPresetChanged(i32),
    NewBlankCaptions,
    OpenSubtitleFile,
    SubtitleFilePicked(PathBuf),
    DraftTextChanged(i32, String),
    /// Enter in a review row: split at the audio-timed word under the caret.
    DraftTextAccepted(i32, String, i32),
    DraftStartChanged(i32, f32),
    DraftEndChanged(i32, f32),
    DraftDelete(i32),
    DraftAdd,
    DraftUndo,
    DraftSave,
    DraftSaveTo(PathBuf),
    SegPresetChanged(i32),
    DraftResplit,
    DraftDiscard,
    DraftInsert,
    DraftInsertNewTrack,
    DraftInsertAndDetect,
    /// Run the pass the sheet describes.
    Begin,
    Cancel,
    /// The transcriber's worker reporting where it is, in percent.
    Progress(i32),
    /// The transcriber's worker is done: what was said, or why not.
    Finished(Result<Vec<Segment>, String>),
}

/// The clip a transcription is running over, as the finished segments
/// need it: where it starts on the timeline and how fast it plays.
#[derive(Clone, Debug)]
struct Subject {
    start: f64,
    speed: f64,
    /// The clip whose voice this is (`empty` when created as unlinked timeline captions).
    clip_id: String,
}

#[derive(Clone, Debug, PartialEq)]
struct CaptionDraft {
    text: String,
    start: f64,
    end: f64,
    words: Vec<concat_project::captionmap::Word>,
    /// True only when word boundaries came from the audio, not from text estimates.
    audio_timed: bool,
    /// A split preserves its original word times even though it is a draft edit.
    preserve_words: bool,
    edited: bool,
}

/// The captions sheet's state.
pub struct CaptionsPane {
    pub open: bool,
    /// "Use voice from timeline": the words come from [`Self::clip`]'s sound
    /// when on, from [`Self::text`] when off.
    pub from_voice: bool,
    /// The clip whose voice is transcribed: one with sound, chosen from the
    /// timeline's. Kept while the switch is off, so turning it back on
    /// returns to the same clip.
    pub clip: Option<String>,
    /// The script's words.
    pub text: String,
    /// Row in the installed transcriber list.
    pub model: usize,
    /// Row in the downloadable transcriber list (`Get model`).
    pub download_model: usize,
    /// `0` = `faster-whisper`, `1` = `whispercpp`.
    pub engine: usize,
    /// `0` = `auto`, `1` = `cuda`, `2` = `cpu`.
    pub device: usize,
    /// `0` = `auto`, `1` = `int8`, `2` = `int8_float16`, `3` = `float16`, `4` = `float32`.
    pub compute_type: usize,
    /// Skip silent spans via VAD (`cVad` in `autocaption.txt`).
    pub vad_filter: bool,
    /// Row in Whisper's language list; zero is auto-detect.
    pub language: usize,
    /// What the language search field holds; empty shows every language.
    pub language_query: String,
    pub translate: bool,
    pub translate_hint: String,
    pub beam_size: u32,
    pub prompt: String,
    pub output_dir: String,
    pub word_timestamps: bool,
    pub temperature_fallback: bool,
    pub normalize_audio: bool,
    pub keep_wav: bool,
    /// 0 bottom, 1 centre, 2 top.
    pub placement: usize,
    /// 0 small, 1 medium, 2 large.
    pub size: usize,
    /// 0 = default title style; 1..=8 = `CAPTION_LOOK_IDS`.
    pub look_preset: usize,
    pub running: bool,
    pub progress: f32,
    /// Elapsed time and ETA string shown while transcribing (`capMeta`).
    pub progress_meta: String,
    started_at: Option<std::time::Instant>,
    /// Why the last run failed, when it did.
    pub message: String,
    /// Layout preset index (`0` Engine, `1` Line, `2` Burst, `3` Word).
    pub seg_preset: usize,
    /// The clip under the running transcription or pending draft review.
    subject: Option<Subject>,
    drafts: Vec<CaptionDraft>,
    /// Original unsplit cues so `Re-split` can switch freely between presets.
    source_cues: Vec<Cue>,
    /// Undo stack for caption editor edits, additions, deletions, and re-splits.
    undo_stack: Vec<Vec<CaptionDraft>>,
    sources_model: Rc<VecModel<SharedString>>,
    download_models_model: Rc<VecModel<SharedString>>,
    devices_model: Rc<VecModel<SharedString>>,
    languages_model: Rc<VecModel<SharedString>>,
    drafts_model: Rc<VecModel<CaptionDraftData>>,
}

impl Default for CaptionsPane {
    fn default() -> Self {
        Self {
            open: false,
            from_voice: false,
            clip: None,
            text: String::new(),
            model: 0,
            download_model: 1, // default to "small" when present
            engine: 1,         // whisper.cpp (Vulkan GPU / CPU) by default
            device: 0,
            compute_type: 0,
            vad_filter: true,
            language: 0,
            language_query: String::new(),
            translate: false,
            translate_hint: String::new(),
            beam_size: 1,
            prompt: String::new(),
            output_dir: String::new(),
            word_timestamps: true,
            temperature_fallback: false,
            normalize_audio: false,
            keep_wav: false,
            placement: 0,
            size: 1,
            look_preset: 0,
            running: false,
            progress: 0.0,
            progress_meta: String::new(),
            started_at: None,
            message: String::new(),
            seg_preset: 0,
            subject: None,
            drafts: Vec::new(),
            source_cues: Vec::new(),
            undo_stack: Vec::new(),
            sources_model: Rc::new(VecModel::default()),
            download_models_model: Rc::new(VecModel::default()),
            devices_model: Rc::new(VecModel::default()),
            languages_model: Rc::new(VecModel::default()),
            drafts_model: Rc::new(VecModel::default()),
        }
    }
}

impl CaptionsPane {
    /// Applies one message. The studio is the rest of the window; while
    /// this runs the studio's copy of the pane is a blank it must not read.
    pub fn update(&mut self, msg: CaptionsMsg, studio: &mut Studio) {
        match msg {
            CaptionsMsg::Open => {
                let selected = studio.sole_selection();
                self.open_on(studio, selected, false);
            }
            CaptionsMsg::OpenOnClip(id) => self.open_on(studio, Some(id), true),
            CaptionsMsg::Close => self.open = false,
            CaptionsMsg::VoiceToggled(on) => {
                self.from_voice = on;
                self.message.clear();
                let sources = sources(studio);
                if on
                    && !self
                        .clip
                        .as_ref()
                        .is_some_and(|id| sources.iter().any(|s| &s.id == id))
                {
                    self.clip = sources.first().map(|source| source.id.clone());
                }
            }
            CaptionsMsg::SourceChanged(index) => {
                if let Some(source) = usize::try_from(index)
                    .ok()
                    .and_then(|index| sources(studio).into_iter().nth(index))
                {
                    self.clip = Some(source.id);
                    self.message.clear();
                }
            }
            CaptionsMsg::TextEdited(text) => self.text = text,
            CaptionsMsg::ModelChanged(index) => {
                self.model = index.max(0) as usize;
                self.refresh_translate_hint(studio);
            }
            CaptionsMsg::DownloadModelChanged(index) => {
                self.download_model = index.max(0) as usize;
            }
            CaptionsMsg::DownloadModelClicked => {
                let managed: Vec<String> = studio
                    .settings
                    .transcribers
                    .iter()
                    .filter(|m| m.managed)
                    .map(|m| m.id.clone())
                    .collect();
                if let Some(id) = managed.get(self.download_model).cloned() {
                    studio.handle(Msg::Settings(SettingsMsg::ModelDownload(id)));
                }
            }
            CaptionsMsg::EngineChanged(index) => {
                self.engine = (index.max(0) as usize).min(1);
            }
            CaptionsMsg::DeviceChanged(index) => {
                let count = concat_speech::transcribe::list_compute_devices(&studio.host.dirs)
                    .len()
                    .max(1);
                self.device = (index.max(0) as usize).min(count - 1);
            }
            CaptionsMsg::ComputeTypeChanged(index) => {
                self.compute_type = (index.max(0) as usize).min(4);
            }
            CaptionsMsg::VadFilterChanged(on) => {
                self.vad_filter = on;
            }
            CaptionsMsg::LanguageChanged(index) => {
                // `index` is a row of the list as the search left it.
                let rows = matching_languages(language_labels(), &self.language_query);
                if let Some(&row) = usize::try_from(index).ok().and_then(|i| rows.get(i)) {
                    self.language = row;
                }
            }
            CaptionsMsg::LanguageQueryChanged(query) => {
                self.language_query = query;
            }
            CaptionsMsg::TranslateChanged(on) => {
                self.translate = on;
                self.apply_translate_model_guard(studio);
            }
            CaptionsMsg::BeamSizeChanged(value) => {
                if value.is_finite() {
                    self.beam_size = (value.round() as i32).clamp(1, 10) as u32;
                }
            }
            CaptionsMsg::PromptChanged(text) => self.prompt = text,
            CaptionsMsg::OutputDirChanged(text) => self.output_dir = text,
            CaptionsMsg::BrowseOutputDir => {
                let title = t("captions.outputFolder");
                let initial = self.output_dir.clone();
                crate::host::after_dialog(
                    move || crate::platform::pick_folder(&title, &initial),
                    |studio, _, _, folder| {
                        studio.handle(Msg::Captions(CaptionsMsg::OutputDirPicked(folder)));
                    },
                );
            }
            CaptionsMsg::OutputDirPicked(folder) => {
                self.output_dir = folder.to_string_lossy().into_owned();
            }
            CaptionsMsg::WordTimestampsChanged(on) => self.word_timestamps = on,
            CaptionsMsg::TemperatureFallbackChanged(on) => self.temperature_fallback = on,
            CaptionsMsg::NormalizeAudioChanged(on) => self.normalize_audio = on,
            CaptionsMsg::KeepWavChanged(on) => self.keep_wav = on,
            CaptionsMsg::PlacementChanged(index) => {
                self.placement = (index.max(0) as usize).min(2);
            }
            CaptionsMsg::SizeChanged(index) => self.size = (index.max(0) as usize).min(2),
            CaptionsMsg::LookPresetChanged(index) => {
                self.look_preset = (index.max(0) as usize).min(CAPTION_LOOK_IDS.len());
            }
            CaptionsMsg::NewBlankCaptions => {
                self.start_blank_captions(studio);
            }
            CaptionsMsg::OpenSubtitleFile => {
                let title = t("captions.openSubtitleFile");
                crate::host::after_dialog(
                    move || {
                        crate::platform::pick_files(&title, Some(("Subtitles", &["srt", "vtt"])))
                            .and_then(|mut paths| paths.drain(..).next())
                    },
                    |studio, _, _, path| {
                        studio.handle(Msg::Captions(CaptionsMsg::SubtitleFilePicked(path)));
                    },
                );
            }
            CaptionsMsg::SubtitleFilePicked(path) => {
                self.load_subtitle_file(studio, &path);
            }
            CaptionsMsg::DraftTextChanged(index, text) => self.edit_draft_text(index, text),
            CaptionsMsg::DraftTextAccepted(index, text, caret) => {
                self.accept_draft_text(index, text, caret);
            }
            CaptionsMsg::DraftStartChanged(index, value) => self.edit_draft_start(index, value),
            CaptionsMsg::DraftEndChanged(index, value) => self.edit_draft_end(index, value),
            CaptionsMsg::DraftDelete(index) => self.delete_draft(index),
            CaptionsMsg::DraftAdd => self.add_draft(),
            CaptionsMsg::DraftUndo => self.undo_draft(),
            CaptionsMsg::DraftSave => self.save_drafts(studio),
            CaptionsMsg::DraftSaveTo(path) => self.write_drafts_to_path(studio, &path),
            CaptionsMsg::SegPresetChanged(index) => {
                self.seg_preset = (index.max(0) as usize).min(3);
            }
            CaptionsMsg::DraftResplit => self.resplit_drafts(),
            CaptionsMsg::DraftDiscard => self.discard_drafts(),
            CaptionsMsg::DraftInsert => {
                let _ = self.insert_drafts_inner(studio, false);
            }
            CaptionsMsg::DraftInsertNewTrack => {
                let _ = self.insert_drafts_inner(studio, false);
            }
            CaptionsMsg::DraftInsertAndDetect => {
                if let Some(clip_id) = self.insert_drafts_inner(studio, false) {
                    if !clip_id.is_empty() {
                        studio.selection = vec![clip_id];
                    }
                    studio.open_sections_panel_or_sheet();
                }
            }
            CaptionsMsg::Begin => {
                self.drafts.clear();
                self.source_cues.clear();
                self.undo_stack.clear();
                if self.from_voice {
                    self.run_sound(studio);
                } else {
                    self.run_script(studio);
                }
            }
            CaptionsMsg::Cancel => {
                studio.host.transcriber.cancel();
                self.running = false;
                self.started_at = None;
                self.progress_meta.clear();
                self.subject = None;
                self.open = false;
            }
            CaptionsMsg::Progress(percent) => {
                self.progress = (percent as f32 / 100.0).clamp(0.0, 1.0);
                if let Some(started) = self.started_at {
                    let elapsed = started.elapsed().as_secs_f64();
                    if (2..100).contains(&percent) {
                        let frac = (f64::from(percent) / 100.0).clamp(0.01, 0.99);
                        let rem = (elapsed * (1.0 - frac) / frac).max(0.0);
                        self.progress_meta = format!(
                            "Elapsed {} · ETA ~{} remaining",
                            eta_clock(elapsed),
                            eta_clock(rem)
                        );
                    } else {
                        self.progress_meta =
                            format!("Elapsed {} · estimating ETA…", eta_clock(elapsed));
                    }
                }
            }
            CaptionsMsg::Finished(result) => {
                self.running = false;
                self.started_at = None;
                self.progress_meta.clear();
                match result {
                    Ok(segments) => {
                        if self.subject.is_none() {
                            return;
                        }
                        if !self.populate_drafts(segments) {
                            self.subject = None;
                            studio.notify(&t("captions.nothingSaidClip"), true);
                        }
                    }
                    // Asked for: the sheet is already on its way down.
                    Err(error) if error.contains("cancel") => {
                        self.subject = None;
                        self.open = false;
                    }
                    Err(error) => {
                        self.subject = None;
                        self.message = error;
                    }
                }
            }
        }
    }

    fn apply_translate_model_guard(&mut self, studio: &Studio) {
        let on_disk = installed(&studio.settings.transcribers);
        if self.translate
            && let Some(current) = on_disk.get(self.model)
            && current.id == "large-v3-turbo"
        {
            let preferred = ["medium", "large-v3", "small", "base", "tiny"];
            if let Some(replacement_idx) = preferred
                .iter()
                .find_map(|cand| on_disk.iter().position(|m| m.id == *cand))
            {
                let replacement_name = on_disk[replacement_idx].name.clone();
                self.model = replacement_idx;
                self.translate_hint = format!(
                    "Note: large-v3-turbo is transcription-only; switched to {replacement_name} for English translation."
                );
                return;
            }
            self.translate_hint =
                "large-v3-turbo cannot translate to English. Download medium, small, or large-v3 above."
                    .to_owned();
            return;
        }
        self.refresh_translate_hint(studio);
    }

    fn refresh_translate_hint(&mut self, studio: &Studio) {
        let on_disk = installed(&studio.settings.transcribers);
        if self.translate {
            if let Some(current) = on_disk.get(self.model) {
                if current.id == "large-v3-turbo" {
                    self.translate_hint =
                        "large-v3-turbo cannot translate to English. Choose medium, small, or large-v3."
                            .to_owned();
                } else {
                    self.translate_hint = format!(
                        "Translating non-English speech into English captions ({}).",
                        current.name
                    );
                }
            } else {
                self.translate_hint = "Outputs English captions from non-English audio.".to_owned();
            }
        } else {
            self.translate_hint.clear();
        }
    }

    fn push_undo(&mut self) {
        if self.undo_stack.last() != Some(&self.drafts) {
            if self.undo_stack.len() >= 50 {
                self.undo_stack.remove(0);
            }
            self.undo_stack.push(self.drafts.clone());
        }
    }

    fn edit_draft_text(&mut self, index: i32, text: String) {
        let Ok(idx) = usize::try_from(index) else {
            return;
        };
        if idx >= self.drafts.len() {
            return;
        }
        if !self.drafts[idx].edited {
            self.push_undo();
        }
        if let Some(draft) = self.drafts.get_mut(idx) {
            draft.text = text;
            draft.preserve_words = false;
            draft.edited = true;
            self.message.clear();
        }
    }

    fn accept_draft_text(&mut self, row: i32, text: String, caret: i32) {
        let Ok(index) = usize::try_from(row) else {
            return;
        };
        if index >= self.drafts.len() {
            return;
        }

        // The TextInput can emit `accepted` before its last `edited` callback
        // reaches the model. Save the pre-Enter state once so a successful
        // split is one undo even when this row was already edited earlier.
        let text_changed = self.drafts[index].text != text;
        if text_changed {
            self.push_undo();
            if let Some(draft) = self.drafts.get_mut(index) {
                draft.text = text;
                draft.preserve_words = false;
                draft.edited = true;
            }
        }
        let Ok(caret) = usize::try_from(caret) else {
            self.message = "Enter needs a caret inside a word with audio timing.".to_owned();
            return;
        };
        let Some(draft) = self.drafts.get(index).cloned() else {
            return;
        };
        match split_draft_at_caret(&draft, caret) {
            Ok((head, tail)) => {
                // One undo returns to the exact row state visible before Enter.
                if !text_changed {
                    self.push_undo();
                }
                self.drafts.remove(index);
                self.drafts.insert(index, head);
                self.drafts.insert(index + 1, tail);
                self.message.clear();
            }
            Err(reason) => self.message = reason.to_owned(),
        }
    }

    fn edit_draft_start(&mut self, index: i32, value: f32) {
        if !value.is_finite() {
            return;
        }
        let Ok(idx) = usize::try_from(index) else {
            return;
        };
        if idx >= self.drafts.len() {
            return;
        }
        self.push_undo();
        if let Some(draft) = self.drafts.get_mut(idx) {
            draft.start = f64::from(value).max(0.0).min(draft.end - 0.05);
            draft.edited = true;
        }
    }

    fn edit_draft_end(&mut self, index: i32, value: f32) {
        if !value.is_finite() {
            return;
        }
        let Ok(idx) = usize::try_from(index) else {
            return;
        };
        if idx >= self.drafts.len() {
            return;
        }
        self.push_undo();
        if let Some(draft) = self.drafts.get_mut(idx) {
            draft.end = f64::from(value).max(draft.start + 0.05);
            draft.edited = true;
        }
    }

    fn delete_draft(&mut self, index: i32) {
        if let Ok(index) = usize::try_from(index)
            && index < self.drafts.len()
        {
            self.push_undo();
            self.drafts.remove(index);
        }
    }

    fn add_draft(&mut self) {
        self.push_undo();
        let start = self.drafts.last().map_or_else(
            || self.subject.as_ref().map_or(0.0, |s| s.start),
            |cue| cue.end,
        );
        self.drafts.push(CaptionDraft {
            text: String::new(),
            start,
            end: start + 2.0,
            words: Vec::new(),
            audio_timed: false,
            preserve_words: false,
            edited: true,
        });
        self.message.clear();
    }

    fn undo_draft(&mut self) {
        if let Some(previous) = self.undo_stack.pop() {
            self.drafts = previous;
            self.message.clear();
        }
    }

    fn resplit_drafts(&mut self) {
        if self.drafts.is_empty() && self.source_cues.is_empty() {
            return;
        }
        self.push_undo();
        let preset = CaptionLayoutPreset::from_index(self.seg_preset);
        let base_cues: Vec<Cue> =
            if !self.source_cues.is_empty() && !self.drafts.iter().any(|d| d.edited) {
                self.source_cues.clone()
            } else {
                self.drafts
                    .iter()
                    .map(|d| Cue {
                        start: d.start,
                        end: d.end,
                        text: d.text.clone(),
                        words: if d.edited && !d.preserve_words {
                            concat_project::captionmap::estimate_words(&d.text, d.start, d.end)
                        } else {
                            d.words.clone()
                        },
                        word_source: Some(if d.audio_timed && (!d.edited || d.preserve_words) {
                            WordSource::Whisper
                        } else {
                            WordSource::Estimated
                        }),
                        ..Cue::default()
                    })
                    .collect()
            };
        let split = concat_project::captionmap::resplit_cues(&base_cues, preset);
        self.drafts = split
            .into_iter()
            .map(|cue| CaptionDraft {
                text: cue.text,
                start: cue.start,
                end: cue.end,
                audio_timed: matches!(cue.word_source, Some(WordSource::Whisper | WordSource::Whispercpp)),
                words: cue.words,
                preserve_words: false,
                edited: false,
            })
            .collect();
        self.message.clear();
    }

    fn ensure_editor_subject(&mut self, studio: &Studio) {
        if self.subject.is_some() {
            return;
        }
        if self.clip.is_none() {
            self.clip = sources(studio).first().map(|s| s.id.clone());
        }
        let clip_opt = self.clip.as_ref().and_then(|id| studio.clip(id)).cloned();
        if let Some(clip) = clip_opt {
            self.subject = Some(Subject {
                start: clip.start,
                speed: clip.speed,
                clip_id: clip.id,
            });
        } else {
            self.subject = Some(Subject {
                start: f64::from(studio.playhead).max(0.0),
                speed: 1.0,
                clip_id: String::new(),
            });
        }
    }

    fn start_blank_captions(&mut self, studio: &Studio) {
        self.ensure_editor_subject(studio);
        let start = self.subject.as_ref().map_or(0.0, |s| s.start);
        self.undo_stack.clear();
        self.source_cues.clear();
        self.drafts = vec![CaptionDraft {
            text: String::new(),
            start,
            end: start + 2.0,
            words: Vec::new(),
            audio_timed: false,
            preserve_words: false,
            edited: true,
        }];
        self.message.clear();
    }

    fn load_subtitle_file(&mut self, studio: &Studio, path: &std::path::Path) {
        let raw = match std::fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(err) => {
                self.message = err.to_string();
                return;
            }
        };
        let parsed = concat_project::subtitle::parse(&raw);
        if parsed.is_empty() {
            self.message = t("captionImport.noCues");
            return;
        }
        self.ensure_editor_subject(studio);
        let offset = self.subject.as_ref().map_or(0.0, |s| s.start);
        let cues: Vec<Cue> = parsed
            .into_iter()
            .map(|cue| {
                let start = offset + cue.start;
                let end = offset + cue.end.max(cue.start + 0.2);
                let words = concat_project::captionmap::estimate_words(&cue.text, start, end);
                Cue {
                    start,
                    end,
                    text: cue.text,
                    words,
                    word_source: Some(WordSource::Estimated),
                    ..Cue::default()
                }
            })
            .collect();
        self.undo_stack.clear();
        self.source_cues = cues.clone();
        self.drafts = cues
            .into_iter()
            .map(|cue| CaptionDraft {
                text: cue.text,
                start: cue.start,
                end: cue.end,
                audio_timed: matches!(cue.word_source, Some(WordSource::Whisper | WordSource::Whispercpp)),
                words: cue.words,
                preserve_words: false,
                edited: false,
            })
            .collect();
        self.message.clear();
    }

    fn save_drafts(&mut self, studio: &mut Studio) {
        if self.drafts.is_empty() {
            return;
        }
        let out_dir = self.output_dir.trim();
        if !out_dir.is_empty() {
            let path = PathBuf::from(out_dir).join("captions.srt");
            self.write_drafts_to_path(studio, &path);
            return;
        }
        let title = t("captions.saveCaptions");
        let default_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        crate::host::after_dialog(
            move || {
                crate::platform::save_file(
                    &title,
                    &default_dir,
                    "captions.srt",
                    ("Subtitles", &["srt", "vtt"]),
                )
            },
            |studio, _, _, path| {
                studio.handle(Msg::Captions(CaptionsMsg::DraftSaveTo(path)));
            },
        );
    }

    fn write_drafts_to_path(&mut self, studio: &mut Studio, path: &std::path::Path) {
        let cues: Vec<concat_project::subtitle::Cue> = self
            .drafts
            .iter()
            .filter(|d| !d.text.trim().is_empty() && d.end > d.start)
            .map(|d| concat_project::subtitle::Cue {
                start: d.start,
                end: d.end,
                text: d.text.trim().to_owned(),
            })
            .collect();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && let Err(err) = std::fs::create_dir_all(parent)
        {
            self.message = err.to_string();
            return;
        }
        let is_vtt = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("vtt"));
        let primary_content = if is_vtt {
            concat_project::subtitle::to_vtt(&cues)
        } else {
            concat_project::subtitle::to_srt(&cues)
        };
        if let Err(err) = std::fs::write(path, primary_content) {
            self.message = err.to_string();
            return;
        }
        // Also write the companion .vtt / .srt alongside it (matching VTS save_captions).
        let companion = if is_vtt {
            path.with_extension("srt")
        } else {
            path.with_extension("vtt")
        };
        let companion_content = if is_vtt {
            concat_project::subtitle::to_srt(&cues)
        } else {
            concat_project::subtitle::to_vtt(&cues)
        };
        let _ = std::fs::write(&companion, companion_content);
        studio.notify(&format!("Saved {}", path.display()), false);
    }

    fn discard_drafts(&mut self) {
        self.drafts.clear();
        self.source_cues.clear();
        self.undo_stack.clear();
        self.subject = None;
        self.message.clear();
    }

    /// Populates `self.drafts` from transcribed segments using `self.subject`.
    /// Returns `true` when at least one non-empty draft cue was created.
    fn populate_drafts(&mut self, segments: Vec<Segment>) -> bool {
        let Some(subject) = self.subject.as_ref() else {
            return false;
        };
        let cues: Vec<Cue> = segments
            .into_iter()
            .filter_map(|segment| {
                let text = segment.text.trim().to_owned();
                if text.is_empty() {
                    return None;
                }
                let start = subject.start + segment.start / subject.speed;
                let end = start + ((segment.end - segment.start) / subject.speed).max(0.2);
                let heard: Vec<concat_project::captionmap::Word> = segment
                    .words
                    .iter()
                    .map(|word| concat_project::captionmap::Word {
                        text: word.text.clone(),
                        start: word.start,
                        end: word.end,
                    })
                    .collect();
                let words = concat_project::captionmap::caption_words(
                    &text,
                    &heard,
                    |at| subject.start + at / subject.speed,
                    start,
                    end,
                );
                let word_source = Some(if concat_project::captionmap::clean_words(
                    &heard,
                    Some(start),
                    Some(end),
                ).0.is_empty() {
                    WordSource::Estimated
                } else if self.engine == 0 {
                    WordSource::Whisper
                } else {
                    WordSource::Whispercpp
                });
                Some(Cue {
                    start,
                    end,
                    text,
                    words,
                    word_source,
                    ..Cue::default()
                })
            })
            .collect();
        self.undo_stack.clear();
        self.source_cues = cues.clone();
        let preset = CaptionLayoutPreset::from_index(self.seg_preset);
        let split = concat_project::captionmap::resplit_cues(&cues, preset);
        self.drafts = split
            .into_iter()
            .map(|cue| CaptionDraft {
                text: cue.text,
                start: cue.start,
                end: cue.end,
                audio_timed: matches!(cue.word_source, Some(WordSource::Whisper | WordSource::Whispercpp)),
                words: cue.words,
                preserve_words: false,
                edited: false,
            })
            .collect();
        !self.drafts.is_empty()
    }

    /// The sheet opened afresh. `clip` is the clip it was opened on, if
    /// any: the switch starts on when that clip has sound, or when a clip's
    /// menu asked for its voice (`voice`), in which case a clip without
    /// sound falls to the first clip that has some.
    fn open_on(&mut self, studio: &Studio, clip: Option<String>, voice: bool) {
        let installed = installed(&studio.settings.transcribers);
        let model = installed.iter().position(|model| model.active).unwrap_or(0);
        let sources = sources(studio);
        let sounding = clip.filter(|id| sources.iter().any(|source| &source.id == id));
        let from_voice = voice || sounding.is_some();
        let clip = sounding.or_else(|| {
            from_voice
                .then(|| sources.first().map(|source| source.id.clone()))
                .flatten()
        });
        *self = CaptionsPane {
            open: true,
            from_voice,
            clip,
            model,
            download_model: self.download_model,
            engine: self.engine,
            device: self.device,
            compute_type: self.compute_type,
            vad_filter: self.vad_filter,
            language: self.language,
            translate: self.translate,
            translate_hint: self.translate_hint.clone(),
            beam_size: self.beam_size,
            prompt: self.prompt.clone(),
            output_dir: self.output_dir.clone(),
            word_timestamps: self.word_timestamps,
            temperature_fallback: self.temperature_fallback,
            normalize_audio: self.normalize_audio,
            keep_wav: self.keep_wav,
            placement: 0,
            size: 1,
            look_preset: self.look_preset,
            seg_preset: self.seg_preset,
            sources_model: Rc::clone(&self.sources_model),
            download_models_model: Rc::clone(&self.download_models_model),
            devices_model: Rc::clone(&self.devices_model),
            languages_model: Rc::clone(&self.languages_model),
            drafts_model: Rc::clone(&self.drafts_model),
            ..CaptionsPane::default()
        };
        self.refresh_translate_hint(studio);
    }

    /// A caption's look, by the sheet's rows: where it sits and its size.
    fn look(&self) -> (f64, f64) {
        (
            CAPTION_OFFSETS[self.placement.min(2)],
            CAPTION_SIZES[self.size.min(2)],
        )
    }

    /// Title prefs, or a tagged caption preset when the sheet picked one.
    /// The bool is "the look already decided karaoke on/off".
    fn caption_style_base(&self, studio: &Studio) -> (Option<TextStyle>, bool) {
        if self.look_preset == 0 {
            return (studio.prefs.title_style.clone(), false);
        }
        let Some(id) = CAPTION_LOOK_IDS.get(self.look_preset - 1) else {
            return (studio.prefs.title_style.clone(), false);
        };
        match studio.text_presets.iter().find(|held| held.id == *id) {
            Some(found) => (Some(found.style.clone()), true),
            None => (studio.prefs.title_style.clone(), false),
        }
    }

    /// The script as titles, one after another from the playhead.
    fn run_script(&mut self, studio: &mut Studio) {
        let lines = script_captions(&self.text);
        if lines.is_empty() {
            self.message = t("captions.nothingToCaptionYet");
            return;
        }
        let has_existing = studio
            .timeline()
            .clips
            .iter()
            .any(|c| c.kind == concat_project::model::ClipKind::Text);
        let look = if has_existing {
            non_overlapping_caption_look(studio, "", self.look())
        } else {
            self.look()
        };
        let target_track = has_existing.then(|| "__top__".to_owned());
        let mut at = f64::from(studio.playhead);
        let (base, look_decided) = self.caption_style_base(studio);
        let mut commands: Vec<Command> = Vec::new();
        if has_existing {
            commands.push(Command::AddTrack { bottom: false });
        }
        let mut count = 0usize;
        for (text, seconds) in lines {
            let command = caption_clip_on_track(
                text,
                at,
                seconds,
                look,
                base.as_ref(),
                None,
                None,
                target_track.clone(),
                look_decided,
            );
            at += seconds;
            commands.push(command);
            count += 1;
        }
        self.open = false;
        studio.apply(Command::Batch { commands });
        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
    }

    /// The sheet's clip through the transcriber on a worker, reporting into
    /// the sheet as it goes.
    fn run_sound(&mut self, studio: &mut Studio) {
        let Some(clip) = self.clip.as_ref().and_then(|id| studio.clip(id)).cloned() else {
            self.message = t("captions.clipNoLongerTimeline");
            return;
        };
        let Some(media) = studio.project().media_by_id(&clip.media_id).cloned() else {
            self.message = t("captions.clipHasNoFile");
            return;
        };
        let Some(model) = installed(&studio.settings.transcribers)
            .get(self.model)
            .map(|model| model.id.clone())
        else {
            self.message = t("captions.downloadTranscriberModelFirst");
            return;
        };
        let out_dir =
            (!self.output_dir.trim().is_empty()).then(|| self.output_dir.trim().to_owned());
        let engine_str = if self.engine == 0 {
            "faster-whisper"
        } else {
            "whispercpp"
        };
        let device_list = concat_speech::transcribe::list_compute_devices(&studio.host.dirs);
        let device_str = device_list
            .get(self.device)
            .map(|(id, _)| id.as_str())
            .unwrap_or("auto");
        let compute_str = match self.compute_type {
            1 => "int8",
            2 => "int8_float16",
            3 => "float16",
            4 => "float32",
            _ => "auto",
        };
        let request = concat_speech::transcribe::TranscribeRequest {
            path: media.path.clone(),
            audio_stream: clip.audio_stream,
            source_start: clip.source_start,
            window: clip.duration * clip.speed,
            model_id: model,
            language: language_codes()[self.language.min(language_codes().len() - 1)].to_owned(),
            translate: self.translate,
            prompt: self.prompt.clone(),
            engine: engine_str.to_owned(),
            device: device_str.to_owned(),
            compute_type: compute_str.to_owned(),
            vad_filter: self.vad_filter,
            beam_size: self.beam_size,
            word_timestamps: self.word_timestamps,
            temperature_fallback: self.temperature_fallback,
            normalize_audio: self.normalize_audio,
            keep_wav: self.keep_wav,
            output_dir: out_dir,
        };
        let dirs = studio.host.dirs.clone();
        let transcriber = Arc::clone(&studio.host.transcriber);
        self.subject = Some(Subject {
            start: clip.start,
            speed: clip.speed,
            clip_id: clip.id.clone(),
        });
        self.running = true;
        self.progress = 0.0;
        self.started_at = Some(std::time::Instant::now());
        self.progress_meta = "Elapsed 0:00 · estimating ETA…".to_owned();
        self.message.clear();
        let epoch = crate::host::project_epoch();
        spawn_in_project(
            move || {
                transcriber.transcribe(&dirs, &request, move |percent| {
                    on_ui_in_project(epoch, move |studio, _, _| {
                        studio.handle(Msg::Captions(CaptionsMsg::Progress(percent)));
                    });
                })
            },
            |studio, _, _, result| studio.handle(Msg::Captions(CaptionsMsg::Finished(result))),
        );
    }

    /// Inserts draft cues onto the timeline and returns the linked `clip_id` (if any).
    /// When `replace_existing` is `false`, keeps all existing caption tracks and
    /// places the new captions on a fresh track above (`+ Add as new caption track`).
    fn insert_drafts_inner(
        &mut self,
        studio: &mut Studio,
        replace_existing: bool,
    ) -> Option<String> {
        let subject = self.subject.as_ref()?.clone();
        let clip_id = if !subject.clip_id.is_empty() {
            subject.clip_id.clone()
        } else {
            self.clip.clone().unwrap_or_default()
        };
        let has_existing_text = studio
            .timeline()
            .clips
            .iter()
            .any(|c| c.kind == concat_project::model::ClipKind::Text);
        let look = if !replace_existing && has_existing_text {
            non_overlapping_caption_look(studio, &clip_id, self.look())
        } else {
            self.look()
        };
        // Always place the caption batch onto the dedicated top track (`__top__`).
        let target_track = Some("__top__".to_owned());
        let (base, look_decided) = self.caption_style_base(studio);
        let caption_of = (!clip_id.is_empty()).then_some(clip_id.as_str());
        let valid_drafts: Vec<&CaptionDraft> = self
            .drafts
            .iter()
            .filter(|cue| !cue.text.trim().is_empty() && cue.end > cue.start)
            .collect();
        let caption_commands: Vec<Command> = valid_drafts
            .iter()
            .map(|cue| {
                // Preserve concurrent draft cues rather than cutting one at
                // the next cue's start. They can occupy the same caption
                // track and remain separately editable on the timeline.
                let duration = (cue.end - cue.start).max(0.05);
                let words = if cue.edited && !cue.preserve_words {
                    concat_project::captionmap::estimate_words(&cue.text, cue.start, cue.end)
                } else {
                    cue.words.clone()
                };
                caption_clip_on_track(
                    cue.text.trim().to_owned(),
                    cue.start,
                    duration,
                    look,
                    base.as_ref(),
                    caption_of,
                    Some(words),
                    target_track.clone(),
                    look_decided,
                )
            })
            .collect();
        let count = caption_commands.len();
        if count == 0 {
            self.message = t("captions.nothingToCaptionYet");
            return None;
        }
        let mut commands = Vec::new();
        if replace_existing && !clip_id.is_empty() {
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
        commands.extend(caption_commands);
        self.drafts.clear();
        self.source_cues.clear();
        self.undo_stack.clear();
        self.subject = None;
        self.open = false;
        studio.apply(Command::Batch { commands });
        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
        Some(clip_id)
    }

    /// The sheet as Slint shows it.
    pub fn data(&self, studio: &Studio) -> CaptionsSheetData {
        let sources = if self.open {
            sources(studio)
        } else {
            Vec::new()
        };
        let source = self
            .clip
            .as_ref()
            .and_then(|id| sources.iter().position(|source| &source.id == id))
            .map_or(-1, |index| index as i32);
        let labels: Vec<SharedString> = sources
            .into_iter()
            .map(|source| source.label.into())
            .collect();
        let managed_models: Vec<_> = studio
            .settings
            .transcribers
            .iter()
            .filter(|m| m.managed)
            .collect();
        let download_models: Vec<SharedString> = managed_models
            .iter()
            .map(|m| {
                if m.installed {
                    format!("{} — {:.0} MB (installed)", m.id, m.megabytes).into()
                } else {
                    format!("{} — {:.0} MB", m.id, m.megabytes).into()
                }
            })
            .collect();
        let active_dl = studio
            .settings
            .transcribers
            .iter()
            .find(|m| m.fetched.is_some());
        let downloading_model = active_dl.is_some();
        let download_progress = active_dl
            .map(|m| {
                let fetched = m.fetched.unwrap_or(0.0);
                if m.megabytes > 0.0 {
                    (fetched / m.megabytes).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        let on_disk = installed(&studio.settings.transcribers);
        let engine_str = if self.engine == 0 {
            "faster-whisper"
        } else {
            "whispercpp"
        };
        let engine_note = if self.open {
            concat_speech::transcribe::engine_status_note(&studio.host.dirs, engine_str)
        } else {
            String::new()
        };
        let download_status = if let Some(dl) = active_dl {
            format!(
                "Downloading {} ({:.0} / {:.0} MB)…",
                dl.id,
                dl.fetched.unwrap_or(0.0),
                dl.megabytes
            )
        } else if on_disk.is_empty() {
            t("captions.downloadTranscriberModelFirst")
        } else if !engine_note.is_empty() {
            format!("{} model(s) · {engine_note}", on_disk.len())
        } else {
            format!(
                "{} model(s) installed · {}",
                on_disk.len(),
                studio.settings.whisper_models_dir
            )
        };
        let devices: Vec<SharedString> = if self.open {
            concat_speech::transcribe::list_compute_devices(&studio.host.dirs)
                .into_iter()
                .map(|(_, label)| SharedString::from(label))
                .collect()
        } else {
            vec![SharedString::from("Auto — Default Vulkan GPU")]
        };
        let language_names = language_labels();
        let rows = matching_languages(language_names, &self.language_query);
        let languages: Vec<SharedString> = rows
            .iter()
            .map(|&row| SharedString::from(language_names[row]))
            .collect();
        // The chosen language is hidden when the search leaves it out; the
        // trigger still names it, from `language_text`.
        let language = rows
            .iter()
            .position(|&row| row == self.language)
            .map_or(-1, |position| position as i32);
        let language_text = language_names[self.language.min(language_names.len() - 1)];
        let drafts: Vec<CaptionDraftData> = self
            .drafts
            .iter()
            .map(|cue| CaptionDraftData {
                text: cue.text.as_str().into(),
                start: cue.start as f32,
                end: cue.end as f32,
            })
            .collect();
        sync(&self.sources_model, labels);
        sync(&self.download_models_model, download_models);
        sync(&self.devices_model, devices);
        sync(&self.languages_model, languages);
        sync(&self.drafts_model, drafts);
        CaptionsSheetData {
            open: self.open,
            from_voice: self.from_voice,
            sources: slint::ModelRc::from(Rc::clone(&self.sources_model)),
            source,
            text: self.text.as_str().into(),
            model: self.model as i32,
            download_models: slint::ModelRc::from(Rc::clone(&self.download_models_model)),
            download_model: self
                .download_model
                .min(managed_models.len().saturating_sub(1)) as i32,
            downloading_model,
            download_progress,
            download_status: download_status.into(),
            engine: self.engine as i32,
            devices: slint::ModelRc::from(Rc::clone(&self.devices_model)),
            device: self.device as i32,
            compute_type: self.compute_type as i32,
            vad_filter: self.vad_filter,
            languages: slint::ModelRc::from(Rc::clone(&self.languages_model)),
            language,
            language_text: language_text.into(),
            translate: self.translate,
            translate_hint: self.translate_hint.as_str().into(),
            beam_size: self.beam_size as f32,
            prompt: self.prompt.as_str().into(),
            output_dir: self.output_dir.as_str().into(),
            word_timestamps: self.word_timestamps,
            temperature_fallback: self.temperature_fallback,
            normalize_audio: self.normalize_audio,
            keep_wav: self.keep_wav,
            placement: self.placement as i32,
            size: self.size as i32,
            look_preset: self.look_preset as i32,
            running: self.running,
            progress: self.progress,
            progress_meta: self.progress_meta.as_str().into(),
            ready: !on_disk.is_empty(),
            reviewing: !self.running && self.subject.is_some(),
            can_undo: !self.undo_stack.is_empty(),
            seg_preset: self.seg_preset as i32,
            drafts: slint::ModelRc::from(Rc::clone(&self.drafts_model)),
            message: self.message.as_str().into(),
        }
    }
}

/// Languages understood by multilingual Whisper models (BCP-47 / Whisper code).
fn language_codes() -> &'static [&'static str] {
    &[
        "auto", "af", "am", "ar", "as", "az", "ba", "be", "bg", "bn", "bo", "br", "bs", "ca", "cs",
        "cy", "da", "de", "el", "en", "es", "et", "eu", "fa", "fi", "fo", "fr", "gl", "gu", "ha",
        "haw", "he", "hi", "hr", "ht", "hu", "hy", "id", "is", "it", "ja", "jw", "ka", "kk", "km",
        "kn", "ko", "la", "lb", "ln", "lo", "lt", "lv", "mg", "mi", "mk", "ml", "mn", "mr", "ms",
        "mt", "my", "ne", "nl", "nn", "no", "oc", "pa", "pl", "ps", "pt", "ro", "ru", "sa", "sd",
        "si", "sk", "sl", "sn", "so", "sq", "sr", "su", "sv", "sw", "ta", "te", "tg", "th", "tk",
        "tl", "tr", "tt", "uk", "ur", "uz", "vi", "yi", "yo", "zh",
    ]
}

/// The rows of `labels` that `query` leaves, in their order, as indices
/// into `labels`. A row stays when its label contains the query, ignoring
/// case and surrounding spaces; an empty query leaves every row.
fn matching_languages(labels: &[&str], query: &str) -> Vec<usize> {
    let needle = query.trim().to_lowercase();
    labels
        .iter()
        .enumerate()
        .filter(|(_, label)| needle.is_empty() || label.to_lowercase().contains(&needle))
        .map(|(row, _)| row)
        .collect()
}

/// Human-readable `"code — Name"` labels matching VTS `autocaption.txt`.
fn language_labels() -> &'static [&'static str] {
    &[
        "auto — Auto-detect",
        "af — Afrikaans",
        "am — Amharic",
        "ar — Arabic",
        "as — Assamese",
        "az — Azerbaijani",
        "ba — Bashkir",
        "be — Belarusian",
        "bg — Bulgarian",
        "bn — Bengali",
        "bo — Tibetan",
        "br — Breton",
        "bs — Bosnian",
        "ca — Catalan",
        "cs — Czech",
        "cy — Welsh",
        "da — Danish",
        "de — German",
        "el — Greek",
        "en — English",
        "es — Spanish",
        "et — Estonian",
        "eu — Basque",
        "fa — Persian",
        "fi — Finnish",
        "fo — Faroese",
        "fr — French",
        "gl — Galician",
        "gu — Gujarati",
        "ha — Hausa",
        "haw — Hawaiian",
        "he — Hebrew",
        "hi — Hindi",
        "hr — Croatian",
        "ht — Haitian Creole",
        "hu — Hungarian",
        "hy — Armenian",
        "id — Indonesian",
        "is — Icelandic",
        "it — Italian",
        "ja — Japanese",
        "jw — Javanese",
        "ka — Georgian",
        "kk — Kazakh",
        "km — Khmer",
        "kn — Kannada",
        "ko — Korean",
        "la — Latin",
        "lb — Luxembourgish",
        "ln — Lingala",
        "lo — Lao",
        "lt — Lithuanian",
        "lv — Latvian",
        "mg — Malagasy",
        "mi — Maori",
        "mk — Macedonian",
        "ml — Malayalam",
        "mn — Mongolian",
        "mr — Marathi",
        "ms — Malay",
        "mt — Maltese",
        "my — Myanmar",
        "ne — Nepali",
        "nl — Dutch",
        "nn — Nynorsk",
        "no — Norwegian",
        "oc — Occitan",
        "pa — Punjabi",
        "pl — Polish",
        "ps — Pashto",
        "pt — Portuguese",
        "ro — Romanian",
        "ru — Russian",
        "sa — Sanskrit",
        "sd — Sindhi",
        "si — Sinhala",
        "sk — Slovak",
        "sl — Slovenian",
        "sn — Shona",
        "so — Somali",
        "sq — Albanian",
        "sr — Serbian",
        "su — Sundanese",
        "sv — Swedish",
        "sw — Swahili",
        "ta — Tamil",
        "te — Telugu",
        "tg — Tajik",
        "th — Thai",
        "tk — Turkmen",
        "tl — Tagalog",
        "tr — Turkish",
        "tt — Tatar",
        "uk — Ukrainian",
        "ur — Urdu",
        "uz — Uzbek",
        "vi — Vietnamese",
        "yi — Yiddish",
        "yo — Yoruba",
        "zh — Chinese",
    ]
}

/// A clip on the timeline whose voice can be captioned, as the sheet's
/// list shows it.
struct Source {
    id: String,
    label: String,
}

/// The clips on the current timeline that have sound, in the order they
/// play: an audio clip, or a video whose file has an audio stream.
fn sources(studio: &Studio) -> Vec<Source> {
    let project = studio.project();
    let mut clips: Vec<_> = studio
        .timeline()
        .clips
        .iter()
        .filter(|clip| studio.clip_has_sound(clip))
        .map(|clip| {
            let name = project
                .media_by_id(&clip.media_id)
                .map_or(clip.name.as_str(), |media| media.name.as_str());
            (clip.start, clip.id.clone(), name.to_owned())
        })
        .collect();
    clips.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    clips
        .into_iter()
        .map(|(start, id, name)| Source {
            id,
            label: source_label(&name, start),
        })
        .collect()
}

/// A clip as the list names it: its file and where it starts on the
/// timeline, so two clips of one file are told apart.
fn source_label(name: &str, start: f64) -> String {
    let seconds = start.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{name} · {hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{name} · {minutes}:{seconds:02}")
    }
}

/// Default caption placement and size (bottom placement, medium size).
pub(crate) fn default_caption_look() -> (f64, f64) {
    (CAPTION_OFFSETS[0], CAPTION_SIZES[1])
}

/// Picks a vertical offset `(offset_y, font_size)` in Concat's normalized
/// frame-height fraction `[-1.0, 1.0]` that does not collide with any existing
/// caption/text track on the timeline, so adding a second or third language
/// track (e.g. English + Japanese) stacks cleanly without overlapping.
pub(crate) fn non_overlapping_caption_look(
    studio: &Studio,
    clip_id: &str,
    preferred: (f64, f64),
) -> (f64, f64) {
    let (mut offset_y, font_size) = preferred;
    let existing_ids: std::collections::HashSet<String> = if clip_id.is_empty() {
        std::collections::HashSet::new()
    } else {
        studio.timeline().captions_of(clip_id).into_iter().collect()
    };
    let used_offsets: Vec<f64> = studio
        .timeline()
        .clips
        .iter()
        .filter(|c| {
            c.kind == concat_project::model::ClipKind::Text
                && (existing_ids.is_empty()
                    || existing_ids.contains(&c.id)
                    || c.caption_of.is_some())
        })
        .map(|c| c.offset_y)
        .collect();
    if used_offsets.is_empty() {
        return (offset_y, font_size);
    }
    // Nudge upward by 0.12 (12% of frame height) per existing caption layer at the same height.
    for _ in 0..6 {
        let collides = used_offsets.iter().any(|&y| (y - offset_y).abs() < 0.06);
        if !collides {
            break;
        }
        offset_y = (offset_y - 0.12).clamp(-0.42, 0.42);
    }
    (offset_y, font_size)
}

/// Formats seconds as `m:ss` or `h:mm:ss` for the transcription ETA readout.
fn eta_clock(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    let (hours, minutes, secs) = (total / 3600, (total / 60) % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes}:{secs:02}")
    }
}

/// One caption as a title clip: its words, when and for how long, its look,
/// and - for a caption transcribed from a clip's voice - its word timings.
#[allow(dead_code)]
pub(crate) fn caption_clip(
    text: String,
    start: f64,
    duration: f64,
    look: (f64, f64),
    base: Option<&TextStyle>,
    caption_of: Option<&str>,
    words: Option<Vec<concat_project::captionmap::Word>>,
) -> Command {
    caption_clip_on_track(
        text,
        start,
        duration,
        look,
        base,
        caption_of,
        words,
        None,
        false,
    )
}

/// Same as `caption_clip`, optionally targeting an explicit `track_id` (such
/// as `Some("__top__".to_owned())` right after `Command::AddTrack { bottom: false }`).
pub(crate) fn caption_clip_on_track(
    text: String,
    start: f64,
    duration: f64,
    look: (f64, f64),
    base: Option<&TextStyle>,
    caption_of: Option<&str>,
    words: Option<Vec<concat_project::captionmap::Word>>,
    track_id: Option<String>,
    look_preset_decided: bool,
) -> Command {
    let (offset_y, font_size) = look;
    let mut style = match base {
        Some(base) => TextStyle {
            content: text,
            font_size,
            max_width: 0.0,
            max_height: 0.0,
            ..base.clone()
        },
        None => TextStyle {
            content: text,
            font_family: "Hanken Grotesk".to_owned(),
            font_size,
            font_weight: 600.0,
            ..TextStyle::default()
        },
    };
    apply_generated_karaoke(
        &mut style.highlight_color,
        words.is_some(),
        look_preset_decided,
        concat_project::model::DEFAULT_HIGHLIGHT,
    );
    Command::AddTextClip {
        track_id,
        above: true,
        start,
        style: Some(style),
        duration: Some(duration),
        offset_y: Some(offset_y),
        caption_of: caption_of.map(str::to_owned),
        words,
    }
}

/// Longest a caption line gets before it is wrapped: about what two lines
/// of broadcast subtitle hold, and what a reader takes in at a glance.
const CAPTION_CHARS: usize = 42;

fn split_draft_at_caret(
    draft: &CaptionDraft,
    caret: usize,
) -> Result<(CaptionDraft, CaptionDraft), &'static str> {
    if !draft.audio_timed {
        return Err("This caption has estimated, not audio-timed, words; Enter cannot split it.");
    }
    let word_index = concat_project::captionmap::word_at_caret(&draft.text, &draft.words, caret)
        .ok_or("Enter needs a timed word with text on both sides.")?;
    let located = concat_project::captionmap::locate_words(&draft.text, &draft.words);
    let range = located
        .get(word_index)
        .and_then(Option::as_ref)
        .ok_or("Enter needs a timed word with text on both sides.")?;
    let boundary = draft
        .words
        .get(word_index)
        .map(|word| word.start)
        .filter(|time| time.is_finite())
        .ok_or("Enter needs a timed word with text on both sides.")?;
    if !draft.start.is_finite()
        || !draft.end.is_finite()
        || boundary < draft.start + 0.05
        || boundary > draft.end - 0.05
    {
        return Err("There is not enough audio time on both sides of this split.");
    }
    let head_text = draft.text[..range.start].trim_end().to_owned();
    let tail_text = draft.text[range.start..].trim_start().to_owned();
    if head_text.is_empty() || tail_text.is_empty() {
        return Err("Enter must leave text on both sides of the split.");
    }
    let head_words = draft.words[..word_index].to_vec();
    let tail_words = draft.words[word_index..].to_vec();
    if head_words.is_empty() || tail_words.is_empty() {
        return Err("Enter needs audio-timed words on both sides of the split.");
    }
    Ok((
        CaptionDraft {
            text: head_text,
            start: draft.start,
            end: boundary,
            words: head_words,
            audio_timed: true,
            preserve_words: true,
            edited: true,
        },
        CaptionDraft {
            text: tail_text,
            start: boundary,
            end: draft.end,
            words: tail_words,
            audio_timed: true,
            preserve_words: true,
            edited: true,
        },
    ))
}

fn script_captions(text: &str) -> Vec<(String, f64)> {
    text.lines()
        .flat_map(sentences)
        .flat_map(|sentence| wrap_caption(&sentence))
        .map(|line| {
            let seconds =
                (line.chars().count() as f64 / f64::from(CHARS_PER_SECOND)).clamp(1.0, 7.0);
            (line, seconds)
        })
        .collect()
}

fn sentences(paragraph: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = paragraph.chars().peekable();
    while let Some(ch) = chars.next() {
        current.push(ch);
        let ends = match ch {
            '。' | '！' | '？' => true,
            '.' | '!' | '?' => chars.peek().is_none_or(|next| next.is_whitespace()),
            _ => false,
        };
        if ends {
            let sentence = current.trim();
            if !sentence.is_empty() {
                out.push(sentence.to_owned());
            }
            current.clear();
        }
    }
    let rest = current.trim();
    if !rest.is_empty() {
        out.push(rest.to_owned());
    }
    out
}

fn wrap_caption(sentence: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_chars = 0;
    for word in sentence.split_whitespace() {
        let word_chars = word.chars().count();
        if line_chars > 0 && line_chars + 1 + word_chars > CAPTION_CHARS {
            lines.push(std::mem::take(&mut line));
            line_chars = 0;
        }
        if word_chars > CAPTION_CHARS {
            let mut piece = String::new();
            for ch in word.chars() {
                piece.push(ch);
                if piece.chars().count() == CAPTION_CHARS {
                    lines.push(std::mem::take(&mut piece));
                }
            }
            line = piece;
            line_chars = line.chars().count();
            continue;
        }
        if line_chars > 0 {
            line.push(' ');
            line_chars += 1;
        }
        line.push_str(word);
        line_chars += word_chars;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use concat_project::captionmap::Word;
    use concat_speech::transcribe::{HeardWord, Segment};

    use super::{
        CaptionDraft, CaptionsPane, Subject, language_codes, language_labels,
        script_captions, source_label, split_draft_at_caret,
    };

    #[test]
    fn enter_splits_a_draft_at_the_careted_words_audio_start() {
        let draft = CaptionDraft {
            text: "Halo dunia lagi".to_owned(),
            start: 2.0,
            end: 5.0,
            words: vec![
                Word { text: "Halo".to_owned(), start: 2.0, end: 2.8 },
                Word { text: "dunia".to_owned(), start: 3.0, end: 4.0 },
                Word { text: "lagi".to_owned(), start: 4.1, end: 4.8 },
            ],
            audio_timed: true,
            preserve_words: false,
            edited: false,
        };
        let (head, tail) = split_draft_at_caret(&draft, 7).expect("timed middle word");
        assert_eq!(head.text, "Halo");
        assert_eq!((head.start, head.end), (2.0, 3.0));
        assert_eq!(head.words.len(), 1);
        assert_eq!(tail.text, "dunia lagi");
        assert_eq!((tail.start, tail.end), (3.0, 5.0));
        assert_eq!(tail.words.len(), 2);
    }

    #[test]
    fn enter_split_is_one_undo_step() {
        let original = CaptionDraft {
            text: "Halo dunia".to_owned(),
            start: 0.0,
            end: 2.0,
            words: vec![
                Word { text: "Halo".to_owned(), start: 0.0, end: 0.8 },
                Word { text: "dunia".to_owned(), start: 1.0, end: 1.8 },
            ],
            audio_timed: true,
            preserve_words: false,
            edited: false,
        };
        let mut pane = CaptionsPane::default();
        pane.drafts = vec![original.clone()];
        pane.accept_draft_text(0, original.text.clone(), 7);
        assert_eq!(pane.drafts.len(), 2);
        assert_eq!(pane.drafts[0].text, "Halo");
        assert_eq!(pane.drafts[1].text, "dunia");
        pane.undo_draft();
        assert_eq!(pane.drafts.len(), 1);
        assert_eq!(pane.drafts[0].text, "Halo dunia");
        assert_eq!((pane.drafts[0].start, pane.drafts[0].end), (0.0, 2.0));
    }

    #[test]
    fn enter_with_a_final_text_edit_is_one_undo_even_for_an_already_edited_cue() {
        let original = CaptionDraft {
            text: "Halo dunia".to_owned(),
            start: 0.0,
            end: 2.0,
            words: vec![
                Word { text: "Halo".to_owned(), start: 0.0, end: 0.8 },
                Word { text: "dunia".to_owned(), start: 1.0, end: 1.8 },
            ],
            audio_timed: true,
            preserve_words: false,
            edited: true,
        };
        let mut pane = CaptionsPane::default();
        pane.drafts = vec![original.clone()];

        // TextInput may deliver the last edit together with Enter.
        pane.accept_draft_text(0, "Halo dunia!".to_owned(), 7);
        assert_eq!(pane.drafts.len(), 2);
        assert_eq!(pane.drafts[0].text, "Halo");
        assert_eq!(pane.drafts[1].text, "dunia!");
        pane.undo_draft();
        assert_eq!(pane.drafts, vec![original]);
    }

    #[test]
    fn enter_refuses_a_draft_without_audio_word_timings() {
        let draft = CaptionDraft {
            text: "Halo dunia".to_owned(),
            start: 0.0,
            end: 2.0,
            words: Vec::new(),
            audio_timed: false,
            preserve_words: false,
            edited: true,
        };
        assert!(split_draft_at_caret(&draft, 2).is_err());
    }

    #[test]
    fn enter_refuses_estimated_words_even_when_they_have_numeric_times() {
        let draft = CaptionDraft {
            text: "Halo dunia".to_owned(),
            start: 0.0,
            end: 2.0,
            words: vec![
                Word { text: "Halo".to_owned(), start: 0.0, end: 1.0 },
                Word { text: "dunia".to_owned(), start: 1.0, end: 2.0 },
            ],
            audio_timed: false,
            preserve_words: false,
            edited: false,
        };
        assert!(split_draft_at_caret(&draft, 7).is_err());
    }

    #[test]
    fn a_voice_is_named_by_its_file_and_where_it_starts() {
        assert_eq!(source_label("Interview.mp4", 0.0), "Interview.mp4 · 0:00");
        assert_eq!(source_label("Interview.mp4", 72.9), "Interview.mp4 · 1:12");
        assert_eq!(source_label("Talk.wav", 3_725.0), "Talk.wav · 1:02:05");
        assert_eq!(source_label("Talk.wav", -1.0), "Talk.wav · 0:00");
    }

    #[test]
    fn a_script_is_cut_into_readable_lines() {
        let lines = script_captions(
            "Hello there. This is version 3.5, mind!\n\nA sentence that runs on for far \
             longer than a caption line has any business running on for. Ok?",
        );
        let text: Vec<&str> = lines.iter().map(|(line, _)| line.as_str()).collect();
        assert_eq!(
            text,
            [
                "Hello there.",
                "This is version 3.5, mind!",
                "A sentence that runs on for far longer",
                "than a caption line has any business",
                "running on for.",
                "Ok?",
            ]
        );
        assert!(
            lines
                .iter()
                .all(|(_, seconds)| (1.0..=7.0).contains(seconds))
        );
        assert_eq!(lines[0].1, 1.0);
        assert!(lines[2].1 > lines[0].1);
        assert!(script_captions("  \n ").is_empty());
        assert_eq!(script_captions("你好。再见！").len(), 2);
    }

    #[test]
    fn draft_cues_populate_keep_subject_and_support_review_edits_undo_and_resplit() {
        let mut pane = CaptionsPane {
            subject: Some(Subject {
                start: 10.0,
                speed: 2.0,
                clip_id: "clip-1".to_owned(),
            }),
            ..CaptionsPane::default()
        };
        assert_eq!(pane.beam_size, 1);
        assert!(pane.word_timestamps);
        assert_eq!(language_codes().len(), language_labels().len());
        assert_eq!(language_labels()[0], "auto — Auto-detect");

        let created = pane.populate_drafts(vec![
            Segment {
                start: 0.0,
                end: 4.0,
                text: "  Hello world from Concat  ".to_owned(),
                words: vec![
                    HeardWord {
                        text: "Hello".to_owned(),
                        start: 0.0,
                        end: 1.0,
                    },
                    HeardWord {
                        text: "world".to_owned(),
                        start: 1.0,
                        end: 2.0,
                    },
                    HeardWord {
                        text: "from".to_owned(),
                        start: 2.0,
                        end: 3.0,
                    },
                    HeardWord {
                        text: "Concat".to_owned(),
                        start: 3.0,
                        end: 4.0,
                    },
                ],
            },
            Segment {
                start: 4.0,
                end: 5.0,
                text: "   ".to_owned(),
                words: Vec::new(),
            },
        ]);
        assert!(created);
        assert!(pane.subject.is_some());
        assert_eq!(pane.drafts.len(), 1);
        assert_eq!(pane.drafts[0].text, "Hello world from Concat");
        assert_eq!((pane.drafts[0].start, pane.drafts[0].end), (10.0, 12.0));
        assert!(!pane.drafts[0].edited);

        // Re-split into 3-word burst cues (4 words -> 3 + 1 = 2 cues) and undo back to 1 cue.
        pane.seg_preset = 2;
        pane.resplit_drafts();
        assert_eq!(pane.drafts.len(), 2);
        assert_eq!(pane.drafts[0].text, "Hello world from");
        assert_eq!(pane.drafts[1].text, "Concat");
        pane.undo_draft();
        assert_eq!(pane.drafts.len(), 1);
        assert_eq!(pane.drafts[0].text, "Hello world from Concat");

        pane.edit_draft_text(0, "Edited cue".to_owned());
        assert_eq!(pane.drafts[0].text, "Edited cue");
        assert!(pane.drafts[0].edited);

        pane.edit_draft_start(0, 11.98);
        assert!((pane.drafts[0].start - 11.95).abs() < 1e-5);
        pane.edit_draft_end(0, 11.5);
        assert!((pane.drafts[0].end - 12.0).abs() < 1e-5);

        pane.add_draft();
        assert_eq!(pane.drafts.len(), 2);
        assert!((pane.drafts[1].start - 12.0).abs() < 1e-5);
        assert!((pane.drafts[1].end - 14.0).abs() < 1e-5);

        pane.delete_draft(0);
        assert_eq!(pane.drafts.len(), 1);

        pane.discard_drafts();
        assert!(pane.drafts.is_empty());
        assert!(pane.subject.is_none());
        assert_eq!(language_codes()[0], "auto");
        assert!(language_codes().contains(&"en"));
        assert!(language_codes().contains(&"id"));
    }

    #[test]
    fn a_language_search_keeps_the_rows_whose_label_contains_it() {
        use super::matching_languages;
        let labels = [
            "auto — Auto-detect",
            "ja — Japanese",
            "jw — Javanese",
            "en — English",
        ];

        // An empty or blank query leaves every row, in order.
        assert_eq!(matching_languages(&labels, ""), vec![0, 1, 2, 3]);
        assert_eq!(matching_languages(&labels, "   "), vec![0, 1, 2, 3]);

        // Case and surrounding spaces do not matter; the name and the code both match.
        assert_eq!(matching_languages(&labels, "  JAPAN "), vec![1]);
        assert_eq!(matching_languages(&labels, "ja"), vec![1, 2]);
        assert_eq!(matching_languages(&labels, "en —"), vec![3]);

        // Nothing matches: no rows, not a fallback to all of them.
        assert!(matching_languages(&labels, "klingon").is_empty());
    }

    #[test]
    fn every_real_language_is_found_by_its_name_and_by_its_code() {
        use super::matching_languages;
        let labels = language_labels();
        for (row, label) in labels.iter().enumerate() {
            let (code, name) = label.split_once(" — ").expect("a code and a name");
            assert!(
                matching_languages(labels, name).contains(&row),
                "{name} should find itself"
            );
            // Codes are short, so the code can match other rows too; the row itself must still be there.
            assert!(
                matching_languages(labels, code).contains(&row),
                "{code} should find its own row"
            );
        }
        // The chosen language keeps its index in the full list, so a search does not renumber it.
        assert_eq!(
            matching_languages(labels, "Korean"),
            vec![language_codes().iter().position(|c| *c == "ko").unwrap()]
        );
    }
}
