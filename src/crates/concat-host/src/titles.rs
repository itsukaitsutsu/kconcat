// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Titles and shapes, rendered and remembered.
//!
//! The flattener leaves text and shape clips out on purpose: the compositor
//! only knows pictures. This is where a title or a shape becomes one. Each
//! clip's style is painted by `concat-text` onto a frame-sized transparent
//! PNG in the app's data directory, and the clip rejoins the flattened list
//! as image clips pointing at those pixels, carrying the clip's timing,
//! transform and opacity. A timed caption with text motion becomes a short
//! sequence of still clips so the same behavior works in monitor and export.
//! The monitor, playback prefetch and the exporter then treat them as stills.
//!
//! The file is keyed by everything that changes the pixels - the style, the
//! frame size, the fonts the project carries - so a title that has not
//! changed is never painted twice, across sessions included. What a title
//! does *not* key on is where it sits or when it plays: moving one is free.
//! A shape is a title with no words, as far as this file is concerned.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use concat_core::animate::{Ease, Key, Track};
use concat_core::frame::Frame;
use concat_core::shader::RevealMap;

use concat_export::{ClipKind, ExportClip, ExportKey};
use concat_project::captionmap::{Word, layout_words_of, locate_words, word_units};
use concat_project::model::{
    ClipKind as ModelClipKind, Project, ShapeKind, ShapeStyle, TextAlign, TextStyle,
};
use concat_text::{Align, Figure, Fonts, TitleStyle, WordRect};

use crate::dirs::AppDirs;

/// One title or shape, ready for the compositor.
#[derive(Clone)]
pub struct TitleClip {
    /// The text or shape clip this was painted from.
    pub clip_id: String,
    /// The image clip that stands in for it.
    pub clip: ExportClip,
    /// The painted block's size in frame pixels, for an outline on a monitor.
    pub block: (u32, u32),
    /// Where that block's centre is, as an offset in frame pixels from the
    /// clip's own centre: zero for a centred title, half a block to the
    /// right for a left-aligned one, whose block starts at the clip's
    /// position. See `concat_text::Align`.
    pub offset: (i32, i32),
    /// The pixels, when the title was painted live rather than to disk:
    /// what the monitor is to show under `clip.path`, a name no file has.
    /// See [`Titles::clips_live`].
    pub frame: Option<Arc<Frame>>,
}

