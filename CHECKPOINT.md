# CHECKPOINT — kconcat (VTS → Concat port) — updated 2026-10-13 (Asia/Bangkok)

## How to resume in a NEW Arena session (user: read this)
1. Start a new Arena Agent session.
2. Paste this exact first message:
   "Continue the kconcat project. Read CHECKPOINT.md and the newest
   README-PHASE-*.txt in the zip I attach, verify the state, then do the
   NEXT STEP listed in the checkpoint. Give me only a .zip of files to
   replace, like before."
3. Attach: (a) this CHECKPOINT.md, (b) the newest phase zip you received,
   (c) if you already applied a zip, say "applied, tests passed/failed"
   and paste any red error text verbatim.
The assistant in the new session should clone
https://github.com/itsukaitsutsu/kconcat , apply/rebuild the state below,
and continue. User handles all git commands themselves — deliverables are
.zip files whose paths mirror the repo, extracted over the repo root.

## Repo / environment facts
- Repo: https://github.com/itsukaitsutsu/kconcat (Concat is under `src/`,
  Rust workspace, pinned toolchain Rust 1.93.1 via src/rust-toolchain.toml)
- User: newbie, Windows, VSCode; one-command-at-a-time style; stale
  terminal is the #1 past failure (close VSCode fully and reopen).
- Verify cheap first: `cd C:\dev\Concat\src` then
  `cargo test -p concat-project` (18 crates). The full app `concat` crate
  is 724 crates / 30–90 min — build it only when a phase says so.

## Phase status
- Phase 1 — link field: DONE, in user's repo (commit "update3").
  `Clip.caption_of: Option<String>` in concat-project/src/model.rs,
  `Timeline::captions_of` / `captions_with_no_clip`, serde `captionOf`,
  old documents load as unlinked. Tests passed then.
- Phase 3 — silence detection: DONE, in user's repo.
  `concat-media/src/silence.rs` (pure `spans_from_windows` + `detect`).
- Phase 2 — captions follow move/trim: DELIVERED 2026-10-13 as
  `phase-2-captions-follow.zip` (2 files to replace:
  concat-project/src/commands/clips.rs, concat-project/src/lib.rs).
  Verified in sandbox: 221 passed (was 217; +4 new tests),
  clippy -D warnings clean, fmt clean.
  Behaviour: MoveClips shifts linked captions (explicitly dragged caption
  not double-moved); TrimClip clamps/drops captions (one undo step);
  ripple trim/delete carries captions of pulled clips; SplitClips
  re-links captions at/after the cut to the tail. RemoveClips contract
  unchanged (delete does NOT rewrite captions; links dangle for caller).
  >>> USER: apply zip, run `cargo test -p concat-project`, expect
      "221 passed". Report the number back in the next session. <<<
