# Checkpoint — Concat Phase 8

Use this file to resume if the work is interrupted. It records the confirmed scope, current implementation, user-provided test/build results, and remaining validation.

## Repository and baseline

- Repository: `https://github.com/itsukaitsutsu/kconcat`
- Workspace: `/home/user/kconcat`
- Base commit: `85ee8bf0ca2b036d9bc96b81d125921a66be7b87` (`7check`)
- User-reported pre-change tests: `concat-project` 237 passed; `concat-text` 30 passed.
- The assistant sandbox has no `cargo`, `rustc`, or `rustfmt`. Do not claim the current source compiled or passed tests here.

## Confirmed user requirements and decisions

- Sections should become cards plus a VTS-like timeline strip.
- On non-portable Windows, the shared Whisper default is exactly:
  `D:\autotrim\workspace-01a11267-3c03-70c7-ba2b-746368a13969\video-trim-studio\work\whisper-models`
  The path is editable/browseable in Settings. Other platforms keep their app-data default; portable mode defaults to `portable/whisper-models`, while an explicitly saved path is honored.
- User's model files: VTS folder has `ggml-large-v3-turbo.bin` and `ggml-large-v3.bin`; `%LOCALAPPDATA%\app.concat.editor\whisper-models` has `ggml-tiny.bin`.
- User chose **local-only VTS model support**: detect/use the two existing large `.bin` files, but do not offer Concat download or remove actions for them. They can select the old Concat folder to reuse Tiny.
- User requested caption import. User chose `.srt`/`.vtt` into editable timeline caption clips, with a per-import choice between project-time 00:00 and alignment to the selected clip.
- A VTS UI screenshot has not been supplied; the Sections design remains a first pass.

## Implemented in the current source tree

- Sections dialog uses a card list and normalized timeline strip; keep/apply trimming behavior is unchanged. Fractional range display now shows milliseconds.
- Settings persists `whisperModelsDir`; Whisper status, download, delete, and inference use the configured directory. Loaded-model cache matching includes model ID and file path.
- The model list detects `large-v3-turbo` and `large-v3` in the chosen folder. These are local-only and protected from Concat download/removal; VTS models can be selected. Added detection/protection tests.
- Added **File → Import Captions** for SRT and WebVTT. A modal chooses project timeline or selected-clip/source alignment. The parser strips simple markup, respects clip in-point/speed for constant-speed visual clips, clips cues to the selected source range, and creates editable caption text clips in one batch/undo. Variable-speed clips are not offered for source alignment.
- Added parser/alignment tests, English UI strings, and source overlay/checkpoint docs.
- Fixed the Slint `SectionCard` property collision by renaming its input property `row` to `section-item`.

## Scope boundaries and migration notes

- Section-card/timeline work is presentation/data only; the millisecond display does not recalculate timing. Shared-folder work changes discovery/storage, not transcription or timestamp inference. Any underlying transcription/timing bug fix remains distinct.
- Existing Concat models are not copied into the VTS directory. The repository's pre-existing `%APPDATA%` → `%LOCALAPPDATA%` migration remains unchanged. If the user selects the old app-data folder, `ggml-tiny.bin` should be detected.
- Large models require substantially more memory; `large-v3` may be slow on CPU. They are not downloadable or removable in Concat to protect the shared VTS files.

## Windows validation history

- `cargo fmt --all -- --check` first reported formatting-only diffs; user ran `cargo fmt --all`.
- After `cargo clean -p concat`, `cargo check -p concat` initially exposed the Slint `row` property collision. That is fixed in the current source.
- A subsequent `cargo check -p concat` **passed** in the default `dev` profile (2m22s) after the `section-item` fix.
- `cargo dev` then used a separate `quick` profile and hit stale generated Slint code under `target\quick`. The user was advised to clean that profile with `cargo clean -p concat --profile quick`, then rerun `cargo dev`; this has not yet been confirmed.
- The current source adds model and subtitle-import changes **after** that successful `cargo check`; these newer changes have not been compiled or tested on Windows.

## Source files changed for the deliverable

Existing Phase 8 files plus the latest model/import additions:

- `src/crates/concat-api/src/lib.rs`
- `src/crates/concat-host/src/dirs.rs`
- `src/crates/concat-host/src/titles.rs`
- `src/crates/concat-server/src/grpc.rs`
- `src/crates/concat-server/src/lib.rs`
- `src/crates/concat-speech/src/transcribe.rs`
- `src/crates/concat/locales/en.json`
- `src/crates/concat/src/lib.rs`
- `src/crates/concat/src/panes/captions.rs`
- `src/crates/concat/src/panes/import_captions.rs` (new)
- `src/crates/concat/src/panes/mod.rs`
- `src/crates/concat/src/panes/sections.rs`
- `src/crates/concat/src/panes/settings.rs`
- `src/crates/concat/src/prefs.rs`
- `src/crates/concat/src/studio.rs`
- `src/crates/concat/ui/app.slint`
- `src/crates/concat/ui/dialogs/caption-import.slint` (new)
- `src/crates/concat/ui/dialogs/sections.slint`
- `src/crates/concat/ui/dialogs/settings.slint`
- `src/crates/concat/ui/primitives/modal.slint`

## Current validation and next steps

- Current workspace checks after the local-model/caption-import additions: `git diff --check` is clean; English locale JSON parses; `scripts/locales.py --check` exits 0 while reporting the repository's usual missing/untranslated locale warnings (including new English-only keys). No Rust toolchain is installed here.
- `/home/user/kconcat-phase-8.zip` was regenerated as a 23-file source overlay (all source entries preserve `src/` paths; README, checkpoint, and manifest are at the ZIP root). `unzip -t` passed. Rebuild it after any further change to an included file.
- On Windows, apply the latest ZIP at `C:\dev\kconcat` (repo root; preserve paths). Run `cargo fmt --all`, then `cargo fmt --all -- --check`, `cargo check -p concat`, and the test commands in `README-PHASE-8.md`.
- Before `cargo dev`, clean its separate quick profile: `cargo clean -p concat --profile quick`, then `cargo dev`.
- If a command fails, fix the first error before proceeding. Then smoke-test model detection/selection, SRT/VTT import in both timebases, and the Sections view. Ask for the VTS screenshot if visual refinement is wanted.
