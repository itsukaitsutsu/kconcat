# CHECKPOINT — kconcat (VTS → Concat port) — updated 2026-10-13 (Asia/Bangkok), after 6c

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
- Sandbox (Arena) facts, learned 2026-10-13: rustup 1.93.1 installs in
  ~15 s (`curl https://sh.rustup.rs`, `--default-toolchain 1.93.1 -c rustfmt
  -c clippy`); `cargo test -p concat-project` ~35 s cold on 2 cores / 2 GB.
  No sudo, no FFmpeg dev libs, no cmake -> concat-media, concat-speech and
  the concat app cannot compile there. Those are always the user's `cargo check`.
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
1. Repo HEAD ("6check") already holds phases 1-6a, so those were applied.
   Ask for the numbers: `cargo test -p concat-project` should read
   "232 passed" after 6b (227 after 6a) and "237 passed" after 6c.
   If it does not, fix that first - do not stack phases.
2. Confirm Phase 4a `cargo check -p concat` result if the user ran it.
2a. Confirm Phase 6c, 6c-2 (`phase-6c2-colour-control.zip`) and 6c-3
   (`phase-6c3-animated-highlight.zip`, expects concat-core 53 passed): `cargo test -p concat-text` expects "30 passed";
   `cargo test -p concat-project` expects "237 passed"; then
   `cargo check -p concat-host` and `cargo check -p concat`. Do NOT treat
   6c as verified until the user reports these numbers.
