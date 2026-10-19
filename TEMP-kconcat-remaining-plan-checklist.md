# TEMP — kconcat remaining-plan checklist

**Draft:** v0.12 — 19 October 2026, Asia/Bangkok  
**Purpose:** an editable temporary checklist for finishing the 22 grouped milestones, with a parked AI-integrated-editing idea afterward.  
**Location:** separate from the source checkout. This file does not change the official roadmap, modify code, create a Git commit, or expire automatically. Promote, revise, or delete it when you choose.

## How to use this file

- **The assistant will maintain this workspace copy for you as you provide results. You do not need to edit it yourself.** Updates are based on evidence, not automatic access to your PC.
- If you download a copy, it will not automatically synchronize with later updates here. Download the updated file when you need a current local copy. You can optionally edit a downloaded Markdown copy in a text editor, but that is not required.
- Tick a milestone in the overview **when that work package’s stated completion criteria pass**, and update its detailed checks at the same time. Keep infrastructure implementation, real-corpus execution, and actual editing/quality acceptance distinct. A passing unit test alone does not close a user-facing feature.
- **Boundary clarification in v0.7:** milestone 02 closes the timing-comparison infrastructure after implementation and Windows-native verification; milestone 03 owns its first real-media cache/baseline and end-to-end provenance check; milestone 04 owns measured recognition/timing-quality acceptance. The earlier list duplicated the real-cache handoff in 02 and 03. It is now tracked once in 03, not waived or claimed passed.
- Save the command, output, commit, and real-media result for each milestone.
- Recheck the current local code before implementing anything: some foundations already exist, and your Windows checkout has changes beyond the earlier reviewed snapshot.
- Work in **focused feature batches**, not one terminal command per conversation turn. The assistant prepares and checks a patch here; the owner runs one grouped Windows verification where needed. Do not repeat passed checks or gate feature work on optional backup/Git housekeeping. These are 22 work packages, not equally sized tasks or an estimate in days.
- Extras and the AI idea are separate from the 22. They are not automatically approved new scope.

## Quick progress dashboard — work already completed

The unchecked 22-item overview tracks full acceptance and hid real partial progress. It does **not** mean no work has been completed.

- [x] **Windows focused-test checkpoint:** 532 tests passed in the supplied log.
- [x] **Windows app-compilation checkpoint:** passed before the patch in `test.txt` and again **after the patch** in the new `test2.txt`.
- [x] **Checkout identification:** base commit, local status and tracked diff summary received.
- [x] **Milestone-02 implementation package prepared:** five-file timing-baseline patch, without changes to the owner's beat/lib/caption/scrolling files.
- [x] **Milestone-02 pure-code verification:** 35 timing/runner tests passed here (24 existing + 11 new); timing Clippy and formatting of the five patch files passed.
- [x] **Milestone-02 patch-application check:** tested with CRLF files and simulated unrelated local edits; those edits and untracked beat work were preserved.
- [x] **Milestone-02 Windows application:** patch compatibility check and application both succeeded in the supplied `WINDOWS-VERIFY.log`.
- [x] **Milestone-02 Windows check 1:** formatting of all five patch files passed.
- [x] **Milestone-02 Windows check 2:** **38 passed / 0 failed**: 27 library tests (including the three local beat tests) + 4 mode regressions + 7 runner tests.
- [x] **Milestone-02 Windows check 3:** timing Clippy passed with warnings denied.
- [x] **Milestone-02 Windows check 4:** native `make_cache` compilation passed on retry.
- [x] **Milestone-02 Windows check 5:** both native transcription-boundary tests passed; 23 unrelated speech tests were filtered out, not executed.
- [x] **Milestone-02 Windows check 6:** cache-output overwrite-protection test passed.
- [x] **Milestone-02 Windows check 7:** speech/cache Clippy passed with warnings denied.
- [x] **Milestone-02 Windows check 8:** post-patch editor compilation passed.
- [x] **Milestone 02 — infrastructure implementation + Windows-native verification COMPLETE:** all eight check groups exited 0; **41 targeted tests passed, 0 failed** (38 + 2 + 1). The B overview is now checked too.
- [x] **Milestone-03 source input received:** `D:\autotrim\mentahan inheal - people pleaser.MP4`. This is the user-designated local path; actual existence/duration/content are not verified from this workspace.
- [x] **Milestone-03 pilot runner prepared:** an interactive, local-only 30-second cache + three-mode measurement batch, with explicit existing-model-folder selection and original-media preservation.
- [x] **Pilot helper checks here:** 7 standard-library CLI tests and 14 synthetic PowerShell/helper checks passed; CRLF patch compatibility passed. This is not a claim of Windows/ASR execution on the video.
- [x] **Milestone-03 native Clippy retry passed:** `RESUME-REAL-BASELINE.cmd` in folder `people-pleaser-20261019-123808-347fa0`; `cache-example-clippy` **EXIT 0** in **172.300 s** (`-j 1`, warnings still errors). The earlier 0xc0000005 crash did not recur on this retry.
- [x] **Milestone-03 source probe passed:** duration **315.84 s**; window proposed **0.000–30.000**; original MP4 not modified.
- [x] **Milestone-03 model inventory:** three multilingual models listed; default selection **large-v3-turbo** (1.51 GiB) and language Indonesian; no download.
- [ ] **Milestone 03 — in progress:** owner cancelled at `Type Y` before extraction/inference. Re-run and type **Y** to create the 30 s audio/cache and three measurements. No real-video accuracy or GUI/export acceptance yet.

