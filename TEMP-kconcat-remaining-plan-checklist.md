# TEMP — kconcat remaining-plan checklist

**Draft:** v0.1 — 19 October 2026, Asia/Bangkok  
**Purpose:** an editable temporary checklist for finishing the 22 grouped milestones, with a parked AI-integrated-editing idea afterward.  
**Location:** separate from the source checkout. This file does not change the official roadmap, modify code, create a Git commit, or expire automatically. Promote, revise, or delete it when you choose.

## How to use this file

- Mark a completed check by changing `[ ]` to `[x]` in a text editor.
- Tick a milestone in the overview **only after its completion criteria pass**. A helper function or passing unit test alone does not close a user-facing feature.
- Save the command, output, commit, and real-media result for each milestone.
- Recheck the current local code before implementing anything: some foundations already exist, and your Windows checkout has changes beyond the earlier reviewed snapshot.
- Work on one small change at a time. These are 22 **work packages**, not 22 equal-sized coding tasks or an estimate in days.
- Extras and the AI idea are separate from the 22. They are not automatically approved new scope.

**Suggested immediate order:** finish the remaining checks in **01**, fix/verify the measurement path in **02**, save the real baseline in **03**, then close the timing/ASR gate in **04**. You can prepare the source video and corrected transcript while 01–02 are being handled. Do not discard working later-feature code merely because its acceptance is still pending.

---

## A. Current evidence — already credited

### New evidence from your attached `test.txt`

The supplied Windows log shows:

| Check | Result from the supplied log |
|---|---|
| Focused five-crate tests | **532 passed, 0 failed, 0 ignored**, including 2 documentation examples. |
| `concat-project` | **347** total: 264 unit + 83 integration tests. |
| `concat-timing` | **27** tests. |
| `concat-effects` | **72** tests. |
| `concat-text` | **31** tests. |
| `concat-core` | **53** tests + **2** documentation examples. |
| `cargo check --locked -p concat` | **Passed**; finished successfully in the supplied Windows log. |
| Actual `cargo dev` launch | Not shown in this attachment. Earlier repository notes report a successful launch, but this log does not establish the current checkout's launch result. |
| Real transcription/timing baseline, GUI acceptance, export, microphone test | Not established by this attachment. |
| Commit and uncommitted changes for the run | Not shown. Record them before making changes. |

**Important update:** the log includes these three passing beat tests:

- `beat::tests::silent_audio_has_no_beat_grid`
- `beat::tests::snaps_only_inside_the_requested_window`
- `beat::tests::finds_regular_120_bpm_pulses_within_tempo_tolerance`

Therefore, **step 16 now has evidence of a tested local beat-related foundation**. Do not label it completely unbuilt or write a second detector without inspecting your current implementation. These tests do not yet prove real-music beat quality or timeline/UI integration.

The earlier public-source review was at `46a2f08aae3c45f866b5afeae66961ab4c97dee7`, where the focused suite totaled 529. Your new log totals 532 and contains the three beat tests. Its exact revision is unknown; do not assume it is identical to that snapshot.

**Evidence source:** `uploads/test.txt`, received in this conversation. These Windows results were read from your supplied log, not independently run on your computer by the assistant.

---

## B. Overview — the 22 milestones

An unchecked milestone can contain substantial completed code. The checkbox means **full acceptance is still to be recorded**, not “nothing has been built.”

### Core workflow — 01–14

- [ ] **01 — Build/check health.** IN PROGRESS; Windows focused tests and app compilation check passed.
- [ ] **02 — Trustworthy timing comparison.** Recheck/fix baseline provenance before claiming snapping improvement.
- [ ] **03 — Real corpus and baseline.** Save reproducible real-media evidence.
- [ ] **04 — Timing and Indonesian ASR acceptance.** Close the real-audio quality gate.
- [ ] **05 — Caption appearance and panel access.** Finish existing caption foundations and their controls/rendering.
- [ ] **06 — Transcript-editing controls.** Connect and validate editing mechanisms.
- [ ] **07 — Typed lanes and linked editing.** Finish import, migration, selection, and Link/Unlink behavior.
- [ ] **08 — Upload-ready platform exports.** Finish presets and measured final-mix loudness.
- [ ] **09 — Auto ducking.** Connect both routes and verify the listening/export result.
- [ ] **10 — Batch-export execution.** Connect queue state to the separate encoder process and UI.
- [ ] **11 — Auto-reframe.** Connect analysis to crop keys and verify real vertical output.
- [ ] **12 — Speaker workflow.** Finish opt-in detection, labels, correction, lanes, and linking.
- [ ] **13 — Multi-microphone recording.** Finish controls and measure real-device synchronization.
- [ ] **14 — Full core-workflow acceptance.** Complete the long-form scenario and existing-feature smoke tests.

