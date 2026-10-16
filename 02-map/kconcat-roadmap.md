# kconcat roadmap — closing the simple-task gap

*Supersedes the earlier roadmap. No shipping, CI, release or repo-hygiene items:
this is a personal tool. What follows is the shape of the work and the order that
avoids doing it twice — pace is yours.*

**Scope reminder:** a 10–40 min talking-head or screen recording becomes a
45–90 s vertical short — captioned, silence-trimmed, styled, exported — offline.
Nothing outside that sentence.

---

## 0. What is already built (do not rebuild)

Verified against `1aece2d`. Several of my own earlier "missing" claims were wrong — five so far:

| Capability | Where it lives |
|---|---|
| **32 text presets** | `crates/concat/text-presets/` |
| Caption style vocabulary — stroke, plate, radius, padding, tracking, line height, max width | `TextStyle`, `model.rs:980` |
| **`highlight_color`** — the karaoke colour, already rendered as one overlay per heard word | `titles.rs:367`, `titles.rs:520` |
| **"Apply to all captions"** — exists, defaults **on** | `studio.rs:963`, `3550`, `5279` |
| **Word chunking — four layout modes** (Engine / Readable lines / Burst 3-words / One word) + the **Re-split** button | `crates/concat/ui/dialogs/captions.slint:78`, `:615`; `crates/concat-project/src/captionmap.rs:597–627`; `resplit_drafts` `crates/concat/src/panes/captions.rs:596` |
| **Waveform drawing** — timeline sound bands, media pane, speech dialog | `crates/concat/ui/primitives/wave.slint`; `ui/timeline/lanes.slint:184`; `media-pane.slint:558` |
| **Whisper translate-to-English** in Auto-caption | `crates/concat/ui/dialogs/captions.slint:57`; `concat-speech/src/transcribe.rs:248` |
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
 (C3)         (C1)†           (E1)              (E3)
                   │
                   ▼
             caption presets land on
             something worth styling (C2, C4, C5)
