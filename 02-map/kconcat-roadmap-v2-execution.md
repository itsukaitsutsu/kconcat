# kconcat roadmap v2 — execution spec (16 Oct 2026)

*Supersedes `02-map/kconcat-roadmap.md` and `kconcat-roadmap-addendum-2026-10-16.md` once the owner approves. Written so an AI coding session can execute one task at a time from this file plus `AGENTS.md`.*

**Scope:** a long episode (a 10–120 min talking-head, screen, or podcast recording) becomes captioned, silence-trimmed, styled 45–90 s vertical shorts, plus one clean full export. Offline. Personal tool.

---

## 0. How an AI session uses this file

1. Read `AGENTS.md` (repo root) and this file. Read **only** the manual section named by the task (`03-manuals/A-word-timing-level1.md` for T1–T3). Do not paste `05-background/`.
2. Pick **one** task from §2–§5. Check its "Needs" line; if a decision in §1 is still open, stop and ask.
3. Verify before you change: grep the rule you are touching across all crates (`AGENTS.md` §Invariants). Read the UI path, the handler, and a comparator grep before calling anything "missing".
4. Write the failing test(s) listed in the task first. Show them failing.
5. Implement. Keep the change inside the task's "Files" line.
6. Run `cargo test -p <touched crates>` and `cargo clippy -p <touched crates> --all-targets -- -D warnings` from `src/`. If the UI changed, run `cargo check -p concat`. Crates that link FFmpeg (`concat-media`, `concat-render`, `concat-export`, `concat-host`, `concat`) are **not verifiable in the sandbox** unless FFmpeg dev libraries are installed: say so, do not claim pass.
7. Update the task's status in this file (plan status and code status are separate) and the checkpoint. A fix is recorded here in the .md; the code status changes only when code is merged and tested.

**Paths.** Every code path below is relative to `src/` (the Cargo workspace root). Example: `src/crates/concat-speech/src/transcribe.rs`. Line numbers come from upstream docs at `1aece2d` and may have drifted; grep before editing.

---

## 1. Decisions (owner must settle; default applies if the owner says nothing)