- Phase 4a — app writes the link: DELIVERED 2026-10-13 as
  `phase-4a-caption-link-wiring.zip` (see zip's README for file list).
  Design: extend `Command::AddTextClip` with
  `caption_of: Option<String>` (#[serde(default)], so old command JSON
  still parses — same precedent as its `duration`/`offset_y` fields);
  handler in commands/clips.rs stores it on the new text clip;
  `panes/captions.rs`: `Subject` gains the source clip id, `caption_clip`
  gains the link param — voice runs pass Some(clip id), script runs pass
  None (a script belongs to the timeline, not a clip); studio.rs
  duplicate path preserves `source.caption_of`. All other AddTextClip
  constructors gain `caption_of: None`.
  VERIFY LEVEL: concat-project tests in sandbox (cheap). The `concat`
  app crate was NOT compiled in the sandbox (724 crates + FFmpeg/Skia
  native deps) — user must run `cargo check -p concat` and report red
  text verbatim if any. This is expected and planned, not a failure.

## NEXT STEP (for the next session, in order)
1. Confirm Phase 2+4a applied + tests from the user ("221 passed",
   then with 4b-1 "223 passed"). If not, fix that first — do not
   stack phases. (User said "next" on 2026-10-13 without reporting
   numbers; 4b-1 was still delivered because it only touches the
   cheap crate, but ASK for the numbers before doing 4b-2.)
2. Confirm Phase 4a `cargo check -p concat` result if the user ran it.
3. Phase 4b SPLIT IN TWO:
   - 4b-1 DONE, delivered 2026-10-13 as `phase-4b1-sectionlist.zip`:
     new concat-project/src/sectionlist.rs (`clip_sections(timeline,
     clip_id, file-clock silences, pad) -> ClipSections{rows, summary}`,
     `kept_ranges(rows, ticks)`) + lib.rs mod line/tests. Verified in
     sandbox: 223 passed (221 + 2), clippy/fmt clean. Rows carry
     section (source clock) + timeline_start/end + keep (default:
     everything except silence). >>> USER: apply, expect "223 passed". <<<
   - 4b-2 DELIVERED 2026-10-13 as `phase-4b2-sections-panel.zip`:
     new panes/sections.rs + ui/dialogs/sections.slint, tray button,
     wiring in panes/mod.rs, studio.rs, lib.rs, app/editor/seat/
     timeline-pane/tray .slint, en.json keys (sections.*, tray.sections).
     Panel: opens on selected/first sounding clip, runs
     silence::detect on worker (spawn_in_project), shows sectionlist
     rows with keep switches + "Keeping X / Y". NO Apply yet (that is
     Phase 5). NOT compiled in sandbox (app crate); Rust rustfmt-parsed;
     concat-project still 223 green. >>> USER: cargo check -p concat,
     paste first `error` block verbatim if any. <<<
   - (old scouting notes for 4b-2, kept for reference:) NEXT (app crate, NOT compilable in sandbox — user must run
     `cargo check -p concat` and paste red text verbatim):
     Slint sheet modelled on crates/concat/ui/dialogs/captions.slint
     (data structs are declared in .slint, e.g. CaptionsSheetData at
     captions.slint:27, and implemented in Rust in panes/captions.rs);
     tray entry next to Captions (ui/timeline/tray.slint, callback
     `captions()` ~line 204); new panes/sections.rs modelled on
     panes/captions.rs (register in panes/mod.rs Msg + Studio): open on
     a selected clip with sound, run concat_media::silence::detect on a
     worker via spawn_in_project/on_ui_in_project, build rows with
     sectionlist::clip_sections, Apply (Phase 5) uses kept_ranges.
     i18n strings go wherever captions.* keys live (check i18n.rs).
4. Phase 5 — Autotrim: DELIVERED 2026-10-13 as `phase-5-autotrim.zip`.
   Design chosen (better than the split+delete Batch the old note
   imagined): a NEW command `Command::AutotrimClip { clip_id, kept }`
   in concat-project (mod.rs + clips.rs), implemented as a rebuild —
   clip/captions/detached-sound become their kept pieces packed
   together, first piece keeps the id, later clips shift left, one
   undo step; keep-all/keep-nothing are no-ops. Verified in sandbox:
   225 passed (223 + 2 autotrim tests), clippy/fmt clean. Panel Apply
   button wired (panes/sections.rs Apply, sections.slint, app.slint,
   lib.rs, en.json). App side NOT compiled in sandbox as usual.
   >>> USER: expect "225 passed", then cargo check -p concat. <<<
5. NEXT Phase 6 — word-level captions / karaoke: whisper token
   timestamps (concat-speech transcribe Segment - check what timing it
   carries) into caption data, split a cut caption's WORDS with
   captionmap::remap/clean_words (not just its span - the known limit
   Phase 5 states), then karaoke highlight in the stage/text rendering.
   Scout first: concat-speech Segment fields, model::TextStyle and
   concat-text title painting for per-word highlight feasibility.

## Rules established with the user
- Deliverable = one .zip per phase, paths mirroring repo root, plus a
  README-PHASE-*.txt inside the zip (what/why/files/verify/expected).
- Every claim of "tests pass" must be from an actual run in-session.
- Anything touching the `concat` app crate must be labelled
  "not compiled here — run cargo check -p concat" honestly.
