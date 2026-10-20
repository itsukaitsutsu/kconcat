// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The speaker registry and its labels (roadmap P3, owner decisions of
//! 16 Oct 2026: a speaker is a label; tags are editable; D7 gives each
//! speaker's captions a lane; nothing runs unless the user asks).
//!
//! One arm per command, exactly as [`super::apply`] routes them here;
//! everything these arms share lives in the parent module.

use super::*;

/// Applies one of this module's commands. Any other is a routing error.
pub(super) fn apply(
    project: &mut Project,
    mint: &mut IdMint,
    command: Command,
) -> Result<Outcome, CommandError> {
    let refuse = |reason: &str| -> Result<Outcome, CommandError> {
        Err(CommandError::CannotEditSpeakers {
            reason: reason.to_owned(),
        })
    };
    match command {
        Command::AddSpeaker { name, color } => {
            if name.trim().is_empty() {
                return refuse("A speaker needs a name.");
            }
            let id = mint.next("sp");
            let timeline = project.active_mut();
            timeline.speakers.push(crate::model::Speaker {
                id: id.clone(),
                name: name.trim().to_owned(),
                color,
            });
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::AssignCaptionToSpeaker {
            clip_id,
            speaker_id,
        } => {
            let timeline = project.active_mut();
            if !timeline
                .speakers
                .iter()
                .any(|speaker| speaker.id == speaker_id)
            {
                return refuse("That speaker no longer exists.");
            }
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return refuse("That caption no longer exists.");
            };
            if !clip.is_caption() {
                return refuse("Only a caption can be given to a speaker.");
            }
            // D7: one lane per speaker, made on first use - no empty
            // speaker slots before a caption actually joins. The stack is
            // stored bottom first, so pushing puts the new lane on top,
            // where caption lanes live.
            let lane = timeline
                .tracks
                .iter()
                .find(|track| track.speaker.as_deref() == Some(speaker_id.as_str()))
                .map(|track| track.id.clone())
                .unwrap_or_else(|| {
                    let id = mint.next("t");
                    timeline.tracks.push(crate::model::Track {
                        id: id.clone(),
                        kind: crate::model::TrackKind::Video,
                        visible: true,
                        muted: false,
                        caption_nudge_ms: 0,
                        speaker: Some(speaker_id.clone()),
                        extra: Default::default(),
                    });
                    id
                });
            // The label goes on the caption; every word of it inherits the
            // speaker through the caption, which is what "the label lives
            // on each transcript word" means at caption granularity - a
            // caption is one speaker's utterance. The heard timings and
            // the transcript stand as they are.
            let clip = timeline
                .clip_mut(&clip_id)
                .expect("found above, and nothing between removed it");
            clip.track_id = lane;
            clip.speaker = Some(speaker_id);
            Ok(Outcome {
                created_id: None,
                applied: true,
            })
        }

        Command::RenameSpeaker { speaker_id, name } => {
            if name.trim().is_empty() {
                return refuse("A speaker needs a name.");
            }
            let timeline = project.active_mut();
            let Some(speaker) = timeline
                .speakers
                .iter_mut()
                .find(|speaker| speaker.id == speaker_id)
            else {
                return refuse("That speaker no longer exists.");
            };
            let name = name.trim().to_owned();
            let applied = speaker.name != name;
            speaker.name = name;
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SetSpeakerColor { speaker_id, color } => {
            let timeline = project.active_mut();
            let Some(speaker) = timeline
                .speakers
                .iter_mut()
                .find(|speaker| speaker.id == speaker_id)
            else {
                return refuse("That speaker no longer exists.");
            };
            let applied = speaker.color != color;
            speaker.color = color;
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::RemoveSpeaker { speaker_id } => {
            let timeline = project.active_mut();
            if !timeline
                .speakers
                .iter()
                .any(|speaker| speaker.id == speaker_id)
            {
                return refuse("That speaker no longer exists.");
            }
            if timeline
                .clips
                .iter()
                .any(|clip| clip.speaker.as_deref() == Some(speaker_id.as_str()))
            {
                return refuse("That speaker still has captions; move or delete them first.");
            }
            timeline.speakers.retain(|speaker| speaker.id != speaker_id);
            // The speaker's lane - empty, or the refusals above would have
            // fired - goes with the speaker. The floor of one lane is safe:
            // a speaker lane is always an extra.
            timeline
                .tracks
                .retain(|track| track.speaker.as_deref() != Some(speaker_id.as_str()));
            Ok(Outcome {
                created_id: None,
                applied: true,
            })
        }

        _ => unreachable!("commands::apply routes only this module's commands here"),
    }
}
