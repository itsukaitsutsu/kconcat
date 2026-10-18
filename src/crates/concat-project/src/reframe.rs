// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! V1 auto-reframe: a wide clip becomes a framed vertical one, following
//! the subject (roadmap V1).
//!
//! The plan's approach, in the order it names: per-frame mask centroid ->
//! smoothing -> crop keyframes. What lives here is the mathematics, all of
//! it testable without a GPU or a decoder:
//!
//! - [`mask_centroid`], the per-frame primitive: an eight-bit mask grid
//!   (what `concat-vision`'s RVM store keeps on disk) to one normalised
//!   point.
//! - [`pan_track`], the smoothing: a one-euro filter - the plan's first
//!   choice - over the window centres, with the clamp that never lets the
//!   crop window leave the source frame.
//! - [`reframe_plan`], the crop keyframes: the constant fill scale plus
//!   one `KeyProperty::OffsetX` key per smoothed centre, in the model's
//!   own units (`Clip::scale` is a multiplier over the fitted size;
//!   `offset_x` is a fraction of frame width off centred).
//!
//! The masks themselves, the decode that feeds them and the sign of the
//! offset in the painter are the owner-machine side; the offset convention
//! written here - content shifted right by `(0.5 - centre) * scale` frame
//! widths puts the source's `centre` at the frame's middle - is recorded
//! for that confirmation.
//!
//! The one-euro parameters are starting guesses with the T3 flag on them:
//! tune on the owner's real footage, against the plan's acceptance (a
//! talking-head clip framed without manual keyframing, no jitter above a
//! threshold).

use serde::{Deserialize, Serialize};

use crate::model::{Clip, ClipKey, KeyEase, KeyProperty, ranges};

/// One subject position: when, and where in the source frame, normalised
/// `0..1` with `(0, 0)` the top-left corner.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReframePoint {
    /// Timeline seconds.
    pub t: f64,
    /// The subject's horizontal centre, `0..1` of the source width.
    pub x: f64,
    /// The subject's vertical centre; the v1 pan is horizontal (a wide
    /// clip filling a tall frame overflows left and right), and `y` is
    /// carried for the day the vertical crop joins.
    pub y: f64,
}

/// The one-euro filter's three dials. Starting guesses, flagged for
/// tuning on real footage (see the module head).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct OneEuro {
    /// The slowest cutoff, in Hz: how still the track is when the subject
    /// is still.
    pub min_cutoff_hz: f64,
    /// How much the cutoff rises with speed: the lag-versus-jitter dial.
    pub beta: f64,
    /// The cutoff of the derivative's own smoother, in Hz.
    pub d_cutoff_hz: f64,
}

impl Default for OneEuro {
    fn default() -> Self {
        OneEuro {
            min_cutoff_hz: 0.5,
            beta: 10.0,
            d_cutoff_hz: 1.0,
        }
    }
}

/// What the reframe writes: one constant scale - the fill - and the pan's
/// keys.
#[derive(Clone, PartialEq, Debug)]
pub struct ReframePlan {
    /// The multiplier over the fitted size that makes the source fill the
    /// target frame's height; within [`ranges::MIN_SCALE`]..[`ranges::MAX_SCALE`].
    pub scale: f64,
    /// `OffsetX` keys, sorted by `at`, ready for [`Clip::set_key`].
    pub keys: Vec<ClipKey>,
}

/// The weighted centre of an eight-bit mask grid - one frame's person
/// mask, as `concat-vision` stores it - normalised to `0..1`. `None` when
/// the grid is empty, the size does not match, or the mask saw nobody.
pub fn mask_centroid(mask: &[u8], width: usize, height: usize) -> Option<(f64, f64)> {
    if width == 0 || height == 0 || mask.len() != width * height {
        return None;
    }
    let mut total = 0.0f64;
    let mut sum_x = 0.0f64;
    let mut sum_y = 0.0f64;
    for (index, &value) in mask.iter().enumerate() {
        if value == 0 {
            continue;
        }
        let weight = f64::from(value);
        total += weight;
        // Pixel centres, not pixel corners: a one-pixel blob in the first
        // column sits at half a pixel, not at zero.
        sum_x += weight * ((index % width) as f64 + 0.5);
        sum_y += weight * ((index / width) as f64 + 0.5);
    }
    if total <= 0.0 {
        return None;
    }
    Some((sum_x / total / width as f64, sum_y / total / height as f64))
}

/// How much of the source width the target frame shows once the source
/// fills the frame's height: the target's aspect over the source's. Below
/// one, the picture overflows left and right and there is a pan to
/// choose; one or above, the source is narrower than the frame and this
/// v1's wide-to-tall reframe has nothing to do (`None` for a degenerate
/// size, `None` for nothing to pan).
pub fn visible_fraction(source: (u32, u32), target: (u32, u32)) -> Option<f64> {
    let (source_w, source_h) = source;
    let (target_w, target_h) = target;
    if source_w == 0 || source_h == 0 || target_w == 0 || target_h == 0 {
        return None;
    }
    let fraction =
        (f64::from(target_w) / f64::from(target_h)) / (f64::from(source_w) / f64::from(source_h));
    // One or above: the source is as narrow as the frame or narrower, and
    // this v1's wide-to-tall reframe has nothing to pan.
    (fraction < 1.0).then_some(fraction)
}