**Current next action:** Clippy/native crash is cleared on this retry. Run `RESUME-REAL-BASELINE.cmd` again (or `RUN-REAL-BASELINE.cmd` if resume refuses a cancelled log). Accept defaults: model **3** / large-v3-turbo, start **0**, window 0–30 s, then type **Y** (not Enter, not N). Do not launch a second copy. Inference on large-v3-turbo can take several minutes. Send `SHAREABLE-SUMMARY.json` from the **new** output folder when it finishes. Do not treat the cancelled run as a failed native build.

The original broader milestone 01 remains partially open: global cleanup/current GUI project-open/save/other platform checks have not been claimed passed. Its build/test entry checkpoint is passed, so those maintenance tasks do not block the real-media work. **02 is complete as an infrastructure work package**; real-corpus execution is 03 and timing/recognition-quality acceptance is 04. This is not a claim that the official T1/T2/T3 real-media acceptance criteria or the whole application are complete.

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
| Current checkout revision / test provenance | Current HEAD is confirmed as `46a2f08aae3c45f866b5afeae66961ab4c97dee7`, with the local changes below. This establishes the current base revision, not a byte-for-byte snapshot of the earlier test run. |

**Important update:** the log includes these three passing beat tests:

- `beat::tests::silent_audio_has_no_beat_grid`
- `beat::tests::snaps_only_inside_the_requested_window`
- `beat::tests::finds_regular_120_bpm_pulses_within_tempo_tolerance`

Therefore, **step 16 now has evidence of a tested local beat-related foundation**. Do not label it completely unbuilt or write a second detector without inspecting your current implementation. These tests do not yet prove real-music beat quality or timeline/UI integration.

The earlier public-source review was at `46a2f08aae3c45f866b5afeae66961ab4c97dee7`, where the focused suite totaled 529. The user has now confirmed that the current Windows HEAD is the same commit, with uncommitted local changes. The supplied Windows log totals 532 and includes the three beat tests; the base commit alone does not capture those local file contents or prove they are unchanged since the test run.

**Evidence source:** `uploads/test.txt`, received in this conversation. These Windows results were read from your supplied log, not independently run on your computer by the assistant.

### New evidence — local Git status supplied by the user

The user ran `git -C C:\dev\kconcat status --short` and supplied these paths (automatic chat hyperlinks removed):

```text
 M 02-map/kconcat-roadmap-v2-execution.md
 M 04-findings/ui-1-panel-scroll-investigation.md
 D kconcat-next-steps.md
 M src/crates/concat-timing/src/lib.rs
 M src/crates/concat/src/panes/captions.rs
 M src/crates/concat/ui/workspace/keyframes-pane.slint
 M src/crates/concat/ui/workspace/meters-pane.slint
 M src/crates/concat/ui/workspace/pane.slint
?? REPLACE-README.txt
?? src/crates/concat-timing/src/beat.rs
```

**Recorded:** 7 modified tracked files, 1 deleted/missing tracked file, and 2 untracked files. This is normal evidence of local work, not automatically an error.

- `beat.rs` is present but **untracked**. Along with the modified timing `lib.rs` and the prior three passing beat tests, this supports partial local beat work; the implementation still needs review and real/UI acceptance.
- The caption pane and three scrolling-related UI files are modified. Their names identify areas to inspect, **not proof** that caption editing or panel scrolling is fixed.
- `kconcat-next-steps.md` is deleted/missing relative to the tracked state. Whether that was intentional is unknown; do not restore or remove it automatically.
- `REPLACE-README.txt` is untracked and has not been read here. Do not assume its instructions should be applied.
- The current **base commit and tracked diff summary are now recorded** (below). Actual diff contents, intended deletion, and backup verification are still pending. A commit hash identifies the base revision; the uncommitted changes must also be preserved to reproduce the actual working copy.
- A tracked-files-only diff does **not** include the contents of the two untracked files. Any backup/handoff must include them too.

### New evidence — base revision and tracked diff summary

The user supplied:

