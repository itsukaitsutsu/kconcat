# Why words drift out of sync in kconcat — and how to fix it

*Audit of `itsukaitsutsu/kconcat` at `e028a65` (16 Oct 2026). Every claim below is a file:line in your tree, not a guess.*

## Verdict up front

CapCut is better at word-level sync. Not because it's magic — because it **re-times words against the waveform** instead of trusting the recogniser's attention peaks. Your pipeline does the opposite: it trusts Whisper's token timestamps for every word inside a cue, and only snaps *whole cues* to audio. So a caption block sits in roughly the right place while the words inside it land wherever Whisper guessed.

---

## 1. The root cause

`src/crates/concat-speech/src/transcribe.rs:623`

```rust
params.set_token_timestamps(request.word_timestamps);
```

This is `whisper-rs` cross-attention token timestamps. Whisper derives them from attention peaks, then quantises to **centiseconds** (`transcribe.rs:656`, `segment.start_timestamp() as f64 / 100.0`). Published measurements put Whisper's segment timestamps at roughly **±1000 ms** and cross-attention word timings in the **±200–500 ms** band — and they are constrained to move monotonically inside a segment, so an error early in a cue pushes every later word in that cue.

Your own doc comment at `transcribe.rs:200` says these "are heard times, not estimates." That is the one line in the file that isn't true, and it's the reason this bug keeps surviving review.

### What the VAD pass does *not* fix

`transcribe.rs:1068` `refine_segments_with_vad` is good work — 20 ms RMS chunks, −48 dBFS floor, snaps a **segment** start/end to real voice. But look at what it does to words (lines 1108–1111):

```rust
for word in &mut seg.words {
    word.start = word.start.clamp(seg.start, seg.end);
    word.end   = word.end.clamp(word.start + 0.02, seg.end.max(word.start + 0.02));
}
```

Words are only **clamped** into the corrected segment. Their positions *inside* the cue are still untouched Whisper attention estimates. That is precisely the symptom you're describing: the block is approximately right, the individual words aren't.

### Secondary cause — languages without spaces

`transcribe.rs:742`

```rust
let begins_word = words.is_empty() || bytes.first().is_some_and(u8::is_ascii_whitespace);
```

Words are split on ASCII whitespace. For **Thai, Japanese, Chinese, Lao, Khmer** there are no spaces between words, so an entire sentence collapses into one "word" spanning the whole cue. Any per-word reveal or karaoke highlight is then meaningless in those languages — while your `subtitle.rs` deliberately round-trips Thai/Japanese correctly, so the two halves of your own pipeline disagree about what a word is.

### Tertiary — the frame grid

Captions land as title clips on f64 seconds and `concat-export/src/resolve.rs` quantises to the frame grid. That's ±1 frame (17 ms at 30 fps) — negligible for a caption block, noticeable only on a hard karaoke flip. Not the culprit, but it caps how tight you can ever get.

---

## 2. Diagnose yours in 60 seconds

Which of these is it? The fix differs, so don't skip this.

| Symptom | Cause | Where to look |
|---|---|---|
| **Whole caption is late/early by the same amount all the way through** | Clock/offset bug, not ASR | `source_start` on the window vs the clip's trimmed in-point; speed division (`transcribe.rs:452` says the caller divides by speed and adds clip start — verify that, and verify it also applies words, not just segments) |
| **Starts fine, gets worse by the end** | Sample-rate / accumulated loss | The 16 kHz decode at `transcribe.rs:488`; any path that still splices audio for VAD; long windows |
| **Cue block right, words inside it sloppy** | Whisper attention timings | `transcribe.rs:623` — this is the common case and the one worth fixing |
| **One giant "word" per cue in Thai/Japanese/Chinese** | Whitespace split | `transcribe.rs:742` |
| **Fine in the transcript, off after you cut** | Remap arithmetic | `concat-project/src/captionmap.rs` `remap` / `clean_words` |

Pick a clip, find a hard consonant onset (a "p", "t", "k"), and compare its position to the caption at 0:05, mid-clip, and near the end. That single test tells you which row of that table you're in.

---

## 3. The fix ladder

### Level 1 — snap every word to voice onset (do this first)

**Cost: ~150 lines + tests. No new models. Offline. Works for all four engines (whisper-rs, whisper.cpp CLI, faster-whisper, and the script path).**

You already have the DSP. `concat-media/src/silence.rs` exports `rms()`, `spans_from_windows()` and `linear_from_db()` — the exact primitives. Extend the refinement pass from segments to words:

