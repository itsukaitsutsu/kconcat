# A. Word timing — the cap, and Level 1 in full

*Option A: go deep on word onset snapping. Facts below are from your tree at `1aece2d`.*

---

## 1. What the cap actually is

Four different accuracies get confused, so separate them:

| Stage | Typical error | What sets the limit |
|---|---|---|
| Whisper **segment** timestamps | ±1000 ms | 30-second windows, attention over the whole chunk |
| Whisper **token/word** timestamps (what you use) | ±200–500 ms | Attention peaks, then **quantised to centiseconds** (`transcribe.rs:656`), then forced to move monotonically inside a segment |
| **Energy-onset snapping** (Level 1) | ±80–120 ms on words after a gap; **no gain mid-phrase** | Physics: you can only find an onset where the waveform has one |
| **Phoneme forced alignment** (Level 2) | ±50–100 ms everywhere | A model that knows what phonemes sound like |

Two consequences worth internalising:

- **Your error is not random.** Whisper's word times are constrained to be monotonic inside a segment, so one bad early word drags every later word in that cue. That's why it looks like "the whole caption slides", not "one word is off".
- **Level 1 has a hard ceiling and it is not the implementation.** Energy snapping can only succeed where there is an acoustic discontinuity — a pause, a breath, a plosive release. In the middle of a fluent phrase, words run together with no energy dip at all. There is nothing to snap to. No amount of tuning changes that.

**So expect this shape of result, not a uniform improvement:**

```
word after a pause / breath      →  big win   (±300 ms → ±60 ms)
word after a plosive (p,t,k,b)   →  win       (±250 ms → ±80 ms)
word mid-phrase, vowel-to-vowel  →  ~no change (±250 ms → ±230 ms)
```

If you measure "average MAE" and it improves 40%, that is the *right* answer, and it already fixes most of what you perceive as drift — because the words you notice being late are usually the ones after a pause or a cut.

---

## 2. Level 1 — the algorithm, concretely

Everything happens after `refine_segments_with_vad` (`transcribe.rs:1068`), on the **same continuous `samples: &[f32]` buffer** at 16 kHz mono. Two rules that matter:

- Use the continuous media clock. Your Stage-1 change deliberately made VAD post-transcription to stop splice drift — the snapping must sit on that same clock. Never snap against a spliced/compacted buffer.
- Snap **before** the caller divides by speed (`transcribe.rs:452` — the caller maps the window onto the timeline). Snapping must see the true time axis of the audio.

### Step 1 — frame the energy once, for the whole buffer

```
hop   = 80 samples  (5 ms)
win   = 320 samples (20 ms)   ← your VAD already uses CHUNK = 320
frames = (samples.len() - win) / hop + 1
e[i]  = 10 * log10( rms²(samples[i*hop .. i*hop+win]) + eps )    // dB
```

One pass, O(n), negligible next to transcription. `rms()` and `linear_from_db()` already exist in `concat-media/src/silence.rs` — same maths, different consumer.

### Step 2 — a *local* noise floor per search window

This is the part that makes it work on real recordings. A global threshold fails because a quiet room and a noisy room differ by 30 dB.

For each word, take the frames in `[word.start − 0.30, word.start + 0.30]` and use a low percentile as the floor:

```
floor = percentile(e[window], 20)         // 20th percentile ≈ the quiet parts
```

Percentile, not mean: in a window that is mostly speech, the mean sits above the floor and you would never detect a rise.

### Step 3 — find the onset, asymmetrically

Whisper's timings tend to **lag** the real onset (attention smoothing needs context before it fires). So search further forward than backward:

```
search  = [word.start − 0.20, word.start + 0.12]
rule    = first frame i in search where
            e[i] − floor ≥ 8 dB            // clear rise
        and e[i − 1] < floor + 8 dB        // the rise starts here, not mid-slope
        and e[i+1] ≥ floor + 6 dB          // debounce: survives at least 10 ms
```

Then:

```
word.start = t(i)                                        // the onset
prev.end   = min(prev.end, word.start − 0.04)            // 40 ms of air
```

No frame satisfies the rule (mid-phrase word, or a very quiet fricative)? **Keep Whisper's value.** Doing nothing is the correct, honest behaviour — and it is what makes the metric meaningful.

