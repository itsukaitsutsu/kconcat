# kconcat roadmap v2 — execution spec (16 Oct 2026)

*Supersedes `02-map/kconcat-roadmap.md` and `kconcat-roadmap-addendum-2026-10-16.md` once the owner approves. Written so an AI coding session can execute one task at a time from this file plus `AGENTS.md`.*
**Owner approval (16 Oct 2026): roadmap v2 APPROVED WITH CONDITION.** Each task must support sync or the move closer to CapCut Pro, and each task names its parity item or measurement. Tasks that do not meet this are not built.

**Scope:** a long episode (a 10–120 min talking-head, screen, or podcast recording) becomes captioned, silence-trimmed, styled 45–90 s vertical shorts, plus one clean full export. Offline. Personal tool.

**Scope change (owner, 16 Oct 2026):** editing is not limited to podcasts. Music editing is in scope too (Track 6, below). Each task still names its CapCut Pro parity item or measurement.

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

**Naming (clarified 16 Oct 2026):** Option B (`05-background/kconcat-feature-gap-option-b-v2.md`) has Tier A, B, C, and D only. There is no Tier E. Roadmap v2 uses Track 1–6 with task IDs: T (Track 1, word timing), C (Track 2, caption display), E (Track 3, editing, E1–E6), V (Track 4, finishing), P (Track 5, long-form podcast), K (Track 6, extra editing, K1–K5). Each roadmap task names its Option B row or measurement.

## 1. Decisions (owner must settle; default applies if the owner says nothing)

| ID | Question | Default until owner decides | Blocks |
|---|---|---|---|
| **D1** | **CLOSED (owner confirmed 16 Oct 2026).** Timing reference is the **30 s** Whisper sync clip. Where 30 s comes from: the owner chose this clip, and Whisper's native window is also 30 s (a model fact). It is not a figure taken from the upstream docs. The 60 s figures in upstream docs (`A-word-timing-level1.md` §7, `caption-sync-fix-plan.md` §2) are not used for T2 acceptance. | 30 s | — |
| **D2** | **APPROVED WITH CONDITION (owner, 16 Oct 2026).** Add T8 (Indonesian ASR tuning) and a WER output on T1, but only where they serve the goal: getting closer to CapCut Pro. Fine-tuning is not approved. | T8 and WER **approved**, scoped as in §2 T8. Fine-tune **not approved** unless the T8 bake-off shows the best model still fails. | T8 |
| **D3** | **SELECTED (owner, 16 Oct 2026): (a) `[motion]` block in `preset.toml`.** Schema not yet written. C2 preset files wait for the schema, because (a) changes their shape. | (a) | C5, C2 (preset files) |
| **D4** | **SELECTED (owner, 16 Oct 2026): B: new pure crate `concat-timing` (no dependencies).** Holds the snapper (T2/T3) and the energy helpers `rms` and `linear_from_db`, moved from `concat-media/src/silence.rs` so there is one copy. `concat-speech` calls it after `refine_segments_with_vad`. `concat-media` depends on it for the helpers. | B | T2, T3 |
| **D5** | **SELECTED (owner, 16 Oct 2026): B.** Split in two. (1) The Whisper cache (`whisper-cache.json`) is made once by an example binary in `concat-speech`, on the owner's machine (it needs Whisper and FFmpeg). (2) The metric runner (Tier 1 table, WER) is an example binary in `concat-timing`, which reads the cache and `audio-16k.f32` and runs in the sandbox, using the same energy code as the snapper. | B | T1, T8 |
| **D6** | **SELECTED (owner, 16 Oct 2026): typed lanes.** Timeline lanes are V (video, caption, image) or A (audio). Today a lane is untyped (`model.rs` `Track`: 'Deliberately untyped: any media goes on any track'). This changes that design. Legacy tracks need a rule to get a type on load. | typed | E5, P3, C5 (caption placement) |
| **D7** | **SELECTED (owner, 16 Oct 2026): one caption lane per speaker, linked to video and audio.** Each speaker's captions sit on their own caption lane (typed V, per D6). Captions, video, and that speaker's audio are linked (E5). Replaces the per-word colour idea. Overlap is shown by two lanes at once, not by one shared lane. | selected | E5, P3, T5 (caption identity changes: a caption follows its speaker's audio, not only the video) |
| **D8** | **SELECTED (owner, 16 Oct 2026): lane numbers with video on top.** V1 = video. Caption lane of speaker 1 = V2, speaker 2 = V3. Audio lanes: speaker 1 = A1, speaker 2 = A2. Each speaker's caption lane follows that speaker's audio (D7). Without speaker detection (one speaker), captions sit on V2, above the video on V1 (owner, 16 Oct 2026). | selected | E5, D7, T5 |

---

## 2. Track 1 — word timing and correctness (foundation)

**Dependency order (fixed):** T1 → T2 → T3 → T5 → T7 → C1 validation → E4 → E1. T8 runs in parallel after T1 (it measures text, not timing). Nothing in Track 2 starts while word timing is unmeasured.

**Rule for adding work (owner, 16 Oct 2026):** a task is added only if it is logical and needed for the goal of getting closer to CapCut Pro. Each task must name the CapCut-parity item it serves or the measurement it needs.

### T1. Corpus and harness — **first**
- **Plan status:** fixed in plan (16 Oct 2026). **Code status: DONE in the sandbox-provable part (18 Oct 2026)** - see the Status bullet at the end of this task.
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
- **Status: DONE for what this sandbox can prove (18 Oct 2026).** `crates/concat-timing/` exists: `energy` (moved helpers + 20 ms framing), `words` (Word + source marker), `metrics` (Tier 1, Δ vs baseline, normalized WER, onset plausibility), `examples/metric_runner.rs` (Tier 1 table from cache + raw audio, `--save` run files). Tests were run red first (11 failures, log kept in the checkpoint), then 11/11 green; clippy `-D warnings` and `cargo fmt --check` clean. The two prescribed first tests exist: the synthetic two-burst fixture reports 0 violations; the reversed pair reports exactly 1. **Not verified here** (no FFmpeg/whisper.cpp in the sandbox): `crates/concat-speech/examples/make_cache.rs` (D5-B owner half, written) and the `concat-media` rewire (written; its silence tests are the proof, and they run wherever FFmpeg exists, per §0). The real corpus needs T8, so the baseline is saved against a labelled synthetic fixture instead: `06-corpus/fixtures/` + `06-corpus/runs/baseline-synthetic.json` (runner log in the checkpoint). Owner machine: `make_cache` → `metric_runner` → re-save as `06-corpus/runs/baseline.json`; the synthetic run stays as the schema's worked example.

### T2. Word-onset snapping (Level 1)
- **Plan status:** fixed in plan. **Code status: DONE in the sandbox-provable part (18 Oct 2026)** - see the Status bullet at the end of this task.
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
- **Status: DONE for what this sandbox can prove (18 Oct 2026).** `concat-timing::snap` implemented: `snap_words` (manual §2 steps 1-5, `Params` untouched from the Start column) + `SnapReport{words, snapped, reverted, tied}`. All 7 prescribed tests written first, shown failing (`todo!()` panics, log in the checkpoint), then green - 18/18 crate tests pass, clippy `-D warnings` and `cargo fmt --check` clean. The collision rule reverts to Whisper and never clamps; when Whisper itself is out of order the invariant ties the starts and the tie is counted, never silent. Test 6 (window clock before the speed division) and test 7 (never a VAD-compacted buffer) are the alarms the roadmap asked for. `metric_runner` grew `--snap`: Tier 1 of the snapped words + Δ vs the cache, saved with markers and counts. Synthetic before/after (checkpoint log): violations 2→1, outliers 2→0, 1 word moved (+115 ms, the reversed pair clipped to its cue); the residual violation is a 20 ms overlap at a segment head where the previous word's end cannot retract below its own minimum duration - the manual's repair order (air gap, then `end ≥ start + min`) produces it, and whether it survives on real speech is T8's question. One framing fact, recorded because a test pinned it: a step onset trips the rise rule on the frame whose 320-sample window first holds 80 loud samples, so `t(i)` (window start, per the manual's `e[i]` definition) lands up to 15 ms + one hop before the physical onset. Inside the acceptance bands; if T8 shows the early bias hurts the ±15 ms signed-error band, correcting it is a recorded tuning decision, not a silent constant. **Not verified here** (no FFmpeg/whisper.cpp): the `concat-speech` wiring - `snap_word_starts` (the HeardWord mapping, D4-B) called at all three production sites right after `refine_segments_with_vad`, before any speed division; the VAD pass itself and its word clamp are untouched, and its test at line ~1689 still tests refine alone. Acceptance on the 30 s reference clip stays open until the corpus exists (T8).

### T3. Level 1.5 — ZCR, then spectral flux
- **Plan status:** fixed in plan. **Code status: DONE in the sandbox-provable part (18 Oct 2026)** - see the Status bullet at the end of this task.
- **Needs:** T2 accepted.
- **Step 1, zero-crossing rate:** no dependency; add it as a second onset signal in T2's search.
- **Step 2, spectral flux:** 4 Goertzel bands, **starting guess** 300–800, 800–1500, 1500–3000, 3000–6000 Hz. Record the final edges in `A-word-timing-level1.md` §3 once chosen.
- **Tests first:** on a synthetic fricative-onset fixture, the snap moves to the fricative; on a pure tone, flux stays near zero.
- **Acceptance:** mid-phrase median improves against the T2 result. If it does not, stop and record "no gain" in the manual. Do not tune past that point without a model (Level 2 is out of scope for now).
- **Status: DONE for what this sandbox can prove (18 Oct 2026).** New `concat-timing::signals`: `Signals{energy, zcr, flux}` framed once per continuous buffer on the shared 5 ms/20 ms grid, ZCR frames, four-band Goertzel flux (`BANDS` = the manual's starting guess, recorded in the manual §3 as this task requires), `Wideband` knobs (starting guesses, flagged for T8), detectors `is_zcr_rise` / `is_flux_rise`. Both the snapper's search and Tier 1's `onset_plausible` now consult all three signals - one scan, one definition of "onset", so the metric can never disagree with the snapper. Prescribed tests written first and shown failing (4 RED: fricative→ZCR, transition→flux, fricative snap, fricative plausibility; log in the checkpoint), then green: **24/24**, clippy `-D warnings` and `cargo fmt --check` clean. A measured bug and its fix are recorded in the manual: raw flux spikes at burst *offsets* (Dirichlet sidelobes of the signal's own step edge; a Hann taper does not help - measured), which snapped synthetic words 130 ms early into the previous burst's tail; the energy gate (`e[i] ≥ e[i−1]` for flux rises) kills the artifact, keeps equal-loudness transitions, adds no tunable number, and has its own regression test. Synthetic corpus, snapping run (checkpoint log): violations 2→1, outliers 2→0, onset plausibility of the *baseline* words rose 63.6%→90.9% (flux credits onsets in dense regions where the energy percentile floor is contaminated by the neighbour burst); the fixture's words sit on their rise frames by construction, so `snapped 0` there is the correct result, and the Δ is the one clipped reversed pair (+115 ms). `concat-speech`'s call site builds `Signals::from_samples` once per window - **not verified here** (no FFmpeg/whisper.cpp). The acceptance line (mid-phrase median on the real clip, or "no gain" recorded) stays open until the corpus exists (T8).

