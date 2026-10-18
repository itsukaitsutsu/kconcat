// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The Tier 1 table and WER, in seconds, from the corpus (D5-B, sandbox half).
//!
//! ```text
//! cargo run -p concat-timing --example metric_runner -- \
//!     06-corpus/whisper-cache.json 06-corpus/audio-16k.f32 \
//!     [--ref 06-corpus/user-video.srt] [--snap] [--save 06-corpus/runs/run.json]
//! ```
//!
//! The cache is made once, on the machine that has Whisper and FFmpeg, by
//! `concat-speech`'s `make_cache` example; this runner never transcribes.
//! Everything it prints is reproducible from the two files plus `--ref`, so a
//! run and its baseline can be diffed by eye and by `delta_vs_baseline`.
//!
//! With `--snap` the runner applies Level 1 (`snap::snap_words`, the manual's
//! §2) to every segment on the same frames it measures, prints the Tier 1
//! table of the *snapped* words, and reports Δ against the un-snapped cache -
//! the before/after the roadmap's T2 asks for. Save one run each way and the
//! two JSONs diff cleanly.

use std::path::Path;
use std::{env, fs, process};

use concat_timing::Signals;
use concat_timing::metrics::{Tier1, Wer, delta_vs_baseline, normalize, tier1, wer};
use concat_timing::snap::{self, Params};
use concat_timing::{Source, Word};

/// One segment of the cache, as schema 1 stores it.
#[derive(serde::Deserialize)]
struct CacheSegment {
    start: f64,
    end: f64,
    #[serde(default)]
    text: String,
    words: Vec<CacheWord>,
}

/// One word of the cache: text and span, no marker - the cache is what
/// Whisper said, so every word in it starts life as `Source::Whisper`.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct CacheWord {
    text: String,
    start: f64,
    end: f64,
}

