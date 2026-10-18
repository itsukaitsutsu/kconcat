//! V1: auto-reframe 16:9 → 9:16 (roadmap V1).
//!
//! Prescribed test: a fixture with a moving subject produces a smooth
//! crop track, with no jitter above a threshold - set here, at the
//! writing, as the plan requires: the raw subject step of the moving
//! fixture is ~0.011 of the frame per sample, and the smoothed interior
//! of a steady ramp must bend far less than one such step (2e-3). The
//! done-when - a talking-head clip becomes a framed vertical export
//! without manual keyframing - runs on the owner's machine, where the
//! masks and the decoder live; what is proven here is the mathematics
//! the plan names: centroid → smoothing → crop keyframes, with the clamp
//! that never shows picture the source does not have.

use concat_project::commands::{Command, NewMedia};
use concat_project::editor::Editor;
use concat_project::model::{KeyProperty, MediaKind, TextStyle};
use concat_project::reframe::{self, OneEuro, ReframePoint};

fn point(t: f64, x: f64) -> ReframePoint {
    ReframePoint { t, x, y: 0.5 }
}

/// One media of `w`×`h` and one 10 s clip of it on the first lane.
fn setup(w: u32, h: u32) -> (Editor, String) {
    let mut editor = Editor::new();
    let media_id = editor
        .apply(Command::AddMedia {
            item: NewMedia {
                path: format!("/vid-{w}x{h}.mp4"),
                name: format!("vid-{w}x{h}.mp4"),
                duration: Some(10.0),
                kind: MediaKind::Video,
                width: Some(w),
                height: Some(h),
                frame_rate: Some(30.0),
                frame_rate_fraction: Some("30/1".to_owned()),
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
                has_audio: true,
                audio_tracks: Vec::new(),
                origin: None,
                color_space: Default::default(),
            },
        })
        .expect("adds media")
        .created_id
        .expect("media id");
    let track_id = editor.project().active().tracks[0].id.clone();
    editor
        .apply(Command::AddClip {
            media_id,
            track_id,
            start: 0.0,
            ripple: false,
        })
        .expect("adds the clip");
    let clip_id = editor.project().active().clips[0].id.clone();
    (editor, clip_id)
}

#[test]
fn the_centroid_of_a_mask_grid_is_its_weighted_centre() {
    // An 8×8 grid with a 2×2 blob at rows 1-2, columns 1-2: pixel centres
    // 1.5 and 2.5, so the weighted centre is 2.0 of 8 = 0.25 on both axes.
    let mut mask = vec![0u8; 64];
    for y in 1..3 {
        for x in 1..3 {
            mask[y * 8 + x] = 200;
        }
    }
    let (cx, cy) = reframe::mask_centroid(&mask, 8, 8).expect("a blob has a centre");
    assert!((cx - 0.25).abs() < 1e-9, "{cx}");
    assert!((cy - 0.25).abs() < 1e-9, "{cy}");
    assert_eq!(
        reframe::mask_centroid(&[0u8; 64], 8, 8),
        None,
        "a mask that saw nobody has no centre"
    );
    assert_eq!(
        reframe::mask_centroid(&mask, 7, 8),
        None,
        "a grid that does not match the mask is refused, not misread"
    );
}

#[test]
fn a_moving_subject_is_followed_smoothly() {
    let visible =
        reframe::visible_fraction((1920, 1080), (1080, 1920)).expect("wide to tall has a pan");
    assert!((visible - 0.31640625).abs() < 1e-9, "{visible}");
    assert_eq!(
        reframe::visible_fraction((1080, 1920), (1080, 1920)),
        None,
        "a frame the source already fills has nothing to pan"
    );

    // 60 samples at 30 fps, a steady ramp 0.2 → 0.7, inside the window.
    let points: Vec<ReframePoint> = (0..60)
        .map(|i| {
            let t = i as f64 / 30.0;
            point(t, 0.2 + 0.5 * (t / 2.0))
        })
        .collect();
    let track = reframe::pan_track(&points, visible, &OneEuro::default());
    assert_eq!(track.len(), points.len());

    // Follows: the track's end is the subject's end, minus at most the
    // filter's lag on a ramp of this speed.
    let last = track.last().expect("a track").1;
    assert!((last - 0.7).abs() < 0.03, "follows the subject: {last}");

    // Smooth: the interior second difference stays under the threshold
    // this test sets (module head): far below one raw step of ~0.0125.
    let jitter = track
        .iter()
        .skip(6)
        .zip(track.iter().skip(7))
        .zip(track.iter().skip(8))
        .take(track.len().saturating_sub(8))
        .map(|((a, b), c)| (c.1 - 2.0 * b.1 + a.1).abs())
        .fold(0.0, f64::max);
    assert!(jitter < 2e-3, "interior jitter {jitter}");

    // Never leaves the frame.
    assert!(
        track
            .iter()
            .all(|(_, c)| *c >= visible / 2.0 - 1e-9 && *c <= 1.0 - visible / 2.0 + 1e-9)
    );
}

