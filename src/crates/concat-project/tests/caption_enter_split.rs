//! E4: Enter splits a caption at the caret.
//!
//! The owner's decisions (16 Oct 2026): the split time comes from the audio -
//! the heard start of the word the caret falls in - never from a text-length
//! estimate, and a caption with no heard word timings is refused, not
//! guessed. One split is one command, so one undo restores the whole. The
//! prescribed tests: Enter mid-caption gives two rows with the right text
//! and boundary; one undo restores; the karaoke highlight of both halves
//! lands on the spoken words. The sync matrix (§2) is asserted on both
//! halves of every successful split.

use concat_project::captionmap::{Word, locate_words, word_at_caret};
use concat_project::commands::{Command, CommandError, NewMedia};
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

/// The caption these tests split: "halo dunia apa" at 2.0-4.5 s, three
/// heard words in the back half of the cue. Returns (caption id, video id).
fn add_caption(editor: &mut Editor, words: Option<Vec<Word>>) -> (String, String) {
    let video_id = editor.project().active().clips[0].id.clone();
    let id = editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 2.0,
            style: Some(TextStyle {
                content: "halo dunia apa".to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(2.5),
            offset_y: Some(0.38),
            caption_of: Some(video_id.clone()),
            words,
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id");
    (id, video_id)
}

fn add_spoken_caption(editor: &mut Editor) -> (String, String) {
    add_caption(
        editor,
        Some(vec![
            word("halo", 3.5, 3.7),
            word("dunia", 3.7, 3.9),
            word("apa", 3.9, 4.1),
        ]),
    )
}

fn clip_of<'a>(editor: &'a Editor, id: &str) -> &'a Clip {
    editor.project().active().clip(id).expect("the clip")
}

fn words_of(clip: &Clip) -> Vec<(String, f64, f64)> {
    clip.words
        .as_ref()
        .expect("words present")
        .iter()
        .map(|w| (w.text.clone(), w.start, w.end))
        .collect()
}

/// The sync matrix (§2), asserted on both halves of every split.
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
    assert!(clip.is_caption(), "identity survived the split");
    for (i, located) in locate_words(&style.content, words).iter().enumerate() {
        assert!(located.is_some(), "word {i} located in the content");
    }
}

#[test]
fn enter_mid_caption_gives_two_rows_with_the_right_text_and_boundary() {
    let mut editor = setup();
    let (id, _) = add_spoken_caption(&mut editor);
    let track = clip_of(&editor, &id).track_id.clone();
    // The caret sits inside "dunia" ("halo dunia apa", offset 7).
    let word_index = word_at_caret(
        "halo dunia apa",
        clip_of(&editor, &id).words.as_ref().expect("words"),
        7,
    )
    .expect("the caret maps to a word");
    assert_eq!(word_index, 1);
    let outcome = editor
        .apply(Command::SplitCaptionAtWord {
            clip_id: id.clone(),
            word_index,
        })
        .expect("the split is accepted");
    assert!(outcome.applied);
    let tail_id = outcome.created_id.expect("the tail is minted");

    // The head keeps the id, starts where it did, and ends at the heard
    // start of "dunia" - the audio boundary, not a text estimate.
    let head = clip_of(&editor, &id);
    assert_eq!(head.text.as_ref().expect("style").content, "halo");
    assert!((head.start - 2.0).abs() < 1e-9);
    assert!(
        (head.duration - 1.7).abs() < 1e-9,
        "2.0 to the 3.7 boundary"
    );
    assert_eq!(words_of(head), vec![("halo".to_owned(), 3.5, 3.7)]);

    // The tail starts at the boundary with the rest of the words.
    let tail = clip_of(&editor, &tail_id);
    assert_eq!(tail.text.as_ref().expect("style").content, "dunia apa");
    assert!((tail.start - 3.7).abs() < 1e-9, "the heard word start");
    assert!(
        (tail.duration - 0.8).abs() < 1e-9,
        "boundary to the old end"
    );
    assert_eq!(tail.track_id, track, "both halves stay on the caption lane");
    assert_eq!(
        words_of(tail),
        vec![("dunia".to_owned(), 3.7, 3.9), ("apa".to_owned(), 3.9, 4.1)]
    );

    check_matrix(head);
    check_matrix(tail);
}

