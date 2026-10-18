# kconcat-ui-preview.html — what it is and how to read it

*Written 17 Oct 2026. Discussion and design material only; no application code was changed.
The preview itself is `05-background/kconcat-ui-preview.html` — one self-contained HTML file, no
external fonts, scripts or images (same rule as `motion-drawer-preview.html` and
`caption-presets-preview.html`).*

---

## 1. What it is

A whole-app UI/UX preview of kconcat: **every feature that exists today and every feature on the
roadmap**, on one interactive screen. It is the draft of the UI sections that roadmap v2 §9 says
do not exist yet:

> "UI for E1–E6, V1–V5, P3, D2, K1–K5 is not specified yet. Each item gets a UI section (placement,
> labels, states) and is checked in the running app before it is built. Design text in the .md is a
> draft until then."

This file is that draft, for every one of those items, plus the built surfaces they sit in.

**It is not an instruction to a coding session.** Per the standing rule, discussion material stays
in `05-background/` and out of coding sessions. A session that needs a task reads `AGENTS.md`,
`02-map/kconcat-roadmap-v2-execution.md`, and the one manual the task names — not this.

## 2. How to read it

- **Open the file in a browser.** Two states, switched in the title bar: **Today (main @ 1aece2d)**
  and **After roadmap v2**. The timeline re-draws between them: untyped lanes and Detach/Reattach
  become typed V/A lanes (D6), one caption lane per speaker (D7), V1-on-top lane numbers (D8),
  link groups (E5) and beat marks (K1).
- **Every control carries a status.** Built · Partial · Planned · Undecided · Verify only · Bug ·
  Out of scope. Controls that do not exist in the tree wear a **dashed outline**; clicking one
  explains what the roadmap says about it, with the task ID.
- **Feature map** (button top right, or `M`): all 80 entries grouped by track, filterable by status,
  each row carrying its task ID, its status, its evidence path under `src/`, and a jump to the
  screen that demonstrates it.
- **Numbering crosswalk** (in the map drawer): Option B's tier numbers (A1–A12, B1–B2, C1–C4,
  D1–D5) against roadmap v2's task IDs, including the collisions — Option B C1 is roadmap C4,
  Option B C2 is roadmap C5, Option B D1 is roadmap P7, and the motion decision called "D3" is not
  decision D2/D3 of the roadmap's §1 table.
- **Space** plays the demo cue: karaoke highlight on the built renderer's path, caption motion
  (pop / word-reveal / typewriter) on the approved schema v1, and the active Burst group exactly as
  C1 chunks it (3 words / 18 chars / 2.5 s, 0.35 s pause).

## 3. Where every number in it comes from

Nothing in the preview is invented. Each surface was read out of the tree before it was drawn:

| Surface in the preview | Source |
|---|---|
| Colours, surface ladder, kind pairs, playhead | `src/crates/concat/ui/theme/dark.slint` (verbatim tokens) |
| Panes, tabs, tray tools, dialog fields | `src/crates/concat/ui/*.slint` (67 files, all referenced) |
| 32 text presets, 8 caption presets + values | `src/crates/concat/text-presets/`, `03-manuals/B-caption-styles.md` §4 |
| 123 effect packages by kind and category | `src/crates/concat-effects/packages/*/effect.toml` (all 123 named) |
| Statuses, task IDs, decisions, acceptance numbers | `02-map/kconcat-roadmap-v2-execution.md`, `AGENTS.md` |
| T1 harness, corpus gaps, holdout rule | roadmap §2 T1, `06-corpus/` (only `srt_report.py` + `user-video.srt` exist) |
| T5 failure sequence | `04-findings/caption-identity-bug.patch` |
| UI-1 audit table | `04-findings/ui-1-panel-scroll-investigation.md`, roadmap §10c |
| D3 translation UI states and tests | `02-map/guide-d3-translation-languages.md` §3 |
| Crate list and sandbox-verifiability | each crate's `Cargo.toml` description + roadmap §0 step 6 |
| Skip list and out-of-scope rows | roadmap §7, `05-background/kconcat-feature-gap-option-b-v2.md` |

Coverage was checked mechanically, not by eye: 39/39 roadmap task IDs, 8/8 decision IDs, 14/14
Option B tier A/B rows, 123/123 effect packages, 32/32 text presets, 67/67 `.slint` files, 15/15
crates and 23/23 working-set documents all appear in the file; and every one of the 73 feature rows
that names a screen resolves to a screen that exists in it. The interactive behaviour was exercised
headlessly (64 assertions: every dialog and jump screen opens, both states re-draw the timeline,
burst group selection, filters, Shift+L, Esc) with zero console errors.
`preview-shots/` holds 12 rendered screenshots as evidence of what the HTML actually draws.

## 4. What it deliberately does not do

- It does not touch `src/`. No code, no `.slint`, no `Cargo.toml`.
- It does not claim anything runs. "Planned" means plan-fixed in the .md with no code, exactly as
  the roadmap's code-status column says; "Verify only" means the owner must look at the real app.
- It does not bundle the two earlier mocks. `motion-drawer-preview.html` and
  `caption-presets-preview.html` stay where they are; the Motion drawer screen links to the former
  by name.
- It is in English (repo rule), with the one Indonesian string the owner specified: the E1 checkbox
  "Potong video saat kata dihapus".

## 5. Open things the preview surfaces, not settles

1. **P3 engine** — speakrs vs sherpa-onnx: undecided; both cards are shown with their model sizes.
2. **P3b** — separate audio files vs one multi-stream file: check in the app first
   (`Clip.audio_stream` suggests it may already work).
3. **D3 model format** — Opus-MT vs NLLB INT8; support one format first, reject a wrong file.
4. **T8** — blocked on the owner's corrected transcript of the 30 s clip.
5. **UI-1** — H2 (`PaneScroll` overflow reads as 0) is unconfirmed; the owner runs the protocol in
   the findings file before any fix. The preview shows the audit table, not a fix.