### Extensions — 15–22

- [ ] **15 — Caption translation beyond English.** Imported local model and bilingual output.
- [ ] **16 — Beat sync.** PARTIAL; three passing beat tests are evidenced in the supplied log.
- [ ] **17 — Flicker removal.** User-facing effect plus measured improvement.
- [ ] **18 — Camera/object tracking.** Local tracking drives a crop or graphic.
- [ ] **19 — Local AI color correction.** Evaluated, adjustable, reversible correction.
- [ ] **20 — Local AI text removal.** Marked-area removal with acceptable temporal quality.
- [ ] **21 — English regression and fillers.** Real English checks and a separate opt-in list.
- [ ] **22 — Japanese real-clip validation.** Usable units and synchronization, with segmentation limits stated.

English/Japanese work may move forward after step 04 if needed, without delaying the Indonesian foundation. The order here is a practical grouping, not a replacement for each task's approved dependencies.

---

## C. Detailed checklist

### 01 — Establish build/check health

**Why:** know what works on your actual computer before changing more code.

- [x] Supplied Windows log: focused five-crate suite passed **532 tests**.
- [x] Supplied Windows log: `cargo check --locked -p concat` passed.
- [ ] Record `git rev-parse HEAD` and `git status --short` for this checkout; preserve local changes.
- [ ] Run current formatting and focused Clippy checks; save results and resolve failures in small, reviewable changes.
- [ ] Launch the current checkout with `cargo dev`; record successful opening and a basic project-open/save check.
- [ ] Recheck current CI, if applicable. Earlier Linux/macOS setup failures and the Windows workspace-test failure are historical findings, not proof of today's local failure. Diagnose each separately.
- [ ] Verify Unix script executable modes/direct invocation where those workflows are retained; track any deliberately deferred non-Windows issues explicitly.

**Complete when:** the relevant build/test/lint checks and current app launch have evidence; remaining platform issues are fixed or explicitly scoped and tracked. A focused suite is not the full workspace/native test suite.

**Evidence:** `test.txt` already covers the first two items. Add commit, formatting/Clippy logs, launch result, and any CI report.

### 02 — Make the timing comparison trustworthy

**Why:** an improvement cannot be measured honestly if the “before” data already includes that improvement.

- [ ] Inspect the **current local** `make_cache` → transcription → snapping path. Confirm whether the earlier baseline-provenance problem still exists.
- [ ] Provide/verify an evaluation path that preserves word times **before onset snapping**. Keep normal app behavior unchanged unless deliberately approved.
- [ ] Add a regression test proving baseline-cache creation does not already apply the evaluated snapping pass.
- [ ] Record model, actual backend, language, relevant settings, processing mode, clock/time range, and source identity with measurement evidence.
- [ ] Provide a controlled Level-1-only vs Level-1.5 comparison for T3. Do not invent CLI switches that have not been implemented.
- [ ] Document the exact commands and verify the native cache path on Windows, not only the pure metric runner.

**Complete when:** the same source timing/audio can produce a genuine before/after comparison, with tests and recorded provenance.

### 03 — Prepare the real corpus and baseline — T1

- [ ] Locate the original five-minute source video/audio. The SRT alone is not the source.
- [ ] Choose the exact 30-second reference slice and record its source start/end.
- [ ] Listen and correct its transcript; ensure reference text and hypothesis cover the **same audio range**.
- [ ] Pick three known-bad timing segments. Reserve one as a holdout before tuning; tune on the other two.
- [ ] After step 02 is verified, generate the real unsnapped cache/audio pair and save the initial baseline. Keep five-minute and 30-second audio/cache pairs in separate directories.
- [ ] Preserve the original baseline unchanged; save commands, settings, relevant file hashes, and terminal output.
- [ ] Keep private source media out of the public repository unless publication is permitted.

