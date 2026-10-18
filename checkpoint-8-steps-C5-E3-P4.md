# Checkpoint 8 — C5 + E3 + P4 (18 Oct 2026)

Three build steps closing the order line. C5+E3+P4 were the last *code* items — P7/P6 are verify-only.

## What shipped

| Step | Item | Where | Tests |
|------|------|-------|-------|
| 20 C5 | Caption motion — schema v1 | `model.rs: CaptionMotion { mode, duration, scale_from }` on `TextStyle` · `concat-text: Motion` mirror on `TitleStyle` · `wire::caption_motion` (kebab, off-while-default, fallback-to-default) · `tidy()` clamps duration 0.02..2.0, scale_from 1.0..3.0 · `concat/presets.rs: [motion]` block (both spellings, unknown→none) · `concat-host/titles.rs` copy | 5 integration + 1 structural — RED 3/5 vs todo!(), then GREEN |
| 21 E3 | Word-edge drag — clamped, never refused | `commands/mod.rs: WordEdge { Start, End }` + `MoveCaptionWordEdge { clip_id, word_index, edge, time }` · `commands/clips.rs` arm — clamps at neighbours/window/MIN_WORD, pathological meet not panic, no-op→not-applied, NaN→NotANumber before dispatch, gesture folding via `apply_within` | 6 integration — RED 6/6 vs todo!(), then GREEN |
| 22 P4 | Multi-mic — one take per mic, one track per mic, one undo | `concat-project::takes::place_session(editor, playhead, takes)` — all-or-nothing NaN guard, `AddMedia`→`AddTrack(bottom:true)`→`AddClip(start=playhead+offset)` per take, one undo via gesture name · `concat-host::record::Session` + `start_from(device)` + `input_devices()` (written, not verified — needs cpal) | 5 integration — RED 5/5 vs todo!(), then GREEN |

## Counts

- concat-project **336** (was 325 after C5, 331 after E3 — +5 for P4)
- concat-timing 24, concat-effects 72, concat-text 31
- **Sweep 463/0**, clippy `-D warnings` clean, fmt clean on verifiable crates

## What is NOT verified here (honest)

- C5: `presets.rs` `[motion]` reader + its test, `titles.rs` copy — `concat` and `concat-host` do not compile in this sandbox (slint/FFmpeg/cpal). The progress-driving (RevealMap/typewriter/pop timing), dialog controls and picker previews are owner-machine.
- E3: caption-row waveform draw + hit-test and mouse-drag wiring — slint UI, owner-machine. The document + reload round-trip IS verified.
- P4: `record::Session`/`start_from`/`input_devices` — needs cpal hardware. The voiceover single-mic path is unchanged. Meter UI + session wiring and the five-minute sync recording are owner-machine.

Logs:
- `logs/step20-c5-tests-RED.log` / `step20-c5-tests-GREEN.log`
- `logs/step21-e3-tests-RED.log` / `step21-e3-tests-GREEN.log`
- `logs/step22-p4-tests-RED.log` / `step22-p4-tests-GREEN.log`

Patches (apply on base 9e9478e in order 0001..0010):
- `patch8/0001-C5-caption-motion-*.patch`
- `patch9/0001-E3-word-edge-drag-*.patch`
- `patch10/0001-P4-multi-mic-*.patch`

Next: P7 (denoise exists — listening test) + P6 (B-roll exists — placement test) — checkpoint 9.
