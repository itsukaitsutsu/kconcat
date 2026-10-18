//! E1: transcript editing - delete a word, the video cuts.
//!
//! The owner's decisions (16 Oct 2026): the gap after a delete is closed
//! (ripple); the A2 checkbox "Potong video saat kata dihapus" chooses
//! between cutting picture and sound with the word or removing only the
//! text; changing a word's text stays T7's job and is not repeated here.
//! Prescribed tests: deleting one word ripples audio, picture and captions
//! together; deleting a sentence keeps the sync matrix; one undo restores
//! everything.

use concat_project::captionmap::{Word, locate_words};
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

fn video_id(editor: &Editor) -> String {
    editor.project().active().clips[0].id.clone()
}

/// A caption following the video clip, with heard words.
fn add_caption(
    editor: &mut Editor,
    content: &str,
    start: f64,
    duration: f64,
    words: Option<Vec<Word>>,
) -> String {
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
            caption_of: Some(video_id(editor)),
            words,
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id")
}

fn add_halo_dunia_apa(editor: &mut Editor) -> String {
    add_caption(
        editor,
        "halo dunia apa",
        2.0,
        2.5,
        Some(vec![
            word("halo", 3.5, 3.7),
            word("dunia", 3.7, 3.9),
            word("apa", 3.9, 4.1),
        ]),
    )
}

fn delete_word(editor: &mut Editor, clip_id: &str, word_index: usize, cut_video: bool) {
    editor
        .apply(Command::DeleteCaptionWord {
            clip_id: clip_id.to_owned(),
            word_index,
            cut_video,
        })
        .expect("the delete is accepted");
}

fn clip_of<'a>(editor: &'a Editor, id: &str) -> &'a Clip {
    editor.project().active().clip(id).expect("the clip")
}

fn video_pieces(editor: &Editor) -> Vec<(String, f64, f64)> {
    let track = editor.project().active().clips[0].track_id.clone();
    editor
        .project()
        .active()
        .clips
        .iter()
        .filter(|clip| clip.track_id == track && !clip.is_caption())
        .map(|clip| (clip.id.clone(), clip.start, clip.duration))
        .collect()
}

fn caption_pieces(editor: &Editor) -> Vec<(String, f64, f64)> {
    editor
        .project()
        .active()
        .clips
        .iter()
        .filter(|clip| clip.is_caption())
        .map(|clip| (clip.id.clone(), clip.start, clip.duration))
        .collect()
}

fn words_of(clip: &Clip) -> Vec<(String, f64, f64)> {
    clip.words
        .as_ref()
        .expect("words present")
        .iter()
        .map(|w| (w.text.clone(), w.start, w.end))
        .collect()
}

/// Word timings arrive through shift arithmetic, so they are compared with
/// a tolerance; the texts exactly.
fn assert_words_close(actual: &[(String, f64, f64)], expected: &[(&str, f64, f64)], msg: &str) {
    assert_eq!(actual.len(), expected.len(), "{msg}: word count");
    for (a, e) in actual.iter().zip(expected) {
        assert_eq!(a.0, e.0, "{msg}: word text");
        assert!((a.1 - e.1).abs() < 1e-6, "{msg}: start {} vs {}", a.1, e.1);
        assert!((a.2 - e.2).abs() < 1e-6, "{msg}: end {} vs {}", a.2, e.2);
    }
}

/// The sync matrix (§2) for a caption piece after a delete.
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
    assert!(clip.is_caption(), "identity survived the delete");
    for (i, located) in locate_words(&style.content, words).iter().enumerate() {
        assert!(located.is_some(), "word {i} located in the content");
    }
}