**Complete when:** a real, reproducible baseline exists, and the corrected reference matches the exact input. A synthetic fixture remains useful but does not replace this baseline.

### 04 — Close timing and Indonesian ASR acceptance — T2–T8

- [ ] Label/measure the reference onsets; runner plausibility and WER do not automatically provide ground-truth timing error.
- [ ] Evaluate T2: after gaps ≥80 ms, median absolute error <60 ms and p90 <120 ms; mid-phrase no worse than baseline; mean signed error within ±15 ms.
- [ ] Reach **0 timing-order/overlap violations and 0 outliers** on the required accepted inputs. Recheck the earlier synthetic residual-overlap case rather than hiding it.
- [ ] Evaluate T3 against T2 alone. Show improvement or record the roadmap's allowed “no gain” decision.
- [ ] Run real edit/sync regressions for T4–T7: tokenization, caption identity, nudge behavior, text/word consistency, save/reload, and Burn Captions OFF where relevant.
- [ ] Compare installed multilingual ASR models/settings for Indonesian; record WER and runtime using corrected reference text. Agree the quality target after the baseline.
- [ ] Check karaoke at 30 fps against the roadmap's roughly <5% wrong-word-frame criterion, then evaluate the untouched holdout once.

**Complete when:** timing and recognized-text quality have real evidence and chosen settings are recorded. Do not start fine-tuning by default; that requires a separate decision if the bake-off still fails.

### 05 — Finish caption appearance and panel access — C1–C5, T6 UI, UI-1

**Existing foundation:** chunking, preset data, highlight fields, transforms, motion schema, and nudge mechanism. These are not all completed user-facing workflows.

- [ ] Validate real Burst chunking before/after timing changes; do not build another layout UI unnecessarily.
- [ ] Connect the eight caption looks to a usable caption picker; verify preset application to the intended captions/track.
- [ ] Verify karaoke ON/OFF behavior for every preset, including no later default silently overriding OFF.
- [ ] Verify uppercase in preview/export while stored transcript text retains its casing.
- [ ] Drive pop/reveal/typewriter from word timings and connect controls; verify preview/export/reload. Motion data storage alone is not animation.
- [ ] Expose the tested ±10 ms track-nudge mechanism and verify re-transcription behavior.
- [ ] Resize every panel to minimum usable size; reach all controls; confirm scroll wheel, scrollbar, numeric-control drags and keyframe controls still work. Recheck the shared PaneScroll change and panel-specific cases.

**Complete when:** the intended caption look/motion is usable without hidden workarounds, exported output agrees with preview, and smaller panels remain usable.

### 06 — Finish transcript-editing controls — E1–E4

**Existing foundation:** tested delete/filler/word-edge/split commands; latest audited source also had Enter wiring and needs runtime verification, not a duplicate path.

- [ ] Connect the “cut video when a word is deleted” choice to the intended editing surfaces.
- [ ] Connect opt-in Indonesian strong/weak filler groups; preserve meaningful-use safeguards and preview/undo. A literal list is not a semantic understanding of filler use.
- [ ] Connect caption-row waveform drawing and word-edge mouse gestures to the existing edge-edit command.
- [ ] Verify Enter splitting in both Sections and the caption draft/editor; use heard word boundaries and refuse unavailable timing rather than guessing from text length.
- [ ] Verify one-step undo, audio/picture/caption synchronization, and save/reload/export after each relevant edit path.

**Complete when:** a real captioned clip survives these edits without stale words, lost identity, unintended cuts, or export timing changes.

### 07 — Finish typed lanes and linked editing — E5/E6, D6/D7

