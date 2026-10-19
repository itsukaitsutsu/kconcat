// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Creates a pre-onset-snap cache and its 16 kHz mono audio for T1/T2/T3.
//!
//! ```text
//! cargo run -p concat-speech --example make_cache -- \
//!     <media-file> <fresh-output-folder>/whisper-cache.json <model-id> [language]
//! ```
//!
//! The evaluation entry point keeps backend tokenization and the requested
//! VAD refinement, but bypasses onset snapping on every successful backend.
//! The ordinary editor transcription entry point is unchanged. Provenance
//! records the actual backend, requested settings and the pre-snap stage.
//! Existing output files are refused rather than overwriting a baseline.
//!
//! Native verification requires the owner's FFmpeg/Whisper setup. The pure
//! runner and mode/provenance tests run without those native libraries.

use std::io::Write;
use std::path::Path;
use std::{env, fs, process};

use concat_host::AppDirs;
use concat_media::{AudioDecoder, AudioOptions, SampleFormat};
use concat_speech::transcribe::{TranscribeRequest, Transcriber};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if !(3..=4).contains(&args.len()) {
        eprintln!("usage: make_cache <media-file> <cache.json> <model-id> [language]");
        process::exit(2);
    }
    let media = &args[0];
    let cache_path = Path::new(&args[1]);
    let model_id = args[2].clone();
    let language = args.get(3).cloned().unwrap_or_else(|| "id".to_owned());
    let output_dir = cache_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let audio_path = output_dir.join("audio-16k.f32");
    if cache_path.exists() || audio_path.exists() {
        fail(
            "output",
            "cache or audio already exists; choose a fresh output folder so the baseline is not overwritten",
        );
    }

    // Same continuous-clock decoding options as the transcription request:
    // no normalization, first stream, source start zero, 16 kHz mono f32.
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
    let samples = decoder
        .collect_f32()
        .unwrap_or_else(|error| fail(media, &error.to_string()));
    if samples.is_empty() {
        fail(media, "no audio decoded");
    }
    let duration_s = samples.len() as f64 / 16_000.0;

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
    let result = transcriber
        .transcribe_for_timing_cache(&dirs, &request, |percent| {
            if percent % 10 == 0 {
                eprint!("\rwhisper {percent}%  ");
            }
        })
        .unwrap_or_else(|error| fail(media, &error));
    eprintln!();
    let segments = &result.segments;
    let body = serde_json::json!({
        "schema": 1,
        "provenance": {
            "stage": "post-tokenization-and-requested-vad-before-onset-snap",
            "onset_snapping": false,
            "backend": result.backend,
            "requested_backend": request.engine,
            "model_id": request.model_id,
            "language": request.language,
            "translate": request.translate,
            "prompt": request.prompt,
            "device": request.device,
            "compute_type": request.compute_type,
            "vad_filter": request.vad_filter,
            "beam_size": request.beam_size,
            "word_timestamps": request.word_timestamps,
            "temperature_fallback": request.temperature_fallback,
            "normalize_audio": request.normalize_audio,
            "source_start": request.source_start,
            "window_s": request.window,
            "clock": "continuous source window; source_start=0; no speed mapping",
            "generator_version": env!("CARGO_PKG_VERSION")
        },
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
    let json =
        serde_json::to_vec_pretty(&body).unwrap_or_else(|e| fail("cache JSON", &e.to_string()));
    let raw: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    fs::create_dir_all(output_dir).unwrap_or_else(|e| fail("output directory", &e.to_string()));
    // Delay all output writing until transcription succeeds. create_new also
    // closes the race with another process creating the same output paths.
    write_new(&audio_path, &raw)
        .unwrap_or_else(|e| fail(&audio_path.display().to_string(), &e.to_string()));
    write_new(cache_path, &json)
        .unwrap_or_else(|e| fail(&cache_path.display().to_string(), &e.to_string()));
    eprintln!(
        "wrote {} ({duration_s} s, 16 kHz mono)",
        audio_path.display()
    );
    eprintln!(
        "wrote {} ({} segments, {} words; backend {}; onset snapping OFF)",
        cache_path.display(),
        segments.len(),
        segments.iter().map(|s| s.words.len()).sum::<usize>(),
        result.backend
    );
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)
}

fn fail(what: &str, why: &str) -> ! {
    eprintln!("{what}: {why}");
    process::exit(2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_existing_baseline_output_is_never_overwritten() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("kconcat-baseline-{}-{stamp}.json", process::id()));
        write_new(&path, b"original").unwrap();
        let err = write_new(&path, b"replacement").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&path).unwrap(), b"original");
        fs::remove_file(path).unwrap();
    }
}
