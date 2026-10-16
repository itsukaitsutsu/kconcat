# kconcat roadmap — closing the simple-task gap

*Supersedes the earlier roadmap. No shipping, CI, release or repo-hygiene items:
this is a personal tool. What follows is the shape of the work and the order that
avoids doing it twice — pace is yours.*

**Scope reminder:** a 10–40 min talking-head or screen recording becomes a
45–90 s vertical short — captioned, silence-trimmed, styled, exported — offline.
Nothing outside that sentence.

---

## 0. What is already built (do not rebuild)

Verified against `1aece2d`. Several of my own earlier "missing" claims were wrong:

| Capability | Where it lives |
|---|---|
| **32 text presets** | `crates/concat/text-presets/` |
| Caption style vocabulary — stroke, plate, radius, padding, tracking, line height, max width | `TextStyle`, `model.rs:980` |
| **`highlight_color`** — the karaoke colour, already rendered as one overlay per heard word | `titles.rs:367`, `titles.rs:520` |
| **"Apply to all captions"** — exists, defaults **on** | `studio.rs:963`, `3550`, `5279` |
| Per-word reveal (baked reveal-order map) | `concat-core/src/shader.rs:222` |
| SRT + VTT in **and** out | `subtitle.rs` |
| Captions remapped through a cut (split, rejoin, clip at edge) | `captionmap.rs` |
| Silence detection → sections → trim | `sections.rs`, `sectionlist.rs` |
| Offline ASR, 3 engines, VAD, ETA | `transcribe.rs` |
| 3 TTS engines + voice cloning | `tts.rs`, `chatterbox.rs` |
| Subject segmentation (selfie / person with hair detail / object) | `concat-vision/src/segment.rs` |
| 123 effect packages, proxies, keyframes + curve editor, 4K/HEVC/AV1 export | `concat-effects/packages/`, `concat-host`, `concat-media` |
| MCP / JSON-RPC / gRPC API + CLI | `concat-api`, `concat-server` |

**Implication:** most of what CapCut *looks* like it has, you already have the
machinery for. The work below is mostly **aiming existing machinery**, not
building new engines.

---

## 1. The spine

One dependency chain decides the whole order:

```
   ┌─────────────────────────────────────────────────┐
   │  WORD TIMING IS ACCURATE                        │
   │  (T1 harness → T2 snapping → T3 zcr/flux)       │
   └───────────────┬─────────────────────────────────┘
                   │
     ┌─────────────┼──────────────┬──────────────────┐
     ▼             ▼              ▼                  ▼
 karaoke is   word chunking   transcript        waveform
 readable     looks right     editing works     drag is usable
 (C3)         (C1)            (E1)              (E3)
                   │
                   ▼
             caption presets land on
             something worth styling (C2, C4, C5)
```

Everything in Track 1 is upstream of everything else. Do not start Track 2 for
visual gain while word times are ±300 ms — you will re-tune every preset again
afterwards.

---

## 2. Track 1 — Timing and correctness (the foundation)

### T1. Test corpus + harness — **do this first**
- **Why:** without it every change below is a guess, and there is no way to tell a wrong implementation from a physical ceiling.
- **In one line:** the corpus is the 5-minute video you already tested in kconcat and VTS, cached once; measure with label-free metrics first.
- **Effort:** S (cache + runner); M for the optional labelled set
- **Done when:** the runner prints the label-free table for any run, and a baseline exists from before any change.
- → full method: `A-word-timing-level1.md` §4a–4e

### T2. Word-onset snapping (Level 1)
- **Why:** Whisper attention timings are ±200–500 ms, quantised to centiseconds, and monotonic per segment — one bad early word drags the rest of the cue. Currently words are only *clamped* into the corrected segment (`transcribe.rs:1108-1111`).
- **Approach:** snap each word to the real energy rise before its start; keep Whisper's value where no onset exists. Parameters and the full algorithm: `A-word-timing-level1.md` §2.
- **Effort:** M
- **Done when:** on the harness, **after-a-gap median under 60 ms, p90 under 120 ms**, signed error within ±15 ms, and mid-phrase is **unchanged or better** (no regression).

### T3. Level 1.5 — ZCR + spectral flux
- **Why:** mid-phrase words have no energy dip, so T2 cannot help them. Fricatives and consonant→vowel transitions are visible in *spectral* change, not energy.
- **Approach:** zero-crossing rate first (no dependency), then 3–4 Goertzel bands for spectral flux (~30 lines, still no new crate).
- **Effort:** S–M
- **Done when:** mid-phrase median improves and the karaoke frame-check (below) improves with it. This is the honest upper bound of "cheap" — past it, only a model helps.

