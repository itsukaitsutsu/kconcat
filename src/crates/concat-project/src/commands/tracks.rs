// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Lanes: adding, removing and muting or hiding them.
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
    match command {
        Command::AddTrack { bottom } => {
            let timeline = project.active_mut();
            let id = mint.next("t");
            let track = Track {
                id: id.clone(),
                visible: true,
                muted: false,
                caption_nudge_ms: 0,
                speaker: None,
                extra: Default::default(),
            };
            // The stack is stored bottom first: the last track is the lane
            // drawn on top.
            if bottom {
                timeline.tracks.insert(0, track);
            } else {
                timeline.tracks.push(track);
            }
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::RemoveTrack { track_id } => {
            let timeline = project.active_mut();
            if timeline.tracks.len() <= 1 {
                return Err(CommandError::LastTrack);
            }
            let track_count = timeline.tracks.len();
            timeline.tracks.retain(|track| track.id != track_id);
            timeline.clips.retain(|clip| clip.track_id != track_id);
            // Clips only ever sit on existing tracks, so an unknown id - the
            // tolerated no-op - removes neither.
            let applied = timeline.tracks.len() != track_count;
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SetTrackFlag {
            track_id,
            flag,
            value,
        } => {
            let timeline = project.active_mut();
            let applied = timeline
                .tracks
                .iter_mut()
                .find(|track| track.id == track_id)
                .is_some_and(|track| match flag {
                    TrackFlag::Visible => assign(&mut track.visible, value),
                    TrackFlag::Muted => assign(&mut track.muted, value),
                });
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::NudgeCaptionTrack { track_id, delta_ms } => {
            use crate::model::ranges::{CAPTION_NUDGE_STEP_MS, MAX_CAPTION_NUDGE_MS};
            let timeline = project.active_mut();
            let Some(track) = timeline
                .tracks
                .iter_mut()
                .find(|track| track.id == track_id)
            else {
                return Err(CommandError::TrackGone);
            };
            // Quantise to the step; f64::round sends a tie away from zero, so
            // +15 ms is two steps and -15 ms is two steps back.
            let steps = (f64::from(delta_ms) / f64::from(CAPTION_NUDGE_STEP_MS)).round() as i32;
            let wanted = track
                .caption_nudge_ms
                .saturating_add(steps.saturating_mul(CAPTION_NUDGE_STEP_MS));
            // The one clamp site for this number (AGENTS.md): the lane's
            // accumulated correction never passes a second either way.
            let capped = wanted.clamp(-MAX_CAPTION_NUDGE_MS, MAX_CAPTION_NUDGE_MS);
            let applied_ms = capped - track.caption_nudge_ms;
            track.caption_nudge_ms = capped;
            if applied_ms == 0 {
                return Ok(Outcome {
                    created_id: None,
                    applied: false,
                });
            }
            // Every caption on the lane moves with its word timings, so the
            // karaoke highlight stays on the cue. What counts as a caption is
            // the stored identity (T5), not the presence of words.
            let shift = f64::from(applied_ms) / 1000.0;
            for slot in timeline.clips.iter_mut() {
                if slot.track_id == track_id && slot.is_caption() {
                    let mut clip = (**slot).clone();
                    clip.start = (clip.start + shift).max(0.0);
                    if let Some(words) = clip.words.as_mut() {
                        for word in words {
                            word.start = (word.start + shift).max(0.0);
                            word.end = (word.end + shift).max(word.start);
                        }
                    }
                    *slot = std::sync::Arc::new(clip);
                }
            }
            Ok(Outcome {
                created_id: None,
                applied: true,
            })
        }

        _ => unreachable!("commands::apply routes only this module's commands here"),
    }
}
