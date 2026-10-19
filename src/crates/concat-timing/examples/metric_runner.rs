// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The Tier 1 table and WER, in seconds, from the corpus (D5-B, sandbox half).
//!
//! ```text
//! cargo run -p concat-timing --example metric_runner -- \
//!     06-corpus/whisper-cache.json 06-corpus/audio-16k.f32 \
//!     [--ref reference.srt] [--snap=level1|--snap=level15] [--save run.json]
//! ```
//!
//! The cache is made once, on the machine that has Whisper and FFmpeg, by
//! `concat-speech`'s `make_cache` example; this runner never transcribes.
//! Everything it prints is reproducible from the two files plus `--ref`, so a
//! run and its baseline can be diffed by eye and by `delta_vs_baseline`.
//!
//! No snap flag preserves the cache exactly. `--snap=level1` evaluates T2;
//! `--snap=level15` evaluates T3 on the same cache (`--snap` remains its alias).
//! Provenance must declare onset_snapping=false. Old/synthetic caches without
//! provenance require explicit `--allow-legacy-cache`, and their run is marked
//! unverified. A cache known to have been snapped is always refused.

use std::path::Path;
use std::{env, fs, process};

use concat_timing::Signals;
use concat_timing::metrics::{Tier1, Wer, delta_vs_baseline, normalize, tier1, wer};
use concat_timing::snap::{self, Mode, Params};
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
    #[serde(default)]
    provenance: Option<serde_json::Value>,
    segments: Vec<CacheSegment>,
}

#[derive(Debug)]
struct Options {
    positional: Vec<String>,
    reference: Option<String>,
    save: Option<String>,
    mode: Mode,
    allow_legacy: bool,
}

const USAGE: &str = "usage: metric_runner <cache.json> <audio-16k.f32> [--ref <reference.srt>] [--snap|--snap=level1|--snap=level15] [--save <run.json>] [--allow-legacy-cache]";

fn options(args: &[String]) -> Result<Options, String> {
    let mut out = Options {
        positional: Vec::new(),
        reference: None,
        save: None,
        mode: Mode::Unsnapped,
        allow_legacy: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ref" | "--save" => {
                let flag = &args[i];
                i += 1;
                let value = args
                    .get(i)
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| format!("{flag} needs a file path"))?
                    .clone();
                if flag == "--ref" {
                    out.reference = Some(value);
                } else {
                    out.save = Some(value);
                }
            }
            "--snap" | "--snap=level15" => out.mode = Mode::Level15,
            "--snap=level1" => out.mode = Mode::Level1,
            "--allow-legacy-cache" => out.allow_legacy = true,
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other => out.positional.push(other.to_owned()),
        }
        i += 1;
    }
    if out.positional.len() != 2 {
        return Err(USAGE.to_owned());
    }
    Ok(out)
}

