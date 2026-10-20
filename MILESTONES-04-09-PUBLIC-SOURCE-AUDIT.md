# Milestones 04–09 — public-source audit and targeted fixes

**Audit basis:** public GitHub checkout `itsukaitsutsu/kconcat`, base commit `ede7e5906fa43cb79de0d8488999b5af0b45a7ef` (main snapshot). This is **not** the owner's Windows working tree and does not include their uncommitted changes. The owner selected a public-source-only audit.

**Scope:** static review against milestones 04–09 in `TEMP-kconcat-remaining-plan-checklist.md`. This report does not claim runtime, media-quality, export-quality, or Windows build acceptance.

## Summary

| Milestone | Public-source status | Why it remains open |
|---|---|---|
| 04 — Timing and Indonesian ASR acceptance | **Not accepted** | The checklist records milestone 03 was cancelled before extraction/inference. Real audio, a corrected same-range reference, timing-error measurements, WER, and holdout evidence are absent. Infrastructure alone cannot pass this gate. |
| 05 — Caption appearance and panel access | **Partial foundations; user-facing motion not complete** | Motion values/schema and preset parsing exist, and the host copies values into a text-painter mirror. I found no user-facing motion controls or complete mode-driven timed rendering path. The owner separately reports no animation controls/effects in their `cargo dev` app. |
| 06 — Transcript-editing controls | **Partial** | Command/model support exists for deleting a heard word, moving a word edge, removing filler words, and splitting at a timed word. That is not proof all controls are exposed or the real editing/undo/sync/export workflow passes. The filler and word-edge controls, plus the cut-video choice, still need UI/runtime verification. |
| 07 — Typed lanes and linked editing | **Partial; typed lanes not implemented** | Link/Unlink commands and context-menu actions exist. However, `Track` and the timeline UI explicitly describe tracks as untyped; there is no V/A lane type or typed-lane migration. Detach/Reattach actions also remain. |
| 08 — Upload-ready platform exports | **Partial** | Platform preset definitions and an export-sheet selector exist. The selector currently sets the bitrate default; export size/rate remain project-derived. Preset frame/FPS and loudness/true-peak specifications are not wired through to an actual platform-ready render/mix. No measured export acceptance was possible here. |
| 09 — Auto ducking | **Partial** | The project crate has duck-region/keyframe logic and a `DuckMusic` command. The current UI action supplies caption-word regions and default depth/fade; I found no connected audio-analysis route or surfaced controls. Listening to a real music/dialogue mix and export verification remain unperformed. |

**Bottom line:** the public source contains useful lower-level foundations for 05–09, but the checklist's end-to-end criteria for 04–09 are not satisfied. Do not interpret comments, schemas, command variants, or unit tests as proof a feature is available in the running application.

## Evidence checked

- **04:** `TEMP-kconcat-remaining-plan-checklist.md` says the real pilot was cancelled before audio extraction/inference. The checklist's acceptance requires corrected same-window references and measured real-audio results.
- **05:** `concat-project/src/model.rs` defines `CaptionMotion`; `concat/src/presets.rs` parses preset motion; `concat-host/src/titles.rs` copies the fields into `concat-text`. A repository-wide search found no actual UI motion picker and no mode-specific render behavior in `concat-text` beyond type/schema tests. This aligns with the owner's runtime report.
- **06:** `concat-project/src/commands/mod.rs` contains `DeleteCaptionWord`, `MoveCaptionWordEdge`, `RemoveFillerWords`, and `SplitCaptionAtWord`; command logic exists under `commands/clips.rs`. The public-source audit does not establish the checklist's full exposed control set and runtime acceptance.
- **07:** `concat-project/src/model.rs` documents `Track` as deliberately untyped. `concat/ui/timeline/model.slint` and `timeline-pane.slint` also declare untyped tracks. Link/unlink support is present in `commands/links.rs` and `studio.rs`.
- **08:** `concat-project/src/platform_export.rs` defines platform specs. `concat/src/panes/export.rs` handles `PlatformChanged` by updating the bitrate default; the resulting request uses the export pane's project-derived size/rate. The platform normalization constants are not sufficient evidence of an applied audio normalization pipeline.
- **09:** `concat-project/src/duck.rs` contains `regions_from_samples` and `duck_keys`; `studio.rs`'s `duck` context action gathers caption words and applies default parameters. Static search found no UI call into `regions_from_samples`.

## Changes in this package

1. Added the missing `look_preset_decided: bool` argument (`false`) to both `caption_clip_on_track` calls in `src/crates/concat/src/panes/sections.rs`. This fixes the reported E0061 source error in this public snapshot.
2. Added documentation comments to the six public items that produced the `missing-docs` warnings in `src/crates/concat-project/src/platform_export.rs`.
3. Updated `TEMP-kconcat-remaining-plan-checklist.md` so 04–09 stay open pending actual source/runtime evidence and step 05 explicitly records the missing animation controls reported by the owner.
4. Added this audit report.

## Verification limits

- `git diff --check` passed for the edited files.
- A Rust build/test **could not be run here**: the sandbox has no `cargo` executable (`cargo: command not found`). The package is therefore a source patch, not a verified Windows build.
- No local Windows checkout, private media, actual export, or user's `cargo dev` process was available to inspect. Public-source findings may differ from the user's uncommitted local code.
- Milestone 04 and the measured/real-media acceptance parts of 08–09 require owner-side inputs and tests; they cannot be honestly completed by changing code in this public clone alone.

## Recommended verification on the owner's Windows checkout

1. Compare/overlay only after preserving current local changes; this archive is based on the public commit above.
2. Run the targeted `cargo check --locked -p concat` in the configured Windows build terminal.
3. Inspect each actual local diff and run the relevant `cargo dev` UI flow before marking any item complete.
4. Keep real-media acceptance for 04 and measured export/listening acceptance for 08–09 open until their evidence exists.