#[test]
fn deleting_one_word_ripples_audio_picture_and_captions_together() {
    let mut editor = setup();
    let cap = add_halo_dunia_apa(&mut editor);
    let video = video_id(&editor);
    // "dunia" is heard at 3.7-3.9: cutting it takes 0.2 s out of every lane.
    delete_word(&mut editor, &cap, 1, true);

    // The picture (and its sound - one AV clip) is two packed pieces now:
    // 0-3.7 and 3.7-9.8, ten seconds minus the word.
    let pieces = video_pieces(&editor);
    assert_eq!(pieces.len(), 2, "the video lane is cut and packed");
    assert!((pieces[0].1 - 0.0).abs() < 1e-6 && (pieces[0].2 - 3.7).abs() < 1e-6);
    assert!((pieces[1].1 - 3.7).abs() < 1e-6 && (pieces[1].2 - 6.1).abs() < 1e-6);
    assert_eq!(pieces[0].0, video, "the head keeps the original id");

    // The caption rode the same cut: "halo" before the boundary, "apa"
    // after it, packed against each other, each following its own piece of
    // the source.
    let caps = caption_pieces(&editor);
    assert_eq!(caps.len(), 2, "one caption became two packed pieces");
    let halo = clip_of(&editor, &caps[0].0);
    let apa = clip_of(&editor, &caps[1].0);
    assert_eq!(halo.text.as_ref().expect("style").content, "halo");
    assert!((halo.start - 2.0).abs() < 1e-6 && (halo.duration - 1.7).abs() < 1e-6);
    assert_words_close(&words_of(halo), &[("halo", 3.5, 3.7)], "halo piece");
    assert_eq!(apa.text.as_ref().expect("style").content, "apa");
    assert!((apa.start - 3.7).abs() < 1e-6 && (apa.duration - 0.6).abs() < 1e-6);
    assert_words_close(
        &words_of(apa),
        &[("apa", 3.7, 3.9)],
        "the surviving word moved left with the cut",
    );
    assert_eq!(halo.caption_of.as_deref(), Some(video.as_str()));
    assert_eq!(
        apa.caption_of.as_deref(),
        Some(pieces[1].0.as_str()),
        "the tail caption follows the tail piece of the source"
    );
    check_matrix(halo);
    check_matrix(apa);
}

#[test]
fn deleting_a_sentence_keeps_the_sync_matrix() {
    let mut editor = setup();
    // Two cues over the video: "halo dunia" 2.0-4.0 and "apa kabar" 4.0-6.0.
    let first = add_caption(
        &mut editor,
        "halo dunia",
        2.0,
        2.0,
        Some(vec![word("halo", 3.5, 3.7), word("dunia", 3.7, 3.9)]),
    );
    let second = add_caption(
        &mut editor,
        "apa kabar",
        4.0,
        2.0,
        Some(vec![word("apa", 4.5, 4.7), word("kabar", 4.7, 4.9)]),
    );
    // The sentence goes word by word, left to right (the order the delete
    // path is driven in).
    delete_word(&mut editor, &first, 0, true);
    // What is left of the first cue keeps its id and shows only "dunia".
    assert_eq!(
        clip_of(&editor, &first)
            .text
            .as_ref()
            .expect("style")
            .content,
        "dunia"
    );
    delete_word(&mut editor, &first, 0, true);

    // The first cue is gone entirely - no wordless husk is left behind.
    assert!(
        editor.project().active().clip(&first).is_none(),
        "the emptied caption went with its last word"
    );
    // The video lost both words: 10 - 0.4 = 9.6, packed with no gap.
    let pieces = video_pieces(&editor);
    assert_eq!(pieces.len(), 2);
    assert!((pieces[0].1 - 0.0).abs() < 1e-6 && (pieces[0].2 - 3.5).abs() < 1e-6);
    assert!((pieces[1].1 - 3.5).abs() < 1e-6 && (pieces[1].2 - 6.1).abs() < 1e-6);
    // The second cue kept its sync: shifted left by exactly the cut total,
    // text and words together.
    let survivor = clip_of(&editor, &second);
    assert_eq!(survivor.text.as_ref().expect("style").content, "apa kabar");
    assert!((survivor.start - 3.6).abs() < 1e-6 && (survivor.duration - 2.0).abs() < 1e-6);
    assert_words_close(
        &words_of(survivor),
        &[("apa", 4.1, 4.3), ("kabar", 4.3, 4.5)],
        "the survivor kept its sync",
    );
    check_matrix(survivor);
    assert_eq!(
        caption_pieces(&editor).len(),
        1,
        "only the survivor is left"
    );
}

