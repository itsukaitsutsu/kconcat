//! E3: word-edge drag - the sandbox-provable half.
//!
//! The plan's "done when": dragging a word edge moves its highlight and
//! timing, and the edit survives export. The highlight is a pure function
//! of the word timings (C3's mechanism), so a test that the timing moved
//! IS a test that the highlight moved; the waveform under the caption row
//! and the drag gesture itself are owner-machine UI (§9), and "survives
//! export" rides the document, tested here as a save/reload round trip -
//! the export binary does not compile in this sandbox, recorded.
//!
//! The drag's kindness rule: an edge is CLAMPED, never refused - it stops
//! at its neighbours (the heard order survives any drag) and at the
//! shortest a word may be. Only a clip that cannot hold the drag at all
//! (not a caption, no heard words, no such word) or a time that is not a
//! number is refused.

use concat_project::captionmap::{MIN_WORD, Word};
use concat_project::commands::{Command, CommandError, NewMedia, WordEdge};
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

fn video_id(editor: &Editor) -> String {
    editor.project().active().clips[0].id.clone()
}

/// The caption the drags run on: window 2.0..4.5, three heard words
/// 3.5..4.1, each a fifth of a second long.
fn add_caption(editor: &mut Editor, words: Option<Vec<Word>>) -> String {
    editor
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
            caption_of: Some(video_id(editor)),
            words,
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id")
}

fn three_words() -> Vec<Word> {
    vec![
        word("halo", 3.5, 3.7),
        word("dunia", 3.7, 3.9),
        word("apa", 3.9, 4.1),
    ]
}

fn drag(editor: &mut Editor, clip_id: &str, word_index: usize, edge: WordEdge, time: f64) {
    editor
        .apply(Command::MoveCaptionWordEdge {
            clip_id: clip_id.to_owned(),
            word_index,
            edge,
            time,
        })
        .expect("the drag is accepted");
}

fn words_of<'a>(editor: &'a Editor, id: &str) -> &'a [Word] {
    editor
        .project()
        .active()
        .clip(id)
        .expect("the clip")
        .words
        .as_deref()
        .expect("heard words")
}

/// The prescribed core: dragging a word's edge moves its timing - and
/// only that edge. The text, the window and the other words stand.
#[test]
fn dragging_a_word_end_moves_only_that_edge() {
    let mut editor = setup();
    let id = add_caption(&mut editor, Some(three_words()));

    drag(&mut editor, &id, 0, WordEdge::End, 3.65);

    let words = words_of(&editor, &id);
    assert!((words[0].end - 3.65).abs() < 1e-9, "the dragged edge moved");
    assert!((words[0].start - 3.5).abs() < 1e-9, "its own start stands");
    assert!((words[1].start - 3.7).abs() < 1e-9);
    assert!((words[1].end - 3.9).abs() < 1e-9);
    assert!((words[2].start - 3.9).abs() < 1e-9);
    assert!((words[2].end - 4.1).abs() < 1e-9);

    let clip: &Clip = editor.project().active().clip(&id).expect("clip");
    assert!((clip.start - 2.0).abs() < 1e-9, "the window stands");
    assert!((clip.duration - 2.5).abs() < 1e-9);
    assert_eq!(
        clip.text.as_ref().expect("style").content,
        "halo dunia apa",
        "a timing-only edit never touches the text (T7's guard)"
    );
}

/// The kindness rule, first half: an edge stops at its neighbour, so no
/// drag can overlap the heard order - monotonicity survives anything the
/// mouse does.
#[test]
fn an_edge_stops_at_its_neighbour() {
    let mut editor = setup();
    let id = add_caption(&mut editor, Some(three_words()));

    // A wild drag past the next word lands ON the next word's start.
    drag(&mut editor, &id, 0, WordEdge::End, 9.0);
    let words = words_of(&editor, &id);
    assert!(
        (words[0].end - 3.7).abs() < 1e-9,
        "clamped to the neighbour, got {}",
        words[0].end
    );

    // And the mirror: a start dragged before the previous word's end
    // lands on it.
    drag(&mut editor, &id, 1, WordEdge::Start, 0.0);
    let words = words_of(&editor, &id);
    assert!((words[1].start - 3.7).abs() < 1e-9);

    // Whatever happened, the heard order is intact.
    let words = words_of(&editor, &id);
    assert!(words[0].start <= words[0].end);
    assert!(words[0].end <= words[1].start);
    assert!(words[1].start <= words[1].end);
    assert!(words[1].end <= words[2].start);
}

