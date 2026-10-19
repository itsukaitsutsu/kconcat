kconcat batch 05-09 overlay
===========================

Copy these files ON TOP of the same paths in C:\dev\kconcat
(keep your beat.rs, caption/scroll local edits if those files are NOT listed here).

Files in this zip (replace each):
  src/crates/concat-project/src/platform_export.rs   (NEW)
  src/crates/concat-project/src/lib.rs
  src/crates/concat/src/panes/captions.rs
  src/crates/concat/src/panes/export.rs
  src/crates/concat/src/panes/import_captions.rs
  src/crates/concat/src/studio.rs
  src/crates/concat/src/lib.rs
  src/crates/concat/ui/dialogs/captions.slint
  src/crates/concat/ui/dialogs/export.slint
  src/crates/concat/ui/app.slint

Do NOT copy over:
  src/crates/concat-timing/src/beat.rs
  src/crates/concat-timing/src/lib.rs
  src/crates/concat/ui/workspace/*.slint
  TEMP-kconcat-remaining-plan-checklist.md

From C:\dev\kconcat\src in the real build prompt:

  cargo test --locked -p concat-project
  cargo check --locked -p concat
  cargo dev

What this batch adds (not closed as user-facing until you run it):
  05 Look picker on Captions (Default + 8 looks); karaoke fill only if no look chosen
  06 no extra edit path
  07 Link / Unlink on clip menu, Ctrl+L
  08 Export platform: YouTube TikTok Reels Shorts (bitrate hint; no sequence rewrite)
  09 Audio clip menu: Duck under speech

03/04 real-media quality is still yours.
