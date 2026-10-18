//! T6: the caption track's ±10 ms sync nudge.
//!
//! Prescribed by the roadmap: "a shift is applied, and survives
//! regeneration." The nudge lives on the track (`Track::caption_nudge_ms`),
//! so when a caption run is regenerated - old clips removed, new ones added
//! to the same lane - the correction is still there and the new captions are
//! placed already shifted.

use concat_project::captionmap::Word;
use concat_project::commands::{Command, NewMedia};
use concat_project::editor::Editor;
use concat_project::model::{MediaKind, TextStyle, Track};

fn setup() -> Editor {
    let mut editor = Editor::new();
    let media_id = editor
        .apply(Command::AddMedia {
            item: NewMedia {
                path: "/a.mp4".to_owned(),
                name: "a.mp4".to_owned(),
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
        .expect("adds clip");
    editor
}

fn word(text: &str, start: f64, end: f64) -> Word {
    Word {
        text: text.to_owned(),
        start,
        end,
    }
}

/// One imported-style caption at 2.0 s, words at the back half of the cue.
/// Returns (caption id, its track id).
fn add_caption(editor: &mut Editor, track_id: Option<String>, start: f64) -> (String, String) {
    let id = editor
        .apply(Command::AddTextClip {
            above: true,
            track_id,
            start,
            style: Some(TextStyle {
                content: "halo dunia".to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(2.0),
            offset_y: Some(0.38),
            caption_of: None,
            words: Some(vec![
                word("halo", start + 1.5, start + 1.7),
                word("dunia", start + 1.7, start + 1.9),
            ]),
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id");
    let track = editor
        .project()
        .active()
        .clip(&id)
        .expect("caption")
        .track_id
        .clone();
    (id, track)
}

#[test]
fn a_nudge_shifts_the_tracks_captions_and_their_words() {
    let mut editor = setup();
    let (caption_id, track_id) = add_caption(&mut editor, None, 2.0);

    for _ in 0..3 {
        editor
            .apply(Command::NudgeCaptionTrack {
                track_id: track_id.clone(),
                delta_ms: 10,
            })
            .expect("nudges");
    }

    let clip = editor
        .project()
        .active()
        .clip(&caption_id)
        .expect("caption");
    assert!(
        (clip.start - 2.03).abs() < 1e-9,
        "three +10 ms steps move the cue 30 ms: {}",
        clip.start
    );
    let words = clip.words.as_ref().expect("words ride along");
    assert!((words[0].start - 3.53).abs() < 1e-9, "{}", words[0].start);
    assert!((words[1].end - 3.93).abs() < 1e-9, "{}", words[1].end);
    assert_eq!(
        editor
            .project()
            .active()
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .expect("track")
            .caption_nudge_ms,
        30,
        "the lane remembers"
    );

    // Undo is whole-state, so one undo restores both the clips and the lane.
    editor.undo();
    let clip = editor
        .project()
        .active()
        .clip(&caption_id)
        .expect("caption");
    assert!((clip.start - 2.02).abs() < 1e-9, "undo took one step back");
}

#[test]
fn a_nudge_quantizes_to_ten_ms_steps() {
    let mut editor = setup();
    let (_caption, track_id) = add_caption(&mut editor, None, 2.0);
    let nudge = |editor: &mut Editor, delta_ms: i32| {
        editor
            .apply(Command::NudgeCaptionTrack {
                track_id: track_id.clone(),
                delta_ms,
            })
            .expect("nudges");
        editor
            .project()
            .active()
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .expect("track")
            .caption_nudge_ms
    };
    assert_eq!(nudge(&mut editor, 14), 10, "14 rounds to one step");
    assert_eq!(nudge(&mut editor, -24), -10, "-24 rounds to two steps back");
    assert_eq!(
        nudge(&mut editor, 15),
        10,
        "a tie rounds away from zero: -10 + 20"
    );
    assert_eq!(nudge(&mut editor, 4), 10, "4 rounds to no step at all");
}

#[test]
fn the_accumulated_nudge_is_capped() {
    let mut editor = setup();
    let (caption_id, track_id) = add_caption(&mut editor, None, 2.0);
    editor
        .apply(Command::NudgeCaptionTrack {
            track_id: track_id.clone(),
            delta_ms: 5_000,
        })
        .expect("nudges");
    assert_eq!(
        editor
            .project()
            .active()
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .expect("track")
            .caption_nudge_ms,
        1_000,
        "the cap is a second either way"
    );
    let clip = editor
        .project()
        .active()
        .clip(&caption_id)
        .expect("caption");
    assert!(
        (clip.start - 3.0).abs() < 1e-9,
        "the cue moved exactly the applied cap: {}",
        clip.start
    );

    // A second huge nudge applies nothing further - and reports no change.
    let outcome = editor
        .apply(Command::NudgeCaptionTrack {
            track_id: track_id.clone(),
            delta_ms: 5_000,
        })
        .expect("nudges");
    assert!(!outcome.applied, "nothing left to apply at the cap");
    let clip = editor
        .project()
        .active()
        .clip(&caption_id)
        .expect("caption");
    assert!(
        (clip.start - 3.0).abs() < 1e-9,
        "the cue did not move twice"
    );
}

#[test]
fn a_nudge_survives_regeneration() {
    let mut editor = setup();
    let (caption_id, track_id) = add_caption(&mut editor, None, 2.0);
    editor
        .apply(Command::NudgeCaptionTrack {
            track_id: track_id.clone(),
            delta_ms: 30,
        })
        .expect("nudges");

    // Regenerate: the run's captions go, and a new run's captions arrive on
    // the same lane at the same nominal place.
    editor
        .apply(Command::RemoveClips {
            clip_ids: vec![caption_id],
            ripple: false,
        })
        .expect("removes");
    let (new_id, new_track) = add_caption(&mut editor, Some(track_id.clone()), 2.0);
    assert_eq!(
        new_track, track_id,
        "the regeneration landed on the nudged lane"
    );

    let clip = editor.project().active().clip(&new_id).expect("caption");
    assert!(
        (clip.start - 2.03).abs() < 1e-9,
        "the fresh caption is born already nudged: {}",
        clip.start
    );
    let words = clip.words.as_ref().expect("words");
    assert!(
        (words[0].start - 3.53).abs() < 1e-9,
        "and so are its word timings: {}",
        words[0].start
    );
    assert_eq!(
        editor
            .project()
            .active()
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .expect("track")
            .caption_nudge_ms,
        30,
        "the lane still remembers"
    );
}

#[test]
fn a_track_saved_before_the_nudge_loads_with_zero() {
    // A document from before T6: no captionNudgeMs key at all.
    let track: Track = serde_json::from_value(serde_json::json!({
        "id": "t1",
        "visible": true,
        "muted": false,
    }))
    .expect("an older track still loads");
    assert_eq!(track.caption_nudge_ms, 0, "absent means never nudged");

    // And a nudged track round-trips its correction.
    let nudged = Track {
        id: "t2".to_owned(),
        caption_nudge_ms: -30,
        ..Track::default()
    };
    let value = serde_json::to_value(&nudged).expect("serialises");
    assert_eq!(
        value.get("captionNudgeMs").and_then(|v| v.as_i64()),
        Some(-30)
    );
    let back: Track = serde_json::from_value(value).expect("loads");
    assert_eq!(back.caption_nudge_ms, -30);

    // An un-nudged track stays byte-identical to its pre-T6 shape: the key
    // is skipped at zero.
    let value = serde_json::to_value(Track::default()).expect("serialises");
    assert!(value.get("captionNudgeMs").is_none());
}