/// What is painted: a title's words or a shape's figure. One painter for
/// both, since they are cached, keyed and handed on the same way.
#[derive(Clone, Copy)]
enum Painting<'a> {
    Title(&'a TextStyle),
    Shape(&'a ShapeStyle),
    /// One title's words in its highlight colour, for the karaoke overlay
    /// laid over the title. `words` are indices into the title's layout
    /// words - see `concat_text::word_ranges` - as a half-open pair.
    Highlight {
        style: &'a TextStyle,
        words: (usize, usize),
    },
}

/// What one render left behind.
#[derive(Clone, Debug)]
struct Art {
    block: (u32, u32),
    offset: (i32, i32),
    /// The title's per-word reveal order, baked once here rather than on
    /// every clip build - see `concat_core::RevealMap`.
    reveal: Arc<RevealMap>,
}

/// The title painter and its cache.
pub struct Titles {
    dir: PathBuf,
    /// The system's faces plus the project's files, loaded once each.
    fonts: Mutex<Option<Fonts>>,
    loaded_files: Mutex<HashSet<String>>,
    /// What is known to be on disk, by key.
    memo: Mutex<HashMap<u64, Art>>,
    /// Titles painted live, by key: every title on the timeline, and the
    /// most recently asked for of the rest, so a width dragged back and
    /// forth over the same values paints each once.
    live: Mutex<LivePaintings>,
}

/// The live paintings kept, in bytes of pixels, beyond the ones the
/// timeline is showing; see [`LivePaintings`]. Each is a picture the
/// frame's size - 8 MB at 1080p, 33 MB at 4K - so a count cost four times
/// as much on a 4K timeline as on an HD one. 128 MB keeps sixteen HD
/// paintings or four 4K ones.
const LIVE_BYTES: usize = 128 * 1024 * 1024;

/// The most the paintings of the titles on the timeline may hold between
/// them, in bytes of pixels: 64 HD titles, or 256 at the monitor's half
/// size. Past it the first painted are kept and the rest painted again
/// each time; see [`LivePaintings::keep`].
const LIVE_CEILING: usize = 512 * 1024 * 1024;

/// One live painting, and the pass that last asked for it.
struct LivePainting {
    art: Art,
    frame: Arc<Frame>,
    used: u64,
}

/// The titles painted live, by key, least recently used out first.
///
/// Every live ask paints every title on the timeline in turn, the same
/// order each time. Kept first in, first out - and a hit not counting as a
/// use - past 64 half-size portrait titles each one asked for was the one
/// thrown out the step before, so every title was painted again on every
/// change, and the editor lagged (#261). Now what the pass in hand has
/// asked for is never thrown out for the byte budget: the floor is every
/// title on the timeline, up to [`LIVE_CEILING`].
struct LivePaintings {
    kept: HashMap<u64, LivePainting>,
    /// Which pass over the timeline this is; see [`LivePaintings::begin`].
    pass: u64,
    /// Bytes of pixels held.
    bytes: usize,
    /// What the paintings past passes asked for may hold.
    budget: usize,
    /// What all of them may hold.
    ceiling: usize,
}

impl LivePaintings {
    fn new(budget: usize, ceiling: usize) -> Self {
        LivePaintings {
            kept: HashMap::new(),
            pass: 0,
            bytes: 0,
            budget,
            ceiling,
        }
    }

    /// Starts a pass over the timeline: what it asks for is held whatever
    /// the budget.
    fn begin(&mut self) {
        self.pass += 1;
    }

    /// The painting under `key`, now the most recently used.
    fn get(&mut self, key: u64) -> Option<(Art, Arc<Frame>)> {
        let pass = self.pass;
        self.kept.get_mut(&key).map(|painting| {
            painting.used = pass;
            (painting.art.clone(), Arc::clone(&painting.frame))
        })
    }

    /// Keeps a painting under `key`. Over the budget, the least recently
    /// used go first, never one this pass has asked for. Over the ceiling
    /// with nothing else left, the one just painted is not kept: the ones
    /// painted before it stay for the next pass, rather than each pushing
    /// out the one it will want next. The newest is always kept alone.
    fn keep(&mut self, key: u64, art: Art, frame: Arc<Frame>) {
        let size = frame.pixels().len();
        let painting = LivePainting {
            art,
            frame,
            used: self.pass,
        };
        if let Some(old) = self.kept.insert(key, painting) {
            self.bytes -= old.frame.pixels().len();
        }
        self.bytes += size;
        while self.bytes > self.budget {
            let oldest = self
                .kept
                .iter()
                .filter(|(_, painting)| painting.used < self.pass)
                .min_by_key(|(_, painting)| painting.used)
                .map(|(key, _)| *key);
            let Some(oldest) = oldest else { break };
            self.drop_key(oldest);
        }
        if self.bytes > self.ceiling && self.kept.len() > 1 {
            self.drop_key(key);
        }
    }

    fn drop_key(&mut self, key: u64) {
        if let Some(gone) = self.kept.remove(&key) {
            self.bytes -= gone.frame.pixels().len();
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MotionSlice {
    start: f64,
    end: f64,
    text_end: usize,
    scale: f64,
    offset_y: f64,
    opacity: f64,
}

#[derive(Clone, Copy)]
struct MotionEvent {
    at: f64,
    text_end: usize,
    scale: f64,
    offset_y: f64,
    opacity: f64,
}

/// Turn a text entrance into short stills. The samples are time-based, not
/// tied to the monitor's refresh rate, so preview and export use the same
/// pixels and timing. Typewriter is Unicode-boundary-safe; legacy word reveal
/// continues to use stored speech timings.
fn caption_motion_slices(
    style: &TextStyle,
    words: Option<&[Word]>,
    clip_start: f64,
    clip_duration: f64,
) -> Option<Vec<MotionSlice>> {
    use concat_project::model::MotionMode;

    let mode = style.motion.mode;
    if matches!(mode, MotionMode::None | MotionMode::Karaoke)
        || !clip_start.is_finite()
        || !clip_duration.is_finite()
        || clip_duration <= 0.0
        || style.content.is_empty()
    {
        return None;
    }
    let clip_end = clip_start + clip_duration;
    if !clip_end.is_finite() {
        return None;
    }

    let mut events = Vec::new();
    match mode {
        MotionMode::None | MotionMode::Karaoke => return None,
        MotionMode::Fade | MotionMode::Pop | MotionMode::Bounce | MotionMode::SlideUp => {
            let span = style.motion.duration.min(clip_duration);
            let frames = ((span * 30.0).ceil() as usize).clamp(2, 60);
            for frame in 0..=frames {
                let p = frame as f64 / frames as f64;
                let ease_out = 1.0 - (1.0 - p).powi(3);
                let (scale, offset_y, opacity) = match mode {
                    MotionMode::Fade => (1.0, 0.0, ease_out),
                    MotionMode::Pop => {
                        let from = style.motion.scale_from;
                        let scale = if from > 1.0 {
                            from + (1.0 - from) * ease_out
                        } else if p < 0.68 {
                            let q = (p / 0.68).clamp(0.0, 1.0);
                            from + (1.16 - from) * (1.0 - (1.0 - q).powi(3))
                        } else {
                            let q = ((p - 0.68) / 0.32).clamp(0.0, 1.0);
                            1.16 + (1.0 - 1.16) * (1.0 - (1.0 - q).powi(3))
                        };
                        (scale, 0.0, (p * 3.0).min(1.0))
                    }
                    MotionMode::Bounce => {
                        let (y, scale) = if p < 0.58 {
                            let q = (p / 0.58).clamp(0.0, 1.0);
                            (-0.065 + 0.078 * (1.0 - (1.0 - q).powi(3)), 0.88 + 0.17 * q)
                        } else if p < 0.8 {
                            let q = ((p - 0.58) / 0.22).clamp(0.0, 1.0);
                            (0.013 - 0.021 * q, 1.05 - 0.04 * q)
                        } else {
                            let q = ((p - 0.8) / 0.2).clamp(0.0, 1.0);
                            (-0.008 * (1.0 - q), 1.01 - 0.01 * q)
                        };
                        (scale, y, (p * 3.0).min(1.0))
                    }
                    MotionMode::SlideUp => (1.0, 0.075 * (1.0 - ease_out), ease_out),
                    _ => unreachable!(),
                };
                events.push(MotionEvent {
                    at: clip_start + span * p,
                    text_end: style.content.len(),
                    scale,
                    offset_y,
                    opacity,
                });
            }
        }
        MotionMode::Typewriter => {
            let chars: Vec<(usize, usize)> = style
                .content
                .char_indices()
                .map(|(start, ch)| (start, start + ch.len_utf8()))
                .collect();
            if chars.is_empty() {
                return None;
            }
            let ranges = text_word_ranges(&style.content);
            let word_count = ranges.len().max(1);
            let requested = style.motion.duration
                + style.motion.stagger * word_count.saturating_sub(1) as f64;
            let fit = clip_duration.min(requested) / requested;
            for (index, &(start, end)) in chars.iter().enumerate() {
                let word_pauses = ranges
                    .iter()
                    .filter(|range| range.1 <= start)
                    .count()
                    .min(word_count.saturating_sub(1));
                let at = clip_start
                    + (style.motion.duration * (index + 1) as f64
                        / chars.len().max(1) as f64
                        + style.motion.stagger * word_pauses as f64)
                        * fit;
                events.push(MotionEvent {
                    at,
                    text_end: end,
                    scale: 1.0,
                    offset_y: 0.0,
                    opacity: 1.0,
                });
            }
            events.push(MotionEvent {
                at: clip_start + requested * fit,
                text_end: style.content.len(),
                scale: 1.0,
                offset_y: 0.0,
                opacity: 1.0,
            });
        }
        MotionMode::WordReveal => {
            let timed = words?;
            let located = locate_words(&style.content, timed);
            let mut valid: Vec<(f64, usize, usize)> = timed
                .iter()
                .zip(located.iter())
                .filter_map(|(word, range)| {
                    let range = range.as_ref()?;
                    (word.start.is_finite() && word.end.is_finite() && word.end > word.start)
                        .then_some((word.start.max(clip_start), range.start, range.end))
                })
                .filter(|(start, _, _)| *start < clip_end)
                .collect();
            valid.sort_by(|a, b| a.0.total_cmp(&b.0));
            if valid.is_empty() {
                return None;
            }
            for (index, (at, _, word_end)) in valid.iter().copied().enumerate() {
                let text_end = valid
                    .get(index + 1)
                    .map(|next| next.1)
                    .unwrap_or(style.content.len())
                    .max(word_end)
                    .min(style.content.len());
                events.push(MotionEvent {
                    at: at.clamp(clip_start, clip_end),
                    text_end,
                    scale: 1.0,
                    offset_y: 0.0,
                    opacity: 1.0,
                });
            }
        }
    }

    events.sort_by(|a, b| a.at.total_cmp(&b.at));
    let mut collapsed: Vec<MotionEvent> = Vec::with_capacity(events.len());
    for event in events {
        let event = MotionEvent {
            at: event.at.clamp(clip_start, clip_end),
            ..event
        };
        if let Some(previous) = collapsed.last_mut()
            && (previous.at - event.at).abs() < 1e-9
        {
            *previous = event;
        } else {
            collapsed.push(event);
        }
    }
    let slices: Vec<MotionSlice> = collapsed
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            let start = event.at.max(clip_start);
            let end = collapsed
                .get(index + 1)
                .map_or(clip_end, |next| next.at)
                .min(clip_end);
            (end > start + 1e-4 && event.text_end > 0).then_some(MotionSlice {
                start,
                end,
                text_end: event.text_end.min(style.content.len()),
                scale: event.scale,
                offset_y: event.offset_y,
                opacity: event.opacity.clamp(0.0, 1.0),
            })
        })
        .collect();
    (!slices.is_empty()).then_some(slices)
}

/// Locate language-aware word units in the stored text for typewriter pauses.
fn text_word_ranges(content: &str) -> Vec<(usize, usize)> {
    let words: Vec<Word> = word_units(content)
        .into_iter()
        .map(|text| Word { text, start: 0.0, end: 0.0 })
        .collect();
    locate_words(content, &words)
        .into_iter()
        .flatten()
        .map(|range| (range.start, range.end))
        .collect()
}

/// Give untimed text a simple, language-aware word clock for Karaoke.
fn synthetic_words(content: &str, start: f64, duration: f64) -> Vec<Word> {
    let units = word_units(content);
    if units.is_empty() || !start.is_finite() || !duration.is_finite() || duration <= 0.0 {
        return Vec::new();
    }
    let span = duration / units.len() as f64;
    units
        .into_iter()
        .enumerate()
        .map(|(index, text)| Word {
            text,
            start: start + span * index as f64,
            end: start + span * (index + 1) as f64,
        })
        .collect()
}

impl Titles {
    /// A painter that caches under the app's data directory.
    pub fn new(dirs: &AppDirs) -> Titles {
        let dir = dirs.data.join("titles");
        sweep(&dir, TITLES_KEPT);
        Titles {
            dir,
            fonts: Mutex::new(None),
            loaded_files: Mutex::new(HashSet::new()),
            memo: Mutex::new(HashMap::new()),
            live: Mutex::new(LivePaintings::new(LIVE_BYTES, LIVE_CEILING)),
        }
    }

    /// Every text and shape clip on the active timeline, as an image clip
    /// each, for a `width` × `height` frame. One that fails to paint is left
    /// out and said once on stderr; the rest of the edit still renders.
    pub fn clips(&self, project: &Project, width: u32, height: u32) -> Vec<TitleClip> {
        self.clips_with(project, width, height, false, None)
    }

    /// [`Titles::clips`] for a monitor showing a change as it is made:
    /// every title is painted in memory at `width` by `height` - the
    /// monitor's own size, not the output's - and comes back with its
    /// pixels under a name no file has, for the monitor to hold. Nothing
    /// touches the disk, which is what makes a drag smooth: a PNG per
    /// pointer step, encoded, written and decoded again, was the lag.
    pub fn clips_live(&self, project: &Project, width: u32, height: u32) -> Vec<TitleClip> {
        self.clips_with(project, width, height, true, None)
    }

    /// Same as [`Titles::clips_live`], but only rasterizes titles and word
    /// highlights overlapping `[win_from, win_to]` (plus any clip in
    /// `include_ids`), so styling 200+ captions at once (`Apply to all captions`)
    /// only paints the 1–2 captions visible near the playhead instead of
    /// freezing the UI thread across the entire timeline.
    pub fn clips_live_window(
        &self,
        project: &Project,
        width: u32,
        height: u32,
        win_from: f64,
        win_to: f64,
        include_ids: &[String],
    ) -> Vec<TitleClip> {
        self.clips_with(
            project,
            width,
            height,
            true,
            Some((win_from, win_to, include_ids)),
        )
    }

    fn clips_with(
        &self,
        project: &Project,
        width: u32,
        height: u32,
        live: bool,
        window: Option<(f64, f64, &[String])>,
    ) -> Vec<TitleClip> {
        if live {
            self.live.lock().unwrap_or_else(|e| e.into_inner()).begin();
        }
        let timeline = project.active();
        let mut out = Vec::new();
        for clip in &timeline.clips {
            if let Some((win_from, win_to, include_ids)) = window {
                let clip_end = clip.start + clip.duration;
                let in_window = clip_end >= win_from && clip.start <= win_to;
                if !in_window && !include_ids.iter().any(|id| id == &clip.id) {
                    continue;
                }
            }
            let (text, shape);
            let painting = match clip.kind {
                ModelClipKind::Text => {
                    text = clip.text.clone().unwrap_or_default();
                    Painting::Title(&text)
                }
                ModelClipKind::Shape => {
                    shape = clip.shape.clone().unwrap_or_default();
                    Painting::Shape(&shape)
                }
                _ => continue,
            };
            let Some(index) = timeline
                .tracks
                .iter()
                .position(|track| track.id == clip.track_id)
            else {
                continue;
            };
            let track = &timeline.tracks[index];
            let painted = if live {
                self.painted_live(project, painting, width, height)
                    .map(|(path, art, frame)| (path, art, Some(frame)))
            } else {
                self.painted(project, painting, width, height)
                    .map(|(path, art)| (path, art, None))
            };
            let (path, art, frame) = match painted {
                Ok(art) => art,
                Err(error) => {
                    log::warn!("title {}: {error}", clip.id);
                    continue;
                }
            };
            let title = TitleClip {
                clip_id: clip.id.clone(),
                clip: ExportClip {
                    path: path.to_string_lossy().into_owned(),
                    hidden: !track.visible,
                    muted: true,
                    volume: 0.0,
                    animation: concat_export::flatten::export_keys(clip),
                    flip_h: clip.flip_h,
                    flip_v: clip.flip_v,
                    blend: clip.blend.clone(),
                    effects: clip.video_effects.clone(),
                    scale: clip.scale,
                    offset_x: clip.offset_x,
                    offset_y: clip.offset_y,
                    rotation: clip.rotation,
                    // A title is never pulled along an axis: its box is
                    // sized by the style, and its glyphs keep their shape.
                    // 0.2.2's side grips wrote a stretch onto title clips
                    // to make a long caption fit, and honouring it here
                    // kept those captions squashed after the wrap arrived.
                    // https://github.com/jub0t/Concat/issues/119
                    // A shape is pulled: that is how a square becomes
                    // the rectangle that was wanted.
                    stretch_x: match painting {
                        Painting::Shape(_) => clip.stretch_x,
                        Painting::Title(_) | Painting::Highlight { .. } => 1.0,
                    },
                    stretch_y: match painting {
                        Painting::Shape(_) => clip.stretch_y,
                        Painting::Title(_) | Painting::Highlight { .. } => 1.0,
                    },
                    // The style's own opacity multiplies the clip's: a
                    // half-transparent title fades to half, not to solid.
                    opacity: match painting {
                        Painting::Title(text) => (clip.opacity * text.opacity).clamp(0.0, 1.0),
                        Painting::Shape(_) | Painting::Highlight { .. } => {
                            clip.opacity.clamp(0.0, 1.0)
                        }
                    },
                    media_width: Some(width),
                    media_height: Some(height),
                    has_audio: Some(false),
                    // A shape has no words to reveal one at a time.
                    reveal_map: match painting {
                        Painting::Title(_) => Some(Arc::clone(&art.reveal)),
                        Painting::Shape(_) | Painting::Highlight { .. } => None,
                    },
                    ..ExportClip::blank(ClipKind::Image, clip.start, clip.duration, index)
                },
                block: art.block,
                offset: art.offset,
                frame,
            };
            // Karaoke overlays sit directly over the base title. Timed captions
            // use their stored word clock; ordinary titles get an even word clock.
            let overlays = match painting {
                Painting::Title(style)
                    if style.motion.mode == concat_project::model::MotionMode::Karaoke
                        || (!style.highlight_color.is_empty()
                            && style.motion.mode == concat_project::model::MotionMode::None) =>
                {
                    let synthetic = if style.motion.mode == concat_project::model::MotionMode::Karaoke
                        && clip.words.as_deref().is_none_or(|words| words.is_empty())
                    {
                        synthetic_words(&style.content, clip.start, clip.duration)
                    } else {
                        Vec::new()
                    };
                    let words = clip
                        .words
                        .as_deref()
                        .filter(|words| !words.is_empty())
                        .or_else(|| (!synthetic.is_empty()).then_some(synthetic.as_slice()));
                    if let Some(words) = words {
                        let karaoke_style = if style.motion.mode == concat_project::model::MotionMode::Karaoke
                            && style.highlight_color.is_empty()
                        {
                            let mut style = style.clone();
                            style.highlight_color = concat_project::model::DEFAULT_HIGHLIGHT.to_owned();
                            Some(style)
                        } else {
                            None
                        };
                        self.highlights(
                            project,
                            &title,
                            karaoke_style.as_ref().unwrap_or(style),
                            words,
                            width,
                            height,
                            live,
                            window.map(|(f, t, _)| (f, t)),
                        )
                    } else {
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };
            let animated = match painting {
                Painting::Title(style) => {
                    caption_motion_slices(style, clip.words.as_deref(), clip.start, clip.duration)
                    .map(|slices| {
                        slices
                            .into_iter()
                            .filter_map(|slice| {
                                let mut partial = style.clone();
                                partial.content = style.content.get(..slice.text_end)?.to_owned();
                                let painting = Painting::Title(&partial);
                                let painted = if live {
                                    self.painted_live(project, painting, width, height)
                                        .map(|(path, art, frame)| (path, art, Some(frame)))
                                } else {
                                    self.painted(project, painting, width, height)
                                        .map(|(path, art)| (path, art, None))
                                };
                                let (path, art, frame) = match painted {
                                    Ok(result) => result,
                                    Err(error) => {
                                        log::warn!("animated text {}: {error}", clip.id);
                                        return None;
                                    }
                                };
                                let mut animated = title.clone();
                                animated.clip.path = path.to_string_lossy().into_owned();
                                animated.clip.start = slice.start;
                                animated.clip.duration = slice.end - slice.start;
                                animated.clip.scale *= slice.scale;
                                animated.clip.offset_y += slice.offset_y;
                                animated.clip.opacity *= slice.opacity;
                                let from =
                                    (slice.start - clip.start) / clip.duration.max(f64::MIN_POSITIVE);
                                let to = (slice.end - clip.start) / clip.duration.max(f64::MIN_POSITIVE);
                                animated.clip.animation = window_animation(&title.clip.animation, from, to);
                                animated.clip.reveal_map = Some(Arc::clone(&art.reveal));
                                animated.block = art.block;
                                animated.offset = art.offset;
                                animated.frame = frame;
                                Some(animated)
                            })
                            .collect::<Vec<_>>()
                    })
                }
                _ => None,
            };
            if let Some(animated) = animated {
                out.extend(animated);
            } else {
                out.push(title);
            }
            out.extend(overlays);
        }
        out
    }

    /// The karaoke highlight of a word-timed caption: one picture per heard
    /// word, each the caption's own words with only that word in the accent,
    /// on the clip's window for that word and nothing else.
    ///
    /// Each overlay copies the caption's clip, so its place, opacity and
    /// track are the caption's. What does not carry over is what is timed
    /// against the clip's own window: fades, a transition into the clip, and
    /// the reveal map, none of which belongs to a word's moment. Keyframed
    /// animation is carried over as the caption's own motion on the word's
    /// window - see [`window_animation`] - so the overlay moves with the
    /// caption it lights.
    #[allow(clippy::too_many_arguments)]
    fn highlights(
        &self,
        project: &Project,
        title: &TitleClip,
        style: &TextStyle,
        words: &[Word],
        width: u32,
        height: u32,
        live: bool,
        window: Option<(f64, f64)>,
    ) -> Vec<TitleClip> {
        let base = &title.clip;
        // No length, no moment to read the caption's motion against.
        if base.duration.is_nan() || base.duration <= 0.0 {
            return Vec::new();
        }
        let content = style.content.as_str();
        let layout = concat_text::word_ranges(content);
        let covers = layout_words_of(&locate_words(content, words), &layout);
        let (window_start, window_end) = (base.start, base.start + base.duration);
        let mut out = Vec::new();
        for (word, covered) in words.iter().zip(covers) {
            let Some(covered) = covered else { continue };
            // Clipped to the caption's window: a word the trim has cut off
            // has no moment to light, and a timing that is not a number
            // has none either.
            let from = word.start.max(window_start);
            let to = word.end.min(window_end);
            if !from.is_finite() || !to.is_finite() || to <= from {
                continue;
            }
            if let Some((win_from, win_to)) = window
                && (to < win_from || from > win_to)
            {
                continue;
            }
            let painting = Painting::Highlight {
                style,
                words: (covered.start, covered.end),
            };
            let painted = if live {
                self.painted_live(project, painting, width, height)
                    .map(|(path, _, frame)| (path, Some(frame)))
            } else {
                self.painted(project, painting, width, height)
                    .map(|(path, _)| (path, None))
            };
            let (path, frame) = match painted {
                Ok(painted) => painted,
                Err(error) => {
                    log::warn!("highlight of {}: {error}", title.clip_id);
                    continue;
                }
            };
            let mut clip = base.clone();
            clip.path = path.to_string_lossy().into_owned();
            clip.start = from;
            clip.duration = to - from;
            clip.fade_in = 0.0;
            clip.fade_out = 0.0;
            clip.video_fade_in = 0.0;
            clip.transition = None;
            clip.transition_shapes.clear();
            clip.reveal_map = None;
            // The caption's motion over this word, read on the word's own
            // 0..=1, so the picture moves as the caption moves.
            clip.animation = window_animation(
                &base.animation,
                (from - window_start) / base.duration,
                (to - window_start) / base.duration,
            );
            out.push(TitleClip {
                clip_id: title.clip_id.clone(),
                clip,
                block: title.block,
                offset: title.offset,
                frame,
            });
        }
        out
    }

    /// One style at one size, painted in memory: the name the monitor is
    /// to hold the pixels under, the block, and the pixels.
    fn painted_live(
        &self,
        project: &Project,
        painting: Painting<'_>,
        width: u32,
        height: u32,
    ) -> Result<(PathBuf, Art, Arc<Frame>), String> {
        let key = key_of(project, painting, width, height);
        let path = PathBuf::from(format!("memory://titles/{key:016x}"));
        if let Some((art, frame)) = self.live.lock().unwrap_or_else(|e| e.into_inner()).get(key) {
            return Ok((path, art, frame));
        }
        let rendered = match painting {
            Painting::Title(style) => {
                let title = title_style(style);
                let mut fonts = self.fonts.lock().unwrap_or_else(|e| e.into_inner());
                let fonts = self.fonts_for(project, &mut fonts);
                concat_text::render_frame(fonts, &title, width, height)
                    .map_err(|error| error.to_string())?
            }
            Painting::Shape(style) => {
                concat_text::render_shape_frame(&shape_style(style), width, height)
                    .map_err(|error| error.to_string())?
            }
            Painting::Highlight { style, words } => {
                let title = title_style(style);
                let mut fonts = self.fonts.lock().unwrap_or_else(|e| e.into_inner());
                let fonts = self.fonts_for(project, &mut fonts);
                concat_text::render_highlight_frame(
                    fonts,
                    &title,
                    width,
                    height,
                    words.0..words.1,
                    &style.highlight_color,
                )
                .map_err(|error| error.to_string())?
            }
        };
        let art = Art {
            block: (rendered.block_width, rendered.block_height),
            offset: (rendered.block_dx, rendered.block_dy),
            reveal: reveal_of(&rendered.words, width, height),
        };
        let frame = Arc::new(
            Frame::from_rgba(rendered.width, rendered.height, rendered.rgba)
                .ok_or_else(|| "the painter returned a frame of the wrong size".to_owned())?,
        );
        self.live.lock().unwrap_or_else(|e| e.into_inner()).keep(
            key,
            art.clone(),
            Arc::clone(&frame),
        );
        Ok((path, art, frame))
    }

    /// The PNG for one style at one frame size, painted if it is not on disk
    /// yet, and the block it holds.
    fn painted(
        &self,
        project: &Project,
        painting: Painting<'_>,
        width: u32,
        height: u32,
    ) -> Result<(PathBuf, Art), String> {
        let key = key_of(project, painting, width, height);
        let png = self.dir.join(format!("{key:016x}.png"));
        let side = self.dir.join(format!("{key:016x}.json"));

        if let Some(art) = self
            .memo
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&key)
            && png.is_file()
        {
            return Ok((png, art.clone()));
        }
        // On disk from an earlier session: the sidecar says how big the
        // block was and where, and which words it held, which the PNG
        // alone cannot.
        if png.is_file()
            && let Some(art) = read_block(&side, width, height)
        {
            self.remember(key, art.clone());
            return Ok((png, art));
        }

        let rendered = match painting {
            Painting::Title(style) => {
                let title = title_style(style);
                let mut fonts = self.fonts.lock().unwrap_or_else(|e| e.into_inner());
                let fonts = self.fonts_for(project, &mut fonts);
                concat_text::render(fonts, &title, width, height)
                    .map_err(|error| error.to_string())?
            }
            Painting::Shape(style) => concat_text::render_shape(&shape_style(style), width, height)
                .map_err(|error| error.to_string())?,
            Painting::Highlight { style, words } => {
                let title = title_style(style);
                let mut fonts = self.fonts.lock().unwrap_or_else(|e| e.into_inner());
                let fonts = self.fonts_for(project, &mut fonts);
                concat_text::render_highlight(
                    fonts,
                    &title,
                    width,
                    height,
                    words.0..words.1,
                    &style.highlight_color,
                )
                .map_err(|error| error.to_string())?
            }
        };
        std::fs::create_dir_all(&self.dir).map_err(|error| error.to_string())?;
        std::fs::write(&png, &rendered.png).map_err(|error| error.to_string())?;
        let art = Art {
            block: (rendered.block_width, rendered.block_height),
            offset: (rendered.block_dx, rendered.block_dy),
            reveal: reveal_of(&rendered.words, width, height),
        };
        let words: Vec<String> = rendered
            .words
            .iter()
            .map(|w| format!("[{},{},{},{}]", w.x, w.y, w.width, w.height))
            .collect();
        let _ = std::fs::write(
            &side,
            format!(
                "{{\"w\":{},\"h\":{},\"x\":{},\"y\":{},\"words\":[{}]}}",
                art.block.0,
                art.block.1,
                art.offset.0,
                art.offset.1,
                words.join(",")
            ),
        );
        self.remember(key, art.clone());
        Ok((png, art))
    }

    /// The faces, loaded once, with the project's own files added - each
    /// loaded once for the process.
    fn fonts_for<'a>(&self, project: &Project, held: &'a mut Option<Fonts>) -> &'a Fonts {
        let fonts = held.get_or_insert_with(Fonts::new);
        let mut loaded = self.loaded_files.lock().unwrap_or_else(|e| e.into_inner());
        for font in &project.fonts {
            if !font.path.is_empty() && loaded.insert(font.path.clone()) {
                fonts.add_file(Path::new(&font.path));
            }
        }
        fonts
    }

    fn remember(&self, key: u64, art: Art) {
        self.memo
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(key, art);
    }
}

/// Everything the pixels depend on, hashed. Placement and timing are left
/// out on purpose; see the module docs.
///
/// FNV-1a over the bytes, as the other caches on disk are keyed, because a
/// file's name has to mean the same thing after a toolchain upgrade and the
/// standard hasher makes no such promise.
fn key_of(project: &Project, painting: Painting<'_>, width: u32, height: u32) -> u64 {
    let mut bytes = Vec::new();
    // The style as JSON: every field, in a stable order, with no need to
    // keep a serialiser in step with the struct. A shape's is prefixed, so
    // it can never collide with a title's.
    match painting {
        Painting::Title(style) => {
            bytes.extend_from_slice(serde_json::to_string(style).unwrap_or_default().as_bytes());
        }
        Painting::Shape(style) => {
            bytes.extend_from_slice(b"shape:");
            bytes.extend_from_slice(serde_json::to_string(style).unwrap_or_default().as_bytes());
        }
        // A highlight is its title, which carries the accent, and the words
        // it lights: a different word is a different picture.
        Painting::Highlight { style, words } => {
            bytes.extend_from_slice(b"highlight:");
            bytes.extend_from_slice(serde_json::to_string(style).unwrap_or_default().as_bytes());
            bytes.extend_from_slice(format!("|{}..{}", words.0, words.1).as_bytes());
        }
    }
    bytes.push(0);
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    for font in &project.fonts {
        bytes.extend_from_slice(font.family.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(font.path.as_bytes());
        bytes.push(0);
    }
    // Bumped when the painter's output changes for the same input, so stale
    // files are not mistaken for current ones. 3: left- and right-aligned
    // blocks moved to their anchors. 4: the sidecar gained each word's box,
    // for a per-word reveal effect to read back.
    bytes.extend_from_slice(&4u32.to_le_bytes());
    crate::media::fnv1a(&bytes)
}

/// The caption's keyframes over one word's window, as keys on that window's
/// own `0..=1`, for the word's overlay. `x0` and `x1` are the window's ends on
/// the caption's own `0..=1`. The engine reads a clip's keys against its own
/// window, so these are the same values at the same moments; see
/// `concat_core::animate::Track::window` for how the window is cut.
fn window_animation(keys: &[ExportKey], x0: f64, x1: f64) -> Vec<ExportKey> {
    // The properties a key can move: the engine's own list (`animation_of`).
    const PROPERTIES: [&str; 6] = [
        "scale", "offsetX", "offsetY", "rotation", "opacity", "volume",
    ];
    // Points between the window's ends at which the caption's curve is read.
    const SAMPLES: usize = 16;
    let mut out = Vec::new();
    for property in PROPERTIES {
        let track = Track::new(
            keys.iter()
                .filter(|key| key.property == property)
                .map(|key| Key {
                    at: key.at,
                    value: key.value,
                    ease: Ease::new(key.ease[0], key.ease[1], key.ease[2], key.ease[3]),
                })
                .collect(),
        );
        if track.is_empty() {
            continue;
        }
        out.extend(
            track
                .window(x0, x1, SAMPLES)
                .keys()
                .iter()
                .map(|key| ExportKey {
                    property: property.to_owned(),
                    at: key.at,
                    value: key.value,
                    ease: [key.ease.x1, key.ease.y1, key.ease.x2, key.ease.y2],
                }),
        );
    }
    out
}

/// How many painted titles the directory keeps. Each is a frame-sized PNG,
/// mostly transparent, so this is tens of megabytes at most; above it the
/// oldest go, and a title that is still in use paints again.
const TITLES_KEPT: usize = 400;

/// Removes the oldest painted titles until `keep` remain. Sidecars go with
/// their PNGs. Best effort throughout: a file that will not go is left.
fn sweep(dir: &Path, keep: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut pngs: Vec<(std::time::SystemTime, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .filter_map(|path| {
            let modified = std::fs::metadata(&path).ok()?.modified().ok()?;
            Some((modified, path))
        })
        .collect();
    if pngs.len() <= keep {
        return;
    }
    pngs.sort();
    for (_, png) in pngs.iter().take(pngs.len() - keep) {
        let _ = std::fs::remove_file(png);
        let _ = std::fs::remove_file(png.with_extension("json"));
    }
}

fn read_block(side: &Path, width: u32, height: u32) -> Option<Art> {
    let text = std::fs::read_to_string(side).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let offset = |name: &str| value.get(name).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let rects: Vec<(i32, i32, u32, u32)> = value
        .get("words")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|word| {
            let word = word.as_array()?;
            Some((
                word.first()?.as_i64()? as i32,
                word.get(1)?.as_i64()? as i32,
                word.get(2)?.as_u64()? as u32,
                word.get(3)?.as_u64()? as u32,
            ))
        })
        .collect();
    Some(Art {
        block: (
            value.get("w")?.as_u64()? as u32,
            value.get("h")?.as_u64()? as u32,
        ),
        offset: (offset("x"), offset("y")),
        reveal: Arc::new(RevealMap::from_rects(width, height, &rects)),
    })
}

/// A title's per-word reveal map, baked from the boxes its layout left
/// behind - a byproduct `concat-text` already computed, not new work.
fn reveal_of(words: &[WordRect], width: u32, height: u32) -> Arc<RevealMap> {
    let rects: Vec<(i32, i32, u32, u32)> = words
        .iter()
        .map(|w| (w.x, w.y, w.width, w.height))
        .collect();
    Arc::new(RevealMap::from_rects(width, height, &rects))
}

/// The document's style, field for field, in the painter's terms.
fn title_style(style: &TextStyle) -> TitleStyle {
    TitleStyle {
        // The painter draws the render-time transform (roadmap C4); the
        // document's `content` - the transcript - keeps its casing, and the
        // word-matching above runs on the stored text, so the highlight
        // lands on the words the transcript says.
        content: style.display_content().into_owned(),
        // The caption's way in (roadmap C5): the document's motion copied
        // into the painter's mirror, field for field like the rest of
        // this struct. What drives `progress` - the schema's open
        // question - is the render loop's business, not this copy's.
        motion: concat_text::Motion {
            mode: match style.motion.mode {
                concat_project::model::MotionMode::None => concat_text::MotionMode::None,
                concat_project::model::MotionMode::Fade => concat_text::MotionMode::Fade,
                concat_project::model::MotionMode::Pop => concat_text::MotionMode::Pop,
                concat_project::model::MotionMode::Bounce => concat_text::MotionMode::Bounce,
                concat_project::model::MotionMode::SlideUp => concat_text::MotionMode::SlideUp,
                concat_project::model::MotionMode::WordReveal => concat_text::MotionMode::WordReveal,
                concat_project::model::MotionMode::Typewriter => concat_text::MotionMode::Typewriter,
                concat_project::model::MotionMode::Karaoke => concat_text::MotionMode::Karaoke,
            },
            duration: style.motion.duration,
            stagger: style.motion.stagger,
            scale_from: style.motion.scale_from,
        },
        font_family: style.font_family.clone(),
        font_size: style.font_size,
        font_weight: style.font_weight,
        italic: style.italic,
        color: style.color.clone(),
        align: match style.align {
            TextAlign::Left => Align::Left,
            TextAlign::Center => Align::Center,
            TextAlign::Right => Align::Right,
        },
        stroke_width: style.stroke_width,
        stroke_color: style.stroke_color.clone(),
        shadow: style.shadow,
        background: style.background.clone(),
        background_radius: style.background_radius,
        background_padding_x: style.background_padding_x,
        background_padding_y: style.background_padding_y,
        line_height: style.line_height,
        max_width: style.max_width,
        max_height: style.max_height,
        tracking: style.tracking,
    }
}

/// The document's shape, in the painter's terms.
fn shape_style(style: &ShapeStyle) -> concat_text::ShapeStyle {
    concat_text::ShapeStyle {
        figure: match style.kind {
            ShapeKind::Square => Figure::Square,
            ShapeKind::Circle => Figure::Circle,
            ShapeKind::Triangle => Figure::Triangle,
            ShapeKind::Parallelogram => Figure::Parallelogram,
            ShapeKind::Trapezoid => Figure::Trapezoid,
            ShapeKind::Line => Figure::Line,
            ShapeKind::Arrow => Figure::Arrow,
        },
        size: style.size,
        fill: style.fill.clone(),
        stroke: style.stroke.clone(),
        stroke_width: style.stroke_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use concat_project::{Command, Editor};

    #[test]
    fn word_reveal_emits_timed_prefixes_without_guessing_word_boundaries() {
        let style = TextStyle {
            content: "Halo dunia!".to_owned(),
            motion: concat_project::model::CaptionMotion {
                mode: concat_project::model::MotionMode::WordReveal,
                ..Default::default()
            },
            ..Default::default()
        };
        let words = vec![
            Word { text: "Halo".to_owned(), start: 0.2, end: 0.45 },
            Word { text: "dunia!".to_owned(), start: 0.5, end: 0.9 },
        ];
        let slices = caption_motion_slices(&style, Some(&words), 0.0, 1.2).expect("timed captions");
        assert_eq!(slices.len(), 2);
        assert_eq!((slices[0].start, slices[0].end), (0.2, 0.5));
        assert_eq!(&style.content[..slices[0].text_end], "Halo ");
        assert_eq!(&style.content[..slices[1].text_end], "Halo dunia!");
    }

    #[test]
    fn typewriter_reveals_unicode_character_boundaries_without_transcript_words() {
        let style = TextStyle {
            content: "Halo 世界".to_owned(),
            motion: concat_project::model::CaptionMotion {
                mode: concat_project::model::MotionMode::Typewriter,
                ..Default::default()
            },
            ..Default::default()
        };
        let slices = caption_motion_slices(&style, None, 0.0, 1.0).expect("title entrance");
        let prefixes: Vec<&str> = slices
            .iter()
            .map(|slice| &style.content[..slice.text_end])
            .collect();
        assert!(prefixes.contains(&"H"));
        assert!(prefixes.contains(&"Halo 世"));
        assert_eq!(prefixes.last(), Some(&"Halo 世界"));
        assert!(slices.iter().all(|slice| slice.end > slice.start));
    }

    #[test]
    fn pop_springs_from_the_configured_scale_and_settles_to_one() {
        let style = TextStyle {
            content: "Hello".to_owned(),
            motion: concat_project::model::CaptionMotion {
                mode: concat_project::model::MotionMode::Pop,
                duration: 0.45,
                stagger: 0.09,
                scale_from: 0.35,
            },
            ..Default::default()
        };
        let slices = caption_motion_slices(&style, None, 0.0, 0.8).expect("text entrance");
        assert_eq!(slices.first().map(|slice| slice.scale), Some(0.35));
        assert_eq!(slices.last().map(|slice| slice.scale), Some(1.0));
    }

    #[test]
    fn fade_bounce_and_slide_animate_any_text_clip() {
        use concat_project::model::MotionMode;
        for mode in [MotionMode::Fade, MotionMode::Bounce, MotionMode::SlideUp] {
            let style = TextStyle {
                content: "Make every word move.".to_owned(),
                motion: concat_project::model::CaptionMotion {
                    mode,
                    ..Default::default()
                },
                ..Default::default()
            };
            let slices = caption_motion_slices(&style, None, 1.0, 2.0)
                .expect("untimed titles still animate");
            assert!(slices.len() >= 5, "{mode:?} should have sampled frames");
            assert_eq!(slices.first().map(|slice| slice.opacity), Some(0.0));
            assert_eq!(slices.last().map(|slice| slice.opacity), Some(1.0));
            assert_eq!(slices.first().map(|slice| slice.start), Some(1.0));
        }
    }

    #[test]
    fn karaoke_can_make_an_even_fallback_clock_for_untimed_titles() {
        let words = synthetic_words("Make every word move.", 0.0, 3.2);
        assert_eq!(words.len(), 4);
        assert!((words[0].start - 0.0).abs() < 1e-9);
        assert!((words[0].end - 0.8).abs() < 1e-9);
        assert!((words[3].end - 3.2).abs() < 1e-9);
    }

    fn scratch(name: &str) -> AppDirs {
        let dir =
            std::env::temp_dir().join(format!("concat-titles-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        AppDirs {
            config: dir.join("config"),
            data: dir.join("data"),
            whisper_models: None,
        }
    }

    /// A title clip carrying a stretch - what 0.2.2's side grips wrote to
    /// make a long caption fit - comes back unstretched: the words wrap
    /// now, and a squashed glyph was never wanted.
    /// https://github.com/jub0t/Concat/issues/119
    #[test]
    fn a_title_is_never_stretched_whatever_its_clip_says() {
        let dirs = scratch("title");
        let mut editor = Editor::new();
        let id = editor
            .apply(Command::AddTextClip {
                above: false,
                track_id: None,
                start: 0.0,
                style: None,
                duration: Some(3.0),
                offset_y: None,
                caption_of: None,
                words: None,
            })
            .expect("a title is added")
            .created_id
            .expect("with an id");
        editor
            .apply(Command::SetClipTransform {
                clip_id: id.clone(),
                scale: Some(1.03),
                offset_x: None,
                offset_y: None,
                rotation: None,
                stretch_x: Some(0.46),
                stretch_y: Some(2.2),
            })
            .expect("the stretch a 0.2.2 grip left behind");
        let clip = editor.project().active().clip(&id).expect("kept");
        assert_eq!(
            (clip.stretch_x, clip.stretch_y),
            (0.46, 2.2),
            "it is on the clip"
        );

        let out = Titles::new(&dirs).clips(editor.project(), 640, 360);
        assert_eq!((out[0].clip.stretch_x, out[0].clip.stretch_y), (1.0, 1.0));
        assert_eq!(
            out[0].clip.scale, 1.03,
            "the scale is honoured; only the stretch is not"
        );
    }

    /// A text clip comes back as an image clip on its own track, pointing
    /// at a PNG on disk that is the frame's size, and a second ask for the
    /// same title paints nothing new.
    #[test]
    fn a_title_rejoins_as_a_still() {
        let dirs = scratch("a_title_rejo");
        let mut editor = Editor::new();
        let id = editor
            .apply(Command::AddTextClip {
                above: false,
                track_id: None,
                start: 2.0,
                style: None,
                duration: Some(3.0),
                offset_y: Some(0.3),
                caption_of: None,
                words: None,
            })
            .expect("a title is added")
            .created_id
            .expect("with an id");

        let titles = Titles::new(&dirs);
        let out = titles.clips(editor.project(), 640, 360);
        assert_eq!(out.len(), 1);
        let title = &out[0];
        assert_eq!(title.clip_id, id);
        assert!(matches!(title.clip.kind, ClipKind::Image));
        assert_eq!((title.clip.start, title.clip.duration), (2.0, 3.0));
        assert_eq!(title.clip.offset_y, 0.3);
        assert_eq!(title.clip.media_width, Some(640));
        assert!(title.block.0 > 0 && title.block.1 > 0);
        assert_eq!(
            title.offset,
            (0, 0),
            "a centred title's block is on the clip"
        );
        // "Your text": two words, so their reveal order spans 0 to 255.
        let reveal = title.clip.reveal_map.as_ref().expect("a title reveals");
        assert_eq!((reveal.width, reveal.height), (640, 360));
        assert!(reveal.gray.contains(&0));
        assert!(reveal.gray.contains(&255), "a second word reaches the top");
        let painted = Path::new(&title.clip.path);
        assert!(painted.is_file(), "the PNG is on disk");
        let stamp = std::fs::metadata(painted).and_then(|m| m.modified()).ok();

        // Same words, same frame: same file, untouched.
        let again = titles.clips(editor.project(), 640, 360);
        assert_eq!(again[0].clip.path, title.clip.path);
        assert_eq!(
            std::fs::metadata(painted).and_then(|m| m.modified()).ok(),
            stamp
        );

        // A different frame is a different picture.
        let wide = titles.clips(editor.project(), 1280, 720);
        assert_ne!(wide[0].clip.path, title.clip.path);

        // Aligned left, the block starts at the clip's position and its
        // centre is reported half a block to the right - and a second
        // painter, reading the sidecar cold, says the same.
        let style = concat_project::model::TextStyle {
            align: concat_project::model::TextAlign::Left,
            ..Default::default()
        };
        editor
            .apply(Command::UpdateClip {
                clip_id: id.clone(),
                patch: concat_project::commands::ClipPatch {
                    text: Some(Some(style)),
                    ..Default::default()
                },
            })
            .expect("aligns");
        let left = titles.clips(editor.project(), 640, 360);
        assert!(left[0].offset.0 > 0);
        assert!((left[0].offset.0 as u32).abs_diff(left[0].block.0 / 2) <= 1);
        let cold = Titles::new(&dirs).clips(editor.project(), 640, 360);
        assert_eq!(cold[0].offset, left[0].offset);
        assert_eq!(cold[0].block, left[0].block);
        // The sidecar carries the words too, so a cold read reveals the
        // same order as the one that painted them.
        assert_eq!(
            cold[0].clip.reveal_map.as_ref().map(|r| &*r.gray),
            left[0].clip.reveal_map.as_ref().map(|r| &*r.gray)
        );
        let _ = std::fs::remove_dir_all(dirs.data.parent().unwrap());
    }

    /// A word-timed caption with a highlight colour comes back as itself
    /// plus one overlay per heard word: the same block and place, on that
    /// word's window, with its own picture. A caption with no colour is one
    /// clip, as it was before.
    #[test]
    fn a_word_timed_caption_gets_one_highlight_per_heard_word() {
        let dirs = scratch("a_word_highlight");
        let mut editor = Editor::new();
        let style = TextStyle {
            content: "one two three".to_owned(),
            highlight_color: "#ffd21f".to_owned(),
            ..TextStyle::default()
        };
        let words = vec![
            concat_project::captionmap::Word {
                text: "one".to_owned(),
                start: 2.0,
                end: 2.4,
            },
            concat_project::captionmap::Word {
                text: "two".to_owned(),
                start: 2.4,
                end: 2.9,
            },
            concat_project::captionmap::Word {
                text: "three".to_owned(),
                start: 2.9,
                end: 3.5,
            },
        ];
        let id = editor
            .apply(Command::AddTextClip {
                above: false,
                track_id: None,
                start: 2.0,
                style: Some(style),
                duration: Some(2.0),
                offset_y: Some(0.3),
                caption_of: None,
                words: Some(words.clone()),
            })
            .expect("a caption is added")
            .created_id
            .expect("with an id");

        let out = Titles::new(&dirs).clips(editor.project(), 640, 360);
        assert_eq!(out.len(), 4, "the caption, and one highlight per word");
        assert_eq!(out[0].clip_id, id);
        for (overlay, word) in out[1..].iter().zip(&words) {
            assert_eq!(overlay.clip_id, id);
            assert_eq!(
                (overlay.clip.start, overlay.clip.duration),
                (word.start, word.end - word.start)
            );
            assert_eq!(overlay.block, out[0].block);
            assert_eq!(overlay.offset, out[0].offset);
            assert!(overlay.clip.reveal_map.is_none());
            assert!(Path::new(&overlay.clip.path).is_file());
        }
        assert_ne!(out[1].clip.path, out[2].clip.path, "a word, a picture");

        // No colour, no highlight: the caption alone, as before.
        editor
            .apply(Command::UpdateClip {
                clip_id: id.clone(),
                patch: concat_project::commands::ClipPatch {
                    text: Some(Some(TextStyle {
                        content: "one two three".to_owned(),
                        ..TextStyle::default()
                    })),
                    ..Default::default()
                },
            })
            .expect("the colour is taken off");
        let alone = Titles::new(&dirs).clips(editor.project(), 640, 360);
        assert_eq!(alone.len(), 1);
        let _ = std::fs::remove_dir_all(dirs.data.parent().unwrap());
    }

    /// More titles than the byte budget holds are all kept while the
    /// timeline shows them: a second live pass paints none of them again.
    /// Kept first in, first out, each was thrown out just before it was
    /// wanted, and every pass painted them all (#261).
    #[test]
    fn a_timeline_of_titles_past_the_budget_is_painted_once() {
        let dirs = scratch("live_many");
        let mut editor = Editor::new();
        // Four megabytes a painting at 1024 square: 40 is more than the
        // 128 MB budget holds.
        let count = LIVE_BYTES / (1024 * 1024 * 4) + 8;
        for index in 0..count {
            editor
                .apply(Command::AddShapeClip {
                    above: false,
                    track_id: None,
                    start: index as f64,
                    style: Some(concat_project::model::ShapeStyle {
                        size: 0.1 + index as f64 * 0.001,
                        ..Default::default()
                    }),
                    duration: Some(1.0),
                    name: format!("Shape {index}"),
                })
                .expect("a shape is added");
        }
        let titles = Titles::new(&dirs);
        let first = titles.clips_live(editor.project(), 1024, 1024);
        assert_eq!(first.len(), count);
        let second = titles.clips_live(editor.project(), 1024, 1024);
        let repainted = first
            .iter()
            .zip(&second)
            .filter(|(before, after)| {
                !Arc::ptr_eq(
                    before.frame.as_ref().expect("live"),
                    after.frame.as_ref().expect("live"),
                )
            })
            .count();
        assert_eq!(repainted, 0, "of {count}");
    }

    /// A painting asked for again is the most recently used, and outlives
    /// one asked for less recently; past the ceiling the first painted
    /// stay and the newest is not kept.
    #[test]
    fn live_paintings_go_least_recently_used_first() {
        let art = || Art {
            block: (1, 1),
            offset: (0, 0),
            reveal: Arc::new(RevealMap::from_rects(1, 1, &[])),
        };
        let frame = || Arc::new(Frame::transparent(4, 4));
        let size = frame().pixels().len();
        let mut live = LivePaintings::new(2 * size, 3 * size);

        live.begin();
        live.keep(1, art(), frame());
        live.keep(2, art(), frame());
        live.begin();
        assert!(live.get(1).is_some(), "1 is used again");
        live.begin();
        live.keep(3, art(), frame());
        assert!(live.get(1).is_some(), "the recently used stays");
        assert!(live.get(2).is_none(), "the least recently used goes");

        // One pass asking for more than the budget keeps them all, to the
        // ceiling; past it, the newest is the one not kept.
        live.begin();
        for key in 10..14 {
            live.keep(key, art(), frame());
        }
        assert!(live.get(10).is_some() && live.get(11).is_some() && live.get(12).is_some());
        assert!(live.get(13).is_none());
        assert!(live.bytes <= 3 * size);
    }

    /// A shape clip rejoins the flattened list as a still at its clip's
    /// stretch - that is how a square becomes a rectangle - with the
    /// figure's box reported and no words to reveal.
    #[test]
    fn a_shape_rejoins_as_a_still_and_keeps_its_stretch() {
        let dirs = scratch("shape");
        let mut editor = Editor::new();
        let id = editor
            .apply(Command::AddShapeClip {
                above: true,
                track_id: None,
                start: 0.0,
                style: None,
                duration: Some(3.0),
                name: "Square".to_owned(),
            })
            .expect("a shape is added")
            .created_id
            .expect("with an id");
        editor
            .apply(Command::SetClipTransform {
                clip_id: id.clone(),
                scale: None,
                offset_x: None,
                offset_y: None,
                rotation: None,
                stretch_x: Some(2.0),
                stretch_y: Some(0.5),
            })
            .expect("pulled wide");

        let out = Titles::new(&dirs).clips(editor.project(), 640, 360);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].clip_id, id);
        assert!(matches!(out[0].clip.kind, ClipKind::Image));
        assert_eq!((out[0].clip.stretch_x, out[0].clip.stretch_y), (2.0, 0.5));
        assert_eq!(out[0].block, (126, 126), "35 % of 360, squared");
        assert_eq!(out[0].offset, (0, 0));
        assert!(out[0].clip.reveal_map.is_none());
        assert!(
            Path::new(&out[0].clip.path).is_file(),
            "painted to disk like a title"
        );
    }
}
