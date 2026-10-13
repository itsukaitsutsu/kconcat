// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Import timed subtitle files as editable caption clips.

use std::path::{Path, PathBuf};

use concat_project::Command;

use crate::host::spawn_in_project;
use crate::i18n::{t, tf};
use crate::panes::Msg;
use crate::panes::captions::caption_clip;
use crate::studio::Studio;
use crate::ui::CaptionImportData;

/// The timebase chosen for an import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timebase {
    /// Subtitle times already refer to the project timeline.
    ProjectStart,
    /// Subtitle times refer to the selected clip's source media.
    SelectedClip,
}

/// The selected clip's clocks, captured before the file is parsed.
#[derive(Clone, Debug)]
pub struct ClipTarget {
    id: String,
    label: String,
    timeline_start: f64,
    source_start: f64,
    source_end: f64,
    speed: f64,
}

/// One parsed subtitle cue, in its file's clock.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionCue {
    start: f64,
    end: f64,
    text: String,
}

/// A cue after it has been placed on the project timeline.
#[derive(Clone, Debug, PartialEq)]
struct PlacedCue {
    start: f64,
    duration: f64,
    text: String,
    caption_of: Option<String>,
}

/// One thing the import dialog or its worker reports.
#[derive(Clone, Debug)]
pub enum ImportCaptionsMsg {
    /// The native picker returned one or more SRT/VTT paths.
    FilesPicked(Vec<PathBuf>),
    /// Use the subtitle times from project time zero.
    ProjectStart,
    /// Align source-media times to the selected, constant-speed video clip.
    SelectedClip,
    Cancel,
    Parsed {
        timebase: Timebase,
        target: Option<ClipTarget>,
        result: Result<Vec<CaptionCue>, String>,
    },
}

/// The choice sheet and its pending file import.
#[derive(Default)]
pub struct ImportCaptionsPane {
    pub open: bool,
    pub loading: bool,
    pub message: String,
    files: Vec<PathBuf>,
    selected_clip: Option<ClipTarget>,
}

impl ImportCaptionsPane {
    /// Applies one import-sheet message.
    pub fn update(&mut self, msg: ImportCaptionsMsg, studio: &mut Studio) {
        match msg {
            ImportCaptionsMsg::FilesPicked(paths) => self.picked(paths, studio),
            ImportCaptionsMsg::ProjectStart => {
                self.begin(Timebase::ProjectStart, None, studio);
            }
            ImportCaptionsMsg::SelectedClip => {
                let Some(target) = self.selected_clip.clone() else {
                    self.message = t("captionImport.selectClipFirst");
                    return;
                };
                self.begin(Timebase::SelectedClip, Some(target), studio);
            }
            ImportCaptionsMsg::Cancel => *self = Self::default(),
            ImportCaptionsMsg::Parsed {
                timebase,
                target,
                result,
            } => {
                self.loading = false;
                match result {
                    Err(error) => self.message = error,
                    Ok(cues) => {
                        let placed = place_cues(&cues, timebase, target.as_ref());
                        if placed.is_empty() {
                            self.message = if timebase == Timebase::SelectedClip {
                                t("captionImport.noCuesInClip")
                            } else {
                                t("captionImport.noCues")
                            };
                            return;
                        }
                        let count = placed.len();
                        let commands = placed
                            .into_iter()
                            .map(|cue| {
                                caption_clip(
                                    cue.text,
                                    cue.start,
                                    cue.duration,
                                    (0.0, 0.05),
                                    None,
                                    cue.caption_of.as_deref(),
                                    None,
                                )
                            })
                            .collect();
                        studio.apply(Command::Batch { commands });
                        *self = Self::default();
                        studio.notify(&tf("captions.addedCaptions", &[&count]), false);
                    }
                }
            }
        }
    }

    /// What the user chose in the native file picker; open the timebase sheet.
    fn picked(&mut self, paths: Vec<PathBuf>, studio: &mut Studio) {
        if studio.session.is_none() {
            studio.notify(&t("captionImport.openProjectFirst"), true);
            return;
        }
        let paths: Vec<PathBuf> = paths
            .into_iter()
            .filter(|path| is_subtitle_file(path))
            .collect();
        if paths.is_empty() {
            studio.notify(&t("captionImport.noSupportedFiles"), true);
            return;
        }
        self.files = paths;
        self.selected_clip = selected_target(studio);
        self.message.clear();
        self.loading = false;
        self.open = true;
    }

