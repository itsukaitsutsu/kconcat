// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Word timing, and the arithmetic that decides whether a timing change helped.
//!
//! This crate exists because of decision D4-B: the snapper (roadmap T2/T3) and
//! the energy helpers it shares with silence detection live here, in a crate
//! with **no dependencies**, so its tests run in a sandbox with nothing
//! installed - no FFmpeg, no GPU, no model. `concat-speech` maps its
//! `HeardWord` to and from the plain [`Word`] here and calls the snapper after
//! `refine_segments_with_vad`; `concat-media` depends on this crate for
//! [`energy::rms`] and [`energy::linear_from_db`] so there is one copy of each.
//!
//! Three layers, in dependency order:
//!
//! 1. [`energy`] - framing the waveform once into decibel frames, plus the two
//!    helpers moved out of `concat-media/src/silence.rs`.
//! 2. [`signals`] - the Level 1.5 signals on the same frame grid (`03-manuals/
//!    A-word-timing-level1.md` §3): zero-crossing rate and four Goertzel bands
//!    of spectral flux, bundled with energy as [`Signals`].
//! 3. [`snap`] - the Level 1 onset snapper (manual §2), searching all three
//!    signals at once.
//! 4. [`metrics`] - the Tier 1 table and WER: measurements that need no
//!    labels, and the one that needs a corrected transcript. The example
//!    binary `metric_runner` reads `06-corpus/whisper-cache.json` and
//!    `audio-16k.f32` and prints both in seconds (D5-B).
//!
//! Nothing here knows about files, FFmpeg, or the project model. A word is a
//! [`Word`]: text, a span, and where the span came from.

pub mod beat;
pub mod energy;
pub mod metrics;
pub mod signals;
pub mod snap;
pub mod words;

pub use signals::Signals;
pub use words::{Source, Word};

/// The sample rate every timing in this crate is measured against.
///
/// The corpus audio is decoded once to 16 kHz mono little-endian f32
/// (`06-corpus/audio-16k.f32`), and Whisper's own window works at 16 kHz, so
/// one rate serves both and no resampling ever sits between a word and the
/// samples it is measured on.
pub const SAMPLE_RATE: u32 = 16_000;
