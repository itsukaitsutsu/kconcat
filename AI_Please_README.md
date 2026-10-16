# kconcat checkpoint — 16 October 2026

Session type: **discussion and analysis only.** No application code was changed.
One helper script was written (`06-corpus/srt_report.py`); nothing in the repo
was touched.

> **File naming:** this file is `AI_Please_README.md` on purpose — an agent
> session should find and open it first. Use the same name for every future
> checkpoint.

---

## Where the discussion stands (frozen 16 Oct 2026)

- **Option B — the feature list** (what kconcat has vs CapCut, item by item):
  **recorded in full** — `05-background/kconcat-feature-gap.md`, section "Option B
  — your 15-item list, verified item by item". Each row carries its evidence and
  its plan home; the ordered build list is that document's §6.
- **Option C — caption styles end to end** (offered when this checkpoint was
  frozen; not started): map exactly what stands between what the engine already
  does — a per-word reveal the shader reads from a baked map
  (`concat-core/src/shader.rs:222–234`, bound `:60–64`; baked per title in
  `concat-host/src/titles.rs:795`) with the karaoke highlight overlay built on
  top (`concat-host/src/titles.rs:390–402`), plus 32 TOML presets in
  `crates/concat/text-presets/` (language-tagged, user-loadable,
  `crates/concat/src/presets.rs:12–13`) — and CapCut-style one-click caption
  styles.
- **Option D — something else entirely.** Open.
- **#15 "Tuning ASR Indonesia"** — answered with five sub-points (15a WER first ·
  15b model bake-off · 15c prompt biasing already ships · 15d VAD/params · 15e self
  fine-tune = optional, last). Its plan home — a Track-1 **T8**, plus a **WER
  output on T1** — is proposed and still **waiting for the owner's yes/no**.

---

## Repo state at this checkpoint

| | |
|---|---|
| Repository | `itsukaitsutsu/kconcat` |
| HEAD | `1aece2d` ("plan to get closer", 16 Oct 2026 07:11 +0700) |
| Verified in sandbox | `cargo test -p concat-project` → **252 passed, 0 failed** |
| Verified | `cargo clippy -p concat-project --all-targets -- -D warnings` → clean |
| Verified | `cargo fmt -p concat-project -- --check` → clean |
| **Not verified** | every crate linking FFmpeg (`concat-media`, `concat-render`, `concat-export`, `concat-host`, the `concat` GUI) — no FFmpeg dev libraries in the sandbox |

---

## What was decided this session

1. **Language: English** in chat and in every repo file. Indonesian only when
   explicitly requested for one explanation.
2. **Personal tool, not a public project.** No repo hygiene, CI, README, release
   or licensing strategy advice. Working on your own schedule; this thread is for
   discussion and depth.
3. **Scope: simple tasks only** — captions, silence trim, captions-follow-the-cut,
   export. Not Premiere/Resolve territory.
4. **Local AI only.** Offline is the entire point; cloud AI is out.
5. **Files stay separate, not merged.** Discussion material must not be handed to
   a coding agent — it dilutes attention. See `01-rules/working-agreements.md`
   for the "which file for what" table.
6. **One fact lives in one file**; other files link, they do not repeat.
7. **The test corpus is the existing 5-minute video**, not a new recording —
   cached once so tuning costs seconds, measured with label-free metrics first.

---

## What was found

**One confirmed bug**, reproduced with three tests
(`04-findings/caption-identity-bug.patch`):

A caption created from an imported `.srt`/`.vtt` carries no `caption_of` link —
its identity rests on word timings alone. When a trim keeps the cue but removes
every word inside it, `commands/clips.rs:883` clears `words` to `None`, and the
clip silently stops being a caption.

Consequences, in order of severity:

1. **Export, worst case** — `is_caption_clip` (`export.rs:835`) no longer
   recognises it, so with **"Burn captions" switched OFF the caption is still
   burned into the video** (`export.rs:643`). The opposite of the setting.
2. **Placement** — it loses the caption↔caption overlap allowance and starts
   blocking caption lanes (`clips.rs:1282`).
3. **Sections panel** — measured **1 caption section before the trim → 0 after**
   (`sectionlist.rs:189`), so a later "Remove selected" no longer knows a
   caption sits there.
4. **Bulk caption edits** skip it (`studio.rs:3532`).

**Root cause:** identity is *derived*, not stored, and the same predicate is
duplicated in **six places across three crates**.

