// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Whisper transcription: audio in, timed caption segments out.
//!
//! whisper.cpp runs in-process through `whisper-rs`, the way Kokoro runs
//! through sherpa-onnx: no binary to ship or find, and cancellation is a
//! callback returning `true`. On macOS the Metal build is used, so the
//! encoder runs on the GPU.
//!
//! Two pieces:
//!
//! 1. **Models.** Downloaded on demand into the selected shared folder (or
//!    the platform's app-data default) as plain `.bin` files, streamed with progress, written to a
//!    `.part` and renamed - a torn download must never look usable. A loaded
//!    model is cached until another is asked for, because loading one takes
//!    seconds.
//! 2. **Transcription.** The engine's audio decoder hands over the clip's
//!    window as 16 kHz mono floats - the input whisper wants - and the
//!    segments come back relative to the window.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use concat_host::{AppDirs, SingleFlight};
use concat_media::{AudioDecoder, AudioOptions, SampleFormat};
use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::DownloadProgress;

/// A downloadable Concat model or a local-only model supplied by another app.
/// Sizes are approximate, for progress bars when the server declines to send a Content-Length.
struct KnownModel {
    id: &'static str,
    label: &'static str,
    /// One line for the settings row: what this size trades away.
    blurb: &'static str,
    approx_bytes: u64,
    /// What the finished file must hash to; a download with nothing to
    /// check against is refused. Empty for local-only models. See [`concat_host::models`].
    sha256: &'static str,
    english_only: bool,
    /// Managed by Concat (download/remove) versus supplied by another app.
    managed: bool,
}

/// Models the settings panel can use, fastest first. Concat manages downloads
/// through `small`; the two large models are local-only because they can be
/// slow and memory-hungry, and are offered here to reuse VTS's existing files.
const KNOWN_MODELS: &[KnownModel] = &[
    KnownModel {
        id: "tiny.en",
        label: "Tiny (English)",
        blurb: "Fastest draft. Rough on names and punctuation.",
        approx_bytes: 77_704_715,
        sha256: "921e4cf8686fdd993dcd081a5da5b6c365bfde1162e72b08d75ac75289920b1f",
        english_only: true,
        managed: true,
    },
    KnownModel {
        id: "tiny",
        label: "Tiny (Multilingual)",
        blurb: "Fastest draft, any language.",
        approx_bytes: 77_691_713,
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        english_only: false,
        managed: true,
    },
    KnownModel {
        id: "base.en",
        label: "Base (English)",
        blurb: "The sweet spot: solid captions at ~10x realtime.",
        approx_bytes: 147_964_211,
        sha256: "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002",
        english_only: true,
        managed: true,
    },
    KnownModel {
        id: "base",
        label: "Base (Multilingual)",
        blurb: "Solid captions, any language.",
        approx_bytes: 147_951_465,
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        english_only: false,
        managed: true,
    },
    KnownModel {
        id: "small.en",
        label: "Small (English)",
        blurb: "Noticeably better wording; a few times slower.",
        approx_bytes: 487_614_201,
        sha256: "c6138d6d58ecc8322097e0f987c32f1be8bb0a18532a3f88f734d1bbf9c41e5d",
        english_only: true,
        managed: true,
    },
    KnownModel {
        id: "small",
        label: "Small (Multilingual)",
        blurb: "Best quality offered, any language.",
        approx_bytes: 487_601_967,
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        english_only: false,
        managed: true,
    },
    KnownModel {
        id: "large-v3-turbo",
        label: "Large v3 Turbo (Multilingual)",
        blurb: "Local VTS model only. Higher memory use; faster than Large v3.",
        approx_bytes: 1_620_000_000,
        sha256: "",
        english_only: false,
        managed: false,
    },
    KnownModel {
        id: "large-v3",
        label: "Large v3 (Multilingual)",
        blurb: "Local VTS model only. Highest quality; very high memory use.",
        approx_bytes: 3_100_000_000,
        sha256: "",
        english_only: false,
        managed: false,
    },
];

fn known(id: &str) -> Option<&'static KnownModel> {
    KNOWN_MODELS.iter().find(|model| model.id == id)
}

/// The file a model arrives as, and what it is called on the mirror.
fn model_archive(id: &str) -> String {
    format!("ggml-{id}.bin")
}

