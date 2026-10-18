# Checkpoint 3 — roadmap v2 steps 7–9 (T7, C1 validation, E4) — FINAL of the 9-step run

Date: 2026-10-18 · Repo: kconcat @ 0ec7a6a (previous checkpoint: 57ac21e) · Completes the
owner-ordered first 9 steps (T1→T2→T3→T4→T5→T6→T7→C1val→E4).

## Step 7 — T7. One writer for text and words

- The suspected bug was confirmed by reading: Sections' `edit_row_text` already routes
  through `Command::UpdateClip` (content-changed style); the handler wrote the style and
  left `clip.words` stale → `locate_words` dropped the karaoke highlight.
- Fix lands where the single-writer rule lives — the `UpdateClip` text arm in
  `concat-project/src/commands/properties.rs` (`reconcile_caption_words`). `sections.rs`
  needed no change. Rules: same token count (`captionmap::word_units`, T4's one split rule)
  re-texts in position keeping every timing; different count prunes multiset-wise in old
  order, never inventing; cleared style clears words; the stored caption flag (T5) is
  untouched; titles fall through.
- Tests: 3 prescribed + reorder + repeated-word multiset + style-clear; sync matrix (§2)
  asserted after every edit. Red first (5/6), then green. **Zero unverified-crate changes.**

## Step 8 — C1 validation (no new UI)

- `src/crates/concat-project/examples/c1_burst_check.rs` drives the shipped
  `resplit_cues(.., Burst)` over the Tier-1 runner's own outputs
  (`06-corpus/runs/{baseline,snapped}-synthetic.json`). Captured table:
  `06-corpus/runs/c1-burst-check-synthetic.txt` (also in this zip).
- Result: every burst passes its policy limits (≤3 words, ≤18 chars, ≤2.5 s), ordered,
  non-overlapping, no word lost — before AND after T2. Tier 1: violations 2→1, outliers
  2→0, onset 90.9% both, WER unchanged. T2 visibly improves chunking: the mis-timed
  "tetap" (2.49 s, at the cap) stranded "jaga" as a 0.05 s sliver before; after the snap,
  "tetap jaga" is one 0.89 s burst — 6 chunks → 5.
- Limits, plainly: synthetic episode (T8's real corpus does not exist yet); the by-eye
  comparison in the running captions dialog needs the GUI — **not verified here**, left for
  the owner with the same example.

## Step 9 — E4. Enter splits a caption at the caret

- Mechanism (verified, `concat-project`): `captionmap::word_at_caret` (caret → the word it
  falls in; gap caret → next word; None when a side would be empty) and
  `Command::SplitCaptionAtWord{clip_id, word_index}`: cut at the word's **heard start**
  (audio boundary, never text length — owner A8), content split at the located bytes, head
  keeps id/transition/way-in, tail minted at the boundary on the same lane (look, offset_y,
  caption_of kept; heard word times stand; no second lane-nudge, T6), flag born Some(true)
  (T5). Refusals are user-facing sentences via `CommandError::CannotSplitCaption{reason}`:
  no heard timings → "refused with a message, not guessed". One command → one undo.
- Tests: 3 prescribed + no-timing refusal + degenerate-index refusal + 3 caret unit tests.
  Red first (5/5 + 3/3 vs `todo!()` stubs), then green. Sync matrix on both halves.
- UI wiring — **written but NOT VERIFIED here** (slint/FFmpeg crates cannot compile in this
  sandbox): modal.slint `Field` declares/forwards `accepted(text, caret)` (following the
  speech.slint:364 `cursor-position-byte-offset` precedent); `row-text-accepted` forwarded
  at every hop beside `row-text-edited` (sections.slint list→dialog, SectionsView global,
  app.slint); `sections.rs::accept_row_text` commits the pending edit first (T7), maps the
  caret, issues the command — `Studio::apply` already toasts refusals — and refreshes rows;
  lib.rs hooks both callback paths (`on_window!` is variadic — checked).
- Deliberately left unwired, recorded: the captions dialog's own draft editor
  (captions.slint:484) — its draft↔clip mapping needs runtime investigation; not blind-patched.

## Final verification sweep (whole 9-step run state)

- `cargo test -p concat-project -p concat-timing`: **259 lib + 5 nudge + 3 identity +
  6 one-writer + 5 enter-split + 24 timing = all green, 0 failed.**
- `cargo clippy -p concat-project -p concat-timing --all-targets -- -D warnings`: clean.
- `cargo fmt --check` (concat-project, concat-timing, concat): clean.

## Evidence in this zip

- `patch/0001-T7-C1-validation-E4-*.patch` — the full step 7–9 diff.
- `logs/step7-t7-tests-{RED,GREEN}.log`, `logs/step8-c1val-burst-synthetic.log`,
  `logs/step9-e4-tests-{RED,GREEN}.log`.
- `runs/c1-burst-check-synthetic.txt` — the C1 before/after burst table.

## Not verified in this whole run (standing disclosure)

`concat-speech` (T4), `concat` crate delegations (T5), and the E4 UI wiring: written with
extra care, compile-unchecked — no FFmpeg / whisper.cpp / slint toolchain in this sandbox.
They are never claimed as "pass". Real-corpus acceptance (T8 transcript) remains open.

## The 9 steps are complete: T1 ✓ T2 ✓ T3 ✓ (checkpoint 1) · T4 ✓ T5 ✓ T6 ✓ (checkpoint 2) · T7 ✓ C1val ✓ E4 ✓ (checkpoint 3)
