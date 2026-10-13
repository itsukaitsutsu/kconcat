// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The captions sheet: the tray's Captions tool, as a form and then as a
//! progress report.
//!
//! One switch says what it captions: "Use voice from timeline", on, takes
//! the words from a clip with sound, chosen in the sheet from the clips on
//! the timeline that have any; off, from a script typed in it. It opens on
//! when a clip with sound is selected, or when a clip's menu opened it on
//! that clip, and off otherwise. Either lands as one batch of title clips,
//! one undo step.
//! The transcriber runs on a worker and reports as
//! [`CaptionsMsg::Progress`], ending as [`CaptionsMsg::Finished`].

use std::sync::Arc;

use concat_project::Command;
use concat_project::model::TextStyle;
use concat_speech::transcribe::Segment;

use crate::host::{on_ui_in_project, spawn_in_project};
use crate::i18n::{t, tf};
use crate::panes::Msg;
use crate::panes::settings::installed;
use crate::studio::Studio;
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
    LanguageChanged(i32),
    TranslateChanged(bool),
    PromptChanged(String),
    PlacementChanged(i32),
    SizeChanged(i32),
    DraftTextChanged(i32, String),
    DraftStartChanged(i32, f32),
    DraftEndChanged(i32, f32),
    DraftDelete(i32),
    DraftAdd,
    DraftDiscard,
    DraftInsert,
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
// Clone but not Copy: Phase 4a gave it a `String`, and a clip id cannot
// be copied bitwise. Nothing needed Copy - the subject is taken once.
#[derive(Clone, Debug)]
struct Subject {
    start: f64,
    speed: f64,
    /// The look chosen when the run began.
    look: (f64, f64),
    /// Phase 4a: the clip whose voice this is. Kept from when the run
    /// began (the sheet's selection could in principle change while the
    /// worker runs), and written onto every caption it makes as
    /// `caption_of`, so the words follow this clip's moves and trims.
    clip_id: String,
}

#[derive(Clone, Debug)]
struct CaptionDraft {
    text: String,
    start: f64,
    end: f64,
    words: Vec<concat_project::captionmap::Word>,
    edited: bool,
}

