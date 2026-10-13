# Phase 8 + VTS Integration — Sections, Auto-Caption Editor, Shared Whisper Models, and Caption Import

This is a **source overlay** for the `kconcat` repository (`https://github.com/itsukaitsutsu/kconcat`). It is not a compiled installer. It contains all Phase 8 and VTS Sections/Captions integration code changes since commit `85ee8bf`, along with `CHECKPOINT-VTS-INTEGRATION.md`, `CHECKPOINT-PHASE-8.md`, and `PHASE-8-MANIFEST.txt`.

## Included

- **Detect Sections controls (`detect-section.txt`)**:
  - **Silence** detection toggle (`detect_silence`, default `true`), configurable **Threshold** in dB (default `-45`), **Min silence** in ms (default `800`, step `50`), and **Min section** in ms (default `120`, step `20`).
  - **Captions (.srt / .vtt)** toggle (`use_captions`, default `true`), optional `.srt`/`.vtt` path field + **Upload .srt/.vtt** file picker (`sub_info` cue count), and explicit **Detect sections** button.
  - Backed by `clip_sections_with_cues(timeline, clip_id, silences, pad, use_captions, min_section_ms, external_cues)` in `concat-project`.
- **Searchable/filterable Sections list & 7-column table (`section.txt`)**:
  - Clip-wide normalized timeline strip and 7-column table (`[checkbox, track, start, end, dur, kind, text]`).
  - Toggleable **Caption**, **Silence**, and **Other** filter chips, **min section** filter in seconds (`s`, step `0.1`), and **Search caption text…** field.
  - Bulk selection actions (**Select visible**, **Clear**, **Remove selected**, **Invert**, **All silence**, **All captions**) + selection hint banner.
  - Active playhead row highlight (`at_playhead`), click-to-seek on non-text cells (`track`, `start`, `end`, `dur`, `kind`), and inline caption text editing that updates both the section row and any linked timeline caption clip.
- **Auto-caption Whisper options & interactive Caption Editor (`autocaption.txt`)**:
  - Whisper model selector + inline **Get model** downloader with progress bar and model folder status.
  - Whisper **Language** selector with all 99 languages formatted as `"code — Name"` (`"auto — Auto-detect"`, `"en — English"`, `"id — Indonesian"`, etc.).
  - **Translate speech to English** toggle with dynamic `translateHint` and automatic fallback away from transcription-only `large-v3-turbo`.
  - Collapsible **Advanced options**: **Beam size** (`1..10`), **Hint / prompt**, **Output folder** + **Browse**, and checkboxes for **word timestamps (JSON)**, **temperature fallback**, **normalize audio**, and **keep 16 kHz wav**.
  - Manual start buttons: **New blank captions** and **Open .srt / .vtt…**.
  - Interactive **Caption editor**:
    - Header bar with `(N cues)` counter, **+ Add cue**, **Undo changes**, and **Save captions** (exports both `.srt` and `.vtt`).
    - 4-mode **Layout** re-splitter (`Engine segments (as transcribed)`, `Readable lines (subtitles)`, `Burst - 3 words (vertical video)`, `One word at a time`) + **Re-split** button backed by `concat_project::captionmap::resplit_cues`.
    - Cue table (`#`, `Start (s)`, `End (s)`, `Text`, `×`).
    - Footer actions: **Discard**, **Use these captions**, and **Use these captions → Detect sections** (inserts captions onto the timeline linked to the clip and immediately opens the Sections dialog).
- **Shared Whisper model folder & local-only VTS models**:
  - Configurable Whisper model folder under **Settings → Transcriber**, defaulting on non-portable Windows to:
    `D:\autotrim\workspace-01a11267-3c03-70c7-ba2b-746368a13969\video-trim-studio\work\whisper-models`
  - Detects and allows selecting existing VTS `ggml-large-v3-turbo.bin` and `ggml-large-v3.bin` files as **local-only** models (protected from Concat download/removal).
- **Subtitle import (`File → Import Captions`)**:
  - Imports `.srt` and `.vtt` files into editable timeline caption clips with a choice between project timecodes (`00:00` at timeline start) and alignment to a selected constant-speed video clip.

## Apply the ZIP

1. Back up your checkout (or make a Git commit/stash first).
2. Extract the overlay ZIP **into the repository root** — for example `C:\dev\kconcat`, the folder containing `src\` — and allow it to overwrite the included files.
3. Open `open-a-real-build-terminal.bat`, then run the checks below from `C:\dev\kconcat\src`.

## Verify on Windows

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

Before launching `cargo dev`, clean its separate `quick` profile so Slint regenerates the updated dialog bindings:

```bat
cargo clean -p concat --profile quick
cargo dev
```

## In-session verification summary

Verified inside the Linux sandbox (Rust 1.93.1 + `slint-build 1.18.1`):

- Slint UI compilation of `src/crates/concat/ui/app.slint`: **passed** (0 errors, 0 warnings)
- `python3 scripts/locales.py --check`: **exits 0** (`1115 strings in en.json`, 0 missing English keys, 0 stale keys)
- `python3 scripts/models.py --check`: **exits 0** (`23 models, 3.37 GB, mirrored on models-v1`)
- `cargo fmt --all -- --check`: **exits 0**
- `cargo clippy -p concat-project -p concat-render --all-targets -- -D warnings`: **exits 0**
- `cargo test -p concat-project`: **240 passed**, 0 failed
- `cargo test -p concat-render -- --skip gpu::tests`: **45 passed**, 0 failed
- `pane_logic` harness (`rustc --test` against `libconcat_project.rlib`): **2 passed**, 0 failed

Full native linking of `concat-media`, `concat-speech`, and `concat` requires your Windows FFmpeg / ONNX Runtime / sherpa-onnx development environment (`cargo check -p concat`).
