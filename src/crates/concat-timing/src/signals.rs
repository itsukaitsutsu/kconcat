// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Level 1.5: the two signals cheaper than a model (roadmap T3,
//! `03-manuals/A-word-timing-level1.md` §3).
//!
//! - **Zero-crossing rate.** Fricatives and sibilants (*s, sh, f, t*) raise
//!   ZCR sharply even when energy barely moves - the word starts the energy
//!   snapper misses, and they are common in English and Indonesian.
//! - **Spectral flux**, as four Goertzel band energies: the sum of positive
//!   band-magnitude changes between frames. Finds consonant→vowel transitions
//!   mid-phrase, where there is no energy dip. Goertzel, not an FFT: about
//!   thirty lines and no dependency, which is the manual's whole point.
//!
//! All three signals (energy included) share the frame grid of
//! [`crate::energy`]: frame `i` is the window `[i*hop, i*hop+win)`, so a rise
//! found by any of them means the same moment in the same buffer, and the
//! snapper's "first rise in the search window" stays one scan.
//!
//! The band edges below are the manual's *starting guess*, recorded here as
//! the implemented set per T3; the thresholds beside them are starting
//! guesses in the same spirit - numbers T8 may tune, each change recorded.

use crate::SAMPLE_RATE;
use crate::energy::{HOP_SAMPLES, WIN_SAMPLES};

/// The four Goertzel bands, in Hz: the manual §3 starting guess, selected
/// for T3's implementation. Retuning is recorded in the manual, not here.
pub const BANDS: [(f64, f64); 4] = [
    (300.0, 800.0),
    (800.0, 1500.0),
    (1500.0, 3000.0),
    (3000.0, 6000.0),
];

/// Probes per band: Goertzel measures single frequencies, so a band's
/// magnitude is the mean of a few geometrically spaced probes inside it.
/// Four probes over a 2-3x-wide band, on a 20 ms window, is the manual's
/// "about 30 lines, no dependency" budget.
const PROBES_PER_BAND: usize = 4;

/// The Level 1.5 tuning, beside [`crate::snap::Params`] and inside it: every
/// value a starting guess for T8 to tune on real corpus runs, each change
/// recorded per T8 step 3. None of these is in the manual's Level 1 table -
/// that table is Level 1's, and these knobs only exist because §3 adds two
/// signals the table never covered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wideband {
    /// How far above the local ZCR floor a frame must sit to be a fricative
    /// onset, in crossings per sample. A voiced frame runs near 0.05-0.15,
    /// a fricative near 0.4-0.6, so 0.10 above a quiet floor is conservative.
    pub zcr_margin: f64,
    /// A flux frame is a rise when it reaches this multiple of the local
    /// median flux - the same percentile-floor idea as energy, at the 50th.
    pub flux_multiple: f64,
    /// Absolute floor under the multiple, in band-magnitude units (0..1-ish
    /// per probe mean): in near-silence the median flux is near zero and any
    /// jitter would double past it, so a rise must also clear this. Start:
    /// 0.0005, chosen from measurement, not scale-guessing - the synthetic
    /// equal-loudness transition (0.05-amplitude tones, this crate's test)
    /// peaks at about 0.0016, and steady-state jitter is about 1e-15, so the
    /// floor sits ~3x under a real transition and ~14 orders above the
    /// numerical noise. Flux scales with amplitude, so louder speech clears
    /// it further; room-noise rooms are the median multiple's job, not this.
    /// Burst offsets spike higher than this floor - the energy gate on
    /// [`is_flux_rise`] is what refuses them, not this number.
    pub flux_min: f64,
    /// The Goertzel bands. [`BANDS`] by default.
    pub bands: [(f64, f64); 4],
}

impl Default for Wideband {
    fn default() -> Self {
        Self {
            zcr_margin: 0.10,
            flux_multiple: 2.0,
            flux_min: 0.0005,
            bands: BANDS,
        }
    }
}