```text
HEAD: 46a2f08aae3c45f866b5afeae66961ab4c97dee7

02-map/kconcat-roadmap-v2-execution.md                 |   3 +-
04-findings/ui-1-panel-scroll-investigation.md         |  14 ++-
kconcat-next-steps.md                                 | 105 deleted lines
src/crates/concat-timing/src/lib.rs                    |   1 +
src/crates/concat/src/panes/captions.rs                 |  44 ++++++++-
src/crates/concat/ui/workspace/keyframes-pane.slint     |  14 ++-
src/crates/concat/ui/workspace/meters-pane.slint        |  21 +++--
src/crates/concat/ui/workspace/pane.slint               |  14 ++-

8 files changed, 91 insertions(+), 125 deletions(-)
```

The displayed file paths above are expanded/normalized from the pasted Git output; the line-count totals are the supplied values. This is a summary, not an inspection of the source changes. The two untracked files are absent from `git diff --stat` and still need their own backup.

**LF/CRLF warnings:** Git reported that LF line endings will be replaced by CRLF next time it writes certain working-copy files. These are line-ending conversion warnings, not compiler/test failures. Do not change global Git line-ending settings or perform mass conversion merely to silence this message.

**Pager:** `(END)` is the diff viewer waiting for input, not a failed command. If it is still displayed, press **q** to return to the command prompt.

**Backup clarification:** the user asked whether a backup is necessary. It is an optional recovery precaution, not a requirement for continuing or for read-only checks. No backup result has been reported; do not mark it completed or assume the user has made a final decision about future backups. Git commits protect the committed base, not the current uncommitted edits or untracked `beat.rs`. Recommend preservation before overwriting/replacing/deleting local work, but do not block read-only progress on it.

### Historical owner Windows attempt 1 — native dependency failure, now resolved on retry

Source: the user-pasted terminal output and `WINDOWS-VERIFY.log` in this conversation.

| Check | Supplied result |
|---|---|
| Patch compatibility and application | Both exit **0**; patch is already applied. |
| 1. Formatting, five patch files | Exit **0**. |
| 2. Timing tests, all targets | **38 passed, 0 failed**: 27 + 4 + 7. |
| 3. Timing Clippy | Exit **0**, warnings denied. |
| 4. Native `make_cache` compilation | Exit **101** in `ffmpeg-sys-the-third`'s build script. |
| 5–8. Native boundary/output tests, speech Clippy, editor check | **Not run**, because the batch stopped at check 4. |

Actionable failure (the large preceding `cargo::metadata` block is diagnostic output, not thousands of individual errors):

```text
C:\Concat-deps\ffmpeg\include\libavutil/channel_layout.h:26:10:
fatal error: 'stdlib.h' file not found
Unable to generate bindings: ClangDiagnostic(...)
```

The log shows that FFmpeg's headers/library path and Clang 21.1 were found. The failure is Clang's C-standard-header search while generating FFmpeg bindings, before the modified speech Rust code was checked. It does **not** establish a Rust-code failure, nor prove the SDK is uninstalled.

The prior successful `uploads/test.txt` explicitly initialized `D:\VStudio\VC\Auxiliary\Build\vcvars64.bat` and reported `cl.exe found, INCLUDE is set`. The repository's **existing** `open-a-real-build-terminal.bat` performs that MSVC setup and sets the native DLL paths. Its source even documents this exact `stdlib.h` failure. The older `open-a-build-terminal.bat` does not initialize MSVC and is not an equivalent launcher.

**Repair used:** the user retried in the configured build prompt. The second supplied run below passed all eight groups, with no further source patch or dependency reinstall requested. The first-attempt failure is retained as history, not an active blocker.

### Current owner Windows evidence — all eight groups passed (`uploads/test2.txt`)

The newly attached raw log establishes:

| Check | Confirmed result |
|---|---|
| 1. Formatting, five patch files | Exit **0**. |
| 2. Timing unit/integration/runner tests | **38 passed, 0 failed**: 27 + 4 + 7. |
| 3. Timing Clippy | Exit **0**, warnings denied. |
| 4. Native cache-example compilation | Exit **0**; completed in 1m 54s in the supplied run. |
| 5. Native transcription-boundary tests | **2 passed, 0 failed**; 23 other speech tests filtered out. |
| 6. Baseline-output overwrite-protection test | **1 passed, 0 failed**. |
| 7. Speech/cache Clippy | Exit **0**, warnings denied. |
| 8. Post-patch editor compilation | Exit **0**; completed in 3m 37s in the supplied run. |

**Total executed in this targeted batch: 41 passed, 0 failed. All eight exit codes are 0.** This is not a full-workspace test total and must not be added to the older 532 as if the suites were disjoint. The native speech-boundary tests finished in the supplied 4m 35s test build; model inference was not run.

**Evidence limits:** this log proves native compilation and the named tests, not real transcription, a generated real cache, a GUI opening/save cycle, export or measured timing accuracy. Those checks remain in their appropriate milestones. These results were read from the user’s Windows attachment, not executed independently by the assistant.

**Overview correction:** 02’s implemented and native-verified infrastructure is now complete. The first real cache/provenance check is explicitly tracked under 03 rather than duplicated as an extra open gate in 02. No real-media check has been marked passed.

### Current milestone-03 pilot attempt 1 — diagnostic needed