### Step 4 — enforce what the model promises

After snapping every word in a segment:

- monotonic: `words[i].end ≤ words[i+1].start` (never let a snap reorder words)
- minimum duration: `end ≥ start + 0.02` (your existing clamp uses 0.02 — keep it)
- clip to the refined segment bounds (as today)
- never move a word outside the segment; if snapping pushes past it, keep Whisper's value for that word

### Step 5 — record where the time came from

Add a marker per word (an enum or a bool) saying **whisper** or **snapped**. Two reasons: you can A/B the feature, and a debug export can print `word, whisper_time, snapped_time, delta_db` so a bad result is diagnosable instead of mysterious. Without this, "is it better?" becomes a feeling.

### Starting parameters

| Parameter | Start | Tune by |
|---|---|---|
| hop / window | 5 ms / 20 ms | smaller hop = finer onset, more noise |
| floor percentile | 20th | lower (10th) in noisy rooms |
| rise threshold | 8 dB | raise to 10–12 if false onsets appear on breaths |
| debounce | 2 frames (10 ms) | raise on very noisy audio |
| search back / forward | 0.20 s / 0.12 s | widen forward if words are consistently early |
| gap left before previous word | 40 ms | 30–60 ms is the perceptual sweet spot |
| min word duration | 20 ms | matches your existing clamp |

---

## 3. Level 1.5 — cheaper than a model, catches more

Two extra signals, no new dependency, still offline:

- **Zero-crossing rate (ZCR).** Trivial to compute, no FFT. Fricatives and sibilants (*s, sh, f, t*) raise ZCR sharply even when energy barely moves. That catches word starts that energy alone misses — very common in English and Indonesian.
- **Spectral flux.** Sum of positive changes in band magnitudes between frames. Finds consonant→vowel transitions mid-phrase, where there is no energy dip. Needs a small transform: either a short FFT (you link FFmpeg, which has one, but pulling it into `concat-speech` widens the dependency) or 3–4 band energies computed with Goertzel filters — about 30 lines, no dependency.

With ZCR alone, expect the mid-phrase column above to improve from "no change" to "some change". Full spectral flux is a Level-2-grade improvement without a model download, and it is the best value in this whole ladder. It is also the honest upper bound of "cheap": past this point, only a model helps.

**Implemented record (T3, 18 Oct 2026) — the band edges this section asks to be recorded once chosen:**

