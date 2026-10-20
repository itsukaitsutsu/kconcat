//! Typed V/A lanes reject incompatible placements while old projects keep
//! loading their untyped lanes as permissive mixed lanes.

use concat_project::commands::{Command, NewMedia};
use concat_project::editor::Editor;
use concat_project::model::{MediaKind, TrackKind};

fn video_media() -> NewMedia {
    NewMedia {
        path: "/typed-lane.mp4".to_owned(),
        name: "typed-lane.mp4".to_owned(),
        duration: Some(8.0),
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
    }
}

#[test]
fn missing_lane_kind_migrates_to_permissive_legacy_mixed() {
    let old: concat_project::model::Track = serde_json::from_value(serde_json::json!({
        "id": "old-track",
        "visible": true,
        "muted": false
    }))
    .expect("legacy lane loads");
    assert_eq!(old.kind, TrackKind::LegacyMixed);
    assert!(old.kind.accepts_clip(concat_project::model::ClipKind::Video));
    assert!(old.kind.accepts_clip(concat_project::model::ClipKind::Audio));
    let saved = serde_json::to_value(old).expect("legacy lane serializes");
    assert!(saved.get("kind").is_none(), "legacy wire shape stays stable");
}

#[test]
fn typed_audio_lane_rejects_video_and_persists_its_kind() {
    let mut editor = Editor::new();
    let media_id = editor
        .apply(Command::AddMedia { item: video_media() })
        .expect("adds video media")
        .created_id
        .expect("media id");
    let track_id = editor
        .apply(Command::AddTypedTrack {
            bottom: false,
            kind: TrackKind::Audio,
        })
        .expect("adds typed audio lane")
        .created_id
        .expect("track id");

    let outcome = editor.apply(Command::AddClip {
        media_id,
        track_id: track_id.clone(),
        start: 0.0,
        ripple: false,
    });
    assert_eq!(
        outcome.expect_err("video cannot land on an audio-only lane"),
        concat_project::commands::CommandError::IncompatibleTrack
    );
    let track = editor.project().active().track(&track_id).expect("track exists");
    assert_eq!(track.kind, TrackKind::Audio);
    let wire = serde_json::to_value(track).expect("typed lane serializes");
    assert_eq!(wire.get("kind").and_then(|v| v.as_str()), Some("audio"));
}
