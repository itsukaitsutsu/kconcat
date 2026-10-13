PHASE 6c-3 - HIGHLIGHTS FOLLOW ANIMATED CAPTIONS  (for itsukaitsutsu/kconcat)
==========================================================================

WHAT THIS IS
------------
Phase 6c lit the spoken word in an accent, but skipped any caption with
keyframed animation (motion, a fade, a zoom). The highlight was left out
rather than drawn in the wrong place. 6c-3 removes that skip.

Each word's highlight now carries the caption's own animation over the
word's moment. So if a caption slides in, or zooms, the lit word moves
with it, and stays on top of the plain caption it belongs to.

HOW IT WORKS
------------
The export engine reads a clip's keyframes against that clip's own window
(0 at its start, 1 at its end). A word's highlight has a shorter window, so
its keys cannot be copied across as they are. Instead:

  * concat-core gains `Track::window(x0, x1, samples)`. It re-times a
    track onto a sub-window: both ends are pinned to the caption's value
    there, keys inside the window keep their own value and easing, and
    sample points between them are read from the caption's curve.
  * concat-host's titles.rs cuts the caption's keyframes for each word's
    window with that function (`window_animation`), and gives each
    overlay the result. Overlays copy the caption's other settings as
    before.

Honest limits:
  * The curve is read at 16 points across the word's window, joined by
    straight lines. Over a word (a fraction of a second) that is
    indistinguishable from the real curve. A caption with very fast,
    sharp motion inside one word could differ slightly.
  * The window is cut from the caption's keyframes as the engine reads
    them: a clip's own 0..1 is taken to be start-to-end in timeline time,
    with no speed change. Captions are made at speed 1, so this holds for
    them. If a caption is ever sped up, check its highlight by eye.
  * Fades and transitions on the caption still do not carry to the
    highlight (as in 6c).
  * Nothing about the painted pictures changes: keyframes move the picture,
    they are not baked into it, so the cache is unchanged.

FILES TO REPLACE (2 code files + 2 docs)
----------------------------------------
  src/crates/concat-core/src/animate.rs        (Track::window, 4 new tests;
      nothing else in the file changed)
  src/crates/concat-host/src/titles.rs         (window_animation; highlights()
      no longer skips animated captions; imports)
  README-PHASE-6C3.txt                         (this file - new)
  CHECKPOINT.md                                (updated)

Apply this zip ON TOP OF the cargo6 state, with 6c and 6c-2 already in.

HOW TO VERIFY
-------------
In a FRESH VSCode terminal (close VSCode fully and reopen first).

1. The curve maths (fast, runs in the sandbox too):
     cd C:\dev\Concat\src
     cargo test -p concat-core
   Expected: test result: ok. 53 passed; 0 failed
   and the 4 names containing "window" or "eased_ramp" or "keeps_its_place"
   or "plain" among them.

2. Project (fast): cargo test -p concat-project
   Expected: test result: ok. 237 passed; 0 failed

3. Host - NOT COMPILED IN THE SANDBOX (needs FFmpeg dev libs, as before):
     cargo check -p concat-host
   If it fails, paste the FIRST `error` block verbatim and do not edit
   around it.

4. By eye: in the app, make a voice caption with a highlight colour (6c),
   then give the caption an animation (a keyframed fade, slide or zoom,
   by whatever keyframing the app offers for a title). Play it. The lit word should move
   with the caption. If it lags or jumps, tell me which motion and which
   word.

EXPECTED
--------
  Steps 1 and 2: the numbers above.
  Step 3: compiles with no errors.
  Step 4: the lit word follows the caption's motion.

WHAT WAS CHECKED IN THE SANDBOX, AND WHAT WAS NOT
-------------------------------------------------
  Run, passing:  concat-core 53 tests (4 new, for Track::window), clippy
                 with -D warnings clean, rustfmt clean on animate.rs and
                 titles.rs. concat-project 237 still green.
  NOT compiled:  concat-host (titles.rs). The window function it calls is
                 the tested one; the glue (the ExportKey conversion and the
                 call in highlights()) was checked by reading only.
