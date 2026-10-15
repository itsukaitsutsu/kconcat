# Checkpoint — Stage 2: reliable caption burn/export UI

**Created:** 2026-10-15 (Asia/Bangkok)  
**Repository:** `https://github.com/itsukaitsutsu/kconcat`  
**Base commit:** `fe6a66b5d8f15a04a160a661d2c4dec7fd405908` (`more bug more fug`)  
**Branch:** `arena/fix-sync-export-checkpoints`  
**Use this checkpoint:** it is cumulative; it includes Stage 1 as well as Stage 2.

## What this stage fixes

### 1. The Export checkbox is now always visible

The **Burn captions into the video** checkbox is always shown under Export → Output. The prior code hid it when **Same as copy** was selected, which made the option appear missing.

- With H.264 / HEVC / AV1, the switch is active and defaults to **on**.
- With **Same as copy**, the switch remains visible but disabled, shows **Choose an encoder to burn**, and is forced off. Stream copy cannot burn captions because it copies encoded streams and never creates video frames.
- A programmatic request to enable caption burning while stream copy is selected switches back to H.264 rather than silently producing a caption-less stream-copy output.

### 2. The switch controls captions, not every text overlay

When the switch is off, export excludes identified caption clips only. Hand-made title cards, lower-thirds, and shapes still render. Before this stage, disabling “Burn captions” removed every text clip, including ordinary titles.

Caption identity matches Stage 1: a text clip with `caption_of` or word timing (`words`) is a caption. Generated and imported captions meet this condition.

### 3. Stream-copy SRT sidecars now include captions only

The stream-copy sidecar export no longer turns ordinary title cards into subtitle cues.

### 4. Locale inventory is clean

Removed the stale `tray.sections` key from built-in locale files. English now contains the two caption-burn messages; non-English catalogues correctly fall back to English until translated.

## Included work from Stage 1

This cumulative checkpoint also includes the Sections synchronization repair:

- `Remove selected` relinks split caption/audio tails to their correct rebuilt source clips.
- Caption rows carry exact source ids, so editing/deleting one language does not overwrite/delete a translation track.
- Manual titles are not treated as removable caption rows.

## Validation actually run in this session

1. `cargo test -p concat-project -j 2`

```text
251 passed; 0 failed
```

2. Standalone Slint compilation of the actual `src/crates/concat/ui/app.slint` using `slint-build = 1.18.1`:

```text
Finished dev profile …
```

3. `python3 scripts/locales.py --check`

- Exited `0` after stale keys were cleaned.
- It reports expected non-fatal missing translations for the two English caption-burn strings (the runtime falls back to English).

4. Attempted `cargo check -p concat -j 2`.

- It reached native dependency setup but **cannot complete in this Linux sandbox** because FFmpeg development packages are absent (`pkg-config` cannot find `libavutil`).
- This is environment-specific, not reported as a passing app build. Run it on the Windows development environment with `FFMPEG_DIR` / project dependencies configured.

## Files this cumulative zip overlays

- `CHECKPOINT-ARENA-2026-10-15-STAGE-1.md`
- `CHECKPOINT-ARENA-2026-10-15-STAGE-2.md`
- all `src/crates/concat/locales/*.json` locale inventory files
- `src/crates/concat-project/src/commands/clips.rs`
- `src/crates/concat-project/src/commands/mod.rs`
- `src/crates/concat-project/src/lib.rs`
- `src/crates/concat-project/src/sectionlist.rs`
- `src/crates/concat/src/panes/export.rs`
- `src/crates/concat/src/panes/sections.rs`
- `src/crates/concat/ui/dialogs/export.slint`

## Windows smoke test to run next

From `C:\dev\kconcat\src` after extracting the latest zip:

```powershell
cargo test -p concat-project
cargo check -p concat
cargo dev
```

Manual check:

1. Create a caption and a hand-made title on the same timeline.
2. Export with H.264 and **Burn captions into the video** on: both caption and title must be visible in the MP4.
3. Export again with it off: title remains; caption disappears from the picture.
4. Select **Same as copy**: the checkbox must still be visible but disabled with its explanation; the `.srt` sidecar must include captions, not title text.
5. In Sections, remove a silence that splits a caption/video. Drag the resulting tail video clip. Its tail caption must move with it.

## Resume instructions

Attach the newest `kconcat-stage-2-cumulative.zip` and this checkpoint in a new Arena session. Ask the next agent to read both stage checkpoints, inspect the current branch, and continue from the next unverified issue rather than reapplying old overlays.
