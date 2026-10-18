# Checkpoint 4 — roadmap v2 steps 10–12 (E1, E2, V2-loudness)

Date: 2026-10-18 · Repo: kconcat @ db3bab5 · Continues checkpoints 1–3 (T1–T7, C1val, E4).

## Step 10 — E1. Delete a word, the video cuts

- `Command::DeleteCaptionWord{clip_id, word_index, cut_video}` — the A2 checkbox
  ("Potong video saat kata dihapus") arrives as the bool, ready for any UI.
- Checkbox OFF: only text goes — the word leaves the content at its located bytes,
  T7's single writer prunes its timing slot, survivors keep their heard times,
  picture and sound stand still.
- Checkbox ON: delegates to the remove-selection machinery, as the roadmap predicted
  ("about 80% exists"): `AutotrimClip` on the clip the caption follows (caption_of,
  else the clip the word is heard over), kept = owner span minus the word's heard
  span → every lane cut at the same moments, gap closed, captions rebuilt with
  remapped words and repaired links. Husk rule on top: a caption whose last word
  went is removed, not left wordless.
- Refusals are user sentences (`CannotDeleteWord{reason}`). One command = one undo
  (tested on the cut path). Prescribed tests: one word ripples audio+picture+captions;
  a sentence (word by word, left to right) keeps the sync matrix on the survivor;
  one undo restores everything. Plus text-only mode + four refusals. RED 5/5 → GREEN.
- UI checkbox NOT built: §9 holds E1–E6 UI until checked in the running app.

## Step 11 — E2. Filler-word removal (Indonesian)

- Two opt-in tiers exactly per A7 — strong: `eee, anu, ehm`; weak: `kayak, gitu,
  terus, apa ya` — the one fact in `src/filler.rs`, recorded in the new manual
  `03-manuals/C-filler-words.md` (the manual the roadmap asks for).
- Matching: whole words only ("begitu" ≠ "gitu"), case/edge-punctuation ignored,
  "apa ya" needs its consecutive run, longer entries win first.
- `Command::RemoveFillerWords{clip_ids, strong, weak, cut_video}`: selections may
  hold picture (skipped, not errors); a no-hit pass is not an edit (`applied:false`,
  no nothing-snapshot); one pass = one undo. OFF = text-only (T7 rule); ON = merged
  heard spans cut through E1's path, one autotrim per owner left to right, scoped
  husk sweep (pre-existing captions — including imported wordless cues — are never
  swept).
- **One source fix inside the existing autotrim rebuild**: a rebuilt text piece that
  lost ALL its words kept the stale full sentence as content (the re-join sat inside
  `!kept.is_empty()`) — the exact repetition its own comment forbids. Re-join moved
  out; empty text now marks the husk. Whole existing suite stayed green.
- Tests: 5 matcher units + 5 integration (the prescribed negative case included:
  "terus" survives unticked; "begitu" survives ticked). RED 10/10 → GREEN.
- UI option next to "silent detected" NOT built (§9, same as E1).

## Step 12 — V2 loudness-first half

- `concat.loudness` retargeted per the owner's explicit A4 decision ("reuse this
  effect"): default **−23 → −14 LUFS**, FFmpeg chain true-peak **−1.5 → −1.0 dBTP**;
  range −30..−12 unchanged. The per-clip toggle (A4 option D) is this effect, so it
  now targets −14.
- `concat-effects` turned out **sandbox-verifiable** (no FFmpeg deps — it parses
  manifests and renders chain strings). Test-first via its builtin-fixture harness:
  fixtures pinned to the new chains → RED (all 3 loudness cases named, other 71
  green) → manifest change → GREEN **72/72**, clippy clean.
- Platform-preset half (4 presets, A5 bitrate/audio fields, master-mix loudnorm per
  A4 point 4) **NOT built here**: render-path surgery across export.slint, concat,
  concat-export, FFmpeg invocation — none compilable in this sandbox — and A5 itself
  says "Build waits for the diff" (sizes still "proposed, not yet confirmed"). The
  acceptance test is fixed and quoted in the roadmap for that session: exported file
  at Integrated −14 LUFS ±1 LU, true peak ≤ −1 dBTP.

## Verification sweep at this checkpoint

- concat-project: **264 lib + 29 integration** tests green (RED shown first per task).
- concat-effects: **72/72** green. concat-timing: 24/24 green.
- clippy `-D warnings` and fmt clean on all three.
- Logs in this zip: step10-e1-{RED,GREEN}, step11-e2-{RED,GREEN}, step12-v2-loudness-{RED,GREEN}.

## Note on repo history

The sandbox restore twice reverted `.git` to the base commit while leaving the
working tree intact; the state was re-verified (markers + full test suite) and
recommitted each time (420f8e2, then 29b3f01). The patch series below is the
canonical incremental history from base `9e9478e`.
