//! E5: Link / Unlink audio and video (owner decision 16 Oct 2026, the
//! Premiere model), and E6: the whole-lane selection rule behind Shift + L.
//!
//! The roadmap's seven E5 tests, as far as this crate reaches. Test 7
//! ("export audio of the clip still produces the same file") needs FFmpeg;
//! its sandbox half is here instead: Link touches nothing an export reads
//! (media, stream, mute, filters, source range), and the wire shape is
//! pinned so an older build's document is unchanged while nothing is
//! linked.

use concat_project::DocumentSettings;
use concat_project::captionmap::Word;
use concat_project::commands::{ClipMove, Command, NewMedia, TrimEdge};
use concat_project::editor::Editor;
use concat_project::model::{ClipKind, MediaKind, TextStyle};

fn media(editor: &mut Editor, name: &str, kind: MediaKind, has_audio: bool) -> String {
    editor
        .apply(Command::AddMedia {
            item: NewMedia {
                path: format!("/{name}"),
                name: name.to_owned(),
                duration: Some(10.0),
                kind,
                width: Some(1920),
                height: Some(1080),
                frame_rate: Some(30.0),
                frame_rate_fraction: Some("30/1".to_owned()),
                video_codec: Some("h264".to_owned()),
                audio_codec: has_audio.then(|| "aac".to_owned()),
                has_audio,
                audio_tracks: Vec::new(),
                origin: None,
                color_space: Default::default(),
            },
        })
        .expect("adds media")
        .created_id
        .expect("media id")
}

/// A video clip on V (the first lane) and an audio clip on A (a lane added
/// under it), both at `start` for 10 s. Returns (video id, audio id).
fn pair(editor: &mut Editor, start: f64) -> (String, String) {
    let video_media = media(editor, "a.mp4", MediaKind::Video, true);
    let audio_media = media(editor, "a.wav", MediaKind::Audio, true);
    let v_track = editor.project().active().tracks[0].id.clone();
    let a_track = editor
        .apply(Command::AddTrack { bottom: true })
        .expect("adds the audio lane")
        .created_id
        .expect("track id");
    let video = editor
        .apply(Command::AddClip {
            media_id: video_media,
            track_id: v_track,
            start,
            ripple: false,
        })
        .expect("adds the video")
        .created_id
        .expect("video id");
    let audio = editor
        .apply(Command::AddClip {
            media_id: audio_media,
            track_id: a_track,
            start,
            ripple: false,
        })
        .expect("adds the audio")
        .created_id
        .expect("audio id");
    (video, audio)
}

fn settings() -> DocumentSettings {
    DocumentSettings {
        name: "E5".to_owned(),
        width: 1920,
        height: 1080,
        rate_num: 30,
        rate_den: 1,
    }
}

fn start_of(editor: &Editor, id: &str) -> f64 {
    editor
        .project()
        .active()
        .clip(id)
        .expect("clip exists")
        .start
}

fn duration_of(editor: &Editor, id: &str) -> f64 {
    editor
        .project()
        .active()
        .clip(id)
        .expect("clip exists")
        .duration
}

fn group_of(editor: &Editor, id: &str) -> Option<String> {
    editor
        .project()
        .active()
        .clip(id)
        .expect("clip exists")
        .link_group
        .clone()
}

fn move_to(editor: &mut Editor, id: &str, start: f64) {
    let track_id = editor
        .project()
        .active()
        .clip(id)
        .expect("clip exists")
        .track_id
        .clone();
    editor
        .apply(Command::MoveClips {
            moves: vec![ClipMove {
                clip_id: id.to_owned(),
                start,
                track_id,
            }],
        })
        .expect("moves");
}

fn link(editor: &mut Editor, ids: &[&str]) {
    editor
        .apply(Command::LinkClips {
            clip_ids: ids.iter().map(|id| (*id).to_owned()).collect(),
        })
        .expect("links");
}

#[test]
fn test_1_link_then_move_the_video_moves_the_audio_by_the_same_offset() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 2.0);
    link(&mut editor, &[&video, &audio]);
    assert!(
        group_of(&editor, &video).is_some(),
        "the video is in a group"
    );
    assert_eq!(
        group_of(&editor, &video),
        group_of(&editor, &audio),
        "the same group"
    );
    let selection = editor.project().active().link_mates(&video);
    assert_eq!(
        selection.len(),
        2,
        "selecting one clip of the group selects the group"
    );

    move_to(&mut editor, &video, 5.0);
    assert_eq!(start_of(&editor, &video), 5.0);
    assert_eq!(start_of(&editor, &audio), 5.0, "the audio moved with it");

    // And the other way: dragging the audio takes the video.
    move_to(&mut editor, &audio, 1.0);
    assert_eq!(start_of(&editor, &video), 1.0);
    assert_eq!(start_of(&editor, &audio), 1.0);
}