/// The whisper.cpp commit the table's digests were taken at: what the
/// fallback fetches is pinned to it, so `main` moving on changes nothing.
const UPSTREAM_COMMIT: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";

/// Where the mirror was filled from, and the second place a download tries.
fn model_upstream(id: &str) -> String {
    format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/{UPSTREAM_COMMIT}/ggml-{id}.bin")
}

/// Where downloaded models live: the selected shared folder, or the app-data
/// default when no override is configured.
fn models_dir(dirs: &AppDirs) -> Result<PathBuf, String> {
    let dir = dirs.whisper_models_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("could not create {}: {error}", dir.display()))?;
    Ok(dir)
}

/// whisper's abort callback: `user_data` is the run's cancel flag, and a
/// true here stops the computation where it stands. See `transcribe`.
unsafe extern "C" fn abort_when_set(user_data: *mut std::ffi::c_void) -> bool {
    // SAFETY: the caller passed `Arc::as_ptr` of an `AtomicBool` it keeps
    // alive for as long as whisper may call this.
    unsafe { (*(user_data as *const AtomicBool)).load(Ordering::Relaxed) }
}

fn model_file(dirs: &AppDirs, id: &str) -> Result<PathBuf, String> {
    // Ids come from our own table; anything else is a bug, not input.
    known(id).ok_or_else(|| format!("unknown model {id:?}"))?;
    Ok(models_dir(dirs)?.join(format!("ggml-{id}.bin")))
}

/// One model, as the settings panel shows it.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    /// The model id, e.g. "base.en".
    pub id: String,
    /// Display name.
    pub label: String,
    /// One line on what this size trades away.
    pub blurb: String,
    /// Whether the model only understands English.
    pub english_only: bool,
    /// Approximate download size in bytes, for display.
    pub size_bytes: u64,
    /// Whether the file is on disk.
    pub downloaded: bool,
    /// Whether Concat may download and remove the file, or it is externally managed.
    pub managed: bool,
}

/// What the settings panel shows.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TranscriberStatus {
    /// Where models are stored, so the settings panel can say so.
    pub models_dir: String,
    /// Every model the panel offers.
    pub models: Vec<ModelStatus>,
}

/// One word, and the moment whisper heard it: seconds from the window's start.
///
/// Phase 6b: these come from whisper's token timestamps, so they are heard
/// times, not estimates. The app turns them into caption word timings.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HeardWord {
    /// The word as whisper spelled it, punctuation included.
    pub text: String,
    /// Seconds from the window's start where the word begins.
    pub start: f64,
    /// Seconds from the window's start where it ends.
    pub end: f64,
}

/// One caption, in seconds relative to the transcribed window's start.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    /// Seconds from the window's start.
    pub start: f64,
    /// Seconds from the window's start.
    pub end: f64,
    /// What was said.
    pub text: String,
    /// Each word and when it was heard, in order. Empty when whisper gave
    /// no token timings for this segment; the caller then estimates.
    pub words: Vec<HeardWord>,
}

/// What to transcribe.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeRequest {
    /// The media file to transcribe.
    pub path: String,
    /// Which of the file's audio streams, by index; `None` is the first -
    /// the same choice the clip's playback makes, so the words come from
    /// the track that is heard.
    pub audio_stream: Option<u32>,
    /// Seconds into the file where the clip's source window begins.
    pub source_start: f64,
    /// How much source the clip covers, in seconds (`duration * speed`).
    pub window: f64,
    /// Which model to use, e.g. "base.en".
    pub model_id: String,
    /// ISO-639-1 Whisper language code, or "auto".
    #[serde(default = "default_language")]
    pub language: String,
    /// Ask multilingual Whisper models to translate speech to English.
    #[serde(default)]
    pub translate: bool,
    /// Optional names and jargon to bias decoding.
    #[serde(default)]
    pub prompt: String,
    /// Beam search width (`1..=10`; `1` uses greedy decoding).
    #[serde(default = "default_beam_size")]
    pub beam_size: u32,
    /// Whether token-level word timestamps should be computed.
    #[serde(default = "default_word_timestamps")]
    pub word_timestamps: bool,
    /// Retry hard segments at higher temperature (`0.2` step).
    #[serde(default)]
    pub temperature_fallback: bool,
    /// Apply FFmpeg `loudnorm` filter before transcription.
    #[serde(default)]
    pub normalize_audio: bool,
    /// Keep a copy of the decoded 16 kHz mono WAV file.
    #[serde(default)]
    pub keep_wav: bool,
    /// Optional output directory for saved artifacts (`keep_wav` / exported subtitles).
    #[serde(default)]
    pub output_dir: Option<String>,
}

