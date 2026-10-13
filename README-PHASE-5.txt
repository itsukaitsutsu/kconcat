PHASE 5 - AUTOTRIM  (for itsukaitsutsu/kconcat)
================================================

WHAT THIS IS
------------
The Sections panel gets its Apply button, and behind it a new,
properly tested command:

  Command::AutotrimClip { clip_id, kept: [(start, end), ...] }

`kept` is exactly what the panel's ticks produce
(sectionlist::kept_ranges). The command rebuilds the clip from those
ranges in ONE undo step:

  * the clip becomes its kept pieces, packed back to back from where
    it started; each piece's in-point advances, so the surviving
    picture is frame-for-frame what it was;
  * its linked captions (Phase 4a's links) are rebuilt through the
    same ranges - a caption over surviving picture rides it to its
    new place, one spanning a cut becomes two pieces;
  * the clip's detached sound, if any, is rebuilt the same way, so
    it stays in sync with the picture;
  * everything that started after the clip shifts left by what was
    removed - the timeline closes up;
  * the first piece of the clip (and of each caption) keeps the
    original id, so links and selection still find it.

Keeping everything, keeping nothing, or ranges outside the clip are
no-ops, not errors - deleting a whole clip is RemoveClips' job.

Known limit, said out loud: a caption cut in two keeps its full words
on both pieces - the document's text clips hold one string, and
splitting words at the cut is Phase 6 (word-level captions), which
already has captionmap::remap waiting for it.

FILES TO REPLACE (8 code files)
-------------------------------
  src/crates/concat-project/src/commands/mod.rs    (the command)
  src/crates/concat-project/src/commands/clips.rs  (its implementation;
      also contains Phases 2 + 4a, so this zip is safe on its own)
  src/crates/concat-project/src/lib.rs             (tests; also newest)
  src/crates/concat/src/panes/sections.rs          (Apply + can-apply)
  src/crates/concat/ui/dialogs/sections.slint      (the Apply button)
  src/crates/concat/ui/app.slint                   (apply callback)
  src/crates/concat/src/lib.rs                     (apply wiring)
  src/crates/concat/locales/en.json                (two new keys)
Plus docs: README-PHASE-5.txt, CHECKPOINT.md.

Apply the phases in order: 2, 4a, 4b-1, 4b-2, then this.

HOW TO VERIFY
-------------
Step 1 (cheap):
  cd C:\dev\Concat\src
  cargo test -p concat-project
Expected: test result: ok. 225 passed; 0 failed
(223 + the 2 autotrim tests: an_autotrim_rebuilds_the_clip_and_its_
 captions_from_the_kept_ranges, an_autotrim_splits_a_caption_that_
 spans_a_cut - they check the pieces, the captions, the closing
 timeline, the no-ops, and that ONE undo restores all of it.)

Step 2 (the expensive one - only if you haven't already):
  cargo check -p concat
Still not compilable in the sandbox; you are the compiler. If it
errors, paste the first `error` block verbatim into the next session
with CHECKPOINT.md. The new risk here over 4b-2 is small: one button,
one callback, one pane message.

Then, in the app: caption a clip, open Sections, leave the ticks as
they are, Apply trim. The silences go, the captions stay glued to the
words, Ctrl+Z brings everything back in one step.

WHAT IS NEXT
------------
Phase 6 - word-level captions / karaoke: whisper token timestamps
into caption clips, split a cut caption's WORDS (not just its span)
using captionmap, and highlight the spoken word. See CHECKPOINT.md.