#[test]
fn test_2_unlink_then_move_the_video_leaves_the_audio_put() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 2.0);
    link(&mut editor, &[&video, &audio]);
    editor
        .apply(Command::UnlinkClips {
            clip_ids: vec![video.clone()],
        })
        .expect("unlinks");
    assert!(group_of(&editor, &video).is_none());
    assert!(
        group_of(&editor, &audio).is_none(),
        "unlinking one member dissolves the group for both"
    );
    assert!(editor.project().active().link_mates(&video).is_empty());

    move_to(&mut editor, &video, 5.0);
    assert_eq!(start_of(&editor, &video), 5.0);
    assert_eq!(start_of(&editor, &audio), 2.0, "the audio stayed");
}

#[test]
fn test_3_relink_and_the_group_moves_again() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 2.0);
    link(&mut editor, &[&video, &audio]);
    editor
        .apply(Command::UnlinkClips {
            clip_ids: vec![audio.clone()],
        })
        .expect("unlinks");
    move_to(&mut editor, &video, 4.0);
    assert_eq!(start_of(&editor, &audio), 2.0, "apart while unlinked");

    // Relink with the clips offset: the offset between them is kept.
    link(&mut editor, &[&video, &audio]);
    move_to(&mut editor, &video, 7.0);
    assert_eq!(start_of(&editor, &video), 7.0);
    assert_eq!(
        start_of(&editor, &audio),
        5.0,
        "the audio moved by the same 3 s and kept its 2 s offset"
    );
}

#[test]
fn test_4_a_caption_follows_its_video_through_link_unlink_and_move() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 0.0);
    let caption = editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 1.0,
            style: Some(TextStyle {
                content: "halo dunia".to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(2.0),
            offset_y: Some(0.38),
            caption_of: Some(video.clone()),
            words: Some(vec![
                Word {
                    text: "halo".to_owned(),
                    start: 1.0,
                    end: 1.5,
                },
                Word {
                    text: "dunia".to_owned(),
                    start: 1.5,
                    end: 2.0,
                },
            ]),
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id");

    // A caption cannot be a group member: it already follows the clip.
    let refused = editor.apply(Command::LinkClips {
        clip_ids: vec![video.clone(), caption.clone()],
    });
    assert!(refused.is_err(), "a caption is refused as a link member");

    link(&mut editor, &[&video, &audio]);
    move_to(&mut editor, &audio, 3.0);
    assert_eq!(start_of(&editor, &video), 3.0, "video followed the audio");
    assert_eq!(
        start_of(&editor, &caption),
        4.0,
        "the caption followed the video through the group move"
    );

    editor
        .apply(Command::UnlinkClips {
            clip_ids: vec![video.clone()],
        })
        .expect("unlinks");
    move_to(&mut editor, &video, 6.0);
    assert_eq!(start_of(&editor, &caption), 7.0, "still follows its video");
    assert_eq!(start_of(&editor, &audio), 3.0, "the audio no longer does");
}

#[test]
fn test_5_an_old_project_with_detached_from_loads_as_a_linked_group() {
    let mut editor = Editor::new();
    let video_media = media(&mut editor, "a.mp4", MediaKind::Video, true);
    let track = editor.project().active().tracks[0].id.clone();
    let video = editor
        .apply(Command::AddClip {
            media_id: video_media,
            track_id: track,
            start: 1.0,
            ripple: false,
        })
        .expect("adds the video")
        .created_id
        .expect("video id");
    let sound = editor
        .apply(Command::DetachAudio {
            clip_id: video.clone(),
        })
        .expect("detaches")
        .created_id
        .expect("sound id");
    // The pre-E5 document: a detached pair, no link group anywhere.
    let mut document = editor.to_document(&settings());
    for clip in document["timelines"][0]["clips"]
        .as_array_mut()
        .expect("clips")
    {
        clip.as_object_mut().expect("object").remove("linkGroup");
    }
    assert!(
        !document.to_string().contains("linkGroup"),
        "fixture is a pre-E5 document"
    );

    let restored = Editor::from_document(&document).expect("loads");
    let timeline = restored.project().active();
    let video_clip = timeline.clip(&video).expect("video survives");
    let sound_clip = timeline.clip(&sound).expect("sound survives");
    assert!(
        video_clip.link_group.is_some(),
        "the pair is linked on load"
    );
    assert_eq!(video_clip.link_group, sound_clip.link_group);
    assert_eq!(video_clip.muted, Some(true), "the video's mute is kept");
    assert_eq!(video_clip.start, 1.0);
    assert_eq!(sound_clip.start, 1.0, "same timing");
    assert_eq!(sound_clip.kind, ClipKind::Audio);

    // And the group works: the old pair now moves together.
    let mut restored = restored;
    move_to(&mut restored, &video, 4.0);
    assert_eq!(start_of(&restored, &sound), 4.0);
}

#[test]
fn test_6_one_undo_restores_each_link_change() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 2.0);
    link(&mut editor, &[&video, &audio]);
    assert!(group_of(&editor, &video).is_some());
    assert!(editor.undo(), "one undo for the link");
    assert!(group_of(&editor, &video).is_none());
    assert!(group_of(&editor, &audio).is_none());
    assert!(editor.redo(), "and it comes back");
    assert!(group_of(&editor, &video).is_some());

    editor
        .apply(Command::UnlinkClips {
            clip_ids: vec![audio.clone()],
        })
        .expect("unlinks");
    assert!(group_of(&editor, &video).is_none());
    assert!(editor.undo(), "one undo for the unlink");
    assert_eq!(group_of(&editor, &video), group_of(&editor, &audio));
    assert!(group_of(&editor, &video).is_some());

    // A group move is one undo step too.
    move_to(&mut editor, &video, 6.0);
    assert_eq!(start_of(&editor, &audio), 6.0);
    assert!(editor.undo());
    assert_eq!(start_of(&editor, &video), 2.0);
    assert_eq!(start_of(&editor, &audio), 2.0, "both back in one step");
}