fn default_language() -> String {
    "auto".to_owned()
}

fn default_beam_size() -> u32 {
    1
}

fn default_word_timestamps() -> bool {
    true
}

struct LoadedModel {
    id: String,
    /// The same model id can exist in two configured folders. Include its
    /// location in the cache key so changing the folder loads the right file.
    path: PathBuf,
    context: WhisperContext,
}

/// The transcriber: the one transcription and the one model download that
/// can run at a time, and the model kept loaded between runs.
pub struct Transcriber {
    gate: Arc<SingleFlight>,
    downloads: Arc<SingleFlight>,
    /// Held for the whole transcription, which is what lets
    /// [`Transcriber::delete_model`] refuse to delete a model mid-use.
    model: Mutex<Option<LoadedModel>>,
}

impl Default for Transcriber {
    fn default() -> Self {
        Self::new()
    }
}

impl Transcriber {
    /// An idle transcriber, loading nothing until asked to transcribe.
    pub fn new() -> Self {
        Self {
            gate: Arc::new(SingleFlight::new()),
            downloads: Arc::new(SingleFlight::new()),
            model: Mutex::new(None),
        }
    }

    /// Per-model state, for the settings panel.
    pub fn status(dirs: &AppDirs) -> Result<TranscriberStatus, String> {
        let dir = models_dir(dirs)?;
        Ok(TranscriberStatus {
            models_dir: dir.to_string_lossy().into_owned(),
            models: KNOWN_MODELS
                .iter()
                .map(|model| ModelStatus {
                    id: model.id.to_owned(),
                    label: model.label.to_owned(),
                    blurb: model.blurb.to_owned(),
                    english_only: model.english_only,
                    size_bytes: model.approx_bytes,
                    downloaded: dir.join(format!("ggml-{}.bin", model.id)).is_file(),
                    managed: model.managed,
                })
                .collect(),
        })
    }

    /// Streams one model from Concat's mirror into the models folder. Blocks
    /// for the whole download: run it on its own thread.
    ///
    /// Written to `<file>.part` and renamed at the end: a download killed
    /// halfway must never be mistaken for a model, because whisper would fail
    /// on it with a message that blames the model rather than the download.
    pub fn download_model(
        &self,
        dirs: &AppDirs,
        id: &str,
        mut progress: impl FnMut(DownloadProgress),
    ) -> Result<(), String> {
        let model = known(id).ok_or_else(|| format!("unknown model {id:?}"))?;
        if !model.managed {
            return Err(format!(
                "{} is a local-only model; use the existing file in the selected Whisper folder",
                model.label
            ));
        }
        if model.sha256.is_empty() {
            return Err(format!("no download digest is configured for {}", model.id));
        }
        let job = self.downloads.begin("model download")?;
        let destination = model_file(dirs, id)?;
        if destination.is_file() {
            return Ok(());
        }
        let estimate = model.approx_bytes;
        let partial = destination.with_extension("bin.part");
        let (received, total) = crate::fetch_model(
            &model_archive(id),
            &model_upstream(id),
            &partial,
            id,
            estimate,
            model.sha256,
            job.cancel_flag(),
            &mut progress,
        )?;
        std::fs::rename(&partial, &destination)
            .map_err(|error| format!("could not finish {}: {error}", destination.display()))?;
        progress(DownloadProgress {
            id: id.to_owned(),
            received,
            total,
            unpacking: false,
            done: true,
        });
        Ok(())
    }

    /// Asks the running download to stop. Idle is a harmless no-op.
    pub fn cancel_download(&self) {
        self.downloads.cancel();
    }

    /// Removes a downloaded model file, unless a transcription is using it.
    pub fn delete_model(&self, dirs: &AppDirs, id: &str) -> Result<(), String> {
        let model = known(id).ok_or_else(|| format!("unknown model {id:?}"))?;
        if !model.managed {
            return Err(format!(
                "{} is supplied by the shared Whisper folder and cannot be removed here",
                model.label
            ));
        }
        let file = model_file(dirs, id)?;
        let mut loaded = self.model.try_lock().map_err(|_| {
            "a transcription is running - wait for it or cancel it first".to_owned()
        })?;
        if loaded
            .as_ref()
            .is_some_and(|model| model.id == id && model.path == file)
        {
            *loaded = None;
        }
        std::fs::remove_file(&file)
            .map_err(|error| format!("could not remove {}: {error}", file.display()))
    }