```rust
// For each word inside a refined segment:
//   1. take the window [word.start - 0.15, word.start + 0.15]
//   2. compute RMS in 5 ms hops over a rolling noise floor
//   3. word.start = first hop where energy rises above floor
//   4. previous word's end = that onset minus a small gap (30-50 ms)
//   5. never let a word cross its neighbour; if no onset found, keep Whisper's guess
```

This is the well-documented "CTC/attention → waveform" trick that takes word accuracy from ±200 ms to about ±100 ms, and it is the single highest-value change available to you. Because it's audio-driven, it also self-corrects the engine: whisper.cpp Vulkan and faster-whisper will start agreeing with each other.

**Do not** tune this against a synthetic file. Use 60 seconds of your own speech with plosives and stops; count words landing late by more than 80 ms before and after.

### Level 2 — real forced alignment (the proper fix)

Whisper's attention peaks cap you at "close". Phoneme-level **forced alignment** (wav2vec2/CTC, WhisperX-style, or whisper.cpp's DTW alignment heads) is what gets word boundaries to ±50–100 ms, which is where CapCut's karaoke styles live.

It fits your architecture better than you'd think:
- `models/manifest.toml` already pins every download with a **SHA-256** and mirrors them on the `models-v1` release — an alignment model just becomes one more row, and `verify` already refuses undigested entries.
- **ONNX Runtime is already loaded** for the vision models (`concat-vision`, behind the `infer` feature), so the inference machinery is in the tree.

Cost: one more model download (tens of MB to a few hundred MB, depending on language coverage), plus a fallback path when it isn't installed. Gate it behind "High accuracy word timing" in the Auto-caption sheet.

### Level 3 — the UX that makes CapCut *feel* synced

CapCut's timings aren't perfect either; it's that correcting them is trivial and its styles re-derive per word. Cheap wins for you:

1. **Shift-all-captions nudge** on a track (±10 ms steps, and the offset stored on the track so it survives a re-generate).
2. **Waveform under the caption words** with draggable word boundaries — you already render a section strip, so the drawing code exists.
3. **"Snap cues to waveform"** on a selected caption run that re-runs the Level-1 pass on demand.
4. **Auto re-time on cut** — `captionmap.rs` already clips and rejoins words through a cut; make sure the rejoin re-snaps rather than reusing stale Whisper times.

### Level 4 — fix the word-splitting for space-less scripts

`transcribe.rs:742` should split on `char::is_whitespace` **plus** a script-aware rule: for CJK/Thai, fall back to one token per recognised unit (whisper's tokens are already syllable-ish for those languages) and never emit a cue-spanning single word. `clean_words` in `captionmap.rs` already tests `is_alphanumeric` (not ASCII) — apply the same standard at the token-splitting step so the whole pipeline agrees.

---

## 4. What CapCut actually does better (be fair to it)

| | CapCut | kconcat today |
|---|---|---|
| Word timing source | Re-derived against audio per word | Whisper attention peaks, clamped to the cue |
| Languages | 55–130+, auto-detected | 24 UI locales; ASR quality varies by model |
| Karaoke/reveal styles | Large preset library, one click, per-word retimed | Per-word reveal exists in the engine (`concat-core/src/shader.rs` bakes a reveal-order texture); no preset library |
| Correction UX | Drag, split, merge, inline edit on desktop *and* phone | In-place editing exists; no waveform snapping |
| Cost | Now mostly **Pro-gated** (auto-captions include Pro-only functionality since 2026) | Free, local, offline, no cap |
| Your timings as data | Not exportable | **SRT/VTT out, word timings preserved, remap through cuts** — you're strictly ahead here |

So: CapCut wins on *polish*, you win on *data*. Close the word-timing gap with Level 1 and you've got a genuinely defensible claim — "local, free, offline, and your caption timings are real files you own" — that no one else in this comparison can make.

**And it isn't cosmetic.** Accurate word timings are the prerequisite for text-based editing (cut the video by deleting words in the transcript). `captionmap.rs` was clearly built for that. Right now the timings aren't good enough to trust, so that whole feature stays locked behind this fix.

---

## 5. Recommended order

1. Run the 60-second diagnostic above; confirm it's row 3 of the table (attention timings), not a clock bug.
2. Ship **Level 1** word-onset snapping + tests. Measure: count of words >80 ms off, target under ~10%.
3. Fix **Level 4** splitting for Thai/CJK — small, and it stops your own `subtitle.rs` tests from being misleading.
4. Ship **Level 3.1/3.2** (nudge + waveform drag) so users can fix the remainder without re-transcribing.
5. Only then evaluate **Level 2** forced alignment — worth it if you want Descript-style text editing or true karaoke, and the model plumbing is already there.
