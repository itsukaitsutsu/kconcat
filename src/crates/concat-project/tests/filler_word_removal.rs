//! E2: filler-word removal (Indonesian).
//!
//! Owner decision A7 (16 Oct 2026): two opt-in tiers - strong (eee, anu,
//! ehm) and weak (kayak, gitu, terus, apa ya) - and nothing is removed
//! unless its tier is ticked, because "terus" can be meaningful. The video
//! effect follows the A2 checkbox, through E1's cut path. Done-when: a
//! one-click pass removes the ticked groups inside a selected caption run,
//! leaves meaningful words alone, with at least one negative case.

use concat_project::captionmap::{Word, locate_words};
use concat_project::commands::{Command, NewMedia};
use concat_project::editor::Editor;
use concat_project::model::{Clip, MediaKind, TextStyle};

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

fn add_caption(
    editor: &mut Editor,
    content: &str,
    start: f64,
    duration: f64,
    words: Vec<Word>,
) -> String {
    let video = editor.project().active().clips[0].id.clone();
    editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start,
            style: Some(TextStyle {
                content: content.to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(duration),
            offset_y: Some(0.38),
            caption_of: Some(video),
            words: Some(words),
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id")
}

/// The run these tests use: "eee kita pergi" 2.0-4.5, then
/// "anu terus selesai" 4.5-6.5, over a 10 s video.
fn add_run(editor: &mut Editor) -> (String, String) {
    let first = add_caption(
        editor,
        "eee kita pergi",
        2.0,
        2.5,
        vec![
            word("eee", 3.0, 3.3),
            word("kita", 3.4, 3.6),
            word("pergi", 3.7, 4.0),
        ],
    );
    let second = add_caption(
        editor,
        "anu terus selesai",
        4.5,
        2.0,
        vec![
            word("anu", 5.0, 5.2),
            word("terus", 5.3, 5.6),
            word("selesai", 5.7, 6.0),
        ],
    );
    (first, second)
}

fn remove_fillers(
    editor: &mut Editor,
    clip_ids: Vec<String>,
    strong: bool,
    weak: bool,
    cut_video: bool,
) -> bool {
    editor
        .apply(Command::RemoveFillerWords {
            clip_ids,
            strong,
            weak,
            cut_video,
        })
        .expect("the filler pass is accepted")
        .applied
}

fn clip_of<'a>(editor: &'a Editor, id: &str) -> &'a Clip {
    editor.project().active().clip(id).expect("the clip")
}

fn content_of(editor: &Editor, id: &str) -> String {
    clip_of(editor, id)
        .text
        .as_ref()
        .expect("style")
        .content
        .clone()
}

fn words_of(clip: &Clip) -> Vec<(String, f64, f64)> {
    clip.words
        .as_ref()
        .expect("words present")
        .iter()
        .map(|w| (w.text.clone(), w.start, w.end))
        .collect()
}

fn assert_words_close(actual: &[(String, f64, f64)], expected: &[(&str, f64, f64)], msg: &str) {
    assert_eq!(actual.len(), expected.len(), "{msg}: word count");
    for (a, e) in actual.iter().zip(expected) {
        assert_eq!(a.0, e.0, "{msg}: word text");
        assert!((a.1 - e.1).abs() < 1e-6, "{msg}: start {} vs {}", a.1, e.1);
        assert!((a.2 - e.2).abs() < 1e-6, "{msg}: end {} vs {}", a.2, e.2);
    }
}