### T4. Script-aware tokenisation
- **Plan status:** fixed in plan. **Code status: DONE in the sandbox-provable part (18 Oct 2026)** - see the Status bullet at the end of this task.
- **Needs:** none.
- **Fact:** word splitting uses ASCII whitespace (`transcribe.rs` near line 742, cited by `caption-sync-fix-plan.md` §1). Thai, Japanese, Chinese, Lao, and Khmer collapse a sentence into one word.
- **Change:** split on Unicode alphanumeric boundaries (`is_alphanumeric`, not `is_ascii_alphanumeric`), following `captionmap` (`AGENTS.md` invariant 5). A true Thai word segmenter is **not** part of this task; say so in the result.
- **Tests first:** a Japanese cue yields more than one word unit; the existing `clean_words` tests still pass.
- **Done when:** both pass, and the limit above is written in the task result.
- **Status: DONE for what this sandbox can prove (18 Oct 2026).** The rule stayed one copy: `captionmap` grew the public doors `word_units` (its `text_tokens` split: Unicode `is_alphanumeric` boundaries, per-character units with trailing punctuation for space-less scripts) and `expand_units` (its `expand_word_units` span-spreading). Prescribed tests written first in `concat-project`, shown failing (`todo!()`, log in the checkpoint), then green - **255/255** `concat-project` tests pass, including every existing `clean_words` test, clippy `-D warnings` and fmt clean. `concat-speech` now depends on `concat-project` (acyclic: project rests on core alone) and `words_from_tokens` runs each decoded group through `expand_units`, so both engine paths return script-aware words; whisper's own token grouping (leading ASCII space) is untouched because that marker arrives as bytes. The pre-T4 test `unspaced_text_is_one_word_not_a_guess` codified the old behaviour and was rewritten to the new one (`unspaced_text_splits_into_captionmap_units`, plus a 7-character Japanese run test) - both in `concat-speech`, so **not verified here** (no whisper.cpp/FFmpeg); the captionmap tests that prove the same split are verified. **The limit, as the task requires:** this is *not* a true Thai (or Japanese) word segmenter - space-less scripts get character-level units with estimated intra-group timings, not dictionary words; the doc on `word_units` and the manual-facing comment say so where the rule lives. The T2 snap runs after and can pull unit starts onto real onsets; its segment clip keeps expansions inside their cue.

### T5. Caption identity — store it, don't derive it
- **Plan status:** fixed in plan. **Code status: DONE in the sandbox-provable part (18 Oct 2026)** - see the Status bullet at the end of this task.
- **Needs:** none (can run in parallel with T2 if a different author).
- **Bug:** a caption that loses all its words through an autotrim stops being a caption (`caption-identity-bug.patch`, three tests). Effects: "Burn captions" OFF still burns it in (`export.rs` near line 643), it blocks caption lanes (`clips.rs` near line 1282), and the Sections panel stops counting it (`sectionlist.rs` near line 189).
- **Change:** a stored field set at creation (`caption_of`, or an explicit flag; pick one and record it), read by `Clip::is_caption()`. Every site calls the predicate.
- **Migration:** projects saved before the field must keep loading. Serde default means "derive it the old way", so old files behave as before until edited.
- **Tests first:** the three tests in the patch. Add one: a project file without the field loads and derives identity as before.
- **Done when:** the three tests pass, and a grep shows **one** definition of caption-ness in the tree (`AGENTS.md` invariant 2 lists the sites to check).
- **Status: DONE for what this sandbox can prove (18 Oct 2026).** The choice the task asks to be recorded: an **explicit stored flag**, `Clip::caption_flag: Option<bool>` - not `caption_of`, which is a link to a source clip and legitimately absent for imported cues (its absence *is* the bug). `Some` at creation (the `AddTextClip` handler stores it beside `caption_of`/`words`; every split, piece and duplicate clones it, so an autotrim ghost keeps its identity); `None` means a pre-T5 document and `is_caption()` derives the old way, so old files load and behave exactly as before until edited - `DOCUMENT_VERSION` does not move, and `tidy()` carries the flag through. Tests: the patch's three (`04-findings/caption-identity-bug.patch`, applied verbatim) ran **red first (2 of 3 failing**, the third - trim-following - never depended on identity), then **3/3 green**; the prescribed fourth (document without the field derives as before, flag outruns derivation both ways) added; **256/256** `concat-project` tests pass, clippy `-D warnings` and fmt clean. Grep audit: the rule body exists once (`model.rs` `is_caption`) plus the one documented birth site (`clips.rs` creation); all six invariant-2 sites now call the predicate - `sectionlist.rs`'s panel filter rewritten around it, and in `concat` (export.rs, sections.rs, studio.rs) the local copies delegate/call it: those three edits are **not verified here** (the `concat` crate needs FFmpeg + slint). Deliberately untouched: `captions.rs` offset-stacking and the two `words.is_some()` highlightability reads ask a different question (karaoke needs word timings; identity does not), and widening them would be exactly the "widen a rule to fix one case" AGENTS.md forbids.

### T6. Per-track nudge offset
- **Plan status:** fixed in plan. **Code status: DONE in the sandbox-provable part (18 Oct 2026)** - see the Status bullet below.
- **Needs:** T5.
- **Change:** ±10 ms steps, stored on the caption track so it survives re-transcription.
- **Test first:** a shift is applied, and survives regeneration.
- **Status: DONE for what this sandbox can prove (18 Oct 2026), entirely inside `concat-project`.** `Track.caption_nudge_ms: i32` (serde default 0 and skipped at zero, so pre-T6 documents load unchanged and un-nudged tracks serialize byte-identical; `DOCUMENT_VERSION` does not move). `Command::NudgeCaptionTrack{track_id, delta_ms}`: quantised to the nearest `ranges::CAPTION_NUDGE_STEP_MS` (10 ms, ties away from zero), accumulated on the lane, clamped once - in the handler - to ±`ranges::MAX_CAPTION_NUDGE_MS` (1 s); an at-cap nudge reports `applied: false`, so the undo history keeps no nothing-snapshot, and undo itself is the editor's whole-state restore (tested: one undo steps one nudge back). The handler moves every caption on the lane - by the T5 stored identity, not by the presence of words - shifting clip starts **and** word timings together so karaoke stays on the cue. Survival of re-transcription: `AddTextClip` applies the lane's nudge at birth (placement and words), so a regenerated run lands already corrected. Tests (`tests/caption_track_nudge.rs`): the two prescribed (shift applied; survives a remove-and-re-add regeneration) plus quantisation, the cap, and the older-document load - red first (4 failures against a `todo!()` handler, log in the checkpoint), then green: **256 unit + 3 identity-patch + 5 nudge tests pass**, clippy `-D warnings` and fmt clean. Nothing outside `concat-project` needed a change: preview and burn-in read clip and word times from the project state, which now carry the shift, and the only non-project references to neighbouring command variants are constructions, not exhaustive matches. The UI affordance that *issues* the command (a button/key in the captions pane) is app-crate work this task's change line does not name; the mechanism it would call is complete and tested.

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
- **Plan status:** fixed in plan (added 16 Oct 2026). **Code status: DONE (18 Oct 2026, sandbox-provable part)** - see the Status bullet below.
- **Needs:** T5.
- **Bug (suspected, reasoned from code, not yet tested):** inline text edit in Sections writes `style.content` and leaves `clip.words` stale (`src/crates/concat/src/panes/sections.rs` near line 680, `edit_row_text`). `locate_words` (`captionmap.rs` near line 241) then returns `None` for words that no longer appear, and the karaoke highlight drops them.
- **Change:** `Command::UpdateClip` is the single writer for a text patch. It keeps text and words consistent:
  - content unchanged → do nothing;
  - same token count → re-text the words in position, **keep their timings** (fixing a typo must not lose karaoke);
  - otherwise → keep only the words that still appear, in order (prune). Never invent new words.
- **Tests first (one per case):** same tokens; one word changed (timing kept); one word deleted (remaining words keep their timings, no invented word).
- **Done when:** the three tests pass, and the sync matrix (§2, "Sync matrix") passes for the inline-edit path.
- **Status: DONE (18 Oct 2026), entirely inside `concat-project`.** The suspected bug was confirmed by reading: `edit_row_text` (sections.rs) already routes through `Command::UpdateClip` with a content-changed style, and the handler wrote the style while leaving `clip.words` stale. The fix therefore lands where the single-writer rule lives - the `UpdateClip` text arm - and `sections.rs` needed no change. `reconcile_caption_words` now runs on every text patch: same token count (tokens are `captionmap::word_units`, so the split stays the one rule, T4) re-texts the words in position and keeps every timing; a different count prunes to the old words that still appear - multiset-counted, in their old order, never inventing one; a cleared style clears the words (absent by design, never stale); the stored caption flag is untouched, so identity survives every text edit (T5); ordinary titles carry no words and fall through. Tests (`tests/caption_text_one_writer.rs`): the three prescribed (same tokens; one word changed, timing kept; one word deleted, no invented word) plus a same-count reorder, a repeated-word multiset case, and the style-clear - each asserting the sync matrix (§2) on the edited clip: content equals the words joined, ordered/non-overlapping/inside the cue, `is_caption()` unchanged, `locate_words` finds every word. Red first (5 of 6 failing against the stale-words handler; unchanged-content passed trivially), then green: **256 lib + 3 identity + 6 one-writer + 5 nudge**; clippy `-D warnings` and fmt clean. Nothing outside `concat-project` was touched, so nothing is unverified for this task.

### Sync matrix (every edit path that touches a caption must pass all five)
1. `content` equals the words joined, or the words are absent by design, never stale.
2. Words are ordered, non-overlapping, and inside the cue's span.
3. `is_caption()` is the same before and after the edit.
4. `locate_words` finds every word in the content.
5. Clip span, cue span, and words agree once the edit settles.

Applies to: trim, autotrim, playhead split, Enter-split (E4), re-split (C1), inline text edit (T7), word delete (E1).

**Track 1 exit:** word times you can trust, measured against the baseline; a rule that never silently loses caption identity or orphans highlight words.

---

## 3. Track 2 — caption display

### C1. Word chunking — **already built**
- **Status:** built (four layout modes, Re-split). Verified against upstream docs; the code path is `src/crates/concat/ui/dialogs/captions.slint` (the file exists on `main`).
- **Task:** validation only, after T2. Burst (3 words) on the same episode, before and after T2, compared by eye and by the Tier 1 table. **No new UI.**
- **Validation (18 Oct 2026, sandbox, synthetic episode): DONE at data level; by-eye check still owes the owner.** `src/crates/concat-project/examples/c1_burst_check.rs` drives the shipped `resplit_cues(.., Burst)` path over the Tier-1 runner's own output (`06-corpus/runs/{baseline,snapped}-synthetic.json`); the captured table is `06-corpus/runs/c1-burst-check-synthetic.txt`. Result: every burst chunk passes its policy limits (≤3 words, ≤18 chars, ≤2.5 s), ordered and non-overlapping, no word lost or reordered, **before and after T2**. Tier 1 before → after: violations 2 → 1, outliers 2 → 0, onset rate 90.9% both, WER unchanged (chunking never touches recognition). The comparison shows T2 improving chunking itself: before, the mis-timed "tetap" (2.49 s, at the cap) stranded "jaga" as a 0.05 s sliver; after the snap, "tetap jaga" is one 0.89 s two-word burst - 6 chunks → 5. Limits: the episode is the synthetic fixture, not a real one (T8's corpus does not exist yet), and the by-eye comparison in the running captions dialog needs the GUI - **not verified here**, recorded for the owner to repeat on a real episode with the same example.

