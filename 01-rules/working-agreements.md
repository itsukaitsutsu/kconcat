# Working agreements — kconcat sessions

Paste this at the start of a new session (or point the agent at this file) so
the ground rules survive a context reset.

Owner's context: solo developer in Depok, Indonesia. kconcat is a **personal
fork of `jub0t/Concat`** (AGPL-3.0). Built because CapCut's auto-caption
requires internet, and there is no offline option.

## File layout — keep these separate (decided 16 Oct 2026)

Files are **not** merged. Discussion material must not be handed to a coding
agent: it dilutes attention and pulls the agent toward meta-conversation
instead of the task.

| File | Type | Feed to a coding agent? |
|---|---|---|
| `AGENTS.md` | Rules (lives in the repo) | **Yes** — at the start of every coding session |
| `kconcat-roadmap.md` | Map: what, order, why, done-when | **Yes** — the one file that gives the whole picture |
| `A-word-timing-level1.md` | Manual: how, parameters, traps | **Yes** — only when the task is T2/T3 |
| `B-caption-styles.md` | Manual: how, presets, schema decision | **Yes** — only when the task is C1–C5 |
| `working-agreements.md` | Rules for the assistant | No — it is for the human side |
| `kconcat-audit-2026-10-16.md` | Findings, with evidence | Only §4 as a bug report (`caption-identity-bug.patch`) |
| `kconcat-feature-gap.md`, `kconcat-strategy-advice.md`, `opencut-vs-kconcat-vs-capcut.md`, `caption-sync-fix-plan.md` | Background and discussion | No — reasoning trail, kept for the owner |

Rule of thumb: **one fact lives in one file; other files point at it, they do
not repeat it.** Merging two files is only ever right when the same fact would
otherwise sit in both.

## Rules for the assistant

1. **Language: English.** In chat and in every repo file. Indonesian only when
   explicitly requested for a specific explanation.
2. **This is a personal tool, not a public project.** Do not advise on repo
   hygiene, CI badges, README polish, community, releases, licensing strategy,
   or "ship it" — that advice is explicitly rejected. (Exception: never let the
   owner lose their own work — backups and a reproducible build stay valid.)
3. **Discuss, don't push to build.** The owner implements on their own
   schedule. This thread is for planning, understanding and depth.
4. **No rivalry framing, no hustle pressure.** CapCut is not a competitor to
   beat. Money is not the goal; if a real opportunity appears, the owner will
   decide then. Do not optimise advice for revenue.
5. **Scope: simple tasks.** Close the gap with CapCut on basic editing —
   captions, silence trimming, captions-follow-the-cut, export. Not Premiere /
   Resolve / multi-track post-production territory.
6. **Deepen local AI, never add cloud AI.** Offline is the whole point. Local
   models (Whisper, TTS, matting) yes; cloud generation no.
7. **Verify before claiming.** Say which command or file:line a claim rests on.
   If something was not tested, say so plainly.

## What "closer to CapCut" means here

The job: a 10–40 minute talking-head or screen recording becomes a 45–90 second
vertical short — captioned, silence-trimmed, styled, exported — **on a laptop
with no internet.**

Everything outside that sentence is out of scope.

## Open threads

- `kconcat-feature-gap.md` §6 — the 12-item shortlist (captions first: word
  timing, style presets, one-click apply, auto-reframe, platform export presets)
- `caption-sync-fix-plan.md` — why words drift, and the fix ladder (Level 1:
  snap each word to the real voice onset, ~150 lines, no new models)
- `kconcat-audit-2026-10-16.md` — the caption-identity bug, reproduced with
  3 tests; `caption-identity-bug.patch` applies them
- `AGENTS.md` — repo rules for coding sessions (soft rules; tests are the hard
  ones)

## Known state (16 Oct 2026)

- Repo `itsukaitsutsu/kconcat`, HEAD `1aece2d`, 0 stars, 0 releases
- `cargo test -p concat-project` → 252 pass, 0 fail; clippy and fmt clean
- Everything linking FFmpeg is unverified in-sandbox (no FFmpeg dev libs)
- One real bug found: a caption that keeps none of its words through an
  autotrim silently stops being a caption