Source: user-pasted console output plus the newly uploaded `BASELINE-RUN.log` and `06-cache-example-clippy.stdout.txt`. The main log is available; the stdout file is 0 bytes. The complete matching **stderr** log has not yet been supplied.

- Run folder: `C:\dev\kconcat-corpus\people-pleaser-20261019-021016-1267c0`.
- Source/environment preflight reached the run folder, so the source path exists and the initial compiler/INCLUDE-presence checks passed. This does not prove every native header is discoverable.
- Helper patch compatibility/application, formatting, and cache-example tests completed according to the script’s control flow.
- `cache-example-clippy` exited **101**. Its last displayed lines contain `cargo::metadata`/FFmpeg feature output and the generic `build failed` message, not the actionable error.
- **Earlier status before receiving stderr:** the root error was not available from the tail alone. The decoded diagnostic below now identifies a native access violation, while the faulting module/underlying cause remains unknown.
- Model inventory/selection, source probing/extraction, real inference, and metrics have **not** run. There is no successful real cache or shareable summary from this attempt yet.

Received evidence now confirms `git apply --check` and application both exited 0, formatting exited 0, tests exited 0 (56.937 seconds), and Clippy exited 101 (70.267 seconds). The initial reverse-check exit 1 merely meant the helper patch was not already applied; it is not a failed application or test.

**Diagnostic now received:** `uploads/06-cache-example-clippy.stderr.txt` is 743,744 bytes of UTF-16 text (5,989 lines). It was decoded explicitly rather than treated as the media type guessed by the viewer. The decisive lines near the beginning are:

```text
error: failed to run custom build command for ffmpeg-sys-the-third
...build-script-build (exit code: 0xc0000005, STATUS_ACCESS_VIOLATION)
loaded clang version: clang version 21.1.0
```

This is a **native build-helper crash**, not a normal Clippy/Rust lint error. There is no faulting-module/native-stack report here, so the precise cause (e.g. which native library or resource issue) remains unknown. Do not claim libclang, concurrency or insufficient RAM is proved responsible. The earlier missing-`stdlib.h` diagnosis does not describe this log.

The first line also records waiting for a build-directory lock. That indicates a lock wait, not corruption or proof of the crash’s cause. Let any other active Cargo build finish; do not delete lock files or kill unrelated processes.

**Bounded first workaround:** match the earlier successful `--lib --example make_cache` target selection and use **`-j 1`**, preserving `--offline --locked` and `-D warnings`. The updated runner can resume at this stage only if the previous local log proves formatting/tests passed and both helper source texts match the approved hashes. It will not apply a missing source patch in resume mode.

**Logging repair:** native stdout/stderr are normalized to UTF-8 without losing Unicode; actionable error context is copied into `BASELINE-RUN.log` instead of being hidden by the last 18 metadata lines. A private whitelisted compiler/SDK/libclang-path snapshot is saved for a possible repeat failure. The full environment/credentials are not dumped.

`kconcat-step03-retry.zip` changes runner files only in this resume path; its included source patch is unchanged and used to recognize the existing helper. No global dependency reinstall/downgrade, Cargo clean, reset or lint bypass is requested. **23 synthetic/helper checks passed on PowerShell 7.4.6/Linux, including tests against the uploaded real diagnostic and main log; the actual Windows native retry remains pending.**

The runner’s last-18-lines preview omitted the cause; that is a diagnostic presentation limitation, not evidence that the user missed a step. Improve that preview after inspecting the actual error if necessary.

The checklist maintained here is separate from the deleted local `kconcat-next-steps.md`. Updating this checklist does not modify or restore that Windows file.

---

## B. Overview — the 22 milestones

An unchecked milestone can contain substantial completed code. A checked milestone means **its stated work-package gate has passed**, not that every downstream acceptance check has passed. In particular: **02 = infrastructure implemented/native-verified; 03 = real cache/baseline; 04 = measured timing/recognition quality.** The real-media boundary is explicit rather than counted twice.

### Core workflow — 01–14