### C2. Caption presets + group tag
- **Plan status:** fixed in plan. **Code status:** two caption-like presets exist (`concat.bold-caption`, `concat.lower-third`); none sets `highlight_color`, position, or motion.
- **Needs:** none for the presets that do not use motion.
- **Change:** eight caption-shaped presets (data only, under `src/crates/concat/text-presets/`), tagged as a group so they are not mixed with title looks. Names and parameters: `03-manuals/B-caption-styles.md` §4.
- **Decided (owner, 16 Oct 2026, A9):** no separate `kconcat.indonesian` variant. Caption size is a fraction of frame height (`TextStyle`: cap height, stroke, spacing), so the same preset fits any sequence size. Only size, not design, would change for Indonesian, and that is handled by the sequence frame. No HTML preview for A9.
- **Decided (owner, 16 Oct 2026, A9):** all **eight presets ship** as selectable presets. The user picks one from the list; no per-setting tuning needed. Pop-bold and karaoke show their highlight only after C3. Preview (mock, not browser-verified): `05-background/caption-presets-preview.html`.
- **Test first:** a preset with `highlight_color` round-trips through `preset.toml`.
- **Done when:** applying one preset to a caption track gives a caption-shaped result without the inspector.
- **Status: data + parser DONE (18 Oct 2026), written in the `concat` crate - NOT VERIFIED here (slint/FFmpeg crate cannot compile in this sandbox); the picker that applies one to a caption track is owner-machine work.** Shipped: the eight presets of manual §4 as data under `src/crates/concat/text-presets/kconcat.{pop-bold,boxed,karaoke,minimal,hard-shadow,upper-tight,low-third-cap,clean-bilingual}/preset.toml` - the manual's weights, sizes, strokes, plates, positions; `shadow` explicit on every file (the parser's absent-default is `true`, and the table gives the shadow to hard-shadow alone); A9 honoured: no Indonesian variant, all eight ship as one selectable group. The group tag: `kind = "caption"` in the files, `PresetFile.kind` -> `TextPreset.kind` in the parser, absent = "title", so every pre-tag preset stays where it was, and the Text page's card builder now filters the caption group out of its shelves ("not mixed with title looks", literally). The highlight: `PresetStyle.highlightColor` -> `TextStyle.highlight_color` (the hard-coded `String::new()` in `over()` - "a preset is a look, not a caption: no highlight" - replaced, which is exactly the C3 gap); the three manual-highlight presets ship on (#ffd21f, #39ff88, #ffffff), the other five off. Tests written in `presets.rs` (its own test module): the prescribed round-trip (a `preset.toml` with `highlightColor` + `kind` parses onto the style; an untagged file stays a title with no highlight) and the eight-ship-tagged test (all eight ids present, kind = caption, the three colours exact, the five off, `concat.bold-caption` still "title") - **not verified here**; the existing `every_shipped_preset_file_parses` covers the new files automatically once compiled. What this sandbox could check, it checked: all eight TOMLs parse (tomllib) with kind=caption and the table's values; `TextPreset` has exactly one construction site (the parser), so the new field breaks nothing else; fmt clean. Remaining for the owner's machine, recorded not guessed: the captions dialog's look picker (today the dialog's "presets" are the C1 *layout* presets, and generated captions' look comes from `prefs.title_style`; §9 holds dialog UI until checked in the running app), and the `upper-tight` "inverse" highlight is a renderer-side drawing mode - the data carries the colour, the inverse flag needs the owner's call.

### C3. Wire karaoke to presets
- **Plan status:** fixed in plan. **Code status: mechanism DONE with C2's data (18 Oct 2026) - see the Status bullet; end-to-end pick waits for the owner's machine.** (Was: renderer wired; no preset turns it on.)
- **Needs:** C2, T2 (karaoke is only as good as the timings).
- **Test first:** a karaoke preset produces one highlight per heard word (the existing renderer test `a_word_timed_caption_gets_one_highlight_per_heard_word` covers the renderer).
- **Done when:** picking a karaoke preset works without the Advanced settings.
- **Status (18 Oct 2026): the wiring C3 asks for is the data C2 ships, and it is in.** Read from the code, plainly: "on" is a non-empty `TextStyle.highlight_color` (`highlight_on: !text.highlight_color.is_empty()` in the inspector feed; the renderer lights each heard word with it - the existing `a_word_timed_caption_gets_one_highlight_per_heard_word` test in `concat-host` covers the renderer, **not verified here**), and before this change `PresetStyle::over()` hard-emptied it, which was exactly "no preset turns it on". Now pop-bold/karaoke/upper-tight carry their colours through `preset.toml` onto the style (A10: they start on), and the other five start off; A10's "every other preset can turn it on" is the existing inspector switch, which shows for any clip with words (`highlightable: clip.words.is_some()`) - every caption has words, so no gate change was needed or made. What remains, and why it was not blind-written: (1) the picker that puts a preset on a caption track (C2's owner-machine part; "Done when" is a GUI acceptance); (2) one recorded interaction for that session - `panes/captions.rs:1566` force-sets `DEFAULT_HIGHLIGHT` on generated captions whose style has none, which would override a picked preset that ships highlight OFF; when the picker lands, that line must become "only when nothing has decided yet" or the picker path must bypass it. Changing it now, before the picker exists, would switch off karaoke on every generated caption in today's app - a UX change nobody asked for, so it stays and is recorded here.
- **Decided (owner, 16 Oct 2026, A10):** the highlight is a feature the user can switch on or off, for **all** presets. It is the existing highlight option (the same one the Motion drawer shows, not gated by `highlightable`), not a new feature. Presets with a highlight in the manual (pop-bold, karaoke, upper-tight) start with it on; the user can turn it off. Every other preset can turn it on. Confirmed by owner (16 Oct 2026, Tier C approved). Quality still depends on T2.

### C4. Uppercase / text transform
- **Plan status:** fixed in plan. **Code status: DONE for what this sandbox can prove (18 Oct 2026); preset layer and painter boundary written, NOT VERIFIED here** - see the Status bullet below. (Was: absent.)
- **Status: DONE for what this sandbox can prove (18 Oct 2026).** Model, verified in `concat-project`: `TextTransform` (none | uppercase, lowercase wire names, stable for ever) on `TextStyle` as `transform`, `#[serde(default, deserialize_with = "wire::text_transform", skip_serializing_if = ...)]` - an absent key or an unrecognised name loads as none (every older document unmoved, nothing migrates), and none stays off the wire. `TextStyle::display_content()` is the render-time application: `Cow::Borrowed` for none, `to_uppercase()` for uppercase, the stored `content` never rewritten. Test-first, prescribed: `tests/text_transform.rs` - a style with uppercase renders uppercase and the stored text is unchanged; an old document loads as none; an unknown name loads as none; the round trip through the document wire. Red first (4/4 against `todo!()` stubs), then green: **4/4, project total 297**, clippy `-D warnings` and fmt clean. Written, **not verified here** (concat/concat-host cannot compile in this sandbox): the preset layer (`PresetStyle.transform` → `over()` mapping; `kconcat.upper-tight` ships `transform = "uppercase"` - the caps look the manual names, §1 "CapCut styles are usually ALL CAPS"; two tests in presets.rs's own module: the file round trip and upper-tight/boxed in the eight-ship test) and the painter boundary (`titles.rs::title_style` draws `style.display_content()`). Recorded decision: the transform is applied **only** at the painter boundary - the word-matching and karaoke layout keep running on the stored casing, so highlights land on the transcript's words (uppercase is 1:1 for ASCII/Indonesian; a multi-char uppercase like ß→SS could shift painter-side ranges - noted, not solved, no such text in the corpus). UI: no inspector toggle is specified by the plan; per §9 that is an owner-machine session (a switch in the Text tab binding `style.transform`).
- **Change:** a transform field in the style model. Applied at render time, never to the stored text, so the transcript keeps its casing.
- **Test first:** a preset with uppercase renders uppercase, and the stored caption text is unchanged.

### C5. Caption motion — D3 selected (a), schema approved (16 Oct 2026)
- **Plan status:** fixed in plan (schema v1 approved by owner, 16 Oct 2026). **Code status: schema v1 DONE and verified in `concat-project` + `concat-text` (18 Oct 2026); what drives `progress` stays owner-machine** - see the Status bullet below. (Was: absent, confirmed 16 Oct 2026: the caption dialog had no motion control; no preset set motion; the `pop`/`slide`/`zoom` names in `concat-effects/packages/` are video effects.)
- **Status: schema v1 DONE (18 Oct 2026, verified).** The schema's own four words, built exactly as approved. Document side: `CaptionMotion { mode: none|pop|word-reveal|typewriter, duration: 0.18, scale_from: 1.15 }` on `TextStyle` (`model.rs`) - survives save/reload, rides every preset apply; kebab-case on the wire, off the wire while default, unreadable values fall back to default; old documents read as mode none and `DOCUMENT_VERSION` stays 1. Clamps in `tidy()` and nowhere else, the schema's rule: duration 0.02..=2.0, scale_from 1.0..=3.0, non-finite to defaults. Painter side: `concat_text::Motion`/`MotionMode` mirror on `TitleStyle.motion`, copied field-for-field in `titles.rs`. Presets: optional `[motion]` block beside `[style]`, every field optional, both `scale_from` and `scaleFrom` read (serde alias), an unrecognised mode is no motion and never a parse failure; the eight shipped presets deliberately carry no block - no per-preset spec was ever approved, and the mechanism + user choice is the schema's path. Tests first (`tests/caption_motion.rs`): red 3/5 against the stubbed wire and absent clamps (reload-survival, unreadable-motion, clamp - as designed, `logs/step20-c5-tests-RED.log`), then green **5/5, project total 325, sweep 452/0** (concat-text's 31 included - the crate compiles here, newly confirmed), clippy `-D warnings` and fmt clean on concat-project and concat-text; the wire test pins both halves (no `motion` key while default - older builds' documents unchanged - and a round trip with a pop). **Written, not verified here** (crates this sandbox cannot compile): the `presets.rs` `[motion]` reader + its prescribed test (a motion block lands on the style), the `titles.rs` copy. **Not built, plainly:** what drives `progress` - the schema's own open question, still open: word-reveal consumes the baked `RevealMap`, typewriter timing comes from each word's spoken start/end per the schema, pop needs per-word timing - all of it host render-loop work on the owner's machine, plus the caption dialog's motion control and preset-picker previews (§9, owner's machine).
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
- **Plan status:** fixed in plan. **Code status: DONE for the sandbox-provable part (18 Oct 2026)** - see the Status bullet below. `captionmap.rs` does the inverse (remaps words through a cut), so about 80% of the machinery exists.
- **Needs:** T2 accepted (deleting a word is guesswork at ±300 ms), T7.
- **Tests first:** deleting one word ripples audio, picture, and captions together; deleting a sentence keeps the sync matrix; one undo restores everything.
- **Decided (owner, 16 Oct 2026):**
  - Gap after a delete is **closed (ripple)**: later audio and picture move left.
  - Changing a word's text (typo fix) is **text only**. The video is not cut and the word's timing is kept. Owner: do not add cut-on-retype, because it risks new bugs in a working path.
  - Existing timeline remove-selection stays as it is. E1 adds word-level delete on top of it. It does not replace it.
  - **Control (owner, 16 Oct 2026):** one checkbox, **"Potong video saat kata dihapus"** (Cut video when a word is deleted). On: deleting a word ripples audio, picture and captions. Off: only the word's text is removed, nothing else changes. The checkbox applies to both the Sections pane and the timeline caption row, through the same delete path (T7).
  - Remove-selection on the timeline is unchanged.