    /// Transcribes one clip's audio window into caption segments. Blocks
    /// for the whole run: run it on its own thread. `progress` is called
    /// with a 0..100 percentage as whisper works.
    ///
    /// Timestamps come back relative to the window: 0 is `source_start`. The
    /// caller owns the clip, so the map onto the timeline (divide by speed,
    /// add the clip's start) happens there, next to the numbers it needs.
    /// Each segment's words come back on the same clock.
    pub fn transcribe(
        &self,
        dirs: &AppDirs,
        request: &TranscribeRequest,
        progress: impl FnMut(i32) + Send + 'static,
    ) -> Result<Vec<Segment>, String> {
        let job = self.gate.begin("transcription")?;
        let cancel = job.cancel_handle();

        let model_path = model_file(dirs, &request.model_id)?;
        if !model_path.is_file() {
            return Err(format!(
                "model {} is not downloaded - see Settings > Transcriber",
                request.model_id
            ));
        }
        // NaN is refused along with zero and negative windows.
        if request.window.is_nan() || request.window <= 0.0 {
            return Err("nothing to transcribe: the clip covers no time".to_owned());
        }

        // The window, as whisper wants it: 16 kHz mono floats.
        let mut decoder = AudioDecoder::open(
            &request.path,
            &AudioOptions {
                start: Some(request.source_start),
                duration: Some(request.window),
                filters: if request.normalize_audio {
                    vec!["loudnorm".to_owned()]
                } else {
                    Vec::new()
                },
                rate: 16_000,
                channels: 1,
                format: SampleFormat::F32,
                stream: request.audio_stream.map(|index| index as usize),
            },
        )
        .map_err(|error| error.to_string())?;
        let samples = decoder.collect_f32().map_err(|error| error.to_string())?;
        if samples.is_empty() {
            return Err("nothing to transcribe: the clip has no audio".to_owned());
        }
        if request.keep_wav {
            let _ = write_16khz_wav(&samples, &request.path, request.output_dir.as_deref());
        }
        if cancel.load(Ordering::Relaxed) {
            return Err("transcription cancelled".to_owned());
        }

        let mut loaded = self
            .model
            .lock()
            .map_err(|_| "transcriber state poisoned".to_owned())?;
        if !loaded
            .as_ref()
            .is_some_and(|model| model.id == request.model_id && model.path == model_path)
        {
            // Load before overwriting: a failed load keeps the old model.
            // The folder is part of the cache key: changing it must not keep
            // transcribing with the file loaded from the previous directory.
            let context =
                WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
                    .map_err(|error| format!("could not load the model: {error}"))?;
            *loaded = Some(LoadedModel {
                id: request.model_id.clone(),
                path: model_path.clone(),
                context,
            });
        }
        let context = &loaded.as_ref().expect("model loaded above").context;
        let mut state = context
            .create_state()
            .map_err(|error| format!("could not start whisper: {error}"))?;

        let threads = std::thread::available_parallelism()
            .map(|count| count.get().min(8))
            .unwrap_or(4);
        let strategy = if request.beam_size > 1 {
            SamplingStrategy::BeamSearch {
                beam_size: request.beam_size.clamp(1, 10) as i32,
                patience: -1.0,
            }
        } else {
            SamplingStrategy::Greedy { best_of: 1 }
        };
        let mut params = FullParams::new(strategy);
        params.set_n_threads(threads as i32);
        // The language is whisper's to hear, when it can hear more than
        // one: a multilingual model detects it from the first window. An
        // English-only model asked for "auto" still runs the detection,
        // over a vocabulary it was never trained on, and "hears" Thai at
        // one percent - so it is simply told English.
        let (language, translate, prompt) =
            effective_whisper_options(request, context.is_multilingual());
        params.set_language(Some(language));
        params.set_translate(translate);
        if let Some(prompt) = prompt {
            params.set_initial_prompt(prompt);
        }
        params.set_temperature_inc(if request.temperature_fallback {
            0.2
        } else {
            0.0
        });
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        // Phase 6b: a time for every token, which is what makes a word's own
        // start and end heard rather than estimated. Whisper computes them
        // as it goes; the text it returns is the same either way.
        params.set_token_timestamps(request.word_timestamps);
        // The raw hooks, not `set_abort_callback_safe`. whisper-rs 0.16's
        // safe one hands whisper a pointer to a boxed trait object with a
        // trampoline typed for the closure itself, so whisper reads the
        // box's two words as the closure and the answer is whatever byte
        // sits there: true, as a rule, and every encode was aborted before
        // it began - "failed to encode", error -6, on the first clip. The
        // flag itself is what whisper is given here, and it outlives the
        // call: `cancel` is held to the end of this function.
        // SAFETY: `abort_when_set` reads `user_data` as the `AtomicBool`
        // inside `cancel`, which lives until `full` has returned.
        unsafe {
            params.set_abort_callback(Some(abort_when_set));
            params.set_abort_callback_user_data(Arc::as_ptr(&cancel) as *mut std::ffi::c_void);
        }
        params.set_progress_callback_safe(progress);

        state
            .full(params, &samples)
            .map_err(|error| format!("transcription failed: {error}"))?;
        if cancel.load(Ordering::Relaxed) {
            return Err("transcription cancelled".to_owned());
        }

        // Whisper's special tokens - end of text, the language tag, the
        // timestamp markers - all have ids from end-of-text up. The text
        // leaves them out, and so must the words.
        let end_of_text = context.token_eot();

        Ok(state
            .as_iter()
            .filter_map(|segment| {
                // Whisper's timestamps are centiseconds.
                let start = segment.start_timestamp() as f64 / 100.0;
                let end = segment.end_timestamp() as f64 / 100.0;
                let text = segment.to_str_lossy().ok()?.trim().to_owned();
                let words = if request.word_timestamps {
                    // Each token's bytes and span. The bytes are kept raw: a
                    // token can hold part of a character, and only the joined
                    // bytes of a whole word decode.
                    let tokens: Vec<(Vec<u8>, f64, f64)> = (0..segment.n_tokens())
                        .filter_map(|index| segment.get_token(index))
                        .filter(|token| token.token_id() < end_of_text)
                        .filter_map(|token| {
                            let data = token.token_data();
                            Some((
                                token.to_bytes().ok()?.to_vec(),
                                data.t0 as f64 / 100.0,
                                data.t1 as f64 / 100.0,
                            ))
                        })
                        .collect();
                    words_from_tokens(
                        tokens
                            .iter()
                            .map(|(bytes, start, end)| (bytes.as_slice(), *start, *end)),
                    )
                } else {
                    Vec::new()
                };
                keep_segment(start, end, text, words)
            })
            .collect())
    }