// A legacy cache is explicitly usable for exploration, never silently
// promoted to a declared pre-snap baseline. The override cannot accept a
// cache that declares snapping was already applied.
fn declared_pre_snap(cache: &Cache, allow_legacy: bool) -> Result<bool, String> {
    match &cache.provenance {
        Some(meta) if meta.get("onset_snapping").and_then(|v| v.as_bool()) == Some(false) => Ok(true),
        Some(_) => Err("cache does not declare onset_snapping=false; regenerate with the updated make_cache before evaluating the first snapping pass".to_owned()),
        None if allow_legacy => Ok(false),
        None => Err("cache has no timing provenance; regenerate it with the updated make_cache, or explicitly use --allow-legacy-cache for unverified legacy/synthetic exploration".to_owned()),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let opts = options(&args).unwrap_or_else(|why| fail("arguments", &why));
    let positional = opts.positional;
    let reference = opts.reference;
    let save = opts.save;
    let mode = opts.mode;
    let snapping = mode != Mode::Unsnapped;

    let cache: Cache = serde_json::from_str(&read(&positional[0]))
        .unwrap_or_else(|e| fail(&positional[0], &e.to_string()));
    if cache.schema != 1 {
        eprintln!(
            "cache schema {} is not 1; refuse to guess at its shape",
            cache.schema
        );
        process::exit(2);
    }
    let pre_snap = declared_pre_snap(&cache, opts.allow_legacy)
        .unwrap_or_else(|why| fail(&positional[0], &why));
    if !pre_snap {
        eprintln!(
            "WARNING: legacy/synthetic cache has unverified timing provenance; this is not accepted proof of an unsnapped baseline."
        );
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
            let report = snap::snap_words_with_mode(
                &words,
                &signals,
                (segment.start, segment.end),
                &params,
                mode,
            );
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
    println!("timing mode   {}", mode.name());
    println!(
        "cache timing  {}",
        if pre_snap {
            "declared pre-onset-snap"
        } else {
            "UNVERIFIED legacy/synthetic"
        }
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
        println!(
            "snap          {} · snapped {snapped} · reverted {reverted} · tied {tied}",
            mode.name()
        );
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
            "timing_mode": mode.name(),
            "cache_declares_pre_snap": pre_snap,
            "cache_provenance": cache.provenance,
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
    let normalized = srt.replace("\r\n", "\n");
    for block in normalized.split("\n\n") {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(rest: &[&str]) -> Vec<String> {
        ["cache.json", "audio.f32"]
            .into_iter()
            .chain(rest.iter().copied())
            .map(str::to_owned)
            .collect()
    }

    fn cache(meta: Option<serde_json::Value>) -> Cache {
        Cache {
            schema: 1,
            provenance: meta,
            segments: Vec::new(),
        }
    }

    #[test]
    fn new_cache_declares_pre_snap_and_is_accepted() {
        let c = cache(Some(
            serde_json::json!({"onset_snapping": false, "backend": "whisper-rs"}),
        ));
        assert!(declared_pre_snap(&c, false).unwrap());
    }

    #[test]
    fn already_snapped_cache_is_refused_even_with_a_legacy_override() {
        let c = cache(Some(serde_json::json!({"onset_snapping": true})));
        assert!(declared_pre_snap(&c, false).is_err());
        assert!(declared_pre_snap(&c, true).is_err());
    }

    #[test]
    fn missing_provenance_requires_an_explicit_unverified_legacy_override() {
        let c = cache(None);
        assert!(declared_pre_snap(&c, false).is_err());
        assert!(!declared_pre_snap(&c, true).unwrap());
    }

    #[test]
    fn malformed_provenance_cannot_silently_become_an_unsnapped_claim() {
        for meta in [
            serde_json::json!({}),
            serde_json::json!({"onset_snapping": "false"}),
        ] {
            assert!(declared_pre_snap(&cache(Some(meta)), true).is_err());
        }
    }

    #[test]
    fn cli_selects_comparable_modes_and_preserves_the_snap_alias() {
        assert_eq!(options(&args(&[])).unwrap().mode, Mode::Unsnapped);
        assert_eq!(
            options(&args(&["--snap=level1"])).unwrap().mode,
            Mode::Level1
        );
        assert_eq!(
            options(&args(&["--snap=level15"])).unwrap().mode,
            Mode::Level15
        );
        assert_eq!(options(&args(&["--snap"])).unwrap().mode, Mode::Level15);
    }

    #[test]
    fn cli_refuses_missing_paths_and_unknown_switches() {
        assert!(options(&args(&["--ref"])).is_err());
        assert!(options(&args(&["--save", "--snap"])).is_err());
        assert!(options(&args(&["--snap=typo"])).is_err());
    }

    #[test]
    fn reference_srt_has_the_same_text_on_windows_and_unix() {
        let unix = "1\n00:00:00,000 --> 00:00:01,000\nHalo dunia.\n\n2\n00:00:01,000 --> 00:00:02,000\nYa, benar.\n";
        assert_eq!(srt_text(unix), "Halo dunia. Ya, benar. ");
        assert_eq!(srt_text(&unix.replace('\n', "\r\n")), srt_text(unix));
    }
}