fn check_matrix(clip: &Clip) {
    let style = clip.text.as_ref().expect("style present");
    let words = clip.words.as_ref().expect("words present");
    let joined = words
        .iter()
        .map(|w| w.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(joined, style.content, "content equals the words joined");
    let mut prev_end = clip.start;
    for w in words {
        assert!(w.start >= prev_end - 1e-9, "ordered, non-overlapping");
        assert!(w.end >= w.start, "word span sane");
        assert!(w.start >= clip.start - 1e-9, "inside the cue");
        assert!(w.end <= clip.start + clip.duration + 1e-9, "inside the cue");
        prev_end = w.end;
    }
    assert!(clip.is_caption(), "identity survived the pass");
    for (i, located) in locate_words(&style.content, words).iter().enumerate() {
        assert!(located.is_some(), "word {i} located in the content");
    }
}

fn video_pieces(editor: &Editor) -> Vec<(f64, f64)> {
    editor
        .project()
        .active()
        .clips
        .iter()
        .filter(|clip| !clip.is_caption())
        .map(|clip| (clip.start, clip.duration))
        .collect()
}

#[test]
fn one_pass_removes_the_ticked_groups_and_leaves_the_rest_alone() {
    let mut editor = setup();
    let (first, second) = add_run(&mut editor);
    // Strong only: "eee" and "anu" go; "terus" is weak-tier and unticked,
    // so it stays even though it often is filler - the negative case.
    assert!(remove_fillers(
        &mut editor,
        vec![first.clone(), second.clone()],
        true,
        false,
        false
    ));
    assert_eq!(content_of(&editor, &first), "kita pergi");
    assert_words_close(
        &words_of(clip_of(&editor, &first)),
        &[("kita", 3.4, 3.6), ("pergi", 3.7, 4.0)],
        "text-only keeps every surviving timing",
    );
    assert_eq!(content_of(&editor, &second), "terus selesai");
    assert_words_close(
        &words_of(clip_of(&editor, &second)),
        &[("terus", 5.3, 5.6), ("selesai", 5.7, 6.0)],
        "the unticked tier stands",
    );
    check_matrix(clip_of(&editor, &first));
    check_matrix(clip_of(&editor, &second));
    // Text only: picture and sound never moved.
    let pieces = video_pieces(&editor);
    assert_eq!(pieces.len(), 1);
    assert!((pieces[0].1 - 10.0).abs() < 1e-9, "the video is untouched");
    // One undo restores the whole pass.
    editor.undo();
    assert_eq!(content_of(&editor, &first), "eee kita pergi");
    assert_eq!(content_of(&editor, &second), "anu terus selesai");
    assert_eq!(words_of(clip_of(&editor, &first)).len(), 3);
    assert_eq!(words_of(clip_of(&editor, &second)).len(), 3);
}

#[test]
fn ticking_both_tiers_takes_the_weak_words_too() {
    let mut editor = setup();
    let (first, second) = add_run(&mut editor);
    assert!(remove_fillers(
        &mut editor,
        vec![first.clone(), second.clone()],
        true,
        true,
        false
    ));
    assert_eq!(content_of(&editor, &first), "kita pergi");
    assert_eq!(content_of(&editor, &second), "selesai");
    assert_words_close(
        &words_of(clip_of(&editor, &second)),
        &[("selesai", 5.7, 6.0)],
        "the survivor keeps its heard time",
    );
    check_matrix(clip_of(&editor, &second));
}

#[test]
fn with_the_checkbox_the_heard_spans_come_out_of_every_lane() {
    let mut editor = setup();
    let (first, _second) = add_run(&mut editor);
    // Strong tier over the first cue: "eee" is heard at 3.0-3.3, so 0.3 s
    // comes out of the video and every lane packs closed.
    assert!(remove_fillers(
        &mut editor,
        vec![first.clone()],
        true,
        false,
        true
    ));
    let pieces = video_pieces(&editor);
    assert_eq!(pieces.len(), 2, "the video is cut and packed");
    assert!((pieces[0].0 - 0.0).abs() < 1e-6 && (pieces[0].1 - 3.0).abs() < 1e-6);
    assert!((pieces[1].0 - 3.0).abs() < 1e-6 && (pieces[1].1 - 6.7).abs() < 1e-6);
    // The caption rode the cut as one rebuilt cue: filler gone, the rest
    // shifted left by exactly the cut, text and words together.
    let rebuilt = clip_of(&editor, &first);
    assert_eq!(rebuilt.text.as_ref().expect("style").content, "kita pergi");
    assert!((rebuilt.start - 3.0).abs() < 1e-6 && (rebuilt.duration - 1.2).abs() < 1e-6);
    assert_words_close(
        &words_of(rebuilt),
        &[("kita", 3.1, 3.3), ("pergi", 3.4, 3.7)],
        "the survivors moved with the cut",
    );
    check_matrix(rebuilt);
    // One undo puts every lane back.
    editor.undo();
    let pieces = video_pieces(&editor);
    assert_eq!(pieces.len(), 1);
    assert!((pieces[0].1 - 10.0).abs() < 1e-9);
    assert_eq!(content_of(&editor, &first), "eee kita pergi");
    assert_eq!(words_of(clip_of(&editor, &first)).len(), 3);
}

#[test]
fn a_caption_of_nothing_but_filler_goes_with_it() {
    let mut editor = setup();
    let only = add_caption(&mut editor, "eee", 2.0, 1.0, vec![word("eee", 2.2, 2.6)]);
    assert!(remove_fillers(
        &mut editor,
        vec![only.clone()],
        true,
        false,
        true
    ));
    // No wordless, textless husk is left on the caption lane.
    assert!(
        editor.project().active().clip(&only).is_none(),
        "the emptied caption went with its only word"
    );
    assert_eq!(
        editor
            .project()
            .active()
            .clips
            .iter()
            .filter(|clip| clip.is_caption())
            .count(),
        0,
        "and left no rebuilt slivers behind"
    );
    let pieces = video_pieces(&editor);
    assert!((pieces.iter().map(|(_, d)| d).sum::<f64>() - 9.6).abs() < 1e-6);
}

#[test]
fn a_lookalike_is_not_a_filler_and_an_unticked_run_changes_nothing() {
    let mut editor = setup();
    // "begitu" contains "gitu" but is a word of its own; nothing is ticked
    // strong enough to touch it, and a run with no filler reports no edit.
    let lookalike = add_caption(
        &mut editor,
        "begitu indah",
        2.0,
        2.0,
        vec![word("begitu", 3.0, 3.4), word("indah", 3.5, 3.9)],
    );
    let clips_before = editor.project().active().clips.len();
    let applied = remove_fillers(&mut editor, vec![lookalike.clone()], true, true, true);
    assert!(!applied, "no filler found is not an edit");
    assert_eq!(content_of(&editor, &lookalike), "begitu indah");
    assert_eq!(words_of(clip_of(&editor, &lookalike)).len(), 2);
    assert_eq!(editor.project().active().clips.len(), clips_before);
    // A selection may hold picture: non-captions are skipped, not errors.
    let video = editor.project().active().clips[0].id.clone();
    let applied = remove_fillers(&mut editor, vec![video], true, true, true);
    assert!(!applied, "a picture clip holds no heard caption words");
}