- [ ] **01 — Build/check health.** BUILD/TEST ENTRY CHECKPOINT PASSED; broader cleanup/current GUI/platform checks remain. Not a blocker for step 02; see completed checkpoints above.
- [x] **02 — Timing-comparison infrastructure.** COMPLETE — implemented and Windows-native verified: all eight check groups passed; **41 targeted tests passed, 0 failed**. First real-video cache/baseline is tracked in 03; accuracy acceptance remains 04.
- [ ] **03 — Real corpus and baseline.** IN PROGRESS — native Clippy retry **EXIT 0** (172 s); source probed 315.84 s; cancelled before extraction at `Type Y`. Real cache/measurements still pending.
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
- [x] Received the user's local `git status --short`: 7 modified tracked files, 1 deleted/missing tracked file, 2 untracked files; exact paths are recorded in section A.
- [x] Received current HEAD: `46a2f08aae3c45f866b5afeae66961ab4c97dee7`, the same base commit as the earlier public review.
- [x] Received tracked diff summary: **8 files changed, 91 insertions, 125 deletions**. The two untracked files are not included in this summary.
- [ ] **Optional recovery precaution, not a milestone gate:** back up the listed tracked changes/deletion, untracked `beat.rs` and `REPLACE-README.txt`, and revision/status metadata if the owner chooses. No result reported. Strongly recommended before overwriting/replacing/deleting local work; not required for read-only checks. The small procedure in section F is not a full media/project/dependency backup.
- [ ] Review the actual relevant diffs and untracked file contents before proposing edits; a filename or line-count summary is not a code review.
- [ ] Establish whether deleting `kconcat-next-steps.md` was intentional before deciding its fate. Do not reset, clean, overwrite, blindly restore, or apply another overlay.
- [ ] Complete global formatting cleanup when relevant; it is not a prerequisite for applying the targeted milestone-02 fix. Its script checks formatting of the five patch files only, without rewriting unrelated local work.
- [ ] Complete remaining lint/maintenance checks as appropriate. Milestone-02 timing/speech Clippy is already grouped into its one Windows batch; do not request each command in a separate turn.
- [ ] Review any reported problems and agree on small fixes before applying source modifications. Do not substitute plain `cargo fmt` for the read-only check without discussing the write operation.
- [ ] Launch the current checkout with `cargo dev`; record successful opening and a basic project-open/save check.
- [ ] Recheck current CI, if applicable. Earlier Linux/macOS setup failures and the Windows workspace-test failure are historical findings, not proof of today's local failure. Diagnose each separately.
- [ ] Verify Unix script executable modes/direct invocation where those workflows are retained; track any deliberately deferred non-Windows issues explicitly.

**Complete when:** the relevant build/test/lint checks and current app launch have evidence; remaining platform issues are fixed or explicitly scoped and tracked. A focused suite is not the full workspace/native test suite.

**Evidence:** `test.txt` covers the focused suite and native compilation check. Section A records user-supplied status, base commit and tracked diff summary. Add formatting/Clippy logs, actual diff review, current launch result, and any applicable CI report. Record backup results only if a backup is performed; lack of a backup does not fail a read-only check.

### 02 — Implement and native-verify trustworthy timing-comparison infrastructure

**Why:** an improvement cannot be measured honestly if the “before” data already includes that improvement.