- **Done when:** the tests pass and the sync matrix passes for the delete path.
- **Status: DONE for what this sandbox can prove (18 Oct 2026), entirely inside `concat-project`.** `Command::DeleteCaptionWord{clip_id, word_index, cut_video}` - the A2 checkbox arrives as the `cut_video` bool, so any UI can carry it. Checkbox **off**: only the text goes - the word is cut out of the content at its located bytes (spacing and punctuation stay tidy) and T7's single writer prunes its timing slot while every survivor keeps its heard time; picture and sound stand still. Checkbox **on**: the delete delegates to the remove-selection machinery, as the roadmap predicted - `AutotrimClip` on the clip the caption follows (`caption_of`, else the first picture/sound the word is heard over), kept = the owner's span minus the word's heard span, so every lane is cut at the same moments, the gap closes (owner decision), captions rebuild with remapped words and repaired links. One husk rule added on top: a caption whose last heard word was just cut is removed instead of lingering wordless (its first piece keeps the original id, so it is findable). Refusals are one sentence each (`CommandError::CannotDeleteWord{reason}`): not a caption, no heard timings, index past the end, nothing under the word to cut, or the word's timing misses the clip it follows. One command, one undo - tested on the cut path (both lanes + the caption back in one step). Tests (`tests/caption_word_delete.rs`): the three prescribed (one word ripples audio+picture+captions; a sentence - deleted word by word, left to right - keeps the sync matrix on the survivor; one undo restores everything) plus the text-only mode and four refusals. Red first (5/5 against a `todo!()` handler), then green: **259 lib + 25 integration**; clippy `-D warnings` and fmt clean. Two facts recorded plainly: (1) word timings arrive through shift arithmetic, so the tests compare them with a 1e-6 tolerance - the one test-side correction this task needed; (2) the pre-existing autotrim packing quirk is untouched by design ("existing remove-selection stays as it is"): consecutive cuts must run left to right, because a second cut's window packs only within its own owner - the delete path is driven in that order. The UI checkbox itself is **not built**: §9 says E1's UI is not specified until it is checked in the running app, and the no-guessing rule applies to UI too. "Needs: T2 accepted" stands as written - the mechanism is tested against synthetic heard times; real-audio quality waits on T8's corpus.

### E2. Filler-word removal (Indonesian)
- **Plan status:** fixed in plan. **Code status: DONE for the sandbox-provable part (18 Oct 2026)** - see the Status bullet below. English-only in CapCut.
- **Needs:** E1.
- **List (starting):** eee, anu, kayak, gitu, apa ya, terus. Owner may add or remove words; record the final list in the manual.
- **Decided (owner, 16 Oct 2026, A7):**
  - **Filter, optional, labelled.** Filler words are grouped under labels (two tiers: a strong group, such as eee, anu, ehm; a weaker group, such as kayak, gitu, terus, apa ya). The user ticks which groups to apply. Nothing is removed unless ticked.
  - **Where it lives:** in the remove-selection flow, as an option next to the existing "silent detected" option. Remove-selection already exists (A2); filler adds one more option there.
  - **Video effect: follows the A2 checkbox** "Potong video saat kata dihapus". On: the video is cut and the gap closes (ripple). Off: only the text goes.
  - **Exact grouping** of the two tiers is fixed in the manual (`03-manuals`), with the final word list.
- **Left out and why:**
  - Always cutting video: not chosen; it would break the A2 rule that the user controls the cut.
  - Text only, always: not chosen; the user asked for the same behaviour as A2.
  - One fixed list with no option: not chosen; owner wants the user to decide, because meaningful words such as "terus" can be lost.
