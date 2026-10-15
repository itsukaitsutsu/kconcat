// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Placing, moving and cutting clips: the edits that change what is on the timeline and when.
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
        Command::AddClip {
            media_id,
            track_id,
            start,
            ripple,
        } => {
            let media = project
                .media_by_id(&media_id)
                .ok_or(CommandError::MediaGone)?
                .clone();
            let timeline = project.active_mut();
            if timeline.track(&track_id).is_none() {
                return Err(CommandError::TrackGone);
            }
            // Magnetic, the lane makes room; otherwise the clip takes the
            // nearest gap it fits, never the top of another clip.
            let start = if ripple {
                ripple_room_for(timeline, &track_id, start, &media)
            } else {
                timeline.nearest_free_start(&track_id, start, clip_length(&media), &[])
            };
            let id = mint.next("c");
            timeline
                .clips
                .push(Arc::new(default_clip(id.clone(), track_id, &media, start)));
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::AddClipAtFirstFree { media_id, start } => {
            let media = project
                .media_by_id(&media_id)
                .ok_or(CommandError::MediaGone)?
                .clone();
            let duration = clip_length(&media);
            let timeline = project.active_mut();
            let track_id = first_free_track(timeline, start, duration)
                .unwrap_or_else(|| fresh_track(timeline, mint));
            let id = mint.next("c");
            timeline
                .clips
                .push(Arc::new(default_clip(id.clone(), track_id, &media, start)));
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::AddTextClip {
            track_id,
            start,
            style,
            duration,
            offset_y,
            above,
            caption_of,
            words,
        } => {
            let style = style.unwrap_or_default();
            let duration = duration
                .unwrap_or(DEFAULT_TEXT_DURATION)
                .max(MIN_CLIP_DURATION);
            let timeline = project.active_mut();
            let track_id = match track_id {
                Some(ref id) if id == "__top__" => timeline
                    .tracks
                    .last()
                    .map(|t| t.id.clone())
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
                Some(id) if timeline.track(&id).is_some() => id,
                Some(_) => return Err(CommandError::TrackGone),
                // Every lane above the video is taken: a new one at the top
                // for the words to land on.
                None if above => first_free_track_above(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
                None => first_free_track(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
            };
            let start = timeline.nearest_free_start(&track_id, start, duration, &[]);
            let id = mint.next("c");
            let mut clip = Clip::blank(
                id.clone(),
                track_id,
                ClipKind::Text,
                first_line(&style.content),
                start,
                duration,
            );
            clip.offset_y = offset_y.unwrap_or(0.0).clamp(-MAX_OFFSET, MAX_OFFSET);
            // Phase 4a: the link the caption run asked for. Stored as
            // given - the run names a clip on this timeline, and a link
            // that later dangles (its clip deleted) is Phase 1's
            // `captions_with_no_clip` case, not an error here.
            clip.caption_of = caption_of;
            clip.words = words;
            clip.text = Some(style);
            timeline.clips.push(Arc::new(clip));
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::AddShapeClip {
            track_id,
            above,
            start,
            style,
            duration,
            name,
        } => {
            let style = style.unwrap_or_default().tidy();
            let duration = duration
                .unwrap_or(DEFAULT_TEXT_DURATION)
                .max(MIN_CLIP_DURATION);
            let timeline = project.active_mut();
            let track_id = match track_id {
                Some(id) if timeline.track(&id).is_some() => id,
                Some(_) => return Err(CommandError::TrackGone),
                None if above => first_free_track_above(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
                None => first_free_track(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
            };
            let start = timeline.nearest_free_start(&track_id, start, duration, &[]);
            let id = mint.next("c");
            let name = if name.trim().is_empty() {
                style.kind.id().to_owned()
            } else {
                name
            };
            let mut clip =
                Clip::blank(id.clone(), track_id, ClipKind::Shape, name, start, duration);
            clip.shape = Some(style);
            timeline.clips.push(Arc::new(clip));
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::AddLayerClip {
            track_id,
            start,
            duration,
            effect_id,
            name,
        } => {
            let duration = duration
                .unwrap_or(DEFAULT_LAYER_DURATION)
                .max(MIN_CLIP_DURATION);
            let timeline = project.active_mut();
            let track_id = match track_id {
                Some(id) if timeline.track(&id).is_some() => id,
                Some(_) => return Err(CommandError::TrackGone),
                None => first_free_track(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
            };
            let start = timeline.nearest_free_start(&track_id, start, duration, &[]);
            let id = mint.next("c");
            let name = if name.trim().is_empty() {
                effect_id.clone()
            } else {
                name
            };
            let mut clip =
                Clip::blank(id.clone(), track_id, ClipKind::Layer, name, start, duration);
            clip.video_effects = vec![AppliedFilter::new(effect_id)];
            timeline.clips.push(Arc::new(clip));
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::MoveClips { moves } => {
            let timeline = project.active_mut();
            // Onto another clip is never where a move lands: the group goes
            // to the nearest gap that holds it, and a move whose own clips
            // would cover each other stays where it was.
            let Some(moves) = timeline.resolve_moves(&moves) else {
                return Ok(Outcome::default());
            };
            // Phase 2 (linked captions): remember where every moved clip
            // started, so its captions can be shifted by the same amount
            // below. A caption named in `moves` itself is excluded - it was
            // dragged explicitly, and shifting it twice would double-move it.
            let before: std::collections::HashMap<String, f64> = moves
                .iter()
                .filter_map(|wanted| {
                    timeline
                        .clip(&wanted.clip_id)
                        .map(|clip| (clip.id.clone(), clip.start))
                })
                .collect();
            let explicit: HashSet<String> =
                moves.iter().map(|wanted| wanted.clip_id.clone()).collect();
            let mut applied = false;
            for wanted in moves {
                if let Some(clip) = timeline.clip_mut(&wanted.clip_id) {
                    applied |= assign(&mut clip.start, wanted.start);
                    applied |= assign(&mut clip.track_id, wanted.track_id);
                }
            }
            if applied {
                applied |= follow_captions(timeline, &before, &HashSet::new(), &explicit);
            }
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::TrimClip {
            clip_id,
            edge,
            delta,
            ripple,
        } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let track_id = clip.track_id.clone();
            let anchor = clip.start;
            let old_end = clip.start + clip.duration;
            // Phase 2 (linked captions): the clip's range before the trim.
            // Captions are re-windowed into the range after it below.
            let old_range = (clip.start, old_end);
            // Starts of everything, so a ripple's later clips can carry
            // their captions with them (see `follow_captions`).
            let starts_before: std::collections::HashMap<String, f64> = timeline
                .clips
                .iter()
                .map(|clip| (clip.id.clone(), clip.start))
                .collect();
            // A plain trim stops at the neighbours; a magnetic one moves
            // them instead, so it has the lane's whole length.
            let (floor, ceiling) = if ripple {
                (0.0, f64::INFINITY)
            } else {
                let (before, after) = timeline
                    .room_around(&clip_id)
                    .unwrap_or((0.0, f64::INFINITY));
                (before, after)
            };
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            // What the trim did, and what the lane behind it does about
            // it when it is magnetic: `by` is how far the later clips
            // move, `behind` where "later" begins.
            // `caption_shift` is how far this trim slides the *content*:
            // 0 for an end trim and for a plain head trim (the remaining
            // picture stays put), `-moved` for a ripple head trim (the
            // lane closes, so everything left slides left with it).
            let (applied, by, behind, caption_shift) = match edge {
                TrimEdge::End => {
                    let duration = (clip.duration + delta)
                        .min(ceiling - clip.start)
                        .max(MIN_CLIP_DURATION);
                    let old = clip.duration;
                    let applied = assign(&mut clip.duration, duration);
                    if applied {
                        clip.rewindow_keys(old, 0.0, duration);
                    }
                    (applied, duration - old, old_end - JOIN_EPSILON, 0.0)
                }
                TrimEdge::Start => {
                    // Dragging the head moves the in-point too, so the pixels
                    // under the remaining part of the clip do not slide. The
                    // head cannot reach before the source begins.
                    let shift = delta
                        .min(clip.duration - MIN_CLIP_DURATION)
                        .max(-clip.source_start / clip.speed);
                    // A magnetic head trim never moves the clip, so the
                    // timeline's own start is no limit to it; a plain one
                    // stops at zero.
                    let start = if ripple {
                        clip.start + shift
                    } else {
                        (clip.start + shift).max(floor)
                    };
                    let moved = start - clip.start;
                    let duration = clip.duration - moved;
                    let source_start = (clip.source_start + moved * clip.speed).max(0.0);
                    let old = clip.duration;
                    // Bitwise so no assignment is short-circuited away.
                    let applied = assign(&mut clip.start, start)
                        | assign(&mut clip.duration, duration)
                        | assign(&mut clip.source_start, source_start);
                    if applied {
                        clip.rewindow_keys(old, moved, old);
                    }
                    if ripple {
                        // The in-point moved; the clip stays put, and the
                        // lane behind it closes by what came off the head.
                        clip.start = anchor;
                    }
                    (
                        applied,
                        -moved,
                        anchor + JOIN_EPSILON,
                        if ripple { -moved } else { 0.0 },
                    )
                }
            };
            if ripple && applied && by != 0.0 {
                for other in timeline.clips_where(|other| {
                    other.id != clip_id && other.track_id == track_id && other.start >= behind
                }) {
                    other.start = (other.start + by).max(0.0);
                }
            }
            let mut applied = applied;
            if applied {
                // Phase 2: the trimmed clip's captions are re-windowed into
                // its new range; captions of clips the ripple just moved
                // follow those clips. The trimmed clip itself is excluded
                // from the second step - the first already handled it.
                if let Some(clip) = timeline.clip(&clip_id).cloned() {
                    applied |= rewindow_captions(
                        timeline,
                        &clip_id,
                        old_range,
                        (clip.start, clip.start + clip.duration),
                        caption_shift,
                    );
                }
                let exclude: HashSet<String> = [clip_id.clone()].into_iter().collect();
                applied |= follow_captions(timeline, &starts_before, &exclude, &exclude);
            }
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SplitClips { clip_ids, time } => {
            let timeline = project.active_mut();
            let mut created = None;
            for clip_id in clip_ids {
                let Some(index) = timeline.clips.iter().position(|clip| clip.id == clip_id) else {
                    continue;
                };
                {
                    // A curve does not survive a cut in halves: the map from
                    // here to the source is not affine, so both halves go to
                    // the constant mean, which is what they averaged.
                    let clip = timeline.clip_at_mut(index);
                    let offset = time - clip.start;
                    if offset > MIN_CLIP_DURATION
                        && offset < clip.duration - MIN_CLIP_DURATION
                        && clip.speed_curve.is_some()
                    {
                        clip.speed_curve = None;
                    }
                }
                let clip: &Clip = &timeline.clips[index];
                let offset = time - clip.start;
                if offset <= MIN_CLIP_DURATION || offset >= clip.duration - MIN_CLIP_DURATION {
                    continue;
                }
                let (head_source, tail_source) =
                    split_source(clip.source_start, clip.speed, offset);
                let whole = clip.duration;
                let mut tail = clip.clone();
                tail.id = mint.next("c");
                tail.start = clip.start + offset;
                tail.duration = clip.duration - offset;
                tail.source_start = tail_source;
                // The transition belongs to the cut at the original clip's
                // start, which the head keeps; the way in belongs to the
                // head and the way out to the tail, so neither piece plays
                // an entrance or an exit the whole did not have at the cut.
                tail.transition_in = None;
                tail.fade_in = 0.0;
                tail.rewindow_keys(whole, offset, whole);
                created = Some(tail.id.clone());
                let head = timeline.clip_at_mut(index);
                head.duration = offset;
                head.source_start = head_source;
                head.fade_out = 0.0;
                head.rewindow_keys(whole, 0.0, offset);
                // Phase 2: a caption that starts at or after the cut now
                // belongs to the tail piece - if the tail is later moved,
                // the caption must go with the piece it sits on. A caption
                // straddling the cut stays with the head.
                if let Some(tail_id) = created.clone() {
                    for caption_id in timeline.captions_of(&clip_id) {
                        if let Some(caption) = timeline.clip_mut(&caption_id)
                            && caption.start >= time - JOIN_EPSILON
                        {
                            caption.caption_of = Some(tail_id.clone());
                        }
                    }
                }
                timeline.clips.insert(index + 1, Arc::new(tail));
            }
            // A split always mints the tail, so "minted anything" and
            // "changed anything" are the same fact here.
            let applied = created.is_some();
            Ok(Outcome {
                created_id: created,
                applied,
            })
        }

        Command::ReplaceClipMedia {
            clip_id,
            item,
            source_start,
            enhanced,
        } => {
            if project.active().clip(&clip_id).is_none() {
                return Ok(Outcome::default());
            }
            // The copy's bin entry, or the one already there for its path:
            // two clips of one file enhanced in turn share one copy.
            let media_id = match project.media.iter().find(|media| media.path == item.path) {
                Some(existing) => existing.id.clone(),
                None => {
                    let id = mint.next("m");
                    project.media.push(MediaItem {
                        id: id.clone(),
                        path: item.path,
                        name: item.name,
                        duration: item.duration,
                        kind: item.kind,
                        width: item.width,
                        height: item.height,
                        frame_rate: item.frame_rate,
                        frame_rate_fraction: item.frame_rate_fraction,
                        video_codec: item.video_codec,
                        audio_codec: item.audio_codec,
                        has_audio: item.has_audio,
                        audio_tracks: item.audio_tracks,
                        origin: item.origin,
                        placeholder: false,
                        color_range: None,
                        color_space: item.color_space,
                        extra: Default::default(),
                    });
                    id
                }
            };
            let timeline = project.active_mut();
            let Some(index) = timeline.clips.iter().position(|clip| clip.id == clip_id) else {
                return Ok(Outcome::default());
            };
            let clip = timeline.clip_at_mut(index);
            let link = if enhanced {
                // The original is the one Enhance started from, kept through
                // a second run on a clip already showing a copy.
                let original = clip
                    .enhanced
                    .as_ref()
                    .map(|link| link.original.clone())
                    .unwrap_or_else(|| clip.media_id.clone());
                Some(Enhanced {
                    original,
                    copy: media_id.clone(),
                    on: true,
                })
            } else {
                None
            };
            // Bitwise so no assignment is short-circuited away.
            let applied = assign(&mut clip.media_id, media_id.clone())
                | source_start.is_some_and(|start| assign(&mut clip.source_start, start.max(0.0)))
                | assign(&mut clip.enhanced, link);
            if !applied {
                return Ok(Outcome::default());
            }
            Ok(Outcome {
                created_id: Some(media_id),
                applied: true,
            })
        }

        Command::SetClipEnhanced { clip_id, on } => {
            let timeline = project.active_mut();
            let Some(index) = timeline.clips.iter().position(|clip| clip.id == clip_id) else {
                return Ok(Outcome::default());
            };
            let Some(link) = timeline.clips[index].enhanced.clone() else {
                return Ok(Outcome::default());
            };
            let clip = timeline.clip_at_mut(index);
            let media = if on {
                link.copy.clone()
            } else {
                link.original.clone()
            };
            let applied = assign(&mut clip.media_id, media)
                | assign(&mut clip.enhanced, Some(Enhanced { on, ..link }));
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::FreezeFrame {
            clip_id,
            time,
            duration,
            still,
        } => {
            let hold = duration
                .filter(|value| *value > 0.0)
                .unwrap_or(DEFAULT_FREEZE_DURATION)
                .max(MIN_CLIP_DURATION);

            let (kind, media_id, track_id, start, clip_duration, speed, source_start, picture) = {
                let timeline = project.active();
                let Some(clip) = timeline.clip(&clip_id) else {
                    return Ok(Outcome::default());
                };
                if clip.kind != ClipKind::Video && clip.kind != ClipKind::Image {
                    return Ok(Outcome::default());
                }
                let offset = time - clip.start;
                if offset <= MIN_CLIP_DURATION || offset >= clip.duration - MIN_CLIP_DURATION {
                    return Ok(Outcome::default());
                }
                (
                    clip.kind,
                    clip.media_id.clone(),
                    clip.track_id.clone(),
                    clip.start,
                    clip.duration,
                    clip.speed,
                    clip.source_start,
                    clip.clone(),
                )
            };

            let freeze_media_id = if kind == ClipKind::Image && still.is_none() {
                media_id
            } else {
                let Some(item) = still else {
                    return Ok(Outcome::default());
                };
                if let Some(existing) = project.media.iter().find(|media| media.path == item.path) {
                    existing.id.clone()
                } else {
                    let id = mint.next("m");
                    project.media.push(MediaItem {
                        id: id.clone(),
                        path: item.path,
                        name: item.name,
                        duration: item.duration,
                        kind: MediaKind::Image,
                        width: item.width,
                        height: item.height,
                        frame_rate: item.frame_rate,
                        frame_rate_fraction: item.frame_rate_fraction,
                        video_codec: item.video_codec,
                        audio_codec: None,
                        has_audio: false,
                        audio_tracks: Vec::new(),
                        origin: item.origin,
                        placeholder: false,
                        color_range: None,
                        color_space: item.color_space,
                        extra: Default::default(),
                    });
                    id
                }
            };

            let timeline = project.active_mut();
            let Some(index) = timeline.clips.iter().position(|clip| clip.id == clip_id) else {
                return Ok(Outcome::default());
            };
            // The cut is a split's, so it leaves the pieces as a split does:
            // under a curve the map is not affine, and the in-point below
            // assumes it is, so both pieces go to the constant mean they
            // averaged.
            timeline.clip_at_mut(index).speed_curve = None;
            let offset = time - start;
            let (head_source, tail_source) = split_source(source_start, speed, offset);
            let mut tail = Clip::clone(&timeline.clips[index]);
            tail.id = mint.next("c");
            tail.start = time;
            tail.duration = clip_duration - offset;
            tail.source_start = tail_source;
            tail.transition_in = None;
            tail.fade_in = 0.0;
            tail.rewindow_keys(clip_duration, offset, clip_duration);
            let head = timeline.clip_at_mut(index);
            head.duration = offset;
            head.source_start = head_source;
            head.fade_out = 0.0;
            head.rewindow_keys(clip_duration, 0.0, offset);
            timeline.clips.insert(index + 1, Arc::new(tail));

            // Ripple every later placement on this track (including the new
            // tail) so the freeze does not sit on top of the remainder.
            for clip in timeline.clips_where(|clip| clip.track_id == track_id && clip.start >= time)
            {
                clip.start += hold;
            }

            // The still is the source clip turned into a picture: cloning it
            // first carries every look field - transform, effects, crop,
            // flips, whatever the model grows - and then the hold's own
            // facts overwrite the moving ones.
            let freeze_id = mint.next("c");
            let mut frozen = picture;
            frozen.id = freeze_id.clone();
            frozen.track_id = track_id;
            frozen.kind = ClipKind::Image;
            frozen.media_id = freeze_media_id;
            frozen.start = time;
            frozen.duration = hold;
            frozen.source_start = 0.0;
            frozen.speed = 1.0;
            frozen.speed_curve = None;
            frozen.volume = 1.0;
            frozen.fade_in = 0.0;
            frozen.fade_out = 0.0;
            frozen.filters = Vec::new();
            frozen.muted = None;
            frozen.detached_from = None;
            frozen.transition_in = None;
            frozen.text = None;
            timeline.clips.push(Arc::new(frozen));

            Ok(Outcome {
                created_id: Some(freeze_id),
                applied: true,
            })
        }

        Command::MergeClips { clip_ids } => {
            let timeline = project.active_mut();
            if let Some(reason) = why_not_merge(timeline, &clip_ids) {
                return Err(CommandError::CannotMerge { reason });
            }
            let mut ordered: Vec<Clip> = clip_ids
                .iter()
                .filter_map(|id| timeline.clip(id).cloned())
                .collect();
            ordered.sort_by(|left, right| left.start.total_cmp(&right.start));
            let first = ordered.first().expect("validated above").clone();
            let last = ordered.last().expect("validated above");
            let merged_duration = last.start + last.duration - first.start;

            // A set, not a Vec: the retain below tests every clip on the
            // timeline against it.
            let doomed: HashSet<String> =
                ordered.iter().skip(1).map(|clip| clip.id.clone()).collect();
            timeline.clips.retain(|clip| !doomed.contains(&clip.id));
            let survivor = timeline
                .clip_mut(&first.id)
                .expect("the first piece survives the retain");
            survivor.duration = merged_duration;
            // Every piece's keys land where they were on the picture; the
            // way out is the last piece's, as the way in is the first's.
            survivor.rewindow_keys(first.duration, 0.0, merged_duration);
            for piece in ordered.iter().skip(1) {
                survivor.absorb_keys(piece, piece.start - first.start);
            }
            survivor.fade_out = last.fade_out;
            // A validated merge always absorbs at least one piece.
            Ok(Outcome {
                created_id: Some(first.id),
                applied: true,
            })
        }

        Command::AutotrimClip { clip_id, kept } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip(&clip_id).cloned() else {
                return Ok(Outcome::default());
            };
            let (clip_start, clip_end) = (clip.start, clip.start + clip.duration);

            // The kept ranges, made sane: inside the clip, sorted,
            // overlaps merged, slivers dropped.
            let mut kept: Vec<(f64, f64)> = kept
                .iter()
                .map(|(start, end)| (start.max(clip_start), end.min(clip_end)))
                .filter(|(start, end)| end - start >= MIN_CLIP_DURATION)
                .collect();
            kept.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut merged: Vec<(f64, f64)> = Vec::new();
            for range in kept {
                match merged.last_mut() {
                    Some(last) if range.0 <= last.1 + JOIN_EPSILON => {
                        last.1 = last.1.max(range.1);
                    }
                    _ => merged.push(range),
                }
            }
            let kept_total: f64 = merged.iter().map(|(a, b)| b - a).sum();
            // Keeping everything (or, with nothing sane, nothing) is not
            // an edit: a "trim" that deletes a whole clip is RemoveClips'
            // job, and says so by doing nothing here.
            if merged.is_empty() || (clip.duration - kept_total).abs() < JOIN_EPSILON {
                return Ok(Outcome::default());
            }
            let removed_total = clip.duration - kept_total;

            // Who is rebuilt through the ranges: the clip, the captions
            // linked to it, and the sound detached from it. Everything
            // else either sits before the edit or shifts behind it.
            let mut participants: Vec<String> = vec![clip_id.clone()];
            participants.extend(timeline.captions_of(&clip_id));
            participants.extend(
                timeline
                    .clips
                    .iter()
                    .filter(|other| other.detached_from.as_deref() == Some(clip_id.as_str()))
                    .map(|other| other.id.clone()),
            );

            // Where each kept range lands: the pieces pack back to back
            // from where the clip started.
            let mut landings: Vec<f64> = Vec::with_capacity(merged.len());
            let mut cursor = clip_start;
            for (start, end) in &merged {
                landings.push(cursor);
                cursor += end - start;
            }

            for participant_id in &participants {
                let Some(index) = timeline
                    .clips
                    .iter()
                    .position(|other| &other.id == participant_id)
                else {
                    continue;
                };
                let original = (*timeline.clips[index]).clone();
                let old_duration = original.duration;
                // This participant's pieces: its span through each kept
                // range, in order. A caption clipped by a range keeps
                // its words on every piece it survives as - a text clip
                // holds one string, and word timings are Phase 6's.
                let mut pieces: Vec<Clip> = Vec::new();
                for ((range_start, range_end), lands_at) in merged.iter().zip(&landings) {
                    let start = range_start.max(original.start);
                    let end = range_end.min(original.start + original.duration);
                    if end - start < MIN_CLIP_DURATION {
                        continue;
                    }
                    // The first piece produced keeps the original's id -
                    // whatever links to this clip (a caption's
                    // `caption_of`, a sound's `detached_from`) must still
                    // find it after the rebuild.
                    let first = pieces.is_empty();
                    let mut piece = original.clone();
                    piece.id = if first {
                        original.id.clone()
                    } else {
                        mint.next("c")
                    };
                    let rel = start - original.start;
                    piece.start = lands_at + (start - range_start);
                    piece.duration = end - start;
                    if piece.kind != ClipKind::Text {
                        piece.source_start = original.source_start + rel * original.speed;
                    }
                    if !first {
                        // The way in belongs to the first piece only, as
                        // a split's tail has none.
                        piece.transition_in = None;
                        piece.fade_in = 0.0;
                    }
                    if end < original.start + original.duration - JOIN_EPSILON {
                        piece.fade_out = 0.0;
                    }
                    // A curve does not survive being cut into pieces, as
                    // a split's halves go to the constant mean.
                    if merged.len() > 1 {
                        piece.speed_curve = None;
                    }
                    piece.rewindow_keys(old_duration, rel, rel + piece.duration);
                    // Phase 6: the piece keeps the words it covers,
                    // moved to where the piece landed, and - when the
                    // caption has words - its text becomes exactly
                    // those words, so a caption cut at a silence no
                    // longer repeats its whole sentence on both sides
                    // (the Phase 5 limit, lifted where words exist).
                    if let Some(words) = &original.words {
                        let kept: Vec<crate::captionmap::Word> = words
                            .iter()
                            .filter(|word| word.end > start && word.start < end)
                            .map(|word| crate::captionmap::Word {
                                text: word.text.clone(),
                                start: piece.start + (word.start.max(start) - start),
                                end: piece.start + (word.end.min(end) - start),
                            })
                            .collect();
                        if !kept.is_empty() {
                            if piece.kind == ClipKind::Text
                                && let Some(style) = &mut piece.text
                            {
                                style.content = kept
                                    .iter()
                                    .map(|word| word.text.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" ");
                            }
                            piece.words = Some(kept);
                        } else {
                            piece.words = None;
                        }
                    }
                    pieces.push(piece);
                }
                timeline.clips.remove(index);
                for (offset, piece) in pieces.into_iter().enumerate() {
                    timeline.clips.insert(index + offset, Arc::new(piece));
                }
            }

            // The timeline closes up: whatever began at or after the
            // clip's old end moves left by what came out of it.
            for other in timeline.clips_where(|other| {
                !participants.contains(&other.id) && other.start >= clip_end - JOIN_EPSILON
            }) {
                other.start = (other.start - removed_total).max(0.0);
            }
            Ok(Outcome {
                created_id: None,
                applied: true,
            })
        }

        Command::RemoveClips { clip_ids, ripple } => {
            let timeline = project.active_mut();
            // Phase 2: starts before the delete, so captions of clips the
            // ripple pulls left can follow them. The captions of a *removed*
            // clip are NOT deleted or rewritten here - that contract is
            // pinned by a Phase 1 test; their links simply dangle until the
            // caller clears them via `captions_with_no_clip`.
            let starts_before: std::collections::HashMap<String, f64> = timeline
                .clips
                .iter()
                .map(|clip| (clip.id.clone(), clip.start))
                .collect();
            let doomed: HashSet<&str> = clip_ids.iter().map(String::as_str).collect();
            // The spans going, per track, taken before they are gone: the
            // ripple closes exactly these.
            let removed: Vec<(String, f64, f64)> = timeline
                .clips
                .iter()
                .filter(|clip| doomed.contains(clip.id.as_str()))
                .map(|clip| {
                    (
                        clip.track_id.clone(),
                        clip.start,
                        clip.start + clip.duration,
                    )
                })
                .collect();
            let clip_count = timeline.clips.len();
            timeline
                .clips
                .retain(|clip| !doomed.contains(clip.id.as_str()));
            let applied = timeline.clips.len() != clip_count;
            if ripple && applied {
                close_gaps(timeline, &removed);
                follow_captions(timeline, &starts_before, &HashSet::new(), &HashSet::new());
            }
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        _ => unreachable!("commands::apply routes only this module's commands here"),
    }
}

/// How far apart two clips may sit and still count as touching, in seconds.
const JOIN_EPSILON: f64 = 1e-6;

/// Phase 2 (linked captions): shift every caption whose source clip moved.
///
/// `before` maps clip id -> start before the edit. For each clip that
/// actually moved, its captions (see [`Timeline::captions_of`]) are shifted
/// by the same delta - unless the caption itself already moved by that
/// edit (it sits on a rippled track, say) or its id is in `exclude` (it was
/// dragged explicitly, or already re-windowed by a trim). Returns whether
/// any caption changed. One helper for move, ripple-trim and ripple-delete,
/// because "the words stay over the picture they belong to" is one rule.
fn follow_captions(
    timeline: &mut Timeline,
    before: &std::collections::HashMap<String, f64>,
    skip_sources: &HashSet<String>,
    skip_captions: &HashSet<String>,
) -> bool {
    // Collect first, mutate second: shifting while iterating `clips` would
    // fight the borrow checker and double-count a caption whose source is
    // itself a moved caption's neighbour.
    let mut shifts: Vec<(String, f64)> = Vec::new();
    for clip in timeline.clips.iter() {
        let Some(was) = before.get(&clip.id) else {
            continue;
        };
        let delta = clip.start - was;
        if delta == 0.0 || skip_sources.contains(&clip.id) {
            continue;
        }
        for caption_id in timeline.captions_of(&clip.id) {
            if skip_captions.contains(&caption_id) {
                continue;
            }
            // A caption the edit already moved (its own start differs from
            // `before`) must not be shifted a second time.
            let already_moved = timeline.clip(&caption_id).is_some_and(|caption| {
                before
                    .get(&caption_id)
                    .is_some_and(|was| caption.start != *was)
            });
            if !already_moved {
                shifts.push((caption_id, delta));
            }
        }
    }
    let mut applied = false;
    for (caption_id, delta) in shifts {
        if let Some(caption) = timeline.clip_mut(&caption_id) {
            let start = (caption.start + delta).max(0.0);
            applied |= assign(&mut caption.start, start);
            // The words ride the same shift: their clock is the timeline.
            if let Some(words) = &mut caption.words {
                for word in words.iter_mut() {
                    word.start = (word.start + delta).max(0.0);
                    word.end = (word.end + delta).max(0.0);
                }
            }
        }
    }
    applied
}

/// Phase 2 (linked captions): re-window one clip's captions after a trim.
///
/// Each caption is shifted by `shift` (how far the trim slid the content),
/// then intersected with the clip's new range. A caption partly trimmed is
/// clamped; a caption wholly trimmed away is removed - it captioned source
/// that is no longer on the timeline, and leaving it behind would strand
/// words over somebody else's picture. This runs inside the trim command,
/// so the whole thing is still one undo step. Returns whether anything
/// changed.
fn rewindow_captions(
    timeline: &mut Timeline,
    clip_id: &str,
    _old_range: (f64, f64),
    new_range: (f64, f64),
    shift: f64,
) -> bool {
    let mut applied = false;
    let mut doomed: Vec<String> = Vec::new();
    for caption_id in timeline.captions_of(clip_id) {
        let Some(caption) = timeline.clip_mut(&caption_id) else {
            continue;
        };
        let start = (caption.start + shift).max(new_range.0);
        let end = (caption.start + caption.duration + shift).min(new_range.1);
        if end - start < MIN_CLIP_DURATION {
            doomed.push(caption_id);
            continue;
        }
        applied |= assign(&mut caption.start, start);
        applied |= assign(&mut caption.duration, end - start);
        // The words are clipped at the same edges: shifted with the
        // content, kept only where they still fall inside the caption,
        // and a word straddling the edge is heard on the kept side.
        if let Some(words) = &mut caption.words {
            words.retain(|word| word.end + shift > start && word.start + shift < end);
            for word in words.iter_mut() {
                word.start = (word.start + shift).max(start);
                word.end = (word.end + shift).min(end);
            }
            if words.is_empty() {
                caption.words = None;
            }
        }
    }
    if !doomed.is_empty() {
        timeline.clips.retain(|clip| !doomed.contains(&clip.id));
        applied = true;
    }
    applied
}

/// The ripple of a delete: pulls every clip that sits after a removed
/// span left by the length of the removed spans before it, on that clip's
/// own track. Spans that overlap count once - a clip behind two doomed
/// clips that shared five seconds moves by their union, not their sum, or
/// it would land in front of what was in front of it. A span reaching past
/// the clip's start counts only up to it, and nothing is pulled before
/// zero.
/// https://github.com/jub0t/Concat/issues/106
fn close_gaps(timeline: &mut Timeline, removed: &[(String, f64, f64)]) {
    let behind_a_span = |clip: &Clip| {
        removed
            .iter()
            .any(|(track, start, _)| *track == clip.track_id && *start < clip.start)
    };
    for clip in timeline.clips_where(behind_a_span) {
        let mut spans: Vec<(f64, f64)> = removed
            .iter()
            .filter(|(track, start, _)| *track == clip.track_id && *start < clip.start)
            .map(|(_, start, end)| (*start, end.min(clip.start)))
            .collect();
        if spans.is_empty() {
            continue;
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut gap = 0.0;
        let (mut from, mut to) = spans[0];
        for (start, end) in spans.into_iter().skip(1) {
            if start <= to {
                to = to.max(end);
            } else {
                gap += to - from;
                from = start;
                to = end;
            }
        }
        gap += to - from;
        clip.start = (clip.start - gap).max(0.0);
    }
}

fn default_clip(id: String, track_id: String, media: &MediaItem, start: f64) -> Clip {
    let kind = match media.kind {
        MediaKind::Video => ClipKind::Video,
        MediaKind::Audio => ClipKind::Audio,
        MediaKind::Image => ClipKind::Image,
    };
    let mut clip = Clip::blank(
        id,
        track_id,
        kind,
        media.name.clone(),
        start,
        clip_length(media),
    );
    clip.media_id = media.id.clone();
    clip
}

/// Makes room on `track_id` for a new clip at `start`: a drop onto the
/// middle of a clip lands at that clip's end instead, and every clip at
/// or after the place it lands moves right by the new clip's length, so
/// the new clip slots in and nothing is covered (#129). Returns where the
/// new clip lands. A drop with room to spare changes nothing.
fn ripple_room_for(timeline: &mut Timeline, track_id: &str, start: f64, media: &MediaItem) -> f64 {
    let duration = clip_length(media);
    // Dropped onto a clip: after it, rather than over it or through it.
    let start = timeline
        .clips
        .iter()
        .filter(|clip| {
            clip.track_id == track_id && clip.start < start && start < clip.start + clip.duration
        })
        .map(|clip| clip.start + clip.duration)
        .fold(start, f64::max);
    let end = start + duration;
    let overlaps = timeline.clips.iter().any(|clip| {
        clip.track_id == track_id && clip.start < end && start < clip.start + clip.duration
    });
    if !overlaps {
        return start;
    }
    for clip in timeline.clips_where(|clip| clip.track_id == track_id && clip.start >= start) {
        clip.start += duration;
    }
    start
}

/// How long a new clip of `media` is: a still's default hold, or the
/// file's own length.
fn clip_length(media: &MediaItem) -> f64 {
    match media.kind {
        MediaKind::Image => DEFAULT_IMAGE_DURATION,
        _ => media.duration.unwrap_or(UNKNOWN_DURATION),
    }
}

/// The lowest track with nothing occupying `[start, start + duration)`.
/// `None` when every lane is taken there: the caller opens a new one
/// rather than covering a clip.
fn first_free_track(timeline: &Timeline, start: f64, duration: f64) -> Option<String> {
    timeline
        .tracks
        .iter()
        .find(|track| timeline.is_free(&track.id, start, duration, &[]))
        .map(|track| track.id.clone())
}

/// A new, empty lane at the top, for a clip with nowhere else to go.
fn fresh_track(timeline: &mut Timeline, mint: &mut IdMint) -> String {
    let id = mint.next("t");
    timeline.tracks.push(Track {
        id: id.clone(),
        visible: true,
        muted: false,
        extra: Default::default(),
    });
    id
}

/// First free lane *above* the highest one occupied over `[start, start +
/// duration)`. `None` when every lane above is taken, which is the caller's
/// cue to mint a new one at the top.
///
/// Captions and titles go through this so they sit over the video, not
/// under it. The plain `first_free_track` still walks from the bottom,
/// which is what a sound or a picture wants.
fn first_free_track_above(timeline: &Timeline, start: f64, duration: f64) -> Option<String> {
    let end = start + duration;
    let occupied = |track_id: &str| {
        timeline.clips.iter().any(|clip| {
            clip.track_id == track_id && clip.start < end && start < clip.start + clip.duration
        })
    };
    let floor = timeline
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| occupied(&track.id))
        .map(|(row, _)| row + 1)
        .max()
        .unwrap_or(0);
    timeline
        .tracks
        .iter()
        .skip(floor)
        .find(|track| !occupied(&track.id))
        .map(|track| track.id.clone())
}

/// Where each piece of a clip cut at `offset` begins in the source: the
/// head keeps its in-point and the tail starts `offset × speed` later, so
/// the two pieces together show exactly what the whole did.
fn split_source(source_start: f64, speed: f64, offset: f64) -> (f64, f64) {
    (source_start, source_start + offset * speed)
}

/// Why these clips cannot be merged, or None if they can. A sentence, because
/// a disabled button that will not say why is worse than no button.
pub fn why_not_merge(timeline: &Timeline, clip_ids: &[String]) -> Option<String> {
    if clip_ids.len() < 2 {
        return Some("Select two or more clips to merge.".to_owned());
    }
    let clips: Vec<&Clip> = clip_ids.iter().filter_map(|id| timeline.clip(id)).collect();
    if clips.len() < 2 {
        return Some("Select two or more clips to merge.".to_owned());
    }
    if clips.iter().any(|clip| clip.track_id != clips[0].track_id) {
        return Some("Merged clips must be on the same track.".to_owned());
    }
    if clips.iter().any(|clip| clip.media_id != clips[0].media_id) {
        return Some("Merged clips must come from the same file.".to_owned());
    }
    if clips.iter().any(|clip| clip.speed != clips[0].speed) {
        return Some("Merged clips must play at the same speed.".to_owned());
    }
    if clips.iter().any(|clip| clip.kind != clips[0].kind) {
        return Some("Merged clips must be the same kind.".to_owned());
    }
    if clips.iter().any(|clip| clip.speed_curve.is_some()) {
        return Some("A clip with a speed curve cannot be merged.".to_owned());
    }
    if clips
        .iter()
        .any(|clip| clip.audio_stream != clips[0].audio_stream)
    {
        return Some("Merged clips must play the same audio track.".to_owned());
    }

    let mut ordered = clips.clone();
    ordered.sort_by(|left, right| left.start.total_cmp(&right.start));
    for pair in ordered.windows(2) {
        let (previous, current) = (pair[0], pair[1]);
        if (current.start - (previous.start + previous.duration)).abs() > JOIN_EPSILON {
            return Some("Merged clips must touch, with no gap or overlap.".to_owned());
        }
        // The next piece starts where the last one's source ended.
        let continuous =
            current.source_start - (previous.source_start + previous.duration * previous.speed);
        if continuous.abs() > JOIN_EPSILON {
            return Some("These pieces are no longer in their original order.".to_owned());
        }
    }
    None
}