- [x] Inspected the matching base source and traced the three transcription backend returns applying snapping before cache creation. The owner's supplied status lists no modifications to the four existing files targeted by this fix; the application script still checks for unexpected changes.
- [x] Implemented `transcribe_for_timing_cache` and explicit timing modes, keeping the ordinary transcription API and Level 1.5 behavior. Native compilation and the two boundary tests now pass in the owner’s log.
- [x] Added four pure mode regressions, observed their pre-implementation failure, then ran them green: unsnapped identity, Level 1 energy-only, Level 1.5 comparison, unchanged production default.
- [x] Implemented cache provenance recording actual backend and requested settings/stage; native boundary and overwrite-protection tests passed on the owner’s Windows machine. Actual real-media provenance is the milestone-03 handoff, not claimed generated yet.
- [x] Implemented/tested `--snap=level1`, `--snap=level15`, the existing `--snap` alias, and strict legacy/already-snapped-cache handling. Added a Windows-CRLF SRT regression.
- [x] Ran `cargo test --locked -p concat-timing --all-targets`: **35 passed / 0 failed** here (the owner's three beat tests are not in this review copy).
- [x] Timing Clippy, formatting of the five patch files, and patch whitespace checks passed. Synthetic three-mode CLI runs passed; the known residual overlap was not hidden or claimed fixed.
- [x] Packaged an application/verification script that checks the base, refuses unexpected target-file edits, preserves unrelated local work, asks approval, and runs the Windows checks in one batch.
- [x] Owner applied the patch successfully; `git apply --check` and `git apply` both exited 0.
- [x] Owner check 1 passed: formatting of the five patch files.
- [x] Owner check 2 passed: **38 timing/runner tests**, including the preserved three beat tests.
- [x] Owner check 3 passed: timing Clippy with warnings denied.
- [x] MSVC/header-search issue cleared in the retried native build; no new source patch was needed for that error.
- [x] Owner check 4: native cache-example compilation passed.
- [x] Owner check 5: both transcription-boundary tests passed; ordinary production snapping is retained and the baseline boundary leaves word times untouched.
- [x] Owner check 6: existing baseline-output protection test passed.
- [x] Owner check 7: speech/cache Clippy passed with warnings denied.
- [x] Owner check 8: post-patch editor compilation passed.

**Status: COMPLETE — infrastructure implementation and Windows-native verification.** All eight groups passed in `uploads/test2.txt`. The same status is checked in B, not only here.

**Required handoff to 03, still open there:** generate the first real cache and verify its provenance/clock against the actual input, then save baseline/comparative runs. Recognition/timing accuracy and real-media acceptance are not inferred from these tests. This removes a duplicated handoff item, not a real test requirement.

**Deliverables:** `kconcat-step02-baseline-fix.zip` (the delivered implementation package); current verification evidence is `uploads/test2.txt`. The owner has applied its four-file modification plus one test-file addition, with no manifest/lockfile change. Local beat/lib/caption/scrolling work was not targeted; all three beat tests passed. All eight Windows check groups, including native speech and editor checks, now pass. The ZIP’s creation-time notes predate this owner verification; do not reapply it just because an older note says verification was pending.

**Completion gate for 02 (clarified to avoid overlap):** the genuine pre-onset-snap evaluation path, provenance safeguards and controlled Level-1/Level-1.5 interfaces are implemented, with their tests, native example compilation, relevant lint/format checks and editor compilation passing on the owner’s machine. **Gate met by `test2.txt`.** The first actual real-source execution/provenance check is required in 03; quality thresholds remain 04. This clarification does not change the official roadmap’s real-media acceptance requirements.

### 03 — Prepare the real corpus and baseline — T1

- [x] Received the user-designated source path: `D:\autotrim\mentahan inheal - people pleaser.MP4`.
- [x] Owner pilot runner passed its source-file existence preflight and printed the source/run folder. This does not verify media content or five-minute coverage.
- [ ] Probe the audio/duration and establish the exact source window. The first pilot attempt stopped before this stage.
- [x] Inspected model-directory resolution: the standalone cache example used AppDirs directly, while the editor applies its shared-folder preference. Prepared an example-only `--models-dir`/`--list-models` enhancement to use existing models explicitly without modifying app settings/model files.
- [x] Prepared `kconcat-step03-real-baseline.zip`: two-path example/helper patch and one interactive batch. Existing editor/timing/beat/caption/scrolling source is not targeted. Seven CLI tests and fourteen synthetic helper checks passed here; Windows-native helper verification is included in the batch, not assumed passed.
- [x] Owner ran `RUN-REAL-BASELINE.cmd` and explicitly approved/applied the small helper patch.
- [x] Owner helper formatting and cache-example-test commands completed: the script advanced past both to Clippy. Exact test counts are not established by the short pasted console output.
- [x] Received/decoded the actual stderr (UTF-16, 5,989 lines). The native FFmpeg build helper exited `0xc0000005, STATUS_ACCESS_VIOLATION`; Cargo returned 101. Loaded Clang reports 21.1.0. This identifies a native crash, **not** its faulting DLL/root cause and not a proven RAM shortage.
- [x] Prepared a runner-only diagnostic/resume repair: readable UTF-8 logs, actual-error excerpts in the main log, prior-log + source-hash guard, previous lib+example target selection, and one Cargo build job. **23 helper checks passed here**; native Windows resolution is still unverified.
- [x] Run the controlled retry once using `RESUME-REAL-BASELINE.cmd`; Clippy **EXIT 0** in 172.300 s; no reinstall/clean/lint bypass. Access-violation did not recur on this run.
- [x] Model inventory, source probe (315.84 s), and proposed 0–30 s window reached. Owner cancelled at confirmation (`STOP: Cancelled before media extraction/inference`).
- [ ] Re-run and type **Y** so extraction/inference and three measurements actually run. Cancelled attempt must not be reused as a successful cache.
- [ ] Read the resulting `SHAREABLE-SUMMARY.json` and cache/run evidence before checking successful real capture. No real media has been processed here. This default first run is a 30-second pilot, not the full corpus.
- [ ] Choose the exact 30-second reference slice and record its source start/end.
- [ ] Listen and correct its transcript; ensure reference text and hypothesis cover the **same audio range**.
- [ ] Pick three known-bad timing segments. Reserve one as a holdout before tuning; tune on the other two.
- [ ] Using the now-native-verified step-02 implementation, generate the first real pre-onset-snap cache/audio pair. Verify `onset_snapping=false`, actual backend, requested settings, source/window clock, sample rate and matching duration; the previous duplicate handoff in 02 is tracked here. No `--allow-legacy-cache` workaround for the accepted real baseline.
- [ ] Save the initial real baseline and controlled Level-1/Level-1.5 runs from that same cache/audio pair, using distinct run filenames. Keep five-minute and 30-second cache/audio pairs in separate directories and match any reference text to the exact evaluated range.
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

**New local-status evidence:** `keyframes-pane.slint`, `meters-pane.slint`, `pane.slint`, and the scrolling-investigation note are modified. Inspect those existing changes before adding another fix. Their contents and runtime scrolling result have not yet been supplied.

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

**New local-status evidence:** `src/crates/concat/src/panes/captions.rs` is modified. Review that local diff before proposing another caption-pane edit; the status listing alone does not establish what changed.

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

**Updated evidence:** your supplied Windows log includes three passing beat tests. A later Git status explicitly lists untracked `src/crates/concat-timing/src/beat.rs` and modified `src/crates/concat-timing/src/lib.rs`. Preserve and inspect that local implementation before changing it; its exact contents remain unreviewed. The base commit is now confirmed as `46a2f08aae3c45f866b5afeae66961ab4c97dee7`, but untracked/modified source is not captured by that hash.

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

### Current checkpoint — metadata checks complete

The user already supplied Git status, base HEAD and diff summary. Do not repeat those merely to recreate evidence unless the checkout changes.

If the terminal still shows `(END)`, press **q** before typing another command.

### Milestone 03 — controlled retry of the identified native crash

1. Extract `kconcat-step03-retry.zip` into the existing `C:\dev\kconcat-step03` folder. Allow replacement of runner files; no Rust source-tree overlay or model/media replacement is involved.
2. Let any other Cargo build finish. Use the configured MSVC prompt and run:

```bat
C:\dev\kconcat-step03\RESUME-REAL-BASELINE.cmd
```

3. Resume requires the previous local `BASELINE-RUN.log` and unchanged cache-helper source hashes. It skips the already-proved format/test commands, but **does not skip Clippy**. It retries Clippy once with lib+example targets and one build job, retaining warnings-as-errors.
4. If it passes, continue the model/start-time/approval prompts and the original 30-second pilot workflow below. Send the new `SHAREABLE-SUMMARY.json`.
5. If it fails, send the new `BASELINE-RUN.log` (now containing actual error context). For a repeat native access violation, also share `PRIVATE-build-environment.json` after reviewing its compiler/library paths. Do not reset/clean/reinstall or repeat unbounded retries.

**Success is not claimed in advance.** The native process-level crash is identified, but its underlying cause is not yet proven. The following original pilot workflow remains the reference after the build check succeeds.

**The milestone-02 eight-group batch has passed; do not rerun or reapply it.**

Source already supplied:

```text
D:\autotrim\mentahan inheal - people pleaser.MP4
```

1. Download `kconcat-step03-real-baseline.zip`; extract the whole ZIP into `C:\dev\kconcat-step03`.
2. In the existing configured build terminal, run:

```bat
C:\dev\kconcat-step03\RUN-REAL-BASELINE.cmd
```

3. The runner checks the source/environment and asks before a small cache-EXAMPLE-only patch. It tests/lints just that example change, not the entire passed milestone-02 batch.
4. Choose a displayed installed multilingual model. The runner reads model-related preferences and inventories known local folders; `.en`, empty and missing weights are not offered. It does not download a model, copy gigabytes into AppData, or change the editor’s preferences. If no supported model is found it stops and reports what is missing.
5. Enter the start of a clear-speech section in numeric seconds (Enter = 0), then approve the stated window. The default is **30 seconds**, not the whole MP4. Model loading/inference can take time; do not launch a duplicate process.
6. Send **`SHAREABLE-SUMMARY.json`** from the printed fresh output folder under `C:\dev\kconcat-corpus`. The summary omits transcript words, media and full private paths. If a step fails, share the main `BASELINE-RUN.log`/error; review individual native logs before sharing because they may contain transcript content.

The runner validates pre-onset-snap provenance and audio sample-count/duration, records private source-window/code/hash metadata, and saves unsnapped/Level-1/Level-1.5 measurements. It deliberately does **not** compute WER against its own uncorrected draft. Next the owner must listen/correct the same-window reference, cover the full five-minute corpus, and select the bad-segment/holdout set before tuning. A successful pilot does not automatically close all of 03 or any quality gate in 04.

Continue using `open-a-real-build-terminal.bat` for native commands if the configured prompt was closed. Cargo commands in the new runner use `--offline --locked`; no dependency/model download is silently initiated. The CMD wrapper’s PowerShell policy override is child-process-only, not a permanent policy change.

### Optional — a small backup of the listed changes (Windows Command Prompt)

The following procedure is available if the owner chooses a backup now or later. It is **not** the required next step. No backup success has been reported.

This writes only into a new backup folder outside the source checkout. It captures the tracked changes (including the deletion), the two listed untracked files, and revision/status information. It does **not** back up videos, ignored files, dependencies, build output, or the entire repository/history.

Run one command at a time. **Stop if any command fails.** If the backup folder already exists, stop and ask for a different name; do not overwrite an earlier backup.

1. Create a fresh backup folder:

```bat
mkdir "C:\dev\kconcat-change-backup-01"
```

2. Export all tracked changes relative to HEAD into a patch. `--output` writes the backup file; it does not apply changes:

```bat
git -C C:\dev\kconcat diff --binary --output="C:\dev\kconcat-change-backup-01\tracked-changes.patch" HEAD
```

The LF/CRLF warning can reappear here; it is not by itself a failure. If Git reports an actual error, stop.

3. Copy the two untracked files; a Git diff does not include them:

```bat
copy /b "C:\dev\kconcat\src\crates\concat-timing\src\beat.rs" "C:\dev\kconcat-change-backup-01\beat.rs"
copy /b "C:\dev\kconcat\REPLACE-README.txt" "C:\dev\kconcat-change-backup-01\REPLACE-README.txt"
```

Each successful copy normally reports `1 file(s) copied.` If a source is missing or an overwrite prompt appears, stop and share the message.

4. Record the base revision and file-status list:

```bat
git -C C:\dev\kconcat rev-parse HEAD > "C:\dev\kconcat-change-backup-01\base-commit.txt"
git -C C:\dev\kconcat status --short > "C:\dev\kconcat-change-backup-01\git-status.txt"
```

5. List the backup files and paste the result into the conversation:

```bat
dir "C:\dev\kconcat-change-backup-01"
```

Expected: `tracked-changes.patch`, `beat.rs`, `REPLACE-README.txt`, `base-commit.txt`, and `git-status.txt`. The patch and the two copied source/text files should not be empty. Share any error as well. This evidence will let the assistant update the backup check; it has not been marked complete in advance.

Do not apply the patch, restore the deleted file, run `git reset`/`git clean`, pull updates, or format source as part of this optional backup. Whether or not the owner chooses it, inspect existing changes and explain write/destructive operations before proposing the next code edit. Read-only checks can proceed without a backup.

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
- Additional evidence: user-pasted Windows Git status, confirmed base HEAD, and tracked diff summary, recorded in section A. Full local diffs/untracked file contents and backup results have not yet been supplied.

| Date | Revision | Change |
|---|---|---|
| 2026-10-19 | v0.1 | Created 22-step working checklist; credited supplied 532-test Windows run and app compile check; noted partial beat foundation; parked optional extras and post-22 AI-editing discussion. |
| 2026-10-19 | v0.2 | Assistant takes responsibility for maintaining the workspace checklist. Recorded supplied local Git status (7 modified, 1 deleted/missing, 2 untracked), credited the status check, and noted existing beat/caption/scrolling changes without claiming completion. Commit, backup, and diff review remained pending; no source files changed. |
| 2026-10-19 | v0.3 | Recorded confirmed HEAD `46a2f08...` and tracked diff summary (8 files, +91/−125). Explained LF/CRLF warnings and pager exit. Marked metadata checks passed; added a small change-backup procedure covering the two untracked files and tracked deletion. Backup/detailed review remained pending; no source files changed. |
| 2026-10-19 | v0.4 | Clarified that backup is optional, not a blocker for read-only checks; no backup completion or final refusal inferred. Set the then-next task to the read-only format check; retained backup instructions as optional. No source files changed in that revision. |
| 2026-10-19 | v0.5 | Switched to focused feature batches following the request for faster progress. Added a visible completed-checkpoint dashboard. Prepared the actual five-file milestone-02 patch, 11 new pure tests (35 total green here), strict cache provenance, comparative modes, CRLF reference handling, and a single Windows application/verification batch. Broader milestone-01 cleanup and native/real-media acceptance remained open. |
| 2026-10-19 | v0.6 | Recorded the first owner attempt: patch applied, Windows checks 1–3 passed (38 tests), native check 4 failed on missing `stdlib.h`, and 5–8 did not run. Recommended the existing MSVC launcher; no new source patch/reinstall. This first-attempt blocker is historical after v0.7. |
| 2026-10-19 | v0.7 | Verified `uploads/test2.txt`: all eight exits 0, 41 targeted tests passed; checked 02 in B/C as infrastructure implemented/native-verified. Kept first real cache/provenance under 03 and real quality under 04. Broader 01 checks remain open. |
| 2026-10-19 | v0.8 | Recorded the source path and prepared the 30-second pilot runner/model-folder helper. Seven CLI and fourteen synthetic helper checks passed here; no real-media acceptance was claimed. |
| 2026-10-19 | v0.9 | Recorded first pilot attempt: helper applied, formatting/tests completed, Clippy exited 101. Requested the actual diagnostic rather than speculating. No real-media/model run yet. |
| 2026-10-19 | v0.10 | Read main log and empty stdout; requested the still-missing stderr only. |
| 2026-10-19 | v0.11 | Decoded the supplied stderr: FFmpeg native build-helper access violation (`0xc0000005`), not a lint or the previous missing-header error. Underlying faulting module/cause remains unproved. Prepared a guarded runner-only resume with lib+example targets, one build job, UTF-8/error-context logging and no source edits/reinstall/clean/lint bypass. 23 helper checks passed here; owner retry and actual real-media results remain pending. |

**Immediate next action:** extract `kconcat-step03-retry.zip` into the existing `C:\dev\kconcat-step03` and run `RESUME-REAL-BASELINE.cmd` once in the configured prompt. The updated runner checks previous evidence/unchanged source, then retries the native build with lib+example targets and one job. Send the new shareable summary if successful or main diagnostic log if it stops. **02 remains complete; 03’s real-data/quality acceptance is not claimed until results arrive.**
