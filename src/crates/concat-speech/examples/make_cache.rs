// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Runs Whisper once over the corpus and writes the two files the metric
//! runner reads (D5-B, owner-machine half):
//!
//! ```text
//! cargo run -p concat-speech --example make_cache -- \
//!     <media-file> 06-corpus/whisper-cache.json <model-id> [language]
//! ```
//!
//! It writes `06-corpus/audio-16k.f32` beside the cache: the same decode
//! Whisper heard, as raw little-endian f32 at 16 kHz mono, so a snapped run
//! and its baseline are measured on identical samples. Whisper runs once;
//! every later iteration of the snapper is the runner, in seconds.
//!
//! NOT VERIFIED IN THE SANDBOX: this example links FFmpeg and whisper.cpp
//! through `concat-media` and `concat-host`, so it compiles and runs only on
//! a machine with those installed (roadmap v2 §0 step 6). Read it as the
//! specification of the cache's provenance; the runner's schema check is the
//! contract between the two halves.

use std::path::Path;
use std::{env, fs, process};

use concat_host::AppDirs;
use concat_media::{AudioDecoder, AudioOptions, SampleFormat};
use concat_speech::transcribe::{TranscribeRequest, Transcriber};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: make_cache <media-file> <whisper-cache.json> [model-id] [language]");
        process::exit(2);
    }
    let media = &args[0];
    let cache_path = Path::new(&args[1]);
    let model_id = args.get(2).cloned().unwrap_or_else(|| "large-v3-turbo".to_owned());
    let language = args.get(3).cloned().unwrap_or_else(|| "id".to_owned());

    // The decode, exactly as `transcribe` asks for it: 16 kHz mono f32.
    let mut decoder = AudioDecoder::open(
        Path::new(media),
        &AudioOptions {
            start: None,
            duration: None,
            filters: Vec::new(),
            rate: 16_000,
            channels: 1,
            format: SampleFormat::F32,
            stream: None,
        },
    )
    .unwrap_or_else(|error| fail(media, &error.to_string()));
    let samples = decoder.collect_f32().unwrap_or_else(|error| fail(media, &error.to_string()));
    if samples.is_empty() {
        fail(media, "no audio decoded");
    }
    let duration_s = samples.len() as f64 / 16_000.0;
    let audio_path = cache_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("audio-16k.f32");
    let raw: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    fs::write(&audio_path, &raw).unwrap_or_else(|e| fail(&audio_path.display().to_string(), &e.to_string()));
    eprintln!("wrote {} ({} s of 16 kHz mono)", audio_path.display(), duration_s);

    let dirs = AppDirs::locate().unwrap_or_else(|error| fail("app dirs", &error));
    let transcriber = Transcriber::new();
    let request = TranscribeRequest {
        path: media.clone(),
        audio_stream: None,
        source_start: 0.0,
        window: duration_s,
        model_id,
        language,
        translate: false,
        prompt: String::new(),
        engine: "whispercpp".to_owned(),
        device: "auto".to_owned(),
        compute_type: "auto".to_owned(),
        vad_filter: true,
        beam_size: 5,
        word_timestamps: true,
        temperature_fallback: true,
        normalize_audio: false,
        keep_wav: false,
        output_dir: None,
    };
    let segments = transcriber
        .transcribe(&dirs, &request, |percent| {
            if percent % 10 == 0 {
                eprint!("\rwhisper {percent}%  ");
            }
        })
        .unwrap_or_else(|error| fail(media, &error));
    eprintln!();

    // Schema 1, exactly as the runner's `Cache` reads it: absolute seconds on
    // the file's own clock (source_start is 0, so window-relative is absolute
    // here), words in order, no marker - the cache is what Whisper said.
    let body = serde_json::json!({
        "schema": 1,
        "audio": {"file": media, "sample_rate": 16_000, "channels": 1,
                   "duration_s": duration_s},
        "segments": segments.iter().map(|segment| serde_json::json!({
            "start": segment.start,
            "end": segment.end,
            "text": segment.text,
            "words": segment.words.iter().map(|word| serde_json::json!({
                "text": word.text, "start": word.start, "end": word.end})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });
    if let Some(dir) = cache_path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    fs::write(cache_path, serde_json::to_string_pretty(&body).unwrap())
        .unwrap_or_else(|e| fail(&cache_path.display().to_string(), &e.to_string()));
    eprintln!(
        "wrote {} ({} segments, {} words)",
        cache_path.display(),
        segments.len(),
        segments.iter().map(|s| s.words.len()).sum::<usize>()
    );
}

fn fail(what: &str, why: &str) -> ! {
    eprintln!("{what}: {why}");
    process::exit(2);
}
