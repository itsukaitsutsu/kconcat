# Checkpoint — VTS UI/feature integration into kconcat

- **Created:** 2026-10-13 (Asia/Bangkok)
- **Updated:** 2026-10-14 (Asia/Bangkok) — Full VTS `detect-section.txt`, `section.txt`, and `autocaption.txt` parity completed & verified (Stages A–D)
- **Repository:** https://github.com/itsukaitsutsu/kconcat
- **Baseline commit:** `bc636a6` (`UIrecheck`, on top of Phase 8 base `85ee8bf`)

## User request

Re-check and complete the VTS-derived Sections and Captions functionality in `kconcat`, following the three supplied UI reference files carefully:

- `detect-section.txt` — silence/caption section detection controls (`Silence` toggle, `Threshold (dB)` default `-45`, `Min silence (ms)` default `800` step `50`, `Captions (.srt / .vtt)` toggle, optional `.srt`/`.vtt` path field + `Upload .srt/.vtt` file picker + cue count status, `Min section (ms)` default `120`, and `Detect sections`)
- `section.txt` — searchable/filterable timeline section list (`caption`/`silence`/`other` filter chips, `min section` in seconds `s` step `0.1`, `search caption text…`, `select visible`, `clear`, `remove selected`, `invert`, `all silence`, `all captions`, selection hint banner, and 7-column section table `[checkbox, track, start, end, dur, kind, text]` with active playhead row highlight, click-to-seek on non-text cells, and inline caption text editing)
- `autocaption.txt` — full VTS Auto-caption setup and interactive Caption Editor (Whisper model selector + inline `Get model` downloader with progress bar & folder status, 99 Whisper languages formatted as `"code — Name"`, `translate speech to English` with dynamic `translateHint` and automatic fallback away from transcription-only `large-v3-turbo`, collapsible `Advanced options` [`Beam size`, `Hint / prompt`, `Output folder`, `word timestamps (JSON)`, `temperature fallback`, `normalize audio`, `keep 16 kHz wav`], manual start buttons [`New blank captions`, `Open .srt / .vtt…`], and interactive `Caption editor` with `+ Add cue`, `Undo changes`, `Save captions` to `.srt`/`.vtt`, 4-mode layout preset selector [`Engine segments (as transcribed)`, `Readable lines (subtitles)`, `Burst - 3 words (vertical video)`, `One word at a time`] + `Re-split`, `# / Start (s) / End (s) / Text / ×` table, `Discard`, `Use these captions`, and `Use these captions → Detect sections`)

Reference implementation: https://github.com/itsukaitsutsu/video-trim-studio

## Safety / resume instructions

To resume or apply on Windows:

