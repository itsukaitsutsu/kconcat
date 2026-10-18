# Checkpoint 6 — roadmap v2 steps 15–17 (C4, V3, V4)

Date: 2026-10-18 · Repo: kconcat @ a639fb6 · Continues checkpoints 1–5.

## Sandbox note: revert #3, and a cleaner recovery than before

Mid-step-16 the sandbox reverted again (.git to base, toolchain destroyed,
some tracked files rolled back while others survived). Recovery was the
proven pattern, improved: the six-patch series in this checkpoints folder
was `git am`-ed back onto base, rebuilding the FULL commit history (no more
single "recreated" commits), then this turn's edits were re-applied by their
exact scripts (every one assertion-guarded) and the C4 suite re-run to prove
the state. One honest find during the audit: the properties.rs handler edit
had never actually applied (a script bug — a replace built but not assigned),
which the mixed-state audit caught and the RED run then proved. Current
history: 7 linear commits on base 9e9478e, HEAD a639fb6.

## Step 15 — C4. Uppercase / text transform

- `TextTransform` (none | uppercase) on `TextStyle`, wire-stable lowercase
  names, `#[serde(default)]` + unknown-name fallback: every older document
  reads as none, nothing migrates, none stays off the wire.
- `display_content()` is the render-time application: stored `content` — the
  transcript — is never rewritten (the plan's exact prescription).
- Tests: `tests/text_transform.rs` 4/4 (RED 4/4 first). Preset layer +
  painter boundary written, NOT VERIFIED (concat/concat-host): presets.rs
  `transform` → `over()`, `kconcat.upper-tight` ships uppercase, titles.rs
  `title_style` draws `display_content()`.
- Recorded decision: transform applies only at the painter boundary —
  word-matching/karaoke keep running on stored casing. UI toggle = owner's
  machine (§9).

## Step 16 — V3. Auto ducking

- `concat_project::duck`: ONE keyframe writer + BOTH routes owner A6 chose —
  Route B (heard words) and Route A (20 ms RMS energy gate, sensitivity dial,
  −50 dBFS floor so silence never ducks everything; gate numbers flagged as
  starting guesses for the owner's real audio, the T3 precedent).
- `Command::DuckMusic` → ordinary Volume keys via `set_key`: editable,
  erasable (ClearClipKeys), one undo. Defaults −12 dB / 500 ms kept and
  editable; clamps; zero-fade steps; edge clamping; NaN refused pre-dispatch.
- Merge rule recorded: a gap ≤ the fade stays ducked (a ramp that cannot
  complete is flapping).
- Tests: `tests/auto_ducking.rs` 6/6 (RED 5/6 — the refusal test passes from
  the start because validation precedes dispatch; recorded, not hidden).
- Owner-machine: decode call site (FFmpeg), Music/Dialogue tags (names not
  decided in code — not invented), Duck Against UI.

## Step 17 — V4. Batch export queue

- `concat_project::queue`: `ExportQueue` with A11's invariants, each tested —
  failure continues; order is the user's (pending items move before or during
  the run; running = runner's, finished = history in place); one encoder at a
  time; late report from a killed runner cannot rewrite its replacement.
- Placement decision recorded: session state in concat-project (no Command,
  no undo, no document version move) because it is the ONLY crate on the
  export path this sandbox can compile — **correction**: concat-export,
  concat-api, concat-server, concat-cli all fail `cargo check` here (earlier
  session memory claimed api/cli/server verifiable; they are not).
- Tests: `tests/export_queue.rs` 5/5 (RED 5/5 first).
- Owner-machine: the runner process itself, the queue panel, the export
  dialog's add-to-queue.

## Verification sweep at this checkpoint

- concat-project: **308** (264 lib + 44 integration) — green.
- concat-timing: 24 — green. concat-effects: 72 — green.
- clippy `-D warnings` + fmt clean on all three; fmt also run on concat and
  concat-host (written-not-verified crates).
- Full log: logs/checkpoint6-sweep.log.

## Series so far

17 of 24 roadmap-order steps done: T1–T7, C1val, E4, E1, E2, V2-loudness,
C2, C3 (cps 1–5) + C4, V3, V4 (cp6). Next: steps 18–19 = P3, V1 →
checkpoint 7. After the order line: the unscheduled decisions (E5, E6, A4-7,
V2 platform presets, D2, D3, K1–K5, X1, X2, UI-1, T8, P5) await the owner's
keep/drop call.
