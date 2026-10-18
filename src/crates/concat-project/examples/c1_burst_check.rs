//! C1 validation: the built burst (3-word) chunking, run over the same
//! episode before and after T2, compared by the Tier 1 table and by the
//! chunk list itself. Validation only - no new UI, no new chunking code:
//! this drives the shipped `resplit_cues(.., Burst)` path.
//!
//! Usage:
//!   cargo run -p concat-project --example c1_burst_check -- \
//!       <baseline-run.json> <snapped-run.json>
//!
//! The run JSONs are the Tier-1 runner's output (`06-corpus/runs/`): their
//! `segments[].words` carry the heard timings, which become `Cue`s here
//! exactly as the captions dialog would see them.

use concat_project::captionmap::{CaptionLayoutPreset, Cue, Word, WordSource, resplit_cues};
use serde_json::Value;
use std::process::ExitCode;

fn cues_from_run(path: &str) -> (Vec<Cue>, Value) {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let run: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"));
    let segments = run["segments"].as_array().expect("segments array");
    let cues = segments
        .iter()
        .map(|seg| {
            let words: Vec<Word> = seg["words"]
                .as_array()
                .expect("segment words")
                .iter()
                .map(|w| Word {
                    text: w["text"].as_str().expect("word text").to_owned(),
                    start: w["start"].as_f64().expect("word start"),
                    end: w["end"].as_f64().expect("word end"),
                })
                .collect();
            Cue {
                start: seg["start"].as_f64().expect("segment start"),
                end: seg["end"].as_f64().expect("segment end"),
                text: words
                    .iter()
                    .map(|w| w.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                words,
                word_source: Some(WordSource::Whisper),
                ..Cue::default()
            }
        })
        .collect();
    (cues, run)
}

fn print_tier1(label: &str, run: &Value) {
    let t = &run["tier1"];
    println!(
        "{label} Tier 1: words={} onset_rate={} monotonic_violations={} outliers={} | wer={} | snapped={} reverted={} tied={}",
        t["words"],
        t["onset_rate"],
        t["monotonic_violations"],
        t["outliers"],
        run["wer"],
        run["counts"]["snapped"],
        run["counts"]["reverted"],
        run["counts"]["tied"],
    );
    if !run["delta"].is_null() {
        println!(
            "{label} delta: moved={} mean_shift_s={:.4} max_abs_shift_s={:.4}",
            run["delta"]["moved"],
            run["delta"]["mean_shift_s"].as_f64().unwrap_or(0.0),
            run["delta"]["max_abs_shift_s"].as_f64().unwrap_or(0.0),
        );
    }
}

fn print_bursts(label: &str, cues: &[Cue]) {
    println!("{label} burst chunks ({}):", cues.len());
    for cue in cues {
        println!(
            "  {:7.3} - {:7.3}  ({} words, {:4.2}s)  {}",
            cue.start,
            cue.end,
            cue.words.len(),
            cue.end - cue.start,
            cue.text,
        );
    }
}

/// The burst policy's own limits plus the ordering every layout must keep.
/// Returns the list of failures so `main` can exit non-zero on any.
fn check_bursts(label: &str, cues: &[Cue], source: &[Cue]) -> Vec<String> {
    let mut fails = Vec::new();
    for cue in cues {
        if cue.words.len() > 3 {
            fails.push(format!("{label}: burst over 3 words: {:?}", cue.text));
        }
        if cue.text.chars().count() > 18 {
            fails.push(format!("{label}: burst over 18 chars: {:?}", cue.text));
        }
        if cue.end - cue.start > 2.5 + 1e-9 {
            fails.push(format!("{label}: burst over 2.5 s: {:?}", cue.text));
        }
        if cue.end <= cue.start {
            fails.push(format!("{label}: empty span: {:?}", cue.text));
        }
    }
    for pair in cues.windows(2) {
        if pair[1].start + 1e-6 < pair[0].end {
            fails.push(format!(
                "{label}: overlapping bursts: {:?} then {:?}",
                pair[0].text, pair[1].text
            ));
        }
    }
    let kept: Vec<&str> = cues
        .iter()
        .flat_map(|cue| cue.words.iter().map(|w| w.text.as_str()))
        .collect();
    let original: Vec<&str> = source
        .iter()
        .flat_map(|cue| cue.words.iter().map(|w| w.text.as_str()))
        .collect();
    if kept != original {
        fails.push(format!(
            "{label}: chunking lost or reordered words ({} in, {} out)",
            original.len(),
            kept.len()
        ));
    }
    fails
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: c1_burst_check <baseline-run.json> <snapped-run.json>");
        return ExitCode::FAILURE;
    }
    let (before_src, before_run) = cues_from_run(&args[0]);
    let (after_src, after_run) = cues_from_run(&args[1]);

    print_tier1("before T2", &before_run);
    print_tier1("after T2 ", &after_run);
    println!();

    let before = resplit_cues(&before_src, CaptionLayoutPreset::Burst);
    let after = resplit_cues(&after_src, CaptionLayoutPreset::Burst);
    print_bursts("before T2", &before);
    println!();
    print_bursts("after T2 ", &after);
    println!();

    let same_bounds = before
        .iter()
        .zip(after.iter())
        .all(|(b, a)| b.text == a.text && (b.start - a.start).abs() < 1e-9);
    println!(
        "chunk boundaries identical before/after T2: {} (timings inside the chunks follow the snapped words)",
        if same_bounds { "yes" } else { "no" }
    );

    let mut fails = check_bursts("before T2", &before, &before_src);
    fails.extend(check_bursts("after T2", &after, &after_src));
    if fails.is_empty() {
        println!(
            "burst checks: PASS (<=3 words, <=18 chars, <=2.5 s, ordered, non-overlapping, no word lost)"
        );
        ExitCode::SUCCESS
    } else {
        for fail in &fails {
            println!("burst checks: FAIL - {fail}");
        }
        ExitCode::FAILURE
    }
}