1. Extract the source overlay ZIP (`kconcat-vts-integration-overlay.zip` or `kconcat-phase-8.zip`) into the repository root (`C:\dev\kconcat`, the folder containing `src\`), or extract the full recovery ZIP (`kconcat-vts-stage-6-complete.zip`) into a fresh checkout.
2. Read this file, `CHECKPOINT-PHASE-8.md`, and `README-PHASE-8.md`.
3. Run the Windows verification commands listed at the bottom of this file.

## Stage-by-stage implementation summary

### Stage A — Backend & Segmentation Engine (`concat-project` & `concat-speech`)

1. **VTS 4-Mode Caption Layout Re-segmentation (`src/crates/concat-project/src/captionmap.rs`)**:
   - Added `CaptionLayoutPreset` (`Engine`, `Line`, `Burst`, `Word`) and `resplit_cues(cues: &[Cue], preset: CaptionLayoutPreset) -> Vec<Cue>`, porting VTS `segmenter.py`'s `regroup_cues` / `_split_cue_words` / `_expand_cues_cjk_chars` rules:
     - `Engine`: preserves original cues and cleans word timings.
     - `Line` (`Readable lines (subtitles)`): max `10` words, max `42` chars, max `4.5` s duration, `0.6` s pause gap, breaking on sentence-ending (`.!?。！？…`) and clause (`,;:—–，；：、`) punctuation, with CJK per-character expansion (`max_chars: 18`, `max_words: 18`).
     - `Burst` (`Burst - 3 words (vertical video)`): max `3` words, max `22` chars, max `1.8` s duration, `0.35` s pause gap, breaking on sentence and clause punctuation (CJK `max_chars: 6`).
     - `Word` (`One word at a time`): `1` word per cue (`max_words: 1`, CJK `1` character per cue).
   - Added unit test `resplit_cues_supports_engine_line_burst_and_word_layouts`.
2. **External `.srt` / `.vtt` Cue Participation in Section Detection (`src/crates/concat-project/src/sectionlist.rs`)**:
   - Added `clip_sections_with_cues` accepting `external_cues: Option<&[crate::subtitle::Cue]>` on the file's clock (shifted by `-clip.source_start` into the clip's source window when provided, falling back to linked timeline captions otherwise).
   - Added unit test `clip_sections_with_cues_uses_external_srt_cues_shifted_to_source_start` in `src/crates/concat-project/src/lib.rs`.
3. **Whisper Advanced Options & Sidecar Export (`src/crates/concat-speech/src/transcribe.rs`)**:
   - Extended `TranscribeRequest` and `EffectiveWhisperOptions` with:
     - `beam_size: u32` (clamped `1..=10`; `1` uses `SamplingStrategy::Greedy { best_of: 1 }`, `2..=10` uses `SamplingStrategy::BeamSearch { beam_size, patience: -1.0 }`)
     - `word_timestamps: bool` (`FullParams::set_token_timestamps`)
     - `temperature_fallback: bool` (`FullParams::set_temperature_inc(0.2)` when enabled, `0.0` when disabled)
     - `normalize_audio: bool` (peak-normalizes decoded 16 kHz mono PCM to `0.95` before inference)
     - `keep_wav: bool` & `output_dir: Option<String>` (writes 16 kHz mono PCM `.wav` via `write_16khz_wav` and word-timestamp `.words.json` sidecar when requested)
   - Added explicit guard disabling `translate` on transcription-only `large-v3-turbo` in `effective_whisper_options`.

### Stage B — `detect-section.txt` & `section.txt` (`sections.slint` & `sections.rs`)

1. **Detect Sections Header (`detect-section.txt`)**:
   - Added `detect_silence: bool` (`Silence` checkbox, default `true`), `noise_db` (`Threshold`, default `-45` dB), `min_silence_ms` (`Min silence`, default `800` ms, step `50` ms), and `min_section_ms` (`Min section`, default `120` ms, step `20` ms).
   - Added `use_captions: bool` (`Captions (.srt / .vtt)` checkbox, default `true`), `subtitle_path: String` (`path to .srt / .vtt (optional)`), `Upload .srt/.vtt` file picker (`SectionsMsg::BrowseSubtitle` -> `SubtitlePicked`), and `sub_info` cue counter.
   - When `detect_silence` is unchecked, `Detect sections` builds sections immediately on the UI thread from captions alone without decoding audio.
2. **Sections Filter, Bulk-Selection Toolbar, and 7-Column Table (`section.txt`)**:
   - Added `caption`, `silence`, and `other` filter chips, `min section` filter in seconds (`s`, step `0.1`), and `Search caption text…` field.
   - Added `Select visible`, `Clear`, `Remove selected` (danger button right in the toolbar as well as the modal footer), `Invert`, `All silence`, `All captions`, and the VTS selection hint banner.
   - Upgraded the sections list into the 7-column table (`[checkbox, track, start, end, dur, kind, text]`):
     - Checkbox bound to `!row.keep` (checked = selected for removal, matching VTS `section.txt`).
     - Active playhead highlight (`row.at_playhead`, accent wash + left bar) when `studio.playhead` lies inside `[row.timeline_start, row.timeline_end)`.
     - Clicking any non-text cell (`track`, `start`, `end`, `dur`, `kind`) seeks the playhead (`SectionsMsg::RowSeek` -> `studio.seek`).
     - Inline `Field` in the `text` column for caption rows (`SectionsMsg::RowTextEdited`) that updates `row.section.text`, any loaded external cue, and any overlapping linked timeline caption clip via `Command::UpdateClip`.

### Stage C — `autocaption.txt` (`captions.slint` & `captions.rs`)

1. **Auto-caption Setup & Inline Model Downloader**:
   - Added inline `Get model` selector (`download-models`, `download-model`) and `Get model` button (`CaptionsMsg::DownloadModelClicked` -> `SettingsMsg::ModelDownload`), live download progress bar (`download-progress`), and model folder status line (`download-status`).
   - Updated language dropdown to display all 99 Whisper languages with human-readable `"code — Name"` labels (`"auto — Auto-detect"`, `"en — English"`, `"id — Indonesian"`, etc.).
   - Added dynamic `translate-hint` and automatic fallback from transcription-only `large-v3-turbo` to an installed multilingual model (`medium`, `large-v3`, `small`, `base`, `tiny`) when `Translate speech to English` is enabled.
   - Added collapsible `Advanced options` section with `Beam size` (`1..10`), `Hint / prompt`, `Output folder` + `Browse`, and checkboxes for `word timestamps (JSON)`, `temperature fallback`, `normalize audio`, and `keep 16 kHz wav`.
   - Added manual entry buttons `New blank captions` (`CaptionsMsg::NewBlankCaptions`) and `Open .srt / .vtt…` (`CaptionsMsg::OpenSubtitleFile`).
2. **Interactive Caption Editor (`autocaption.txt`)**:
   - Upgraded header bar with `(N cues)` counter, `+ Add cue`, `Undo changes` (`undo_stack`), and `Save captions` (writing both `.srt` and `.vtt` via `concat_project::subtitle::{to_srt, to_vtt}`).
   - Added layout preset bar (`Layout` dropdown with `Engine segments (as transcribed)`, `Readable lines (subtitles)`, `Burst - 3 words (vertical video)`, `One word at a time` + `Re-split` button calling `concat_project::captionmap::resplit_cues`).
   - Added `# / Start (s) / End (s) / Text / ×` table header and rows.
   - Added `Use these captions → Detect sections` primary workflow button (`CaptionsMsg::DraftInsertAndDetect`) that inserts the reviewed captions onto the timeline linked to the clip and immediately opens the Sections dialog to detect sections from them.

## Stage D — In-session verification results

- **Slint UI compiler (`slint-build = "=1.18.1"` against `src/crates/concat/ui/app.slint`)**: `Finished dev profile` (0 errors, 0 warnings).
- **`python3 scripts/locales.py --check`**: exits `0` (`1115 strings in en.json`, 0 missing English keys, 0 stale keys).
- **`python3 scripts/models.py --check`**: exits `0` (`23 models, 3.37 GB, mirrored on models-v1`).
- **`cargo fmt --all -- --check`**: exits `0`.
- **`cargo clippy -p concat-project -p concat-render --all-targets -- -D warnings`**: exits `0` (0 warnings).
- **Unit test suites executed in-session**:
  - `cargo test -p concat-project`: **240 passed**, 0 failed (includes `resplit_cues_supports_engine_line_burst_and_word_layouts`, `clip_sections_with_cues_uses_external_srt_cues_shifted_to_source_start`, and `clip_sections_with_options_respects_use_captions_and_min_section_ms`)
  - `cargo test -p concat-render -- --skip gpu::tests`: **45 passed**, 0 failed
  - `pane_logic` harness (`rustc --test` against `libconcat_project.rlib`): **2 passed**, 0 failed

## Windows verification commands

Because this Linux container does not have the Windows FFmpeg 7 / ONNX Runtime / sherpa-onnx native libraries installed, run the following on your Windows machine after extracting the overlay ZIP:

```bat
cd C:\dev\kconcat\src
cargo fmt --all -- --check
cargo test -p concat-project
cargo test -p concat-speech
cargo test -p concat
cargo check -p concat
```
