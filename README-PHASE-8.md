# CHECKPOINT — VTS Parity Integration (`detect-section.txt`, `section.txt`, `autocaption.txt`) + Stages 1–4 Enhancements

- **Repository**: `https://github.com/itsukaitsutsu/kconcat.git`
- **Base Commit**: `4381c05` (`Phase 7 Stage 6: Complete all remaining tasks (6a-6e)`)
- **Completed Stages**:
  - **Stages A–E** (`b58089f` → `1fb3e08`): Full VTS parity + direct timeline caption insertion (verified on Windows: `95 / 95` tests passing in `concat`).
  - **Stage 1** (`3ade750`): Engine selector (`faster-whisper` vs `whisper.cpp` Vulkan GPU `whisper-cli.exe` / CPU), `Device`, `Compute type`, `VAD (skip silence)` compaction + timestamp remapping, and live Elapsed + ETA countdown during Auto-caption.
  - **Stage 2** (`edda2a4`): Multi-track bilingual/multi-language captions (`+ Add as new caption track` in both `Captions` and `Sections`, using `__top__` track resolution and non-overlapping vertical offsets) + `Apply to all captions` (`capApplyAll`, checked by default) in Concat's Text inspector and stage gestures, scoped per caption track.
  - **Stage 3** (`3dfa8f5`): Hold-and-drag checkbox painting + `Shift`+click range selection in `Sections`, live timeline selection sync when rows are checked/unchecked, and a dedicated `Delete selected captions` button alongside `Remove selected` (which trims video).
  - **Stage 4** (`HEAD`): Full 24-locale i18n synchronization (`1,124` keys, `0 missing, 0 stale`), full test suite verification (`443 / 443` sandbox tests passing), and updated deliverable `.zip` archives.

---

## 1. What Was Built in Stages 1–4

### Stage 1 — Transcription Speedups (Vulkan GPU `whisper-cli.exe` + `faster-whisper` + VAD) & Live ETA
- **Engine Selector (`captions.engine`)**:
  - Added **Engine** selector (`faster-whisper (Python)` vs `whisper.cpp (Vulkan GPU / CPU)`) in the Auto-caption sheet (`ui/dialogs/captions.slint`, `src/panes/captions.rs`), plus **Device** (`auto`, `cuda`, `cpu`), **Compute type** (`auto`, `int8`, `int8_float16`, `float16`, `float32`), and **VAD (skip silence)** (`vad-filter`, checked by default) in **Advanced options**.
  - Displays live engine status in the model status bar (e.g., whether Vulkan GPU `whisper-cli.exe` or Python `faster-whisper` was auto-detected on the machine).
