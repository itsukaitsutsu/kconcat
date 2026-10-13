# Phase 8 — Sections, shared Whisper models, and caption import

This is a **source overlay** for the `kconcat` repository. It is not a compiled installer. It contains the Phase 8 code changes, this README, `CHECKPOINT-PHASE-8.md`, and a small overlay manifest.

## Included

- Replaces the Sections dialog's plain rows with a scrollable card list and a clip-wide timeline strip. The strip and cards use the same section boundaries; keep switches continue to drive the existing trim behavior.
- Shows millisecond precision for fractional section boundaries, so short valid sections no longer appear as `0:00 – 0:00`.
- Adds a configurable Whisper model folder under **Settings → Transcriber**. On Windows, the initial default is the VTS folder the user specified:

  `D:\autotrim\workspace-01a11267-3c03-70c7-ba2b-746368a13969\video-trim-studio\work\whisper-models`

  Use **Browse** or type a path and press **Apply** to change it. On an explicit change, Concat validates/creates the chosen folder on a worker and saves the choice in Settings. On non-Windows platforms the existing app-data model directory remains the default. Portable mode defaults to `portable/whisper-models`; an explicitly saved Settings path is still honored.
- Makes the Whisper model cache include both model ID and file path, so a folder change cannot silently keep using a model loaded from the previous directory.
- Recognizes the existing VTS files `ggml-large-v3-turbo.bin` and `ggml-large-v3.bin` as selectable, **local-only** models. Concat will not download or remove these externally managed files. Large models need substantially more memory; `large-v3` can be especially slow on CPU.
- Adds **File → Import Captions** for `.srt` and `.vtt` files. The import sheet lets you use project timecodes (00:00 at timeline start) or align source-media timecodes to a selected constant-speed video clip. Clip alignment clips cues to the selected clip's source range and links them to that clip. Imported captions are editable text clips and one undo step.
- Adds tests for shared-folder model detection, local-only model protection, SRT/WebVTT parsing, clip-time mapping, and precise section-time formatting.

### Existing models

Concat's old app-data models are **not copied automatically** into the shared folder. Models already present in the VTS folder will be used there. If a model exists only in Concat's old location (`%LOCALAPPDATA%\app.concat.editor\whisper-models`), either choose that folder in Settings or download that model into the shared folder again. Concat creates the selected folder when it first checks model status; this setting does not transfer old models into it. The repository's pre-existing `%APPDATA%` → `%LOCALAPPDATA%` migration remains unchanged.

## Scope boundaries

The card/timeline work is a presentation change: it derives strip positions from the existing section boundaries and leaves the keep/apply trim behavior intact. The millisecond change affects displayed times only; it does not recalculate the section boundaries. Whisper-folder and local-model support changes model discovery/storage and makes existing VTS model files selectable; it does not change transcription, segmentation, or timestamp inference. Caption import is a separate feature that creates editable clips from external SRT/VTT files. Any underlying transcription/timing bug fix should be tracked and verified separately from the Sections redesign.

## Apply the ZIP

1. Back up your checkout (or make a Git commit/stash first).
2. Extract the ZIP **into the repository root**—for example `C:\dev\kconcat`, the folder containing `src\`—and allow it to overwrite the included files. The archive is an overlay, not a full copy of the repository.
3. Open `open-a-real-build-terminal.bat`, then run the checks below from `C:\dev\kconcat\src`.

## Verify on Windows

First format the changed Rust files, then verify formatting and run the checks:

```bat
cargo fmt --all
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

For the app smoke test, the `quick` profile is separate from the default `dev` profile. If `cargo dev` shows stale generated Slint fields or callbacks, clean that profile before launching:

```bat
cargo clean -p concat --profile quick
cargo dev
```

Before this overlay was made, the user reported these passing baselines:

- `concat-project`: 237 passed, 0 failed
- `concat-text`: 30 passed, 0 failed

Those are **pre-change baselines**, not test results for this ZIP. This environment has no Rust toolchain, so the modified code has not been compiled or run here. The two baseline crates should retain those counts; the new tests are in `concat-speech` and `concat`.

## UI smoke checks

1. Open a project and open **Sections** on a clip with several captions and silences. Confirm the top strip spans the clip and its colored blocks align with the cards below.
2. Toggle a card's keep switch. Confirm the strip changes with it, the “Keeping” total updates, and Apply still performs the existing trim operation.
3. Check a short section boundary: fractional times should include milliseconds instead of both reading `0:00`.
4. Open **Settings → Transcriber**. Confirm the displayed folder is the VTS path above. Browse to a temporary folder and Apply; confirm it is created and remains selected after restarting Concat.
5. Confirm `large-v3-turbo` and `large-v3` appear installed from the VTS folder and can be selected. They have no Download or Remove action because they are shared files. If switching to the old `%LOCALAPPDATA%` folder, confirm `tiny` appears installed there.
6. Choose **File → Import Captions**, select an SRT or VTT sample, then test both project-time and selected-clip alignment. Confirm captions appear as editable text clips and undo removes the import as one action.
7. If desired, keep the VTS large models selected and transcribe a short clip. Large-model inference can be memory-intensive and slow on CPU.

The VTS screenshot/reference view was not attached when this overlay was prepared, so the cards/timeline are an implementation of the described layout, not a pixel-perfect recreation. Send the reference screenshot for a follow-up visual adjustment.
