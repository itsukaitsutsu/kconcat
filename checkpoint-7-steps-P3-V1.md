# Checkpoint 7 — roadmap v2 steps 18–19 (P3, V1) — FINAL of this 5-step run

Date: 2026-10-18 · Repo: kconcat @ d0721f8 · Continues checkpoints 1–6.
Completes the owner-ordered "next 5 steps": C4 → V3 → V4 (checkpoint 6) →
P3 → V1 (this checkpoint). 19 of the 24 order-line steps are now done.

## Step 18 — P3. Speaker labels (manual route)

- What is DECIDED is built and verified in concat-project; what is OPEN
  stays open, recorded not guessed:
  - `Speaker { id, name, color }` registry on Timeline; `Track.speaker`
    links one caption lane per speaker (D7); `Clip.speaker` labels the
    caption (and any audio clip). All serde-defaulted: older documents
    read as "no speakers", DOCUMENT_VERSION unmoved, and an empty
    registry changes nothing in the editing flow — the owner's opt-in
    rule enforced by default.
  - Recorded deviation (deliberate): the label rides on the caption, not
    on `captionmap::Word` — every word inherits its speaker, lossless for
    the plan's own main case (one utterance per caption; overlap is
    manual by decision), and a Word field would break struct literals
    across concat-speech and every test harness — crates that cannot
    compile here. Word-granular moves compose: E4-split, then assign.
  - Five commands (new `commands/speakers.rs`): AddSpeaker (name never
    blank), AssignCaptionToSpeaker (lane made on first use — no empty
    slots — on top where caption lanes live; heard timings and
    transcript untouched), RenameSpeaker + SetSpeakerColor (tags
    editable), RemoveSpeaker (refused while captions carry the label;
    the empty lane goes with an unused speaker).
  - One command, one undo: undoing an assignment takes the made lane
    with it; the speaker (an earlier command) stays.
- Tests: RED 6/6 → GREEN 6/6 (`tests/speaker_labels.rs`), including the
  wire test (no `speakers` key while empty; whole-project round trip).
- NOT built (open/owner-machine): detection engine (sherpa-onnx vs
  speakrs — waiting on your 2–3 min ground truth), splitting captions at
  speaker changes (arrives with detection), overlap (manual by
  decision), the multi-stream import question (you check the app first),
  D7 link groups (E5 dependency), all UI (§9).

## Step 19 — V1. Auto-reframe 16:9 → 9:16 (mathematics)

- The plan's exact chain, as pure verified mathematics in
  `concat_project::reframe`:
  - per-frame mask centroid: `mask_centroid` — an 8-bit grid (what
    concat-vision's RVM store keeps) to one normalised point; pixel
    centres; None when nobody is in the mask or the grid mismatches.
  - smoothing: `pan_track` — clamp to the visible window → one-euro
    filter (the plan's first choice) → clamp again (a low-pass can
    overshoot past a clamp its input respected). Parameters 0.5 Hz /
    beta 10 / 1.0 Hz are starting guesses with the T3 flag: tune on
    your real footage.
  - crop keyframes: `reframe_plan` — constant fill scale (1/visible,
    clamped by `ranges`) + one OffsetX key per centre, in the model's
    own documented units; same-instant keys merge by `set_key`'s rule.
- `Command::AutoReframe { clip_id, target_width, target_height, points }`
  — the caller brings the points (masks live beside the decoder/model);
  five one-sentence refusals; NaN refused pre-dispatch; one undo
  restores fit + keys together.
- Tests: RED 6/6 → GREEN 6/6 (`tests/auto_reframe.rs`). The prescribed
  jitter threshold was set at the writing, as the plan requires: 2e-3
  interior second difference against a ~0.0125 raw step; ramp lag
  < 0.03; ±0.01 noise held within 0.008; edge-pinned subject never
  leaves the frame; command writes the 3.160494 fill scale for 16:9 →
  9:16 with bounded keys; undo restores.
- NOT built (owner-machine): the RVM inference pipeline (`concat-vision`
  `infer`/ort cannot compile here), the host call site producing
  per-frame points, the renderer-side sign check of `offset_x` (the
  convention is written in the module docs for exactly that), and the
  done-when run on a real talking-head clip.

## Verification sweep at this checkpoint

- concat-project: **320** (264 lib + 56 integration) — green.
- concat-timing: 24 — green. concat-effects: 72 — green. **Total 416.**
- clippy `-D warnings` + fmt clean on all three (log:
  logs/checkpoint7-sweep.log).
- Sandbox note: a THIRD mid-run revert happened (during step 16);
  recovery was `git am` of the six-patch series + assertion-guarded
  re-application of the turn's edits + re-running C4's suite green. The
  audit also caught a script bug of mine (a properties.rs replace built
  but never assigned) — fixed and proven by the RED→GREEN runs.

## The order line: 19 of 24 done

T1–T7, C1val, E4, E1, E2, V2-loudness, C2, C3 (cps 1–5) · C4, V3, V4
(cp 6) · P3, V1 (cp 7). Remaining: **C5 (caption motion) → E3 (waveform +
word-edge drag) → P4 (multi-mic) → P7 (denoise verify) → P6 (B-roll
verify)** — 5 steps, of which P7 and P6 are runtime checks in the real
app (yours), so the last building steps are C5, E3, P4.