| ID | Question | Default until owner decides | Blocks |
|---|---|---|---|
| **D1** | **CLOSED (owner confirmed 16 Oct 2026).** Timing reference is the **30 s** Whisper sync clip. The 60 s figures in upstream docs (`A-word-timing-level1.md` §7, `caption-sync-fix-plan.md` §2) are not used for T2 acceptance. | 30 s | — |
| **D2** | **APPROVED WITH CONDITION (owner, 16 Oct 2026).** Add T8 (Indonesian ASR tuning) and a WER output on T1, but only where they serve the goal: getting closer to CapCut Pro. Fine-tuning is not approved. | T8 and WER **approved**, scoped as in §2 T8. Fine-tune **not approved** unless the T8 bake-off shows the best model still fails. | T8 |
| **D3** | **SELECTED (owner, 16 Oct 2026): (a) `[motion]` block in `preset.toml`.** Schema not yet written. C2 preset files wait for the schema, because (a) changes their shape. | (a) | C5, C2 (preset files) |
| **D4** | **SELECTED (owner, 16 Oct 2026): B: new pure crate `concat-timing` (no dependencies).** Holds the snapper (T2/T3) and the energy helpers `rms` and `linear_from_db`, moved from `concat-media/src/silence.rs` so there is one copy. `concat-speech` calls it after `refine_segments_with_vad`. `concat-media` depends on it for the helpers. | B | T2, T3 |
| **D5** | **SELECTED (owner, 16 Oct 2026): B.** Split in two. (1) The Whisper cache (`whisper-cache.json`) is made once by an example binary in `concat-speech`, on the owner's machine (it needs Whisper and FFmpeg). (2) The metric runner (Tier 1 table, WER) is an example binary in `concat-timing`, which reads the cache and `audio-16k.f32` and runs in the sandbox, using the same energy code as the snapper. | B | T1, T8 |
| **D6** | **SELECTED (owner, 16 Oct 2026): typed lanes.** Timeline lanes are V (video, caption, image) or A (audio). Today a lane is untyped (`model.rs` `Track`: 'Deliberately untyped: any media goes on any track'). This changes that design. Legacy tracks need a rule to get a type on load. | typed | E5, P3, C5 (caption placement) |
| **D7** | **SELECTED (owner, 16 Oct 2026): one caption lane per speaker, linked to video and audio.** Each speaker's captions sit on their own caption lane (typed V, per D6). Captions, video, and that speaker's audio are linked (E5). Replaces the per-word colour idea. Overlap is shown by two lanes at once, not by one shared lane. | selected | E5, P3, T5 (caption identity changes: a caption follows its speaker's audio, not only the video) |
| **D8** | **SELECTED (owner, 16 Oct 2026): lane numbers with video on top.** V1 = video. Caption lane of speaker 1 = V2, speaker 2 = V3. Audio lanes: speaker 1 = A1, speaker 2 = A2. Each speaker's caption lane follows that speaker's audio (D7). | selected | E5, D7, T5 |

---

## 2. Track 1 — word timing and correctness (foundation)

**Dependency order (fixed):** T1 → T2 → T3 → T5 → T7 → C1 validation → E4 → E1. T8 runs in parallel after T1 (it measures text, not timing). Nothing in Track 2 starts while word timing is unmeasured.

**Rule for adding work (owner, 16 Oct 2026):** a task is added only if it is logical and needed for the goal of getting closer to CapCut Pro. Each task must name the CapCut-parity item it serves or the measurement it needs.

### T1. Corpus and harness — **first**
- **Plan status:** fixed in plan (16 Oct 2026). **Code status:** not started.
- **Needs:** D5 (B, selected). D1 closed.
- **Location (D5-B):**
  - Cache: an example binary in `src/crates/concat-speech/examples/` runs Whisper once over the corpus and writes `06-corpus/whisper-cache.json`. Owner runs it on the machine that has FFmpeg and Whisper.
  - Metric runner: an example binary in `src/crates/concat-timing/examples/` reads `06-corpus/whisper-cache.json` and `06-corpus/audio-16k.f32`, and prints the Tier 1 table and WER. It runs in the sandbox.
  - `06-corpus/srt_report.py` stays as an SRT-only structural report. It is not the metric runner.
- **Goal:** print the Tier 1 table for any run in seconds, and save a **baseline before any code change**.
- **Inputs (corpus, all in `06-corpus/`):**
  - the 5-minute video's decoded audio: `audio-16k.f32` (16 kHz mono, little-endian f32). Decode once with FFmpeg.
  - `whisper-cache.json`: Whisper output for the corpus, produced once. Schema:
    ```json
    {"schema": 1,
     "audio": {"file": "...", "sample_rate": 16000, "channels": 1, "duration_s": 300.0},
     "segments": [{"start": 0.0, "end": 4.2, "text": "...",
                   "words": [{"text": "...", "start": 0.12, "end": 0.40}]}]}
    ```
  - `06-corpus/user-video.srt` already exists and is the text reference for Tier 2.
- **Reference clip (D1):** the 30 s clip is cut from the corpus, with its own `whisper-cache` slice. It is the Tier 2 sample.
- **Holdout (fixes the old "2–3 segments" vs "set one aside" conflict):**
  - Pick **3** known-bad segments in the reference clip.
  - **Hold out 1** before tuning. Never tune on it. Open it only for the final number.
  - Tune on the other 2.
- **Tier 1 outputs (no labels):**
  - Onset plausibility rate (% of words with an energy rise within ±150 ms of their start). **Target: none yet.** Record the baseline first; set a target only after a baseline exists.
  - Monotonicity violations (reversed or overlapping words). **Target: 0.**
  - Outliers (words outside their cue, negative duration, duration > 2 s). **Target: 0.**
  - Δ vs baseline (words moved, direction, distance). A uniform shift is an offset bug.
- **Tests to write first:** a run on the synthetic two-word fixture reports 0 violations; a deliberately reversed pair is counted as 1 violation.
- **Done when:** the runner prints the Tier 1 table for `baseline.json`, and `06-corpus/runs/baseline.json` is saved before T2 starts.

### T2. Word-onset snapping (Level 1)
- **Plan status:** fixed in plan. **Code status:** not started.
- **Needs:** T1 baseline saved, D4 (B, selected).
- **Location (D4-B):** `src/crates/concat-timing/` (new crate, no dependencies). The pure snapper takes plain slices and plain word structs; `concat-speech` maps its `HeardWord` to and from them. The call goes after `refine_segments_with_vad` (`src/crates/concat-speech/src/transcribe.rs`, private fn near line 1068), on the same samples. Add the crate to the workspace; check `src/Cargo.toml` members first.
- **Helpers:** move `rms` and `linear_from_db` (`src/crates/concat-media/src/silence.rs` near lines 71 and 80) into `concat-timing`, and make `concat-media` depend on `concat-timing` for them. Their existing tests stay in `concat-media`; those tests need FFmpeg, so run them on the owner's machine.
- **Sandbox:** the `concat-timing` tests run in the sandbox, because the crate has no FFmpeg dependency.
- **Algorithm:** `A-word-timing-level1.md` §2, Steps 1–5. Parameters from its "Starting parameters" table. Do not invent new parameters; tune only the ones marked "tune by".
- **Collision rule (fixes the open question in the manual):** after snapping, if a word's start is earlier than the previous word's (final) start, **revert that word to Whisper's value**. Do not clamp it. Check the monotonic invariant again after the revert.
- **Tests to write first:**
  1. A word after a clear energy rise moves to the rise (within one hop).
  2. A word with no rise in the search window keeps Whisper's value.
  3. A snap that would reorder words is reverted, and the result is monotonic.
  4. A snap never moves a word outside its segment.
  5. Every word carries a marker: `whisper` or `snapped`.
  6. On a speed-changed clip, snapping runs on the window clock before the speed division (`A-word-timing-level1.md` §5).
  7. Snapping never runs on a VAD-compacted buffer (add a test that fails if it does).
- **Acceptance (on the 30 s reference clip, held-out segment excluded, then the held-out segment checked once):**
  - after a gap ≥ 80 ms: median |error| < 60 ms, p90 < 120 ms
  - mid-phrase: no worse than baseline
  - mean signed error within ±15 ms
  - Tier 1: 0 monotonicity violations, 0 outliers
  - Karaoke frame check (30 fps): under about 5% of frames on the wrong word
- **Do not:** change the VAD pass (`refine_segments_with_vad`). Its word clamp (lines 1108–1111) stays until T2 replaces it.

### T3. Level 1.5 — ZCR, then spectral flux
- **Plan status:** fixed in plan. **Code status:** not started.
- **Needs:** T2 accepted.
- **Step 1, zero-crossing rate:** no dependency; add it as a second onset signal in T2's search.
- **Step 2, spectral flux:** 4 Goertzel bands, **starting guess** 300–800, 800–1500, 1500–3000, 3000–6000 Hz. Record the final edges in `A-word-timing-level1.md` §3 once chosen.
- **Tests first:** on a synthetic fricative-onset fixture, the snap moves to the fricative; on a pure tone, flux stays near zero.
- **Acceptance:** mid-phrase median improves against the T2 result. If it does not, stop and record "no gain" in the manual. Do not tune past that point without a model (Level 2 is out of scope for now).

### T4. Script-aware tokenisation
- **Plan status:** fixed in plan. **Code status:** not started.
- **Needs:** none.
- **Fact:** word splitting uses ASCII whitespace (`transcribe.rs` near line 742, cited by `caption-sync-fix-plan.md` §1). Thai, Japanese, Chinese, Lao, and Khmer collapse a sentence into one word.
- **Change:** split on Unicode alphanumeric boundaries (`is_alphanumeric`, not `is_ascii_alphanumeric`), following `captionmap` (`AGENTS.md` invariant 5). A true Thai word segmenter is **not** part of this task; say so in the result.
- **Tests first:** a Japanese cue yields more than one word unit; the existing `clean_words` tests still pass.
- **Done when:** both pass, and the limit above is written in the task result.

### T5. Caption identity — store it, don't derive it
- **Plan status:** fixed in plan. **Code status:** not started.
- **Needs:** none (can run in parallel with T2 if a different author).
- **Bug:** a caption that loses all its words through an autotrim stops being a caption (`caption-identity-bug.patch`, three tests). Effects: "Burn captions" OFF still burns it in (`export.rs` near line 643), it blocks caption lanes (`clips.rs` near line 1282), and the Sections panel stops counting it (`sectionlist.rs` near line 189).
- **Change:** a stored field set at creation (`caption_of`, or an explicit flag; pick one and record it), read by `Clip::is_caption()`. Every site calls the predicate.
- **Migration:** projects saved before the field must keep loading. Serde default means "derive it the old way", so old files behave as before until edited.
- **Tests first:** the three tests in the patch. Add one: a project file without the field loads and derives identity as before.
- **Done when:** the three tests pass, and a grep shows **one** definition of caption-ness in the tree (`AGENTS.md` invariant 2 lists the sites to check).

### T6. Per-track nudge offset
- **Plan status:** fixed in plan. **Code status:** not started.
- **Needs:** T5.
- **Change:** ±10 ms steps, stored on the caption track so it survives re-transcription.
- **Test first:** a shift is applied, and survives regeneration.

### T8. Indonesian ASR tuning (approved, D2)
- **Plan status:** fixed in plan (16 Oct 2026, owner approved with condition). **Code status:** not started.
- **Why it is needed for the goal:** caption text is what the viewer reads. CapCut's auto-caption is cloud and generic. A tuned local model is the one place kconcat can beat it on Indonesian speech, and WER is the only way to show it.
- **Needs:** T1 harness (for the WER output). Does **not** depend on T2: WER measures text, T2 measures timing. They can run in parallel, but they share the 30 s clip, so record both on the same run.
- **Reference text:** WER needs a correct transcript of the reference clip. The existing `06-corpus/user-video.srt` is a generated file, not verified ground truth. Owner corrects or confirms the 30 s clip transcript first (corrections are small: pick the 2–3 bad segments, nudge the text, don't retype everything).
- **Steps (in this order, stop when a step is good enough):**
  1. Add a WER output to the T1 harness (word-level, normalised: lowercase, punctuation stripped, one agreed rule for numbers and filler words, recorded in the manual).
  2. Run each installed model in the ladder on the 30 s clip; record WER and time per run.
  3. Tune the parameters that already exist (VAD filter, beam size, the Prompt field that already ships end to end). One change at a time; holdout rule applies (one segment set aside).
  4. Choose the best model/setting that fits the machine's speed. Record the choice in the manual.
  5. Fine-tuning only if step 4 still leaves WER clearly above what the caption needs. That is a separate project (L) and needs its own approval.
- **Tests first:** WER on a fixture with known errors returns the expected value (substitution, deletion, insertion); normalisation gives the same result for "Ya," and "ya".
- **Acceptance:** owner sets the WER target after step 2 (a baseline first). No target is invented here.
- **Do not:** change the default model for users until the owner approves step 4's choice.

### T7. One writer for text and words (new in v2; required before E4 and E1)
- **Plan status:** fixed in plan (added 16 Oct 2026). **Code status:** not started.
- **Needs:** T5.
- **Bug (suspected, reasoned from code, not yet tested):** inline text edit in Sections writes `style.content` and leaves `clip.words` stale (`src/crates/concat/src/panes/sections.rs` near line 680, `edit_row_text`). `locate_words` (`captionmap.rs` near line 241) then returns `None` for words that no longer appear, and the karaoke highlight drops them.
- **Change:** `Command::UpdateClip` is the single writer for a text patch. It keeps text and words consistent:
  - content unchanged → do nothing;
  - same token count → re-text the words in position, **keep their timings** (fixing a typo must not lose karaoke);
  - otherwise → keep only the words that still appear, in order (prune). Never invent new words.
- **Tests first (one per case):** same tokens; one word changed (timing kept); one word deleted (remaining words keep their timings, no invented word).
- **Done when:** the three tests pass, and the sync matrix (§2, "Sync matrix") passes for the inline-edit path.

### Sync matrix (every edit path that touches a caption must pass all five)
1. `content` equals the words joined, or the words are absent by design, never stale.
2. Words are ordered, non-overlapping, and inside the cue's span.
3. `is_caption()` is the same before and after the edit.
4. `locate_words` finds every word in the content.
5. Clip span, cue span, and words agree once the edit settles.

Applies to: trim, autotrim, playhead split, Enter-split (E4), re-split (C1), inline text edit (T7).

**Track 1 exit:** word times you can trust, measured against the baseline; a rule that never silently loses caption identity or orphans highlight words.

---

## 3. Track 2 — caption display

### C1. Word chunking — **already built**
- **Status:** built (four layout modes, Re-split). Verified against upstream docs; the code path is `src/crates/concat/ui/dialogs/captions.slint` (the file exists on `main`).
- **Task:** validation only, after T2. Burst (3 words) on the same episode, before and after T2, compared by eye and by the Tier 1 table. **No new UI.**

### C2. Caption presets + group tag
- **Plan status:** fixed in plan. **Code status:** two caption-like presets exist (`concat.bold-caption`, `concat.lower-third`); none sets `highlight_color`, position, or motion.
- **Needs:** none for the presets that do not use motion.
- **Change:** eight caption-shaped presets (data only, under `src/crates/concat/text-presets/`), tagged as a group so they are not mixed with title looks. Names and parameters: `03-manuals/B-caption-styles.md` §4.
- **Test first:** a preset with `highlight_color` round-trips through `preset.toml`.
- **Done when:** applying one preset to a caption track gives a caption-shaped result without the inspector.

### C3. Wire karaoke to presets
- **Plan status:** fixed in plan. **Code status:** renderer wired; no preset turns it on.
- **Needs:** C2, T2 (karaoke is only as good as the timings).
- **Test first:** a karaoke preset produces one highlight per heard word (the existing renderer test `a_word_timed_caption_gets_one_highlight_per_heard_word` covers the renderer).
- **Done when:** picking a karaoke preset works without the Advanced settings.

### C4. Uppercase / text transform
- **Plan status:** fixed in plan. **Code status:** absent (`TextStyle` has no transform field; `to_uppercase` hits are UI labels only).
- **Change:** a transform field in the style model. Applied at render time, never to the stored text, so the transcript keeps its casing.
- **Test first:** a preset with uppercase renders uppercase, and the stored caption text is unchanged.

### C5. Caption motion — D3 selected (a), schema approved (16 Oct 2026)
- **Plan status:** fixed in plan (schema v1 approved by owner, 16 Oct 2026). **Code status:** absent. **Code status:** absent (confirmed 16 Oct 2026: the caption dialog has no motion control; no preset sets motion; the `pop`/`slide`/`zoom` names in `concat-effects/packages/` are video effects).
- **Do not start** before C5's place in the plan order (after D3 schema, which is now approved). Then: write one test that a preset yields motion on apply and the choice survives a project reload.
- **Schema v1 (approved, 16 Oct 2026):** motion modes none / pop / word-reveal / typewriter, `duration` (pop and word-reveal), and `scale_from` (pop only). Reads the per-word reveal order already baked (`src/crates/concat-core/src/shader.rs`). Typewriter timing comes from each word's spoken start and end.

**Motion schema v1 (D3-a). APPROVED by owner, 16 Oct 2026. Owner decisions on the open questions are recorded below.**

*Facts it rests on (checked 16 Oct 2026):*
- `TextStyle` (`src/crates/concat-project/src/model.rs`, near line 980) has no motion field. `highlight_color` is the karaoke accent and stays separate.
- Preset files are parsed by `PresetFile` (`src/crates/concat/src/presets.rs`, near line 71). It has `style` and top-level keys, and **no `deny_unknown_fields`**, so an unknown `[motion]` block is silently ignored by older builds. This is safe, but must be tested.
- The per-word reveal exists in the engine: `RevealMap` (`src/crates/concat-core/src/shader.rs`, near line 222) bakes each word's order into a map, and the shader compares it with `progress`. **Not verified:** what drives `progress` for a caption track today.

*Proposed preset block (v1, smallest set that gives a CapCut-style pop and reveal):*
```toml
[motion]
mode = "none"       # none | pop | word-reveal | typewriter
duration = 0.18     # seconds per word for pop / word-reveal (not used by typewriter)
scale_from = 1.15   # pop only: starting scale of each word
```
*Meaning of each mode (agreed definition for the preview and the engine):*
- `none`: the whole cue is on screen from its start. Only the highlight moves.
- `word-reveal`: the cue starts empty; each word appears at its spoken time and stays.
- `pop`: each word appears at its spoken time with a scale-in from `scale_from` to 1 over `duration`.
- `typewriter`: letters of each word appear one by one across the word's spoken time (start to end). `duration` is not used.

*Stored on the clip too*, as a field on `TextStyle` with `serde(default)` so older projects load as `mode = "none"`. The preset copies its block into the clip on apply. This is what lets "the choice survives a project reload" pass.

*Rules:*
- Clamp `duration` and `scale_from` in the one place where `TextStyle` values are clamped (`AGENTS.md` invariant 6). Do not clamp elsewhere.
- Motion is timed on the word timings. Until T2 is accepted, pop will land on the wrong word by the same ±200–500 ms as the highlight.
- `typewriter` is in v1 by owner decision (16 Oct 2026). Its tests must be the same as pop and word-reveal (apply, reload) plus one check that letters appear in order within each word.

*Owner decisions (16 Oct 2026):*
1. v1 has four modes: none / pop / word-reveal / typewriter. Owner approved typewriter for v1 on 16 Oct 2026 (earlier plan had it deferred to v2).
2. The Motion drawer shows for **both captions and titles** (Inspector → Text tab).
3. Default `duration` = 0.18 s, accepted for now. Tune after T2 is accepted.

**Track 2 exit:** a caption track that looks like the CapCut Pro reference, without the inspector, once T2 is accepted.

---

## 4. Track 3 — editing

### E1. Transcript editing — delete a word, the video cuts
- **Plan status:** fixed in plan. **Code status:** not started. `captionmap.rs` does the inverse (remaps words through a cut), so about 80% of the machinery exists.
- **Needs:** T2 accepted (deleting a word is guesswork at ±300 ms), T7.
- **Tests first:** deleting one word ripples audio, picture, and captions together; deleting a sentence keeps the sync matrix; one undo restores everything.
- **Decided (owner, 16 Oct 2026):**
  - Gap after a delete is **closed (ripple)**: later audio and picture move left.
  - Changing a word's text (typo fix) is **text only**. The video is not cut and the word's timing is kept. Owner: do not add cut-on-retype, because it risks new bugs in a working path.
  - Existing timeline remove-selection stays as it is. E1 adds word-level delete on top of it. It does not replace it.
  - **Control (owner, 16 Oct 2026):** one checkbox, **"Potong video saat kata dihapus"** (Cut video when a word is deleted). On: deleting a word ripples audio, picture and captions. Off: only the word's text is removed, nothing else changes. The checkbox applies to both the Sections pane and the timeline caption row, through the same delete path (T7).
  - Remove-selection on the timeline is unchanged.
- **Done when:** the tests pass and the sync matrix passes for the delete path.

### E2. Filler-word removal (Indonesian)
- **Plan status:** fixed in plan. **Code status:** not started. English-only in CapCut.
- **Needs:** E1.
- **List (starting):** eee, anu, kayak, gitu, apa ya, terus. Owner may add or remove words; record the final list in the manual.
- **Done when:** a one-click pass removes them inside a selected caption run, and leaves them alone when they are meaningful. Test with at least one negative case.

### E3. Waveform under the caption row + word-edge drag
- **Plan status:** fixed in plan. **Code status:** the timeline waveform exists (`src/crates/concat/ui/primitives/wave.slint`). Caption-row waveform and word-edge drag do not.
- **Needs:** T2, E1 recommended.
- **Done when:** dragging a word edge moves its highlight and timing, and the edit survives export.

### E4. Enter splits a caption at the caret
- **Plan status:** fixed in plan. **Code status:** not built. Enter is unwired: the caption row editor binds `edited` only (`src/crates/concat/ui/primitives/modal.slint` near lines 615 and 634; `Field` does not declare `accepted`). The handler `edit_row_text` (`sections.rs` near line 680) writes `style.content` and stops.
- **Needs:** T7 (the single writer), T2 for the boundary to land on the right word. Without T2 it is no worse than the character-proportional estimate already in use.
- **Approach:**
  - Forward `accepted(text, byte-offset)` from the field (or a caption-only variant).
  - Map the caret to the word it falls in.
  - Split time = that word's start (real words) or the proportional estimate (no words).
  - One `Batch` of `UpdateClip` (head: shortened text and duration, words before the caret) plus `AddTextClip` (tail: remaining text, words from the caret). One undo step.
  - Existing pieces: caret readable via `cursor-position-byte-offset`, precedent at `src/crates/concat/ui/dialogs/speech.slint` near line 364; time split `Command::SplitClips` (`commands/mod.rs` near line 513) for the time axis only; the trim-path text rule (`commands/clips.rs` near lines 840–885).
- **Tests first:** Enter mid-caption gives two rows with the right text and boundary; one undo restores; the karaoke highlight of both halves lands on the spoken words.
- **Done when:** the tests pass and the sync matrix passes for the split path.

---

### E5. Link / unlink audio and video (replaces Detach / Reattach)
- **Owner decision (16 Oct 2026):** replace Detach / Reattach with Link / Unlink. Every video sits on the **video line**; its audio sits on the **audio line**; the two are linked. Lanes are typed V / A (D6). Reference: Adobe Premiere Pro (Link, Unlink, relink) and the owner's VTS, where audio, video and captions are separate.
- **Parity item served:** audio on its own line, movable separately. Owner's reference, not a CapCut feature claim (not verified against CapCut in this session).
- **Plan status:** fixed in plan (owner decision 16 Oct 2026). **Code status:** not started. Detach / Reattach exists today (`studio.rs` near lines 7151–7156 `sound_placement`, 9842–9855 context menu, 10495–10590 handlers); the model has `muted` and `detached_from` (`concat-project/src/model.rs` near lines 1356–1361).
- **Current gap (verified by reading):** after Detach, the audio clip moves on its own and the link is only used by Reattach (`studio.rs` near line 4272). There is no lock that keeps video and audio moving together.
- **Behaviour to build:**
  - **Link:** select a video clip and an audio clip, right-click, Link. They form one group. Moving, trimming, splitting or deleting one acts on the group while linked.
  - **Unlink:** right-click a linked clip, Unlink. Both clips stay on their lines and move independently. Link can be applied again at any time.
  - **Linked selection:** selecting one clip of a group selects the group (as Premiere does) so a group move is one click.
  - **Captions** (D7): each speaker's caption lane is linked to the video and to that speaker's audio. Moving the group moves its captions. Today captions follow the video clip (`captions_of`, `captionmap::remap`); with D7 they also follow the speaker's audio.
  - **Remove** the Detach and Reattach menu items. Their replacement is Link / Unlink only.
- **Lane typing (D6, typed):** a legacy untyped track needs a type on load. Rule to decide in the diff: type it by the clips on it; a track with both video and audio is split or flagged. Test with an old-project fixture.
- **Import default (E5-b, decided by owner, 16 Oct 2026): yes.** Each imported video becomes two linked clips: video on V1, its audio on A1. The owner's words ("every video on the video line, audio on the audio line") read as: import always creates two linked clips. Confirm before coding.
- **Migration (required):** projects saved with `detached_from` must load. Each detached pair becomes a linked group, and `muted` on the video is kept. Test with an old-project fixture.
- **Needs:** T5 (captions must follow the group; `captions_of` is the identity this relies on). Place after E1 in plan order. Does not touch word timing, so it does not wait for T2.
- **Tests first:**
  1. Link video + audio; move the video; the audio moves by the same offset.
  2. Unlink; move the video; the audio stays put.
  3. Relink; the group moves again.
  4. A caption follows its video clip through link, unlink and move.
  5. An old project with `detached_from` loads as a linked group with the same timing and mute state.
  6. One undo restores each link change.
  7. Export audio of the clip still produces the same file.
- **Done when:** tests 1–7 pass; the sync matrix (section 2) passes for move, trim and split on a linked group; the Detach / Reattach menu items are gone from the code and the menu.

### E6. Select one whole lane with one shortcut
- **Owner decision (16 Oct 2026):** shortcut **Shift + L** selects every clip on the current lane, from the first to the last, and no other lane. The user does not have to zoom out or hold Shift while dragging to the end.
- **Plan status:** fixed in plan (owner decision 16 Oct 2026). **Code status:** not started. The gesture today is zoom out, or Shift-drag to the end. I did not find a lane-select command in the code I read (`studio.rs`, `timeline/lanes.slint`); verify before coding.
- **Check before coding:** Shift + L must not clash with an existing shortcut. Confirm the keymap in the diff. Confirm which lane counts as "current" (the lane under the playhead, or the last lane clicked).
- **Tests first:** Shift + L on V2 selects every clip on V2 and nothing on V1 or A1; a second press or Escape clears it; one undo restores the selection; a locked lane is skipped or refused (decide in the diff).
- **Done when:** the tests pass and the shortcut is listed in the shortcut help.

## 5. Track 4 — finishing (vertical and export)

### V1. Auto-reframe 16:9 → 9:16
- **Plan status:** fixed in plan. **Code status:** not started. RVM masks exist (`concat-vision/src/segment.rs`).
- **Approach:** per-frame mask centroid → smoothing (one-euro or Savitzky–Golay) → crop keyframes. Keyframes, crop, and curve editor exist.
- **Tests first:** a fixture with a moving subject produces a smooth crop track with no jitter above a threshold to be set when the test is written.
- **Done when:** a talking-head 16:9 clip becomes a framed 9:16 export without manual keyframing.

### V2. Platform export presets + loudness
- **Plan status:** fixed in plan. **Code status:** absent. Export has three quality cards (`src/crates/concat/ui/dialogs/export.slint` near line 137). `loudnorm` is used before transcription only.
- **Targets:** TikTok, Reels, Shorts, YouTube: aspect, bitrate, and **−14 LUFS integrated, ±1 LU**.
- **Tests first:** an exported fixture measures −14 LUFS ±1 LU; each preset yields the right aspect.
- **Done when:** one menu choice produces an upload-ready file.

### V3. Auto ducking
- **Plan status:** fixed in plan. **Code status:** absent.
- **Tests first:** a music bed drops under speech and returns after; the change is reversible.

### V4. Batch export queue
- **Plan status:** fixed in plan. **Code status:** API jobs only (`src/crates/concat-server/src/lib.rs` near line 20).
- **Done when:** five shorts export in one pass without babysitting.

### V5. Stabilisation — optional
- Only if you film handheld. Not scheduled.

---

## 6. Track 5 — long-form podcast (new in v2)

*Order: after Track 1 and E1. Each item states its own dependency.*

### P1. Transcript editing for long episodes = E1, promoted
- **Needs:** T2, T7. Same tests as E1, plus a 60-minute episode in the corpus for timing.

### P2. Loudness to −14 LUFS = V2 (loudness half), split out
- Same tests as V2.

### P3. Speaker labels (diarisation)
- **Plan status:** fixed in plan (scope decision pending). **Code status:** absent.
- **Owner decision (16 Oct 2026):** a speaker is a **label**, not a lane. The label lives on the audio clip, on each transcript word, and on each caption. It does not add a lane.
- **Speaker count (owner confirmed 16 Oct 2026):** the count is **detected automatically**, not forced. No fixed cap; tested up to 4. No empty speaker slots in the UI. (A forced 2 with one speaker splits one voice into two speakers.)
- **Recording setup (owner, 16 Oct 2026): each person has their own track.** So the speaker is the track, and no detection is needed for the main case. Speaker labels go on the audio clip of each track, on its transcript words, and on its captions. Each track is transcribed on its own, so overlapping speech keeps both voices. Caption order across tracks needs a merge step.
- **Owner (16 Oct 2026): most recordings are one mixed stream.** So auto-detection is **required**, not a fallback. The per-track path above covers only files that arrive already separated.
- **Engine choice (open):** sherpa-onnx or speakrs. Decide by a test on the 5-minute corpus against the owner's ground truth. Check disk before adding the dependency (see Disk footprint).
- **Overlap handling for now (owner, 16 Oct 2026):** manual. Overlap is not detected automatically at first. The user assigns speakers by hand.
- **Speaker captions on lanes (D7, owner 16 Oct 2026):** each speaker gets a caption lane. The per-word colour idea is dropped. Words still carry the speaker label, so each caption goes to the lane of its speaker. Manual route: select words and assign a speaker, and the words move to that speaker's lane. Depends on T7 (one writer for text and words), E1, and E4. Colour (owner clarified, 16 Oct 2026): colour belongs to the **speaker**, not to single words. Every caption of a speaker takes that speaker's colour as a whole, through the label. With one lane per speaker, the lane carries the same colour. Link groups: **one group per speaker** (owner, 16 Oct 2026): the video stays linked to every speaker group, and each speaker's audio and caption lane are linked together. Moving one speaker's captions does not move the others.
- **Scope boundary (added 16 Oct 2026):** with auto-detection off, no speaker processing runs. Manual speaker assignment is separate from detection and still works, and it creates the same speaker lanes. Changes that are NOT speaker-only and apply to every user: lane typing (D6), link/unlink replacing Detach/Reattach (E5), word timing (T2/T3), caption identity (T5), and word delete (E1). Single-speaker editing keeps one V lane for video and captions and one A lane for audio, linked, as the default.
- **Opt-in (owner confirmed, 16 Oct 2026):** detection runs only when the user asks (for example a "Detect speakers" button). It never runs on import. When the feature is off, the editing flow is unchanged: no speaker labels, no extra processing, and no effect on captions, cuts, or export. This covers single-speaker editing and simple tasks.
- **Open scope question:** labels only (on words, captions, and the clip), or also one audio lane per speaker (A1 Host, A2 Guest)? Per-speaker lanes depend on detection (or per-track files) and are a second opt-in layer, not part of the first version.
- **Existing code hint (checked 16 Oct 2026):** `model.rs` `Clip.audio_stream` lets a clip play one of several audio streams of one file. So one file with several audio streams may already work. Confirm in the app before planning import.
- **Auto-detection (16 Oct 2026): feasible, required for mixed recordings, not built.** Audio-only diarization (who spoke when) runs locally after transcription. Candidates: `speakrs` (Rust, ONNX Runtime, claims pyannote-level accuracy, crates.io v0.5.0); sherpa-onnx offline diarization (pyannote segmentation 3.0 + CAM++, no Python). The owner names each detected speaker once (for example SPEAKER_00 → Host). No per-line labelling.
- **Limits (stated, not measured here):** overlapping speech (two people talking at once) is hard. One speaker per word is lossy in those places. Audio-only detection does not use video, so "who is on screen" is out of scope.
- **Effort estimate:** M for basic auto labels (one diarization pass, map speaker turns to words by time, split captions at speaker changes, rename once). L if overlap handling and polish are needed. A new native dependency is added (ONNX Runtime and models), so its crate placement needs a decision like D4.
- **Accuracy test (next step, owner):** the owner labels about 2–3 minutes of the 5-minute corpus once, as ground truth. This is for testing only, not for daily use. Needed to choose the engine.
- **Disk footprint (checked 16 Oct 2026, partly):** sherpa-onnx path = pyannote segmentation 3.0 (about 6.6–7 MB) + CAM++ embedding (about 28 MB) = about 35 MB of models, plus the ONNX Runtime library (size not checked). speakrs model sizes were not checked. The bigger disk cost is usually Cargo build output (`target/`) for the new crates, not the models. Models can be stored outside `target/`, in the app data folder.
- **Prerequisite (answered by owner, 16 Oct 2026):** one track per person. Open next: are these separate audio files, or one file with several audio streams?
- **Approach:** a local diarisation model after transcription; output stored as a speaker tag per word or cue.
- **Tests first:** on a two-speaker corpus episode, turns are assigned; one manual correction is kept.

### P4. Screen and webcam recording
- **Plan status:** fixed in plan. **Code status:** microphone only (`src/crates/concat-host/src/record.rs`).
- **Done when:** a screen-plus-webcam take lands as two synced clips.

### P5. Long-form end-to-end check
- **Done when:** a 60-minute episode becomes three 60-second vertical clips and one full export, offline, start to finish. Count manual corrections; if the count rises after a track closes, something regressed.

### P6. Verify before planning
- **Multi-clip / B-roll:** owner reports it exists; untested. Verify in the running app before any task.
- **Caption animation:** owner reports it exists; the repo check found no caption motion (C5). Verify in the running app.

---

## 7. Out of scope — permanently

Templates, music / sticker / font libraries, cloud AI (including generated video), collaboration, cloud sync, mobile parity, 130-language counts, colour-grading depth, release and marketing work. Also: **no in-house cloud AI of any kind** (`AGENTS.md` and `working-agreements.md`).

---

## 8. Measurements that decide whether the work is real

| Check | When | Pass |
|---|---|---|
| Tier 1 table (onset rate, monotonicity, outliers, Δ) | every run | 0 violations, 0 outliers; onset rate recorded against baseline |
| Tier 2 on the 30 s reference (D1) | after T2, T3 | §2 T2 acceptance |
| Karaoke frame check at 30 fps | after C3 | under ~5% wrong-word frames |
| Sync matrix | every edit path | all five assertions |
| 20-minute recording → 60 s short | when a track closes | manual corrections do not rise |
| 60-minute episode → three 60 s clips | after Track 5 | manual corrections do not rise |

**Duration glossary (do not mix):**

| Duration | Meaning |
|---|---|
| 30 s | Whisper timing reference clip (D1). Whisper's native window is also 30 s (a model fact). |
| 60 s | Output short length (vertical clip). |
| 45–90 s | Output short range (scope). |

---

## 9. Open questions (answer before the task named)

- D3(a) `[motion]` schema fields (§3, C5), D5 (harness location) — §1.
- T8: owner's corrected transcript of the 30 s clip (needed before step 1 of T8).
- Which of the episodes are two-speaker, for P3?
- Is the room noisy enough that vocal isolation moves up? (D-track, not scheduled.)
- Do you record remote guests, which raises P4?

---

## 10. Backlog — Japanese romaji and karaoke (last)

Parked. The per-word machinery is what Track 1 is still fixing. Full design: the ladder L0–L5 in the previous roadmap §9, unchanged. Start with L0 (`whisper.cpp --dtw` presets) as an experiment, not a feature. Known gaps: Whisper returns a Japanese cue as one word (`transcribe.rs` near line 740), and the SRT/VTT importer drops per-word timestamps (`subtitle.rs` near line 184).

---

## 10b. Additional features — languages (owner, 16 Oct 2026)

*Not in the main order. Added at the owner's request: the owner also makes English and Japanese content. Start only after Track 1 exit, and do not let them delay the Indonesian work above.*

### X1. English support
- **Plan status:** fixed in plan (additional). **Code status:** not started.
- **Why:** the owner makes English content, and CapCut's filler removal is English-centred, so English is a parity point as well as a regression risk.
- **Steps:**
  1. Add one English clip to `06-corpus/` with its own Whisper cache and reference text (owner supplies the corrected transcript, as for T8).
  2. Run the T1 harness on it, once before T2 and once after. The English result is a **regression check**: a change that helps Indonesian must not worsen English.
  3. Add an English filler list (for example: um, uh, like, you know) as a separate list, not mixed with the Indonesian one. Use the same E2 machinery.
- **Tests first:** English filler list removes "um" and keeps "like" when it is meaningful (negative case).
- **Done when:** the English regression check runs on every T2/T3 run, and the English filler pass works on one caption run.

### X2. Japanese support
- **Plan status:** fixed in plan (additional). **Code status:** not started.
- **Dependency:** T4 (script-aware tokenisation) is required first. Without it, a Japanese cue is one "word".
- **Scope now:** per-word timing and captions for spoken Japanese. Romaji and karaoke stay in §10 (backlog, last).
- **Steps:**
  1. Add a Japanese clip to `06-corpus/` with its own cache and reference text.
  2. After T4, check that Japanese cues split into word units and the sync matrix passes.
  3. Measure Japanese WER only if the owner wants Japanese T8 tuning. Not planned now.
- **Done when:** a Japanese cue produces multiple word units, and the sync matrix passes on the Japanese clip.

**Rule:** X1 and X2 are added to the order only after Track 1 exit. They do not change the Indonesian priority for T8, E2, or the podcast track.

## 10c. UI bug — panels cannot scroll when resized smaller (owner-reported, 16 Oct 2026)

*Owner reference: in VTS every panel scrolls when made smaller. In kconcat the user has to reset panels to default size to see all content. Owner's priority: fix. Rule check: a panel that hides its controls at a smaller window size blocks editing on laptops, so this serves the CapCut-closer goal.*

**Code audit (read from `src/crates/concat/ui/`, 16 Oct 2026). Runtime behaviour NOT verified; resize each panel to its minimum and confirm.**

| Panel | Scroll mechanism in code | Status from code |
|---|---|---|
| Inspector (config pane: adjust, text, sound, effects) | `PaneScroll` wraps the selected-clip tabs (`workspace/config-pane.slint` near line 609) | Has scroll. The owner's report needs runtime check. |
| Sections (list) | `PaneScroll` (`dialogs/sections.slint` near line 647) | Has scroll. |
| Media | `PaneScroll` for pages (`workspace/media-pane.slint` near line 2232) | Has scroll. |
| Transitions | `PaneScroll` (`config-pane.slint` near line 496) | Has scroll. |
| Timeline | own `ThinScrollbar` / `ThinHScrollbar` (`workspace/timeline-pane.slint`) | Has scroll, its own kind. |
| **Keyframes** | none (dopesheet and curve canvases, `workspace/keyframes-pane.slint`) | **No scroll. Candidate for the bug.** |
| **Meters** | none (`workspace/meters-pane.slint`) | **No scroll. Likely fixed-size; check.** |
| Preview | one hit only (`workspace/preview-pane.slint`) | Check. |

**Steps (for the AI session):**
1. Runtime check first: resize each panel to its minimum and record which ones clip with no way to reach the hidden content. Write the result into this table (not from memory).
2. For each panel that clips, wrap its scrollable content in `PaneScroll` (`workspace/pane.slint`, the existing component). Do not add a second scroller. Keyframes canvases may need a vertical `PaneScroll` around the controls only, not around the canvas.
3. Check the `PaneScroll` contract: `content` must size to `preferred-height`, and the wheel and scrollbar must still work with the controls inside (number-field scrubbing, knobs, bezier handles must keep their drags).

**Tests / checks:**
- Slint layout check: `cargo check -p concat` after each panel change (UI bindings are generated at compile time).
- Manual check, recorded: each panel at minimum size can reach its lowest control by wheel scroll, and a drag on a number field still changes its value.

**Done when:** every panel in the table can reach all its content at minimum size, and the runtime table is filled in.

**Not in scope:** redesigning the panel layout. Only scrolling.

**Investigation result (16 Oct 2026, code reading, not run):** see `04-findings/ui-1-panel-scroll-investigation.md`. Summary: Keyframes and Meters have no scroll at all (H1). The strongest code-level suspect is that `PaneScroll` measures content height from `preferred-height`, which can be smaller than what is drawn, so overflow reads as 0 and the bar and wheel never appear (H2, unconfirmed). Commit `f1426c9` (15 Oct) changed `PaneScroll` but does not remove scrolling by reading. The owner runs the protocol in the findings file before any fix.

## 11. Plan status summary

| Item | Plan | Code |
|---|---|---|
| T1 harness | fixed | not started |
| T8 Indonesian ASR tuning + WER | fixed (approved with condition) | not started |
| T2 snapping | fixed | not started |
| T3 ZCR + flux | fixed | not started |
| T4 tokenisation | fixed | not started |
| T5 identity | fixed | not started |
| T6 nudge | fixed | not started |
| T7 one writer | fixed (new) | not started |
| C1 chunking | built | built |
| C2 presets | fixed | 2 close presets exist |
| C3 karaoke wiring | fixed | renderer wired |
| C4 case transform | fixed | absent |
| C5 motion | D3 selected (a), schema v1 approved 16 Oct (4 modes, typewriter included) | absent |
| E1 transcript edit | fixed | not started |
| E2 filler removal | fixed | not started |
| E3 waveform / word-edge drag | fixed | not started |
| E4 Enter-split | fixed | not built |
| E5 link / unlink audio and video (replaces Detach / Reattach) | fixed (owner 16 Oct) | not started; Detach / Reattach exists |
| E6 select one lane with Shift + L | fixed (owner 16 Oct) | not started |
| V1 auto-reframe | fixed | not started |
| V2 presets + loudness | fixed | absent |
| V3 ducking | fixed | absent |
| V4 batch queue | fixed | API only |
| P3 speaker labels | fixed (scope open) | absent |
| P4 screen / webcam | fixed | mic only |
| P6 B-roll / animation | verify | owner-reported |

**Order (fixed):** T1 → T2 → T3 → T4 (parallel) → T5 → T6 → T7 → C1 validation → E4 → E1 → E2 → V2 (loudness first) → C2 → C3 → C4 → V3 → V4 → P3 → V1 → C5 (after D3) → E3 → P4 → P6. **T8 runs in parallel after T1. X1 and X2 (§10b) come after Track 1 exit.** **UI-1 (§10c, panel scroll) can run in parallel with Track 1, since it touches no word-timing code.**
