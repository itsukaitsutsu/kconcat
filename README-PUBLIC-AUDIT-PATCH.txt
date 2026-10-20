KCONCAT — MILESTONES 04–09 PUBLIC-SOURCE AUDIT + TARGETED PATCH

Basis: public GitHub checkout at ede7e5906fa43cb79de0d8488999b5af0b45a7ef.
This is NOT a copy of your Windows working tree and does NOT include your uncommitted local changes.

CONTENTS
- src/crates/concat/src/panes/sections.rs
  Adds the missing final `false` argument to both caption_clip_on_track calls.
- src/crates/concat-project/src/platform_export.rs
  Adds doc comments for the six public API items named by the missing-docs warnings.
- TEMP-kconcat-remaining-plan-checklist.md
  Updated workflow status and audit notes; milestones 04–09 remain open.
- MILESTONES-04-09-PUBLIC-SOURCE-AUDIT.md
  Static source findings, gaps, and verification limits.

SAFE USE
1. Preserve your local changes first. Do not replace your entire repository with this archive.
2. Compare the two Rust files with your current versions before copying; the archive is from public main and may not contain your local edits.
3. The checklist and audit report are documents, not source patches.

AUDIT LIMITS
- This sandbox has no Cargo executable, so no `cargo check` or test run was possible.
- The public-source review found milestones 04–09 incomplete as end-to-end work packages. In particular, real-media timing/ASR acceptance and measured listening/export acceptance cannot be supplied by this source-only archive.
- The archive is an audit/targeted-fixes package, NOT a claim that all features in milestones 04–09 are implemented or verified.
