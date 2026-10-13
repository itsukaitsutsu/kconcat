# Checkpoint — Concat Phase 8 + VTS Sections & Captions Integration

Use this file to resume if the work is interrupted. It records the confirmed scope, current implementation, in-session test/lint results, and remaining Windows native validation.

## Repository and baseline

- Repository: `https://github.com/itsukaitsutsu/kconcat`
- Workspace: `/home/user/kconcat`
- Phase 8 base commit: `85ee8bf0ca2b036d9bc96b81d125921a66be7b87` (`7check`)
- VTS integration baseline commit: `bc636a6` (`UIrecheck`)
- Detailed VTS integration log & control matrix: see `CHECKPOINT-VTS-INTEGRATION.md`.

## Confirmed user requirements and decisions

- Sections dialog upgraded to match the supplied VTS references (`detect-section.txt` and `section.txt`):
  - Detect Sections controls: silence threshold (dB), minimum silence (ms), minimum section (ms), use linked captions toggle, and explicit re-run button.
  - Searchable/filterable Sections list: Caption / Silence / Other kind filter chips, minimum visible duration filter (ms), search box, and bulk selection buttons (`Select visible`, `Clear`, `Invert`, `All silence`, `All captions`) plus clip-wide timeline strip and section cards with millisecond boundary formatting.
- Auto-caption dialog upgraded to match the supplied VTS reference (`autocaption.txt`):
  - Whisper language selector (`auto` + 99 ISO-639-1 language codes), speech-to-English translation toggle, and initial prompt/hint input.
  - Post-transcription editable caption review editor (`Add cue`, editable start/end/text per cue, delete cue, `Discard`, and `Use these captions`) before timeline insertion.
- Shared Whisper model folder on non-portable Windows defaults to:
  `D:\autotrim\workspace-01a11267-3c03-70c7-ba2b-746368a13969\video-trim-studio\work\whisper-models`
  Editable/browseable in **Settings → Transcriber**, with local-only detection/selection of VTS `ggml-large-v3-turbo.bin` and `ggml-large-v3.bin` files (protected from Concat download/removal).
- **File → Import Captions** imports `.srt`/`.vtt` files into editable timeline caption clips with a choice between project-time `00:00` and selected-clip source alignment.

## Source files changed since `85ee8bf`

- `CHECKPOINT-PHASE-8.md`
- `CHECKPOINT-VTS-INTEGRATION.md`
- `PHASE-8-MANIFEST.txt`
- `README-PHASE-8.md`
- `scripts/models.py`
- `src/crates/concat-api/src/lib.rs`
- `src/crates/concat-host/src/dirs.rs`
- `src/crates/concat-host/src/titles.rs`
- `src/crates/concat-project/src/lib.rs`
- `src/crates/concat-project/src/sectionlist.rs`
- `src/crates/concat-server/src/grpc.rs`
- `src/crates/concat-server/src/lib.rs`
- `src/crates/concat-speech/src/transcribe.rs`
- `src/crates/concat/locales/en.json`
- `src/crates/concat/src/lib.rs`
- `src/crates/concat/src/panes/captions.rs`
- `src/crates/concat/src/panes/import_captions.rs`
- `src/crates/concat/src/panes/mod.rs`
- `src/crates/concat/src/panes/sections.rs`
- `src/crates/concat/src/panes/settings.rs`
- `src/crates/concat/src/prefs.rs`
- `src/crates/concat/src/studio.rs`
- `src/crates/concat/ui/app.slint`
- `src/crates/concat/ui/dialogs/caption-import.slint`
- `src/crates/concat/ui/dialogs/captions.slint`
- `src/crates/concat/ui/dialogs/sections.slint`
- `src/crates/concat/ui/dialogs/settings.slint`
- `src/crates/concat/ui/primitives/modal.slint`

## In-session validation completed

- Slint UI compilation (`slint-build = "=1.18.1"` on `src/crates/concat/ui/app.slint`): **passed** (`SLINT_COMPILE_OK`).
- `python3 scripts/locales.py --check`: **exits 0** (0 missing English keys, 0 stale keys).
- `python3 scripts/models.py --check`: **exits 0** (`23 models, 3.37 GB, mirrored on models-v1`).
- `cargo fmt --all -- --check`: **exits 0**.
- `cargo clippy -- -D warnings`: **exits 0** across pure workspace crates and harness-compiled `dirs.rs`, `jobs.rs`, `transcribe.rs`, `prefs.rs`, `captions.rs`, `sections.rs`, and `import_captions.rs`.
- Unit tests executed in-session:
  - `concat-core`: **53 passed** (+ 2 doc-tests)
  - `concat-project`: **238 passed**
  - `concat-text`: **30 passed**
  - `concat-effects`: **72 passed**
  - `concat-vision`: **26 passed**
  - `concat-render` (CPU suite): **45 passed**
  - `concat-speech` (`transcribe.rs` harness): **12 passed**, 1 ignored
  - `concat` / `concat-host` (`pane_logic` harness): **24 passed**

## Next steps on Windows

1. Extract the overlay ZIP into `C:\dev\kconcat` (preserving paths).
2. Run from `C:\dev\kconcat\src`:
   - `cargo fmt --all -- --check`
   - `cargo test -p concat-project`
   - `cargo test -p concat-speech`
   - `cargo test -p concat`
   - `cargo check -p concat`
   - `cargo clean -p concat --profile quick`
   - `cargo dev`