#[test]
fn one_undo_restores_the_whole_caption() {
    let mut editor = setup();
    let (id, _) = add_spoken_caption(&mut editor);
    let before = clip_of(&editor, &id).clone();
    let clips_before = editor.project().active().clips.len();
    editor
        .apply(Command::SplitCaptionAtWord {
            clip_id: id.clone(),
            word_index: 2,
        })
        .expect("the split is accepted");
    assert_eq!(editor.project().active().clips.len(), clips_before + 1);
    editor.undo();
    assert_eq!(editor.project().active().clips.len(), clips_before);
    let restored = clip_of(&editor, &id);
    assert_eq!(
        restored.text.as_ref().expect("style").content,
        before.text.as_ref().expect("style").content
    );
    assert!((restored.duration - before.duration).abs() < 1e-9);
    assert_eq!(words_of(restored), words_of(&before));
    check_matrix(restored);
}

#[test]
fn the_karaoke_of_both_halves_lands_on_the_spoken_words() {
    let mut editor = setup();
    let (id, _) = add_spoken_caption(&mut editor);
    let outcome = editor
        .apply(Command::SplitCaptionAtWord {
            clip_id: id.clone(),
            word_index: 1,
        })
        .expect("the split is accepted");
    let tail_id = outcome.created_id.expect("tail id");
    // The highlight reads word timings; both halves kept the heard ones
    // exactly - nothing re-estimated, nothing invented, nothing shifted.
    assert_eq!(
        words_of(clip_of(&editor, &id)),
        vec![("halo".to_owned(), 3.5, 3.7)]
    );
    assert_eq!(
        words_of(clip_of(&editor, &tail_id)),
        vec![("dunia".to_owned(), 3.7, 3.9), ("apa".to_owned(), 3.9, 4.1)]
    );
    // And every word is locatable in its own half, which is what the
    // highlighter asks before it can light a word up.
    for half in [clip_of(&editor, &id), clip_of(&editor, &tail_id)] {
        let style = half.text.as_ref().expect("style");
        for located in locate_words(&style.content, half.words.as_ref().expect("words")) {
            assert!(
                located.is_some(),
                "every word of both halves is highlightable"
            );
        }
    }
}

#[test]
fn a_caption_without_heard_timings_is_refused_not_guessed() {
    let mut editor = setup();
    // caption_of makes it a caption (T5 identity) but it carries no words:
    // an imported cue with no audio timing. The split must be refused.
    let (id, _) = add_caption(&mut editor, None);
    assert!(clip_of(&editor, &id).is_caption());
    let result = editor.apply(Command::SplitCaptionAtWord {
        clip_id: id.clone(),
        word_index: 1,
    });
    assert!(
        matches!(result, Err(CommandError::CannotSplitCaption { .. })),
        "refused with a message: {result:?}"
    );
    assert!(
        clip_of(&editor, &id).words.is_none(),
        "nothing was guessed in"
    );
}

#[test]
fn a_split_that_would_leave_an_empty_side_is_refused() {
    let mut editor = setup();
    let (id, _) = add_spoken_caption(&mut editor);
    let clips_before = editor.project().active().clips.len();
    for word_index in [0usize, 3] {
        let result = editor.apply(Command::SplitCaptionAtWord {
            clip_id: id.clone(),
            word_index,
        });
        assert!(
            matches!(result, Err(CommandError::CannotSplitCaption { .. })),
            "index {word_index} refused: {result:?}"
        );
    }
    assert_eq!(editor.project().active().clips.len(), clips_before);
    let clip = clip_of(&editor, &id);
    assert_eq!(clip.text.as_ref().expect("style").content, "halo dunia apa");
    assert!((clip.duration - 2.5).abs() < 1e-9, "untouched");
    assert_eq!(words_of(clip).len(), 3, "untouched");
}