**Verified as working** (tested, not assumed): a caption entirely inside a
removed region is deleted cleanly with no orphan, and a cut through the middle of
a caption splits it into two pieces that each keep their own words and text. A
ghost also still follows a second trim correctly — it does not drift.

---

**Second pass, later the same day.** Five of my own "missing feature" claims
turned out to be wrong — word chunking (four Auto-caption layout modes + a
Re-split button), the preset library, per-track "apply to all", split/merge at
the playhead and the karaoke renderer all ship. Evidence:
`05-background/kconcat-feature-gap.md` §Corrections. In exchange, one genuinely
missing thing and one suspected defect came out of the same pass:

- **Missing:** Enter does not split a caption at the caret into a new cue
  (CapCut and VTS do). Build plan, with what already exists to build on:
  `02-map/kconcat-roadmap.md` §E4.
- **Suspected defect — reasoned from the code, not tested:** an inline text edit
  in Sections writes `style.content` and leaves `clip.words` stale
  (`concat/src/panes/sections.rs:680`), so `locate_words` can no longer find
  those words and the karaoke highlight drops them.
- **Third pass, same day — every remaining ❌/🟡 in the feature list re-checked
  against `1aece2d`.** Two more rows were wrong: the timeline **does** draw
  waveforms (`ui/primitives/wave.slint`), and Auto-caption **does** ship
  Whisper's *translate to English*. Three rows were made exact (microphone-only
  recording, `loudnorm` before transcription only, API jobs ≠ export queue).
  Verified genuinely absent: stabilisation, auto-ducking, vocal isolation, beat
  sync, filler removal, diarisation, auto-reframe, platform export presets.

---

## What is still open

- **Tier 1 metric runner** — the one roadmap item usable without touching app
  code, and the prerequisite for measuring word-timing work at all.
  `06-corpus/srt_report.py` is a start but needs the audio and word timings.
- **`whisper-cache.json` + `audio-16k.f32`** — not yet produced for the corpus.
- **No baseline numbers yet**, because nothing has been measured.
- **Motion schema decision** (C5 blocker) — where caption motion lives: preset
  `.toml` block, inspector-only, or an effect package. Must be decided before
  preset files are written, because it changes their shape.
- **Caption pipeline map** — transcription → `captionmap` → placement → reveal →
  render → export, with a `file:line` per hop.
- **Decision on #15** — whether to add the proposed Track-1 "T8" and the WER
  output on T1 (see the option list near the top).

---

## How to use this checkpoint in a coding session

Feed the agent **only** what the task needs:

| Task | Hand over |
|---|---|
| Any coding session | `01-rules/AGENTS.md` |
| Whole picture | `02-map/kconcat-roadmap.md` |
| Word timing (T2/T3) | `01-rules/AGENTS.md` + `03-manuals/A-word-timing-level1.md` |
| Caption display (C1–C5) | `01-rules/AGENTS.md` + `03-manuals/B-caption-styles.md` |
| The identity bug (T5) | `01-rules/AGENTS.md` + `04-findings/caption-identity-bug.patch` |
| Human decisions, tone, file layout | `01-rules/working-agreements.md` — keep it, do not paste it at an agent |
| Continuing the *discussion* (no code) | the whole checkpoint — start with this file, then `05-background/kconcat-feature-gap.md` §Option B |

**Do not** paste `05-background/` into a coding session. It is the reasoning
trail and belongs to the owner.

---

## Next action

Start the execution session with the Tier 1 metric runner: decode the corpus
audio once, cache the Whisper output once, then print onset plausibility,
monotonicity violations, outliers and Δ versus baseline. Everything in Track 1 is
unmeasurable until that exists.

For a **discussion** session (no code), resume from the option list near the top
of this file — Option C/D, or decide #15 → T8.

---

## Folder map

```
AI_Please_README.md   → read this first
01-rules/        AGENTS.md                    → goes in the repo root
                 working-agreements.md        → for the owner, not for agents
02-map/          kconcat-roadmap.md           → what, order, why, done-when
03-manuals/      A-word-timing-level1.md      → how: word timing
                 B-caption-styles.md          → how: caption display
04-findings/     kconcat-audit-2026-10-16.md  → evidence, with file:line
                 caption-identity-bug.patch   → 3 tests, git apply
05-background/   reasoning trail: sync analysis, feature gap (the Option-B list), strategy,
                 the three-way comparison with OpenCut and CapCut
06-corpus/       user-video.srt               → the fixed test corpus
                 srt_report.py                → cue-level structural report
```