#[derive(serde::Deserialize)]
struct Cache {
    schema: u32,
    segments: Vec<CacheSegment>,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut positional: Vec<String> = Vec::new();
    let mut reference: Option<String> = None;
    let mut save: Option<String> = None;
    let mut snapping = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ref" => {
                i += 1;
                reference = args.get(i).cloned();
            }
            "--save" => {
                i += 1;
                save = args.get(i).cloned();
            }
            "--snap" => snapping = true,
            other => positional.push(other.to_string()),
        }
        i += 1;
    }
    if positional.len() != 2 {
        eprintln!(
            "usage: metric_runner <whisper-cache.json> <audio-16k.f32> [--ref <file.srt>] [--snap] [--save <out.json>]"
        );
        process::exit(2);
    }

    let cache: Cache = serde_json::from_str(&read(&positional[0]))
        .unwrap_or_else(|e| fail(&positional[0], &e.to_string()));
    if cache.schema != 1 {
        eprintln!(
            "cache schema {} is not 1; refuse to guess at its shape",
            cache.schema
        );
        process::exit(2);
    }
    let bytes = fs::read(&positional[1]).unwrap_or_else(|e| fail(&positional[1], &e.to_string()));
    if !bytes.len().is_multiple_of(4) {
        eprintln!("audio file is not a whole number of little-endian f32 samples");
        process::exit(2);
    }
    let samples: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    // Step 1: frame every signal once, for the whole continuous buffer -
    // energy for Level 1, ZCR and flux for Level 1.5, one grid for all.
    let signals = Signals::from_samples(&samples);
    let seconds = samples.len() as f64 / f64::from(concat_timing::SAMPLE_RATE);

    let params = Params::default();
    let mut words_measured = 0;
    let mut onset_hits = 0;
    let mut violations = 0;
    let mut outliers = 0;
    let (mut snapped, mut reverted, mut tied) = (0usize, 0usize, 0usize);
    let mut before_all: Vec<Word> = Vec::new();
    let mut after_all: Vec<Word> = Vec::new();
    let mut per_segment: Vec<serde_json::Value> = Vec::new();
    for segment in &cache.segments {
        let words: Vec<Word> = segment
            .words
            .iter()
            .map(|w| Word::whisper(&w.text, w.start, w.end))
            .collect();
        // Level 1 runs per segment, on the continuous buffer's frames, on the
        // cache's own clock - the same clock rule the crate's tests police.
        let measured: Vec<Word> = if snapping {
            let report = snap::snap_words(&words, &signals, (segment.start, segment.end), &params);
            snapped += report.snapped;
            reverted += report.reverted;
            tied += report.tied;
            report.words
        } else {
            words.clone()
        };
        let t: Tier1 = tier1(&measured, &signals, (segment.start, segment.end));
        words_measured += t.words;
        onset_hits += (t.onset_rate * t.words as f64).round() as usize;
        violations += t.monotonic_violations;
        outliers += t.outliers;
        per_segment.push(serde_json::json!({
            "start": segment.start,
            "end": segment.end,
            "tier1": {"onset_rate": t.onset_rate, "monotonic_violations": t.monotonic_violations,
                       "outliers": t.outliers, "words": t.words},
            "words": measured.iter().map(|w| serde_json::json!({
                "text": w.text, "start": w.start, "end": w.end,
                "source": match w.source { Source::Whisper => "whisper", Source::Snapped => "snapped" }})).collect::<Vec<_>>(),
        }));
        before_all.extend(words);
        after_all.extend(measured);
    }
    // Δ against the un-snapped cache: what snapping did, word for word. A
    // uniform mean shift with everything moved is an offset bug, not tuning.
    let delta = snapping.then(|| delta_vs_baseline(&before_all, &after_all));
    let table = Tier1 {
        onset_rate: if words_measured == 0 {
            0.0
        } else {
            onset_hits as f64 / words_measured as f64
        },
        monotonic_violations: violations,
        outliers,
        words: words_measured,
    };

    let w: Option<Wer> = reference.as_ref().map(|path| {
        let srt = read(path);
        let reference = normalize(&srt_text(&srt));
        let hypothesis: Vec<String> = normalize(
            &cache
                .segments
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        );
        wer(&reference, &hypothesis)
    });

    println!(
        "corpus        {}  ({seconds:.1} s of 16 kHz mono)",
        positional[1]
    );
    println!(
        "segments      {}  ·  words {}",
        cache.segments.len(),
        words_measured
    );
    println!("┌─────────────────────────────┬──────────┬──────────────────────┐");
    println!("│ metric                      │ value    │ target               │");
    println!("├─────────────────────────────┼──────────┼──────────────────────┤");
    println!(
        "│ onset plausibility          │ {:>7.1}% │ none yet: baseline   │",
        table.onset_rate * 100.0
    );
    println!(
        "│ monotonicity violations     │ {:>8} │ 0                    │",
        table.monotonic_violations
    );
    println!(
        "│ outliers                    │ {:>8} │ 0                    │",
        table.outliers
    );
    if let Some(w) = w {
        println!(
            "│ WER (word-level, norm.)     │ {:>7.1}% │ owner sets after T8.2│",
            w.rate * 100.0
        );
        println!(
            "│   sub / del / ins           │ {:>3}/{:<3}/{} │                      │",
            w.substitutions, w.deletions, w.insertions
        );
    }
    println!("└─────────────────────────────┴──────────┴──────────────────────┘");
    if snapping {
        println!("snap          on · snapped {snapped} · reverted {reverted} · tied {tied}");
        if let Some(d) = &delta {
            println!(
                "Δ vs cache    {} of {} words moved · mean {:+.1} ms · max {:.1} ms",
                d.moved,
                before_all.len(),
                d.mean_shift * 1000.0,
                d.max_abs_shift * 1000.0
            );
        }
    }

    if let Some(path) = save {
        let body = serde_json::json!({
            "kind": "tier1-run",
            "snap": snapping,
            "counts": {"snapped": snapped, "reverted": reverted, "tied": tied},
            "delta": delta.as_ref().map(|d| serde_json::json!({"moved": d.moved,
                "mean_shift_s": d.mean_shift, "max_abs_shift_s": d.max_abs_shift})),
            "audio": {"file": positional[1], "sample_rate": concat_timing::SAMPLE_RATE,
                       "channels": 1, "duration_s": seconds},
            "cache": positional[0],
            "tier1": {"onset_rate": table.onset_rate,
                       "monotonic_violations": table.monotonic_violations,
                       "outliers": table.outliers, "words": table.words},
            "wer": w.map(|w| serde_json::json!({"substitutions": w.substitutions,
                "deletions": w.deletions, "insertions": w.insertions,
                "errors": w.errors, "reference_words": w.reference_words, "rate": w.rate})),
            "segments": per_segment,
        });
        if let Some(dir) = Path::new(&path).parent() {
            let _ = fs::create_dir_all(dir);
        }
        fs::write(&path, serde_json::to_string_pretty(&body).unwrap())
            .unwrap_or_else(|e| fail(&path, &e.to_string()));
        println!("saved         {path}");
    }
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| fail(path, &e.to_string()))
}

fn fail(path: &str, why: &str) -> ! {
    eprintln!("{path}: {why}");
    process::exit(2);
}

/// The spoken text of an SRT file: every cue's lines, in order, timings and
/// indexes dropped. Good enough as a WER reference and as Tier 2's text;
/// `concat-project::subtitle` is the real parser and stays the one a project
/// import uses.
fn srt_text(srt: &str) -> String {
    let mut out = String::new();
    for block in srt.split("\n\n") {
        let mut lines = block.lines().filter(|l| !l.trim().is_empty());
        let Some(first) = lines.next() else { continue };
        if !first.contains("-->") {
            // An index line: the timing is the next one.
            if !lines.next().is_some_and(|l| l.contains("-->")) {
                continue;
            }
        }
        for line in lines {
            out.push_str(line.trim());
            out.push(' ');
        }
    }
    out
}
