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
            // Generated/imported captions may overlap one another on one
            // caption lane. Hand-made text remains an ordinary title, which
            // keeps the timeline's usual no-overlap placement behaviour.
            // This is the birth of the clip's identity, the one place the
            // inputs are read to make it (T5): from here on the stored flag
            // decides, and `Clip::is_caption` is the only reader.
            let is_caption = caption_of.is_some() || words.is_some();
            let timeline = project.active_mut();
            let track_id = match track_id {
                Some(ref id) if id == "__top__" => timeline
                    .tracks
                    .last()
                    .map(|t| t.id.clone())
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
                Some(id) if timeline.track(&id).is_some() => id,
                Some(_) => return Err(CommandError::TrackGone),
                // Captions choose the first overlay lane without an ordinary
                // clip over this span, so an overlapping imported cue remains
                // on the same subtitle lane rather than being shifted or
                // forced onto a fresh track.
                None if is_caption && above => first_caption_track_above(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
                None if is_caption => first_caption_track(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
                // Every lane above the video is taken: a new one at the top
                // for the words to land on.
                None if above => first_free_track_above(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
                None => first_free_track(timeline, start, duration)
                    .unwrap_or_else(|| fresh_track(timeline, mint)),
            };
            // The lane's stored nudge applies at birth (T6): a regenerated
            // caption run lands already corrected, because the correction
            // belongs to the track, not to the run that was removed.
            let nudge = if is_caption {
                f64::from(
                    timeline
                        .track(&track_id)
                        .map_or(0, |track| track.caption_nudge_ms),
                ) / 1000.0
            } else {
                0.0
            };
            let start = if is_caption {
                timeline.nearest_caption_start(&track_id, (start + nudge).max(0.0), duration, &[])
            } else {
                timeline.nearest_free_start(&track_id, start, duration, &[])
            };
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
            // The lane's nudge shifts the word timings with the cue, so the
            // highlight stays on the text it belongs to.
            clip.words = words.map(|mut words| {
                if nudge != 0.0 {
                    for word in &mut words {
                        word.start = (word.start + nudge).max(0.0);
                        word.end = (word.end + nudge).max(word.start);
                    }
                }
                words
            });
            clip.caption_flag = Some(is_caption);
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
            // An ordinary clip goes to the nearest gap that holds it. Two
            // captions are the intentional exception: they can land on top
            // of one another on a caption lane, as overlapping subtitle cues
            // are valid and must retain their original timings.
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

        Command::MoveCaptionWordEdge {
            clip_id,
            word_index,
            edge,
            time,
        } => {
            let refuse = |reason: &str| -> Result<Outcome, CommandError> {
                Err(CommandError::CannotMoveWordEdge {
                    reason: reason.to_owned(),
                })
            };
            // The drag lands on the millisecond grid the word timings
            // already live on, so a saved document never carries a
            // finer number than the one next to it.
            let time = crate::sections::round_millis(time);
            let timeline = project.active_mut();
            let Some(index) = timeline.clips.iter().position(|clip| clip.id == clip_id) else {
                return refuse("That caption no longer exists.");
            };
            let clip = Arc::make_mut(&mut timeline.clips[index]);
            if !clip.is_caption() {
                return refuse("Only a caption has heard words to drag.");
            }
            let cue_start = clip.start;
            let cue_end = clip.start + clip.duration;
            let Some(words) = clip.words.as_mut().filter(|words| !words.is_empty()) else {
                return refuse(
                    "This caption has no heard word timings, so there is no edge to drag.",
                );
            };
            if words.get(word_index).is_none() {
                return refuse("That word is past the end of the caption.");
            }
            // The drag's kindness rule: the edge is clamped, never
            // refused. It stops at its neighbours - or at the cue's
            // window for the first and last word, so a word is never
            // heard outside its own caption - and it stops leaving the
            // word its minimum width. No drag can overlap the heard
            // order or thin a word out of existence.
            let (lower, upper) = match edge {
                WordEdge::Start => {
                    let lower = if word_index > 0 {
                        words[word_index - 1].end
                    } else {
                        cue_start
                    };
                    (lower, words[word_index].end - crate::captionmap::MIN_WORD)
                }
                WordEdge::End => {
                    let upper = if word_index + 1 < words.len() {
                        words[word_index + 1].start
                    } else {
                        cue_end
                    };
                    (words[word_index].start + crate::captionmap::MIN_WORD, upper)
                }
            };
            // A pathological input - a word already thinner than the
            // minimum, imported without a clean - must not invert the
            // clamp and panic: the bounds simply meet.
            let upper = upper.max(lower);
            let wanted = time.clamp(lower, upper);
            let moved = match edge {
                WordEdge::Start => (words[word_index].start - wanted).abs() > 1e-9,
                WordEdge::End => (words[word_index].end - wanted).abs() > 1e-9,
            };
            match edge {
                WordEdge::Start => words[word_index].start = wanted,
                WordEdge::End => words[word_index].end = wanted,
            }
            // A drag that lands where it started applied nothing, and
            // the editor records no undo step for nothing.
            Ok(Outcome {
                created_id: None,
                applied: moved,
            })
        }

        Command::DeleteCaptionWord {
            clip_id,
            word_index,
            cut_video,
        } => {
            let refuse = |reason: &str| -> Result<Outcome, CommandError> {
                Err(CommandError::CannotDeleteWord {
                    reason: reason.to_owned(),
                })
            };
            // Everything the two paths need is read off the whole state
            // first, as owned values, before anything is written.
            let (word_span, owner_id, new_content) = {
                let timeline = project.active();
                let Some(clip) = timeline.clip(&clip_id) else {
                    return refuse("That caption no longer exists.");
                };
                if !clip.is_caption() {
                    return refuse("Only a caption has heard words to delete.");
                }
                let Some(words) = clip.words.as_ref().filter(|words| !words.is_empty()) else {
                    return refuse(
                        "This caption has no heard word timings, so there is no word to delete.",
                    );
                };
                let Some(target) = words.get(word_index) else {
                    return refuse("That word is past the end of the caption.");
                };
                if cut_video {
                    // The clip whose time gets cut: the one the caption
                    // follows, or - an imported cue may follow nothing - the
                    // first picture or sound the word is heard over.
                    let owner = clip
                        .caption_of
                        .as_deref()
                        .and_then(|id| timeline.clip(id).map(|owner| owner.id.clone()))
                        .or_else(|| {
                            timeline
                                .clips
                                .iter()
                                .find(|other| {
                                    !other.is_caption()
                                        && other.start < target.end - JOIN_EPSILON
                                        && other.start + other.duration
                                            > target.start + JOIN_EPSILON
                                })
                                .map(|other| other.id.clone())
                        });
                    let Some(owner) = owner else {
                        return refuse(
                            "There is no picture or sound under this word, so there is nothing to cut.",
                        );
                    };
                    ((target.start, target.end), Some(owner), None)
                } else {
                    // Text only: the word comes out of the content at its
                    // located bytes, so spacing and punctuation stay tidy,
                    // and the single writer (T7) prunes its timing slot.
                    let Some(style) = clip.text.as_ref() else {
                        return refuse("This caption has no text to delete from.");
                    };
                    let Some(range) =
                        crate::captionmap::locate_words(&style.content, words)[word_index].clone()
                    else {
                        return refuse(
                            "The word is not in the caption's text, so there is nothing to delete.",
                        );
                    };
                    let head = style.content[..range.start].trim_end();
                    let tail = style.content[range.end..].trim_start();
                    let content = if head.is_empty() {
                        tail.to_owned()
                    } else if tail.is_empty() {
                        head.to_owned()
                    } else {
                        format!("{head} {tail}")
                    };
                    ((target.start, target.end), None, Some(content))
                }
            };

            if let Some(content) = new_content {
                let timeline = project.active_mut();
                let Some(clip) = timeline.clip_mut(&clip_id) else {
                    return refuse("That caption no longer exists.");
                };
                if let Some(style) = clip.text.as_mut() {
                    style.content = content.clone();
                }
                clip.name = first_line(&content);
                super::properties::reconcile_caption_words(clip);
                return Ok(Outcome {
                    created_id: None,
                    applied: true,
                });
            }

            // The cut path: the word's heard span comes out of the clip it
            // follows, and AutotrimClip is the machine that cuts every lane
            // at the same moments, closes the gap, and rebuilds the captions
            // and their links over it - the remove-selection flow's own
            // machinery, so a word delete ripples exactly like a selection
            // delete (owner decision: the gap closes).
            let owner_id = owner_id.expect("the cut path read an owner");
            let kept = {
                let owner = project
                    .active()
                    .clip(&owner_id)
                    .expect("the owner was just read");
                let owner_end = owner.start + owner.duration;
                let mut kept = Vec::new();
                if word_span.0 > owner.start {
                    kept.push((owner.start, word_span.0.min(owner_end)));
                }
                if word_span.1 < owner_end {
                    kept.push((word_span.1.max(owner.start), owner_end));
                }
                kept
            };
            let outcome = apply(
                project,
                mint,
                Command::AutotrimClip {
                    clip_id: owner_id,
                    kept,
                },
            )?;
            if !outcome.applied {
                return refuse(
                    "The word's timing does not sit inside the clip it follows, so there is nothing to cut.",
                );
            }
            // The husk rule: a caption whose last heard word was just cut
            // keeps no wordless shell. Its first piece holds the original
            // id, so that is the one to look at; the sliver logic in
            // AutotrimClip already dropped the other wordless pieces.
            let timeline = project.active_mut();
            if let Some(index) = timeline.clips.iter().position(|clip| clip.id == clip_id)
                && timeline.clips[index].is_caption()
                && timeline.clips[index].words.is_none()
            {
                timeline.clips.remove(index);
            }
            Ok(outcome)
        }

        Command::RemoveFillerWords {
            clip_ids,
            strong,
            weak,
            cut_video,
        } => {
            // The read pass: what the ticked tiers actually hit, per caption.
            // A selection holds what the user marked, not only cues, so
            // picture clips and unheard captions are skipped, not errors.
            let mut targets: Vec<(String, Vec<usize>)> = Vec::new();
            {
                let timeline = project.active();
                for id in clip_ids {
                    let Some(clip) = timeline.clip(&id) else {
                        continue;
                    };
                    if !clip.is_caption() {
                        continue;
                    }
                    let Some(words) = clip.words.as_ref().filter(|words| !words.is_empty()) else {
                        continue;
                    };
                    let indices = crate::filler::filler_indices(words, strong, weak);
                    if !indices.is_empty() {
                        targets.push((id.clone(), indices));
                    }
                }
            }
            if targets.is_empty() {
                // A pass that hits nothing is not an edit: the undo history
                // keeps no nothing-snapshot.
                return Ok(Outcome::default());
            }

            if !cut_video {
                // Text only: the words leave the content at their located
                // bytes and their timing slots go with them, written
                // together (T7's single-writer rule); every survivor keeps
                // its heard time and the picture stands still.
                let timeline = project.active_mut();
                for (id, indices) in &targets {
                    let Some(clip) = timeline.clip_mut(id) else {
                        continue;
                    };
                    let (content, kept) = {
                        let Some(style) = clip.text.as_ref() else {
                            continue;
                        };
                        let Some(words) = clip.words.as_ref() else {
                            continue;
                        };
                        let filler: std::collections::HashSet<usize> =
                            indices.iter().copied().collect();
                        let located = crate::captionmap::locate_words(&style.content, words);
                        let mut out = String::new();
                        let mut cursor = 0usize;
                        for (index, range) in located.iter().enumerate() {
                            if filler.contains(&index)
                                && let Some(range) = range
                            {
                                out.push_str(&style.content[cursor..range.start]);
                                cursor = range.end;
                            }
                        }
                        out.push_str(&style.content[cursor..]);
                        let content = out.split_whitespace().collect::<Vec<_>>().join(" ");
                        let kept: Vec<crate::captionmap::Word> = words
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| !filler.contains(index))
                            .map(|(_, word)| word.clone())
                            .collect();
                        (content, kept)
                    };
                    if let Some(style) = clip.text.as_mut() {
                        style.content = content.clone();
                    }
                    clip.name = first_line(&content);
                    clip.words = Some(kept);
                }
                return Ok(Outcome {
                    created_id: None,
                    applied: true,
                });
            }

            // The cut path: every filler's heard span comes out of the clip
            // it rides, through the remove-selection machinery (E1's path).
            let mut spans: Vec<(f64, f64)> = Vec::new();
            let mut owners: Vec<(f64, String)> = Vec::new();
            let mut affected: Vec<String> = Vec::new();
            let before_ids: std::collections::HashSet<String> = {
                let timeline = project.active();
                timeline.clips.iter().map(|clip| clip.id.clone()).collect()
            };
            {
                let timeline = project.active();
                for (id, indices) in &targets {
                    let clip = timeline.clip(id).expect("read above");
                    let words = clip.words.as_ref().expect("read above");
                    for &index in indices {
                        spans.push((words[index].start, words[index].end));
                    }
                    affected.push(id.clone());
                    // The clip whose time gets cut: the one the caption
                    // follows, else the first picture or sound any of its
                    // filler words is heard over.
                    let owner = clip
                        .caption_of
                        .as_deref()
                        .and_then(|owner| timeline.clip(owner))
                        .map(|owner| (owner.start, owner.id.clone()))
                        .or_else(|| {
                            indices.iter().find_map(|&index| {
                                timeline.clips.iter().find_map(|other| {
                                    (!other.is_caption()
                                        && other.start < words[index].end - JOIN_EPSILON
                                        && other.start + other.duration
                                            > words[index].start + JOIN_EPSILON)
                                        .then(|| (other.start, other.id.clone()))
                                })
                            })
                        });
                    let Some(owner) = owner else {
                        return Err(CommandError::CannotDeleteWord {
                            reason: "There is no picture or sound under a filler word, so there is nothing to cut.".to_owned(),
                        });
                    };
                    owners.push(owner);
                }
                // An unselected caption riding the same cut is rebuilt by it
                // too, so it is a husk candidate as well - but only one that
                // carried heard words: an imported cue never had any, and
                // must survive the cut it merely rode.
                for clip in &timeline.clips {
                    if clip.is_caption()
                        && clip.words.is_some()
                        && spans.iter().any(|&(start, end)| {
                            clip.start < end - JOIN_EPSILON
                                && clip.start + clip.duration > start + JOIN_EPSILON
                        })
                    {
                        affected.push(clip.id.clone());
                    }
                }
            }
            spans.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut merged: Vec<(f64, f64)> = Vec::new();
            for span in spans {
                match merged.last_mut() {
                    Some(last) if span.0 <= last.1 + JOIN_EPSILON => {
                        last.1 = last.1.max(span.1);
                    }
                    _ => merged.push(span),
                }
            }
            owners.sort_by(|a, b| a.0.total_cmp(&b.0));
            owners.dedup_by(|a, b| a.1 == b.1);

            // One autotrim per owner, left to right: kept = the owner's span
            // minus the filler spans inside it. Every lane is cut at the
            // same moments and the gaps close, exactly as a selection
            // delete does.
            for (_, owner_id) in &owners {
                let kept = {
                    let timeline = project.active();
                    let Some(owner) = timeline.clip(owner_id) else {
                        continue;
                    };
                    let owner_end = owner.start + owner.duration;
                    let mut kept = Vec::new();
                    let mut cursor = owner.start;
                    for &(start, end) in &merged {
                        if end <= owner.start + JOIN_EPSILON || start >= owner_end - JOIN_EPSILON {
                            continue;
                        }
                        let start = start.max(cursor);
                        let end = end.min(owner_end);
                        if start - cursor >= MIN_CLIP_DURATION {
                            kept.push((cursor, start));
                        }
                        cursor = cursor.max(end);
                    }
                    if owner_end - cursor >= MIN_CLIP_DURATION {
                        kept.push((cursor, owner_end));
                    }
                    kept
                };
                apply(
                    project,
                    mint,
                    Command::AutotrimClip {
                        clip_id: owner_id.clone(),
                        kept,
                    },
                )?;
            }

            // The husk sweep, scoped so it can only touch what this pass
            // rebuilt: a caption that carried heard words and now has none
            // goes (its first rebuilt piece keeps the original id), and a
            // piece born by this cut with empty text goes - empty text
            // shows nothing and highlights nothing. Captions that existed
            // before keep whatever they had.
            let timeline = project.active_mut();
            timeline.clips.retain(|clip| {
                if !clip.is_caption() {
                    return true;
                }
                if !before_ids.contains(&clip.id)
                    && clip
                        .text
                        .as_ref()
                        .is_some_and(|style| style.content.trim().is_empty())
                {
                    return false;
                }
                !(affected.contains(&clip.id) && clip.words.is_none())
            });
            Ok(Outcome {
                created_id: None,
                applied: true,
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

        Command::SplitCaptionAtWord {
            clip_id,
            word_index,
        } => {
            let refuse = |reason: &str| -> Result<Outcome, CommandError> {
                Err(CommandError::CannotSplitCaption {
                    reason: reason.to_owned(),
                })
            };
            let timeline = project.active_mut();
            let Some(index) = timeline.clips.iter().position(|clip| clip.id == clip_id) else {
                return refuse("That caption no longer exists.");
            };
            // Everything the two halves need is read off the whole clip
            // first, as owned values, before either piece is written.
            let clip: &Clip = &timeline.clips[index];
            if !clip.is_caption() {
                return refuse("Only a caption can split at a heard word.");
            }
            let Some(words) = clip.words.as_ref().filter(|words| !words.is_empty()) else {
                return refuse(
                    "This caption has no heard word timings, so there is nothing to split at.",
                );
            };
            if word_index == 0 || word_index >= words.len() {
                return refuse(
                    "The caret leaves no word on one side, so there is no boundary to split at.",
                );
            }
            let Some(style) = clip.text.as_ref() else {
                return refuse("This caption has no text to split.");
            };
            let Some(range) =
                crate::captionmap::locate_words(&style.content, words)[word_index].clone()
            else {
                return refuse(
                    "The word is not in the caption's text, so there is no boundary to split at.",
                );
            };
            // The cut lands on the word's heard start (T2/T3 timing), never
            // on a text-length estimate, and the content splits at the byte
            // where that word sits, so both halves keep the original
            // punctuation and spacing.
            let boundary = words[word_index].start;
            let whole = clip.duration;
            let head_duration = boundary - clip.start;
            let tail_duration = clip.start + whole - boundary;
            if head_duration < MIN_CLIP_DURATION || tail_duration < MIN_CLIP_DURATION {
                return refuse(
                    "The word sits too close to the caption's edge to leave two pieces.",
                );
            }
            let head_text = style.content[..range.start].trim_end().to_owned();
            let mut tail_style = style.clone();
            tail_style.content = style.content[range.start..].trim_start().to_owned();
            let head_words = words[..word_index].to_vec();
            let tail_words = words[word_index..].to_vec();
            let owner = clip.caption_of.clone();
            let offset = boundary - clip.start;
            // The tail is born as Enter needs it: the same lane and look, at
            // the boundary, with the words from the caret on - their heard
            // times stand as they are. The lane's nudge is not applied again:
            // the boundary and the words already carry it (T6).
            let mut tail = (*clip).clone();
            tail.id = mint.next("c");
            tail.start = boundary;
            tail.duration = tail_duration;
            tail.name = first_line(&tail_style.content);
            tail.text = Some(tail_style);
            tail.words = Some(tail_words);
            tail.caption_of = owner;
            tail.caption_flag = Some(true);
            // The transition belongs to the original start, which the head
            // keeps; the way in belongs to the head and the way out to the
            // tail (the SplitClips rule).
            tail.transition_in = None;
            tail.fade_in = 0.0;
            tail.rewindow_keys(whole, offset, whole);
            let tail_id = tail.id.clone();
            let head = timeline.clip_at_mut(index);
            if let Some(style) = head.text.as_mut() {
                style.content = head_text.clone();
            }
            head.name = first_line(&head_text);
            head.duration = head_duration;
            head.words = Some(head_words);
            head.fade_out = 0.0;
            head.rewindow_keys(whole, 0.0, head_duration);
            timeline.clips.insert(index + 1, Arc::new(tail));
            Ok(Outcome {
                created_id: Some(tail_id),
                applied: true,
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
            // Nothing given at all removes the whole clip. A list that was
            // given but holds nothing usable is not an edit, as before.
            let remove_all = kept.is_empty();

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
            // Keeping everything is not an edit; a trim that deletes a whole
            // clip is the remove_all case, not a no-op.
            if !remove_all
                && (merged.is_empty() || (clip.duration - kept_total).abs() < JOIN_EPSILON)
            {
                return Ok(Outcome::default());
            }
            let removed_total = clip.duration - kept_total;

            // Who is rebuilt through the ranges: the clip, and every other clip
            // on any lane that overlaps its span - the sound detached from it,
            // the captions over it, and whatever sits under the picture on
            // other lanes. Each lane is cut at the same moments as the picture;
            // a lane left at its old length drifts out of step with it.
            let mut participants: Vec<String> = vec![clip_id.clone()];
            for other in &timeline.clips {
                if other.id == clip_id {
                    continue;
                }
                let overlaps_window = other.start < clip_end - JOIN_EPSILON
                    && other.start + other.duration > clip_start + JOIN_EPSILON;
                if overlaps_window {
                    participants.push(other.id.clone());
                }
            }

            // Where time lands. Each segment is (range start, range end, shift):
            // the time before the clip stays where it is; each kept range moves
            // by `shift` to where it packs back to; the time after the clip
            // moves by what came out. A participant that reaches past the
            // clip keeps its own time outside it, which is what stops a bed
            // or a voice-over losing its tail.
            let mut segments: Vec<(f64, f64, f64)> = Vec::with_capacity(merged.len() + 2);
            segments.push((f64::NEG_INFINITY, clip_start, 0.0));
            let mut cursor = clip_start;
            for (start, end) in &merged {
                segments.push((*start, *end, cursor - start));
                cursor += end - start;
            }
            segments.push((clip_end, f64::INFINITY, -removed_total));

            // A rebuilt caption or detached-audio piece must point at the
            // matching rebuilt source piece, not always at the head. Without
            // this map, a caption over a tail created by Remove selected
            // still names the old head, then stops following the tail on the
            // next drag, trim or export.
            let mut rebuilt_pieces: HashMap<String, Vec<(f64, f64, String)>> = HashMap::new();
            let mut original_span_of_piece: HashMap<String, (f64, f64)> = HashMap::new();

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
                let (orig_start, orig_end) = (original.start, original.start + original.duration);
                // The segments this clip reaches into, clipped to its span:
                // (start, end, shift) in its own time.
                let reach: Vec<(f64, f64, f64)> = segments
                    .iter()
                    .filter(|&&(rs, re, _)| {
                        re.min(orig_end) - rs.max(orig_start) >= MIN_CLIP_DURATION
                    })
                    .map(|&(rs, re, shift)| (rs.max(orig_start), re.min(orig_end), shift))
                    .collect();
                let surviving_ranges = reach.len();
                let any_range_has_words = original.words.as_ref().is_some_and(|words| {
                    reach
                        .iter()
                        .any(|&(s, e, _)| words.iter().any(|w| w.end > s && w.start < e))
                });
                // This participant's pieces: its span through each segment,
                // in order. A caption clipped by a range keeps its words on
                // every piece it survives as - a text clip holds one string,
                // and word timings are Phase 6's.
                let mut pieces: Vec<Clip> = Vec::new();
                for &(start, end, shift) in &reach {
                    // The first piece produced keeps the original's id -
                    // whatever links to this clip (a caption's `caption_of`, a
                    // sound's `detached_from`) must still find it after the
                    // rebuild.
                    let first = pieces.is_empty();
                    let mut piece = original.clone();
                    piece.id = if first {
                        original.id.clone()
                    } else {
                        mint.next("c")
                    };
                    let rel = start - original.start;
                    piece.start = start + shift;
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
                    if end < orig_end - JOIN_EPSILON {
                        piece.fade_out = 0.0;
                    }
                    // A curve does not survive being cut into pieces, as
                    // a split's halves go to the constant mean.
                    if reach.len() > 1 {
                        piece.speed_curve = None;
                    }
                    piece.rewindow_keys(old_duration, rel, rel + piece.duration);
                    // Phase 6: the piece keeps the words it covers, moved to
                    // where the piece landed, and - when the caption is split
                    // across ranges - its text becomes those words, so a
                    // caption cut at a silence no longer repeats its whole
                    // sentence on both sides.
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
                        if surviving_ranges > 1 && any_range_has_words && kept.is_empty() {
                            // A silent edge sliver of a word-timed caption that
                            // spilled into an adjacent range with zero spoken words.
                            continue;
                        }
                        // A rebuilt piece's text is its surviving words -
                        // also when none survived. A piece that kept the
                        // whole old sentence with no words left is exactly
                        // the stale repetition the rebuild exists to stop,
                        // and the E2 husk sweep reads the empty text as
                        // "this piece went with its words".
                        if surviving_ranges > 1
                            && piece.kind == ClipKind::Text
                            && let Some(style) = &mut piece.text
                        {
                            style.content = kept
                                .iter()
                                .map(|word| word.text.as_str())
                                .collect::<Vec<_>>()
                                .join(" ");
                        }
                        if !kept.is_empty() {
                            piece.words = Some(kept);
                        } else {
                            piece.words = None;
                        }
                    }
                    // Remember the original timeline interval before the
                    // packed piece loses that coordinate. Link repair below
                    // uses the interval to pair a caption/audio tail with
                    // the source tail made from the same kept range.
                    let piece_id = piece.id.clone();
                    rebuilt_pieces
                        .entry(original.id.clone())
                        .or_default()
                        .push((start, end, piece_id.clone()));
                    original_span_of_piece.insert(piece_id, (start, end));
                    pieces.push(piece);
                }
                timeline.clips.remove(index);
                for (offset, piece) in pieces.into_iter().enumerate() {
                    timeline.clips.insert(index + offset, Arc::new(piece));
                }
            }

            // A split keeps the first source id, but every later piece gets
            // a fresh one. Re-link text and detached sound pieces to the
            // source piece occupying their original interval. This is the
            // part that makes Sections' Remove selected stay synchronized
            // after a subsequent timeline edit, instead of merely looking
            // synchronized until the first drag.
            for piece in
                timeline.clips_where(|other| original_span_of_piece.contains_key(&other.id))
            {
                let Some(&(raw_start, raw_end)) = original_span_of_piece.get(&piece.id) else {
                    continue;
                };
                if let Some(source) = piece.caption_of.clone()
                    && let Some(replacement) =
                        rebuilt_piece_for(&rebuilt_pieces, &source, raw_start, raw_end)
                {
                    piece.caption_of = Some(replacement);
                }
                if let Some(source) = piece.detached_from.clone()
                    && let Some(replacement) =
                        rebuilt_piece_for(&rebuilt_pieces, &source, raw_start, raw_end)
                {
                    piece.detached_from = Some(replacement);
                }
            }

            // The clips after the window that were not rebuilt close up:
            // whatever began at or after the clip's old end moves left by what
            // came out of it (words included).
            for other in timeline.clips_where(|other| {
                !participants.contains(&other.id) && other.start >= clip_end - JOIN_EPSILON
            }) {
                other.start = (other.start - removed_total).max(0.0);
                if let Some(words) = &mut other.words {
                    for word in words.iter_mut() {
                        word.start = (word.start - removed_total).max(0.0);
                        word.end = (word.end - removed_total).max(0.0);
                    }
                }
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

/// The new source-piece id that owns a rebuilt child interval.
///
/// `AutotrimClip` creates every piece from one of the same kept segments,
/// so the interval midpoint normally lands in exactly one source piece. The
/// overlap fallback keeps a link deterministic when a pre-existing document
/// has slightly mismatched caption boundaries at a cut edge.
fn rebuilt_piece_for(
    rebuilt: &HashMap<String, Vec<(f64, f64, String)>>,
    source_id: &str,
    child_start: f64,
    child_end: f64,
) -> Option<String> {
    let pieces = rebuilt.get(source_id)?;
    let middle = (child_start + child_end) / 2.0;
    if let Some((_, _, id)) = pieces
        .iter()
        .find(|(start, end, _)| *start - JOIN_EPSILON <= middle && middle <= *end + JOIN_EPSILON)
    {
        return Some(id.clone());
    }
    pieces
        .iter()
        .filter_map(|(start, end, id)| {
            let overlap = end.min(child_end) - start.max(child_start);
            (overlap > JOIN_EPSILON).then_some((overlap, id))
        })
        .max_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, id)| id.clone())
}

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

/// A caption may share a span with another caption, but not an ordinary
/// clip. This is the lower-lane counterpart to
/// [`first_caption_track_above`].
fn first_caption_track(timeline: &Timeline, start: f64, duration: f64) -> Option<String> {
    timeline
        .tracks
        .iter()
        .find(|track| timeline.is_caption_free(&track.id, start, duration, &[]))
        .map(|track| track.id.clone())
}

/// A new, empty lane at the top, for a clip with nowhere else to go.
fn fresh_track(timeline: &mut Timeline, mint: &mut IdMint) -> String {
    let id = mint.next("t");
    timeline.tracks.push(Track {
        id: id.clone(),
        visible: true,
        muted: false,
        caption_nudge_ms: 0,
        speaker: None,
        extra: Default::default(),
    });
    id
}

/// First lane above all ordinary clips over `[start, start + duration)` that
/// a caption may use. Existing captions on that lane are allowed to overlap,
/// preserving the timing of cues that overlap in an imported subtitle file.
fn first_caption_track_above(timeline: &Timeline, start: f64, duration: f64) -> Option<String> {
    let end = start + duration;
    let floor = timeline
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| {
            timeline.clips.iter().any(|clip| {
                clip.track_id == track.id
                    && !clip.is_caption()
                    && clip.start < end
                    && start < clip.start + clip.duration
            })
        })
        .map(|(row, _)| row + 1)
        .max()
        .unwrap_or(0);
    timeline
        .tracks
        .iter()
        .skip(floor)
        .find(|track| timeline.is_caption_free(&track.id, start, duration, &[]))
        .map(|track| track.id.clone())
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
