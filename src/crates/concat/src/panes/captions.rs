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
use concat_project::captionmap::{CaptionLayoutPreset, Cue};
use concat_project::model::TextStyle;
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
    LanguageChanged(i32),
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
    NewBlankCaptions,
    OpenSubtitleFile,
    SubtitleFilePicked(PathBuf),
    DraftTextChanged(i32, String),
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
    /// The look chosen when the run began.
    look: (f64, f64),
    /// The clip whose voice this is (`empty` when created as unlinked timeline captions).
    clip_id: String,
}

#[derive(Clone, Debug, PartialEq)]
struct CaptionDraft {
    text: String,
    start: f64,
    end: f64,
    words: Vec<concat_project::captionmap::Word>,
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
    /// Row in Whisper's language list; zero is auto-detect.
    pub language: usize,
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
    pub running: bool,
    pub progress: f32,
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
            language: 0,
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
            running: false,
            progress: 0.0,
            message: String::new(),
            seg_preset: 0,
            subject: None,
            drafts: Vec::new(),
            source_cues: Vec::new(),
            undo_stack: Vec::new(),
            sources_model: Rc::new(VecModel::default()),
            download_models_model: Rc::new(VecModel::default()),
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
            CaptionsMsg::LanguageChanged(index) => {
                self.language = (index.max(0) as usize).min(language_codes().len() - 1);
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
                let _ = self.insert_drafts_inner(studio);
            }
            CaptionsMsg::DraftInsertAndDetect => {
                if let Some(clip_id) = self.insert_drafts_inner(studio) {
                    if !clip_id.is_empty() {
                        studio.selection = vec![clip_id];
                    }
                    studio.handle(Msg::Sections(crate::panes::sections::SectionsMsg::Open));
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
                self.subject = None;
                self.open = false;
            }
            CaptionsMsg::Progress(percent) => {
                self.progress = (percent as f32 / 100.0).clamp(0.0, 1.0);
            }
            CaptionsMsg::Finished(result) => {
                self.running = false;
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
            draft.edited = true;
            self.message.clear();
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
                        words: if d.edited {
                            concat_project::captionmap::estimate_words(&d.text, d.start, d.end)
                        } else {
                            d.words.clone()
                        },
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
                words: cue.words,
                edited: false,
            })
            .collect();
        self.message.clear();
    }

    fn ensure_editor_subject(&mut self, studio: &Studio) {
        if self.subject.is_some() {
            return;
        }
        let clip_opt = self.clip.as_ref().and_then(|id| studio.clip(id)).cloned();
        if let Some(clip) = clip_opt {
            self.subject = Some(Subject {
                start: clip.start,
                speed: clip.speed,
                look: self.look(),
                clip_id: clip.id,
            });
        } else {
            self.subject = Some(Subject {
                start: f64::from(studio.playhead).max(0.0),
                speed: 1.0,
                look: self.look(),
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
                words: cue.words,
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
                Some(Cue {
                    start,
                    end,
                    text,
                    words,
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
                words: cue.words,
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
            seg_preset: self.seg_preset,
            sources_model: Rc::clone(&self.sources_model),
            download_models_model: Rc::clone(&self.download_models_model),
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

    /// The script as titles, one after another from the playhead.
    fn run_script(&mut self, studio: &mut Studio) {
        let lines = script_captions(&self.text);
        if lines.is_empty() {
            self.message = t("captions.nothingToCaptionYet");
            return;
        }
        let look = self.look();
        let mut at = f64::from(studio.playhead);
        let base = studio.prefs.title_style.clone();
        let commands: Vec<Command> = lines
            .into_iter()
            .map(|(text, seconds)| {
                let command = caption_clip(text, at, seconds, look, base.as_ref(), None, None);
                at += seconds;
                command
            })
            .collect();
        let count = commands.len();
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
        let request = concat_speech::transcribe::TranscribeRequest {
            path: media.path.clone(),
            audio_stream: clip.audio_stream,
            source_start: clip.source_start,
            window: clip.duration * clip.speed,
            model_id: model,
            language: language_codes()[self.language.min(language_codes().len() - 1)].to_owned(),
            translate: self.translate,
            prompt: self.prompt.clone(),
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
            look: self.look(),
            clip_id: clip.id.clone(),
        });
        self.running = true;
        self.progress = 0.0;
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
    fn insert_drafts_inner(&mut self, studio: &mut Studio) -> Option<String> {
        let subject = self.subject.as_ref()?.clone();
        let base = studio.prefs.title_style.clone();
        let caption_of = (!subject.clip_id.is_empty()).then_some(subject.clip_id.as_str());
        let commands: Vec<Command> = self
            .drafts
            .iter()
            .filter(|cue| !cue.text.trim().is_empty() && cue.end > cue.start)
            .map(|cue| {
                let words = if cue.edited {
                    concat_project::captionmap::estimate_words(&cue.text, cue.start, cue.end)
                } else {
                    cue.words.clone()
                };
                caption_clip(
                    cue.text.trim().to_owned(),
                    cue.start,
                    cue.end - cue.start,
                    subject.look,
                    base.as_ref(),
                    caption_of,
                    Some(words),
                )
            })
            .collect();
        let count = commands.len();
        if count == 0 {
            self.message = t("captions.nothingToCaptionYet");
            return None;
        }
        self.drafts.clear();
        self.source_cues.clear();
        self.undo_stack.clear();
        self.subject = None;
        self.open = false;
        studio.apply(Command::Batch { commands });
        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
        Some(subject.clip_id)
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
        let download_status = if let Some(dl) = active_dl {
            format!(
                "Downloading {} ({:.0} / {:.0} MB)…",
                dl.id,
                dl.fetched.unwrap_or(0.0),
                dl.megabytes
            )
        } else if on_disk.is_empty() {
            t("captions.downloadTranscriberModelFirst")
        } else {
            format!(
                "{} model(s) installed · {}",
                on_disk.len(),
                studio.settings.whisper_models_dir
            )
        };
        let languages: Vec<SharedString> = language_labels()
            .iter()
            .map(|label| SharedString::from(*label))
            .collect();
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
            languages: slint::ModelRc::from(Rc::clone(&self.languages_model)),
            language: self.language as i32,
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
            running: self.running,
            progress: self.progress,
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

/// One caption as a title clip: its words, when and for how long, its look,
/// and - for a caption transcribed from a clip's voice - its word timings.
pub(crate) fn caption_clip(
    text: String,
    start: f64,
    duration: f64,
    look: (f64, f64),
    base: Option<&TextStyle>,
    caption_of: Option<&str>,
    words: Option<Vec<concat_project::captionmap::Word>>,
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
    if words.is_some() && style.highlight_color.is_empty() {
        style.highlight_color = concat_project::model::DEFAULT_HIGHLIGHT.to_owned();
    }
    Command::AddTextClip {
        track_id: None,
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
    use concat_speech::transcribe::{HeardWord, Segment};

    use super::{
        CaptionsPane, Subject, language_codes, language_labels, script_captions, source_label,
    };

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
                look: (0.35, 0.05),
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
}