### T4. Script-aware tokenisation
- **Why:** `transcribe.rs:742` splits on `is_ascii_whitespace`. In Thai/Japanese/Chinese an entire sentence becomes one "word", so per-word reveal is meaningless — while `subtitle.rs` deliberately round-trips those scripts. The two halves of your own pipeline disagree about what a word is.
- **Effort:** S
- **Done when:** a Japanese or Thai cue produces multiple word units, and `clean_words` still passes its non-ASCII tests.

### T5. Caption identity — store it, don't derive it
- **Why:** a caption that keeps none of its words through an autotrim silently stops being a caption (reproduced; `caption-identity-bug.patch`). Consequence: **"Burn captions" OFF still burns it in** (`export.rs:643`), it blocks caption placement (`clips.rs:1282`), and the Sections panel stops seeing it (`sectionlist.rs:189` — measured 1 → 0 caption sections). The predicate is duplicated in **six places across three crates**.
- **Approach:** a stored field set at creation, read by `Clip::is_caption()`; every other site calls it. Detail: `kconcat-audit-2026-10-16.md` §4.
- **Effort:** S–M
- **Done when:** the three tests in the patch pass, and a grep shows **one** definition of caption-ness in the tree.

### T6. Per-track ±nudge offset
- **Why:** the cheap escape hatch for whatever T2/T3 cannot fix. CapCut's timings are not perfect either — it just makes correcting them trivial.
- **Approach:** ±10 ms steps, stored on the track so it survives a re-transcribe.
- **Effort:** S
- **Done when:** a caption run can be shifted and the offset persists across regeneration.

**Track 1 exit:** word times you can trust, plus a rule that never silently
loses caption identity. Nothing in Track 2 is worth starting before this.

---

## 3. Track 2 — Caption display (what you actually see)

### C1. Word chunking — 1–3 words per caption group
- **Why:** this is *the* reason CapCut captions look fast, and it is not a styling effect. A five-second cue with a whole sentence reads like a subtitle; the same sentence chopped into 2-word groups reads like a Reel. Nothing in the tree does this.
- **Approach:** the split-and-remap machinery already exists (`captionmap.rs`, `sections.rs`) — point it at word counts instead of cuts. Configurable max words per group; never split inside a word; keep groups from straddling a pause.
- **Effort:** M
- **Depends on:** T2/T3 — group boundaries are only readable if word times are right.
- **Done when:** a generated track splits into groups with a chosen max, and karaoke highlight stays aligned through the split.

### C2. Eight caption presets + a group tag
- **Why:** the 32 shipped presets are **title looks** (Western, blackletter, comic, stencil…). **None sets `highlight_color`, position, or motion** — verified: `grep -rl highlight_color crates/concat/text-presets/` returns nothing. The mechanism already supports it: `highlight_color` is a `TextStyle` field, so a preset can set it **today with no Rust and no schema change**.
- **Approach:** ship eight caption-shaped presets and tag them as a group so captions are not mixed into the title list. Names and parameters: `B-caption-styles.md` §4.
- **Effort:** S (data + one round-trip test)
- **Done when:** applying one preset to a caption track gives a CapCut-shaped caption **with no inspector use**, and a test asserts `highlightColor` round-trips through `preset.toml`.

### C3. Wire karaoke to presets
- **Why:** the renderer exists and is tested (`a_word_timed_caption_gets_one_highlight_per_heard_word`), but nothing in the UI reaches it in one step.
- **Effort:** S
- **Done when:** picking a karaoke preset produces highlighted word overlays without visiting Advanced settings.

### C4. Uppercase / text transform
- **Why:** most CapCut caption styles are ALL CAPS; there is no transform field in `TextStyle`.
- **Effort:** S
- **Done when:** a preset can uppercase without editing the source text (so the transcript keeps its real casing).

### C5. Caption motion (pop / typewriter / word-reveal)
- **Why:** the last visual gap. The reveal-order map is already baked per title (`shader.rs:222`) and effect packages are folders discovered at compile time.
- **Blocker — decide first:** where motion lives. (a) `[motion]` block in `preset.toml` — closest to CapCut, presets carry a whole look; (b) inspector-only — less coupling, presets stop being one-click; (c) a `concat.caption-pop` effect package the preset names — reuses existing machinery, keeps presets data-only. **Declide before writing preset files** — it changes their shape.
- **Effort:** M
- **Done when:** a preset yields motion on apply, and the choice survives a project reload.

**Track 2 exit:** a caption track that looks like the thing you were paying
CapCut Pro for — without touching the inspector.

---

## 4. Track 3 — Editing leverage (what you can do)

