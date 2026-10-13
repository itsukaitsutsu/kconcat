PHASE 6c-4 - TURN THE SPOKEN-WORD HIGHLIGHT OFF AND ON  (for itsukaitsutsu/kconcat)
=================================================================================

WHAT THIS IS
------------
6c-2 added a colour swatch for a caption's spoken-word highlight, but there
was no way to turn the highlight off again. 6c-4 adds a "Spoken word"
switch above the swatch, on captions only.

  * Switch on  -> the caption lights its spoken word in the default yellow
                  (or the colour already chosen). The swatch appears.
  * Switch off -> the caption's highlight colour is cleared, the swatch
                  hides, and the caption plays as plain text.

Switching back on restores the default yellow. Picking a colour with the
swatch still works as in 6c-2.

HOW IT WORKS
------------
The switch sends the colour through the same path as the swatch. A
transparent colour means "no highlight": the studio stores it as an empty
highlight colour, which is what an empty value has always meant.

FILES TO REPLACE (3 code files + 2 docs)
----------------------------------------
  src/crates/concat/ui/timeline/model.slint        (SelectedClipData gains
      highlight-on)
  src/crates/concat/ui/inspector/text-panel.slint  (switch above the swatch;
      swatch shown only when on)
  src/crates/concat/src/studio.rs                  (transparent -> none;
      highlight_on filled in)
  README-PHASE-6C4.txt                             (this file - new)
  CHECKPOINT.md                                    (updated)

Apply this zip ON TOP OF the cargo6 state, with 6c, 6c-2 and 6c-3 already in.

HOW TO VERIFY
-------------
In a FRESH VSCode terminal (close VSCode fully and reopen first).

1. Project (fast, runs in the sandbox too):
     cd C:\dev\Concat\src
     cargo test -p concat-project
   Expected: test result: ok. 237 passed; 0 failed

2. App - NOT COMPILED IN THE SANDBOX (30-90 minutes the first time):
     cargo check -p concat
   If it fails, paste the FIRST `error` block verbatim and do not edit
   around it. Slint errors name a file and line; paste those.

3. By eye: select a voice caption, open the colour section, switch "Spoken
   word" off. The swatch should hide and the stage should show the caption
   plain while the voice speaks. Switch it on again: the swatch returns
   yellow and the spoken word lights again.

EXPECTED
--------
  Step 1: 237 passed.
  Step 2: compiles with no errors.
  Step 3: switch off means no lit word; switch on means the lit word is back.

WHAT WAS CHECKED IN THE SANDBOX, AND WHAT WAS NOT
-------------------------------------------------
  Run, passing:  rustfmt clean on studio.rs.
  NOT compiled:  the concat app, including studio.rs and the .slint files.
                 Written by copying the existing SwitchRow and ColourField
                 patterns already used in text-panel.slint.
