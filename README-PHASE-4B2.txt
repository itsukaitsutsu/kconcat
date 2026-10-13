PHASE 4b-2 - THE SECTIONS PANEL  (for itsukaitsutsu/kconcat)
=============================================================

WHAT THIS IS
------------
The picture half of Phase 4b, sitting on 4b-1's tested numbers.
A new "Sections" tool in the timeline tray (next to Captions):

  1. Click it. The sheet opens on the clip you selected (if it has
     sound), else the first clip with sound, and listens for silences
     on a worker - the window never freezes.
  2. The list appears: one row per caption / silence / other piece,
     with the caption's words, its timeline times, and a switch.
     Silence starts UNticked, everything else ticked.
  3. Flip switches to taste. The bottom line - "Keeping 0:42 / 1:00" -
     is the promise the autotrim will be held to.

There is NO Apply button yet, deliberately: turning ticks into cuts
is Phase 5 (autotrim), and a button that pretends is worse than none.
Old captions (made before Phase 4a) still count: the list reads the
clip's linked captions, so a clip captioned before 4a shows its
silences and "other" but no caption rows. That is the truth about
that clip, not a bug.

FILES TO REPLACE / ADD (11 code files)
--------------------------------------
  NEW  src/crates/concat/src/panes/sections.rs
  NEW  src/crates/concat/ui/dialogs/sections.slint
  src/crates/concat/src/panes/mod.rs        (registers the pane)
  src/crates/concat/src/studio.rs           (holds it, routes, publishes)
  src/crates/concat/src/lib.rs              (tray/dialog callbacks)
  src/crates/concat/ui/app.slint            (the dialog + its data)
  src/crates/concat/ui/editor.slint         (sections callback)
  src/crates/concat/ui/workspace/seat.slint (forwards it)
  src/crates/concat/ui/workspace/timeline-pane.slint (forwards it)
  src/crates/concat/ui/timeline/tray.slint  (the tray button)
  src/crates/concat/locales/en.json         (the sheet's English words;
      other languages fall back to English, as every new key does)
Plus docs: README-PHASE-4B2.txt, CHECKPOINT.md.

Apply the phases in order: 2, then 4a, then 4b-1, then this. (Each zip's
shared files are the newest versions, so re-extracting in order lands
in the right place.)

HOW TO VERIFY
-------------
Step 1 (cheap, always first):
  cd C:\dev\Concat\src
  cargo test -p concat-project
Expected: 223 passed. This phase did not touch that crate.

Step 2 (THE EXPENSIVE ONE - 30-90 min, plan for it):
  cargo check -p concat
This phase was NOT compiled before packaging: the sandbox has no
FFmpeg/Skia, so the app crate cannot build there. The Rust files were
rustfmt-parsed and the cheap crate stayed green, and the wiring copies
the Captions sheet line for line - but you are the compiler here.

- If it finishes clean: run `cargo dev`, select a clip, click the new
  Sections tray button, and check you see rows. Say so in the next
  session, with what the list showed.
- If it errors: copy the FULL output from the first line starting
  with `error` (include the file and line it names), paste it into
  the next Arena session with CHECKPOINT.md. Do not edit around it.
  The likely spots, in order: a Slint name in dialogs/sections.slint,
  the generated SectionRowData fields in panes/sections.rs, or a
  callback name in tray/timeline-pane/seat/editor. Each is a
  one-line fix in the next zip.

WHAT IS NEXT
------------
Phase 5 - Autotrim: an Apply on this sheet that turns the ticks into
cuts (split at the kept boundaries, ripple-delete the rest, in one
Batch - Phase 2 makes the captions follow for free). Then
word-level captions / karaoke. See CHECKPOINT.md.