### E1. Transcript editing — delete a word, the video cuts
- **Why:** the payoff of Track 1, and Descript's entire business. CapCut only gestures at it (filler removal). Nothing in yours or upstream does it. **You are ~80% there** — `captionmap.rs` does the inverse (remap words through a cut), `sections.rs` turns ticks into trims.
- **Effort:** M
- **Depends on:** T2/T3 — deleting a word is guesswork with ±300 ms timings.
- **Done when:** deleting "um" or a whole sentence in the transcript ripples every lane, and audio, picture and captions stay in sync.

### E2. Indonesian filler-word removal
- **Why:** CapCut's filler removal is English-shaped ("um", "uh"). Indonesian fillers are different: *eee, anu, kayak, gitu, apa ya, terus*.
- **Effort:** S (a word list plus the E1 machinery)
- **Done when:** a one-click pass removes them within a selected caption run, and does not touch the same words when they are meaningful.

### E3. Waveform under the caption row + draggable word edges
- **Why:** lets you fix the residue of T2/T3 in seconds instead of re-transcribing. This is what makes people *trust* a timing system.
- **Effort:** M
- **Done when:** dragging a word edge moves the highlight and the timing, and the edit survives export.

---

## 5. Track 4 — Finishing the job (vertical + export)

### V1. Auto-reframe 16:9 → 9:16
- **Why:** the biggest single thing you'd otherwise open CapCut for — and CapCut moved it to **Pro-only** in April 2026. You already have the hard part: RVM carries recurrent state frame-to-frame "so an outline does not flicker" (`segment.rs`).
- **Approach:** per-frame mask centroid → smooth (one-euro or Savitzky-Golay) → drive crop keyframes. Keyframes, crop and the curve editor all exist.
- **Effort:** M
- **Done when:** a talking-head 16:9 clip becomes a framed 9:16 export with no manual keyframing, and the crop does not jitter.

### V2. Platform export presets + loudness
- **Why:** removes the last reason to open CapCut for a simple export. *(Not verified that none exist — the string search was ambiguous; confirm before building.)*
- **Approach:** TikTok / Reels / Shorts / YouTube targets: aspect, bitrate, and **−14 LUFS**. `loudnorm` is already referenced in the effects chains.
- **Effort:** S
- **Done when:** one menu choice gives a compliant upload-ready file.

### V3. Auto ducking
- **Why:** music under speech is the most common two-track edit, and doing it by hand every time is the friction people cite.
- **Effort:** S–M
- **Done when:** a music bed drops under speech automatically and is still reversible.

### V4. Batch export queue
- **Effort:** M
- **Done when:** five shorts can be exported in one pass without babysitting.

### V5. Stabilisation — *optional*
- Only if you film handheld. Long, and easy to live without otherwise.

---

## 6. Out of scope — permanently

Templates, music/sticker/font libraries, cloud AI, collaboration, cloud sync,
mobile parity, 130-language counts, colour-grading depth, and any
release/marketing work. Each is a licensing business, an infra business, or a
GPU-spend race — and none serves the workflow in §0's scope line.

Also out: **in-house cloud AI of any kind.** It would destroy the only durable
property this tool has. If generated video is ever wanted, it goes through the
API you already own, with the user's own key.

---

## 7. The numbers that say whether this is working

**Tier 1 — no labels, seconds per run:**
1. **Onset plausibility rate (%)** — share of words whose start sits on a real energy rise. Only Track 1 moves this.
2. **Monotonicity violations + outliers** — must be zero.
3. **Δ vs baseline** — direction and distance. A uniform shift means an offset bug, not an improvement.

**Tier 2 — only once labels exist:**
4. **After-a-gap median error (ms)** on the **held-out** segment.
5. **Karaoke frames on the wrong word (%)** at 30 fps, target under ~5% — what a viewer actually sees.

Then one end-to-end check, re-run whenever a track closes: a 20-minute
recording → 60-second vertical short, offline, start to finish, counting manual
corrections. If that count rises, something regressed.

**Bonus the corpus makes possible:** the same video, captioned by kconcat and by
VTS, side by side. A fixed corpus with two implementations on top of it is an
objective answer to "which is better" — not a feeling.

---

## 8. Analysis still owed (the "C" conversation)

- **Tier 1 metric script** — reads `whisper-cache.json` (+ `audio-16k.f32`) and prints onset plausibility, monotonicity, outliers and Δ vs baseline. The only roadmap item usable **without touching app code**, and the prerequisite for measuring T2/T3 at all.
- **Motion schema decision** (C5 blocker) — pick one of three, document it, then write preset files.
- **Caption pipeline map** — transcription → `captionmap` → placement → reveal → render → export, with a `file:line` at each hop. You asked for this once; it is also how the identity bug becomes debuggable instead of mysterious.
- **Verify V2's starting point** — confirm no platform export presets exist before adding them.
