// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! What a placed clip looks and sounds like: its patchable fields, speed, transform, keys and cutout.
//!
//! One arm per command, exactly as [`super::apply`] routes them here;
//! everything these arms share lives in the parent module.

use super::*;

/// T7: keep `clip.words` consistent with the clip's text after a text patch.
///
/// The roadmap's three cases: the same token count re-texts the words in
/// position and keeps every timing (fixing a typo must not lose the
/// karaoke); a different count prunes to the old words that still appear,
/// in their old order, counted as a multiset so a repeated word survives
/// exactly as often as it still occurs; and nothing ever invents a word.
/// Tokens are `captionmap::word_units`, so the split stays the one rule
/// (T4). A clip whose style was cleared has no content to highlight, so
/// its words go with it - absent by design, never stale. The stored caption
/// flag is not touched: identity survives every text edit (T5). Titles
/// carry no words and fall straight through.
pub(super) fn reconcile_caption_words(clip: &mut Clip) {
    let Some(style) = clip.text.as_ref() else {
        clip.words = None;
        return;
    };
    let Some(words) = clip.words.as_mut() else {
        return;
    };
    let units = crate::captionmap::word_units(&style.content);
    if units.len() == words.len() {
        for (word, unit) in words.iter_mut().zip(units) {
            word.text = unit;
        }
        return;
    }
    let mut remaining: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for unit in &units {
        *remaining.entry(unit.as_str()).or_insert(0) += 1;
    }
    words.retain(|word| match remaining.get_mut(word.text.trim()) {
        Some(count) if *count > 0 => {
            *count -= 1;
            true
        }
        _ => false,
    });
}

