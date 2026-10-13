# Checkpoint — Concat Phase 8

Use this file if work is interrupted or continued in a fresh assistant turn. It records the exact scope, source base, user decisions, work done, and remaining checks.

## Repo and base

- Repository: `https://github.com/itsukaitsutsu/kconcat`
- Local workspace: `/home/user/kconcat`
- Base commit: `85ee8bf0ca2b036d9bc96b81d125921a66be7b87` (`7check`)
- The user ran tests on the pre-change Windows checkout and reported:
  - `cargo test -p concat-project`: 237 passed, 0 failed
  - `cargo test -p concat-text`: 30 passed, 0 failed
- This sandbox has no `rustc`, `cargo`, or `rustfmt`. Do not claim the Phase 8 patch compiled or that post-change tests passed.

## Confirmed user requirements

- Sections should be changed from plain rows to cards plus a timeline strip, like the VTS project.
- Default shared Whisper model folder should be exactly:
  `D:\autotrim\workspace-01a11267-3c03-70c7-ba2b-746368a13969\video-trim-studio\work\whisper-models`
- User explicitly confirmed the path should be changeable in Settings.
- User asked for a ZIP and a checkpoint so they can resume if the conversation hits a limit.
- No VTS UI screenshot has been attached yet; current implementation is a first-pass interpretation, not a visual match to an unseen screenshot.

## Implemented in the checkout

- Sections dialog converted to a scrollable card list plus normalized clip timeline strip (`src/crates/concat/ui/dialogs/sections.slint`).
- Section model/data now carries normalized strip positions and a kind code (`src/crates/concat/src/panes/sections.rs`).
- Fractional card boundaries now display milliseconds, fixing short valid sections that formerly looked like `0:00 – 0:00`.
- Settings now has editable and browseable Whisper storage under **Settings → Transcriber**. Explicit path changes are validated/created on a worker; a path change is blocked while a Whisper transcription/download is active.
- The selected path is stored in `settings.json` as `whisperModelsDir`; non-portable Windows defaults to the user's VTS path, other platforms keep the app-data default, and portable mode defaults to `portable/whisper-models`. A previously saved explicit Settings path remains honored.
- `AppDirs` carries an optional Whisper folder override; Whisper status/download/delete/transcribe use it. The loaded-model cache now keys by both model ID and file path.
- Tests added for selected model folder resolution and millisecond section times.
- `common.close` was added to the English inventory because the existing Sections dialog referenced a missing key.
- AppDirs literals in API/server/host tests were updated for the new optional field.

## Important scope/behavior notes

- The VTS screenshot was not supplied. If the user sends it, compare the current card/timeline styling and refine it.
- Existing Concat models under `%LOCALAPPDATA%\app.concat.editor\whisper-models` are not automatically copied into the selected VTS folder. The repository's pre-existing `%APPDATA%` → `%LOCALAPPDATA%` migration is unchanged. Models already present in the selected folder are detected. Explain this to the user and let them choose the old folder or re-download a missing model.
- This phase changes the Sections presentation and Whisper storage only. It does **not** change transcription, segmentation, timestamp inference, or repeated caption-generation behavior. The millisecond display is formatting only; any underlying transcription/timing bug fix should be implemented and verified separately. Duplicate caption prevention would be a separate behavior fix.
- User-provided test counts above are pre-change baselines only.

## Modified source files

- `src/crates/concat-api/src/lib.rs`
- `src/crates/concat-host/src/dirs.rs`
- `src/crates/concat-host/src/titles.rs`
- `src/crates/concat-server/src/grpc.rs`
- `src/crates/concat-server/src/lib.rs`
- `src/crates/concat-speech/src/transcribe.rs`
- `src/crates/concat/locales/en.json`
- `src/crates/concat/src/lib.rs`
- `src/crates/concat/src/panes/sections.rs`
- `src/crates/concat/src/panes/settings.rs`
- `src/crates/concat/src/prefs.rs`
- `src/crates/concat/src/studio.rs`
- `src/crates/concat/ui/app.slint`
- `src/crates/concat/ui/dialogs/sections.slint`
- `src/crates/concat/ui/dialogs/settings.slint`
- `src/crates/concat/ui/primitives/modal.slint`

## Validation and deliverable status

- Overlay ZIP: `/home/user/kconcat-phase-8.zip`. It contains the 16 modified `src/` files, this checkpoint, `README-PHASE-8.md`, and a root manifest. ZIP integrity was verified.
- `git diff --check` is clean; `src/crates/concat/locales/en.json` parses as JSON.
- `python3 scripts/locales.py --check` exits 0 but reports the repository's missing/untranslated locale backlog (including the new English-only keys).
- No Rust toolchain is installed here; do not claim the Phase 8 patch compiled or that post-change tests passed.

## Next steps

1. Extract the overlay per `README-PHASE-8.md`, then run on the user's Windows build terminal from `C:\dev\kconcat\src`:
   ```bat
   cargo fmt --all -- --check
   cargo test -p concat-project
   cargo test -p concat-text
   cargo test -p concat-host
   cargo test -p concat-speech
   cargo test -p concat-api
   cargo test -p concat-server
   cargo test -p concat
   cargo check -p concat
   ```
3. Fix the first compile/Slint error before making further design changes; then rerun the checks.
4. Do the UI smoke steps in `README-PHASE-8.md`. If the user attaches a VTS screenshot, adjust the card/timeline styling to match it.
5. If interrupted, start by reading this checkpoint and `README-PHASE-8.md`, then inspect `git status` and the first failing command rather than redoing the work.