#[test]
fn a_steady_subject_with_a_little_noise_holds_still() {
    let visible = 0.31640625;
    // Deterministic wobble of ±0.01 - generous for mask centroids, which
    // are temporally steadier than this in practice.
    let points: Vec<ReframePoint> = (0..120)
        .map(|i| {
            let t = i as f64 / 30.0;
            let noise = ((i * 37) % 11) as f64 - 5.0;
            point(t, 0.5 + noise * 0.002)
        })
        .collect();
    let track = reframe::pan_track(&points, visible, &OneEuro::default());
    let deviation = track
        .iter()
        .skip(10)
        .map(|(_, centre)| (centre - 0.5).abs())
        .fold(0.0, f64::max);
    assert!(
        deviation < 0.008,
        "the wobble is smoothed away: {deviation}"
    );
}

#[test]
fn the_window_never_leaves_the_frame() {
    let visible = 0.31640625;
    // A subject pinned off each edge in turn: the raw positions are
    // impossible, and the track must clamp to the reachable window.
    let points: Vec<ReframePoint> = (0..60)
        .map(|i| point(i as f64 / 30.0, if i < 30 { 0.0 } else { 1.0 }))
        .collect();
    let track = reframe::pan_track(&points, visible, &OneEuro::default());
    assert!(!track.is_empty());
    for (_, centre) in &track {
        assert!(
            *centre >= visible / 2.0 - 1e-9 && *centre <= 1.0 - visible / 2.0 + 1e-9,
            "{centre} leaves the frame"
        );
    }
}

#[test]
fn the_command_writes_the_fill_scale_and_the_pan_keys() {
    let (mut editor, clip_id) = setup(1920, 1080);
    let points: Vec<ReframePoint> = (0..30)
        .map(|i| {
            let t = i as f64 * 10.0 / 29.0;
            point(t, 0.3 + 0.4 * (t / 10.0))
        })
        .collect();
    let outcome = editor
        .apply(Command::AutoReframe {
            clip_id: clip_id.clone(),
            target_width: 1080,
            target_height: 1920,
            points,
        })
        .expect("reframes");
    assert!(outcome.applied);

    let clip = &editor.project().active().clips[0];
    let expected_scale = (1920.0 * 1920.0) / (1080.0 * 1080.0);
    assert!(
        (clip.scale - expected_scale).abs() < 1e-9,
        "the fill scale: {}",
        clip.scale
    );
    let keys: Vec<&_> = clip
        .keys
        .iter()
        .filter(|key| key.property == KeyProperty::OffsetX)
        .collect();
    assert!(!keys.is_empty(), "the pan is keyed");
    assert!(keys.iter().all(|key| (0.0..=1.0).contains(&key.at)));
    // value = (0.5 − centre) × scale, and the centre stays in the window,
    // so no key can ask for picture outside the source.
    let bound = (0.5 - 0.31640625 / 2.0) * expected_scale;
    assert!(
        keys.iter().all(|key| key.value.abs() <= bound + 1e-9),
        "{keys:?}"
    );

    assert!(editor.undo(), "one command, one undo");
    let clip = &editor.project().active().clips[0];
    assert!((clip.scale - 1.0).abs() < 1e-9, "the fit is back");
    assert!(
        clip.keys
            .iter()
            .all(|key| key.property != KeyProperty::OffsetX),
        "and so are the keys gone"
    );
}

#[test]
fn the_refusals_are_one_sentence_each() {
    let (mut editor, clip_id) = setup(1920, 1080);

    assert!(
        editor
            .apply(Command::AutoReframe {
                clip_id: clip_id.clone(),
                target_width: 1080,
                target_height: 1920,
                points: Vec::new(),
            })
            .is_err(),
        "nothing to follow"
    );

    assert!(
        editor
            .apply(Command::AutoReframe {
                clip_id: "nope".to_owned(),
                target_width: 1080,
                target_height: 1920,
                points: vec![point(0.0, 0.5)],
            })
            .is_err(),
        "no such clip"
    );

    // A tall source into the same tall frame: it already fills it.
    let (mut tall_editor, tall_id) = setup(1080, 1920);
    assert!(
        tall_editor
            .apply(Command::AutoReframe {
                clip_id: tall_id,
                target_width: 1080,
                target_height: 1920,
                points: vec![point(0.0, 0.5)],
            })
            .is_err(),
        "nothing to pan"
    );

    // A caption has no media picture size.
    editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 2.0,
            style: Some(TextStyle {
                content: "halo".to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(1.0),
            offset_y: Some(0.38),
            caption_of: Some(clip_id.clone()),
            words: None,
        })
        .expect("adds a caption");
    let caption_id = {
        let timeline = editor.project().active();
        timeline
            .clips
            .iter()
            .find(|clip| clip.id != clip_id)
            .expect("the caption")
            .id
            .clone()
    };
    assert!(
        editor
            .apply(Command::AutoReframe {
                clip_id: caption_id,
                target_width: 1080,
                target_height: 1920,
                points: vec![point(0.0, 0.5)],
            })
            .is_err(),
        "a caption has no picture to reframe"
    );

    // Non-finite input is refused before anything runs.
    assert!(
        editor
            .apply(Command::AutoReframe {
                clip_id,
                target_width: 1080,
                target_height: 1920,
                points: vec![ReframePoint {
                    t: f64::NAN,
                    x: 0.5,
                    y: 0.5
                }],
            })
            .is_err(),
        "NaN is refused, not smoothed"
    );
}
