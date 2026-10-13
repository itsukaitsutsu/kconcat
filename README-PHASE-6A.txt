PHASE 6a - WORD TIMINGS LIVE ON CAPTIONS  (for itsukaitsutsu/kconcat)
=====================================================================

WHAT THIS IS
------------
Phase 6 (word-level captions / karaoke) in three parts - this is 6a,
the data layer, in the cheap crate as always:

  * `Clip.words`: a caption clip can now carry each word's timing
    (the captionmap `Word` type, `{"w","s","e"}` in the document).
    Timeline clock, like captionmap's cues. Absent from old documents
    and from titles; `DOCUMENT_VERSION` does not move.
  * `captionmap::estimate_words(text, start, end)`: timings spread
    over a phrase by word length ("aa" gets half of "bbbb"'s time).
    Because whisper today returns phrases, not words - 6b replaces
    these with heard timings; captionmap already calls this kind
    `Estimated`, and these are honestly that.
  * Edits keep the words true: a move shifts them, a trim clips them
    at the edge, and an autotrim SPLITS them - a caption cut at a
    silence now shows only the words that survived on each piece,
    which removes the Phase 5 limit where words exist.
  * The app: a transcribed caption is created with estimated words
    (AddTextClip gained `words`, set from captionmap at creation).
    A script's titles get none - nothing to highlight in a title.

WHAT REMAINS, AND WHEN THIS PROJECT FINISHES  <- your question
--------------------------------------------------------------
Done: Phases 1, 2, 3, 4a, 4b-1, 4b-2, 5, 6a  (8 deliveries)
Left:
  6b  Heard word timings - whisper's token timestamps turned into
      Segment.words, so captions carry real per-word times instead
      of estimates. One zip; touches concat-speech (whisper native
      code), so allow a longer check build on your machine.
  6c  Karaoke highlight - the stage paints the spoken word in the
      caption's accent as it is heard. One zip; app crate + text
      painting, the fiddliest UI phase.
  7   Finish line - full app build + smoke test with a real video,
      README/ARCHITECTURE notes, and (only if you want) tidying the
      work into commits for upstream. One zip, mostly docs.
So: 3 phases left after this one. At one phase per session, and
30-90 minutes of your machine compiling for each app-touching one,
that is realistically 2-4 days of part-time evenings - the code side
is three more sessions like this one. After Phase 7 the VTS feature
list you started with is complete.

FILES TO REPLACE (11 code files)
--------------------------------
  src/crates/concat-project/src/captionmap.rs      (estimate_words)
  src/crates/concat-project/src/model.rs           (Clip.words)
  src/crates/concat-project/src/commands/mod.rs    (AddTextClip words)
  src/crates/concat-project/src/commands/clips.rs  (words follow edits;
      also contains Phases 2 + 4a + 5, so safe on its own)
  src/crates/concat-project/src/lib.rs             (tests; also newest)
  src/crates/concat/src/panes/captions.rs          (estimated words)
  src/crates/concat/src/studio.rs                  (words: None)
  src/crates/concat-api/src/lib.rs                 (words: None)
  src/crates/concat-export/src/flatten.rs          (words: None)
  src/crates/concat-host/src/titles.rs             (words: None)
  src/crates/concat-host/tests/export.rs           (words: None)
Plus docs: README-PHASE-6A.txt, CHECKPOINT.md.

In order: 2, 4a, 4b-1, 4b-2, 5, then this.

HOW TO VERIFY
-------------
  cd C:\dev\Concat\src   (use your CURRENT folder - see the note in
  the last guide about the 221-vs-225 folder mix-up; test where your
  newest code lives, ideally the fresh clone)
  cargo test -p concat-project
Expected: test result: ok. 227 passed; 0 failed
(225 + word_timings_are_estimated_by_word_length +
 words_follow_a_move_and_split_at_an_autotrim_cut.)

`cargo check -p concat` as before if you haven't: first `error`
block verbatim, please, and do not edit around it.

You won't SEE anything new in the app yet from 6a - the words ride
along silently until 6c paints them. That is normal; this phase is
plumbing, and its tests are the point.