```

† **C1 already exists** — the four layout modes and the Re-split button ship (§0 and §3). After T1/T2 it becomes a *validation* task: run Burst on the same video and compare. Nothing to build.

Everything in Track 1 is upstream of everything else. Do not start Track 2 for
visual gain while word times are ±300 ms — you will re-tune every preset again
afterwards.

### Order that minimises sync risk (decided 16 Oct 2026)

The per-word features are where every desync bug lands — so timing and stored
identity come before any new word-level UI:

1. **T1** — the harness. Nothing above it is measurable without it.
2. **T2 → T3** — word timing. This is what makes **Burst and Word look right**
   (C1 — already built).
3. **T5** — caption identity, the class-killer. Swap with step 2 if the pain you
   feel most is *"a caption vanishes or gets burned in after I cut"* rather than
   *"the highlight lands late"*.
4. **C1 validation** — run Burst on the same video and compare; no new UI.
5. **E4** — Enter-split at the caret.
6. Then the visual work (C2–C5), then Track 4, and **romaji / Japanese karaoke
   last** (§9).

**Treat Burst/Word as the canary:** if those look wrong, every sync feature built
on top will look wrong too — so measure there first, and never style over a
timing you have not measured.

---

## 2. Track 1 — Timing and correctness (the foundation)

### The four faces of "out of sync" — diagnose before building

All four look identical to a viewer and have different fixes. Identify which one
you have first:

| What you see | Cause | Fix |
|---|---|---|
| Words light up early/late by a second or more; the cue itself is fine — worst in **Word** mode, where one cue is ~300 ms | Whisper attention timing, ±200–500 ms (a model fact, not a bug) | **T2/T3** |
| After a cut, a caption *vanishes* from lists, blocks placement, or is burned in with "Burn captions" **off** | Identity derived from data a trim clears — reproduced with tests | **T5** |
| The highlight silently drops a word after you edit the caption's text | Text edited, `words` not; nothing keeps the pair consistent | **T7** |
| Boundaries land on the wrong word; imports look worse than generated captions | Word timings *estimated* per character on every import path | **T2** — and know which side of the estimate you are on |

### The sync matrix — five assertions every edit path must pass

Put these in the test of **each** path that can touch a caption (trim, autotrim,
playhead split, Enter-split, re-split, inline text edit):

1. `content` equals the caption's words joined — or the words are absent **by
   design**, never stale;
2. words are ordered, non-overlapping, and inside the cue's span;
3. identity survives: `is_caption()` is as true after the edit as before;
4. `locate_words` finds **every** word in the content (the highlight has no holes);
5. clip span, cue span and words agree once the edit settles.

A path that cannot pass all five is where "sync looks weird" is born. Add the
assertions **before** adding the feature.

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
- **Careful point (16 Oct 2026):** projects saved before the field exists must
  keep loading — give it a serde default meaning *"derive it the old way"*, so
  old files behave exactly as today until they are edited.
- **Done when:** the three tests in the patch pass, and a grep shows **one** definition of caption-ness in the tree.

### T6. Per-track ±nudge offset
- **Why:** the cheap escape hatch for whatever T2/T3 cannot fix. CapCut's timings are not perfect either — it just makes correcting them trivial.
- **Approach:** ±10 ms steps, stored on the track so it survives a re-transcribe.
- **Effort:** S
- **Done when:** a caption run can be shifted and the offset persists across regeneration.

### T7. One writer for text and words
- **Why:** a caption is its text **and** its word timings, and today two paths
  change one without the other. An inline text edit in Sections rewrites
  `style.content` and leaves `clip.words` stale
  (`crates/concat/src/panes/sections.rs:680 edit_row_text` — reasoned from the
  code, **not yet tested**); and every import path *invents* timings per
  character where the file had none. A stale word list makes `locate_words`
  return `None` for the words whose text is gone (`captionmap.rs:241`), so the
  highlight quietly drops them.
- **Approach:** make `Command::UpdateClip` — the one writer for a text patch —
  keep the pair consistent: content unchanged → do nothing; token count unchanged
  → re-text the words positionally and **keep their timings** (fixing a typo must
  not lose karaoke); otherwise keep only the words that still appear in order
  (prune), and never invent new ones.
- **Effort:** S (one writer + a test per case)
- **Done when:** a test edits one caption's text three ways — same tokens, one
  word changed, one word deleted — and asserts in each case that the words still
  describe the content and that the surviving words kept their timings.
- **Depends on:** T5 — once words can be pruned away entirely, identity must not
  rest on them.

**Track 1 exit:** word times you can trust, plus a rule that never silently
loses caption identity or orphans the words the highlight needs. Nothing in
Track 2 is worth starting before this.

---

## 3. Track 2 — Caption display (what you actually see)

### C1. Word chunking — **already built; validate after T2**
- **Correction (16 Oct 2026, second pass):** this is **not** missing. Auto-caption
  carries four layout modes — `Engine` (as transcribed), `Readable lines`,
  `Burst (3 words)`, `One word at a time`
  (`crates/concat/ui/dialogs/captions.slint:78`) — with the VTS-style **Re-split**
  button (`:615` → `CaptionsMsg::DraftResplit`,
  `crates/concat/src/panes/captions.rs:376`). The rule set is
  `GroupPolicy::for_preset` (`crates/concat-project/src/captionmap.rs:597–627`),
  applied by `resplit_drafts` (`crates/concat/src/panes/captions.rs:596`) →
  `resplit_cues` (`captionmap.rs:857`): Burst = 3 words / 18 chars / 2.5 s, break
  on a 0.35 s pause, groups below `min_duration` merged. That is CapCut's rule
  set, already written.
- **Why it still reads badly** — the boundaries come from *word timings*
  (`words_for_resplit`, `captionmap.rs:723`). Real words are used as they are, and
  carry Whisper's ±200–500 ms error; with no words at all the split is
  **estimated from the text, proportional to characters** (`captionmap.rs:737`),
  which is what every import path does
  (`crates/concat/src/panes/import_captions.rs`). Either way the cut lands in the
  wrong place, and on a 0.4–2.5 s Burst cue that error is 15–60 % of what the
  viewer sees.
- **So the work is:** nothing here. T2/T3 are the fix.
- **Effort:** — (validation only)
- **Done when:** after T2, the **same video** re-split as **Burst** looks right
  against today's output, and Word mode's karaoke highlight lands on the spoken
  word. Cheapest end-to-end proof that Track 1 worked, and it needs no new UI.

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
- **Starting point (corrected 16 Oct):** the waveform is **already drawn** (`crates/concat/ui/primitives/wave.slint`, used by `ui/timeline/lanes.slint:184` and the media pane). What does not exist is the waveform under the *caption* row, and any drag on a *word* edge.
- **Effort:** M
- **Done when:** dragging a word edge moves the highlight and the timing, and the edit survives export.

### E4. Enter splits a caption at the caret
- **Why:** in CapCut — and in VTS, per the owner — pressing **Enter** while
  editing a caption cuts it in two at the cursor: the tail becomes a new cue with
  its own timing. In kconcat, Enter is a no-op. The Sections row editor is a
  single-line `Field` whose `TextInput` binds only `edited`
  (`crates/concat/ui/primitives/modal.slint:615`, `:634`), and `Field` declares no
  `accepted` at all — although the same UI uses that callback elsewhere
  (`ui/primitives/number-field.slint:138`, `ui/timeline/tabs.slint:159`). The
  handler behind the row
  (`crates/concat/src/panes/sections.rs:680 edit_row_text`) writes
  `style.content` and stops.
- **What already exists to build on:** the caret is readable —
  `cursor-position-byte-offset`, with a working precedent in the same codebase
  (`crates/concat/ui/dialogs/speech.slint:364` reads it, then re-seats the caret
  with `set-selection-offsets`); splitting exists for **time**, not for text
  (`Command::SplitClips`, `crates/concat-project/src/commands/mod.rs:513` — it
  clones text *and* words whole into the tail; it does not re-partition them);
  and the rule for deriving a piece's text from the words it keeps already exists
  in the trim path (`commands/clips.rs:840–885`, "its text becomes those words").
  With no word timings, the character-proportional estimate used by C1 applies
  (`captionmap.rs:737`).
- **Approach:** let `Field` (or a caption-only variant) forward
  `accepted(text, byte-offset)`; map the caret to the word it falls in; split time
  = that word's start with real words, or the proportional estimate without them;
  issue one `Batch` of `UpdateClip` (head: text, shortened duration, `words`
  before the caret) + `AddTextClip` (tail: remaining text, `words` from the
  caret), so it is a single undo step.
- **Effort:** M
- **Depends on:** T2 only for the boundary to land on the right word when word
  timings are real; without T2 it is no worse than the estimate already in use.
- **Done when:** Enter mid-caption yields two rows with the right text and the
  right boundary, one undo restores it, and the karaoke highlight of both halves
  still lands on the spoken words.

---

## 5. Track 4 — Finishing the job (vertical + export)

### V1. Auto-reframe 16:9 → 9:16
- **Why:** the biggest single thing you'd otherwise open CapCut for — and CapCut moved it to **Pro-only** in April 2026. You already have the hard part: RVM carries recurrent state frame-to-frame "so an outline does not flicker" (`segment.rs`).
- **Approach:** per-frame mask centroid → smooth (one-euro or Savitzky-Golay) → drive crop keyframes. Keyframes, crop and the curve editor all exist.
- **Effort:** M
- **Done when:** a talking-head 16:9 clip becomes a framed 9:16 export with no manual keyframing, and the crop does not jitter.

### V2. Platform export presets + loudness
- **Why:** removes the last reason to open CapCut for a simple export. *(Verified 16 Oct: no platform presets exist — the export dialog has three quality cards, `ui/dialogs/export.slint:137`, and encoder presets like `veryfast`, `panes/export.rs:584`.)*
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
- ~~Verify V2's starting point~~ — **done (16 Oct):** no platform presets exist; only the three quality cards (`ui/dialogs/export.slint:137`).

---

## 9. Backlog — explicitly last: Japanese romaji / karaoke

*Discussed 16 Oct 2026, deliberately parked: the per-word machinery it needs is
the machinery Track 1 is still fixing, and karaoke built on ±300 ms timings would
look broken twice.*

**It is three problems, not one:**

| # | Problem | Difficulty |
|---|---|---|
| 1 | Kana → romaji, katakana → hiragana | easy — a table, no dictionary |
| 2 | Kanji → reading (kana, then romaji) | medium — needs a dictionary + morphological analyser; readings depend on context |
| 3 | Timing per **mora** on a *sung* track | hard — this is the whole feature |

**Two facts that decide the design:**

- **There is music.** Whisper over a mix produces noise; the standard pipeline
  separates vocals first (Demucs / UVR5, ~300 MB) and aligns to the isolated
  vocal.
- **The lyrics are known.** For karaoke the task is not recognition but **forced
  alignment** — sticking known text to seconds. That is the reverse direction of
  how kconcat works today (ASR → text), and it is both more accurate and cheaper.

**A ladder, cheapest first (each level stands alone):**

| Level | What | Effort | What it buys |
|---|---|---|---|
| L0 | try whisper.cpp's **`--dtw`** presets (e.g. `large.v3.turbo`) on a Japanese cue | ~0 — an argument, not a feature | better per-token stamps from the toolchain already in use |
| L1 | kana→romaji table + katakana→hiragana | S | romaji is correct only while the text stays kana |
| L2 | dictionary + morphology, offline (~50 MB; a Rust crate such as lindera/vibrato + IPADIC/UniDic) | S–M | romaji for kanji **and** word boundaries; check the dictionary's licence; the reading may still differ from what is sung |
| L3 | a Japanese Whisper fine-tune — **kotoba-whisper has ggml conversions that run under whisper.cpp** (~780 MB q8) | S | much better Japanese text before any conversion |
| L4 | vocal separation (Demucs / UVR5) | M | required for any real song |
| L5 | forced alignment per mora (e.g. a Japanese wav2vec2 CTC) | M–L | karaoke that follows the singing instead of guessing |

**What kconcat already gives this** (verified in the code): per-character layout
units exist (`captionmap.rs:650 text_tokens`) with a per-unit timing split — so
the highlight **already** steps per character, with *interpolated* timing rather
than sung timing; Japanese fonts ship with the app; and multi-track captions
exist, so a romaji line is simply a second caption track — no new rendering. Two
gaps to know about: Whisper returns a whole Japanese cue as **one word**
(`transcribe.rs:740`), and the SRT/VTT importer **drops inline per-word
timestamps** (`subtitle.rs:184`; `Cue` is `{start, end, text}`) — so external
karaoke timing cannot enter the app today.

**Honest limits:** furigana (small kana above each kanji) is a new rendering job;
romaji as a second line is not. Without L5 the highlight drifts against the
singing — the professional standard for hand-timing is Aegisub's `\k` tags, and
neither CapCut nor kconcat has anything like this today. Songs are the most
claim-sensitive content on the platforms: fine for private practice, a
consideration for publishing.