- **Hardware-Accelerated Backends (`src/crates/concat-speech/src/transcribe.rs`)**:
  - **`whisper.cpp` Vulkan GPU CLI (`transcribe_via_whisper_cli`)**: Automatically discovers `whisper-cli.exe` / `main.exe` in `CONCAT_WHISPER_CLI`, `C:\dev\video-trimmer-subtitle\bin\whisper\`, `PATH`, or next to the executable, runs with `-ojf` (full JSON with word timestamps) and `--print-progress`, parses `progress = XX%` from stderr in real time, and falls back seamlessly to in-process `whisper-rs` if `whisper-cli` is not installed.
  - **`faster-whisper` Python Backend (`transcribe_via_faster_whisper`)**: Automatically discovers a Python interpreter with `faster_whisper` (`CONCAT_PYTHON`, `C:\dev\video-trimmer-subtitle\.venv\Scripts\python.exe`, `python`, `py`), resolves HuggingFace CTranslate2 models (`Systran/faster-whisper-*`), streams JSON progress/segments back to Concat, and falls back to `whisper.cpp` if `faster_whisper` is not installed.
  - **VAD Silence Compaction (`compact_silence_vad` & `remap_segments_from_vad`)**: Strips silent stretches (`>= 0.65 s`) from the 16 kHz mono PCM buffer before inference and remaps all segment and word timestamps back to the original clip clock afterwards.
- **Live Elapsed + ETA Readout (`eta_clock`)**:
  - Tracks `started_at: Option<Instant>` during transcription and displays `Elapsed 0:08 · ETA 0:15 (35%)` in the Auto-caption progress bar.

### Stage 2 — Multi-Track Simultaneous Captions & Per-Track `Apply to all captions`
- **Dedicated Multi-Track Caption Insertion (`__top__` track sentinel)**:
  - Updated `Command::AddTextClip` (`src/crates/concat-project/src/commands/clips.rs`) so `track_id == Some("__top__")` resolves to `timeline.tracks.last()`, allowing a single undoable `Command::Batch` starting with `Command::AddTrack { bottom: false }` to place an entire new language track onto a dedicated top track without captions dropping into silent gaps on lower caption tracks.
  - Added **`+ Add as new caption track`** to both the **Captions** review footer (`ui/dialogs/captions.slint`) and the **Sections** dialog footer (`ui/dialogs/sections.slint`), with `non_overlapping_caption_look` automatically nudging the new track's vertical `offset_y` by `130px` so multiple simultaneous subtitle tracks (e.g., English + Japanese) never overlap on the video canvas.
- **`Apply to all captions` (`capApplyAll`) in Text Inspector & Stage Gestures**:
  - Added `Apply to all captions (N)` switch (`SwitchRow`, checked by default) at the top of `TextPanel` (`ui/inspector/text-panel.slint`) whenever a selected caption clip has sibling captions on the same track (`caption_track_siblings` in `src/crates/concat/src/studio.rs`).
  - Scoped per caption track (`other.track_id == primary.track_id && other.caption_of == primary.caption_of`) so styling or moving an English caption track updates all English captions while leaving a Japanese caption track untouched, and editing the **Content** text box still edits only the individual selected caption.
  - Extended stage canvas gestures (`StageMove`, `StageScale`, `StageRotate`, `StageStretch`, `TextWidth`, `TextHeight`) to fan out across `stage_gesture_targets` when `Apply to all captions` is enabled.

### Stage 3 — Hold-and-Drag Checkboxes, Live Timeline Selection Sync, & `Delete selected captions`
- **Hold-and-Drag & `Shift`+Click Checkbox Selection (`ui/dialogs/sections.slint`, `src/panes/sections.rs`)**:
  - Pressing and dragging vertically across checkboxes in the `Sections` table continuously paints rows (`RowDragBegin` + `RowDragOver`), and `Shift`+clicking a checkbox selects/deselects the entire visible range from the last clicked row (`RowShiftSelect`).
- **Live Timeline Selection Sync (`sync_timeline_selection`)**:
  - Checking or unchecking rows in `Sections` (via click, `Shift`+click, drag-paint, or bulk selection buttons) automatically updates `studio.selection` to highlight the corresponding timeline caption clips on the Concat timeline.
- **`Delete selected captions` vs `Remove selected`**:
  - Added a dedicated **`Delete selected captions`** button (`SectionsMsg::DeleteSelectedCaptions`) in `Sections` that removes only the timeline caption clips overlapping the checked rows (`Command::RemoveClips { ripple: false }`) while keeping the video clip intact, whereas **`Remove selected`** (`Apply`) cuts the checked sections out of the video clip (`Command::AutotrimClip`).

---

## 2. Verification Performed

### Verified in Linux Sandbox (`rustc 1.99.0`)
1. `python3 scripts/locales.py --check` -> **0 missing, 0 stale** across all 24 locale files (`1,124` keys each).
2. `cargo test -p concat-core` -> **53 unit + 2 doc tests passed (0 failed)**.
3. `cargo test -p concat-project` -> **241 unit tests passed (0 failed)** (including `add_text_clip_with_top_track_sentinel_places_multi_track_captions_on_dedicated_top_track`, `resplit_cues_supports_engine_line_burst_and_word_layouts`, `clip_sections_with_options_respects_use_captions_and_min_section_ms`, and `clip_sections_with_cues_uses_external_srt_cues_shifted_to_source_start`).
4. `cargo test -p concat-effects` -> **72 unit tests passed (0 failed)**.
5. `cargo test -p concat-text` -> **30 unit tests passed (0 failed)**.
6. `cargo test -p concat-render -- --skip gpu::tests` -> **45 CPU unit tests passed (0 failed)**.
7. `slint-build::compile("src/crates/concat/ui/app.slint")` (`slint-check`) -> **compiled cleanly (exit code 0)** across all `.slint` components (`app.slint`, `captions.slint`, `sections.slint`, `text-panel.slint`, `model.slint`).

### To Verify on Windows (`C:\dev\kconcat`)
After extracting `kconcat-vts-integration-overlay.zip` over `C:\dev\kconcat`:
```powershell
cd C:\dev\kconcat\src
cargo test -p concat-speech -p concat-project -p concat
cargo run -p concat
```

---

## 3. Deliverable Archives

- `/home/user/kconcat-vts-integration-overlay.zip`: Overlay archive containing all modified/added files from `4381c05..HEAD` with paths relative to the repo root (`src/...`). Extract directly over `C:\dev\kconcat`.
- `/home/user/kconcat-vts-stage-6-complete.zip`: Identical copy of the overlay archive for convenience.
- `/home/user/kconcat-phase-8.zip`: Full repository snapshot at `HEAD` (`git archive --format=zip`).