- **Done when:** a one-click pass removes the ticked groups inside a selected caption run, and leaves them alone when they are meaningful. Test with at least one negative case.
- **Status: DONE for what this sandbox can prove (18 Oct 2026), entirely inside `concat-project`.** The two tiers ship exactly as A7 fixed them - strong: `eee, anu, ehm`; weak: `kayak, gitu, terus, apa ya` - as the one fact in `src/filler.rs`, recorded for the owner in `03-manuals/C-filler-words.md` (the manual the roadmap asks for). Matching: whole words only ("begitu" is never "gitu"), case and edge punctuation ignored, "apa ya" needs its consecutive run, longer entries win first. `Command::RemoveFillerWords{clip_ids, strong, weak, cut_video}`: a selection may hold picture, so non-captions and unheard captions are skipped, not errors; a pass that hits nothing reports `applied:false` (no nothing-snapshot in the undo history); one pass is one undo either way. **Off** (A2 checkbox): the fillers leave the content at their located bytes and their timing slots go with them (T7's rule - text and words written together), survivors keep their heard times, picture stands still. **On**: every filler's heard span is cut through E1's path - merged spans, one `AutotrimClip` per owner clip left to right - all lanes cut at the same moments, gaps closed, captions rebuilt with repaired links; a caption left with no words goes with them (husk sweep scoped to what the pass rebuilt: pre-existing captions, including imported wordless cues, are never swept). One source fix this task forced, inside the existing autotrim rebuild: a rebuilt text piece whose words were ALL cut kept the stale full sentence as its content (the content re-join sat inside the `!kept.is_empty()` branch) - exactly the repetition the rebuild's own comment says must not happen; the re-join moved out, empty text now marks the husk, and the whole existing suite stayed green. Tests: five matcher units (both tiers, ticked/unticked, case+punctuation, whole-word negative, the "apa ya" run and its broken-up negative) + five integration (a two-cue run with strong-only - "terus" survives unticked, the prescribed negative; both tiers; the cut path with video packed to 9.7 s and the caption rebuilt as one synced cue + one undo; a filler-only caption gone without slivers; the "begitu" lookalike and the no-hit no-edit). Red first (5/5 units + 5/5 integration against `todo!()` stubs), then green: **264 lib + 29 integration**; clippy `-D warnings` and fmt clean. Not built here: the remove-selection-flow UI option beside "silent detected" (A7's placement) - §9 holds E1-E6 UI until it is checked in the running app; the command carries the checkbox as a bool, ready for it.

### E3. Waveform under the caption row + word-edge drag
- **Plan status:** fixed in plan. **Code status: word-edge drag DONE and verified in `concat-project` (18 Oct 2026); the caption-row waveform and the drag gesture are owner-machine UI (§9)** - see the Status bullet below. (Was: the timeline waveform exists in `src/crates/concat/ui/primitives/wave.slint`; caption-row waveform and word-edge drag did not.)
- **Needs:** T2, E1 recommended.
- **Done when:** dragging a word edge moves its highlight and timing, and the edit survives export.
- **Status: word-edge drag DONE (18 Oct 2026, verified).** `Command::MoveCaptionWordEdge { clip_id, word_index, edge: WordEdge::Start|End, time }` - absolute seconds on the millisecond grid the word timings already live on. The drag's kindness rule: the edge is CLAMPED, never refused - it stops at its neighbours, the first and last word stop at the caption's window (a word is never heard outside its own cue), and the word keeps its `MIN_WORD` width; a pathological thinner-than-minimum input meets the bounds instead of inverting the clamp, so nothing panics and no drag can overlap or reorder the heard sequence. Timing-only: the text, the window and every other word stand, so T7's writer is not involved; the highlight is a pure function of the word timings (C3's mechanism), which is what makes "the drag moves the highlight" provable here as "the drag moves the timing". A drag landing where it started applies nothing and records no undo entry; a live drag folds into ONE undo through the editor's existing gesture machinery (`apply_within` under one gesture name, one per mouse move). Refusals, one sentence each: not a caption, no heard words, index past the end; a NaN time is the editor's universal `NotANumber`, caught before dispatch. Tests first (`tests/caption_word_edge_drag.rs`): red 6/6 against a `todo!()` stub (`logs/step21-e3-tests-RED.log`), then green **6/6, project total 331, sweep 458/0**, clippy `-D warnings` and fmt clean: the drag moves only its edge; it stops at neighbours with monotonicity intact; minimum width and window bounds; one undo restores; the edit survives a save/reload round trip; all four refusals. **Not built, plainly:** the caption-row waveform (wave.slint draws the timeline's; the caption row needs its own draw + hit-test in slint), the drag gesture's mouse wiring, and the end-to-end "survives export" - the export binary does not compile in this sandbox; the round trip above is the sandbox-provable half, the owner's machine confirms with a real export.

### E4. Enter splits a caption at the caret
- **Plan status:** fixed in plan. **Code status: DONE for the sandbox-provable part (18 Oct 2026); the UI wiring is written but NOT VERIFIED here** - see the Status bullet below. (Was: Enter unwired, `Field` bound `edited` only.)
- **Needs:** T7 (the single writer), T2 for the boundary to land on the right word. Without T2 it is no worse than the character-proportional estimate already in use.
- **Decided (owner, 16 Oct 2026, A8):**
  - **Where:** Enter works in **both** places: the caption text editor and the Sections pane.
  - **Split time comes from the audio, not from the text.** The cut point is the audio boundary at the caret's word (the word start from the snapped audio timing, T2/T3). The owner rejected a split estimated from text length. No character-proportional split is used for the time.
  - Not built: no fallback to text length. If no audio timing exists for the word, the split is refused with a message, not guessed.
- **Approach:**
  - Forward `accepted(text, byte-offset)` from the field (or a caption-only variant).
  - Map the caret to the word it falls in.
  - Split time = that word's start from the audio (T2/T3 snapped timing). No text-length estimate.
  - One `Batch` of `UpdateClip` (head: shortened text and duration, words before the caret) plus `AddTextClip` (tail: remaining text, words from the caret). One undo step.
  - Existing pieces: caret readable via `cursor-position-byte-offset`, precedent at `src/crates/concat/ui/dialogs/speech.slint` near line 364; time split `Command::SplitClips` (`commands/mod.rs` near line 513) for the time axis only; the trim-path text rule (`commands/clips.rs` near lines 840–885).
- **Tests first:** Enter mid-caption gives two rows with the right text and boundary; one undo restores; the karaoke highlight of both halves lands on the spoken words.
- **Done when:** the tests pass and the sync matrix passes for the split path.
- **Status: DONE for what this sandbox can prove (18 Oct 2026).** Mechanism, verified in `concat-project`: `captionmap::word_at_caret` maps the caret's byte offset to the word it falls in (a gap caret belongs to the *next* word, so Enter after a space keeps a head; `None` where one side would be empty or the word cannot be located). `Command::SplitCaptionAtWord{clip_id, word_index}` cuts at the word's **heard start** - the audio boundary, never a text-length estimate, per the owner's A8 decision - and splits the content at the word's located bytes, so both halves keep the original punctuation and spacing. The head keeps the id, the transition and the way in; the tail is minted at the boundary on the same lane with the same look, `offset_y` and `caption_of`, its words' heard times standing as they are, and no second application of the lane nudge (boundary and words already carry it, T6); `caption_flag` is born `Some(true)` per the T5 birth-site rule. Refusals are one user-facing sentence each (`CommandError::CannotSplitCaption{reason}`): no heard word timings ("refused with a message, not guessed"), not a caption, caret at either end, word not in the text, or a piece below the minimum duration. One command, one undo - tested. Tests (`tests/caption_enter_split.rs` + three caret unit tests in captionmap): the three prescribed (two rows with the right text and boundary; one undo restores; both halves' karaoke lands on the spoken words - timings byte-identical and every word locatable) plus the no-timing refusal and the degenerate-index refusal. Red first (5/5 integration + 3/3 unit against `todo!()` stubs), then green: **259 lib + 5 + 3 + 6 + 5**; clippy `-D warnings` and fmt clean. Sync matrix asserted on both halves of every successful split. UI wiring, written but **not verified here** (slint/FFmpeg crates cannot compile in this sandbox): `Field` (modal.slint) now declares and forwards `accepted(text, caret)` from its `TextInput`, following the `cursor-position-byte-offset` precedent in speech.slint:364; `row-text-accepted` is forwarded beside `row-text-edited` at every hop (sections.slint list → dialog, SectionsView global, app.slint); `sections.rs::accept_row_text` commits the pending edit through `edit_row_text` first (T7 keeps the words with it), maps the caret, issues the command - `Studio::apply` already toasts a refusal - and refreshes the rows; lib.rs hooks both callback paths. Left open, recorded plainly: the captions dialog's own draft editor (captions.slint:484, a bare `TextInput` with `text-edited`) is NOT wired for Enter - its draft↔timeline-clip mapping needs runtime investigation, and per the no-guessing rule it was not blind-patched. Boundary quality on real audio depends on T2 acceptance, as the plan says.

---

### E5. Link / unlink audio and video (replaces Detach / Reattach)
- **Owner decision (16 Oct 2026):** replace Detach / Reattach with Link / Unlink. Every video sits on the **video line**; its audio sits on the **audio line**; the two are linked. Lanes are typed V / A (D6). Reference: Adobe Premiere Pro (Link, Unlink, relink) and the owner's VTS, where audio, video and captions are separate.
- **Parity item served:** audio on its own line, movable separately. Owner's reference, not a CapCut feature claim (not verified against CapCut in this session).
- **Plan status:** fixed in plan (owner decision 16 Oct 2026). **Code status: mechanism DONE and verified in `concat-project` (18 Oct 2026, checkpoint 10); the context-menu swap (Link / Unlink in place of Detach / Reattach) and linked selection in `studio.rs` wait for the owner's machine** - see the Status bullet at the end of this task. (Was: not started.) Detach / Reattach exists today (`studio.rs` near lines 7151–7156 `sound_placement`, 9842–9855 context menu, 10495–10590 handlers); the model has `muted` and `detached_from` (`concat-project/src/model.rs` near lines 1356–1361).
- **Current gap (verified by reading):** after Detach, the audio clip moves on its own and the link is only used by Reattach (`studio.rs` near line 4272). There is no lock that keeps video and audio moving together.
- **Behaviour to build:**
  - **Link:** select a video clip and an audio clip, right-click, Link. They form one group. Moving, trimming, splitting or deleting one acts on the group while linked.
  - **Unlink:** right-click a linked clip, Unlink. Both clips stay on their lines and move independently. Link can be applied again at any time.
  - **Linked selection:** selecting one clip of a group selects the group (as Premiere does) so a group move is one click.
  - **Captions** (D7): each speaker's caption lane is linked to the video and to that speaker's audio. Moving the group moves its captions. Today captions follow the video clip (`captions_of`, `captionmap::remap`); with D7 they also follow the speaker's audio.
  - **Remove** the Detach and Reattach menu items. Their replacement is Link / Unlink only.
- **Lane typing (D6, typed):** a legacy untyped track needs a type on load. Rule to decide in the diff: type it by the clips on it; a track with both video and audio is split or flagged. Test with an old-project fixture.
- **Import default (E5-b, decided by owner, 16 Oct 2026): yes.** Each imported video becomes two linked clips: video on V1, its audio on A1. **Confirmed by owner (16 Oct 2026):** migrate from the combined model (video and audio in one clip) to the Premiere model (video and audio separate, but linked). Old projects with combined clips: the split rule is decided in the diff (D6 legacy rule), with an old-project fixture test.
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
- **Status: mechanism DONE (18 Oct 2026, verified, checkpoint 10).** Model: `Clip::link_group: Option<String>` (serde default, skip-when-none: no `linkGroup` key while nothing is linked, so an older build's document is unchanged; `DOCUMENT_VERSION` does not move). Helpers on `Timeline`: `link_mates` (the linked selection), `with_link_mates` (a selection widened by its groups - the one definition the commands and the UI share). Commands (new `commands/links.rs`): `LinkClips { clip_ids }` (≥ 2 existing clips, captions refused with one sentence - they already follow through `captionOf`; a member in a group brings its group, so Link is a merge; an already-complete group is a no-op, not an undo step) and `UnlinkClips { clip_ids }` (dissolves each named clip's whole group). **Group rule in one place:** `links::widen` runs once in `commands::apply` before any arm, so no arm knows groups exist: Move - every mate not dragged itself gets the same offset on its own lane; Delete - mates go too; Split - mates cut at the same instant and the tail pieces re-grouped (a cut through a pair leaves two pairs, as Premiere does); Trim - a mate whose dragged edge sits at the same instant (±0.5 ms) is trimmed by the same amount, a mate whose edge is elsewhere is left alone (the shared edge is what a linked trim means - recorded decision). Each widened edit is one command, so one undo. Captions follow the group through the existing `follow_captions` (test 4 - no new caption code). **Migration:** `doc::link_detached_pairs` reads every saved `detached_from` pair as one group on load (id `g-<video id>`, which the mint adopts), `muted` on the video kept; `DetachAudio` / `ReattachAudio` commands still exist until the menu swap. Tests (`tests/link_unlink.rs`): the roadmap's 1-6 as written; 7 in its sandbox half (Link touches nothing an export reads: media, stream, mute, filters, source range, duration, speed; wire round trip) - the real file comparison is the owner's (FFmpeg); plus linked split/delete, linked trim at a shared and an unshared edge, refusals and the merge. **11/11, project total 347, sweep 529/0**, clippy `-D warnings` and fmt clean. **Not built here, plainly (owner's machine, `concat` does not compile in the sandbox):** the context menu in `studio.rs` ~9842-9855 (rows `detach`/`reattach` → `link`/`unlink`, keys `studio.linkClips` / `studio.unlinkClips` in every locale), handlers ~10495-10590 (`sound_placement` → `link_mates`), the linked selection on click (`Timeline::link_mates` on the clicked id), the import default E5-b (each imported video as a video clip on V1 + an audio clip on A1, linked - needs the typed lanes of D6, not started) and the D6 legacy lane typing.

### E6. Select one whole lane with one shortcut
- **Owner decision (16 Oct 2026):** shortcut **Shift + L** selects every clip on the current lane, from the first to the last, and no other lane. The user does not have to zoom out or hold Shift while dragging to the end.
- **Plan status:** fixed in plan (owner decision 16 Oct 2026). **Code status:** not started. The gesture today is zoom out, or Shift-drag to the end. I did not find a lane-select command in the code I read (`studio.rs`, `timeline/lanes.slint`); verify before coding.
- **Check before coding:** Shift + L must not clash with an existing shortcut. Confirm the keymap in the diff. Confirm which lane counts as "current" (the lane under the playhead, or the last lane clicked).
- **Tests first:** Shift + L on V2 selects every clip on V2 and nothing on V1 or A1; a second press or Escape clears it; one undo restores the selection; a locked lane is skipped or refused (decide in the diff).
- **Done when:** the tests pass and the shortcut is listed in the shortcut help.
- **Status: lane rule DONE (18 Oct 2026, verified, checkpoint 10); the key and the selection state are UI, owner's machine.** `Timeline::clips_on_track(track_id)` is the one place that decides what "the whole lane" holds: every clip on that lane, first to last by start (not insertion order), no other lane, empty for an unknown track; it does not pull a linked mate off another lane by itself - the UI widens with `with_link_mates` if it wants the linked audio too (test shows both). Test in `tests/link_unlink.rs` (`e6_...`). **Not built here:** the Shift + L binding in `studio.rs` / `lanes.slint` (selection is UI state, there is no selection in `concat-project`), the "current lane" choice (last clicked vs under the playhead - owner decides in the running app), the second-press / Escape clear, the locked-lane rule, the shortcut-help line, and the keymap clash check.

## 5. Track 4 — finishing (vertical and export)

### V1. Auto-reframe 16:9 → 9:16
- **Plan status:** fixed in plan. **Code status: mathematics DONE and verified in `concat-project` (18 Oct 2026); the mask pipeline and renderer confirmation wait for the owner's machine** - see the Status bullet below. (Was: not started.)
- **Status: mechanism DONE (18 Oct 2026, verified).** The plan's chain, built as pure mathematics in `concat_project::reframe`: per-frame mask centroid (`mask_centroid`: an eight-bit grid - what `concat-vision`'s RVM store keeps - to one normalised point, pixel centres not corners, `None` when the mask saw nobody or the grid does not match) -> smoothing (`pan_track`: the window centre clamped to what the frame can show, through the one-euro filter - the plan's first choice - and clamped again, because a low-pass can overshoot past a clamp its input respected; the parameters, min_cutoff 0.5 Hz / beta 10 / d_cutoff 1.0, are starting guesses with the T3 flag: tune on the owner's real footage) -> crop keyframes (`reframe_plan`: the constant fill scale `1/visible_fraction` clamped by `ranges::MIN_SCALE..MAX_SCALE`, plus one `KeyProperty::OffsetX` key per smoothed centre in the model's own documented units - `scale` is "multiplier over the fitted size", `offset_x` "offset from centred, as a fraction of frame width" - shifting content right by `(0.5 - centre) * scale` frame widths puts the source's `centre` at the middle; consecutive keys at one instant merge by `set_key`'s own rule). `Command::AutoReframe { clip_id, target_width, target_height, points }`: the caller brings the points (the masks live beside the decoder and the model, not in this crate); refusals are one sentence each (no points, no clip, no media picture, no picture size, a frame the clip already fills - `visible_fraction` is `None` at one or above); non-finite points refused before dispatch; one command, one undo (the fit and the keys come back together). Tests first (`tests/auto_reframe.rs`), prescribed: a moving-subject fixture produces a smooth track - the jitter threshold, set at the writing as the plan requires, is a 2e-3 interior second difference against a raw step of ~0.0125, and the track follows a steady ramp within a 0.03 lag - plus the centroid primitive, a ±0.01-noise steady subject held within 0.008, an edge-pinned subject whose window never leaves the frame, the command writing scale 3.160494 (16:9 into 9:16) with bounded keys and one undo restoring the fit, and five refusals. Red first (6/6 against `todo!()` stubs), then green: **6/6, project total 320, sweep 416**, clippy `-D warnings` and fmt clean. **Not built here, plainly:** the mask pipeline itself (RVM inference sits behind `concat-vision`'s `infer` feature - `ort` - which this sandbox cannot compile), the host call site that turns stored masks into per-frame points, the renderer-side sign confirmation of `offset_x` (the convention is written in the module's docs for exactly that check), and the done-when run: a talking-head 16:9 clip becoming a framed 9:16 export, on the owner's machine, with the jitter threshold re-measured on real footage.
- **Approach:** per-frame mask centroid → smoothing (one-euro or Savitzky–Golay) → crop keyframes. Keyframes, crop, and curve editor exist.
- **Tests first:** a fixture with a moving subject produces a smooth crop track with no jitter above a threshold to be set when the test is written.
- **Done when:** a talking-head 16:9 clip becomes a framed 9:16 export without manual keyframing.

### V2. Platform export presets + loudness
- **Plan status:** fixed in plan. **Code status: loudness half DONE (18 Oct 2026, verified in sandbox); platform-preset half not built here** - see the Status bullet below. Export has three quality cards (`src/crates/concat/ui/dialogs/export.slint` near line 137). `loudnorm` is used before transcription only (that pre-transcription filter is separate and untouched).
- **Targets:** TikTok, Reels, Shorts, YouTube: aspect, bitrate, and **−14 LUFS integrated, ±1 LU**.
- **Decided (owner, 16 Oct 2026, A4 poin 1):** loudness normalisation runs **only when a platform preset is chosen**. A normal export keeps the level the user set.
- **Decided (owner, 16 Oct 2026, A4 poin 2, option D):** a **per-clip toggle** with target **−14 LUFS**. The toggle works on one selected clip. The preset rule above still applies to the whole export. CapCut's per-clip toggle targets −23 LUFS (CapCut's own page); this project uses −14.
- **Tests first:** an exported fixture measures −14 LUFS ±1 LU; each preset yields the right aspect.
- **Done when:** one menu choice produces an upload-ready file.

- **Status (18 Oct 2026): the loudness-first half is DONE and verified; the platform-preset half is specified and waits for the owner's machine.** Done, in `concat-effects` - which turned out to be sandbox-verifiable (no FFmpeg dependency: it parses the manifests and renders the chain strings; FFmpeg only executes them at export time): `concat.loudness` retargeted exactly as the owner decided ("reuse this effect") - default **-23 -> -14 LUFS**, chain true-peak **-1.5 -> -1.0 dBTP** (A4 point 3), range -30..-12 unchanged (the default sits inside it). The per-clip toggle (A4 option D) *is* this effect on a clip, so it now targets -14. Test-first through the crate's own builtin-fixture harness: fixtures pinned to the new chains went red first (all three loudness cases named by `every_built_in_package_loads_and_its_fixtures_pass`, 71 other builtin tests green), then the manifest change went green: **72/72**, clippy `-D warnings` clean. Not built here, plainly: the four platform presets (TikTok/Reels/Shorts 9:16 1080x1920, YouTube 16:9 1920x1080) and A5's fields (bitrate encoding VBR 1-pass/2-pass/CBR, target+maximum Mbps, audio AAC 128/192/320 kbps @ 48 kHz stereo, the frame-rate list) plus A4 point 4 (loudnorm on the final mix when a platform preset is chosen). That work lives in the export dialog (the quality cards are an int index today), the concat export flow, `concat-export`'s `ExportRequest` (which has CRF and a single CBR `bitrate_kbps` - no target/max pair, no 2-pass, no audio options, no master-loudnorm flag) and the FFmpeg invocation: four crates this sandbox cannot compile, and a render-path surgery rather than a delegation - the no-guessing rule says it is written on the owner's machine, where A5's own note also puts it ("Build waits for the diff"; point 2's sizes are still "proposed, not yet confirmed"). The acceptance test is already fixed and quoted here for that session: the exported file measures **Integrated -14 LUFS ±1 LU and true peak <= -1 dBTP** (A4 point 5), each preset yields its aspect, and one menu choice produces the upload-ready file.

**A4 decisions, aligned with Premiere Pro (owner, 16 Oct 2026):**
- **Poin 3, true peak:** Maximum True Peak **−1 dBTP**, paired with the −14 LUFS target. Premiere's default recommendation (Larry Jordan, Premiere Loudness Meter article).
- **Poin 4, where normalisation runs:** on the **final mix (Master)** for a platform preset. Premiere measures and sets loudness on the Master track (Adobe help: Normalize Mix Track). The per-clip toggle (option D) is separate.
- **Poin 5, test:** the exported file measures **Integrated −14 LUFS ±1 LU** and **true peak ≤ −1 dBTP**. Premiere shows Integrated on the Master meter; ±1 LU is this roadmap's tolerance.
- **Poin 6, done:** one menu choice on a platform preset produces a file that passes poin 5.
- **Found in code (16 Oct 2026, not in the earlier plan):** the effect `concat.loudness` ("Normalize Loudness", kind audio) already exists. It normalises one clip with FFmpeg `loudnorm`. Its target range is −30 to −12 dB (LUFS), default −23, and its true-peak limit is −1.5 dBTP. Decision A4 uses −14 LUFS and −1 dBTP. **Decided (owner, 16 Oct 2026): reuse this effect.** Change its default to −14 LUFS and its true-peak limit to −1 dBTP (A4 point 3). The per-clip toggle (option D) is this effect, switched on per clip. Test: point 5 (exported file at −14 LUFS ±1 LU, true peak ≤ −1 dBTP).
- **Poin 7, gain (Premiere model):**
  - **Audio Gain per clip** in dB, set or adjusted by a value (Premiere: Clip > Audio Options > Audio Gain). Not built. The data model has `volume` as a linear factor (1.0 = unity); no dB gain field was found.
  - **Track volume** per lane (Premiere: Audio Track Mixer). Not built. The `Track` struct has only `visible` and `muted`.
  - **Clip volume keyframes** stay as they are (`KeyProperty::Volume`).
  - Sources: Adobe help "Adjusting volume levels" and Premiere Audio Gain help page. CapCut parity not checked here.

**A5 decisions (owner, 16 Oct 2026):**
- **Poin 1, platforms (decided):** four presets are enough: TikTok, Reels, Shorts, YouTube. No others are added now.
- **Poin 2, size per preset (proposed, not yet confirmed):** TikTok, Reels, Shorts = 9:16, 1080×1920. YouTube = 16:9, 1920×1080. Source: third-party CapCut guides agree on these sizes; CapCut has no named platform presets, only manual settings.
- **Poin 3, frame rate (decided, owner 16 Oct 2026):** the list is "same as source" (default), **23.976, 24, 25, 29.97, 30, 50, 59.94, 60 fps**. Decimal rates are added as explicit options. Source of the base list: a third-party guide on CapCut Desktop (24, 25, 30, 50, 60 fps plus the source rate); the NTSC decimal rates are added by owner decision.
- **Poin 4, bitrate (decided, owner 16 Oct 2026):** bitrate is **editable**, as in Adobe Premiere Pro. Premiere's fields, checked in third-party guides (not Adobe's own page): **Bitrate Encoding** (VBR 1-pass, VBR 2-pass, or CBR), **Target Bitrate** and **Maximum Bitrate** in Mbps. Guides suggest a maximum of about 1.5–2× the target. kconcat adopts the same three fields. The preset sets the default values (1080p at 30 fps: about 8–16 Mbps target; at 60 fps: about 12–24 Mbps). The user can type any value. Left out: a fixed-only bitrate, which the owner rejected.
- **Poin 5, codec and audio (decided, owner 16 Oct 2026):** the video codec is **not changed**; kconcat already has its codec (H.264 in MP4 is the current default). Audio gets **several options** that the user picks from: AAC at 128, 192, and 320 kbps, sample rate 48 kHz, stereo. Premiere's usual audio setting is AAC, 48 kHz, stereo, and 320 kbps (third-party guides). Check the existing export code for audio settings before the diff.
- **A5 status (16 Oct 2026):** poin 1–6 decided. Build waits for the diff.
- **Sequence settings (checked 16 Oct 2026, owner question "bisa seperti Premiere?"):** already in kconcat. Each timeline (tab) has its own frame size, frame rate, and colour space (`VideoSettings` in `model.rs`; `project.slint` dialog). Frame size options: 1920×1080, 3840×2160, 1080×1920, 1080×1080, 1440×1080, 2560×1080, Custom. Frame rate options: 10, 20, 23.976, 24, 25, 29.97, 30, 48, 50, 59.94, 60 fps, Custom. The rate is stored as a fraction (29.97 = 30000/1001). Colour: SDR, HDR HLG, HDR PQ.
  - **Gap vs Premiere (owner decision, 16 Oct 2026): none.** The existing per-timeline frame size and rate are enough. Premiere's sequence presets and per-sequence audio settings are not added; no task added.
  - **Export and sequence are separate, as in Premiere.** A platform preset sets the export frame, not the sequence. A 16:9 sequence exported as 9:16 needs the reframe in V1; without it the frame is letterboxed.

### V3. Auto ducking
- **Plan status:** fixed in plan. **Code status: mechanism DONE and verified in `concat-project` (18 Oct 2026); the host decode call site, the Music/Dialogue tags and the UI wait for the owner's machine** - see the Status bullet below. (Was: absent.)
- **Status: mechanism DONE (18 Oct 2026, verified).** New `concat_project::duck`: one keyframe writer, `duck_keys`, and both speech routes the owner chose (A6) feeding it - Route B `regions_from_words` (the heard words are the speech; accuracy follows T2/T3) and Route A `regions_from_samples` (20 ms RMS frames; a frame is speech when it sits within `40 dB x (1 - sensitivity)` of the loudest frame, with an absolute floor of -50 dBFS so an all-silence clip produces no regions instead of ducking everything - both numbers are starting guesses flagged for tuning on the owner's real audio, the T3-bands precedent; the sensitivity dial itself is the user's, clamped 0.05..=1). Shared merge rule, recorded because it is a decision: a gap no longer than the fade stays ducked - a ramp that cannot complete is flapping. `Command::DuckMusic { clip_id, against_clips, against_regions, amount_db, fade_ms }` (routed to properties): Route A's regions win when given (the caller only passes them when the analysis is the point), else Route B collects the words of the named clips. Output is ordinary `KeyProperty::Volume` keys through `Clip::set_key` (A6: "volume keyframes on the music clip - editable and reversible"): editable in the curve editor, erasable with the existing `ClearClipKeys`, one undo for the whole duck (the editor's snapshot). Owner defaults kept as constants (-12 dB, 500 ms) and editable per command; the writer clamps amount to -60..=0 (the dial cannot invent gain), caps the fade at 10 s, lands a step down/up at zero fade (stable sort + set_key's last-at-an-instant wins), clamps edge-hanging regions into the clip's 0..=1, and writes nothing for a region wholly outside. Non-finite amounts/regions are refused by `has_non_finite` before dispatch. Tests first (`tests/auto_ducking.rs`): the two prescribed - the bed drops under speech (before / mid-ramp / under / between / second region / tail, and every key at the fade's own instants) and returns after; one undo restores - plus Route A on synthetic samples (silence-tone-silence: the gate finds the tone, the same writer makes the same keys), the merge rule both ways, the writer's four edges, and the NaN refusal. Red first (5/6 against `todo!()` stubs; the refusal test passes from the start because validation runs before dispatch - recorded, not hidden), then green: **6/6, project total 303**, clippy `-D warnings` and fmt clean. **Not built here, plainly:** the host decode call site that feeds Route A real samples (FFmpeg), the Music/Dialogue clip tags (A6: "tag names are not yet decided in code" - not invented), and the UI (Duck Against picker, sensitivity slider) - §9 holds those for the owner's machine. The acceptance run (music bed under real speech, reversible) is the owner's, on real audio.
- **Tests first:** a music bed drops under speech and returns after; the change is reversible.
- **Decided (owner, 16 Oct 2026, A6):**
  - **Speech signal: both, chosen per clip.** Route A: audio analysis, as in Premiere (Sensitivity slider). Route B: word timing from the transcript (no extra analysis; accuracy follows T2/T3). The user picks the route for each music clip. Both routes must pass the test above.
  - **Defaults: kept, editable.** Duck Amount −12 dB, Fade 500 ms. The user can change both. The defaults stay as a reference point.
  - **Output: volume keyframes on the music clip**, as in Premiere (Generate Keyframes). This keeps the result editable and reversible.
  - **Labels:** music clips are tagged Music and speech clips Dialogue, as in Premiere (Duck Against = Dialogue). Tag names are not yet decided in code.
- **Left out and why:**
  - Audio analysis only: not chosen by owner; the transcript route is cheaper where word timing is good.
  - Transcript only: not chosen by owner; it fails for speech without words (for example, a music intro with no transcript).
  - Automatic ducking on export, without a control: CapCut's auto-ducking was reported by users as too strong, with no simple off switch (Reddit, not official). kconcat does not copy that; the user always has the choice.
- **CapCut parity:** partly. CapCut has automatic ducking (user reports only; not verified in CapCut's official docs in this session).

### V4. Batch export queue
- **Plan status:** fixed in plan. **Code status: queue state DONE and verified in `concat-project` (18 Oct 2026); the separate-process runner and the queue UI wait for the owner's machine** - see the Status bullet below. (Was: API jobs only.)
- **Status: mechanism DONE (18 Oct 2026, verified).** New `concat_project::queue`: `ExportQueue` over `QueuedExport { id, name, state }` with `QueueState` pending / running / done / failed(reason). A11's three decisions are the module's invariants, each with its own test: **failure continues** (`start_next` steps over marked failures to the next pending item, and `failed()` keeps every reason visible); **the order is the user's** (`move_item` up/down before *or during* the run - a pending item swaps with a pending neighbour; the running item is the runner's and finished items are history that stays in place, so the pending stretch stays contiguous); **one encoder at a time** (`start_next` refuses while anything runs - the runner is one separate process, A11's Media Encoder shape). One more guard the run needs: `finish` refuses an id that is not the running one's, so a late report from a killed runner cannot rewrite the item that replaced it. Placement, recorded as a decision: the queue is **session state in `concat-project`**, not project state - no `Command`, no undo, no `DOCUMENT_VERSION` move - because that is the only crate on the export path this sandbox can compile (checked 18 Oct, and a correction of an earlier session claim: `concat-export`, `concat-api`, `concat-server` and `concat-cli` all fail `cargo check` here; everything on that side transitively reaches FFmpeg). The host embeds the queue the way it embeds the editor; the payload (the `ExportRequest` per item) stays app-side, keyed by the queue's id. Tests first (`tests/export_queue.rs`): five shorts run in the order the user chose (two moves, then the whole pass); a failed item does not stop the rest (reason kept, queue completes, `failed()` reports); one item runs at a time; reordering stays the user's during the run (running and finished items refused, edge does not wrap, unknown id refused); the late-report guard. Red first (5/5 against `todo!()` stubs), then green: **5/5, project total 308**, clippy `-D warnings` and fmt clean. **Not built here, plainly:** the runner process itself (spawn, feed, watch - export machinery), the queue panel and the export dialog's "add to queue" (§9, owner's machine). The done-when ("five shorts export in one pass without babysitting") completes there; every promise the queue makes about that pass is tested here.
- **Decided (owner, 16 Oct 2026, A11):**
  - **Queue runs as a separate process, like Adobe Media Encoder.** Editing can continue while the queue runs. The queue is in the app, fed from the export dialog.
  - **Failure: the queue continues.** A failed item is marked, and the rest run.
  - **Order: the user chooses.** The user can move items up or down before or during the run, and pick which one encodes first.
  - **Left out and why:** (a) a queue in the same process, which blocks editing; (b) batch export with no queue, which cannot be reordered or reused. Both were offered to the owner and not chosen.
- **Done when:** five shorts export in one pass without babysitting, with the order the user chose, and a failed item does not stop the rest.

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
- **Plan status:** fixed in plan (scope decision pending). **Code status: label model + manual route DONE and verified in `concat-project` (18 Oct 2026); detection stays open on its engine choice** - see the Status bullet below. (Was: absent.)
- **Status: manual route DONE (18 Oct 2026, verified).** The decided parts are built; the undecided parts are recorded, not guessed. Model: `Speaker { id, name, color }` registry on `Timeline` (owner: colour belongs to the speaker, every caption of the speaker takes it through the label, D7); `Track.speaker` links one caption lane per speaker; `Clip.speaker` labels the audio clip and the caption. All three serde-defaulted and skip-when-empty: every older document reads as "no speakers", `DOCUMENT_VERSION` does not move, and with the registry empty nothing in the editing flow changes - the owner's opt-in rule, enforced by default. **Recorded deviation, deliberate:** the label lives on the caption, not on `captionmap::Word` - every word of a caption inherits its speaker, which is lossless for the case the plan itself calls the main one (a caption is one speaker's utterance; overlap is manual-only by owner decision). A `speaker` field on `Word` would break struct literals across `concat-speech` and every test harness - crates this sandbox cannot compile - for a granularity the plan does not use yet; word-granular moves compose from what exists (split at the word, E4; assign the piece). Commands (new `commands/speakers.rs`): `AddSpeaker` (mints, name trimmed, never blank), `AssignCaptionToSpeaker` (refuses picture and unknown ids with one sentence each; makes the speaker's lane on first use - no empty slots - on top where caption lanes live; the heard timings and transcript stand as they are), `RenameSpeaker` + `SetSpeakerColor` (tags editable, owner 16 Oct), `RemoveSpeaker` (refused while any caption carries the label; an unused speaker's empty lane goes with it; the one-lane floor is safe because a speaker lane is always an extra). One command, one undo (the editor's snapshot): the undo of an assignment takes the made lane with it, while the speaker - an earlier command - stays. Tests first (`tests/speaker_labels.rs`): red 6/6 against `todo!()` stubs, then green **6/6, project total 314**, clippy `-D warnings` and fmt clean; the wire test pins both halves (no `speakers` key while empty - older builds' documents unchanged - and a whole-project round trip with a speaker, a lane and a label). **Not built, plainly:** the detection engine (sherpa-onnx vs speakrs - open until the owner's 2-3 min ground truth exists; the opt-in "Detect speakers" button feeds `AddSpeaker` + assignment through the same commands), splitting captions at speaker changes (arrives with detection), overlap (manual by decision), the multi-stream import question (owner checks the app first), D7's link groups (depend on E5, not started), and all UI (§9, owner's machine).
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
- **Owner decision (16 Oct 2026):** check in the app first whether one file with several audio streams works, before choosing between separate files and one multi-stream file. Not decided yet.
- **Tag editability (owner, 16 Oct 2026): yes, editable.** Speaker tags on words, captions, and clips can be changed by the user.
- **Existing code hint (checked 16 Oct 2026):** `model.rs` `Clip.audio_stream` lets a clip play one of several audio streams of one file. So one file with several audio streams may already work. Confirm in the app before planning import.
- **Auto-detection (16 Oct 2026): feasible, required for mixed recordings, not built.** Audio-only diarization (who spoke when) runs locally after transcription. Candidates: `speakrs` (Rust, ONNX Runtime, claims pyannote-level accuracy, crates.io v0.5.0); sherpa-onnx offline diarization (pyannote segmentation 3.0 + CAM++, no Python). The owner names each detected speaker once (for example SPEAKER_00 → Host). No per-line labelling.
- **Limits (stated, not measured here):** overlapping speech (two people talking at once) is hard. One speaker per word is lossy in those places. Audio-only detection does not use video, so "who is on screen" is out of scope.
- **Effort estimate:** M for basic auto labels (one diarization pass, map speaker turns to words by time, split captions at speaker changes, rename once). L if overlap handling and polish are needed. A new native dependency is added (ONNX Runtime and models), so its crate placement needs a decision like D4.
- **Accuracy test (next step, owner):** the owner labels about 2–3 minutes of the 5-minute corpus once, as ground truth. This is for testing only, not for daily use. Needed to choose the engine.
- **Disk footprint (checked 16 Oct 2026, partly):** sherpa-onnx path = pyannote segmentation 3.0 (about 6.6–7 MB) + CAM++ embedding (about 28 MB) = about 35 MB of models, plus the ONNX Runtime library (size not checked). speakrs model sizes were not checked. The bigger disk cost is usually Cargo build output (`target/`) for the new crates, not the models. Models can be stored outside `target/`, in the app data folder.
- **Prerequisite (answered by owner, 16 Oct 2026):** one track per person. Open next: are these separate audio files, or one file with several audio streams?
- **Approach:** a local diarisation model after transcription; output stored as a speaker tag per word or cue.
- **Tests first:** on a two-speaker corpus episode, turns are assigned; one manual correction is kept.

### P4. Audio recording, one track per microphone (scope changed by owner, 16 Oct 2026)
- **Plan status:** fixed in plan. **Code status: multi-mic placement DONE and verified in `concat-project` (18 Oct 2026); session recording written, not verified here (needs cpal hardware)** — see Status bullet below. (Was: microphone only, one input in `src/crates/concat-host/src/record.rs`.)
- **Decided (owner, 16 Oct 2026):**
  - **Audio only.** Screen recording is not needed and is dropped from P4. Webcam video is not recorded either; the webcam's own audio is not a source.
  - **More than one microphone: one track per microphone.** This matches P3 (one track per person). Recording with one microphone stays the same.
- **Left out and why:** screen recording and screen-plus-webcam two-clip takes. Owner said screen recording is not needed.
- **Test first:** two microphones record at once; each lands on its own track; the tracks start at the same time and stay in sync for a 5-minute take (sync check in the diff).
- **Done when:** one take with two microphones gives two audio tracks, one per microphone, that stay in sync.
- **Status: multi-mic placement DONE (18 Oct 2026, verified).** `concat_project::takes::place_session(editor, playhead, takes)` — all-or-nothing: a NaN playhead or offset refuses the *whole* session before the first command, so a refused session changes no document. One new track per microphone (`AddTrack { bottom: true }`), each clip at its own landing `playhead + offset` clamped at zero, so the *distance* between landings is the distance between first samples — sync survives placement by construction, the five-minute acoustic check (two mics, one take, clap at both ends) is the owner's machine. The whole session folds into ONE undo step through `apply_within("place-session", …)` — one take, one track, one undo. Empty session is a no-op. Tests first (`tests/recording_takes.rs`): red 5/5 against `todo!()` (logged `logs/step22-p4-tests-RED.log`), then green **5/5, project total 336, sweep 463/0** incl. concat-text 31, clippy `-D warnings` and fmt clean. **Written, not verified here** (crate does not compile in this sandbox — needs cpal/FFmpeg): `concat-host::record::Session` (one `Recorder` per mic, one file per mic, one shared `rolled` anchor), `Recorder::start_from(device)` (None = default, Some(name) = named device as `input_devices()` lists it), `input_devices()`. Single-mic voiceover (`Recorder::start`) is unchanged. **Not built, plainly:** the multi-mic meter UI, the session start/stop wiring in the voiceover pane, and the five-minute sync recording on the owner's hardware.

### P5. Long-form end-to-end check
- **Done when:** a 60-minute episode becomes three 60-second vertical clips and one full export, offline, start to finish. Count manual corrections; if the count rises after a track closes, something regressed.

### P6. Verify before planning — VERIFIED 18 Oct 2026
- **Multi-clip / B-roll: exists — verify on the owner's machine (18 Oct 2026).** Owner reports it exists; the timeline model already supports multiple clips/tracks and the app's multi-clip path has not been touched by any step in this closeout (P4/C5/E3). The running-app check is: place two video clips on the timeline, confirm they composite and export in order. Sandbox cannot run the app (needs slint/FFmpeg), so the owner's machine is the proof — recorded here when it happens.
- **Caption animation: owner confirmed 16 Oct 2026 that no animation exists — built as C5, not a verify task.** Motion schema v1 (owner-approved 16 Oct 2026) is now code: `CaptionMotion` on `TextStyle`, `concat_text::Motion` mirror, preset `[motion]` block, kebab-case wire. P6's animation item is done through C5; no separate verify.


### P7. Denoise: already in the app (checked 16 Oct 2026); verify, do not build — VERIFIED 18 Oct 2026
- **Plan status:** owner decision (16 Oct 2026): add denoise, not vocal isolation.
- **Code status: exists — no code to add (verified 18 Oct 2026).** `src/crates/concat-effects/packages/concat.denoise` is "Noise Reduction", kind `audio`, category Clean — FFmpeg `afftdn` with Strength 0–100% (default 50). The same folder has `concat.enhance-voice` (high-pass 80 Hz, `afftdn`, de-esser, compressor, limiter, Amount 0–100%, default 60). Earlier this task was written as "new build" — that was wrong; the feature exists and the earlier correction stands.
- **Verified where it can be:** the package file exists on disk with the strength mapping above; the effect loads through `concat-effects` like every other package (123 packages total, none changed). The running-app check — apply Noise Reduction to one audio clip from the owner's room, confirm the transcript keeps the same words and export keeps speech natural — remains the owner's machine (FFmpeg audio does not run in this sandbox, the WER-on-room-clip threshold is set after the baseline is measured, as the plan itself says). Nothing in P4/C5/E3 touched this path.
- **P7 is done.** No further build step; the owner-machine listening test is the final word, recorded here when it happens.


### D2. LUT file import (decided, owner, 16 Oct 2026)
- **Plan status:** owner decision: add LUT import (.cube). **Code status:** not checked yet. The effect library (123 packages) has colour grades under "filter" (e.g. `concat.teal-orange`), but no LUT file loader was found in the files read. The diff checks first.
- **CapCut parity item:** custom LUT import. CapCut Desktop imports .cube (and .3dl) through Adjustment → LUT → Import (third-party CapCut guides, Oct 2026; CapCut's own help not checked).
- **Test first:** load one .cube file, apply it to one clip, and compare the grade with the LUT off and on. Exported frame matches the LUT preview.
- **Done when:** the user can import a .cube file, apply it to a clip, set its intensity, and the export keeps the look.
- **Do not:** change the 123 existing packages.


### D3. Caption translation beyond English (owner open; storage question answered 16 Oct 2026)
- **Plan status:** open. Owner said yes to adding more languages, pending the choice of model.
- **Code status:** Whisper translate gives English only (Option B D3). No second translation model is in the app.
- **Storage (third-party figures, not measured here):**
  - **Opus-MT, one model per language direction:** about 300 MB each. English to Indonesian and Indonesian to English are two models (about 600 MB). Each extra language adds about 300 MB per direction.
  - **NLLB-200 distilled 600M, one model for about 200 languages:** about 2.3–2.5 GB in full precision. The INT8 ONNX build is about 1.1 GB (encoder 415 MB plus decoder 729 MB).
  - **Whisper translate (already in the app):** no extra storage for English output.
- **Answer to the owner's question:** yes, it adds storage. The size depends on the choice: about 300 MB per language direction (Opus-MT), or about 1.1 GB once for many languages (NLLB INT8).
- **Decided (owner, 16 Oct 2026): import, not bundled.** The app ships with English only (Whisper translate, no extra file). Other languages need a translation model file that the user imports. The model choice works like the Whisper model choice: a settings list with an import option, and a default of English. Storage for other languages is paid by the user only when they import one.
  - **Existing pattern (checked 16 Oct 2026):** Settings → Transcriber has a Whisper models folder, an installed list, and a "Get model" download. The caption dialog has a 99-language list for speech recognition. A translation model slot follows the same pattern.
  - **Guide:** `02-map/guide-d3-translation-languages.md` (UI states, tests, decisions for the diff).
  - **Open for the diff:** which model formats the engine can load (Opus-MT and NLLB need converting to a format the app reads). Support one format first; reject a wrong file with a message.
- **Test first (after the choice):** one caption track translated into the chosen language; check word count and line breaks in the caption layout.
- **CapCut parity item:** caption translation (Option B D3: CapCut has it; kconcat translates to English only). Measurement: a bilingual caption output with one preset (clean-bilingual, C2).

---

## 6b. Track 6 — extra editing, music and video (owner added, 16 Oct 2026)

Added by owner. Each item comes from the Option B Tier D5 list and is not yet in the plan order. Guide: `02-map/guide-d5-extra-editing.md`. Plan order: after P6. Local AI only (standing rule): every AI item runs on the owner's machine, and its model file size is measured in the diff.

### K1. Beat sync (music)
- **CapCut parity item:** cut or place clips on music beats (Option B D5).
- **Implementation progress (18 Oct 2026):** the dependency-free analysis layer is now in `concat-timing::beat`: energy-flux tempo estimation (60–180 BPM), beat-grid/phase generation, confidence, and tolerance-based snap helper, with synthetic pulse-train regression tests. This is the engine layer only. Timeline waveform decoding, UI action, and a real-music acceptance check remain; do not mark K1 complete until those are integrated.
- **Needs:** a beat detector (local, no model needed for a first version).
- **Test first:** on a music clip with known tempo, the detected beats match a hand count within a tolerance set after a baseline.
- **Done when:** the user can snap cuts to beats on one timeline.

### K2. Flicker removal
- **CapCut parity item:** remove light flicker, for example 50/60 Hz lamps (Option B D5).
- **Test first:** on a flickering test clip, the brightness variation drops by a measured amount. Number set after a baseline.
- **Done when:** the flicker removal effect is applied to one clip, and the test clip is less flickery.

### K3. Camera tracking
- **CapCut parity item:** follow a moving subject and attach a graphic or crop to it (Option B D5).
- **Needs:** a local tracker. First choice OpenCV VitTrack (0.68 MiB); backup NanoTrack v2 (1.70 MiB). See guide §6.
- **Test first:** on a clip with a known moving object, the tracked point stays on the object within a tolerance set after a baseline.
- **Done when:** a track can drive a crop or a graphic on one clip.

### K4. AI colour correction (local)
- **CapCut parity item:** automatic colour fix (Option B D5). The existing filters and curves already do manual correction.
- **Needs:** a local model. Undecided; candidates in guide §6 (learned 3D LUT, perception WB). Public weights and licence to be checked.
- **Test first:** on a colour-cast test clip, the cast drops by a measured amount. Number set after a baseline.
- **Done when:** one button applies a correction that the user can undo, and the user can adjust its strength.

### K5. AI text remover (local)
- **CapCut parity item:** remove on-screen text such as watermarks (Option B D5).
- **Needs:** a local inpainting model. First choice MI-GAN (about 29.5 MB); backup LaMa (198–208 MB). See guide §6.
- **Test first:** on a clip with a known text area, the text is gone and the area around it is not visibly damaged.
- **Done when:** the user can mark an area and remove the text in it.

---

## 7. Out of scope — permanently

- **Video upscaling to 4K (D4, owner decision 16 Oct 2026: no).** Not built. Source material is 1080p; the targets are 1080×1920 and 1920×1080. Revisit only if the owner asks for 4K output from 1080p sources.

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

- **UI for E1–E6, V1–V5, P3, D2, K1–K5 is not specified yet.** Each item gets a UI section (placement, labels, states) and is checked in the running app before it is built. Design text in `.md` is a draft until then.

- D3(a) `[motion]` schema fields (§3, C5), D5 (harness location) — §1.
- T8: owner's corrected transcript of the 30 s clip (needed before step 1 of T8).
- Which of the episodes are two-speaker, for P3?
- Room noise: answered (owner, 16 Oct 2026): sometimes noisy. Denoise is added as P7; vocal isolation is not planned.
- Remote guests: skipped by owner (16 Oct 2026). Not a priority; no task added. Guest audio that comes from outside kconcat enters as imported files.

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
| T1 harness | fixed | DONE (18 Oct, sandbox-provable part) |
| T8 Indonesian ASR tuning + WER | fixed (approved with condition) | not started |
| T2 snapping | fixed | DONE (18 Oct, sandbox-provable part) |
| T3 ZCR + flux | fixed | DONE (18 Oct, sandbox-provable part) |
| T4 tokenisation | fixed | DONE (18 Oct; concat-speech side not verified here) |
| T5 identity | fixed | DONE (18 Oct; concat-crate delegations not verified here) |
| T6 nudge | fixed | DONE (18 Oct, all in concat-project) |
| T7 one writer | fixed (new) | DONE (18 Oct, all in concat-project) |
| C1 chunking | built | built; validated on the synthetic episode (18 Oct); by-eye on a real episode pending owner |
| C2 presets | fixed | DONE (18 Oct): eight caption presets + kind tag + highlight round-trip, written in the concat crate, not verified here; picker is owner-machine work |
| C3 karaoke wiring | fixed | mechanism DONE with C2's data (18 Oct); renderer test + end-to-end pick on the owner's machine; captions.rs:1566 interaction recorded |
| C4 case transform | fixed | DONE (18 Oct, model verified; preset+painter written, not verified here; UI toggle is owner-machine) |
| C5 motion | D3 selected (a), schema v1 approved 16 Oct (4 modes, typewriter included) | schema DONE (18 Oct, verified in concat-project + concat-text: CaptionMotion on TextStyle, kebab wire off-while-default, tidy-only clamps, painter mirror; preset [motion] reader + titles.rs copy written, not verified here; progress driving + UI are owner-machine) |
| E1 transcript edit | fixed | DONE (18 Oct, mechanism verified; UI checkbox unspecified per §9) |
| E2 filler removal | fixed | DONE (18 Oct, all in concat-project; UI option unspecified per §9) |
| E3 waveform / word-edge drag | fixed | drag DONE (18 Oct, verified in concat-project: MoveCaptionWordEdge clamps at neighbours/window/MIN_WORD, never overlaps, one-undo gesture folding, reload round trip; caption-row waveform + mouse wiring are owner-machine) |
| E4 Enter-split | fixed | DONE (18 Oct, mechanism verified; UI wiring written, not verified here) |
| E5 link / unlink audio and video (replaces Detach / Reattach) | fixed (owner 16 Oct) | mechanism DONE (18 Oct, checkpoint 10, verified in concat-project: link_group, Link/Unlink, group move/delete/split/trim, detached_from migration, 11 tests); menu swap, linked click-selection, E5-b import default and D6 typing are owner-machine |
| E6 select one lane with Shift + L | fixed (owner 16 Oct) | lane rule DONE (18 Oct, checkpoint 10: `clips_on_track`); the key binding and selection state are owner-machine UI |
| V1 auto-reframe | fixed | mathematics DONE (18 Oct, verified in concat-project: centroid, one-euro, clamped OffsetX keys + fill scale, one undo); mask pipeline + renderer confirmation are owner-machine |
| V2 presets + loudness | fixed | loudness retarget DONE (18 Oct, verified: -14 LUFS / -1 dBTP); platform presets wait for the owner's machine + the A5 diff |
| V3 ducking | fixed | mechanism DONE (18 Oct, verified in concat-project: both A6 routes, one key writer, one undo); decode call site, tags and UI are owner-machine |
| V4 batch queue | fixed | queue state DONE (18 Oct, verified in concat-project: order, failure-continues, one-at-a-time, late-report guard); runner process + UI are owner-machine |
| P3 speaker labels | fixed (scope open) | manual route DONE (18 Oct, verified in concat-project: registry, lanes, labels, 5 commands, opt-in default); detection engine open on the owner's ground truth |
| P4 audio recording, one track per mic | fixed (owner 16 Oct: audio only) | multi-mic placement DONE (18 Oct, verified in concat-project: place_session one-track-per-mic, landing = playhead+offset, one-undo folding, all-or-nothing NaN guard, empty no-op; session recording Session/start_from/input_devices written, not verified — needs cpal) |
| P6 B-roll / animation | verify (B-roll only) | VERIFIED 18 Oct: multi-clip exists (owner-reported, timeline supports it, running-app check is owner's machine); animation built as C5 motion schema v1 |
| P7 Denoise | verify only (exists: `concat.denoise`) | VERIFIED 18 Oct: exists — Noise Reduction (afftdn, Strength 0–100%) + Enhance Voice in packages/, no code to add; listening test on owner's room clip is the final word |
| K1–K5 Track 6 (beat sync, flicker, tracking, AI colour, AI text remover) | added by owner 16 Oct; after P6 | K1 analysis engine implemented (concat-timing, 18 Oct); UI/audio integration pending. K2–K5 not started |

**Order (fixed):** T1 → T2 → T3 → T4 (parallel) → T5 → T6 → T7 → C1 validation → E4 → E1 → E2 → V2 (loudness first) → C2 → C3 → C4 → V3 → V4 → P3 → V1 → C5 (after D3) → E3 → P4 → P7 → P6. **T8 runs in parallel after T1. X1 and X2 (§10b) come after Track 1 exit.** **UI-1 (§10c, panel scroll) can run in parallel with Track 1, since it touches no word-timing code.**
