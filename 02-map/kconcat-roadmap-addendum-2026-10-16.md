# kconcat roadmap — addendum, 16 Oct 2026

*Addendum to `02-map/kconcat-roadmap.md`. Do not merge into the original yet. Discussion only; no code was changed.*
*Paths in this addendum are written with the real prefix `src/`.*

---

## A. Corrections to the roadmap

1. **Path prefix.** Code references like `crates/concat/...` and `crates/concat-project/...` are relative to `src/`. The real path is `src/crates/...`.
2. **Caption motion is absent** (confirmed 16 Oct 2026). The caption dialog, `src/crates/concat/ui/dialogs/captions.slint`, has no motion, animation, karaoke, or case-transform controls. No text preset sets `highlight_color`, motion, or a case transform. The `pop`/`slide`/`zoom` names in `src/crates/concat-effects/packages/` are video effects and transitions.
3. **Two caption-shaped presets exist:** `concat.bold-caption` and `concat.lower-third`. C2 still needs eight caption presets; these two are a head start, not a zero.
4. **Owner-reported, not yet verified:** caption animation and multi-clip / B-roll already exist in kconcat. Both need a hands-on check in the app before planning. Lower priority from now on.

---

## B. New Track 5 — Long-form podcast

Scope reminder for this track: a 60–120 minute podcast becomes a set of short clips and a clean full-length export, offline.

### P1. Transcript editing for long episodes (promotes E1)
- **Why:** an hour of talk needs bulk cuts by text, not by playhead. This is the single biggest time saver for podcasts.
- **Change:** move E1 up to right after E4. It still depends on T2/T3.
- **Done when:** deleting a 30-second off-topic passage by selecting its text ripples audio, picture and captions together, and one undo restores it.

### P2. Loudness to −14 LUFS at export (split from V2)
- **Why:** podcast platforms and clip platforms both normalise. A consistent level is cheaper than fixing it per episode.
- **Effort:** S
- **Done when:** the exported file measures −14 LUFS integrated within ±1 LU on the corpus episode.

### P3. Speaker labels (diarisation)
- **Why:** two-person shows need per-speaker names or colors. CapCut Pro has this; kconcat does not.
- **Approach:** a local diarisation model, run after transcription, output stored as a speaker tag on each word or cue. Scope decision first (how many speakers, and whether tags are editable).
- **Effort:** L
- **Done when:** on a two-person corpus episode, speaker turns are assigned with an error rate you accept, and a tag can be corrected by hand.

### P4. Screen and webcam recording
- **Why:** podcasts with remote guests are often recorded from screens and webcams, not just microphones.
- **Effort:** S
- **Done when:** a screen-plus-webcam take lands on the timeline as two synced clips.

### P5. Long-form end-to-end check
- **Why:** the roadmap's existing end-to-end check uses a 20-minute recording. Podcasts run longer.
- **Done when:** a 60-minute episode becomes three 60-second vertical clips and one full-length export, offline, start to finish. Count the manual corrections; if the count rises after a track closes, something regressed.

### P6. Verify before planning
- Multi-clip / B-roll (owner-reported existing).
- Caption animation (owner-reported existing; repo check found no caption motion).

---

## C. Revised order (podcast-first)

1. T1 harness → T2 → T3 (word timing)
2. T5 (caption identity)
3. C1 validation on a real episode
4. E4 (Enter-split)
5. P1 = E1 (transcript editing), with E2 Indonesian filler removal on the same machinery
6. P2 loudness + V2 platform presets
7. C2 + C3 caption presets and karaoke wiring
8. V3 auto-ducking
9. V4 batch export queue
10. P3 speaker labels (scope first)
11. V1 auto-reframe 9:16
12. C4 case transform, C5 caption motion (after the motion-schema decision), E3 waveform and word-edge drag
13. P4 screen and webcam recording
14. P6 verify B-roll and animation

---

## D. Still skip

Stabilisation (unless you film handheld), stickers and overlays, templates, cloud and sync, collaboration, AI video generation, retouch, 130-language counts.
