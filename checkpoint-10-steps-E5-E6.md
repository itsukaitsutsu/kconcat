# Checkpoint 10 — E5 Link / Unlink + E6 whole-lane rule (18 Oct 2026)

**Base:** `main` @ `b5d095e` ("pleasefix").
**Scope of this checkpoint:** the last two roadmap items that were still
"not started" and provable in the sandbox. Nothing else was touched.

## What was built

| Item | Where | Status |
|---|---|---|
| E5 Link / Unlink audio and video | `src/crates/concat-project/src/{model.rs, doc.rs, commands/mod.rs, commands/links.rs}` | mechanism DONE, 10 tests |
| E6 select one whole lane | `src/crates/concat-project/src/model.rs` (`Timeline::clips_on_track`) | lane rule DONE, 1 test |

### E5 in one paragraph
`Clip::link_group: Option<String>` is the stored group (serde default,
skipped when none, so a project with nothing linked writes exactly what it
wrote before — `DOCUMENT_VERSION` does not move). `Command::LinkClips` and
`Command::UnlinkClips` set and clear it. Every other editing command becomes
group-aware in **one place**: `links::widen` runs at the top of
`commands::apply`, so Move / Delete / Split / Trim never learned about
groups themselves. Move carries each mate by the same offset on its own
lane; Delete takes the mates; Split cuts the mates at the same instant and
re-groups the tails (a cut through a pair leaves two pairs); Trim moves a
mate's edge only when it sits at the dragged edge (±0.5 ms) — an audio
clip that already runs longer than its picture is left alone. One widened
edit is one command, hence one undo. Captions were not made members: they
already follow the video through `captionOf` / `follow_captions`, and test 4
proves they follow the group move for free. **Migration:** on load,
`doc::link_detached_pairs` turns every saved `detached_from` pair into one
group, video mute kept (test 5).

### E6 in one sentence
`Timeline::clips_on_track(track_id)` returns every clip on that lane, first
to last by start, no other lane; the UI's Shift + L maps to it and widens
with `with_link_mates` if the linked audio should come along.

## Validation run in this session (Linux sandbox, Rust 1.93.1)

```
cargo test  -p concat-project -p concat-timing -p concat-effects -p concat-text -p concat-core
  → 529 passed, 0 failed  (concat-project 264 unit + 83 integration; +11 new in tests/link_unlink.rs)
cargo clippy -p concat-project --all-targets -- -D warnings   → clean
cargo fmt   -p concat-project -- --check                       → clean
```

Not compiled here (FFmpeg / whisper.cpp / cpal absent, same as every
earlier checkpoint): `concat`, `concat-host`, `concat-media`, `concat-render`,
`concat-export`, `concat-speech`, `concat-api`, `concat-server`, `concat-cli`.
The new `Command` variants are additive and every `match` on `Command`
outside `concat-project` has a `_` arm (checked by grep), so those crates
should build unchanged; confirm with `cargo check -p concat` on Windows.

## Not built here, plainly (owner's machine)

1. **Menu swap** in `src/crates/concat/src/studio.rs` ~9842–9855: replace the
   `detach` / `reattach` rows with `link` (enabled when the selection holds
   ≥ 2 non-caption clips) and `unlink` (enabled when the clicked clip has
   `link_mates().len() > 1`). Add `studio.linkClips` / `studio.unlinkClips`
   to every `locales/*.json`. Handlers ~10495–10590: send
   `Command::LinkClips { clip_ids: selection }` / `Command::UnlinkClips`.
   Then remove `DetachAudio` / `ReattachAudio` from the menu (the commands
   can stay until the import default below makes them dead).
2. **Linked selection on click:** when a clip is clicked, select
   `timeline.link_mates(&id)` (falls back to just the clip when empty).
3. **Shift + L:** in the key handler, `clips_on_track(current_lane)` →
   selection; decide "current lane" = last clicked lane; confirm no clash in
   the keymap; add the line to shortcut help.
4. **E5-b import default** (video on V1 + audio on A1, linked) and **D6
   typed lanes** — not started; they change the import path, which lives
   in the FFmpeg-side crates.

## Windows smoke test

```powershell
cd C:\dev\kconcat\src
cargo test -p concat-project          # expect 347 passed
cargo check -p concat                 # must still compile (additive variants only)
cargo dev
```

Manual: open an old project that has a detached audio clip → drag the
video → the audio must move with it (migration). Undo → both return.

## Files changed in this checkpoint

```
checkpoint-10-steps-E5-E6.md                                   (new)
AI_Please_README.md                                            (pointer added)
02-map/kconcat-roadmap-v2-execution.md                          (E5/E6 status + §11 rows)
src/crates/concat-project/src/model.rs                          (link_group, link_mates, with_link_mates, clips_on_track)
src/crates/concat-project/src/doc.rs                            (link_detached_pairs migration)
src/crates/concat-project/src/commands/mod.rs                   (LinkClips, UnlinkClips, CannotLink, widen hook, hidden LinkedTrim/LinkedSplit)
src/crates/concat-project/src/commands/links.rs                 (new)
src/crates/concat-project/tests/link_unlink.rs                  (new, 11 tests)
```

## Roadmap state after this checkpoint

Every §11 row now reads DONE / VERIFIED for its sandbox-provable part except:
- **T8** (needs the owner's corrected 30 s transcript — cannot start without it),
- **K1–K5** (Track 6, owner-added "after P6"; designs recorded in
  `02-map/guide-d5-extra-editing.md`, no code — each needs ONNX models / FFmpeg,
  not provable here),
- the **UI halves** of every item (§9 rule: checked in the running app first).

Next sandbox-provable candidate, if the owner wants one: D6 typed lanes in
the model (`Track.kind: V | A` with the legacy typing rule + fixture test),
which E5-b and P3's per-speaker lanes both lean on.