2b. Confirm Phase 6b: `cargo check -p concat-speech` first (whisper.cpp
   native, slow), then `cargo check -p concat`. Paste the first `error`
   block verbatim if any. Optional: the ignored real-audio test,
   `CONCAT_TRANSCRIBE_AUDIO=speech.wav cargo test -p concat-speech -- --ignored real_audio`
   (needs tiny.en downloaded), which now prints each word's heard times.
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
5. Phase 6 SPLIT IN THREE:
   - 6a DELIVERED 2026-10-13 as `phase-6a-word-timings.zip`: Clip.words
     (model.rs), captionmap::estimate_words, AddTextClip.words,
     move/trim/autotrim carry+split words (autotrim rebuilds piece
     text from its words), captions.rs creates estimated words.
     Verified: 227 passed, clippy/fmt clean.
   - 6b DELIVERED 2026-10-13 as `phase-6b-heard-word-timings.zip` (see
     README-PHASE-6B.txt inside it for the file list). Design:
     `set_token_timestamps(true)` in transcribe(); each segment's tokens
     with id < token_eot (text tokens only, same cut as the segment text)
     grouped into words by leading space (`words_from_tokens`, pure, raw
     bytes so split UTF-8 decodes whole) -> `Segment.words: Vec<HeardWord>`
     (window clock). captionmap::caption_words places heard words on the
     timeline (clean_words clips them to the caption) and falls back to
     estimate_words when none survive. captions.rs uses it for voice runs;
     script titles still get None. Verified in sandbox: concat-project
     232 passed (227 + 5 caption_words tests), clippy -D warnings clean,
     fmt clean; words_from_tokens' 6 tests run green in a scratch copy of
     the extracted function (clippy clean too). NOT compiled in sandbox:
     concat-speech (needs FFmpeg dev libs via concat-media + whisper.cpp
     via cmake; sandbox has neither, 2 GB RAM, no sudo) and the concat app.
     Whisper glue calls were checked against whisper-rs 0.16.0 /
     whisper-rs-sys 0.15.0 source. Known limit: languages without spaces
     (ja/zh/th) come back as one word per run - honest, not a guess.
     >>> USER: cargo check -p concat-speech, then -p concat; paste first
         `error` verbatim. Run real_audio for a by-ear check. <<<
   - 6c DELIVERED 2026-10-13 as `phase-6c-karaoke-highlight.zip` (see
     README-PHASE-6C.txt inside it). Design: the spoken word only, in a
     per-caption accent. `TextStyle.highlight_color` (optional; empty =
     none; voice captions default to #ffd21f). concat-text: paint() takes
     an overlay colour; `word_ranges`, `render_highlight_frame`,
     `render_highlight` (PNG). Each heard word gets one overlay picture of
     the whole caption with only that word lit, on that word's window; it
     copies the caption's clip with fades, transition and reveal reset.
     concat-project captionmap: `locate_words`, `layout_words_of`.
     concat-host titles.rs: `Painting::Highlight`, disk and live paths.
     Animated captions get no highlight yet (keys are clip-window-relative).
     Verified in sandbox: concat-text 30 passed (24 + 6), concat-project
     237 passed (232 + 5), clippy -D warnings clean, rustfmt clean.
     NOT compiled: concat-host and the concat app (captions.rs, presets.rs).
     >>> USER: cargo check -p concat-host, then -p concat; paste the first
         `error` verbatim. Then eyeball a voice caption for the yellow word.
         <<<
   - 6c-2 DELIVERED 2026-10-13 as `phase-6c2-colour-control.zip` (see
     README-PHASE-6C2.txt). "Spoken word" swatch in the text inspector's
     colour section, shown only for clips with word timings. Writes
     ClipTextField::HighlightColor -> TextStyle.highlight_color. The
     default yellow is concat-project model::DEFAULT_HIGHLIGHT.
     Verified in sandbox: concat-project 237 passed, clippy clean, fmt
     clean. NOT compiled: the concat app and its .slint files.
     >>> USER: cargo check -p concat; paste the first `error` verbatim.
         Then check the swatch by eye. <<<
     Known open items: older captions (no accent set) show yellow but do
     not light until a colour is picked; live-monitor cost per word
     (measure first). (Clear-to-none: DONE in 6c-4.)
   - 6c-4 DELIVERED 2026-10-13 as `phase-6c4-clear-highlight.zip` (see
     README-PHASE-6C4.txt). "Spoken word" switch above the swatch: off
     clears the caption's highlight colour (transparent = none), on
     restores the default. Verified in sandbox: rustfmt clean on studio.rs;
     concat-project 237 (unchanged). NOT compiled: the concat app.
     >>> USER: cargo check -p concat; paste the first `error` verbatim. <<<
   - 6c-3 DELIVERED 2026-10-13 as `phase-6c3-animated-highlight.zip` (see
     README-PHASE-6C3.txt). Highlights now follow keyframed captions:
     concat-core `Track::window` re-times the caption's keys onto each
     word's window; concat-host titles.rs `window_animation` applies it to
     each overlay. The old "animated -> no highlight" skip is gone.
     Verified in sandbox: concat-core 53 passed (4 new), concat-project
     237 passed, clippy -D warnings clean, rustfmt clean. NOT compiled:
     concat-host (titles.rs).
     >>> USER: cargo test -p concat-core (53), cargo check -p concat-host;
         paste the first `error` verbatim. Then check a moving caption by
         eye. <<<
6. Phase 7 DELIVERED 2026-10-13 as `phase-7-finish-line.zip`: docs only.
   ARCHITECTURE.md section 16 (captions and karaoke, limits, smoke test) and
   one row in "Where to look". No code. The smoke test (section 16 steps 1-8)
   is the owner's, on a real video.
   >>> USER: run the smoke test; report the numbers and the first error. <<<
   Phase 7 (original plan): full build+smoke with a real video, docs
   (README/ARCHITECTURE), optional upstream-tidy commits. After Phase
   7 the original VTS feature list is complete: linked captions,
   follow edits, silence detection, sections list, autotrim,
   word-level/karaoke (+ SRT/VTT modules already in from Tranche A).
   Estimate given to user: 3 phases/sessions after 6a, 2-4 part-time
   days, dominated by user's app builds (30-90 min each).
7. Phase 8 (Stages A-E + Stages 1-4 + Multi-GPU Vulkan + Live Caption Styling Perf + Multi-Caption Track Isolation) DELIVERED 2026-10-15:
   - `kconcat-multi-caption-fix.zip` / `kconcat-vts-gpu-and-perf-fix.zip`
   - Adding new captions via Auto-caption (`Generate` -> `Add to timeline` / `+ Add as new caption track` / `Add & detect sections`), `From script`, `Import captions` (`.srt`/`.vtt`), or `Sections` now always spawns a dedicated new track at the top (`Command::AddTrack { bottom: false }` + `Some("__top__")`), preserves existing caption tracks without deleting or colliding with them, and nudges `offset_y` by `-0.12` in Concat's normalized `[-1.0, 1.0]` frame-height coordinate space.

## Rules established with the user
- Deliverable = one .zip per phase, paths mirroring repo root, plus a
  README-PHASE-*.txt inside the zip (what/why/files/verify/expected).
- Every claim of "tests pass" must be from an actual run in-session.
- Anything touching the `concat` app crate must be labelled
  "not compiled here — run cargo check -p concat" honestly.