- [ ] Inspect current lane/group/import code before adding anything; define typed V/A lanes and legacy mixed-lane migration with fixtures.
- [ ] Implement/verify imported video + separate linked audio on the intended V/A lanes, without doubled/missing audio.
- [ ] Replace the intended Detach/Reattach menu workflow with Link/Unlink and connect linked click selection.
- [ ] Test linked move, trim, split, delete, relink, caption-following, one-step undo, old-project load, and export equivalence.
- [ ] Resolve the single-video/multiple-independent-speaker-group semantics explicitly; the audited `link_group` field stored one group per clip.
- [ ] Verify the existing Shift+L path, current-lane rule, locked lanes, second-press/Escape behavior, and shortcut help. Check any required selection-history behavior.

**Complete when:** new and old projects meet the agreed lane/link behavior through real edits and export, and there is one coherent selection/linking implementation.

### 08 — Complete upload-ready exports — V2/P2

- [ ] Connect four platform choices: TikTok, Reels, Shorts, YouTube. Confirm dimensions/defaults before implementing unresolved settings.
- [ ] Implement/verify the approved frame-rate, bitrate-mode/target/maximum, and audio controls against the actual export engine.
- [ ] Apply final-mix normalization when a platform preset requests it; preserve normal exports' user-set levels.
- [ ] Verify per-clip loudness behavior separately from whole-mix normalization.
- [ ] Measure real output: **−14 LUFS integrated ±1 LU** and **true peak ≤−1 dBTP**; verify aspect, frame rate, audio and preview/export agreement.

**Complete when:** a platform choice creates a verified upload-ready file, not merely a correctly formatted filter string. Do not silently change sequence settings just to change export size.

### 09 — Finish auto ducking — V3

- [ ] Connect the tested keyframe writer to both audio-analysis and transcript-timing routes.
- [ ] Connect Music/Dialogue roles and the route/target selection controls.
- [ ] Expose the agreed editable amount/fade/sensitivity controls as applicable.
- [ ] Listen to real dialogue with a music bed; check gaps, intro/outro, natural recovery, preview and export.
- [ ] Verify editable keyframes, one undo, disabling/re-running behavior, and no accidental gain changes outside the intended region.

**Complete when:** ducking is usable, natural, editable, reversible, and consistent in exported audio.

### 10 — Finish batch-export execution — V4

- [ ] Reuse the existing tested queue state; connect actual export requests and a separate encoder process.
- [ ] Connect Add to Queue, order/reorder, progress, failure reason, and completion UI.
- [ ] Verify one encoder at a time while editing can continue.
- [ ] Run five real jobs in chosen order; deliberately fail one and confirm later jobs continue.
- [ ] Verify cancellation/process exit, output-file handling, late-report protection and no silent overwrites.

**Complete when:** five real exports finish without babysitting, with failures visible and no blocked editing.

### 11 — Finish auto-reframe — V1

- [ ] Connect real subject masks/points to the tested smoothing/crop-key mechanism.
- [ ] Verify renderer sign/units, fill scale, bounds and sequence/export interaction.
- [ ] Expose the agreed control and reversible/manual-adjustment behavior.
- [ ] Test real talking-head movement, edge cases and no-subject intervals; define handling rather than guessing.
- [ ] Export 16:9 footage to 9:16 and measure/inspect jitter and framing against the recorded baseline/tolerance.

**Complete when:** a real subject remains acceptably framed in the vertical export without manual keyframing for the test case.

### 12 — Finish the speaker workflow — P3

- [ ] Prepare 2–3 minutes of speaker-turn ground truth and test candidate local detection engines; record quality, speed, model size and license.
- [ ] Connect opt-in detection for mixed-stream recordings; detection OFF must leave the simple workflow unchanged.
- [ ] Connect manual assignment/rename/color and verify corrections survive save/reload/reprocessing according to the agreed rule.
- [ ] Split captions at speaker changes and verify the required speaker labels/lanes, supported speaker count, and manual overlap handling.
- [ ] Confirm separated-file vs multistream import behavior in the running app before adding a second import workflow.
- [ ] Implement/test the agreed video/audio/caption relationship semantics from step 07.

**Complete when:** real two-speaker material has useful labels and corrections, and the supported overlap/import/linking limitations are explicit.

### 13 — Finish multi-microphone recording — P4