#[test]
fn one_undo_restores_everything() {
    let mut editor = setup();
    let cap = add_halo_dunia_apa(&mut editor);
    delete_word(&mut editor, &cap, 1, true);
    assert_eq!(video_pieces(&editor).len(), 2);
    assert_eq!(caption_pieces(&editor).len(), 2);
    // Undo is whole-state: one step puts every lane back.
    editor.undo();
    let pieces = video_pieces(&editor);
    assert_eq!(pieces.len(), 1, "the video is whole again");
    assert!((pieces[0].1 - 0.0).abs() < 1e-6 && (pieces[0].2 - 10.0).abs() < 1e-6);
    let caps = caption_pieces(&editor);
    assert_eq!(caps.len(), 1, "the caption is one cue again");
    let whole = clip_of(&editor, &cap);
    assert_eq!(
        whole.text.as_ref().expect("style").content,
        "halo dunia apa"
    );
    assert!((whole.start - 2.0).abs() < 1e-6 && (whole.duration - 2.5).abs() < 1e-6);
    assert_words_close(
        &words_of(whole),
        &[("halo", 3.5, 3.7), ("dunia", 3.7, 3.9), ("apa", 3.9, 4.1)],
        "the restored cue",
    );
    check_matrix(whole);
}

#[test]
fn without_the_checkbox_only_the_text_goes() {
    let mut editor = setup();
    let cap = add_halo_dunia_apa(&mut editor);
    delete_word(&mut editor, &cap, 1, false);
    // The picture and its sound stand still: one clip, still ten seconds.
    let pieces = video_pieces(&editor);
    assert_eq!(pieces.len(), 1);
    assert!((pieces[0].2 - 10.0).abs() < 1e-6, "the video is untouched");
    // The caption keeps its span and every surviving word keeps its timing;
    // only the deleted word's text and slot are gone (T7's prune path).
    let clip = clip_of(&editor, &cap);
    assert_eq!(clip.text.as_ref().expect("style").content, "halo apa");
    assert!((clip.start - 2.0).abs() < 1e-6 && (clip.duration - 2.5).abs() < 1e-6);
    assert_words_close(
        &words_of(clip),
        &[("halo", 3.5, 3.7), ("apa", 3.9, 4.1)],
        "the survivors keep their heard times",
    );
    check_matrix(clip);
    editor.undo();
    let clip = clip_of(&editor, &cap);
    assert_eq!(clip.text.as_ref().expect("style").content, "halo dunia apa");
    assert_eq!(
        words_of(clip).len(),
        3,
        "one undo restores the text path too"
    );
}

#[test]
fn deletes_outside_a_heard_caption_are_refused() {
    let mut editor = setup();
    let cap = add_halo_dunia_apa(&mut editor);
    // A hand-made title: no words, not a caption.
    let title = editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 7.0,
            style: Some(TextStyle {
                content: "Judul".to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(1.0),
            offset_y: Some(0.1),
            caption_of: None,
            words: None,
        })
        .expect("adds the title")
        .created_id
        .expect("title id");
    // An imported cue: a caption by identity, but with no heard timings.
    let imported = editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 8.5,
            style: Some(TextStyle {
                content: "tak ada waktu".to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(1.0),
            offset_y: Some(0.38),
            caption_of: Some(video_id(&editor)),
            words: None,
        })
        .expect("adds the imported cue")
        .created_id
        .expect("imported id");

    let clips_before = editor.project().active().clips.len();
    for (id, index) in [
        (&title, 0usize),         // not a caption at all
        (&imported, 0),           // a caption with no heard timings
        (&cap, 3),                // past the last word
        (&"ghost".to_owned(), 0), // a clip that does not exist
    ] {
        let result = editor.apply(Command::DeleteCaptionWord {
            clip_id: id.clone(),
            word_index: index,
            cut_video: true,
        });
        assert!(
            matches!(result, Err(CommandError::CannotDeleteWord { .. })),
            "refused with a message: {result:?}"
        );
    }
    assert_eq!(editor.project().active().clips.len(), clips_before);
    assert_eq!(
        clip_of(&editor, &cap).text.as_ref().expect("style").content,
        "halo dunia apa",
        "nothing moved"
    );
    assert!((clip_of(&editor, video_id(&editor).as_str()).duration - 10.0).abs() < 1e-9);
}