    /// Asks the running transcription to stop at its next check.
    pub fn cancel(&self) {
        self.gate.cancel();
    }

    /// Whether a transcription is running.
    pub fn is_busy(&self) -> bool {
        self.gate.is_busy()
    }
}

/// Whisper marks silence and music with bracketed stage directions
/// ("[BLANK_AUDIO]", "(upbeat music)"). Those are not captions, and neither
/// is an empty or degenerate span.
fn keep_segment(start: f64, end: f64, text: String, words: Vec<HeardWord>) -> Option<Segment> {
    if text.is_empty() || (text.starts_with('[') && text.ends_with(']')) {
        return None;
    }
    (end > start).then_some(Segment {
        start,
        end,
        text,
        words,
    })
}

/// Groups whisper's tokens into words, the way whisper spells them.
///
/// Whisper writes a word's first token with a leading space and the rest of
/// its tokens without one, so a token that starts with whitespace begins a
/// new word and any other continues the one before - "the", " quick", " fox",
/// "." becomes "the", "quick", "fox.". Each word runs from its first token's
/// start to its last token's end, both straight from whisper.
///
/// Input is `(bytes, start, end)` per token, in seconds on the window's clock.
/// Bytes, not strings: some characters arrive split across tokens, and a
/// word is only decoded once its bytes are whole.
///
/// Languages written without spaces (Japanese, Chinese, Thai) get few or
/// no word breaks from this, so a run of them may come back as one long
/// word. That is honest - whisper did not say where the words break - and a
/// caption still highlights as one piece.
fn words_from_tokens<'a>(tokens: impl IntoIterator<Item = (&'a [u8], f64, f64)>) -> Vec<HeardWord> {
    let mut words: Vec<(Vec<u8>, f64, f64)> = Vec::new();
    for (bytes, start, end) in tokens {
        let begins_word = words.is_empty() || bytes.first().is_some_and(u8::is_ascii_whitespace);
        if begins_word {
            words.push((Vec::new(), start, end));
        }
        let word = words.last_mut().expect("a word was begun above");
        word.0.extend_from_slice(bytes);
        word.2 = end;
    }
    words
        .into_iter()
        .filter_map(|(bytes, start, end)| {
            let text = String::from_utf8_lossy(&bytes).trim().to_owned();
            (!text.is_empty()).then_some(HeardWord { text, start, end })
        })
        .collect()
}

