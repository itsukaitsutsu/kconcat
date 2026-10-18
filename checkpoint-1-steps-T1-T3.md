# Checkpoint 1 — roadmap v2 steps 1–3 (T1, T2, T3)

Date: 2026-10-18 · Repo: kconcat @ 38e65af (from 9e9478e) · Session: 9-step execution run, checkpoint every 3 steps.

## What was executed (fixed §11 order)

**Step 1 — T1. Corpus and harness.** New workspace crate `src/crates/concat-timing/` (no
dependencies, per D4-B): `energy` (moved `rms`/`linear_from_db` + 20 ms/5 ms dB framing),
`words` (Word + whisper/snapped marker), `metrics` (Tier 1: onset plausibility, monotonicity
violations, outliers; Δ vs baseline; normalized WER with sub/del/ins), `examples/metric_runner.rs`
(Tier 1 table from cache + raw audio, `--ref` SRT for WER, `--save` run JSON, later `--snap`).
Tests ran red first (11 failures), then 11/11 green. `concat-speech/examples/make_cache.rs`
written (D5-B owner-machine half: Whisper once → `whisper-cache.json` + `audio-16k.f32`).
`concat-media/src/silence.rs` rewired to import the moved helpers.

**Step 2 — T2. Level 1 word-onset snapping.** `snap::snap_words` (manual §2 steps 1–5,
`Params` = the manual's Start column, untouched), collision rule exactly as the roadmap fixes it
(revert to Whisper, never clamp; monotonic invariant re-checked, ties counted in `SnapReport.tied`,
never silent), clock rule enforced by tests (window clock before speed division; never a
VAD-compacted buffer). All 7 prescribed tests red first, then green (18/18). `metric_runner --snap`
+ Δ output. `concat-speech`: `snap_word_starts` (HeardWord mapping) called at all three production
sites after `refine_segments_with_vad`, before any speed division; the VAD pass itself untouched.

**Step 3 — T3. Level 1.5.** `signals`: `Signals{energy, zcr, flux}` on the shared frame grid;
ZCR frames; 4-band Goertzel spectral flux (`BANDS` 300–800/800–1500/1500–3000/3000–6000 Hz, the
manual's starting guess, now recorded in manual §3 as T3 requires); `Wideband` starting-guess
knobs; snapper search and Tier 1 `onset_plausible` both consult all three signals (one definition
of "onset"). Prescribed tests red first (4 RED), then green (24/24). **Measured bug + fix:** raw
flux spikes at burst offsets (Dirichlet sidelobes of the signal's own step edge; Hann measured and
rejected — the step is in the signal, not the window), which snapped synthetic words 130 ms early
into the previous burst's tail; fixed by the energy gate (`e[i] ≥ e[i−1]` for flux rises — no new
tunable number), with regression test `a_burst_offset_is_not_a_flux_rise`. All recorded in manual §3.

## Evidence in this zip

- `patch/0002-*.patch` — the full T1–T3 diff (text; `patch/0001-*.patch` is the earlier
  background-material commit, binary files omitted by git default).
- `logs/step{1,2,3}-*-RED.log` — test-first failures, per step.
- `logs/step{2,3}-*-GREEN.log`, `logs/step1-t1-runner-baseline.log` — green runs.
- `logs/step{2,3}-t*-runner-*-synthetic.log` — metric_runner output.
- `runs/` + `fixtures/` — the SYNTHETIC baseline and snapped runs, and the fixture they were
  measured on (clearly labelled: the real corpus needs T8 + the owner's machine).

## Verification status (plainly, per AGENTS.md)

- **Verified in this sandbox:** `cargo test -p concat-timing` 24/24; `cargo clippy -p concat-timing
  --all-targets -- -D warnings` clean; `cargo fmt --check` clean; metric_runner runs on the
  synthetic fixture (baseline and `--snap`).
- **NOT verified here (no FFmpeg / whisper.cpp in the sandbox — written, compile-unchecked):**
  `concat-media` (the rewire + its silence tests that prove the moved helpers), `concat-speech`
  (`make_cache` example, `snap_word_starts` wiring). Nothing here may be called "pass".
- **Open until the owner's machine / T8:** real `baseline.json` (run `make_cache`, then
  `metric_runner`, save as `06-corpus/runs/baseline.json`); T2/T3 acceptance bands on the 30 s
  reference clip; T3's "mid-phrase median improves, else record no gain".

## Synthetic-corpus numbers (fixture, not real speech)

| run | onset plausibility | violations | outliers | WER | Δ |
|---|---|---|---|---|---|
| baseline (no snap, T1 signals) | 63.6% | 2 | 2 | 9.1% (1 sub, planted) | — |
| baseline (no snap, T3 signals) | 90.9% | 2 | 2 | 9.1% | — |
| snapped (T3) | 90.9% | 1 | 0 | 9.1% | 1/11 moved, +115 ms max |

The residual violation is a 20 ms overlap at a segment head where the reversed fixture pair is
clipped to the cue and the previous word's end cannot retract below its own minimum duration —
the manual's repair order produces it; whether it survives on real speech is T8's question.
Fixture words sit on their rise frames by construction, so `snapped 0` there is correct.

## Next (auto-continuing, per the run instruction)

Step 4 — T4 (script-aware tokenisation), then T5, T6; checkpoint 2 after step 6.
