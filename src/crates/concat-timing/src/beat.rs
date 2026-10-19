// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Local, model-free beat-grid estimation for music edits.
//!
//! This is an intentionally conservative first version: it estimates a steady
//! grid from positive energy changes, suitable for snapping a cut/start. It is
//! not a musical transcription system and should not be used as a metronome
//! for expressive or heavily syncopated music.

use crate::energy::{energy_frames, HOP_SAMPLES};
use crate::SAMPLE_RATE;

const MIN_BPM: f64 = 60.0;
const MAX_BPM: f64 = 180.0;

/// A regular beat grid inferred from one mono f32 waveform.
#[derive(Clone, Debug, PartialEq)]
pub struct BeatGrid {
    /// Estimated tempo, clamped to the supported 60–180 BPM search range.
    pub bpm: f64,
    /// Beat times in seconds, relative to the start of the supplied waveform.
    pub beats: Vec<f64>,
    /// Normalized autocorrelation score in 0..=1; a weak score means the
    /// material did not produce a reliable periodic grid.
    pub confidence: f64,
}

/// Estimate a beat grid from mono audio sampled at [`SAMPLE_RATE`].
///
/// Returns `None` for too-short, silent, or non-periodic input. The 20 ms
/// energy windows and 5 ms hop are shared with the word-timing analyzer.
pub fn detect_beats(samples: &[f32]) -> Option<BeatGrid> {
    let energy = energy_frames(samples);
    if energy.len() < 400 {
        return None;
    }

    // Positive spectral/energy flux: sustained level is not itself a beat.
    let mut onset: Vec<f64> = energy.windows(2).map(|w| (w[1] - w[0]).max(0.0)).collect();
    if onset.iter().all(|v| *v <= f64::EPSILON) {
        return None;
    }
    let mean = onset.iter().sum::<f64>() / onset.len() as f64;
    for value in &mut onset {
        *value = (*value - mean).max(0.0);
    }
    let norm = onset.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm <= f64::EPSILON {
        return None;
    }

    let hop_seconds = HOP_SAMPLES as f64 / f64::from(SAMPLE_RATE);
    let min_lag = (60.0 / MAX_BPM / hop_seconds).round() as usize;
    let max_lag = (60.0 / MIN_BPM / hop_seconds).round() as usize;
    let mut scores = Vec::new();
    for lag in min_lag..=max_lag.min(onset.len() / 2) {
        let score = onset
            .iter()
            .zip(onset.iter().skip(lag))
            .map(|(a, b)| a * b)
            .sum::<f64>();
        scores.push((lag, score));
    }
    let (lag, peak) = scores.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1))?;
    let baseline = scores.iter().map(|(_, score)| *score).sum::<f64>() / scores.len() as f64;
    let confidence = if peak <= f64::EPSILON { 0.0 } else { ((peak - baseline) / peak).clamp(0.0, 1.0) };
    if confidence < 0.08 {
        return None;
    }

    let period = lag as f64 * hop_seconds;
    // Choose the phase with the most onset energy; this avoids assuming the
    // first beat occurs at t=0 when the file begins with a pickup or silence.
    let phase_steps = lag.min(100);
    let phase = (0..phase_steps)
        .max_by(|a, b| {
            phase_score(&onset, lag, *a).total_cmp(&phase_score(&onset, lag, *b))
        })?;
    let first = phase as f64 * hop_seconds;
    let duration = samples.len() as f64 / f64::from(SAMPLE_RATE);
    let beats = std::iter::successors(Some(first), |time| Some(time + period))
        .take_while(|time| *time < duration)
        .collect::<Vec<_>>();
    (beats.len() >= 2).then_some(BeatGrid {
        bpm: 60.0 / period,
        beats,
        confidence,
    })
}

fn phase_score(onset: &[f64], lag: usize, phase: usize) -> f64 {
    (phase..onset.len()).step_by(lag).map(|i| onset[i]).sum()
}

/// Return the closest beat within `tolerance_seconds`; otherwise retain `time`.
pub fn snap_to_beat(time: f64, grid: &BeatGrid, tolerance_seconds: f64) -> f64 {
    if !time.is_finite() || !tolerance_seconds.is_finite() || tolerance_seconds < 0.0 {
        return time;
    }
    let Some(beat) = grid.beats.iter().copied().min_by(|a, b| {
        (a - time).abs().total_cmp(&(b - time).abs())
    }) else {
        return time;
    };
    if (beat - time).abs() <= tolerance_seconds { beat } else { time }
}

#[cfg(test)]
mod tests {
    use super::{detect_beats, snap_to_beat};
    use crate::SAMPLE_RATE;

    fn pulse_train(bpm: f64, seconds: f64) -> Vec<f32> {
        let mut audio = vec![0.0; (f64::from(SAMPLE_RATE) * seconds) as usize];
        let period = (f64::from(SAMPLE_RATE) * 60.0 / bpm) as usize;
        for start in (0..audio.len()).step_by(period) {
            let end = (start + 900).min(audio.len());
            for (offset, sample) in audio[start..end].iter_mut().enumerate() {
                let envelope = 1.0 - offset as f32 / 900.0;
                *sample = envelope * (offset as f32 * 0.19).sin() * 0.8;
            }
        }
        audio
    }

    #[test]
    fn finds_regular_120_bpm_pulses_within_tempo_tolerance() {
        let grid = detect_beats(&pulse_train(120.0, 12.0)).expect("periodic music");
        assert!((grid.bpm - 120.0).abs() < 3.0, "{} BPM", grid.bpm);
        assert!(grid.confidence > 0.08);
    }

    #[test]
    fn snaps_only_inside_the_requested_window() {
        let grid = detect_beats(&pulse_train(120.0, 12.0)).expect("periodic music");
        let beat = grid.beats[4];
        assert!((snap_to_beat(beat + 0.03, &grid, 0.05) - beat).abs() < 1e-9);
        assert!((snap_to_beat(beat + 0.2, &grid, 0.05) - (beat + 0.2)).abs() < 1e-9);
    }

    #[test]
    fn silent_audio_has_no_beat_grid() {
        assert!(detect_beats(&vec![0.0; SAMPLE_RATE as usize * 3]).is_none());
    }
}
