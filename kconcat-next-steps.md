# kconcat — working progress checklist

This file tracks what has been completed in this conversation and what remains. Status is based on the user's reported local results plus the repository roadmap/checkpoint. For local Windows build/test results, the user's checkout is the source of truth.

## 1. Verify the local repo and app build — IN PROGRESS / BUILD SUCCESS

- [x] Ran the local development command from `C:\dev\kconcat\src`: `cargo dev`.
- [x] App build and launch succeeded: `Finished 'quick' profile ...` and Cargo ran `target\quick\concat.exe`.
- [x] Resolved missing `motion` initializer in `crates\concat-host\src\cards.rs`.
- [x] Resolved next missing `motion` initializer in `crates\concat\src\presets.rs`. A replacement ZIP containing both files was provided; user applied the files and the subsequent `cargo dev` succeeded.
- [ ] Run `cargo test -p concat-project` from `C:\dev\kconcat\src` and report the actual pass/fail count. The checkpoint reports 529 tests, but this has not yet been confirmed in the user's local checkout.
- [ ] If tests fail, save the first relevant error block and command; reconcile before making further project changes.

## 2. T1 — make and save the timing baseline — NEXT / NOT STARTED LOCALLY

The repository roadmap says the runner code and synthetic fixture are implemented. The real-media cache and baseline still need to be generated on the user's Windows machine with its FFmpeg/Whisper setup.

### Inputs and repo checks

- [ ] Identify the original five-minute source video. The source media path is not yet known in this conversation.
- [ ] Confirm the corpus directory exists: `C:\dev\kconcat\06-corpus`.
- [x] Repository includes the Whisper cache example: `src/crates/concat-speech/examples/make_cache.rs`.
- [x] Repository includes the metric runner: `src/crates/concat-timing/examples/metric_runner.rs`.
- [x] Example usage verified from source: cache generator takes `<media-file> <whisper-cache.json> <model-id> [language]`; runner takes `<cache> <audio> [--ref <file.srt>] [--snap] [--save <out.json>]`.

### Generate the real-media cache (run from `C:\dev\kconcat\src`)

Replace the video path below with the actual five-minute source file path. The command writes the Whisper cache and decoded 16 kHz mono float audio under `06-corpus`:

```powershell
cargo run -p concat-speech --example make_cache -- "C:\path\to\your-five-minute-video.mp4" "..\06-corpus\whisper-cache.json" large-v3-turbo id
```

- [ ] Run the command with the real source video.
- [ ] Confirm it produced `C:\dev\kconcat\06-corpus\whisper-cache.json` and `C:\dev\kconcat\06-corpus\audio-16k.f32`.
- [ ] If it fails due to missing model/native dependencies or another error, stop and capture the first error block; do not guess at fixes.

### Run and save the baseline

Run from `C:\dev\kconcat\src` after cache generation:

```powershell
cargo run -p concat-timing --example metric_runner -- "..\06-corpus\whisper-cache.json" "..\06-corpus\audio-16k.f32" --ref "..\06-corpus\user-video.srt" --save "..\06-corpus\runs\baseline.json"
```

- [ ] Save the full terminal output, including the Tier 1 metrics.
- [ ] Confirm `C:\dev\kconcat\06-corpus\runs\baseline.json` was written.
- [ ] Treat WER against `user-video.srt` as provisional: the roadmap says it is generated, not yet verified ground truth. Correct/confirm the reference transcript before using WER as an evaluation result.
- [ ] Do not start further timing changes before the real baseline is saved and the result is recorded.

## 3. T8 transcript input — NOT STARTED

- [ ] Correct or confirm the transcript for the 30-second reference clip.
- [ ] Keep it aligned with the exact audio slice used for evaluation; identify the 2–3 known-bad segments and reserve one as the holdout as the roadmap directs.
- [ ] Use the corrected text for the planned Indonesian ASR model comparison after T1 is available.
- [ ] Do not fine-tune a model unless the approved bake-off shows the best available model still fails.

## 4. Track 1 word timing — ROADMAP REPORTS SANDBOX WORK DONE; REAL-CLIP ACCEPTANCE PENDING

The roadmap reports sandbox-provable implementation/tests for these tasks. This does not mean the user's real-audio acceptance checks or Windows native integration have been verified. Follow the roadmap's dependency order and record actual local results.

- [ ] T1 real-media baseline (required first; see Point 2).
- [ ] T2 onset snapping — sandbox portion reported done; real 30-second reference-clip acceptance pending.
- [ ] T3 ZCR / spectral-flux refinement — sandbox portion reported done; real-clip comparison/no-gain decision pending.
- [x] T4 script-aware tokenization — sandbox-provable portion reported done in roadmap; native `concat-speech` verification remains local.
- [x] T5 caption identity — sandbox-provable portion reported done in roadmap; app integration verification remains local.
- [x] T6 per-track nudge mechanism — `concat-project` portion reported done; UI affordance/local app verification remains.
- [x] T7 one writer for text and words — roadmap/checkpoint reports done in sandbox-provable work; verify current checkout and local integration before treating complete.
- [ ] For any new changes: read the task's named manual, write/run the failing test first, keep edits within the task's listed files, then run specified tests and clippy.
- [ ] Do not advance beyond a failed dependency; update this checklist and the roadmap/checkpoint with actual results.

## 5. Verify Windows app/UI and native integration — PARTLY DONE

- [x] `cargo dev` successfully built and launched the Windows app after the two `motion` fixes.
- [ ] Run `cargo test -p concat-project` and record the result (Point 1).
- [ ] When a UI change is made, run `cargo check -p concat` from `src/` on Windows.
- [ ] Check crates that link FFmpeg/Whisper using the local native dependencies; sandbox project tests do not verify those integrations.
- [ ] Complete owner-machine checks called out in the roadmap for UI, audio devices, recording, rendering, and other native integrations.
- [ ] When requesting debugging help, include the exact command and first compiler error block.

## 6. Later roadmap work — NOT STARTED / FOLLOW ROADMAP ORDER

After Track 1 prerequisites and acceptance, follow roadmap v2 rather than choosing features ad hoc. Items listed here are future work unless separately verified in the repo/checkpoint:

- [ ] Validate caption chunking.
- [ ] E4 — Enter-to-split; then E1 — transcript editing.
- [ ] V2 — loudness/platform export work (loudness first; roadmap reports some engine-side work done, owner-machine verification remains).
- [ ] Caption presets and karaoke; then C4/C5 caption look and motion tasks as dependencies allow. Verify current code before treating any as missing or complete.
- [ ] V3 ducking; V4 batch export (roadmap reports mechanism/queue portions done, app integration remains).
- [ ] P3 speaker labels; V1 auto-reframe (roadmap reports portions done; owner-machine verification remains).
- [ ] E3 waveform/word-edge editing; P4 recording; remaining P7/P6 verification.
- [ ] K1–K5 (beat sync, flicker, tracking, AI colour, AI text remover) are listed as not built and later priority.

## Immediate sequence from the current point

1. [x] App build/launch via `cargo dev` — succeeded locally.
2. [ ] Run `cargo test -p concat-project` and report the result.
3. [ ] Locate the five-minute source video and run the T1 `make_cache` command.
4. [ ] Run the T1 metric runner and save `06-corpus/runs/baseline.json` plus terminal output.
5. [ ] Correct/confirm the 30-second transcript for T8 and real-clip evaluation.
6. [ ] Continue real-clip acceptance and any remaining implementation only in roadmap dependency order.

---

**Important distinction:** `cargo dev` verified the app builds and launches. It did not run the project test suite or generate the T1 real-media transcription/timing baseline; those remain separate tasks.
