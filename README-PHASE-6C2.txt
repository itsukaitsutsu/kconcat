PHASE 6c-2 - SPOKEN WORD COLOUR CONTROL  (for itsukaitsutsu/kconcat)
====================================================================

WHAT THIS IS
------------
Phase 6c made a voice caption light the spoken word in a yellow accent.
That accent was fixed. 6c-2 adds the control to change it: a colour
swatch, "Spoken word", in the text inspector's colour section. It shows
only on a clip that has word timings to light (a voice caption).

Changing the swatch writes the caption's own highlight colour, so each
caption can have its own accent. The default is still yellow, #ffd21f,
and it now lives in one place: concat-project's DEFAULT_HIGHLIGHT.

Nothing about the painting changes. 6c-2 only lets you choose the colour
that 6c already paints with.

WHAT YOU WILL SEE
-----------------
Select a voice caption. Open the text inspector's colour section. Below
the Color swatch, a "Spoken word" swatch shows yellow. Pick another
colour: on the stage, the spoken word lights in that colour as the voice
says it. Titles that are not captions do not show the swatch.

Honest limits:
  * No clear button. The swatch always has a colour. To go back to the
    default yellow, pick it again.
  * Older captions, made before 6c, have no accent set. Their swatch
    shows the yellow default, but they do not light a word until you
    pick a colour for them. The swatch and the lighting only agree once
    a colour has been picked.
  * Captions with keyframed animation still get no highlight (as in 6c).

FILES TO REPLACE (5 code/text files + 2 docs)
---------------------------------------------
  src/crates/concat-project/src/model.rs     (DEFAULT_HIGHLIGHT constant;
      the 6c field and everything else in the file are unchanged)
  src/crates/concat/src/panes/captions.rs    (uses DEFAULT_HIGHLIGHT)
  src/crates/concat/src/studio.rs            (ClipTextField::HighlightColor
      set; highlight colour, hex and "is this a caption" in the inspector's
      data)
  src/crates/concat/ui/timeline/model.slint  (ClipTextField.highlight-color;
      SelectedClipData.highlight, highlight-hex, highlightable)
  src/crates/concat/ui/inspector/text-panel.slint  (the Spoken word swatch)
  src/crates/concat/locales/en.json          (textPanel.highlight: "Spoken
      word". Other languages fall back to English for this key.)
  README-PHASE-6C2.txt                       (this file - new)
  CHECKPOINT.md                              (updated)

Apply this zip ON TOP OF the cargo6 state (6a, 6b and 6c already in).

HOW TO VERIFY
-------------
In a FRESH VSCode terminal (close VSCode fully and reopen first).

1. Project (fast, runs in the sandbox too):
     cd C:\dev\Concat\src
     cargo test -p concat-project
   Expected last line: test result: ok. 237 passed; 0 failed

2. App - NOT COMPILED IN THE SANDBOX (30-90 minutes the first time):
     cargo check -p concat
   If it fails, paste the FIRST `error` block verbatim and do not edit
   around it. Changed here: studio.rs, panes/captions.rs, and the two
   .slint files. Slint errors are reported with a file name and line; paste
   those.

3. By eye: start the app, select a voice caption, open the text inspector
   colour section, and check the "Spoken word" swatch. Change the colour
   and watch the stage while the clip plays.

EXPECTED
--------
  Step 1: 237 passed.
  Step 2: compiles with no errors.
  Step 3: the spoken word lights in the colour you picked.

WHAT WAS CHECKED IN THE SANDBOX, AND WHAT WAS NOT
-------------------------------------------------
  Run, passing:  concat-project 237 tests, clippy with -D warnings clean,
                 rustfmt clean on model.rs, captions.rs and studio.rs
                 (parse and format check).
                 en.json is valid JSON, and the new key follows the
                 lowerCamelCase rule the i18n test enforces.
  NOT compiled:  the concat app, including studio.rs and the .slint files
                 (Slint is compiled by the app's build). Written by reading
                 the existing ColourField and stroke-colour code, which
                 this copies. The first real compile is step 2.