#[test]
fn test_7_link_touches_nothing_an_export_reads_and_the_wire_is_unchanged_until_linked() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 2.0);
    let before = editor.to_document(&settings());
    assert!(
        !before.to_string().contains("linkGroup"),
        "no linkGroup key while nothing is linked: an older build's document is unchanged"
    );
    let video_before = editor
        .project()
        .active()
        .clip(&video)
        .cloned()
        .expect("video");

    link(&mut editor, &[&video, &audio]);
    let video_after = editor
        .project()
        .active()
        .clip(&video)
        .cloned()
        .expect("video");
    assert_eq!(video_before.media_id, video_after.media_id);
    assert_eq!(video_before.audio_stream, video_after.audio_stream);
    assert_eq!(video_before.muted, video_after.muted);
    assert_eq!(video_before.filters, video_after.filters);
    assert_eq!(video_before.source_start, video_after.source_start);
    assert_eq!(video_before.duration, video_after.duration);
    assert_eq!(video_before.speed, video_after.speed);

    // Round trip: the group survives a save and a load.
    let document = editor.to_document(&settings());
    assert!(document.to_string().contains("linkGroup"));
    let restored = Editor::from_document(&document).expect("loads");
    assert_eq!(
        restored
            .project()
            .active()
            .clip(&video)
            .expect("video")
            .link_group,
        video_after.link_group
    );
}

#[test]
fn a_linked_split_leaves_two_linked_pairs_and_a_linked_delete_takes_the_mate() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 0.0);
    link(&mut editor, &[&video, &audio]);
    let clips_before = editor.project().active().clips.len();
    editor
        .apply(Command::SplitClips {
            clip_ids: vec![video.clone()],
            time: 4.0,
        })
        .expect("splits");
    let timeline = editor.project().active();
    assert_eq!(
        timeline.clips.len(),
        clips_before + 2,
        "both members were cut"
    );
    assert_eq!(duration_of(&editor, &video), 4.0);
    assert_eq!(
        duration_of(&editor, &audio),
        4.0,
        "the audio was cut at the same instant"
    );
    let heads = timeline.link_mates(&video);
    assert_eq!(heads.len(), 2, "the head pair is one group");
    assert!(heads.contains(&audio));
    let tails: Vec<String> = timeline
        .clips
        .iter()
        .filter(|clip| clip.start == 4.0)
        .map(|clip| clip.id.clone())
        .collect();
    assert_eq!(tails.len(), 2);
    let tail_group = timeline.link_mates(&tails[0]);
    assert_eq!(tail_group.len(), 2, "the tail pair is a group of its own");
    assert_ne!(
        group_of(&editor, &video),
        group_of(&editor, &tails[0]),
        "two groups, not one group of four"
    );

    // Delete the head video: its audio goes with it.
    editor
        .apply(Command::RemoveClips {
            clip_ids: vec![video.clone()],
            ripple: false,
        })
        .expect("removes");
    let timeline = editor.project().active();
    assert!(timeline.clip(&video).is_none());
    assert!(timeline.clip(&audio).is_none(), "the linked audio went too");
    assert_eq!(timeline.clips.len(), 2, "the tail pair stands");
}

