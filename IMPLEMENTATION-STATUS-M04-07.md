# Public-source implementation package — milestones 04–07

**Base:** public `itsukaitsutsu/kconcat` clone at `ede7e59` in `/home/user/kconcat`.
**Scope:** public clone only. No files in the user's Windows checkout were modified or verified.

## Included code changes

- **05 — text motion:** added inspector controls for None, Fade In, Pop In, Bounce, Slide Up, Typewriter, Karaoke, and legacy Word Reveal. Fade/Pop/Bounce/Slide Up use sampled still slices; Typewriter reveals Unicode-safe text with language-aware word pauses; Karaoke overlays use stored caption word timings or evenly timed word units for untimed titles. These render through the title flattener used by monitor and export. Duration, Typewriter stagger, and Pop scale controls are wired to persisted motion fields; existing serialized values and legacy shrink-to-one Pop settings are retained. Motion and preset tests were added, but are unrun in this sandbox.
- **06 — transcript controls:** the per-word delete/timing list was removed at the owner's request as unhelpful. The remaining inspector contribution is the opt-in strong/weak filler UI wired to the existing undoable filler-removal command. Existing delete-word and word-edge commands remain in the project command layer; this inspector no longer exposes the word list.
- **07 — lane/link foundations:** added `TrackKind` (Video, Audio, LegacyMixed); old documents with no lane kind load as permissive legacy-mixed lanes and omit that value again on save. Added typed video/audio lane creation in the timeline menu, lane-kind markers, and compatibility checks for clip creation/moves. Caption-specific newly created lanes are video lanes; detached audio is routed to audio-compatible lanes. Clicking a linked clip now selects its link mates; existing Link/Unlink context-menu commands remain in place.
- **04 — acceptance remains open:** no real media excerpt or corrected reference transcript was available, so timing/ASR quality and media acceptance were not performed.

## CapCut cache import (additional requested scope)

- The Text inspector now has **Import from CapCut draft…**. With one text clip selected, choose a local CapCut `draft_content.json`; Kconcat follows the draft's cached animation paths (or the standard nearby `Cache/effect/<resource-id>/<hash>` layout) and reads, but does not copy, cache files.
- The first supported `studioAnim.lsanim` `RadianceGlow` exposure curve is translated into Kconcat's native `concat.bloom-pulse` amount keyframes and applied as an undoable effect to the selected title. The current public sample's **Subtle Water** cache is the confirmed supported resource. Its glow color, threshold, masks and CapCut shader behavior are not represented, so this is a native approximation, not a pixel-identical CapCut render.
- CapCut Lua, shaders, binary prefabs, and image-sequence packages are never executed or loaded. Entries such as Star Cluster / Canceling Out are not yet rendered by this importer; a draft with no compatible curve produces an explicit unsupported-format message. The implementation adds no cached assets to the source ZIP.
- Parser tests for local cache discovery, curve translation, and unsupported packages were added, but could not be run here.

## Verification limits

- Passed static checks in this sandbox: `git diff --check` and parsing `src/crates/concat/locales/en.json` with Python's JSON parser.
- Rust build, Slint compilation, tests, app launch, preview/export agreement, and Windows `cargo dev` runtime checks were **not run**. `cargo`, `rustfmt`, and the Slint compiler are unavailable in this sandbox. The added Rust tests are unrun.
- This ZIP is a source patch/snapshot for local testing, not evidence that milestones 04–07 are accepted or complete. Please test in the Windows checkout with `cargo dev` and report any compile/UI issues.