/// The smoothed crop-window track: the subject positions clamped to the
/// window the frame can show, put through the one-euro filter, and
/// clamped again - a filter's overshoot must not show picture the source
/// does not have. Times must rise; a point that does not is skipped.
pub fn pan_track(points: &[ReframePoint], visible: f64, filter: &OneEuro) -> Vec<(f64, f64)> {
    if !visible.is_finite() || visible <= 0.0 || visible >= 1.0 {
        return Vec::new();
    }
    let half = visible / 2.0;
    let mut samples: Vec<(f64, f64)> = Vec::with_capacity(points.len());
    let mut last_t = f64::NEG_INFINITY;
    for point in points {
        // Times must rise: a point that does not is a caller's duplicate,
        // and the filter's derivative would divide by zero on it.
        if !(point.t.is_finite() && point.x.is_finite()) || point.t <= last_t {
            continue;
        }
        last_t = point.t;
        samples.push((point.t, point.x.clamp(half, 1.0 - half)));
    }
    if samples.is_empty() {
        return Vec::new();
    }
    one_euro(&samples, filter)
        .into_iter()
        // Clamped again: a low-pass can overshoot past a clamp its input
        // respected, and an overshoot here is picture the source lacks.
        .map(|(t, centre)| (t, centre.clamp(half, 1.0 - half)))
        .collect()
}

/// The whole plan: the fill scale and one `OffsetX` key per smoothed
/// centre, at the clip's own `0..=1`. `None` when the frame has nothing
/// to pan (the source already fits) or no usable point was given. The
/// offset convention, in the model's units: content shifted right by
/// `(0.5 - centre) * scale` frame widths puts the source's `centre` at
/// the frame's middle.
pub fn reframe_plan(
    clip: &Clip,
    source: (u32, u32),
    target: (u32, u32),
    points: &[ReframePoint],
) -> Option<ReframePlan> {
    let visible = visible_fraction(source, target)?;
    if !clip.duration.is_finite() || clip.duration <= 0.0 || !clip.start.is_finite() {
        return None;
    }
    let track = pan_track(points, visible, &OneEuro::default());
    if track.is_empty() {
        return None;
    }
    let scale = (1.0 / visible).clamp(ranges::MIN_SCALE, ranges::MAX_SCALE);
    let mut keys: Vec<ClipKey> = Vec::with_capacity(track.len());
    for (t, centre) in track {
        keys.push(ClipKey {
            property: KeyProperty::OffsetX,
            at: ((t - clip.start) / clip.duration).clamp(0.0, 1.0),
            // The model's own units: `offset_x` is a fraction of frame
            // width off centred, and `scale` multiplies the fitted size,
            // so shifting the content right by (0.5 - centre) * scale
            // frame widths puts the source's `centre` at the middle.
            value: (0.5 - centre) * scale,
            ease: KeyEase::LINEAR,
        });
    }
    // Consecutive keys at one instant: the later centre wins, the same
    // rule `Clip::set_key` applies, stated once here so the plan and the
    // clip can never disagree.
    let mut merged: Vec<ClipKey> = Vec::with_capacity(keys.len());
    for key in keys {
        match merged.last_mut() {
            Some(last) if (last.at - key.at).abs() <= crate::model::KEY_EPSILON => {
                *last = key;
            }
            _ => merged.push(key),
        }
    }
    Some(ReframePlan {
        scale,
        keys: merged,
    })
}

/// The one-euro filter over `(time, value)` pairs, the classic three
/// equations: the derivative's own low-pass, the cutoff rising with the
/// filtered speed, the value's low-pass at that cutoff.
fn one_euro(samples: &[(f64, f64)], filter: &OneEuro) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = Vec::with_capacity(samples.len());
    let mut x_hat = 0.0;
    let mut dx_hat = 0.0;
    for (index, &(t, x)) in samples.iter().enumerate() {
        if index == 0 {
            // Nothing to derive from and nothing to lag behind: the first
            // sample is the first estimate.
            x_hat = x;
            out.push((t, x));
            continue;
        }
        let dt = t - samples[index - 1].0;
        // The derivative of the raw signal, itself low-passed (the
        // classic form: the speed estimate must not inherit the noise the
        // value filter is there to remove).
        let dx = (x - samples[index - 1].1) / dt;
        let d_alpha = alpha(filter.d_cutoff_hz, dt);
        dx_hat += d_alpha * (dx - dx_hat);
        // The cutoff rises with the filtered speed: stillness smooths
        // hard, motion lets the track through.
        let cutoff = filter.min_cutoff_hz + filter.beta * dx_hat.abs();
        let a = alpha(cutoff, dt);
        x_hat += a * (x - x_hat);
        out.push((t, x_hat));
    }
    out
}

fn alpha(cutoff_hz: f64, dt: f64) -> f64 {
    let tau = 1.0 / (std::f64::consts::TAU * cutoff_hz.max(1e-6));
    1.0 / (1.0 + tau / dt.max(1e-9))
}