/// The buffer, framed once into every signal the snapper and Tier 1 read.
///
/// One struct so the three frame arrays can never drift apart: same buffer,
/// same grid, same length - a rise at frame `i` means the same 5 ms in all
/// three. Build it once per window ([`Signals::from_samples`]), not per
/// segment: the clock rule in [`crate::snap`] demands the continuous buffer,
/// and so does the cost model (Goertzel is O(n) but not free).
#[derive(Clone, Debug)]
pub struct Signals {
    /// Energy in dB, [`crate::energy::energy_frames`]: Level 1's signal.
    pub energy: Vec<f64>,
    /// Zero-crossings per sample, per frame, 0..1.
    pub zcr: Vec<f64>,
    /// Spectral flux per frame: the sum of positive band-magnitude changes
    /// against the previous frame. Frame 0 has no previous and reads 0.
    pub flux: Vec<f64>,
}

impl Signals {
    /// Frames the whole buffer with the default [`BANDS`].
    pub fn from_samples(samples: &[f32]) -> Signals {
        Signals::with_bands(samples, &BANDS)
    }

    /// Frames the whole buffer with the bands a tuning run picked.
    pub fn with_bands(samples: &[f32], bands: &[(f64, f64); 4]) -> Signals {
        Signals {
            energy: crate::energy::energy_frames(samples),
            zcr: zcr_frames(samples),
            flux: flux_frames(samples, bands),
        }
    }

    /// How many frames every signal holds - they are all the same, and that
    /// is the invariant this struct exists to keep.
    pub fn len(&self) -> usize {
        self.energy.len()
    }

    /// Whether the buffer was too short to frame at all.
    pub fn is_empty(&self) -> bool {
        self.energy.is_empty()
    }
}

/// How many frames the shared grid yields for this buffer.
fn frame_count(len: usize) -> usize {
    len.saturating_sub(WIN_SAMPLES) / HOP_SAMPLES + usize::from(len >= WIN_SAMPLES)
}

/// The zero-crossing rate per frame: sign changes in the window over the
/// number of sample pairs. A fricative runs near 0.5; voiced speech and
/// silence stay an order of magnitude below.
pub fn zcr_frames(samples: &[f32]) -> Vec<f64> {
    (0..frame_count(samples.len()))
        .map(|i| {
            let window = &samples[i * HOP_SAMPLES..i * HOP_SAMPLES + WIN_SAMPLES];
            let crossings = window
                .windows(2)
                .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
                .count();
            crossings as f64 / (WIN_SAMPLES - 1) as f64
        })
        .collect()
}

