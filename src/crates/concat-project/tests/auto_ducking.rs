//! V3: auto-ducking (owner A6, 16 Oct 2026).
//!
//! The prescribed tests: a music bed drops under speech and returns
//! after, and the change is reversible. Both routes the owner chose must
//! pass them: Route B, the transcript (the heard words are the speech),
//! and Route A, audio analysis - here against synthetic samples, because
//! decoding lives where FFmpeg lives and this crate never touches it.
//! The output is volume keyframes on the music clip, so the result stays
//! editable (ordinary keys) and reversible (one undo; `ClearClipKeys`
//! without the history).

use concat_project::captionmap::Word;
use concat_project::commands::{Command, NewMedia};
use concat_project::duck::{self, DEFAULT_AMOUNT_DB, DEFAULT_FADE_MS, DuckRegion};
use concat_project::editor::Editor;
use concat_project::model::{KeyProperty, MediaKind, TextStyle};

fn word(text: &str, start: f64, end: f64) -> Word {
    Word {
        text: text.to_owned(),
        start,
        end,
    }
}

/// A 10 s music clip at unity, and a caption over it whose heard words
/// are "halo" 2.0-2.5, "dunia" 2.6-3.0 (a 0.1 s gap - inside the default
/// fade, so one region) and "lagi" 5.0-5.8 (a 2.0 s gap - its own
/// region).
fn setup() -> (Editor, String, String) {
    let mut editor = Editor::new();
    let media_id = editor
        .apply(Command::AddMedia {
            item: NewMedia {
                path: "/music.mp4".to_owned(),
                name: "music.mp4".to_owned(),
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
        .expect("adds the music clip");
    let music_id = editor.project().active().clips[0].id.clone();
    let speech_id = editor
        .apply(Command::AddTextClip {
            above: true,
            track_id: None,
            start: 2.0,
            style: Some(TextStyle {
                content: "halo dunia lagi".to_owned(),
                ..TextStyle::default()
            }),
            duration: Some(4.0),
            offset_y: Some(0.38),
            caption_of: Some(music_id.clone()),
            words: Some(vec![
                word("halo", 2.0, 2.5),
                word("dunia", 2.6, 3.0),
                word("lagi", 5.0, 5.8),
            ]),
        })
        .expect("adds the caption")
        .created_id
        .expect("caption id");
    (editor, music_id, speech_id)
}

fn duck(music_id: &str, speech_id: &str) -> Command {
    Command::DuckMusic {
        clip_id: music_id.to_owned(),
        against_clips: vec![speech_id.to_owned()],
        against_regions: Vec::new(),
        amount_db: DEFAULT_AMOUNT_DB,
        fade_ms: DEFAULT_FADE_MS,
    }
}

/// The prescribed test, Route B: the music bed drops under speech and
/// returns after - before, mid-ramp, under, between the regions, under
/// the second one, and on the tail.
#[test]
fn a_music_bed_drops_under_speech_and_returns_after() {
    let (mut editor, music_id, speech_id) = setup();
    let outcome = editor.apply(duck(&music_id, &speech_id)).expect("ducks");
    assert!(outcome.applied, "the duck writes keys");

    let music = &editor.project().active().clips[0];
    let ducked = duck::db_to_linear(DEFAULT_AMOUNT_DB);
    assert!((ducked - 0.251_188_643_150_958).abs() < 1e-9, "{ducked}");
    // The music clip runs 0..10, so a timeline second is `t / 10` of it.
    let v = |t: f64| music.value_at(KeyProperty::Volume, t / 10.0);
    assert!((v(1.0) - music.volume).abs() < 1e-6, "before speech: full");
    assert!(
        (v(1.75) - (music.volume + ducked) / 2.0).abs() < 1e-6,
        "mid-ramp: the linear ease is halfway at 1.75 (ramp 1.5-2.0)"
    );
    assert!((v(2.75) - ducked).abs() < 1e-6, "under speech: ducked");
    assert!(
        (v(4.0) - music.volume).abs() < 1e-6,
        "after speech: returned"
    );
    assert!((v(5.4) - ducked).abs() < 1e-6, "under the second region");
    assert!((v(9.0) - music.volume).abs() < 1e-6, "the tail: full");
    // Every key the writer left is a volume key, and the ramp keys sit
    // where the fade says: 1.5 and 3.5 around the merged 2.0-3.0 region.
    let ats: Vec<f64> = music
        .keys
        .iter()
        .filter(|key| key.property == KeyProperty::Volume)
        .map(|key| key.at * 10.0)
        .collect();
    for expected in [1.5, 2.0, 3.0, 3.5, 4.5, 5.0, 5.8, 6.3] {
        assert!(
            ats.iter().any(|at| (at - expected).abs() < 1e-6),
            "a key at {expected}s: {ats:?}"
        );
    }
}

/// The prescribed test, second half: the change is reversible - one undo
/// takes the whole duck off, keys and all, because the command is one
/// snapshot.
#[test]
fn the_duck_is_one_undo() {
    let (mut editor, music_id, speech_id) = setup();
    editor.apply(duck(&music_id, &speech_id)).expect("ducks");
    assert!(editor.undo(), "the duck is one undo step");
    let music = &editor.project().active().clips[0];
    assert!(
        music
            .keys
            .iter()
            .all(|key| key.property != KeyProperty::Volume),
        "no volume keys survive the undo: {:?}",
        music.keys
    );
    assert!(
        (music.value_at(KeyProperty::Volume, 0.275) - music.volume).abs() < 1e-9,
        "and the ride is the constant again"
    );
}

/// Route A on synthetic audio: one second of silence, one second of a
/// 0.5-amplitude tone, one second of silence. The gate finds the tone
/// and only the tone, and the same writer turns it into the same keys -
/// both routes pass through one definition of "speech region".
#[test]
fn route_a_finds_the_tone_and_writes_the_same_keys() {
    let rate = 16_000f32;
    let mut samples = vec![0f32; 16_000];
    samples
        .extend((0..16_000).map(|i| 0.5 * (std::f32::consts::TAU * 440.0 * i as f32 / rate).sin()));
    samples.extend(vec![0f32; 16_000]);

    let regions = duck::regions_from_samples(&samples, rate, 0.5, 0.5);
    assert_eq!(regions.len(), 1, "the tone is one region: {regions:?}");
    assert!(
        (regions[0].0 - 1.0).abs() < 0.05,
        "starts at the tone: {regions:?}"
    );
    assert!(
        (regions[0].1 - 2.0).abs() < 0.05,
        "ends at the tone: {regions:?}"
    );

    // The command with those regions (what the host passes after it
    // decodes): the same writer, the same keys, ducked under the tone.
    let (mut editor, music_id, _) = setup();
    let outcome = editor
        .apply(Command::DuckMusic {
            clip_id: music_id,
            against_clips: Vec::new(),
            against_regions: vec![DuckRegion {
                start: regions[0].0,
                end: regions[0].1,
            }],
            amount_db: DEFAULT_AMOUNT_DB,
            fade_ms: DEFAULT_FADE_MS,
        })
        .expect("ducks");
    assert!(outcome.applied);
    let music = &editor.project().active().clips[0];
    let ducked = duck::db_to_linear(DEFAULT_AMOUNT_DB);
    assert!(
        (music.value_at(KeyProperty::Volume, 0.15) - ducked).abs() < 1e-6,
        "ducked under the tone"
    );
    assert!(
        (music.value_at(KeyProperty::Volume, 0.03) - music.volume).abs() < 1e-6,
        "full before it (t=0.3 s, ahead of any ramp the ±1-frame region edge could start)"
    );
}

/// The merge rule both routes share: a gap no longer than the fade stays
/// ducked (a ramp that cannot complete is flapping), a longer gap is two
/// regions.
#[test]
fn a_gap_shorter_than_the_fade_merges_and_a_longer_one_does_not() {
    let merged = duck::merge_regions(&[(2.0, 2.5), (2.6, 3.0), (5.0, 5.8)], 0.5);
    assert_eq!(merged, vec![(2.0, 3.0), (5.0, 5.8)]);

    let words = vec![word("a", 2.0, 2.5), word("b", 2.6, 3.0)];
    assert_eq!(duck::regions_from_words(&words, 0.5), vec![(2.0, 3.0)]);
    assert_eq!(
        duck::regions_from_words(&words, 0.05),
        vec![(2.0, 2.5), (2.6, 3.0)],
        "a fade shorter than the gap leaves the two words apart"
    );
}

/// The writer's edges: a fade of zero is a step, a region hanging over
/// the clip's edge clamps into it, a region wholly outside writes
/// nothing, and an amount above zero clamps to no duck at all.
#[test]
fn the_writer_clamps_its_edges() {
    let (editor, _, _) = setup();
    let music = &editor.project().active().clips[0];

    // Zero fade: four keys on two instants, the ducked one last at the
    // entry and the base one last at the exit, so `set_key` lands a step.
    let keys = duck::duck_keys(music, &[(2.0, 3.0)], DEFAULT_AMOUNT_DB, 0.0);
    assert_eq!(keys.len(), 4, "{keys:?}");
    let ducked = duck::db_to_linear(DEFAULT_AMOUNT_DB);
    assert!((keys[0].at - 0.2).abs() < 1e-9 && (keys[0].value - music.volume).abs() < 1e-9);
    assert!((keys[1].at - 0.2).abs() < 1e-9 && (keys[1].value - ducked).abs() < 1e-9);
    assert!((keys[2].at - 0.3).abs() < 1e-9 && (keys[2].value - ducked).abs() < 1e-9);
    assert!((keys[3].at - 0.3).abs() < 1e-9 && (keys[3].value - music.volume).abs() < 1e-9);

    // A region over the edge: every key inside 0..=1, sorted, and the
    // duck holds from the clip's own start.
    let keys = duck::duck_keys(music, &[(-1.0, 0.5)], DEFAULT_AMOUNT_DB, 0.5);
    assert!(!keys.is_empty());
    assert!(
        keys.iter().all(|key| (0.0..=1.0).contains(&key.at)),
        "{keys:?}"
    );
    assert!(
        keys.windows(2).all(|pair| pair[0].at <= pair[1].at),
        "sorted by at: {keys:?}"
    );
    assert!((keys.last().expect("a key").value - music.volume).abs() < 1e-9);

    // Wholly outside: nothing to write.
    assert!(duck::duck_keys(music, &[(12.0, 13.0)], DEFAULT_AMOUNT_DB, 0.5).is_empty());

    // An amount above zero clamps to 0 dB: the keys exist but duck
    // nothing - the dial cannot invent gain.
    let keys = duck::duck_keys(music, &[(2.0, 3.0)], 6.0, 0.5);
    assert!(
        keys.iter()
            .all(|key| (key.value - music.volume).abs() < 1e-9),
        "{keys:?}"
    );
}

/// The numbers a hand-edited command could carry: a non-finite amount or
/// region is refused before anything runs.
#[test]
fn a_non_finite_duck_is_refused() {
    let (mut editor, music_id, speech_id) = setup();
    let refused = editor.apply(Command::DuckMusic {
        clip_id: music_id,
        against_clips: vec![speech_id],
        against_regions: Vec::new(),
        amount_db: f64::NAN,
        fade_ms: DEFAULT_FADE_MS,
    });
    assert!(refused.is_err(), "NaN is refused, not written");
}
