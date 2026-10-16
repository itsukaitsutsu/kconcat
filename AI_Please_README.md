# kconcat checkpoint — 16 October 2026 (v2)

**Session type: discussion and analysis only.** No application code was changed, and nothing in the repo was modified. This checkpoint and the two files it points to were written to the owner's workspace, not pushed to the repo.

> **File naming:** this file is `AI_Please_README.md` on purpose. An agent session should find and open it first. Use the same name for every future checkpoint.

---

## 1. Where the discussion stands

- **Option B (the feature list vs CapCut Pro):** re-checked and fixed in `05-background/kconcat-feature-gap-option-b-v2.md`. It is ranked for **long-form podcast editing** with **short-clip output**.
- **Option C (caption styles end to end):** BELUM dibahas (not yet). Caption motion is confirmed absent. D3 is selected as (a), `[motion]` in `preset.toml`; the schema is not yet written.
- **Option D:** open.
- **#15 "Tuning ASR Indonesia":** approved with condition (16 Oct 2026): T8 and a WER output on T1, only where they serve the goal of getting closer to CapCut Pro. Fine-tuning is not approved.
- **Languages:** Indonesian is the priority (T8, E2). English (X1) and Japanese (X2) are additional features, added after Track 1 exit, per owner (16 Oct 2026).
- **D4 (snapper location):** selected B: new pure crate `concat-timing`, no dependencies, so its tests run in the sandbox.
- **D5 (harness location):** selected B. Cache made once in `concat-speech`; metric runner and WER in `concat-timing` (sandbox).
- **D1 (timing reference):** closed. The 30 s Whisper sync clip is the reference.

---

## 2. Owner context (this session)

- Main use: **long-form podcasts** (60+ minutes), cut into short clips.
- Reference style: **prof.zulys**-style captions, edited in **CapCut Pro**.
- Goal: get **closer to CapCut Pro**. Being behind is accepted; closing the gap is the aim.
- **Lower priority now:** B-roll and multi-clip (reported to exist, untested). Caption animation (reported to exist, not verified in the repo).

---

## 3. Repo state (as of this checkpoint)

| | |
|---|---|
| Repository | `itsukaitsutsu/kconcat` |
| `main` latest commit | `f047c91` ("please read") — docs under `01-rules/` through `06-corpus/` |
| App source (`src/`) latest commit | `1aece2d` ("plan to get closer") |
| Verified in sandbox | `cargo test -p concat-project` → 252 passed, 0 failed (from the earlier checkpoint) |
| Not verified | every crate linking FFmpeg (`concat-media`, `concat-render`, `concat-export`, `concat-host`, the GUI) |

**Path correction:** all code paths in the docs are relative to `src/`. Example: `src/crates/concat/ui/dialogs/captions.slint`.

---

## 4. Decisions made this session

1. **Language:** English in chat and in every repo file.
2. **Personal tool, not a public project.** No repo hygiene, CI, README, release or licensing advice.
3. **Scope:** simple tasks — captions, silence trim, captions-follow-the-cut, export, plus long-form podcast editing and short-clip output.
4. **Local AI only.** Cloud AI is out.
5. **Discussion material stays out of coding sessions.** The `05-background/` folder is the owner's reasoning trail.
6. **One fact lives in one file.** Other files link to it.
7. **The test corpus is the existing 5-minute video.** Podcast tests will extend it to a full episode.
8. **Priority for long-form:** word timing → transcript editing → loudness and platform presets → caption presets and karaoke → ducking → batch export → speaker labels → auto-reframe. B-roll and animation move down until verified.

---

## 4b. Plan fixes recorded in the .md (16 Oct 2026)

Rule: a plan item is **fixed** once its definition, approach, target and order are written in the .md files. Code status is tracked separately.

- **A1 (word timing):** fixed in plan — T1 harness, T2 onset snapping, T3 ZCR + spectral flux, T4 script-aware tokenisation, each with a target. Code: not built.
- **T5 (caption identity):** fixed in plan. Code: not built.
- **T7 (one writer for text and words):** fixed in plan, added this session; must precede E4 and A2. Code: not built.
- **A8 / E4 (Enter-split):** fixed in plan; order after T7. Code: not built (Enter unwired at `1aece2d`).
- **A2 / E1 (transcript editing):** fixed in plan; depends on T2/T3. Code: not built.
- **C1 (word chunking):** fixed in plan as already built. Code: built.
- **C5 (caption motion):** plan open (schema decision pending). Code: absent.

Full table: `02-map/kconcat-roadmap-v2-execution.md` §11.

## 5. What was found (carried forward)

**Confirmed bug (reproduced with three tests, `04-findings/caption-identity-bug.patch`):** a caption from an imported `.srt`/`.vtt` loses its identity when a trim removes every word inside it. The clip stops being a caption. With "Burn captions" off, it can still be burned in. Identity is derived, not stored, and the check is duplicated in six places across three crates.

**Suspected defect (reasoned from code, not tested):** an inline text edit in Sections rewrites `style.content` and leaves `clip.words` stale. The karaoke highlight then drops those words.

**Confirmed absent (16 Oct, this session):**
- Caption motion. None of the 32 presets set it, and the caption dialog has no control for it.
- Caption highlight in any preset (`highlight_color` is unset in all 32).
- Case transform in the text style.
- Platform export presets; loudness is applied only before transcription.
- Speaker labels, auto-ducking, screen and webcam recording, vocal isolation, auto-reframe.

**Confirmed present:** word chunking (four layout modes, Re-split), waveform drawing, Whisper translate-to-English, SRT/VTT import and export, captions that follow cuts, the 123 effect packages, the API and CLI.

---

## 6. Open items

- **T1 metric runner** — the prerequisite for measuring word-timing work. Still not started. This is the first code task when coding resumes.
- **Motion schema decision** (roadmap C5 blocker) — three options: a `[motion]` block in `preset.toml`, inspector-only, or an effect package.
- **Caption pipeline map** — transcription → `captionmap` → placement → reveal → render → export, with file:line per hop.
- **#15 decision** — add Track-1 T8 and the WER output on T1, or not.
- **Verify** B-roll / multi-clip and caption animation in the running app.
- **Podcast episode test** — a full-length episode in the corpus, for P5.

---

## 7. Files in this checkpoint

```
AI_Please_README.md                                  → this file (read first)
02-map/kconcat-roadmap-v2-execution.md               → roadmap v2: execution spec, decisions D1–D5, plan status
05-background/kconcat-feature-gap-option-b-v2.md     → fixed Option B list, ranked for podcasts
```

`02-map/kconcat-roadmap-v2-execution.md` replaces the original roadmap and the addendum once the owner approves. The original `kconcat-roadmap.md` in the repo stays as it is until then.

---

## 8. Next action

- Owner approves or changes roadmap v2 and its decisions D1–D5 (§1 of the roadmap).
- Then the first execution task is **T1** (harness and baseline), and no code is changed before the baseline is saved.
- T8 needs the owner's corrected transcript of the 30 s clip before its first step.
- Open for discussion: Option B questions (`kconcat-feature-gap-option-b-v2.md` §3).