/// The kindness rule, second half: an edge stops at the shortest a word
/// may be, and the first and last word stop at the caption's window - a
/// word is never heard outside its own cue.
#[test]
fn an_edge_stops_at_the_minimum_word_and_the_window() {
    let mut editor = setup();
    let id = add_caption(&mut editor, Some(three_words()));

    // End collapsed onto its own start: the floor is MIN_WORD wide.
    drag(&mut editor, &id, 1, WordEdge::End, 3.0);
    let words = words_of(&editor, &id);
    assert!(
        (words[1].end - (3.7 + MIN_WORD)).abs() < 1e-9,
        "the word keeps its minimum, got {}",
        words[1].end
    );

    // Start pushed past its own end: same floor from the other side.
    drag(&mut editor, &id, 1, WordEdge::Start, 4.4);
    let words = words_of(&editor, &id);
    assert!((words[1].start - 3.7).abs() < 1e-9);
    assert!((words[1].end - (3.7 + MIN_WORD)).abs() < 1e-9);

    // The first word cannot be heard before its cue appears.
    drag(&mut editor, &id, 0, WordEdge::Start, 0.0);
    assert!((words_of(&editor, &id)[0].start - 2.0).abs() < 1e-9);

    // The last word cannot be heard after its cue goes away.
    drag(&mut editor, &id, 2, WordEdge::End, 99.0);
    assert!((words_of(&editor, &id)[2].end - 4.5).abs() < 1e-9);
}

/// One drag, one undo - the editor's snapshot rule, same as every other
/// command.
#[test]
fn one_undo_restores_the_drag() {
    let mut editor = setup();
    let id = add_caption(&mut editor, Some(three_words()));

    drag(&mut editor, &id, 0, WordEdge::End, 3.65);
    assert!((words_of(&editor, &id)[0].end - 3.65).abs() < 1e-9);

    assert!(editor.undo(), "the drag is one undo entry");
    let words = words_of(&editor, &id);
    assert!((words[0].end - 3.7).abs() < 1e-9, "the edge is back");
    assert!((words[1].start - 3.7).abs() < 1e-9);
}

/// "The edit survives export" rides the document: the dragged edge is
/// stored word timing, and stored timing is what save writes and load
/// reads. The export binary itself is owner-machine (does not compile
/// here) - recorded, not claimed.
#[test]
fn the_drag_survives_a_reload() {
    let mut editor = setup();
    let id = add_caption(&mut editor, Some(three_words()));

    drag(&mut editor, &id, 2, WordEdge::Start, 3.95);

    let json = serde_json::to_string(editor.project()).expect("serialises");
    let back: concat_project::model::Project = serde_json::from_str(&json).expect("loads back");
    let caption = back
        .active()
        .clips
        .iter()
        .find(|clip| clip.text.is_some())
        .expect("the caption");
    let words = caption.words.as_ref().expect("heard words");
    assert!(
        (words[2].start - 3.95).abs() < 1e-9,
        "the dragged edge survived the reload"
    );
    assert!((words[2].end - 4.1).abs() < 1e-9);
}

/// The refusals, one sentence each: a clip that cannot hold the drag at
/// all. A drag ON a holdable edge is never refused - it is clamped.
#[test]
fn a_clip_that_cannot_hold_the_drag_refuses_with_one_sentence() {
    let mut editor = setup();

    // Picture has no heard words.
    let video = video_id(&editor);
    let err = editor
        .apply(Command::MoveCaptionWordEdge {
            clip_id: video,
            word_index: 0,
            edge: WordEdge::End,
            time: 1.0,
        })
        .expect_err("refused");
    assert!(matches!(err, CommandError::CannotMoveWordEdge { .. }));

    // A caption with no heard timings (an imported cue) has no edge to
    // drag - the estimate a word layout would use is not stored timing.
    let bare = add_caption(&mut editor, None);
    let err = editor
        .apply(Command::MoveCaptionWordEdge {
            clip_id: bare,
            word_index: 0,
            edge: WordEdge::End,
            time: 3.0,
        })
        .expect_err("refused");
    assert!(matches!(err, CommandError::CannotMoveWordEdge { .. }));

    // An index past the end names no word.
    let id = add_caption(&mut editor, Some(three_words()));
    let err = editor
        .apply(Command::MoveCaptionWordEdge {
            clip_id: id.clone(),
            word_index: 7,
            edge: WordEdge::Start,
            time: 3.6,
        })
        .expect_err("refused");
    assert!(matches!(err, CommandError::CannotMoveWordEdge { .. }));

    // A time that is not a number is the editor's universal refusal,
    // caught before dispatch.
    let err = editor
        .apply(Command::MoveCaptionWordEdge {
            clip_id: id,
            word_index: 0,
            edge: WordEdge::Start,
            time: f64::NAN,
        })
        .expect_err("refused");
    assert!(matches!(err, CommandError::NotANumber));
}
