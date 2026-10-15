Burn captions into the video (kconcat, 2026-10-15)

WHAT CHANGED
The export dialog's Output section has a "Burn captions into the video"
switch. It shows for the re-encodes (H.264, HEVC, AV1) and is hidden for
"Same as copy". It is on by default, so exports behave as before. When it is
off, the export does not paint the caption (text) clips into the picture; shape
clips are still drawn. The .srt sidecar and the soft subtitle track for
"Same as copy" are unchanged.

FILES (apply over /home/user/kconcat)
  src/crates/concat/ui/dialogs/export.slint   switch row; burn-captions on ExportData; callbacks
  src/crates/concat/ui/app.slint              export-burn-captions-changed callback, forwarded
  src/crates/concat/src/lib.rs                wires the callback to ExportMsg::BurnCaptionsChanged
  src/crates/concat/src/panes/export.rs       burn_captions state (default true); filter on title clips
  src/crates/concat/locales/en.json           export.burnCaptions (other locales fall back to English)

NOTES
- app.slint and lib.rs are copied whole from the working tree. They also contain
  the language-search changes (captions-language-query-changed, LanguageQueryChanged).
  If you already applied the language-search overlay, nothing changes in them.
- Not compiled here: the toolchain was lost in the sandbox reset. Please run
  `cargo check -p concat` and tell me the first error verbatim if there is one.
- If captions are still missing with the switch on, check the log for lines
  starting "title " followed by an error. Those are captions that failed to paint.
