PHASE 6b - HEARD WORD TIMINGS  (for itsukaitsutsu/kconcat)
=========================================================

WHAT THIS IS
------------
Phase 6 in three parts: 6a (data layer, done), 6b (this one: real word
times from whisper), 6c (karaoke highlight, next), 7 (finish line).

6a gave every transcribed caption ESTIMATED word times - the phrase
spread over its words by length. 6b replaces the estimate with what
whisper actually heard, where whisper gives it:

  * transcribe(): whisper's token timestamps are switched on. Each
    token now carries a start and end in the audio.
  * Tokens are grouped into words the way whisper spells them: a token
    with a leading space starts a new word, one without continues the
    word before ("the", " quick", " fox", "." -> "the", "quick", "fox.").
    Split characters (a UTF-8 character cut across two tokens) decode
    whole. Grouping is `words_from_tokens`, a pure function, tested.
  * concat-speech Segment gains `words: Vec<HeardWord>` on the window's
    clock, like its start and end. Special tokens (end of text, language,
    timestamp markers) are left out, the same cut the segment text makes.
  * concat-project captionmap::caption_words(): places the heard words on
    the timeline (clip start + source time over speed), clips them to
    the caption with clean_words, and falls back to estimate_words when
    none survive. So a caption is never worse off than in 6a, and a word
    can never sit outside the caption that shows it.
  * The app (captions.rs) uses it for voice runs. Script titles still get
    no word timings: they belong to no voice.

WHAT YOU WILL SEE
-----------------
Nothing new on screen yet - same as 6a. The word timings now ride along
on the caption clips with heard times, ready for 6c's highlight. The
"Phase 6b" difference shows in the data, and in the real-audio check
below, which prints each word with its heard time.

Honest limits:
  * Whisper's token times come from its own alignment, so they are an
    approximation of when each word is spoken. How close they are has NOT
    been measured on real speech yet - that is what the real-audio check
    in step 4 is for. Judge them by ear before 6c builds on them.
  * Languages written without spaces (Japanese, Chinese, Thai) have few
    or no word breaks in whisper's tokens, so a run of them may come back
    as one word. That is honest - whisper did not say where the words
    break - and a caption still shows as one piece.
  * Token timestamps add a little compute per clip. The text is the same.

WHAT REMAINS, AND WHEN THIS PROJECT FINISHES
--------------------------------------------
Done: Phases 1, 2, 3, 4a, 4b-1, 4b-2, 5, 6a, 6b  (9 deliveries)
Left:
  6c  Karaoke highlight - the stage paints the spoken word in the
      caption's accent as it is heard. App crate + text painting, the
      fiddliest UI phase.
  7   Finish line - full app build + smoke test with a real video,
      README/ARCHITECTURE notes, and (only if you want) tidying into
      commits for upstream. Mostly docs.

FILES TO REPLACE (3 code files + 2 docs)
----------------------------------------
  src/crates/concat-speech/src/transcribe.rs   (token timestamps,
      words_from_tokens, Segment.words, HeardWord, real-audio test prints
      words)
  src/crates/concat-project/src/captionmap.rs  (caption_words + 5 tests;
      estimate_words doc updated)
  src/crates/concat/src/panes/captions.rs      (uses caption_words;
      caption_clip takes the words)
  README-PHASE-6B.txt                          (this file - new)
  CHECKPOINT.md                                (updated)

HOW TO VERIFY
-------------
Do these in order, in a FRESH VSCode terminal (close VSCode fully and
reopen first - a stale terminal was the #1 past failure).

1. Cheap check (fast, runs here too):
     cd C:\dev\Concat\src
     cargo test -p concat-project
   Expected last lines:
     test result: ok. 232 passed; 0 failed; 0 ignored
   (227 from 6a + 5 new caption_words tests.)

2. The speech crate (whisper.cpp builds with cmake - slow, first time
   can be 10-20 minutes):
     cargo check -p concat-speech
   If it fails, paste the FIRST `error` block verbatim and do not edit
   around it. Not compiled in the sandbox - honest: it needs FFmpeg dev
   libs and cmake, which it did not have.

3. The app (30-90 minutes the first time):
     cargo check -p concat
   Same rule: first `error` block verbatim if any.

4. Optional, by ear - the real-audio test (needs the Tiny (English) model
   downloaded from Settings > Transcriber first, and a short speech WAV):
     PowerShell:
       $env:CONCAT_TRANSCRIBE_AUDIO="C:\path\to\speech.wav"
       cargo test -p concat-speech -- --ignored real_audio --nocapture
   Each segment prints, then each of its words with its heard times,
   e.g.   "    1.20-1.46 hello". Check a few against the audio.

WHAT WAS CHECKED IN THE SANDBOX, AND WHAT WAS NOT
-------------------------------------------------
  Run, passing:  concat-project 232 tests (227 + 5 new), clippy with
                 -D warnings clean, rustfmt clean on all three files.
  Run, passing:  words_from_tokens' 6 tests, run on an exact copy of the
                 extracted function in a scratch crate (clippy clean).
  Checked by reading the source of whisper-rs 0.16.0 / whisper-rs-sys
                 0.15.0 (the pinned versions in Cargo.lock): every
                 whisper call in the new code has its signature confirmed.
  NOT compiled:  concat-speech and the concat app. Their build needs
                 FFmpeg dev libs and cmake, not available in the sandbox.
                 The first real compile is step 2 above.