#[test]
fn a_linked_trim_moves_the_shared_edge_on_both_and_leaves_an_unshared_edge_alone() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 0.0);
    link(&mut editor, &[&video, &audio]);
    editor
        .apply(Command::TrimClip {
            clip_id: video.clone(),
            edge: TrimEdge::End,
            delta: -2.0,
            ripple: false,
        })
        .expect("trims");
    assert_eq!(duration_of(&editor, &video), 8.0);
    assert_eq!(
        duration_of(&editor, &audio),
        8.0,
        "shared tail trimmed together"
    );
    assert!(editor.undo());
    assert_eq!(
        duration_of(&editor, &audio),
        10.0,
        "one undo for the linked trim"
    );

    // Make the audio's tail different, then trim the video's tail: the
    // audio's edge is not the dragged one and stays.
    editor
        .apply(Command::UnlinkClips {
            clip_ids: vec![video.clone()],
        })
        .expect("unlinks");
    editor
        .apply(Command::TrimClip {
            clip_id: audio.clone(),
            edge: TrimEdge::End,
            delta: -3.0,
            ripple: false,
        })
        .expect("trims the audio alone");
    link(&mut editor, &[&video, &audio]);
    editor
        .apply(Command::TrimClip {
            clip_id: video.clone(),
            edge: TrimEdge::End,
            delta: -1.0,
            ripple: false,
        })
        .expect("trims");
    assert_eq!(duration_of(&editor, &video), 9.0);
    assert_eq!(
        duration_of(&editor, &audio),
        7.0,
        "an unshared edge is left alone"
    );
}

#[test]
fn link_refuses_one_clip_and_merges_groups() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 0.0);
    let refused = editor.apply(Command::LinkClips {
        clip_ids: vec![video.clone()],
    });
    assert!(refused.is_err(), "one clip is not a link");
    let stale = editor.apply(Command::LinkClips {
        clip_ids: vec![video.clone(), "c-gone".to_owned()],
    });
    assert!(stale.is_err(), "a stale id does not count as a second clip");

    // A third clip linked to a member joins the existing group: a merge.
    let image = media(&mut editor, "b.png", MediaKind::Image, false);
    let track = editor
        .apply(Command::AddTrack { bottom: false })
        .expect("adds a lane")
        .created_id
        .expect("track id");
    let still = editor
        .apply(Command::AddClip {
            media_id: image,
            track_id: track,
            start: 0.0,
            ripple: false,
        })
        .expect("adds the still")
        .created_id
        .expect("still id");
    link(&mut editor, &[&video, &audio]);
    link(&mut editor, &[&still, &audio]);
    assert_eq!(editor.project().active().link_mates(&video).len(), 3);
    assert_eq!(group_of(&editor, &still), group_of(&editor, &video));

    // Linking an already-complete group is a no-op, not an undo step.
    let outcome = editor
        .apply(Command::LinkClips {
            clip_ids: vec![video.clone(), audio.clone(), still.clone()],
        })
        .expect("ok");
    assert!(!outcome.applied);
}

#[test]
fn e6_the_whole_lane_is_every_clip_on_it_first_to_last_and_no_other_lane() {
    let mut editor = Editor::new();
    let (video, audio) = pair(&mut editor, 0.0);
    let v_track = editor
        .project()
        .active()
        .clip(&video)
        .expect("video")
        .track_id
        .clone();
    let a_track = editor
        .project()
        .active()
        .clip(&audio)
        .expect("audio")
        .track_id
        .clone();
    let later_media = media(&mut editor, "c.mp4", MediaKind::Video, true);
    let later = editor
        .apply(Command::AddClip {
            media_id: later_media.clone(),
            track_id: v_track.clone(),
            start: 20.0,
            ripple: false,
        })
        .expect("adds")
        .created_id
        .expect("id");
    let middle = editor
        .apply(Command::AddClip {
            media_id: later_media,
            track_id: v_track.clone(),
            start: 12.0,
            ripple: false,
        })
        .expect("adds")
        .created_id
        .expect("id");

    let timeline = editor.project().active();
    assert_eq!(
        timeline.clips_on_track(&v_track),
        vec![video.clone(), middle.clone(), later.clone()],
        "V holds its three clips in time order, not insertion order"
    );
    assert_eq!(timeline.clips_on_track(&a_track), vec![audio.clone()]);
    assert!(timeline.clips_on_track("t-gone").is_empty());
    // The lane selection does not pull a linked mate off another lane
    // by itself; the UI widens with `with_link_mates` if it wants to.
    link(&mut editor, &[&video, &audio]);
    let timeline = editor.project().active();
    assert_eq!(timeline.clips_on_track(&v_track).len(), 3);
    let widened = timeline.with_link_mates(
        timeline
            .clips_on_track(&v_track)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        widened.len(),
        4,
        "widened, the lane brings the linked audio"
    );
}
