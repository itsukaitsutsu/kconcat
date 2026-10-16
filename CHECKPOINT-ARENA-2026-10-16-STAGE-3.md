# Checkpoint — Stage 3: overlapping subtitle cues on one caption track

**Created:** 2026-10-16 (Asia/Bangkok)  
**Repository:** `https://github.com/itsukaitsutsu/kconcat`  
**Base commit:** `fe6a66b5d8f15a04a160a661d2c4dec7fd405908` (`more bug more fug`)  
**Branch:** `arena/fix-sync-export-checkpoints`  
**Use this checkpoint:** it is cumulative and contains Stages 1–3.

## User-reported issue addressed

A caption could not be dragged over another caption on the same timeline track. It looked as though the caption stopped after a small amount of movement, especially for concurrent or long imported subtitle cues.

This was not a two-second rule. The generic timeline placement engine prohibited **every** same-track overlap, and import/extraction code shortened each cue to end at the next cue's start. Those two behaviours silently changed valid overlapping SRT/VTT subtitle timing and prevented a direct caption drag onto another cue.

## Changes in this stage

### Caption-only overlap rule

- Added `Clip::is_caption()` as the shared identity: a text clip with `caption_of` or word timing is a generated/imported caption.
- Two caption clips may now overlap in time on the **same track**.
- The normal no-overlap invariant remains in place for all other combinations:
  - a title, lower-third, shape, audio, video, or layer cannot be placed over another clip on the same track;
  - a caption cannot cover any of those ordinary clips;
  - only a caption/caption pair is permitted.
- Caption dragging now lands at the requested overlapping time instead of snapping to a gap.
- Auto-placement of generated/imported captions retains an overlapping cue's exact intended start and duration on its caption track.

### Preserve source subtitle timings

Removed prior `end = min(end, next_start)` truncation from all relevant paths:

- Import Captions (`.srt` / `.vtt`)
- captions generated from transcription drafts
- Sections caption rows
- Sections external subtitle cues

Concurrent source cues now remain concurrent on the timeline and in export/SRT output.

## Important visual behaviour

Same-track overlapping captions are now valid, but they are both rendered. If two simultaneous cues have the same vertical `offset_y`, their text will draw in the same area. This is intentional: users can move/style a concurrent cue to a different vertical position when they want both visible separately. The application no longer alters subtitle timestamps merely to avoid that visual overlap.

## Validation actually run in this session

1. Targeted formatting/parsing checks on all Stage-3 Rust files with Rustfmt (no workspace-wide formatter run).
2. `git diff --check` passed.
3. `cargo test -p concat-project -j 2`

```text
252 passed; 0 failed
```

The new regression test proves all of the following:

- an overlapping imported caption remains at its requested timestamp;
- two captions share a track and can overlap;
- dragging one caption onto another succeeds;
- a normal title still cannot cover captions on that lane.

`cargo check -p concat` remains unavailable in this Linux sandbox because FFmpeg development pkg-config metadata (`libavutil.pc`) is absent. The Stage-3 app-pane source was Rustfmt-parsed; run the full application check on the Windows development system.

## Files newly changed by Stage 3

- `CHECKPOINT-ARENA-2026-10-16-STAGE-3.md`
- `src/crates/concat-project/src/commands/clips.rs`
- `src/crates/concat-project/src/lib.rs`
- `src/crates/concat-project/src/model.rs`
- `src/crates/concat-project/src/placement.rs`
- `src/crates/concat/src/panes/captions.rs`
- `src/crates/concat/src/panes/import_captions.rs`
- `src/crates/concat/src/panes/sections.rs`

The cumulative ZIP additionally contains all Stage-1 and Stage-2 files and checkpoint documents.

## Windows smoke test

After extracting the newest cumulative ZIP over the repository root:

```powershell
cd C:\dev\kconcat\src
cargo test -p concat-project
cargo check -p concat
cargo dev
```

Manual regression steps:

1. Import an SRT/VTT containing two cues whose time ranges overlap.
2. Confirm the timeline holds both on one caption track at their exact source ranges.
3. Drag either caption so it overlaps the other. It must land there rather than snap away.
4. Place a normal title on that caption track. It must still avoid/choose a gap rather than cover the captions.
5. Export H.264 with **Burn captions into the video** enabled. Both concurrent captions must render (reposition one vertically first if they should be readable separately).

## Resume instructions

Attach the newest Stage-3 cumulative ZIP together with this document in a new Arena session. The next agent should begin by reading all three checkpoint files, inspecting the current branch and native Windows build result, then continue only from remaining verified issues.
