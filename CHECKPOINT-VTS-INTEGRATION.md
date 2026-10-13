# Checkpoint — VTS UI/feature integration into kconcat

**Created:** 2026-10-13 (Asia/Bangkok)  
**Repository:** https://github.com/itsukaitsutsu/kconcat  
**Baseline commit:** `bc636a6` (`UIrecheck`)

## User request

Complete the existing VTS-derived Sections and Captions functionality in kconcat, following these supplied UI references:

- `autocaption.txt` — full VTS Auto-caption card and caption editor
- `detect-section.txt` — silence/caption section detection controls
- `section.txt` — searchable/filterable timeline section list and selection actions

Reference implementation: https://github.com/itsukaitsutsu/video-trim-studio

## Safety / resume instructions

This checkpoint was made before new integration edits. A full source snapshot ZIP named `kconcat-vts-baseline-bc636a6.zip` accompanies it. To resume after interruption:

1. Extract the ZIP into a new folder.
2. Read this file, `CHECKPOINT.md`, `CHECKPOINT-PHASE-8.md`, and the three supplied `.txt` references.
3. Continue from the next-step list below.
4. After each coherent implementation stage, update this checkpoint and create a new ZIP.

## Baseline state confirmed

- Working tree was clean at checkpoint creation.
- Existing project already contains substantial VTS ports:
  - subtitle SRT/VTT parser/writer
  - section arithmetic and section-list model
  - FFmpeg silence detection
  - autotrim command and linked-caption edits
  - whisper.cpp transcription with word timing
  - karaoke/highlight support
  - Sections dialog
  - Captions dialog
  - caption import
  - configurable shared Whisper model directory
- Existing UI is simplified and does **not** yet expose all controls/workflows shown in the supplied VTS HTML references.

## Planned integration stages

1. Map each supplied VTS control to existing Concat backend support; explicitly mark controls needing new backend work.
2. Upgrade Detect Sections UI and state: threshold, minimum silence, caption participation, minimum section, rerun and result summary.
3. Upgrade Auto-caption UI and state: model/language/translation and advanced transcription options where supported; progress/cancel/error handling; editable generated captions before timeline insertion.
4. Add/upgrade the Sections list: kind filters, minimum-duration filter, text search, select visible/clear/invert/all-silence/all-captions, and remove selected.
5. Add tests and locale keys, run formatting/static checks, and document unsupported engine-specific items clearly.
6. Produce a final source overlay ZIP and a full recovery ZIP.

## Validation baseline

At creation, no new code had been applied. Existing repository documentation reports the latest Windows `cargo check -p concat` passed before later model/import additions; those latest additions still require full Windows validation per `CHECKPOINT-PHASE-8.md`.

## Stage checkpoint: Detect Sections controls

Implemented after the baseline snapshot:

- Sections dialog now exposes VTS-style detection controls:
  - silence threshold in dB (default -45)
  - minimum silence in ms (default 500)
  - minimum section in ms (default 120)
  - use linked captions toggle
  - explicit Detect sections / re-run button
- Detection now uses the selected threshold and minimum-silence duration.
- Added `clip_sections_with_options` while preserving the old `clip_sections` API and defaults.
- Caption classification can be disabled and minimum-section folding is configurable.
- Added Slint/Rust callbacks and English locale strings.

Validation:

- `git diff --check`: clean.
- `python3 scripts/locales.py --check`: exits 0 with the repository's existing untranslated-locale warnings plus the new English-only keys.
- `cargo test -p concat-project`: **237 passed**.
- `cargo check -p concat`: began successfully and reached dependency compilation, but exceeded the 180-second sandbox wait. It must be completed on Windows; do not infer failure from the timeout.

Recovery archive: `kconcat-vts-stage-1-detect-sections.zip`.

Next stage: implement the searchable/filterable Sections list actions from `section.txt`, then checkpoint again.

## Stage checkpoint: searchable Sections list

Implemented after the Detect Sections checkpoint:

- Caption / silence / other kind filters.
- Minimum visible section duration filter.
- Caption-text search.
- Select visible, clear, invert, all silence, and all captions actions.
- Filtered rows retain their original source index, so toggling a row edits the correct section.
- Bulk selection follows the VTS removal meaning: selected rows are marked not-kept; **Apply trim** removes them and closes gaps in one undoable autotrim command.
- Enlarged the Sections dialog to fit the VTS-style list toolbar.

Validation:

- `git diff --check`: clean.
- Locale check exits 0 with expected existing untranslated-locale warnings.
- `cargo test -p concat-project`: **237 passed**.
- `cargo check -p concat` reached the expected sandbox limitation: missing system `libavutil` / FFmpeg development pkg-config files. No Rust or Slint diagnostic was reached. Complete this check on the configured Windows build environment.

Recovery archive: `kconcat-vts-stage-2-sections-list.zip`.

Next stage: Auto-caption configuration and supported Whisper controls, then checkpoint again.

## Stage checkpoint: Auto-caption configuration

Implemented after the Sections-list checkpoint:

- Added Whisper language selection with `auto` plus all 99 VTS language codes.
- Added speech-to-English translation toggle.
- Added optional decoding hint/prompt for names and jargon.
- Extended `TranscribeRequest` with backward-compatible serde defaults.
- whisper.cpp now receives the selected language, translation flag, and initial prompt.
- English-only models safely force English and ignore translation, while multilingual models use the requested settings.
- Existing model selection, model-folder sharing, progress, cancellation, word timestamps, and errors remain wired.

Validation:

- `git diff --check`: clean.
- Locale check exits 0 with expected untranslated-locale warnings.
- `cargo test -p concat-project`: **237 passed**.
- `cargo check -p concat-speech` exceeded the sandbox wait while compiling native dependencies; the environment still lacks the FFmpeg development setup required for a final native check. Complete on Windows.

Recovery archive: `kconcat-vts-stage-3-autocaption-options.zip`.

Next stage: editable transcription results/caption editor, then checkpoint again.

## Stage checkpoint: editable caption results

Implemented after Auto-caption options:

- Successful transcription now opens a review editor instead of immediately changing the timeline.
- Each generated cue exposes editable start, end, and text fields.
- Cues can be added and deleted before insertion.
- Discard returns to caption setup without touching the timeline.
- “Use these captions” inserts all reviewed cues as one undoable batch.
- Untouched cues preserve Whisper word timings; edited cues regenerate safe estimated word timings so karaoke/highlight data remains internally consistent.
- Invalid/blank cues are skipped and cue durations are constrained to remain positive.

Validation:

- `git diff --check`: clean.
- Locale check exits 0 with expected untranslated-locale warnings.
- `cargo test -p concat-project`: **237 passed**.
- Full native/Slint application validation still requires the configured Windows environment because this sandbox lacks FFmpeg development libraries.

Recovery archive: `kconcat-vts-stage-4-caption-editor.zip`.

Next stage: final integration audit, manifests/readme, final overlay ZIP, and full recovery ZIP.
