# Checkpoint 5 — roadmap v2 steps 13–14 (C2, C3) — FINAL of this 5-step run

Date: 2026-10-18 · Repo: kconcat @ 995d8c4 · Continues checkpoints 1–4.
Completes the owner-ordered 5 further steps: E1 → E2 → V2(loudness) → C2 → C3
(14 roadmap steps done in total, in §11 order).

## Step 13 — C2. Caption presets + group tag

- The eight looks of manual B-caption-styles §4 ship as data:
  `text-presets/kconcat.{pop-bold, boxed, karaoke, minimal, hard-shadow,
  upper-tight, low-third-cap, clean-bilingual}/preset.toml` — the manual's
  weights, sizes, strokes, plates, highlight colours and positions; `shadow`
  explicit everywhere (parser default is true; the table gives the shadow to
  hard-shadow alone). Owner A9 honoured: no Indonesian variant, all eight ship.
- Group tag: `kind = "caption"` in the files → `PresetFile.kind` →
  `TextPreset.kind`; absent = "title", so every pre-tag preset is unmoved. The
  Text page's card builder filters the caption group out of its shelves —
  "not mixed with title looks", literally.
- The prescribed test-first item: `PresetStyle.highlightColor` →
  `TextStyle.highlight_color` (the hard-coded `String::new()` in `over()` —
  "a preset is a look, not a caption: no highlight" — replaced). Round-trip
  test + eight-ship-tagged test written in `presets.rs`'s own test module.
- **All of this is in the `concat` crate: written carefully, NOT VERIFIED here**
  (slint/FFmpeg — cannot compile in this sandbox). What the sandbox could
  check, it checked: the 8 TOMLs parse (tomllib) with the right kind and
  values; `TextPreset` has exactly one construction site; fmt clean.
- Owner-machine remainder, recorded not guessed: the captions dialog's look
  picker (§9 holds dialog UI until checked in the running app; today the
  dialog's "presets" are C1's layout presets and generated captions take
  `prefs.title_style`), and upper-tight's "inverse" highlight — data carries
  the colour, the inverse drawing mode needs the owner's call.

## Step 14 — C3. Wire karaoke to presets

- The wiring C3 asks for IS the data C2 ships, read plainly from the code:
  "on" = non-empty `TextStyle.highlight_color` (inspector: `highlight_on`;
  the renderer lights each heard word with it — the existing
  `a_word_timed_caption_gets_one_highlight_per_heard_word` test in
  `concat-host` covers the renderer, **not verified here**). Before this run,
  `over()` hard-emptied the colour — exactly "no preset turns it on".
- Now: pop-bold (#ffd21f), karaoke (#39ff88), upper-tight (#ffffff) start ON
  (A10); the other five start OFF. "Every other preset can turn it on" = the
  existing inspector switch, which shows for any clip with words
  (`highlightable: clip.words.is_some()`) — every caption has words, so no
  gate change was needed or made.
- Recorded for the picker session (deliberately NOT changed now):
  `panes/captions.rs:1566` force-sets `DEFAULT_HIGHLIGHT` on generated
  captions with no highlight colour. When the picker lands, it must become
  "only when nothing has decided yet" — changing it today would switch off
  karaoke on every generated caption app-wide, a UX change nobody asked for.

## Verification sweep at this checkpoint (all sandbox-verifiable crates)

- concat-project: 264 lib + 29 integration — green.
- concat-effects: 72/72 — green (includes the V2 loudness fixtures).
- concat-timing: 24/24 — green.
- clippy `-D warnings`, fmt clean (concat-project, concat-timing,
  concat-effects; fmt also run on concat).
- C2/C3 additions compile-checked by nobody here — see the disclosures above.

## The 14 steps: T1 T2 T3 (cp1) · T4 T5 T6 (cp2) · T7 C1val E4 (cp3) · E1 E2 V2 (cp4) · C2 C3 (cp5)

Next in §11 order would be: E1 ✓ done → E2 ✓ done → … the order line continues
after C3 with: C4 (case transform) → V3 (ducking) → V4 (batch queue) → P3 → V1 →
C5 (after D3) → E3 → P4 → P7 → P6.