- [ ] Reuse existing placement/session foundations; connect device selection, meters and session start/stop.
- [ ] Verify one microphone remains simple and audio-only scope is preserved.
- [ ] Record two actual microphones for five minutes; clap near the start and end and measure alignment/drift.
- [ ] Handle unsupported combinations, device loss, cancellation, file errors and cleanup without corrupting the project.
- [ ] Verify separate tracks, retained source files, playback, one-step placement undo and usable exported audio.

**Complete when:** two physical devices produce useful separate tracks that meet the agreed synchronization tolerance throughout the take. Matching timeline starts alone is not proof of matching device clocks.

### 14 — Prove the complete core workflow — P1/P5/P6/P7/LUT D2

- [ ] Complete a real **60-minute episode → three 60-second vertical clips + one clean full export**, offline.
- [ ] Count manual corrections and record runtime, peak memory, disk/temp-file behavior and failures; compare with the baseline where applicable.
- [ ] Verify existing two-clip/B-roll composition and export; do not rebuild a feature that already works.
- [ ] Verify existing denoise/Enhance Voice on room audio and check speech remains natural.
- [ ] Import an existing `.cube` LUT through the existing UI; verify application/intensity and preview/export frame agreement.
- [ ] Reopen the saved project, check relevant edits/links/styles/timings, and rerun the regression suite.

**Complete when:** the principal long-form-to-shorts workflow and existing-feature checks have real outputs and evidence. A synthetic fixture or SRT alone does not satisfy this milestone.

### 15 — Caption translation beyond English — translation D3

- [ ] Choose one supported imported-model format and verify its license, storage and local runtime needs.
- [ ] Connect model import/validation/settings and reject wrong or incompatible files with clear messages.
- [ ] Translate a real caption run while retaining source timing and preserving the original transcript.
- [ ] Verify bilingual layout, line wrapping, save/reload and export with the intended caption preset.
- [ ] Keep English-only Whisper translation and ordinary recognition behavior intact; document offline/model limits.

**Complete when:** a useful non-English target translation is locally reproducible and correctly displayed/exported.

### 16 — Beat sync — K1

**Updated evidence:** your supplied Windows log includes three passing beat tests. The implementation may be local/uncommitted or from a newer revision; inspect it first.

- [x] Supplied log: silence produces no beat grid.
- [x] Supplied log: snapping is limited to the requested window.
- [x] Supplied log: regular synthetic 120 BPM pulses pass a tempo-tolerance test.
- [ ] Record the exact code revision and inspect the current detector, tempo/phase representation, and snapping helper. Do not build a duplicate.
- [ ] Evaluate real music against hand-marked beats, including weak/changing rhythm; record the baseline and an agreed tolerance.
- [ ] Connect beat display and the agreed timeline cut/placement snapping behavior; preserve non-beat editing when disabled.
- [ ] Verify undo, clip speed/time mapping, save/reload if beat data is stored, and actual preview/export cut locations.

**Complete when:** a user can snap cuts/placements to beats on real music with acceptable measured quality. Synthetic beat tests alone do not close K1.

### 17 — Flicker removal — K2

- [ ] Check whether the intended effect already exists in the current local UI/engine before implementing it.
- [ ] Prepare real or controlled flickering footage and measure brightness variation before treatment.
- [ ] Connect an appropriate local effect and control path, with bypass and adjustable behavior as agreed.
- [ ] Measure improvement without unacceptable exposure pumping, lost detail or changed intentional lighting.
- [ ] Verify preview/export agreement and normal editing/undo behavior.

**Complete when:** the user-facing effect improves the measured test case and preserves acceptable image quality.

### 18 — Camera/object tracking — K3

- [ ] Choose/test a local tracker and verify model format, license, file integrity, speed and hardware needs.
- [ ] Provide object/region selection, analysis progress and clear lost-track behavior.
- [ ] Store/use a track to drive an agreed graphic or crop; preserve source/timeline mapping.
- [ ] Evaluate tracking on a known moving object with an agreed baseline/tolerance.
- [ ] Verify manual correction, strength/offset where applicable, undo, save/reload, preview and export.

**Complete when:** a real tracking result reliably drives the intended crop/graphic, with limitations exposed rather than hidden.

### 19 — Local AI color correction — K4

