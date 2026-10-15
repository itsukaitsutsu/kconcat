kconcat - all fixes in one overlay (2026-10-15)

APPLY
Unzip over your repo root (the folder that contains src/):
    unzip -o kconcat-all-fixes.zip -d <your kconcat folder>
It contains full copies of every changed file from the working tree, so
do not mix it with the older separate overlays. It replaces them.

WHAT IS IN IT
1. Remove selected / sync (concat-project, concat)
   - AutotrimClip rebuilds every clip that overlaps a cut on every lane
     (sound, captions, music beds). An empty cut removes the whole section
     on every lane and closes the gap on every lane.
   - Tests: concat-project lib.rs (3 new, plus 1 updated). 250 passed here.
   - Sections list: ticks kept across edits, list refreshes after edits,
     edits no longer overwrite other caption tracks, timeline section strip
     follows captions.
2. Caption language search (Auto-caption dialog)
   - Searchable language list. Compile error fixed: `labels` shadowing in
     panes/captions.rs (now `language_names`).
3. Burn captions into the video (export dialog)
   - "Burn captions into the video" switch in the Output section, shown for
     re-encodes only. On by default; off skips caption paint.
   - en.json key export.burnCaptions; other locales fall back to English.
4. Sections icon removed from the timeline toolbar
   - Only the rows button, tooltip "Sections: what this clip is made of".
   - The "Remove selected" pill is still there.

NOT FIXED
- Scrolling the Sections list can zoom the timeline (Ctrl+scroll behaviour).
  Cause not found yet.
- Captions can vanish from the timeline when zoomed in (cause not found).
- Stream-copy ("Same as copy") still drifts in audio length and ignores
  gaps, speed and effects. Needs a decision.
- Track locks are not respected by Sections removal (UI state, not changed).

NOT COMPILED HERE
The Rust toolchain was lost in the sandbox reset, so the concat crate and
the Slint changes are not compiled. concat-project was built and tested
before the reset. Please run:
    cd src
    cargo check -p concat
    cargo test -p concat-project
and send me the first error verbatim if there is one.