/// One Goertzel magnitude: the strength of `freq` in `window`, normalised to
/// the 0..1-ish scale of the samples themselves (a full-scale sine at exactly
/// `freq` reads near 0.5).
///
/// The sliding form is not needed - each window is measured once - so this is
/// the standard second-order recurrence, `power = s1² + s2² - coeff·s1·s2`.
pub fn goertzel_magnitude(window: &[f32], freq: f64) -> f64 {
    let n = window.len();
    if n == 0 {
        return 0.0;
    }
    let omega = 2.0 * std::f64::consts::PI * freq / f64::from(SAMPLE_RATE);
    let coeff = 2.0 * omega.cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for &sample in window {
        let s0 = f64::from(sample) + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    let power = s1 * s1 + s2 * s2 - coeff * s1 * s2;
    power.max(0.0).sqrt() / n as f64
}

/// The magnitude of one band: the mean of [`PROBES_PER_BAND`] geometrically
/// spaced Goertzel probes inside it. A mean, not a max: flux sums changes
/// across bands, and a max would let one probe's leakage speak for the band.
pub fn band_magnitude(window: &[f32], band: (f64, f64)) -> f64 {
    let (lo, hi) = band;
    let ratio = hi / lo;
    let sum: f64 = (0..PROBES_PER_BAND)
        .map(|p| {
            let f = lo * ratio.powf((p as f64 + 0.5) / PROBES_PER_BAND as f64);
            goertzel_magnitude(window, f)
        })
        .sum();
    sum / PROBES_PER_BAND as f64
}

/// Spectral flux per frame: `Σ_b max(0, m[i][b] - m[i-1][b])`. Only rises
/// count - a band losing energy is not an onset, it is the offset the manual
/// warns about catching by mistake.
pub fn flux_frames(samples: &[f32], bands: &[(f64, f64); 4]) -> Vec<f64> {
    let count = frame_count(samples.len());
    let magnitudes: Vec<[f64; 4]> = (0..count)
        .map(|i| {
            let window = &samples[i * HOP_SAMPLES..i * HOP_SAMPLES + WIN_SAMPLES];
            [
                band_magnitude(window, bands[0]),
                band_magnitude(window, bands[1]),
                band_magnitude(window, bands[2]),
                band_magnitude(window, bands[3]),
            ]
        })
        .collect();
    let mut flux = vec![0.0f64; count];
    for i in 1..count {
        flux[i] = (0..4)
            .map(|b| (magnitudes[i][b] - magnitudes[i - 1][b]).max(0.0))
            .sum();
    }
    flux
}

/// Whether a fricative onset begins at frame `index`: ZCR at least `margin`
/// above the local floor, the frame before below it (the rise starts here),
/// and the frame after still up (the fricative sustains - the same debounce
/// idea as energy's, at 2 frames).
pub fn is_zcr_rise(zcr: &[f64], index: usize, floor: f64, margin: f64) -> bool {
    let debounce = margin - 0.02;
    index > 0
        && index + 1 < zcr.len()
        && zcr[index] - floor >= margin
        && zcr[index - 1] < floor + margin
        && zcr[index + 1] >= floor + debounce
}

/// Whether a spectral-flux rise begins at frame `index`: flux at least
/// `median * flux_multiple` (and at least `flux_min`), the frame before
/// below the threshold, and - the offset gate - **energy not falling**.
///
/// The gate is a measured fix, not a preference (T3): sliding a window off
/// the end of a loud burst makes the band magnitudes oscillate as the step
/// edge crosses the window (Dirichlet sidelobes of the signal's own sharp
/// cut - a Hann taper does not remove it, because the step is in the signal,
/// not the window). Raw flux therefore spikes at burst *offsets*: on the
/// synthetic corpus, words after a gap snapped 130 ms early, into the
/// previous burst's tail. An offset is a decay, not the consonant→vowel
/// transition flux exists to find, and a transition at equal loudness has
/// flat energy while an offset has falling energy - so `e[i] >= e[i-1]`
/// separates them with no new number to tune.
pub fn is_flux_rise(
    flux: &[f64],
    energy: &[f64],
    index: usize,
    median: f64,
    wide: &Wideband,
) -> bool {
    let threshold = (median * wide.flux_multiple).max(wide.flux_min);
    index > 0
        && index < flux.len()
        && index < energy.len()
        && flux[index] >= threshold
        && flux[index - 1] < threshold
        && energy[index] >= energy[index - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::energy_frames;
    use crate::metrics::{is_rise, local_floor_pct};
    use crate::snap::Params;

    /// A deterministic pseudo-noise sign: fricatives are noise, and a test
    /// must not depend on a rng seed drifting between runs.
    fn noise_sign(i: usize) -> f32 {
        if (i.wrapping_mul(2_654_435_761) >> 16) & 1 == 0 {
            1.0
        } else {
            -1.0
        }
    }

    /// Quiet (-60 dB) audio, with a fricative burst: noise at an amplitude
    /// chosen so the *energy* rise rule provably cannot see it (full-window
    /// rms is -54 dB, under the floor + 8 dB threshold), while ZCR jumps
    /// from 0 to about 0.5.
    fn fricative_fixture() -> Vec<f32> {
        let n = SAMPLE_RATE as usize; // 1.0 s
        let mut audio = vec![0.001f32; n];
        let from = n / 2;
        let to = from + n * 3 / 10; // 0.5..0.8 s
        for (i, sample) in audio.iter_mut().enumerate().take(to).skip(from) {
            *sample = 0.002 * noise_sign(i);
        }
        audio
    }

    /// A steady sine, exactly an integer number of periods per hop, so the
    /// sliding window sees the same magnitude every frame.
    fn tone(seconds: f64, freq: f64, amplitude: f32) -> Vec<f32> {
        let n = (seconds * SAMPLE_RATE as f64) as usize;
        (0..n)
            .map(|i| {
                amplitude
                    * (2.0 * std::f64::consts::PI * freq * i as f64 / SAMPLE_RATE as f64).sin()
                        as f32
            })
            .collect()
    }

    // Prescribed test: on a pure tone, flux stays near zero.
    #[test]
    fn on_a_pure_tone_flux_stays_near_zero() {
        let flux = flux_frames(&tone(1.0, 1000.0, 0.05), &BANDS);
        let steady = &flux[8..flux.len() - 8];
        let max = steady.iter().fold(0.0f64, |m, f| m.max(*f));
        assert!(
            max < 1e-3,
            "a steady tone must not look like a transition: max flux {max}"
        );
        // And far below the absolute floor a rise must clear anyway.
        assert!(max < Wideband::default().flux_min / 2.0);
    }

    // The fricative is invisible to energy - this is the gap ZCR exists for.
    #[test]
    fn a_fricative_raises_zcr_while_energy_stays_blind() {
        let audio = fricative_fixture();
        let zcr = zcr_frames(&audio);
        let energy = energy_frames(&audio);

        // ZCR jumps: silence ~0, fricative ~0.5.
        let mid = zcr[SAMPLE_RATE as usize / 2 / HOP_SAMPLES + 10];
        assert!(mid > 0.3, "fricative zcr should be high, was {mid}");
        let before = zcr[10];
        assert!(before < 0.05, "silence zcr should be ~0, was {before}");

        // Energy cannot see it: no rise anywhere in the fricative, at the
        // local floor the snapper itself would use.
        let params = Params::default();
        let floor = local_floor_pct(&energy, 0.65, params.floor_percentile).unwrap();
        let from = crate::energy::frame_at(0.5, energy.len());
        let to = crate::energy::frame_at(0.8, energy.len());
        assert!(
            !(from..to).any(|i| is_rise(&energy, i, floor, params.rise_db, params.debounce_frames)),
            "the fricative must stay invisible to energy for this fixture to mean anything"
        );

        // And the ZCR detector sees it: a rise at the fricative's first frames.
        let z_floor = local_floor_pct(&zcr, 0.5, params.floor_percentile).unwrap_or(0.0);
        let wide = Wideband::default();
        let first = crate::energy::frame_at(0.5, zcr.len());
        let found = (first.saturating_sub(4)..first + 8)
            .any(|i| is_zcr_rise(&zcr, i, z_floor, wide.zcr_margin));
        assert!(
            found,
            "ZCR must catch the fricative onset energy cannot see"
        );
    }

    // The regression test for the offset artifact the energy gate exists
    // for: raw flux spikes where a burst ENDS, and the gated detector must
    // refuse every one of those frames.
    #[test]
    fn a_burst_offset_is_not_a_flux_rise() {
        let mut audio = vec![0.001f32; SAMPLE_RATE as usize];
        audio[3_200..8_000].fill(0.1); // loud 0.2..0.5 s, then the offset
        let flux = flux_frames(&audio, &BANDS);
        let energy = energy_frames(&audio);
        let wide = Wideband::default();
        let offset = 8_000 / HOP_SAMPLES; // frame whose window slides off
        let median = local_floor_pct(&flux, 0.55, 50).unwrap_or(0.0);

        // The fixture really does produce the artifact: raw flux over the
        // offset frames clears the absolute floor.
        assert!(
            (offset - 4..offset + 2).any(|i| flux[i] >= wide.flux_min),
            "fixture must reproduce the offset artifact this gate exists for"
        );
        // And the gated detector refuses all of it: energy is falling there.
        assert!(
            !(offset - 6..offset + 4).any(|i| is_flux_rise(&flux, &energy, i, median, &wide)),
            "an offset is a decay, not a transition"
        );
    }

    // The mid-phrase transition: same loudness on both sides, so energy is
    // flat and no rise fires - flux is the only signal that sees it.
    #[test]
    fn a_mid_phrase_transition_raises_flux_without_an_energy_rise() {
        let mut audio = tone(0.5, 1000.0, 0.05);
        audio.extend(tone(0.5, 4000.0, 0.05));
        let energy = energy_frames(&audio);
        let flux = flux_frames(&audio, &BANDS);

        // Energy is flat: the same amplitude on both sides of 0.5 s.
        let params = Params::default();
        let floor = local_floor_pct(&energy, 0.5, params.floor_percentile).unwrap();
        let from = crate::energy::frame_at(0.45, energy.len());
        let to = crate::energy::frame_at(0.55, energy.len());
        assert!(
            !(from..to).any(|i| is_rise(&energy, i, floor, params.rise_db, params.debounce_frames)),
            "a C→V transition at equal loudness must be invisible to energy"
        );

        // Flux spikes at the transition, and the detector fires there.
        let median = local_floor_pct(&flux, 0.5, 50).unwrap_or(0.0);
        let wide = Wideband::default();
        let found = (from..to).any(|i| is_flux_rise(&flux, &energy, i, median, &wide));
        assert!(found, "flux must catch the transition energy cannot see");
    }
}
