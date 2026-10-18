//! T7: one writer for text and words.
//!
//! The suspected bug: an inline text edit (Sections' `edit_row_text`) issues
//! `Command::UpdateClip` with a new `style.content`; the handler wrote the
//! text and left `clip.words` stale, and `locate_words` then drops the words
//! that no longer appear - the karaoke highlight loses them.
//!
//! The rule under test: `UpdateClip` is the single writer of a text patch, so
//! it keeps text and words consistent - unchanged content does nothing; the
//! same token count re-texts the words in position and keeps their timings (a
//! typo fix must not lose karaoke); any other count prunes to the words that
//! still appear, in order, never inventing one. Every case must also hold the
//! roadmap's sync matrix (§2) for the inline-edit path.

use concat_project::captionmap::{Word, locate_words};
use concat_project::commands::{ClipPatch, Command, NewMedia};
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

/// One caption at 2.0 s (cue 2.0–4.0) carrying `content` and `words`.
fn add_caption(editor: &mut Editor, content: &str, words: Vec<Word>) -> String {
    editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 2.0,
            style: Some(TextStyle {
                content: content.to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(2.0),
            offset_y: Some(0.38),
            caption_of: None,
            words: Some(words),
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id")
}

/// The default caption of these tests: "halo dunia", words in the back half.
fn add_halo_dunia(editor: &mut Editor) -> String {
    add_caption(
        editor,
        "halo dunia",
        vec![word("halo", 3.5, 3.7), word("dunia", 3.7, 3.9)],
    )
}

/// The inline edit: one `UpdateClip` carrying a text style with new content.
fn retext(editor: &mut Editor, clip_id: &str, content: &str) -> bool {
    let style = TextStyle {
        content: content.to_owned(),
        ..TextStyle::default()
    };
    editor
        .apply(Command::UpdateClip {
            clip_id: clip_id.to_owned(),
            patch: ClipPatch {
                text: Some(Some(style)),
                ..Default::default()
            },
        })
        .expect("updates the clip")
        .applied
}

fn clip_of<'a>(editor: &'a Editor, id: &str) -> &'a Clip {
    editor.project().active().clip(id).expect("the caption")
}

fn words_of(clip: &Clip) -> Vec<(String, f64, f64)> {
    clip.words
        .as_ref()
        .expect("words present")
        .iter()
        .map(|w| (w.text.clone(), w.start, w.end))
        .collect()
}

/// The sync matrix (§2), asserted after every edit these tests make.
fn check_matrix(clip: &Clip, was_caption: bool) {
    let style = clip.text.as_ref().expect("style present");
    let words = clip.words.as_ref().expect("words present");
    // 1. content equals the words joined - never stale.
    let joined = words
        .iter()
        .map(|w| w.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(joined, style.content, "content equals the words joined");
    // 2. ordered, non-overlapping, inside the cue's span.
    let mut prev_end = clip.start;
    for w in words {
        assert!(w.start >= prev_end - 1e-9, "ordered and non-overlapping");
        assert!(w.end >= w.start, "word span sane");
        assert!(w.start >= clip.start - 1e-9, "inside the cue");
        assert!(w.end <= clip.start + clip.duration + 1e-9, "inside the cue");
        prev_end = w.end;
    }
    // 3. identity is the same before and after the edit.
    assert_eq!(clip.is_caption(), was_caption, "caption identity unchanged");
    // 4. locate_words finds every word in the content.
    for (i, located) in locate_words(&style.content, words).iter().enumerate() {
        assert!(located.is_some(), "word {i} located in the content");
    }
    // 5. span agreement holds by 1-2 plus the cue's own unchanged span.
}

#[test]
fn a_typo_fix_retexts_in_position_and_keeps_every_timing() {
    let mut editor = setup();
    let id = add_halo_dunia(&mut editor);
    assert!(
        retext(&mut editor, &id, "halo dunja"),
        "the content changed"
    );
    let clip = clip_of(&editor, &id);
    assert_eq!(
        words_of(clip),
        vec![
            ("halo".to_owned(), 3.5, 3.7),
            ("dunja".to_owned(), 3.7, 3.9)
        ],
        "same token count re-texts in position; the timings survive the typo fix"
    );
    check_matrix(clip, true);
}

#[test]
fn a_deleted_word_prunes_without_inventing() {
    let mut editor = setup();
    let id = add_halo_dunia(&mut editor);
    assert!(retext(&mut editor, &id, "halo"), "the content changed");
    let clip = clip_of(&editor, &id);
    assert_eq!(
        words_of(clip),
        vec![("halo".to_owned(), 3.5, 3.7)],
        "the survivor keeps its timing; nothing is invented for the gap"
    );
    check_matrix(clip, true);
}

#[test]
fn unchanged_content_does_nothing() {
    let mut editor = setup();
    let id = add_halo_dunia(&mut editor);
    let style = clip_of(&editor, &id).text.clone().expect("style");
    let before = words_of(clip_of(&editor, &id));
    let applied = editor
        .apply(Command::UpdateClip {
            clip_id: id.clone(),
            patch: ClipPatch {
                text: Some(Some(style)),
                ..Default::default()
            },
        })
        .expect("updates")
        .applied;
    assert!(!applied, "an identical patch changes nothing");
    assert_eq!(words_of(clip_of(&editor, &id)), before);
    check_matrix(clip_of(&editor, &id), true);
}

#[test]
fn a_reorder_of_the_same_count_keeps_timings_by_position() {
    let mut editor = setup();
    let id = add_halo_dunia(&mut editor);
    assert!(retext(&mut editor, &id, "dunia halo"));
    let clip = clip_of(&editor, &id);
    assert_eq!(
        words_of(clip),
        vec![
            ("dunia".to_owned(), 3.5, 3.7),
            ("halo".to_owned(), 3.7, 3.9)
        ],
        "the same count re-texts in position: the timing stays with the slot"
    );
    check_matrix(clip, true);
}

#[test]
fn a_repeated_word_survives_as_often_as_it_still_occurs() {
    let mut editor = setup();
    let id = add_caption(
        &mut editor,
        "halo dunia halo",
        vec![
            word("halo", 3.5, 3.6),
            word("dunia", 3.6, 3.7),
            word("halo", 3.7, 3.8),
        ],
    );
    assert!(retext(&mut editor, &id, "halo halo"));
    let clip = clip_of(&editor, &id);
    assert_eq!(
        words_of(clip),
        vec![("halo".to_owned(), 3.5, 3.6), ("halo".to_owned(), 3.7, 3.8)],
        "pruning counts occurrences: both surviving halos keep their timings"
    );
    check_matrix(clip, true);
}

#[test]
fn clearing_the_style_clears_the_words_but_not_the_identity() {
    let mut editor = setup();
    let id = add_halo_dunia(&mut editor);
    editor
        .apply(Command::UpdateClip {
            clip_id: id.clone(),
            patch: ClipPatch {
                text: Some(None),
                ..Default::default()
            },
        })
        .expect("clears the style");
    let clip = clip_of(&editor, &id);
    assert!(clip.text.is_none(), "the style is gone");
    assert!(
        clip.words.is_none(),
        "no content means nothing to highlight: the words are absent by design, never stale"
    );
    assert!(clip.is_caption(), "the stored identity survives (T5)");
}