/// Applies one of this module's commands. Any other is a routing error.
pub(super) fn apply(
    project: &mut Project,
    _mint: &mut IdMint,
    command: Command,
) -> Result<Outcome, CommandError> {
    match command {
        Command::UpdateClip { clip_id, patch } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let mut applied = false;
            if let Some(name) = patch.name {
                applied |= assign(&mut clip.name, name);
            }
            if let Some(volume) = patch.volume {
                applied |= assign(&mut clip.volume, volume.max(0.0));
            }
            if let Some(fade_in) = patch.fade_in {
                applied |= assign(&mut clip.fade_in, fade_in.max(0.0));
            }
            if let Some(fade_out) = patch.fade_out {
                applied |= assign(&mut clip.fade_out, fade_out.max(0.0));
            }
            if let Some(opacity) = patch.opacity {
                applied |= assign(&mut clip.opacity, opacity.clamp(0.0, 1.0));
            }
            if let Some(preserve) = patch.preserve_pitch {
                applied |= assign(&mut clip.preserve_pitch, preserve);
            }
            if let Some(muted) = patch.muted {
                // Unmuted is the absent value, so a document never carries
                // a `muted: false` that means the same as nothing.
                applied |= assign(&mut clip.muted, muted.then_some(true));
            }
            if let Some(flip) = patch.flip_h {
                applied |= assign(&mut clip.flip_h, flip);
            }
            if let Some(flip) = patch.flip_v {
                applied |= assign(&mut clip.flip_v, flip);
            }
            if let Some(blend) = patch.blend {
                let blend = if blend == "normal" {
                    String::new()
                } else {
                    blend
                };
                applied |= assign(&mut clip.blend, blend);
            }
            if let Some(crop) = patch.crop {
                let crop = crop.map(Crop::tidy).filter(|crop| !crop.is_none());
                applied |= assign(&mut clip.crop, crop);
            }
            if let Some(filters) = patch.filters {
                applied |= assign(&mut clip.filters, filters);
            }
            if let Some(effects) = patch.video_effects {
                applied |= assign(&mut clip.video_effects, effects);
            }
            if let Some(transition) = patch.transition_in {
                applied |= assign(&mut clip.transition_in, transition);
            }
            if let Some(text) = patch.text {
                // The name follows the words, like addTextClip snapshots it.
                if let Some(style) = &text {
                    applied |= assign(&mut clip.name, first_line(&style.content));
                }
                applied |= assign(&mut clip.text, text);
                // T7: this arm is the single writer of a text patch, so it
                // writes the words with the text. Content and word timings
                // can never drift apart through an edit that passes here.
                reconcile_caption_words(clip);
            }
            if let Some(stream) = patch.audio_stream {
                applied |= assign(&mut clip.audio_stream, stream);
            }
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SetClipSpeed { clip_id, speed } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            // The amount of source covered is held constant - that is what
            // makes this a speed change rather than a trim.
            let next = speed.clamp(MIN_SPEED, MAX_SPEED);
            let source_covered = clip.duration * clip.speed;
            let old_end = clip.start + clip.duration;
            // Bitwise so no assignment is short-circuited away. A rate set
            // by hand is a constant rate: the curve goes.
            let applied = assign(&mut clip.speed, next)
                | assign(
                    &mut clip.duration,
                    (source_covered / next).max(MIN_CLIP_DURATION),
                )
                | assign(&mut clip.speed_curve, None);
            // Slowed down, the clip grows into what follows it: the rest of
            // the lane moves along by the growth, as a freeze frame's does,
            // rather than being covered.
            let (track_id, start, duration) = (clip.track_id.clone(), clip.start, clip.duration);
            let grow = start + duration - old_end;
            if grow > 0.0 && !timeline.is_free(&track_id, start, duration, &[&clip_id]) {
                for other in timeline.clips_where(|other| {
                    other.id != clip_id
                        && other.track_id == track_id
                        && other.start >= old_end - crate::placement::TOUCH
                }) {
                    other.start += grow;
                }
            }
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SetClipCutout { clip_id, cutout } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let applied = assign(&mut clip.cutout, cutout.map(Cutout::tidy));
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::AddCutoutStroke { clip_id, stroke } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let Some(stroke) = stroke.tidy() else {
                return Ok(Outcome::default());
            };
            let cutout = clip.cutout.get_or_insert_with(Cutout::auto);
            cutout.mode = CutoutMode::Custom;
            cutout.strokes.push(stroke);
            Ok(Outcome {
                created_id: None,
                applied: true,
            })
        }

        Command::SetClipKey {
            clip_id,
            property,
            at,
            value,
            ease,
        } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            if !at.is_finite() || !value.is_finite() {
                return Ok(Outcome::default());
            }
            // Clamped the way the field itself is, so a key can never hold a
            // value the constant would have been refused. The clamps are the
            // ones ClipPatch applies; keeping them here as well is what stops
            // a key being the back door round them.
            let value = match property {
                KeyProperty::Scale => value.clamp(MIN_SCALE, MAX_SCALE),
                KeyProperty::Opacity => value.clamp(0.0, 1.0),
                // No ceiling, matching ClipPatch: the level fader goes to
                // +24 dB because quiet material needs it, and a key is the
                // same value at a different instant.
                KeyProperty::Volume => value.max(0.0),
                KeyProperty::OffsetX | KeyProperty::OffsetY | KeyProperty::Rotation => value,
            };
            let before = clip.keys.clone();
            clip.set_key(property, at, value, ease);
            Ok(Outcome {
                created_id: None,
                applied: clip.keys != before,
            })
        }

        Command::ClearClipKey {
            clip_id,
            property,
            at,
        } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let applied = clip.clear_key(property, at);
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::ClearClipKeys { clip_id, property } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let applied = clip.clear_keys(property);
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::AutoReframe {
            clip_id,
            target_width,
            target_height,
            points,
        } => {
            let refuse = |reason: &str| -> Result<Outcome, CommandError> {
                Err(CommandError::CannotReframe {
                    reason: reason.to_owned(),
                })
            };
            if points.is_empty() {
                return refuse("No subject positions were given, so there is nothing to follow.");
            }
            let plan = {
                let timeline = project.active();
                let Some(clip) = timeline.clip(&clip_id) else {
                    return refuse("That clip no longer exists.");
                };
                let Some(media) = project.media_by_id(&clip.media_id) else {
                    return refuse("That clip has no media picture to reframe.");
                };
                let (Some(width), Some(height)) = (media.width, media.height) else {
                    return refuse(
                        "That clip's media has no picture size, so there is nothing to reframe.",
                    );
                };
                let Some(plan) = crate::reframe::reframe_plan(
                    clip,
                    (width, height),
                    (target_width, target_height),
                    &points,
                ) else {
                    return refuse(
                        "That clip already fills this frame; there is nothing to reframe.",
                    );
                };
                plan
            };
            let timeline = project.active_mut();
            let clip = timeline
                .clip_mut(&clip_id)
                .expect("found above, and nothing between removed it");
            clip.scale = plan.scale;
            for key in plan.keys {
                clip.set_key(key.property, key.at, key.value, key.ease);
            }
            Ok(Outcome {
                created_id: None,
                applied: true,
            })
        }

        Command::DuckMusic {
            clip_id,
            against_clips,
            against_regions,
            amount_db,
            fade_ms,
        } => {
            let timeline = project.active_mut();
            let fade_s = f64::from(fade_ms.min(crate::duck::MAX_FADE_MS)) / 1000.0;
            // Route A first: regions the caller analysed from audio win
            // over the transcript route, because the caller only passes
            // them when the analysis is the point (owner A6: the route is
            // chosen per clip).
            let regions: Vec<(f64, f64)> = if !against_regions.is_empty() {
                let spans: Vec<(f64, f64)> = against_regions
                    .iter()
                    .map(|region| (region.start, region.end))
                    .collect();
                crate::duck::merge_regions(&spans, fade_s)
            } else {
                // Route B: the heard words of the named clips are the
                // speech. A clip with no words contributes nothing.
                let mut words = Vec::new();
                for id in &against_clips {
                    if let Some(heard) = timeline.clip(id).and_then(|clip| clip.words.as_ref()) {
                        words.extend(heard.iter().cloned());
                    }
                }
                crate::duck::regions_from_words(&words, fade_s)
            };
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let keys = crate::duck::duck_keys(clip, &regions, amount_db, fade_s);
            if keys.is_empty() {
                return Ok(Outcome::default());
            }
            let before = clip.keys.clone();
            for key in keys {
                clip.set_key(key.property, key.at, key.value, key.ease);
            }
            Ok(Outcome {
                created_id: None,
                applied: clip.keys != before,
            })
        }

        Command::SetEffectKey {
            clip_id,
            entry,
            key,
            at,
            value,
            ease,
        } => {
            let timeline = project.active_mut();
            let Some(link) = timeline
                .clip_mut(&clip_id)
                .and_then(|clip| clip.video_effects.get_mut(entry))
            else {
                return Ok(Outcome::default());
            };
            if !at.is_finite() || !value.is_finite() {
                return Ok(Outcome::default());
            }
            let before = link.keys.clone();
            link.set_key(&key, at, value, ease);
            Ok(Outcome {
                created_id: None,
                applied: link.keys != before,
            })
        }

        Command::ClearEffectKey {
            clip_id,
            entry,
            key,
            at,
        } => {
            let timeline = project.active_mut();
            let Some(link) = timeline
                .clip_mut(&clip_id)
                .and_then(|clip| clip.video_effects.get_mut(entry))
            else {
                return Ok(Outcome::default());
            };
            let applied = link.clear_key(&key, at);
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::ClearEffectKeys {
            clip_id,
            entry,
            key,
        } => {
            let timeline = project.active_mut();
            let Some(link) = timeline
                .clip_mut(&clip_id)
                .and_then(|clip| clip.video_effects.get_mut(entry))
            else {
                return Ok(Outcome::default());
            };
            let applied = link.clear_keys(&key);
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SetClipSpeedCurve { clip_id, curve } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let curve = curve.filter(|points| crate::speed::curve_of(points).is_some());
            let source_covered = clip.duration * clip.speed;
            let mean = curve
                .as_ref()
                .map(|points| crate::speed::mean_of(points))
                .unwrap_or(clip.speed)
                .clamp(MIN_SPEED, MAX_SPEED);
            let applied = assign(&mut clip.speed_curve, curve)
                | assign(&mut clip.speed, mean)
                | assign(
                    &mut clip.duration,
                    (source_covered / mean).max(MIN_CLIP_DURATION),
                );
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SetClipTransform {
            clip_id,
            scale,
            offset_x,
            offset_y,
            rotation,
            stretch_x,
            stretch_y,
        } => {
            let timeline = project.active_mut();
            let Some(clip) = timeline.clip_mut(&clip_id) else {
                return Ok(Outcome::default());
            };
            let mut applied = false;
            if let Some(scale) = scale {
                applied |= assign(&mut clip.scale, scale.clamp(MIN_SCALE, MAX_SCALE));
            }
            if let Some(offset) = offset_x {
                applied |= assign(&mut clip.offset_x, offset.clamp(-MAX_OFFSET, MAX_OFFSET));
            }
            if let Some(offset) = offset_y {
                applied |= assign(&mut clip.offset_y, offset.clamp(-MAX_OFFSET, MAX_OFFSET));
            }
            if let Some(rotation) = rotation {
                applied |= assign(&mut clip.rotation, wrap_rotation(rotation));
            }
            if let Some(stretch) = stretch_x {
                applied |= assign(&mut clip.stretch_x, stretch.clamp(MIN_STRETCH, MAX_STRETCH));
            }
            if let Some(stretch) = stretch_y {
                applied |= assign(&mut clip.stretch_y, stretch.clamp(MIN_STRETCH, MAX_STRETCH));
            }
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        _ => unreachable!("commands::apply routes only this module's commands here"),
    }
}
