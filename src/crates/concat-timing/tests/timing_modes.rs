//! Regression checks for a genuine pre-snapping baseline and a controlled
//! Level 1 versus Level 1.5 comparison. No native libraries or model needed.

use concat_timing::snap::{Mode, Params, snap_words, snap_words_with_mode};
use concat_timing::{Signals, Source, Word};

fn zcr_only_onset() -> Signals {
    let mut zcr = vec![0.0; 600];
    zcr[200..240].fill(0.5);
    Signals {
        energy: vec![-40.0; 600],
        zcr,
        flux: vec![0.0; 600],
    }
}

#[test]
fn unsnapped_mode_preserves_even_invalid_input_for_an_honest_baseline() {
    let words = vec![
        Word::whisper("outside", -0.1, 0.2),
        Word::whisper("reversed", 1.4, 1.2),
        Word::whisper("earlier", 1.0, 1.5),
    ];
    let report = snap_words_with_mode(
        &words,
        &zcr_only_onset(),
        (0.0, 2.0),
        &Params::default(),
        Mode::Unsnapped,
    );
    assert_eq!(
        report.words, words,
        "no clamp, repair or snap in a baseline"
    );
    assert_eq!((report.snapped, report.reverted, report.tied), (0, 0, 0));
}

#[test]
fn level1_does_not_consult_a_zcr_only_onset() {
    let words = vec![Word::whisper("satu", 1.10, 1.40)];
    let report = snap_words_with_mode(
        &words,
        &zcr_only_onset(),
        (0.5, 2.0),
        &Params::default(),
        Mode::Level1,
    );
    assert_eq!(report.words, words);
    assert_eq!(report.snapped, 0);
}

#[test]
fn level15_can_be_compared_against_level1_on_the_same_input() {
    let words = vec![Word::whisper("satu", 1.10, 1.40)];
    let signals = zcr_only_onset();
    let report = snap_words_with_mode(
        &words,
        &signals,
        (0.5, 2.0),
        &Params::default(),
        Mode::Level15,
    );
    assert!((report.words[0].start - 1.0).abs() < 1e-9);
    assert_eq!(report.words[0].source, Source::Snapped);
    assert_eq!(report.snapped, 1);
}

#[test]
fn the_existing_production_entry_point_keeps_level15_behavior() {
    let words = vec![Word::whisper("satu", 1.10, 1.40)];
    let signals = zcr_only_onset();
    let old_api = snap_words(&words, &signals, (0.5, 2.0), &Params::default());
    let explicit = snap_words_with_mode(
        &words,
        &signals,
        (0.5, 2.0),
        &Params::default(),
        Mode::default(),
    );
    assert_eq!(Mode::default(), Mode::Level15);
    assert_eq!(old_api.words, explicit.words);
    assert_eq!(old_api.snapped, explicit.snapped);
}