- [ ] Choose a model only after checking weights, license, local runtime and actual-machine memory/storage cost.
- [ ] Define a color-cast/quality test and record the uncorrected baseline.
- [ ] Connect local analysis/inference to a user-controlled, reversible correction.
- [ ] Provide bypass and adjustable strength; preserve source/color-space handling.
- [ ] Compare preview/export and evaluate correction quality on real footage without claiming every grade is objectively better.

**Complete when:** a useful local correction is measured/inspected, adjustable, and reversible.

### 20 — Local AI text removal — K5

- [ ] Choose/test a local model and verify license, model import/integrity and actual-machine resource cost.
- [ ] Connect marked-area selection and processing over the intended frame/time range.
- [ ] Evaluate surrounding-image damage and temporal flicker on real video, not just one still frame.
- [ ] Preserve originals and provide cancel/bypass/undo or another explicitly agreed non-destructive workflow.
- [ ] Verify saved-project references, preview/export agreement, missing-model behavior and temp-file cleanup.

**Complete when:** the selected text area is removed with acceptable image and temporal quality. Use test material you own or have permission to modify.

### 21 — English regression and fillers — X1

- [ ] Add a real English clip/cache with a corrected, same-range reference.
- [ ] Run before/after timing and recognition checks so Indonesian improvements do not silently regress English.
- [ ] Inspect current local code, then add/verify an English filler list separate from Indonesian.
- [ ] Test opt-in behavior and meaningful uses of words such as “like”; do not remove every dictionary match blindly.
- [ ] Verify real editing, undo, caption sync and export, and make the regression easy to rerun.

**Complete when:** the English regression is reproducible and the filler workflow works with negative cases.

### 22 — Japanese real-clip validation — X2

- [ ] Add a real Japanese clip/cache and corrected reference.
- [ ] Verify actual native transcription produces multiple usable units instead of one whole-cue unit.
- [ ] State the current limit: character units with estimated intra-group timings are not dictionary-level word segmentation.
- [ ] Run the full caption sync matrix through representative edits, save/reload and export.
- [ ] Record whether quality is sufficient; Japanese WER/model tuning and romaji/karaoke are separate decisions, not silently included work.

**Complete when:** spoken Japanese captions meet the agreed usable-unit/synchronization requirement on real material, with honest limits.

---

## D. Cross-cutting completion checks — use these inside each milestone

These are quality checks for the existing plan, **not additional numbered feature milestones**.

- [ ] Changes are small/reviewable; local work and source media are preserved.
- [ ] The real UI → handler → command → preview/export path was checked before calling something missing or complete.
- [ ] Relevant automated tests and Clippy pass; UI changes compile with `cargo check -p concat` on the configured machine.
- [ ] Save/reload, undo, wrong input, no-op and cancellation behavior are tested where relevant.
- [ ] Caption identity survives edits; text and word timings are consistent; words are ordered/in bounds; highlights locate the intended words.
- [ ] Preview and exported media agree where the task affects output.
- [ ] Models/dependencies have appropriate licenses and integrity checks; download/import happens only as agreed.
- [ ] The promised workflow runs offline with required models already present; no private media is uploaded silently.
- [ ] Memory, model storage, temporary files and long-run performance are measured on the actual computer, not guessed.
- [ ] Evidence records the exact revision/settings and distinguishes library tests, app compilation, runtime checks and acceptance.

### Optional extras — parked, not approved additions to the 22

- [ ] **EX-01 — Recovery polish:** inspect existing recovery/backup support first; consider autosave/recovery improvements only if the owner wants them.
- [ ] **EX-02 — Diagnostics bundle:** consider a privacy-safe “export diagnostics” action for build/version/settings/logs, excluding private media and credentials.
- [ ] **EX-03 — Accessibility/usability polish:** keyboard reachability, legible focus, progress/cancel/error messages and small-screen behavior; avoid an unsolicited redesign.
- [ ] **EX-04 — Portable project handoff:** investigate missing-media/model/font handling before proposing packaging changes.
- [ ] **EX-05 — Stabilization V5:** schedule only if handheld footage creates a real need.
- [ ] **EX-06 — Romaji/Japanese karaoke:** remains a separate parked design/experiment, not part of step 22.

