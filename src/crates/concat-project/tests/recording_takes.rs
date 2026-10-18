//! P4: one take with two microphones gives two audio tracks, one per
//! microphone, that stay in sync - the placement half, which is the half
//! a sandbox can prove. The recording half (two devices open at once,
//! five minutes, clap at both ends) is the owner's machine, and the
//! session machinery for it lives in concat-host's record.rs, written
//! where this sandbox cannot compile it - recorded, not claimed.
//!
//! What these tests pin: each take lands on its OWN new track; the
//! distance between the two landings is exactly the distance between the
//! two first samples (sync survives placement by construction); a
//! negative offset never lands before zero; the whole session is one
//! undo; an empty session places nothing; and a time that is not a
//! number refuses the WHOLE session before anything is applied.

use concat_project::commands::{CommandError, NewMedia};
use concat_project::editor::Editor;
use concat_project::model::MediaKind;
use concat_project::takes::{SessionTake, place_session};

fn take(device: &str, path: &str, seconds: f64, offset: f64) -> SessionTake {
    SessionTake {
        device: device.to_owned(),
        item: NewMedia {
            path: path.to_owned(),
            name: path.rsplit('/').next().unwrap_or(path).to_owned(),
            duration: Some(seconds),
            kind: MediaKind::Audio,
            width: None,
            height: None,
            frame_rate: None,
            frame_rate_fraction: None,
            video_codec: None,
            audio_codec: Some("pcm_s16le".to_owned()),
            has_audio: true,
            audio_tracks: Vec::new(),
            origin: None,
            color_space: Default::default(),
        },
        offset,
    }
}

fn counts(editor: &Editor) -> (usize, usize, usize) {
    let project = editor.project();
    (
        project.media.len(),
        project.active().tracks.len(),
        project.active().clips.len(),
    )
}

#[test]
fn two_takes_land_on_their_own_tracks_in_sync() {
    let mut editor = Editor::new();
    let baseline = counts(&editor);

    // Mic A heard its first sample 120 ms after the roll, mic B 90 ms
    // after: B spoke 30 ms earlier than A, and the timeline must say so.
    let ids = place_session(
        &mut editor,
        10.0,
        vec![
            take("Blue Yeti", "/audio/take-a.wav", 300.0, 0.12),
            take("Rode NT-USB", "/audio/take-b.wav", 300.0, 0.09),
        ],
    )
    .expect("the session lands");
    assert_eq!(ids.len(), 2, "one clip per microphone");

    let after = counts(&editor);
    assert_eq!(after.0, baseline.0 + 2, "both WAVs are in the bin");
    assert_eq!(after.1, baseline.1 + 2, "one new track per microphone");
    assert_eq!(after.2, baseline.2 + 2);

    let timeline = editor.project().active();
    let a = timeline.clip(&ids[0]).expect("clip a");
    let b = timeline.clip(&ids[1]).expect("clip b");
    assert!((a.start - 10.12).abs() < 1e-9, "A at playhead + its offset");
    assert!((b.start - 10.09).abs() < 1e-9, "B at playhead + its offset");
    assert!(
        a.track_id != b.track_id,
        "each microphone gets its own track"
    );
    assert!(
        ((a.start - b.start) - 0.03).abs() < 1e-9,
        "the distance between the landings is the distance between the
         first samples: placement cannot break the sync"
    );
    assert!(
        (a.duration - 300.0).abs() < 1e-9,
        "the clip plays the whole take"
    );
}

#[test]
fn a_first_sample_before_the_roll_never_lands_before_zero() {
    let mut editor = Editor::new();

    // A fast device can hear before the roll is reported; the playhead
    // can also sit near zero. Either way the timeline starts at zero.
    let ids = place_session(
        &mut editor,
        0.01,
        vec![take("Fast mic", "/audio/take.wav", 60.0, -0.5)],
    )
    .expect("the session lands");

    let clip = editor.project().active().clip(&ids[0]).expect("clip");
    assert_eq!(clip.start, 0.0, "clamped, never before the timeline");
}

#[test]
fn one_undo_restores_the_whole_session() {
    let mut editor = Editor::new();
    let baseline = counts(&editor);

    place_session(
        &mut editor,
        4.0,
        vec![
            take("Mic A", "/audio/a.wav", 120.0, 0.1),
            take("Mic B", "/audio/b.wav", 120.0, 0.2),
        ],
    )
    .expect("the session lands");
    assert_eq!(counts(&editor).1, baseline.1 + 2);

    assert!(editor.undo(), "the whole session is ONE undo step");
    assert_eq!(
        counts(&editor),
        baseline,
        "tracks, clips and bin items all came back off together"
    );
}

#[test]
fn an_empty_session_places_nothing() {
    let mut editor = Editor::new();
    let baseline = counts(&editor);

    let ids = place_session(&mut editor, 2.0, Vec::new()).expect("accepts");
    assert!(ids.is_empty());
    assert_eq!(counts(&editor), baseline, "no take, no track, no undo");
}

#[test]
fn a_time_that_is_not_a_number_refuses_the_whole_session() {
    let mut editor = Editor::new();
    let baseline = counts(&editor);

    let err = place_session(
        &mut editor,
        5.0,
        vec![
            take("Mic A", "/audio/a.wav", 60.0, 0.1),
            take("Mic B", "/audio/b.wav", 60.0, f64::NAN),
        ],
    )
    .expect_err("refused");
    assert!(matches!(err, CommandError::NotANumber));
    assert_eq!(
        counts(&editor),
        baseline,
        "all or nothing: the good take did not land either"
    );
}
