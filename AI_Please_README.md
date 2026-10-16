# kconcat checkpoint — 16 October 2026 (v2)

**Session type: discussion and analysis only.** No application code was changed, and nothing in the repo was modified. This checkpoint and the two files it points to were written to the owner's workspace, not pushed to the repo.

> **File naming:** this file is `AI_Please_README.md` on purpose. An agent session should find and open it first. Use the same name for every future checkpoint.

---

## 1. Where the discussion stands

- **Option B (the feature list vs CapCut Pro):** re-checked and fixed in `05-background/kconcat-feature-gap-option-b-v2.md`. It is ranked for **long-form podcast editing** with **short-clip output**.
- **Option C (caption styles end to end):** partly superseded. Caption motion is confirmed absent, so C5 needs a schema decision before any preset is written.
- **Option D:** open.
- **#15 "Tuning ASR Indonesia":** still awaiting the owner's yes/no for the proposed Track-1 T8 and a WER output on T1.

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
02-map/kconcat-roadmap-addendum-2026-10-16.md        → addendum: Track 5 (long-form podcast), revised order
05-background/kconcat-feature-gap-option-b-v2.md     → fixed Option B list, ranked for podcasts
```

The original roadmap (`02-map/kconcat-roadmap.md`) and feature-gap doc (`05-background/kconcat-feature-gap.md`) stay as they are. Merge the addendum and v2 files in a separate step, after the owner has read them.

---

## 8. Next action

No code work is queued. Discussion continues on the Option B list (owner to answer the questions in `kconcat-feature-gap-option-b-v2.md` §3), then the #15 yes/no.
