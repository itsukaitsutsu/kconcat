PHASE 6c - KARAOKE HIGHLIGHT  (for itsukaitsutsu/kconcat)
========================================================

WHAT THIS IS
------------
Phase 6 in four parts: 6a (estimated word times, done), 6b (heard word
times from whisper, delivered), 6c (this one: the spoken word is painted
in an accent colour as it is heard), 7 (finish line).

A word-timed caption (a voice caption from 6a/6b) now lights the word
being spoken. Each heard word gets its own picture: the whole caption in
its usual look, with only that word in the accent colour. The picture
shows for exactly that word's window on the timeline, then the next word
takes over.

The accent is a new field on the title style, `highlightColor`. Voice
captions get the default accent, #ffd21f (yellow), when they are made.
An empty value means no highlight, which is what every older document
and every hand-typed title reads as, so nothing changes for them.
Script titles still get no word times, so they still get no highlight.

Chosen design (from the checkpoint): the spoken word only, painted in a
per-caption colour. The shader route was rejected: no shipped effect
reads the reveal order, and it could not be tested without a GPU.

WHAT YOU WILL SEE
-----------------
Add captions to a clip's voice (the Captions sheet, as in 6b). The caption
text shows as before, and each word lights in yellow as the voice says it.
Gaps between words show the plain caption. Outline, shadow and background
stay on the caption only; the highlight is the words alone.

Honest limits (read these before judging the feature):
  * NO INSPECTOR CONTROL YET. The colour cannot be changed from the
    window in this phase. The field exists in the document, and it can
    be edited by hand in the project's .json, but the colour picker is
    the next step (6c-2, below). It is not in this zip.
  * Animated captions get NO highlight. A caption with keyframed motion
    or opacity is left plain, and the app writes an info line to the log.
    Making the highlight follow keyframes is a later job.
  * Fades and transitions on the caption do not apply to the highlight
    pictures. A word's highlight is a word-length moment, so a fade on
    the whole caption does not carry over to it.
  * Estimated word times (6a) also get highlights, when whisper gave no
    word times for that caption. They are less exact than heard times.
  * Languages without spaces (Japanese, Chinese, Thai) come back from
    whisper as one word per run, so the highlight lights the whole run.
  * Cost: one picture per heard word. The monitor paints them in memory
    at its own size, and they are kept in a budget of about 512 MB. A
    long caption with many words, on a busy timeline, may repaint on each
    change and feel slower. That is the first thing to measure. The disk
    cache keeps 400 pictures in all, and a long caption adds one per word,
    so old pictures may be swept sooner and painted again.

FILES TO REPLACE (6 code/test files + 2 docs)
---------------------------------------------
  src/crates/concat-text/src/lib.rs            (paint() takes an overlay
      colour; word_ranges, render_highlight_frame, render_highlight, the
      clearing of pixels outside the lit word; 6 new tests)
  src/crates/concat-project/src/captionmap.rs  (locate_words,
      layout_words_of; 4 new tests. Also carries the 6b edits, unchanged
      from the 6b zip, so extracting after 6b is safe.)
  src/crates/concat-project/src/model.rs       (TextStyle.highlight_color,
      optional, not written when empty)
  src/crates/concat-project/src/lib.rs         (1 new serde test)
  src/crates/concat-host/src/titles.rs         (Painting::Highlight; one
      overlay TitleClip per heard word on both the disk and the live
      path; 1 new test)
  src/crates/concat/src/panes/captions.rs      (voice captions get the
      default accent. Also carries the 6b edits, unchanged.)
  src/crates/concat/src/presets.rs             (one line: a preset's
      style has no highlight)
  README-PHASE-6C.txt                          (this file - new)
  CHECKPOINT.md                                (updated)

HOW TO VERIFY
-------------
Do these in order, in a FRESH VSCode terminal (close VSCode fully and
reopen first - a stale terminal was the #1 past failure).

1. Painter (fast, runs in the sandbox too):
     cd C:\dev\Concat\src
     cargo test -p concat-text
   Expected last lines:
     test result: ok. 30 passed; 0 failed; 0 ignored
   (24 from before + 6 new.)

2. Project (fast):
     cargo test -p concat-project
   Expected last lines:
     test result: ok. 237 passed; 0 failed; 0 ignored
   (232 from 6b + 5 new: four locate/layout tests in captionmap, one
   serde test for highlightColor.)

3. Host - NOT COMPILED IN THE SANDBOX. Needs FFmpeg dev libs, same as
   6b. Run:
     cargo check -p concat-host
   If it fails, paste the FIRST `error` block verbatim and do not edit
   around it. Optional, if that passes:
     cargo test -p concat-host a_word_timed_caption
   Expected: 1 passed.

4. App - NOT COMPILED IN THE SANDBOX (30-90 minutes the first time):
     cargo check -p concat
   Same rule: first `error` block verbatim if any. Changed here:
   panes/captions.rs and presets.rs.

5. By eye: start the app, make a voice caption as in 6b, and watch the
   stage while the clip plays. The spoken word should be yellow. Report
   what you see, and whether the timing lines up with the voice.

EXPECTED
--------
  Steps 1 and 2: the numbers above, every time.
  Steps 3 and 4: compile with no errors. Warnings are not expected.
  Step 5: the word lit matches the word spoken, to within the look of
  whisper's timings (6b). If it is badly out of step, say so with a
  sentence about which word, and the clip's speed if it is not 1.

WHAT WAS CHECKED IN THE SANDBOX, AND WHAT WAS NOT
-------------------------------------------------
  Run, passing:  concat-text 30 tests (24 + 6 new), clippy with -D
                 warnings clean, rustfmt clean.
                 The new word_ranges test shows the byte ranges count
                 the layout's words one for one, across Latin text,
                 hyphens, punctuation, CJK and CRLF.
                 The overlay test shows that only the lit word's
                 pixels are set, and that the block and word boxes are
                 the same as the title's own.
  Run, passing:  concat-project 237 tests (232 + 5 new), clippy with
                 -D warnings clean, rustfmt clean.
  NOT compiled:  concat-host (titles.rs) and the concat app (captions.rs,
                 presets.rs). Both need FFmpeg dev libs and cmake, which
                 the sandbox does not have. Their code was written
                 against the signatures checked in the sandbox crates,
                 and read through once, but no compiler has seen it.
                 The first real compile is steps 3 and 4 above.

NEXT (6c-2, not in this zip)
----------------------------
  The highlight colour control in the caption/text inspector (a swatch
  beside the text colour, through ClipTextField, like the stroke colour),
  plus the translations for it. Then the measured live cost, if steps 4
  and 5 show lag.