- Edges: **300–800, 800–1500, 1500–3000, 3000–6000 Hz** — the starting guess above, chosen as the implemented set. Code: `concat_timing::signals::BANDS`; four geometrically spaced Goertzel probes per band, band magnitude = probe mean, rectangular window, flux = Σ of positive band-magnitude deltas per 5 ms hop. Retuning is recorded here, per T8 step 3.
- Hann weighting was measured and **rejected**: tapering the window does not remove the offset artifact below, because the step edge is in the signal, not the window.
- **Offset gate (measured fix):** a flux rise counts only where frame energy is not falling (`e[i] ≥ e[i−1]`). Without it, sliding a window off the end of a loud burst makes band magnitudes oscillate (Dirichlet sidelobes of the signal's own sharp cut) and raw flux spikes at burst *offsets*: on the synthetic corpus, words after a gap snapped **130 ms early, into the previous burst's tail**. An offset is a decay; a consonant→vowel transition at equal loudness has flat energy. The gate separates them with no new tunable number. Regression test: `signals::tests::a_burst_offset_is_not_a_flux_rise`.
- Thresholds (starting guesses, T8-tunable, in `signals::Wideband`): ZCR margin 0.10 crossings/sample above the local 20th-percentile floor; flux ≥ max(2 × local median flux, 0.0005). The flux floor's provenance: the equal-loudness 0.05-amplitude transition fixture peaks at ≈0.0016 raw flux; steady-state jitter is ≈1e-15.

---

## 4. How to measure it — this decides whether the work was real

You cannot tune this by ear. Build the harness first; it is under an hour of work and you will use it for every future change.

### 4a. The corpus is the video you already have

Not a new recording. The right corpus is one whose failure modes you already know — and you have one: the 5-minute video you have run through kconcat **and VTS** repeatedly. It beats fresh audio because it is your real mic, room and speech; you already know where it breaks; and it lets kconcat and VTS be compared on identical input.

```
test-corpus/
  video-5min.mp4
  audio-16k.f32        # decoded once
  whisper-cache.json   # segments + words, once
  runs/baseline.json
  runs/after-t2.json
```

**Whisper runs once.** Tuning the snapper needs only the samples and the cached word list, so every later iteration is seconds of CPU — no GPU, no re-transcription. The 5-minute video is therefore *cheaper* to iterate on than a fresh clip.

### 4b. Tier 1 — metrics that need no ground truth

These prove a timing is wrong without knowing the right one:

| Metric | What it catches |
|---|---|
| **Onset plausibility rate** — for each word, is there an energy rise (±150 ms) at its start? | a word starting mid-silence: provably wrong |
| **Monotonicity violations** — reversed or overlapping words | attention artefacts, snap bugs |
| **Outliers** — words outside their cue, negative duration, > 2 s | clearly broken output |
| **Δ vs baseline** — how many words moved, which way, how far | **a uniform shift is an offset bug, not an improvement** |

Start here. Tier 1 answers "is it better?" for almost every change, in seconds, with no manual work.

### 4c. Tier 2 — absolute numbers (optional, later)

Only when you want to know *by how much*. Take **2–3 segments you already know fail**, not a random 60 s, and **correct rather than label from scratch** — the text comes from a previous .srt run and the plosives are visible in Audacity's waveform, so you are nudging existing times instead of typing 60 words from nothing.

### 4d. The holdout rule

> Never tune parameters on the same segments you use to prove the result.

Set one segment aside **before** tuning and open it only when you want the final answer. If the held-out numbers are good too, the gain is real and not overfitted to the corpus.

### 4e. Tier 2 table — the script

Run your transcribe + snapping, then print:

```
words total                      N
median |error|                   ms
p90 |error|                      ms
within 50 ms / 80 ms / 100 ms    %
--- split ---
after a gap ≥ 80 ms              median, p90, %<80ms
mid-phrase (no gap before)       median, p90, %<80ms
--- direction ---
mean signed error                ms   (+ = late, − = early)
```

**Look at the split, not the average.** If `after a gap` improves a lot and `mid-phrase` doesn't, the implementation is correct and the ceiling is physics. If `after a gap` doesn't improve, the parameters or the search window are wrong.

The **signed** error matters too: if it is consistently −40 ms, you have a constant offset bug, not a detection problem — and that has a trivial fix (shift once).

### A perceptual check, because numbers lie at the edges

Render the karaoke highlight at 30 fps and count frames where the highlight is visibly on the wrong word. Target: under about 5% of frames. That is the number a viewer actually sees.

### Baseline first

Run the harness **before** changing anything and save the output. A change without a before/after is a guess.

---

## 5. Two traps specific to your tree

- **Speed-changed clips.** Transcription runs on the clip's audio window; words come back on that window's clock and the caller divides by speed. If snapping runs on the window clock (correct), do not let the speed division happen first — otherwise every snap position is wrong by the speed factor. Check where `transcribe()`'s result is consumed before wiring anything in.
- **Do not run snapping on a VAD-compacted buffer.** Your Stage-1 change fixed exactly this class of drift. Keep snapping downstream of that decision, on the same samples array, and it stays fixed.

## 6. What this unlocks

Not cosmetic. Accurate word times are the prerequisite for:

- karaoke that looks right (`highlight_color` already renders one overlay per word — it is only as good as the timings)
- word-by-word caption display, CapCut's biggest visual difference
- transcript editing — deleting a word and cutting the video to match

Which is why A comes before B and C. Everything downstream is gated on this number.

## 7. Exit criteria

Level 1 is done when, on your 60-second reference:

- `after a gap` → **median error under 60 ms**, p90 under 120 ms
- `mid-phrase` → unchanged or better (no regression)
- signed mean error within ±15 ms
- the harness runs on every commit, so a regression is caught, not felt

Then Level 1.5, then re-measure. Level 2 (forced alignment) only if mid-phrase accuracy starts to matter for your content — which it will if you ever want word-by-word display.