    /// Reads the subtitle files on a worker; importing remains one undo step.
    fn begin(&mut self, timebase: Timebase, target: Option<ClipTarget>, studio: &Studio) {
        if self.loading || self.files.is_empty() {
            return;
        }
        if timebase == Timebase::SelectedClip && target.is_none() {
            self.message = t("captionImport.selectClipFirst");
            return;
        }
        self.loading = true;
        self.message = t("captionImport.readingFiles");
        let files = self.files.clone();
        spawn_in_project(
            move || parse_files(&files),
            move |studio, _, _, result| {
                studio.handle(Msg::ImportCaptions(ImportCaptionsMsg::Parsed {
                    timebase,
                    target,
                    result,
                }));
            },
        );
    }

    /// The data shown by the timebase choice sheet.
    pub fn data(&self) -> CaptionImportData {
        let files = if self.files.len() == 1 {
            self.files[0]
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else if self.files.is_empty() {
            String::new()
        } else {
            tf("captionImport.fileCount", &[&self.files.len()])
        };
        CaptionImportData {
            open: self.open,
            files: files.into(),
            selected_clip: self
                .selected_clip
                .as_ref()
                .map(|target| target.label.as_str())
                .unwrap_or_default()
                .into(),
            selected_clip_available: self.selected_clip.is_some(),
            loading: self.loading,
            message: self.message.as_str().into(),
        }
    }
}

fn selected_target(studio: &Studio) -> Option<ClipTarget> {
    let id = studio.sole_selection()?;
    let clip = studio.clip(&id)?;
    // A variable speed curve needs an integration of its speed function to
    // map a media clock; do not offer a misleading alignment for it.
    if !clip.kind.is_visual()
        || clip.speed_curve.is_some()
        || !clip.speed.is_finite()
        || clip.speed <= 0.0
    {
        return None;
    }
    let name = studio
        .project()
        .media_by_id(&clip.media_id)
        .map_or(clip.name.as_str(), |media| media.name.as_str());
    Some(ClipTarget {
        id: clip.id.clone(),
        label: format!("{} · {}", name, clock(clip.start)),
        timeline_start: clip.start,
        source_start: clip.source_start,
        source_end: clip.source_start + clip.duration * clip.speed,
        speed: clip.speed,
    })
}

fn is_subtitle_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "srt" | "vtt"))
}

fn parse_files(paths: &[PathBuf]) -> Result<Vec<CaptionCue>, String> {
    let mut cues = Vec::new();
    for path in paths {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let text = String::from_utf8_lossy(&bytes);
        let parsed =
            parse_subtitles(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        cues.extend(parsed);
    }
    cues.sort_by(|left, right| left.start.total_cmp(&right.start));
    if cues.is_empty() {
        return Err(t("captionImport.noCues"));
    }
    Ok(cues)
}

fn parse_subtitles(input: &str) -> Result<Vec<CaptionCue>, String> {
    let input = input
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let lines: Vec<&str> = input.lines().collect();
    let mut cues = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index].trim();
        if line.is_empty() || line == "WEBVTT" || line.starts_with("WEBVTT ") {
            index += 1;
            continue;
        }
        if line == "NOTE" || line.starts_with("NOTE ") || line == "STYLE" || line == "REGION" {
            index += 1;
            while index < lines.len() && !lines[index].trim().is_empty() {
                index += 1;
            }
            continue;
        }
        let Some((start_text, end_text)) = line.split_once("-->") else {
            // SRT sequence numbers and WebVTT cue identifiers are optional.
            index += 1;
            continue;
        };
        let start = parse_time(start_text.trim())
            .ok_or_else(|| format!("invalid cue start on line {}", index + 1))?;
        let end_token = end_text.split_whitespace().next().unwrap_or_default();
        let end = parse_time(end_token)
            .ok_or_else(|| format!("invalid cue end on line {}", index + 1))?;
        if end <= start {
            return Err(format!(
                "cue end must follow its start on line {}",
                index + 1
            ));
        }
        index += 1;
        let mut body = Vec::new();
        while index < lines.len() && !lines[index].trim().is_empty() {
            if lines[index].contains("-->") {
                break;
            }
            body.push(lines[index]);
            index += 1;
        }
        let text = plain_text(&body);
        if !text.trim().is_empty() {
            cues.push(CaptionCue { start, end, text });
        }
        if index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
    }
    Ok(cues)
}

