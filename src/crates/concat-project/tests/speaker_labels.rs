//! P3: speaker labels (owner decisions of 16 Oct 2026).
//!
//! What is decided and built here: a speaker is a label - on the audio
//! clip, on the transcript's words, on each caption - with one caption
//! lane per speaker (D7), the colour belonging to the speaker, tags
//! editable, and everything opt-in: with the registry empty, which is
//! every older document and every project until the user asks, nothing
//! in the editing flow changes. The detection engine is NOT chosen yet
//! (sherpa-onnx or speakrs, waiting on the owner's ground truth), so no
//! detection runs in these tests - the manual route, which the roadmap
//! says "creates the same speaker lanes", is what is tested. Word-level
//! moves compose from what exists: split at the word (E4), assign the
//! piece.

use concat_project::captionmap::Word;
use concat_project::commands::{Command, NewMedia};
use concat_project::editor::Editor;
use concat_project::model::{MediaKind, TextStyle};

fn word(text: &str, start: f64, end: f64) -> Word {
    Word {
        text: text.to_owned(),
        start,
        end,
    }
}

/// A 10 s video with two heard captions over it: "halo dunia" 2.0-3.0
/// and "apa kabar" 5.0-6.0 - one utterance each, as a caption is.
fn setup() -> (Editor, String, String, String) {
    let mut editor = Editor::new();
    let media_id = editor
        .apply(Command::AddMedia {
            item: NewMedia {
                path: "/ep.mp4".to_owned(),
                name: "ep.mp4".to_owned(),
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
        .expect("adds the video");
    let video_id = editor.project().active().clips[0].id.clone();
    let mut captions = Vec::new();
    for (content, start, words) in [
        (
            "halo dunia",
            2.0,
            vec![word("halo", 2.0, 2.4), word("dunia", 2.5, 3.0)],
        ),
        (
            "apa kabar",
            5.0,
            vec![word("apa", 5.0, 5.3), word("kabar", 5.4, 6.0)],
        ),
    ] {
        let id = editor
            .apply(Command::AddTextClip {
                above: true,
                track_id: None,
                start,
                style: Some(TextStyle {
                    content: content.to_owned(),
                    ..TextStyle::default()
                }),
                duration: Some(1.0),
                offset_y: Some(0.38),
                caption_of: Some(video_id.clone()),
                words: Some(words),
            })
            .expect("adds the caption")
            .created_id
            .expect("caption id");
        captions.push(id);
    }
    let [first, second] = [captions.remove(0), captions.remove(0)];
    (editor, video_id, first, second)
}

fn add_speaker(editor: &mut Editor, name: &str, color: &str) -> String {
    editor
        .apply(Command::AddSpeaker {
            name: name.to_owned(),
            color: color.to_owned(),
        })
        .expect("adds the speaker")
        .created_id
        .expect("speaker id")
}

#[test]
fn a_speaker_is_minted_named_and_its_tags_stay_editable() {
    let (mut editor, _, _, _) = setup();
    assert!(
        editor.project().active().speakers.is_empty(),
        "opt-in: a fresh project has no speakers and nothing runs"
    );
    let host = add_speaker(&mut editor, " Host ", "#ff0000");
    {
        let speakers = &editor.project().active().speakers;
        assert_eq!(speakers.len(), 1);
        assert_eq!(speakers[0].id, host);
        assert_eq!(speakers[0].name, "Host", "the name is trimmed");
        assert_eq!(speakers[0].color, "#ff0000");
    }
    // Tags are editable (owner 16 Oct 2026).
    editor
        .apply(Command::RenameSpeaker {
            speaker_id: host.clone(),
            name: "Presenter".to_owned(),
        })
        .expect("renames");
    editor
        .apply(Command::SetSpeakerColor {
            speaker_id: host.clone(),
            color: "#00ff00".to_owned(),
        })
        .expect("recolours");
    let speaker = &editor.project().active().speakers[0];
    assert_eq!(speaker.name, "Presenter");
    assert_eq!(speaker.color, "#00ff00");
    // A blank name is refused - the label is what the user reads.
    assert!(
        editor
            .apply(Command::RenameSpeaker {
                speaker_id: host.clone(),
                name: "   ".to_owned(),
            })
            .is_err()
    );
    assert!(
        editor
            .apply(Command::AddSpeaker {
                name: String::new(),
                color: String::new(),
            })
            .is_err()
    );
}

#[test]
fn assigning_a_caption_moves_it_to_the_speakers_lane_and_labels_it() {
    let (mut editor, _, first, second) = setup();
    let host = add_speaker(&mut editor, "Host", "#ff0000");
    let guest = add_speaker(&mut editor, "Guest", "#0000ff");
    let tracks_before = editor.project().active().tracks.len();
    let lane_before = {
        let timeline = editor.project().active();
        timeline
            .clips
            .iter()
            .find(|clip| clip.id == first)
            .expect("the caption")
            .track_id
            .clone()
    };
    let words_before = editor
        .project()
        .active()
        .clips
        .iter()
        .find(|clip| clip.id == first)
        .and_then(|clip| clip.words.clone())
        .expect("heard words");

    editor
        .apply(Command::AssignCaptionToSpeaker {
            clip_id: first.clone(),
            speaker_id: host.clone(),
        })
        .expect("assigns");

    let timeline = editor.project().active();
    assert_eq!(
        timeline.tracks.len(),
        tracks_before + 1,
        "D7: one lane, made on first use"
    );
    let lane = timeline.tracks.last().expect("the new lane");
    assert_eq!(lane.speaker.as_deref(), Some(host.as_str()));
    let caption = timeline
        .clips
        .iter()
        .find(|clip| clip.id == first)
        .expect("caption");
    assert_eq!(
        caption.track_id, lane.id,
        "the caption moved to the speaker's lane"
    );
    assert_ne!(caption.track_id, lane_before);
    assert_eq!(
        caption.speaker.as_deref(),
        Some(host.as_str()),
        "the label is on the caption"
    );
    assert_eq!(
        caption.words.as_ref().expect("words"),
        &words_before,
        "the heard timings stand as they are - every word inherits the label through the caption"
    );
    assert_eq!(
        caption.text.as_ref().expect("style").content,
        "halo dunia",
        "and so does the transcript"
    );

    // The second caption, same speaker: the same lane, not a new one.
    editor
        .apply(Command::AssignCaptionToSpeaker {
            clip_id: second.clone(),
            speaker_id: host.clone(),
        })
        .expect("assigns the second");
    assert_eq!(
        editor.project().active().tracks.len(),
        tracks_before + 1,
        "one lane per speaker"
    );

    // A different speaker gets a lane of its own, and the label follows
    // the last assignment.
    editor
        .apply(Command::AssignCaptionToSpeaker {
            clip_id: second.clone(),
            speaker_id: guest.clone(),
        })
        .expect("reassigns to the guest");
    let timeline = editor.project().active();
    assert_eq!(
        timeline.tracks.len(),
        tracks_before + 2,
        "the guest's own lane"
    );
    let caption = timeline
        .clips
        .iter()
        .find(|clip| clip.id == second)
        .expect("caption");
    assert_eq!(
        caption.speaker.as_deref(),
        Some(guest.as_str()),
        "the label follows the last assignment"
    );
}

#[test]
fn one_undo_restores_the_assignment_whole() {
    let (mut editor, _, first, _) = setup();
    let host = add_speaker(&mut editor, "Host", "#ff0000");
    let tracks_before = editor.project().active().tracks.len();
    let lane_before = editor
        .project()
        .active()
        .clips
        .iter()
        .find(|clip| clip.id == first)
        .expect("caption")
        .track_id
        .clone();

    editor
        .apply(Command::AssignCaptionToSpeaker {
            clip_id: first.clone(),
            speaker_id: host.clone(),
        })
        .expect("assigns");
    assert!(editor.undo(), "one command, one undo");

    let timeline = editor.project().active();
    assert_eq!(
        timeline.tracks.len(),
        tracks_before,
        "the made lane goes with the undo"
    );
    let caption = timeline
        .clips
        .iter()
        .find(|clip| clip.id == first)
        .expect("caption");
    assert_eq!(
        caption.track_id, lane_before,
        "the caption is back on its lane"
    );
    assert_eq!(caption.speaker, None, "and carries no label");
    assert_eq!(
        timeline.speakers.len(),
        1,
        "the speaker itself was an earlier command and stays"
    );
}

#[test]
fn a_speaker_in_use_is_not_removed_and_an_unused_one_is() {
    let (mut editor, _, first, _) = setup();
    let host = add_speaker(&mut editor, "Host", "#ff0000");
    editor
        .apply(Command::AssignCaptionToSpeaker {
            clip_id: first.clone(),
            speaker_id: host.clone(),
        })
        .expect("assigns");

    let refused = editor.apply(Command::RemoveSpeaker {
        speaker_id: host.clone(),
    });
    assert!(
        refused.is_err(),
        "a speaker never vanishes from under its captions"
    );

    assert!(editor.undo(), "take the assignment back");
    let outcome = editor
        .apply(Command::RemoveSpeaker {
            speaker_id: host.clone(),
        })
        .expect("removes the unused speaker");
    assert!(outcome.applied);
    assert!(editor.project().active().speakers.is_empty());
}

#[test]
fn unknown_ids_and_picture_clips_are_refused_with_a_sentence() {
    let (mut editor, video, first, _) = setup();
    let tracks_before = editor.project().active().tracks.len();
    let host = add_speaker(&mut editor, "Host", "#ff0000");

    for command in [
        Command::AssignCaptionToSpeaker {
            clip_id: video.clone(),
            speaker_id: host.clone(),
        },
        Command::AssignCaptionToSpeaker {
            clip_id: "nope".to_owned(),
            speaker_id: host.clone(),
        },
        Command::AssignCaptionToSpeaker {
            clip_id: first.clone(),
            speaker_id: "nope".to_owned(),
        },
        Command::RenameSpeaker {
            speaker_id: "nope".to_owned(),
            name: "X".to_owned(),
        },
        Command::RemoveSpeaker {
            speaker_id: "nope".to_owned(),
        },
    ] {
        let error = editor.apply(command).expect_err("refused");
        assert!(
            !error.to_string().is_empty(),
            "every refusal is one user-facing sentence"
        );
    }
    // The video clip is picture, not a caption: it kept its lane.
    let timeline = editor.project().active();
    assert_eq!(
        timeline.tracks.len(),
        tracks_before,
        "no lane was made by a refusal"
    );
}

#[test]
fn an_old_document_reads_as_no_speakers_and_a_new_one_round_trips() {
    let (mut editor, _, first, _) = setup();
    // No speakers: the key stays off the wire, exactly as older builds
    // wrote the document.
    let json = serde_json::to_string(editor.project()).expect("serialises");
    assert!(!json.contains("speakers"), "{json}");
    let _: concat_project::model::Project = serde_json::from_str(&json).expect("loads back");

    // With a speaker and an assignment: the registry, the lane link and
    // the caption's label all survive the document round trip.
    let host = add_speaker(&mut editor, "Host", "#ff0000");
    editor
        .apply(Command::AssignCaptionToSpeaker {
            clip_id: first,
            speaker_id: host,
        })
        .expect("assigns");
    let json = serde_json::to_string(editor.project()).expect("serialises");
    assert!(json.contains("speakers"));
    let back: concat_project::model::Project = serde_json::from_str(&json).expect("loads back");
    assert_eq!(back, *editor.project(), "the document round-trips whole");
}