These are ideas for later discussion. Do not start them, download models, or expand scope automatically.

---

## E. AFTER THE 22 — parked AI-integrated editing discussion

**User interest:** an integrated AI editing experience, possibly inspired by “Antigravity” or “Hermes.”  
**Status:** DISCUSSION BACKLOG ONLY. No product/framework/model selected; no integration or compatibility verified. The exact projects/versions meant by those names must be confirmed later. Their availability as a chat-platform connector is separate from whether kconcat could integrate an agent.

**Start condition:** the 22 milestones have the agreed completion evidence, and the owner explicitly asks to design this next phase. This section is not an automatic step 23 or permission to implement a new agent now.

### Working interpretation — confirm later

The proposed idea is **AI that helps edit a kconcat project**, not only an AI coding assistant that modifies Rust source.

Possible request:

> “Turn this episode into three one-minute vertical clips, propose the important parts, apply my caption look, and show the changes before you apply them.”

Or:

> “Buat tiga shorts 60 detik. Jangan ubah makna pembicaraan. Tampilkan rencana dan preview dulu.”

The two jobs should remain distinct:

| Kind of assistant | What it changes | Boundary |
|---|---|---|
| Coding assistant | Source code/build configuration | Developer workflow; not automatic authority to edit source from the in-app editor. |
| Editing assistant | Project/timeline/captions/effect settings | Uses validated editing operations, with preview/approval/undo. |

### Questions for the later discussion

- [ ] Is the desired experience an in-app chat panel, an external agent controlling the editor, or both?
- [ ] Which exact Antigravity/Hermes products, repositories or examples are the references?
- [ ] Which first three editing actions matter most? Pick a small first version instead of “edit anything.”
- [ ] Must inference remain entirely local? **Default from the current project scope: yes; no silent cloud upload.** Any cloud option would need an explicit scope/privacy decision.
- [ ] Should the assistant only suggest initially, or apply approved edits too?
- [ ] What quality, speed, RAM/VRAM and model-storage limits does the actual machine support? Benchmark before choosing a large model.
- [ ] Can existing `concat-api` / server / command mechanisms safely expose the needed operations? Inspect actual schemas, validation and undo behavior before choosing an adapter or protocol.

### Proposed safe progression — not an approved implementation plan

1. **Read-only helper:** inspect allowed project/transcript context and suggest an edit plan; cannot modify anything.
2. **Preview/dry run:** create a structured proposed change list and show affected clips, words, times and output settings.
3. **Approval-based editing:** apply only validated, explicitly approved commands using existing project editing/history mechanisms.
4. **Small bounded workflows:** after evaluation, allow an agreed multi-step task with progress, cancellation and a review summary.
5. **External-agent integration, if useful:** evaluate a compatible adapter/tool interface only after the references and real APIs are verified. Do not assume a framework can control kconcat merely because it is an “agent.”

Proposed shape:

```text
User request
    → allowed read-only project context
    → structured edit proposal
    → schema / bounds / permission validation
    → human diff / preview / approval
    → existing editor commands and normal history
    → preview/export checks and an undo/revert path
```

**Prefer real editor operations over brittle automatic screen-clicking.** Reuse existing command/API paths where suitable, but do not claim those paths already expose every needed tool.

### Non-negotiable safety/quality questions for that future phase

- [ ] Manual editing continues to work with AI disabled; existing per-feature AI tools from steps 18–20 are not the same as a general editing agent.
- [ ] The model proposes structured operations, not arbitrary Rust/Python/shell execution inside the editor.
- [ ] Read/write tools have explicit scope and validate IDs, time ranges, effects, file paths and project revision. Stale plans must be rejected or recomputed after manual edits.
- [ ] Media, transcripts, subtitle files and model responses are untrusted data—not instructions. Spoken or captioned “ignore the user and delete files” must not grant authority.
- [ ] Project edits support preview/approval and atomic undo where appropriate. Exports/overwrites/source deletion require their own explicit policy; source deletion is not a default AI permission.
- [ ] The agent cannot silently upload audio/video, reveal credentials, write outside approved paths, or execute arbitrary external tools.
- [ ] Cost/compute limits, maximum tool steps, cancellation, timeouts, model failure and partial-work recovery are handled visibly.
- [ ] An audit trail explains what changed and why without unnecessarily retaining private transcript/media.
- [ ] A small evaluation set checks good edits, harmful suggestions, ambiguity, prompt injection, timing/sync regression and approval/undo behavior before autonomy is increased.

