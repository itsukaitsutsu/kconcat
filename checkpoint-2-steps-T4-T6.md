# Checkpoint 2 — roadmap v2 steps 4–6 (T4, T5, T6)

Date: 2026-10-18 · Repo: kconcat @ 57ac21e (previous checkpoint: 38e65af) · Continues checkpoint 1 (T1–T3).

## Step 4 — T4. Script-aware tokenisation

- The rule stayed **one copy**: `captionmap` gained public doors `word_units` (its split:
  Unicode `is_alphanumeric` boundaries; per-character units with trailing punctuation for
  space-less scripts) and `expand_units` (span spreading, proportional to unit length).
- Prescribed tests red first in `concat-project` (3 failures, `todo!()`), then green:
  **255/255** including every existing `clean_words` test; clippy/fmt clean.
- `concat-speech` (NOT verifiable here): depends on `concat-project` (acyclic), and
  `words_from_tokens` runs each decoded group through `expand_units` — both engine paths.
  The pre-T4 test `unspaced_text_is_one_word_not_a_guess` codified the old behaviour and was
  rewritten (`unspaced_text_splits_into_captionmap_units` + a 7-character Japanese run test).
- **The limit, as the task requires:** this is NOT a true Thai/Japanese word segmenter —
  character-level units with estimated intra-group timings, not dictionary words.

## Step 5 — T5. Caption identity — stored, not derived

- Recorded choice: **explicit flag** `Clip::caption_flag: Option<bool>` (not `caption_of`,
  whose absence for imported cues *is* the bug). `Some` at creation; `None` = pre-T5 document
  → `is_caption()` derives the old way; `DOCUMENT_VERSION` does not move; `tidy()` carries it.
- The patch's three tests (`04-findings/caption-identity-bug.patch`, applied verbatim):
  **red 2/3 → green 3/3**; prescribed fourth (old document derives as before; stored flag
  outruns derivation both ways) added; **256/256**; clippy/fmt clean.
- Grep audit: caption-ness rule body exists once (`model.rs`) + the one documented birth site
  (`clips.rs` creation). All six AGENTS.md invariant-2 sites call the predicate; the three in
  the `concat` crate (export.rs, sections.rs, studio.rs) are **not verified here** (FFmpeg+slint).
- Deliberately untouched: offset-stacking and highlightability reads ask different questions.

## Step 6 — T6. Per-track nudge offset

- `Track.caption_nudge_ms` (serde default 0, skipped at zero — old documents load unchanged,
  un-nudged tracks serialize byte-identical). `Command::NudgeCaptionTrack{track_id, delta_ms}`:
  ±10 ms quantisation (ties away from zero), accumulated, clamped once in the handler to ±1 s;
  at-cap reports `applied:false` (no nothing-snapshots in undo history); undo tested.
- Handler shifts every caption on the lane (by T5 identity, not words-presence) — clip starts
  **and** word timings together. `AddTextClip` applies the lane nudge at birth → survives
  regeneration (remove + re-add tested).
- Tests red first (4 failures vs `todo!()` handler), then green: **256 + 3 + 5**; clippy/fmt clean.
- Nothing outside `concat-project` needed changes (preview/burn-in read the shifted state;
  external references to neighbouring commands are constructions, not exhaustive matches).
- Not part of this task's change line: the UI affordance that issues the command.

## Evidence in this zip

- `patch/0001-T4-T6-*.patch` — the full step 4–6 diff.
- `logs/step4-t4-tests-{RED,GREEN}.log` — captionmap red (3 failures among 252 passing) and green (255/255 + fmt).
- `logs/step5-t5-tests-RED.log` — the identity patch red: 2/3 failing exactly as documented.
- `logs/step5-t5-tests-GREEN.log` — 256 unit + 3/3 patch.
- `logs/step6-t6-tests-{RED,GREEN}.log` — nudge red (4 failures) and green (256 + 3 + 5).

## Verification status (plainly, per AGENTS.md)

- Verified: everything in `concat-project` (tests, clippy `-D warnings`, fmt).
- NOT verified here: `concat-speech` (T4 expansion + rewritten tests), `concat` crate
  (T5 predicate delegations). Written carefully, compile-unchecked — no FFmpeg/whisper.cpp/slint
  in this sandbox. They must not be called "pass".

## Next (auto-continuing)

Step 7 — T7 (one writer for text and words), step 8 — C1 validation, step 9 — E4;
checkpoint 3 after step 9.
