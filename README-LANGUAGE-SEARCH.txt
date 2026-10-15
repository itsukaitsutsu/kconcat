FIX (2026-10-15, after the first Windows build)
cargo dev failed with E0308 at crates/concat/src/panes/captions.rs:1240
(sync(&self.sources_model, labels) got the language list). Cause: the new
language code declared `let labels = language_labels();` in the same function,
which shadowed the sources `labels` above it. The language list is now
`language_names`; the sources list is back as `labels`. Nothing else changed.
This slipped through because the concat crate was not compiled here. The
cargo check below is still the first thing to run.

Caption language search (kconcat, 2026-10-15)

WHAT CHANGED
The Auto-caption dialog's Language list is now searchable. Open the list and
the search field is focused at the top: type part of a language's name or its
code ("jap", "Korean", "ko") and the list narrows to the matches. You no
longer have to scroll the 99 languages.

- The search matches anywhere in the label, ignoring case, so "ja" finds
  Japanese and Javanese, "jap" finds only Japanese.
- Up/Down move the highlight through the matches, Enter picks the highlighted
  one (after a new search that is the best match, the first row), Escape
  closes the list. Space types a space in the search field.
- Clicking a row picks it, as before.
- The trigger always shows the chosen language, even while the search hides
  it from the list. The list reopens with the search cleared.
- Language choice is unchanged: the same 99 Whisper codes, the same
  transcription behaviour. Only the way you find one is new.

Other dropdowns are unchanged. The search is opt-in on the shared Select
(`searchable: true`); every other Select in the app behaves as it did.

FILES (paths mirror the repo root; apply over /home/user/kconcat)
  src/crates/concat/ui/primitives/select.slint   opt-in search band, query, value-text, step()
  src/crates/concat/ui/dialogs/captions.slint    language Select made searchable; language-text on CaptionsSheetData
  src/crates/concat/ui/app.slint                 captions-language-query-changed callback
  src/crates/concat/src/lib.rs                   wires the query callback to CaptionsMsg::LanguageQueryChanged
  src/crates/concat/src/panes/captions.rs        language_query state; matching_languages(); rows mapped back to the full language index; tests

WHAT WAS VERIFIED
- The Slint tree compiles: app.slint (with every .slint file it imports) builds
  with slint-build 1.18.1, the version the repo pins, and embedded resources as
  concat's build.rs does. A deliberately broken copy fails, so the check is live.
- matching_languages() and the two new tests (search rules, and every real
  label findable by name and by code) pass when compiled on their own against
  the real label table (2 passed).

WHAT WAS NOT VERIFIED (run these on your machine)
- The concat crate was NOT compiled in this sandbox (Skia, FFmpeg, whisper and
  ORT are too heavy here). Please run:
      cd src
      cargo check -p concat
      cargo test -p concat captions
- Behaviour was not exercised on screen. Please check, in cargo dev:
  1. Open Auto-caption > Language. The search field is focused and empty.
  2. Type "jap": one row, Japanese. The trigger still shows the chosen language.
  3. Up/Down, Enter picks; the trigger shows the new language; reopening clears the search.
  4. Type "zzz": the list is empty; Escape closes it; the chosen language is still on the trigger.
  5. Click a row with the mouse while a search is active.
  6. Interface language (Settings) and the other dropdowns still look and behave as before.

NOTES
- The sync fixes (earlier overlay, kconcat-sync-fixes-overlay.zip) are
  separate; this overlay touches none of their files.
- Known limit: an empty search result shows an empty list with no message;
  a "No matches" label would need a new locale key in all 24 locale files.