fn parse_time(value: &str) -> Option<f64> {
    let normalized = value.trim().replace(',', ".");
    let parts: Vec<&str> = normalized.split(':').collect();
    let (hours, minutes, seconds) = match parts.as_slice() {
        [minutes, seconds] => (
            0.0,
            minutes.parse::<u64>().ok()? as f64,
            seconds.parse::<f64>().ok()?,
        ),
        [hours, minutes, seconds] => (
            hours.parse::<u64>().ok()? as f64,
            minutes.parse::<u64>().ok()? as f64,
            seconds.parse::<f64>().ok()?,
        ),
        _ => return None,
    };
    if minutes >= 60.0 && parts.len() == 3
        || !seconds.is_finite()
        || !(0.0..60.0).contains(&seconds)
    {
        return None;
    }
    let total = hours * 3600.0 + minutes * 60.0 + seconds;
    (total.is_finite() && total >= 0.0).then_some(total)
}

fn plain_text(lines: &[&str]) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in lines.join("\n").chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn place_cues(
    cues: &[CaptionCue],
    timebase: Timebase,
    target: Option<&ClipTarget>,
) -> Vec<PlacedCue> {
    cues.iter()
        .filter_map(|cue| match timebase {
            Timebase::ProjectStart => Some(PlacedCue {
                start: cue.start,
                duration: cue.end - cue.start,
                text: cue.text.clone(),
                caption_of: None,
            }),
            Timebase::SelectedClip => {
                let target = target?;
                let start = cue.start.max(target.source_start);
                let end = cue.end.min(target.source_end);
                if end <= start {
                    return None;
                }
                Some(PlacedCue {
                    start: target.timeline_start + (start - target.source_start) / target.speed,
                    duration: (end - start) / target.speed,
                    text: cue.text.clone(),
                    caption_of: Some(target.id.clone()),
                })
            }
        })
        .collect()
}

fn clock(seconds: f64) -> String {
    let seconds = seconds.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::{CaptionCue, ClipTarget, Timebase, parse_subtitles, place_cues};

    #[test]
    fn parses_srt_cues_and_strips_basic_markup() {
        let cues = parse_subtitles(
            "1\r\n00:00:01,250 --> 00:00:02,500\r\n<i>Hello</i> &amp; world\r\n\r\n2\r\n00:00:03,000 --> 00:00:04,000\r\nSecond line\r\n",
        )
        .expect("valid SRT");
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].start, 1.25);
        assert_eq!(cues[0].end, 2.5);
        assert_eq!(cues[0].text, "Hello & world");
    }

    #[test]
    fn parses_webvtt_header_ids_and_cue_settings() {
        let cues =
            parse_subtitles("WEBVTT\n\ncue-1\n00:00.500 --> 00:02.000 align:start\nA caption\n")
                .expect("valid WebVTT");
        assert_eq!(
            cues,
            [CaptionCue {
                start: 0.5,
                end: 2.0,
                text: "A caption".to_owned(),
            }]
        );
    }

    #[test]
    fn selected_clip_alignment_accounts_for_source_in_point_and_speed() {
        let target = ClipTarget {
            id: "c1".to_owned(),
            label: "clip".to_owned(),
            timeline_start: 10.0,
            source_start: 5.0,
            source_end: 15.0,
            speed: 2.0,
        };
        let cues = [CaptionCue {
            start: 4.0,
            end: 8.0,
            text: "overlap".to_owned(),
        }];
        let placed = place_cues(&cues, Timebase::SelectedClip, Some(&target));
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].start, 10.0);
        assert_eq!(placed[0].duration, 1.5);
        assert_eq!(placed[0].caption_of.as_deref(), Some("c1"));
    }

    #[test]
    fn project_timecodes_are_not_shifted_or_linked_to_a_clip() {
        let cues = [CaptionCue {
            start: 2.0,
            end: 3.0,
            text: "project".to_owned(),
        }];
        let placed = place_cues(&cues, Timebase::ProjectStart, None);
        assert_eq!(placed[0].start, 2.0);
        assert_eq!(placed[0].duration, 1.0);
        assert_eq!(placed[0].caption_of, None);
    }
}
