PHASE 4b-1 - THE SECTIONS LIST'S NUMBERS  (for itsukaitsutsu/kconcat)
=====================================================================

WHAT THIS IS, AND WHY THE PANEL IS NOT IN THIS ZIP
--------------------------------------------------
The Sections panel (Phase 4b) has two halves:

  1. The numbers: for one clip, tile it into caption / silence / other
     rows, say where each row is on the timeline, and turn the ticked
     rows back into kept time ranges. Pure arithmetic.
  2. The picture: a Slint sheet that shows those rows with checkboxes.

Half 1 is this zip. It lives in `concat-project`, so it is testable in
seconds with no app build - the same discipline as every phase so far.
Half 2 (next zip) lives in the `concat` app crate, which cannot be
compiled in the sandbox (724 crates, FFmpeg/Skia native deps), so it
is being done LAST and separately, when these numbers are known-good
underneath it. Writing the picture first, blind, is how a newbie ends
up debugging a checkbox and the arithmetic behind it at the same time.

WHAT YOU GET
------------
One new file, `src/crates/concat-project/src/sectionlist.rs`:

  clip_sections(timeline, clip_id, silences, pad) -> ClipSections
      silences: (start, end) seconds on the FILE's clock, exactly as
      concat-media's silence::detect returns them.
      pad: seconds of air kept at each end of every silence (the
      silence is shrunk by it - the "Pre Head / Post Tail" idea).
      Returns rows that tile the clip exactly. Each row has:
        section         kind (caption/silence/other), source-clock
                        times, and the caption's words
        timeline_start  where the row is on the timeline
        timeline_end
        keep            ticked by default: everything EXCEPT silence,
                        i.e. "delete the silences" is the default edit
      plus a Summary (seconds and counts per kind) for the line a
      panel prints above its list. Unknown clip id -> None.

  kept_ranges(rows, ticks) -> [(start, end)]
      The ticked rows as timeline ranges, neighbours merged: the
      input an autotrim will later turn into actual cuts, and the
      thing the panel's Apply button will hand over.

The two-clocks problem is solved here, once, so the panel never does
this arithmetic itself: captions stored on the timeline are mapped
into the clip's source window (through its speed and in-point), and
file-clock silences are shifted by the clip's in-point.

FILES TO REPLACE (2)
--------------------
  src/crates/concat-project/src/sectionlist.rs   (NEW)
  src/crates/concat-project/src/lib.rs           (one `pub mod` line
      + 2 tests; also already contains Phases 2 and 4a - applying this
      zip on top of the 4a zip is the intended order)

Plus docs: README-PHASE-4B1.txt, CHECKPOINT.md (updated).

HOW TO VERIFY
-------------
  cd C:\dev\Concat\src
  cargo test -p concat-project

Expected:  test result: ok. 223 passed; 0 failed
(previous phases: 221; the 2 new tests are
 a_clips_sections_tile_it_and_tick_everything_but_silence
 sections_translate_between_file_timeline_and_speed_clocks - this one
 proves the file/timeline/speed clocks agree, including the pad.)

Then:  cargo fmt -p concat-project -- --check
       cargo clippy -p concat-project --all-targets -- -D warnings
(both silent/Finished when clean - verified that way before packaging)

WHAT IS NEXT
------------
Phase 4b-2 - the panel itself, in the app crate: a Slint sheet modelled
on dialogs/captions.slint, a tray entry next to Captions
(ui/timeline/tray.slint), a pane module that runs silence::detect on a
worker (the captions pane's spawn_in_project pattern) and shows
clip_sections' rows, whose Apply hands kept_ranges to Phase 5's
autotrim. That zip will need you to run `cargo check -p concat` and
report any red text verbatim - it cannot be pre-compiled.
Then Phase 5 autotrim, then word-level captions / karaoke.
All of it is written down in CHECKPOINT.md.