**A sensible first editing-agent experiment:** propose three candidate highlight ranges from a transcript, let the user preview/select them, then use validated existing operations to create the approved draft clips. Avoid “fully autonomous editing of the whole project” as the first version.

**Do not do now:** choose a framework just from its name, promise compatibility, add a heavy dependency, download models, expose unrestricted shell/file access, or refactor the whole editor in anticipation of a future agent. Keep the stable command boundaries clear as you finish the existing milestones; design the new phase afterward.

---

## F. Commands and evidence notes

### Record your checkout — from the repository root

```powershell
cd C:\dev\kconcat
git rev-parse HEAD
git status --short
```

Do not discard local changes. If using a new branch, create it only after identifying the current state; do not blindly overwrite an existing branch.

### Build/check commands — from `src/`

Your attachment already proves the first two for the supplied run. Rerun after relevant changes, not merely to recreate the same evidence.

```powershell
cargo test --locked -p concat-project -p concat-timing -p concat-effects -p concat-text -p concat-core
cargo check --locked -p concat
cargo fmt --all -- --check
cargo clippy --locked -p concat-project -p concat-timing -p concat-effects -p concat-text -p concat-core --all-targets -- -D warnings
cargo dev
```

Run one line at a time. A test-count change is not automatically a regression: record which tests were added/removed and why.

**Small correction to the attached launcher banner:** `cargo dev -- -j 2` passes `-j 2` to the application rather than limiting Cargo build jobs. If you actually need to reduce build parallelism, use the explicit command:

```powershell
cargo run --bin concat --profile quick -j 2
```

Do not assume every freeze/build failure is a RAM problem, and do not delete the entire build cache or reinstall dependencies without diagnosing the first relevant error.

### Evidence note template — copy under a milestone when closing it

```text
Milestone / subtask:
Date:
Commit:
Uncommitted changes present:
Files changed:
Automated command(s) and result:
Native compile check:
App/runtime check:
Source media + exact time range:
Model/backend/settings, if relevant:
Reference/holdout used:
Before vs after measurement:
Preview/export/undo/reload result:
Evidence file(s):
Known limitations / deferred issues:
Next small action:
```

### Next-session handoff template

> Use TEMP-kconcat-remaining-plan-checklist.md as a working checklist, not proof that a feature exists. Read the actual current code and exact revision before editing. Preserve local changes. The supplied Windows log showed 532 focused tests passing and cargo check -p concat passing; it did not establish current app launch, real export, or timing acceptance. Work only on milestone [number] / subtask [name]. Reuse tested foundations, keep the patch small, run relevant tests/Clippy/native checks, record limitations and evidence, and update only the checks that actually passed. Do not start the parked AI-agent phase or optional extras.

## G. References and revision log

- Detailed earlier audit: `kconcat-roadmap-review-2026-10-19.md` in this workspace.
- Official scope/dependencies: [roadmap v2 at the earlier reviewed snapshot](https://github.com/itsukaitsutsu/kconcat/blob/46a2f08aae3c45f866b5afeae66961ab4c97dee7/02-map/kconcat-roadmap-v2-execution.md). Inspect newer/local source before relying on old status prose.
- New Windows evidence: `uploads/test.txt` in this workspace. Keep the raw log private unless you choose to share it.

| Date | Revision | Change |
|---|---|---|
| 2026-10-19 | v0.1 | Created 22-step working checklist; credited supplied 532-test Windows run and app compile check; noted partial beat foundation; parked optional extras and post-22 AI-editing discussion. |

**Immediate next action:** record the exact local commit/status, finish the remaining milestone-01 checks, and inspect the current baseline-generation path for milestone 02. Preparing the real 30-second reference can happen in parallel. No need to redo the already-passed focused test run unless code or environment changes.
