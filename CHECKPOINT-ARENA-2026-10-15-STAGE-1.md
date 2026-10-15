# Checkpoint — Stage 1: Sections / caption synchronization engine

**Created:** 2026-10-15 (Asia/Bangkok)  
**Repository:** `https://github.com/itsukaitsutsu/kconcat`  
**Base commit:** `fe6a66b5d8f15a04a160a661d2c4dec7fd405908` (`more bug more fug`)  
**Work branch:** `arena/fix-sync-export-checkpoints`

## Purpose of this checkpoint

This is the first self-contained overlay checkpoint for the reported Sections sync problem. It can be extracted over a fresh clone at the base commit above. It deliberately contains only this stage's files, so later checkpoints will be cumulative and supersede it.

## Fixed in this stage

1. **Caption pieces now follow the correct video piece after Sections → Remove selected.**
   - `AutotrimClip` used to split the video and captions visually but left every tail caption linked to the original/head video clip.
   - A later drag, trim, or split of the tail video therefore left its caption behind. This is the central “looks correct until the next edit / final result differs” synchronization bug.
   - The trim now records each original interval → rebuilt piece id, then rewrites `caption_of` and `detached_from` on rebuilt pieces to their matching source piece.

2. **Each Caption row identifies its exact timeline caption clip.**
   - `sectionlist::Row` now carries `caption_id` when the row came from a timeline caption.
   - Sections selection, caption-row seeking, inline caption editing, and `Delete selected captions` use that identity instead of deleting/editing every text clip in the same time range.
   - This protects a translation track or another caption track that overlaps the selected caption.

3. **Hand-made title text no longer becomes a removable “Caption” section.**
   - A caption is recognized as a text clip with a source link (`caption_of`) or word timing (`words`).
   - A manual title has neither, so Sections cannot delete or edit it accidentally as a subtitle.

## Files in this overlay

- `src/crates/concat-project/src/commands/clips.rs`
- `src/crates/concat-project/src/commands/mod.rs`
- `src/crates/concat-project/src/lib.rs`
- `src/crates/concat-project/src/sectionlist.rs`
- `src/crates/concat/src/panes/sections.rs`
- `CHECKPOINT-ARENA-2026-10-15-STAGE-1.md`

## Validation actually run in this session

From `src/` using Rust `1.93.1`:

```text
cargo test -p concat-project -j 2
251 passed; 0 failed
```

New/updated regression coverage includes:

- split-caption tail is relinked and follows when the tail video is moved;
- Sections rows retain the exact caption clip id;
- a manual title does not produce a caption row;
- pre-existing split-caption expectations now explicitly account for head and tail links.

## Still pending after this checkpoint

- Stage 2 will make **Burn captions into the video** permanently visible in Export, clarify/handle the stream-copy limitation, and ensure the toggle excludes captions only rather than all text titles.
- Stage 2 will also correct the two Sections “add captions” actions that currently share the same behavior.
- `cargo check -p concat` has not been claimed as passed yet: this environment may need the repo's native FFmpeg/Skia dependencies. Run it after applying the latest checkpoint and retain the first compiler error if it fails.

## Applying / resuming

1. Clone the repository and check out base `fe6a66b` (or use the current repository supplied to Arena).
2. Extract the matching stage zip **over the repository root** — the folder that contains `src/`.
3. On Windows, from `C:\dev\kconcat\src`, run:

```powershell
cargo test -p concat-project
cargo check -p concat
```

4. If this session stops, attach the newest `kconcat-stage-*.zip` and this checkpoint file to the next Arena session. Ask it to read the checkpoint before changing code.
