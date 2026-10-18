//! C5: caption motion, schema v1 (owner-approved 16 Oct 2026).
//!
//! The plan's prescribed test, in the sandbox's reach: a motion choice
//! survives a project reload - the schema's own words, "this is what
//! lets 'the choice survives a project reload' pass". The preset half of
//! the prescription ("a preset yields motion on apply") lives in the
//! concat crate's presets.rs, written and shipped with its own test but
//! not verifiable here. The two dials clamp in `TextStyle::tidy` and
//! nowhere else - the schema's rule - and an older document, which has
//! no motion at all, reads as `mode = "none"`, so nothing migrates.

use concat_project::commands::{Command, NewMedia};
use concat_project::editor::Editor;
use concat_project::model::{CaptionMotion, MediaKind, MotionMode, TextStyle};

#[test]
fn a_motion_choice_survives_a_project_reload() {
    let mut editor = Editor::new();
    let media_id = editor
        .apply(Command::AddMedia {
            item: NewMedia {
                path: "/v.mp4".to_owned(),
                name: "v.mp4".to_owned(),
                duration: Some(10.0),
                kind: MediaKind::Video,
                width: Some(1920),
                height: Some(1080),
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
    let video_id = editor.project().active().clips[0].id.clone();
    editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 2.0,
            style: Some(TextStyle {
                content: "halo dunia".to_owned(),
                motion: CaptionMotion {
                    mode: MotionMode::Pop,
                    duration: 0.25,
                    scale_from: 1.3,
                },
                ..TextStyle::default()
            }),
            duration: Some(1.0),
            offset_y: Some(0.38),
            caption_of: Some(video_id),
            words: None,
        })
        .expect("adds the caption");

    // The reload: out to the document wire and back in.
    let json = serde_json::to_string(editor.project()).expect("serialises");
    assert!(
        json.contains("word-reveal") || json.contains("pop"),
        "the wire carries the mode: {json}"
    );
    let back: concat_project::model::Project = serde_json::from_str(&json).expect("loads back");
    let caption = back
        .active()
        .clips
        .iter()
        .find(|clip| clip.text.is_some())
        .expect("the caption");
    let motion = &caption.text.as_ref().expect("style").motion;
    assert_eq!(
        motion.mode,
        MotionMode::Pop,
        "the choice survived the reload"
    );
    assert!((motion.duration - 0.25).abs() < 1e-9);
    assert!((motion.scale_from - 1.3).abs() < 1e-9);
}

#[test]
fn an_old_document_loads_as_no_motion() {
    let style: TextStyle =
        serde_json::from_str(r#"{ "content": "Halo" }"#).expect("an old style loads");
    assert_eq!(style.motion.mode, MotionMode::None);
    assert!(
        (style.motion.duration - 0.18).abs() < 1e-9,
        "the schema's default dial"
    );
    assert!((style.motion.scale_from - 1.15).abs() < 1e-9);
}

#[test]
fn an_unreadable_motion_object_loads_as_no_motion() {
    let style: TextStyle =
        serde_json::from_str(r#"{ "content": "x", "motion": { "mode": "fireworks" } }"#)
            .expect("a hand-edited way-in must not fail the whole document");
    assert_eq!(style.motion.mode, MotionMode::None);

    // The schema's own wire names all read.
    for (name, expected) in [
        ("none", MotionMode::None),
        ("pop", MotionMode::Pop),
        ("word-reveal", MotionMode::WordReveal),
        ("typewriter", MotionMode::Typewriter),
    ] {
        let json = format!(r#"{{ "content": "x", "motion": {{ "mode": "{name}" }} }}"#);
        let style: TextStyle = serde_json::from_str(&json).expect("the schema's names read");
        assert_eq!(style.motion.mode, expected, "{name}");
    }
}

#[test]
fn the_two_dials_clamp_in_the_one_place() {
    // The schema's rule: duration and scale_from clamp where every
    // TextStyle number clamps - `tidy` - and nowhere else.
    let wild = TextStyle {
        motion: CaptionMotion {
            mode: MotionMode::Pop,
            duration: 100.0,
            scale_from: 0.0,
        },
        ..TextStyle::default()
    };
    let tidied = wild.tidy();
    assert!(
        (tidied.motion.duration - 2.0).abs() < 1e-9,
        "a two-second cap: {}",
        tidied.motion.duration
    );
    assert!(
        (tidied.motion.scale_from - 1.0).abs() < 1e-9,
        "a pop never starts smaller than life: {}",
        tidied.motion.scale_from
    );

    let nan = TextStyle {
        motion: CaptionMotion {
            mode: MotionMode::Typewriter,
            duration: f64::NAN,
            scale_from: f64::INFINITY,
        },
        ..TextStyle::default()
    };
    let tidied = nan.tidy();
    assert!(
        (tidied.motion.duration - 0.18).abs() < 1e-9,
        "non-finite falls back to the default"
    );
    assert!((tidied.motion.scale_from - 1.15).abs() < 1e-9);
    assert_eq!(
        tidied.motion.mode,
        MotionMode::Typewriter,
        "the mode is not a number and does not clamp"
    );
}

#[test]
fn motion_stays_off_the_wire_while_nothing_is_chosen() {
    let json = serde_json::to_string(&TextStyle::default()).expect("serialises");
    assert!(
        !json.contains("motion"),
        "an unchosen way-in writes nothing: {json}"
    );
}