/// Writes a 16 kHz mono 16-bit PCM WAV file next to `source_path` or inside `output_dir`.
fn write_16khz_wav(
    samples: &[f32],
    source_path: &str,
    output_dir: Option<&str>,
) -> Result<PathBuf, String> {
    let src = std::path::Path::new(source_path);
    let stem = src
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "audio".to_owned());
    let dir = output_dir
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| src.parent().map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    let dest = dir.join(format!("{stem}.16k.wav"));
    let data_bytes = (samples.len() * 2) as u32;
    let riff_size = 36u32.saturating_add(data_bytes);
    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&16_000u32.to_le_bytes());
    bytes.extend_from_slice(&32_000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm = (clamped * 32_767.0).round() as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }
    std::fs::write(&dest, bytes).map_err(|e| format!("could not write {}: {e}", dest.display()))?;
    Ok(dest)
}

/// Resolves language, translation, and initial prompt for `whisper.cpp`
/// according to whether the loaded model is multilingual.
fn effective_whisper_options(
    request: &TranscribeRequest,
    is_multilingual: bool,
) -> (&str, bool, Option<&str>) {
    let requested_language = request.language.trim();
    let language = if !is_multilingual {
        "en"
    } else if requested_language.is_empty() {
        "auto"
    } else {
        requested_language
    };
    let translate = request.translate && is_multilingual && request.model_id != "large-v3-turbo";
    let prompt = (!request.prompt.trim().is_empty()).then(|| request.prompt.trim());
    (language, translate, prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_whisper_options_normalizes_language_translate_and_prompt() {
        let req = TranscribeRequest {
            path: "sample.mp4".to_owned(),
            audio_stream: None,
            source_start: 0.0,
            window: 10.0,
            model_id: "base".to_owned(),
            language: "  id  ".to_owned(),
            translate: true,
            prompt: "  Concat, VTS  ".to_owned(),
            beam_size: 1,
            word_timestamps: true,
            temperature_fallback: false,
            normalize_audio: false,
            keep_wav: false,
            output_dir: None,
        };
        assert_eq!(
            effective_whisper_options(&req, true),
            ("id", true, Some("Concat, VTS"))
        );
        assert_eq!(
            effective_whisper_options(&req, false),
            ("en", false, Some("Concat, VTS"))
        );
        let turbo = TranscribeRequest {
            model_id: "large-v3-turbo".to_owned(),
            ..req.clone()
        };
        assert_eq!(
            effective_whisper_options(&turbo, true),
            ("id", false, Some("Concat, VTS"))
        );

        let blank = TranscribeRequest {
            language: "   ".to_owned(),
            translate: false,
            prompt: "   ".to_owned(),
            ..req
        };
        assert_eq!(
            effective_whisper_options(&blank, true),
            ("auto", false, None)
        );
    }

    #[test]
    fn stage_directions_and_degenerate_spans_are_not_captions() {
        assert_eq!(
            keep_segment(0.0, 2.5, " Hello there.".trim().to_owned(), vec![]).map(|s| s.text),
            Some("Hello there.".to_owned())
        );
        assert!(keep_segment(2.5, 2.5, "degenerate".to_owned(), vec![]).is_none());
        assert!(keep_segment(3.0, 4.0, "[BLANK_AUDIO]".to_owned(), vec![]).is_none());
        assert!(keep_segment(3.0, 4.0, String::new(), vec![]).is_none());
    }

    /// Words as whisper's tokens spell them: a leading space starts a word.
    #[test]
    fn a_token_with_a_leading_space_begins_a_word() {
        let tokens: [(&[u8], f64, f64); 4] = [
            (b" The", 0.00, 0.30),
            (b" quick", 0.30, 0.60),
            (b" fox", 0.60, 0.90),
            (b".", 0.90, 0.95),
        ];
        let words = words_from_tokens(tokens);
        let texts: Vec<&str> = words.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(
            texts,
            ["The", "quick", "fox."],
            "the period rides on its word"
        );
        assert_eq!(words[0].start, 0.00);
        assert_eq!(words[0].end, 0.30);
        assert_eq!(words[2].start, 0.60);
        assert_eq!(words[2].end, 0.95, "a word ends where its last token does");
    }

    /// A word whose tokens are split - a stem and a suffix - is one word,
    /// and runs from the first piece's start to the last piece's end.
    #[test]
    fn a_word_split_over_several_tokens_is_still_one_word() {
        let tokens: [(&[u8], f64, f64); 4] = [
            (b" under", 1.0, 1.2),
            (b"stand", 1.2, 1.5),
            (b"ing", 1.5, 1.9),
            (b" now", 1.9, 2.2),
        ];
        let words = words_from_tokens(tokens);
        assert_eq!(words.len(), 2, "{words:?}");
        assert_eq!(words[0].text, "understanding");
        assert_eq!((words[0].start, words[0].end), (1.0, 1.9));
    }

    /// A character split across two tokens is decoded whole, not as two
    /// replacement characters. "日" is E6 97 A5 in UTF-8.
    #[test]
    fn a_character_split_across_tokens_decodes_whole() {
        let bytes = "日".as_bytes();
        let tokens: [(&[u8], f64, f64); 2] = [(&bytes[..2], 0.0, 0.5), (&bytes[2..], 0.5, 1.0)];
        let words = words_from_tokens(tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].text, "日", "not replacement characters");
    }

    /// A whitespace-only token is not a word on its own; the next token that
    /// carries letters gives the word its text, and its start.
    #[test]
    fn whitespace_alone_makes_no_word() {
        let tokens: [(&[u8], f64, f64); 2] = [(b" ", 0.0, 0.1), (b"hi", 0.1, 0.4)];
        let words = words_from_tokens(tokens);
        assert_eq!(words.len(), 1, "{words:?}");
        assert_eq!(words[0].text, "hi");
        assert!(words_from_tokens([(b"  ".as_slice(), 0.0, 0.1)]).is_empty());
    }

    /// Without spaces, the words are not separable, and the honest answer is
    /// one word, not a guess at breaks.
    #[test]
    fn unspaced_text_is_one_word_not_a_guess() {
        let tokens: [(&[u8], f64, f64); 2] =
            [(b"\xE3\x81\x93", 0.0, 0.4), (b"\xE3\x82\x93", 0.4, 0.8)];
        let words = words_from_tokens(tokens);
        assert_eq!(words.len(), 1);
        assert_eq!((words[0].start, words[0].end), (0.0, 0.8));
    }

    #[test]
    fn no_tokens_make_no_words() {
        assert!(words_from_tokens(Vec::<(&[u8], f64, f64)>::new()).is_empty());
    }

    #[test]
    fn every_known_model_is_asked_of_the_mirror_before_upstream() {
        for model in KNOWN_MODELS.iter().filter(|model| model.managed) {
            let file = model_archive(model.id);
            assert_eq!(file, format!("ggml-{}.bin", model.id));
            assert!(model_upstream(model.id).ends_with(&file));
            let sources = concat_host::models::sources(&file, &model_upstream(model.id));
            let (mirror, upstream) = (&sources[0], &sources[1]);
            assert!(mirror.contains(concat_host::models::RELEASE));
            assert!(mirror.ends_with(&file));
            assert_eq!(*upstream, model_upstream(model.id));
            assert!(model.approx_bytes > 0);
        }
    }

    #[test]
    fn status_lists_every_model_undownloaded_in_an_empty_data_dir() {
        let scratch =
            std::env::temp_dir().join(format!("concat-whisper-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        let status = Transcriber::status(&AppDirs::under(&scratch)).expect("status");
        assert_eq!(status.models.len(), KNOWN_MODELS.len());
        assert!(status.models.iter().all(|model| !model.downloaded));
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn local_vts_models_cannot_be_downloaded_or_removed_by_concat() {
        let scratch = std::env::temp_dir().join(format!(
            "concat-whisper-local-only-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&scratch);
        let selected = scratch.join("shared").join("whisper-models");
        std::fs::create_dir_all(&selected).expect("shared model folder");
        let file = selected.join("ggml-large-v3-turbo.bin");
        std::fs::write(&file, b"model").expect("model file");
        let dirs = AppDirs {
            whisper_models: Some(selected),
            ..AppDirs::under(&scratch)
        };
        let transcriber = Transcriber::new();
        assert!(
            transcriber
                .download_model(&dirs, "large-v3-turbo", |_| {})
                .unwrap_err()
                .contains("local-only")
        );
        assert!(
            transcriber
                .delete_model(&dirs, "large-v3-turbo")
                .unwrap_err()
                .contains("cannot be removed")
        );
        assert!(file.is_file(), "the shared VTS model remains untouched");
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn status_uses_the_selected_shared_models_directory() {
        let scratch =
            std::env::temp_dir().join(format!("concat-whisper-shared-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        let selected = scratch.join("shared").join("whisper-models");
        std::fs::create_dir_all(&selected).expect("shared model folder");
        std::fs::write(selected.join("ggml-base.bin"), b"model").expect("model file");
        std::fs::write(selected.join("ggml-large-v3-turbo.bin"), b"model")
            .expect("turbo model file");
        std::fs::write(selected.join("ggml-large-v3.bin"), b"model").expect("large model file");
        let dirs = AppDirs {
            whisper_models: Some(selected.clone()),
            ..AppDirs::under(&scratch)
        };
        let status = Transcriber::status(&dirs).expect("status");
        assert_eq!(status.models_dir, selected.to_string_lossy().into_owned());
        assert!(
            status
                .models
                .iter()
                .any(|model| model.id == "base" && model.downloaded)
        );
        for id in ["large-v3-turbo", "large-v3"] {
            let model = status
                .models
                .iter()
                .find(|model| model.id == id)
                .expect("VTS model is listed");
            assert!(
                model.downloaded,
                "{id} should be detected in the shared folder"
            );
            assert!(
                !model.managed,
                "{id} is local-only and must not be removable"
            );
        }
        let _ = std::fs::remove_dir_all(&scratch);
    }
}

#[cfg(test)]
mod real_audio {
    use super::*;

    /// The whole path through whisper on a real file with a downloaded
    /// model - the thing the unit tests cannot cover, and what the abort
    /// hook broke. Ignored: it wants the app's model directory and a file
    /// named in CONCAT_TRANSCRIBE_AUDIO.
    ///
    ///   CONCAT_TRANSCRIBE_AUDIO=speech.wav cargo test -p concat-speech -- --ignored real_audio
    #[test]
    #[ignore = "needs a downloaded tiny.en and CONCAT_TRANSCRIBE_AUDIO"]
    fn transcribes_a_file_with_the_installed_model() {
        let Ok(audio) = std::env::var("CONCAT_TRANSCRIBE_AUDIO") else {
            eprintln!("CONCAT_TRANSCRIBE_AUDIO not set; nothing to transcribe");
            return;
        };
        let dirs = AppDirs::locate().expect("the app's directories");
        let transcriber = Transcriber::new();
        let request = TranscribeRequest {
            path: audio,
            audio_stream: None,
            source_start: 0.0,
            window: 30.0,
            model_id: "tiny.en".to_owned(),
            language: "auto".to_owned(),
            translate: false,
            prompt: String::new(),
            beam_size: 1,
            word_timestamps: true,
            temperature_fallback: false,
            normalize_audio: false,
            keep_wav: false,
            output_dir: None,
        };
        let segments = transcriber
            .transcribe(&dirs, &request, |percent| eprintln!("{percent}%"))
            .expect("transcription");
        for segment in &segments {
            eprintln!("{:.2}-{:.2} {}", segment.start, segment.end, segment.text);
            // Phase 6b: each word as whisper heard it, to check the timings
            // against the audio by ear.
            for word in &segment.words {
                eprintln!("    {:.2}-{:.2} {}", word.start, word.end, word.text);
            }
        }
        assert!(!segments.is_empty(), "no words heard");
    }
}