/// The captions sheet's state.
#[derive(Default)]
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
    /// Row in Whisper's language list; zero is auto-detect.
    pub language: usize,
    pub translate: bool,
    pub prompt: String,
    /// 0 bottom, 1 centre, 2 top.
    pub placement: usize,
    /// 0 small, 1 medium, 2 large.
    pub size: usize,
    pub running: bool,
    pub progress: f32,
    /// Why the last run failed, when it did.
    pub message: String,
    /// The clip under the running transcription.
    subject: Option<Subject>,
    drafts: Vec<CaptionDraft>,
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
            CaptionsMsg::ModelChanged(index) => self.model = index.max(0) as usize,
            CaptionsMsg::LanguageChanged(index) => {
                self.language = (index.max(0) as usize).min(language_codes().len() - 1)
            }
            CaptionsMsg::TranslateChanged(on) => self.translate = on,
            CaptionsMsg::PromptChanged(text) => self.prompt = text,
            CaptionsMsg::PlacementChanged(index) => {
                self.placement = (index.max(0) as usize).min(2);
            }
            CaptionsMsg::SizeChanged(index) => self.size = (index.max(0) as usize).min(2),
            CaptionsMsg::DraftTextChanged(index, text) => {
                if let Some(draft) = usize::try_from(index)
                    .ok()
                    .and_then(|i| self.drafts.get_mut(i))
                {
                    draft.text = text;
                    draft.edited = true;
                }
            }
            CaptionsMsg::DraftStartChanged(index, value) => {
                if let Some(draft) = usize::try_from(index)
                    .ok()
                    .and_then(|i| self.drafts.get_mut(i))
                {
                    draft.start = f64::from(value).max(0.0).min(draft.end - 0.05);
                    draft.edited = true;
                }
            }
            CaptionsMsg::DraftEndChanged(index, value) => {
                if let Some(draft) = usize::try_from(index)
                    .ok()
                    .and_then(|i| self.drafts.get_mut(i))
                {
                    draft.end = f64::from(value).max(draft.start + 0.05);
                    draft.edited = true;
                }
            }
            CaptionsMsg::DraftDelete(index) => {
                if let Ok(index) = usize::try_from(index) {
                    if index < self.drafts.len() {
                        self.drafts.remove(index);
                    }
                }
            }
            CaptionsMsg::DraftAdd => {
                let start = self.drafts.last().map_or(0.0, |cue| cue.end);
                self.drafts.push(CaptionDraft {
                    text: String::new(),
                    start,
                    end: start + 2.0,
                    words: Vec::new(),
                    edited: true,
                });
            }
            CaptionsMsg::DraftDiscard => {
                self.drafts.clear();
                self.subject = None;
            }
            CaptionsMsg::DraftInsert => self.insert_drafts(studio),
            CaptionsMsg::Begin => {
                self.drafts.clear();
                if self.from_voice {
                    self.run_sound(studio);
                } else {
                    self.run_script(studio);
                }
            }
            CaptionsMsg::Cancel => {
                studio.host.transcriber.cancel();
                self.running = false;
                self.open = false;
            }
            CaptionsMsg::Progress(percent) => {
                self.progress = (percent as f32 / 100.0).clamp(0.0, 1.0);
            }
            CaptionsMsg::Finished(result) => {
                self.running = false;
                let subject = self.subject.take();
                match result {
                    Ok(segments) => {
                        let Some(subject) = self.subject.as_ref() else {
                            return;
                        };
                        self.drafts = segments
                            .into_iter()
                            .filter_map(|segment| {
                                let text = segment.text.trim().to_owned();
                                if text.is_empty() {
                                    return None;
                                }
                                let start = subject.start + segment.start / subject.speed;
                                let end = start
                                    + ((segment.end - segment.start) / subject.speed).max(0.2);
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
                                Some(CaptionDraft {
                                    text,
                                    start,
                                    end,
                                    words,
                                    edited: false,
                                })
                            })
                            .collect();
                        if self.drafts.is_empty() {
                            self.subject = None;
                            studio.notify(&t("captions.nothingSaidClip"), true);
                        }
                    }
                    // Asked for: the sheet is already on its way down.
                    Err(error) if error.contains("cancel") => self.open = false,
                    Err(error) => self.message = error,
                }
            }
        }
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
            placement: 0,
            size: 1,
            ..CaptionsPane::default()
        };
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
                // A script belongs to the timeline, not to one clip's
                // voice, so its captions are unlinked titles.
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
        let request = concat_speech::transcribe::TranscribeRequest {
            path: media.path.clone(),
            audio_stream: clip.audio_stream,
            source_start: clip.source_start,
            window: clip.duration * clip.speed,
            model_id: model,
            language: language_codes()[self.language.min(language_codes().len() - 1)].to_owned(),
            translate: self.translate,
            prompt: self.prompt.clone(),
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

    fn insert_drafts(&mut self, studio: &mut Studio) {
        let Some(subject) = self.subject.take() else {
            return;
        };
        let base = studio.prefs.title_style.clone();
        let commands: Vec<Command> = self
            .drafts
            .drain(..)
            .filter(|cue| !cue.text.trim().is_empty() && cue.end > cue.start)
            .map(|cue| {
                let words = if cue.edited {
                    concat_project::captionmap::estimate_words(&cue.text, cue.start, cue.end)
                } else {
                    cue.words
                };
                caption_clip(
                    cue.text.trim().to_owned(),
                    cue.start,
                    cue.end - cue.start,
                    subject.look,
                    base.as_ref(),
                    Some(subject.clip_id.as_str()),
                    Some(words),
                )
            })
            .collect();
        let count = commands.len();
        if count == 0 {
            self.message = t("captions.nothingToCaptionYet");
            return;
        }
        self.open = false;
        studio.apply(Command::Batch { commands });
        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
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
        let labels: Vec<slint::SharedString> = sources
            .into_iter()
            .map(|source| source.label.into())
            .collect();
        CaptionsSheetData {
            open: self.open,
            from_voice: self.from_voice,
            sources: std::rc::Rc::new(slint::VecModel::from(labels)).into(),
            source,
            text: self.text.as_str().into(),
            model: self.model as i32,
            languages: std::rc::Rc::new(slint::VecModel::from(
                language_codes()
                    .iter()
                    .map(|code| slint::SharedString::from(*code))
                    .collect::<Vec<_>>(),
            ))
            .into(),
            language: self.language as i32,
            translate: self.translate,
            prompt: self.prompt.as_str().into(),
            placement: self.placement as i32,
            size: self.size as i32,
            running: self.running,
            progress: self.progress,
            ready: !installed(&studio.settings.transcribers).is_empty(),
            reviewing: !self.drafts.is_empty(),
            drafts: std::rc::Rc::new(slint::VecModel::from(
                self.drafts
                    .iter()
                    .map(|cue| CaptionDraftData {
                        text: cue.text.as_str().into(),
                        start: cue.start as f32,
                        end: cue.end as f32,
                    })
                    .collect::<Vec<_>>(),
            ))
            .into(),
            message: self.message.as_str().into(),
        }
    }
}

