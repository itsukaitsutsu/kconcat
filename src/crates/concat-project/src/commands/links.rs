// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Link / Unlink (roadmap E5, owner decision 16 Oct 2026): the Premiere
//! model. A video sits on its line, its audio on the audio line, and the
//! two are one group while linked - a move, a split, a delete, or a trim
//! at a shared edge acts on the whole group. Unlink lets them go their
//! own ways; Link puts them back together at any time.
//!
//! Two halves live here. [`apply`] is the pair of commands. [`widen`] is
//! the rule that makes every *other* editing command group-aware: it runs
//! once in [`super::apply`] before the arm sees the command, so no arm
//! needs to know groups exist. Captions are not members: they already
//! follow their clip through `captionOf` (`follow_captions`), which is
//! exactly what the widened move of the video gives them (test 4).

use super::*;

/// Applies one of this module's commands. Any other is a routing error.
pub(super) fn apply(
    project: &mut Project,
    mint: &mut IdMint,
    command: Command,
) -> Result<Outcome, CommandError> {
    let refuse = |reason: &str| -> Result<Outcome, CommandError> {
        Err(CommandError::CannotLink {
            reason: reason.to_owned(),
        })
    };
    match command {
        Command::LinkClips { clip_ids } => {
            let timeline = project.active();
            // Only clips that exist; a stale id in a selection is not a
            // reason to refuse the rest.
            let mut members: Vec<String> = Vec::new();
            for id in &clip_ids {
                if timeline.clip(id).is_some() && !members.contains(id) {
                    members.push(id.clone());
                }
            }
            if members
                .iter()
                .any(|id| timeline.clip(id).is_some_and(Clip::is_caption))
            {
                return refuse("A caption already follows its clip; link the video and its audio.");
            }
            // A member already in a group brings the group: a merge.
            let members = timeline.with_link_mates(members.iter().map(String::as_str));
            if members.len() < 2 {
                return refuse("Link needs at least two clips.");
            }
            // Already one group, nothing to do.
            let groups: HashSet<Option<&str>> = members
                .iter()
                .map(|id| {
                    timeline
                        .clip(id)
                        .and_then(|clip| clip.link_group.as_deref())
                })
                .collect();
            if groups.len() == 1 && !groups.contains(&None) {
                return Ok(Outcome::default());
            }
            let group = mint.next("g");
            let timeline = project.active_mut();
            for id in &members {
                if let Some(clip) = timeline.clip_mut(id) {
                    clip.link_group = Some(group.clone());
                }
            }
            Ok(Outcome {
                created_id: Some(group),
                applied: true,
            })
        }

        Command::UnlinkClips { clip_ids } => {
            let timeline = project.active_mut();
            let members = timeline.with_link_mates(clip_ids.iter().map(String::as_str));
            let mut applied = false;
            for id in &members {
                if let Some(clip) = timeline.clip_mut(id) {
                    applied |= assign(&mut clip.link_group, None);
                }
            }
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        _ => unreachable!("commands::apply routes only this module's commands here"),
    }
}

/// Two edges count as the same edge when they sit within this of one
/// another: half a millisecond, under any frame at any rate, over the
/// float noise a few moves and trims leave behind.
const SAME_EDGE: f64 = 0.0005;

/// Makes a command group-aware (E5). A command that names no linked clip
/// comes back unchanged, so a project without groups never pays for them.
///
/// - **Move:** every mate not named itself gets the same time offset on
///   its own track. A mate named explicitly keeps its own destination -
///   it was dragged, and the group drag already placed it.
/// - **Delete:** the mates go too.
/// - **Split:** the mates are cut at the same instant, and the tail
///   pieces become a group of their own ([`apply_split`]).
/// - **Trim:** a mate whose dragged edge sits at the same instant is
///   trimmed by the same amount ([`apply_trims`]). A mate whose edge is
///   elsewhere - an audio clip that already runs longer than its picture -
///   is left alone: the shared edge is what a linked trim means.
pub(super) fn widen(project: &Project, command: Command) -> Command {
    let timeline = project.active();
    let has_groups = timeline.clips.iter().any(|clip| clip.link_group.is_some());
    if !has_groups {
        return command;
    }
    match command {
        Command::MoveClips { moves } => {
            let explicit: HashSet<String> = moves.iter().map(|m| m.clip_id.clone()).collect();
            let mut widened = moves.clone();
            for wanted in &moves {
                let Some(clip) = timeline.clip(&wanted.clip_id) else {
                    continue;
                };
                let offset = wanted.start.max(0.0) - clip.start;
                for mate_id in timeline.link_mates(&clip.id) {
                    if explicit.contains(&mate_id) || widened.iter().any(|m| m.clip_id == mate_id) {
                        continue;
                    }
                    let Some(mate) = timeline.clip(&mate_id) else {
                        continue;
                    };
                    widened.push(ClipMove {
                        clip_id: mate_id,
                        start: (mate.start + offset).max(0.0),
                        track_id: mate.track_id.clone(),
                    });
                }
            }
            Command::MoveClips { moves: widened }
        }
        Command::RemoveClips { clip_ids, ripple } => Command::RemoveClips {
            clip_ids: timeline.with_link_mates(clip_ids.iter().map(String::as_str)),
            ripple,
        },
        Command::SplitClips { clip_ids, time } => {
            let widened = timeline.with_link_mates(clip_ids.iter().map(String::as_str));
            if widened.len() == clip_ids.len() {
                Command::SplitClips { clip_ids, time }
            } else {
                Command::LinkedSplit {
                    clip_ids: widened,
                    time,
                }
            }
        }
        Command::TrimClip {
            clip_id,
            edge,
            delta,
            ripple,
        } => {
            let Some(clip) = timeline.clip(&clip_id) else {
                return Command::TrimClip {
                    clip_id,
                    edge,
                    delta,
                    ripple,
                };
            };
            let at = match edge {
                TrimEdge::Start => clip.start,
                TrimEdge::End => clip.start + clip.duration,
            };
            let mut trims = vec![(clip_id.clone(), edge, delta, ripple)];
            for mate_id in timeline.link_mates(&clip_id) {
                if mate_id == clip_id {
                    continue;
                }
                let Some(mate) = timeline.clip(&mate_id) else {
                    continue;
                };
                let mate_at = match edge {
                    TrimEdge::Start => mate.start,
                    TrimEdge::End => mate.start + mate.duration,
                };
                if (mate_at - at).abs() <= SAME_EDGE {
                    trims.push((mate_id, edge, delta, ripple));
                }
            }
            if trims.len() == 1 {
                Command::TrimClip {
                    clip_id,
                    edge,
                    delta,
                    ripple,
                }
            } else {
                Command::LinkedTrim { trims }
            }
        }
        other => other,
    }
}

/// One trim per group member at the shared edge, in one command (one undo
/// step). Each is the ordinary [`Command::TrimClip`] on its own lane, so
/// every clamp and ripple rule the single trim has, the linked one has.
pub(super) fn apply_trims(
    project: &mut Project,
    mint: &mut IdMint,
    trims: Vec<(String, TrimEdge, f64, bool)>,
) -> Result<Outcome, CommandError> {
    let mut applied = false;
    for (clip_id, edge, delta, ripple) in trims {
        let outcome = clips::apply(
            project,
            mint,
            Command::TrimClip {
                clip_id,
                edge,
                delta,
                ripple,
            },
        )?;
        applied |= outcome.applied;
    }
    Ok(Outcome {
        created_id: None,
        applied,
    })
}

/// The split of a whole group at one instant, then the tails re-grouped:
/// a cut through a linked pair leaves two linked pairs, not one group of
/// four. A member the instant misses (it does not span `time`) is skipped
/// by the split itself and keeps its group; the tails that were minted get
/// one new group per old one.
pub(super) fn apply_split(
    project: &mut Project,
    mint: &mut IdMint,
    clip_ids: Vec<String>,
    time: f64,
) -> Result<Outcome, CommandError> {
    let before: HashSet<String> = project
        .active()
        .clips
        .iter()
        .map(|clip| clip.id.clone())
        .collect();
    let outcome = clips::apply(project, mint, Command::SplitClips { clip_ids, time })?;
    if !outcome.applied {
        return Ok(outcome);
    }
    // The tails: minted by the split, carrying the head's group.
    let tails: Vec<(String, String)> = project
        .active()
        .clips
        .iter()
        .filter(|clip| !before.contains(&clip.id))
        .filter_map(|clip| {
            clip.link_group
                .clone()
                .map(|group| (clip.id.clone(), group))
        })
        .collect();
    let mut fresh: HashMap<String, String> = HashMap::new();
    for (tail_id, old_group) in tails {
        let new_group = fresh
            .entry(old_group)
            .or_insert_with(|| mint.next("g"))
            .clone();
        if let Some(tail) = project.active_mut().clip_mut(&tail_id) {
            tail.link_group = Some(new_group);
        }
    }
    Ok(outcome)
}