/// Languages understood by multilingual Whisper models. Codes are used as
/// labels too: they stay compact and match VTS's language selector exactly.
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
    // The kept look where there is one - family, weight, colours, stroke,
    // shadow, background - at the caption's own size and place; the
    // bundled face otherwise.
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
    // Phase 6c: a word-timed caption lights the word being spoken in an
    // accent. The default is this; the caption's own colour overrides it.
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
        // Phase 6a/6b: the word timings the caller worked out - heard when
        // whisper gave them, estimated otherwise (see captionmap::caption_words).
        // A script's titles pass None: they belong to no voice, and there is
        // nothing to highlight.
        words,
    }
}

/// Longest a caption line gets before it is wrapped: about what two lines
/// of broadcast subtitle hold, and what a reader takes in at a glance.
const CAPTION_CHARS: usize = 42;

/// A script as caption lines, each with how long it stays up: a line's
/// reading time at [`CHARS_PER_SECOND`], held to one second at least so
/// a short word is not a flicker, and seven at most so a long line does
/// not hang. A line break in the script is a break the author asked for;
/// within a paragraph a sentence is a caption, and a long sentence wraps
/// at its words.
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

/// A paragraph's sentences. A full stop, question or exclamation mark ends
/// one when it is followed by space or by the end - so "3.5" and "e.g." hold
/// together - and the CJK marks end one on their own.
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

/// A sentence in lines of at most [`CAPTION_CHARS`], broken between words;
/// a word longer than a line, or a run of CJK with no spaces, is broken
/// where it must be.
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
    use super::{script_captions, source_label};

    /// A clip in the voice list is its file and where it starts, in
    /// minutes and seconds, with hours once there are any.
    #[test]
    fn a_voice_is_named_by_its_file_and_where_it_starts() {
        assert_eq!(source_label("Interview.mp4", 0.0), "Interview.mp4 · 0:00");
        assert_eq!(source_label("Interview.mp4", 72.9), "Interview.mp4 · 1:12");
        assert_eq!(source_label("Talk.wav", 3_725.0), "Talk.wav · 1:02:05");
        assert_eq!(source_label("Talk.wav", -1.0), "Talk.wav · 0:00");
    }

    /// A script becomes one caption per sentence, a hand line break is
    /// kept, a long sentence wraps at its words, and each line is held for
    /// its reading time within one to seven seconds.
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
}
