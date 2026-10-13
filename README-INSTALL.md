# Concat — linked captions + silence detection

Phases 1–3 of the VTS port. Built and tested in a sandbox on the repo's own
pinned toolchain, **1.93.1**.

```
cargo test -p concat-project                      →  217 passed; 0 failed   (was 215)
cargo test -p concat-media silence::              →   13 passed; 0 failed
cargo fmt -p concat-project -p concat-media -- --check   →  clean
cargo clippy -p concat-project -p concat-media --all-targets -- -D warnings  →  clean
```

Build cost to verify all of this: **~155 MB of `target`, under a minute.**
Not the 22 GB — that is the window crate's 724 dependencies, which nothing
here touches.

---

## What's in the box

| File | Status | What it does |
|---|---|---|
| `src/crates/concat-project/src/model.rs` | **replace** | `caption_of` field on `Clip`, plus `captions_of()` and `captions_with_no_clip()` on `Timeline` |
| `src/crates/concat-project/src/lib.rs` | **replace** | 2 new tests (215 → 217) |
| `src/crates/concat-media/src/silence.rs` | **NEW** | Silence detection: `detect()`, `spans_from_windows()`, `rms()`, `linear_from_db()`, 13 tests |
| `src/crates/concat-media/src/lib.rs` | **replace** | one line: `pub mod silence;` |

`patch/phases-1-3.patch` is the same change as one `git apply`, if you prefer that.

---

## Install — pick ONE route

### Route A: git apply (cleanest, and reversible)

Put `patch/phases-1-3.patch` anywhere, then:

```cmd
cd /d C:\dev\kconcat
git apply --check patch\phases-1-3.patch
git apply patch\phases-1-3.patch
```

Undo it any time with:

```cmd
git apply -R patch\phases-1-3.patch
```

### Route B: copy the files

Back up the three you are replacing first:

```cmd
cd /d C:\dev\kconcat\src\crates
copy concat-project\src\model.rs model.rs.bak
copy concat-project\src\lib.rs   projlib.rs.bak
copy concat-media\src\lib.rs     medialib.rs.bak
```

Then copy from the zip, preserving the paths:

```
files\src\crates\concat-project\src\model.rs  →  C:\dev\kconcat\src\crates\concat-project\src\model.rs
files\src\crates\concat-project\src\lib.rs    →  C:\dev\kconcat\src\crates\concat-project\src\lib.rs
files\src\crates\concat-media\src\lib.rs      →  C:\dev\kconcat\src\crates\concat-media\src\lib.rs
files\src\crates\concat-media\src\silence.rs  →  C:\dev\kconcat\src\crates\concat-media\src\silence.rs
```

One-line shortcut if the zip is unpacked to `C:\dev\kconcat\_drop`:

```cmd
xcopy /s /y C:\dev\kconcat\_drop\files\src C:\dev\kconcat\src
```

---

## Verify

**In the build terminal** — double-click `open-a-real-build-terminal.bat`, title bar
must say `Concat build terminal (MSVC)`:

```cmd
cd /d C:\dev\kconcat\src
cargo test -p concat-project
cargo test -p concat-media silence::
```

Expect **217 passed** and **13 passed**.

Then the repo's own two gates:

```cmd
cargo fmt -p concat-project -p concat-media -- --check
cargo clippy -p concat-project -p concat-media --all-targets -- -D warnings
```

Both should print nothing and exit 0.

### If `concat-media` fails to build

It needs FFmpeg's dev headers, same as before. Check:

```cmd
dir "%FFMPEG_DIR%\include\libavcodec\avcodec.h"
```

If that is missing you are not in the build terminal. Close the window and use
`open-a-real-build-terminal.bat`.

### Two pre-existing failures you may see

```
decode::tests::an_hdr_file_decodes_through_the_tone_map
encode::tests::an_hdr_file_is_tagged_and_keeps_its_levels
```

These fail on **pristine** code in my sandbox too — I have FFmpeg 7.1.5 there,
the repo pins 8.1. I verified by reverting my changes and re-running them. If
they fail for you, run `cargo test -p concat-media an_hdr_file` on a clean
checkout to confirm it is the same pre-existing thing and not this patch.

---

## What you now have, and what you still don't

### Have

**1. `Clip::caption_of`** — a caption can name the clip it was transcribed from.
Optional, `#[serde(default, skip_serializing_if = "Option::is_none")]`, exactly like
the existing `detached_from`. **`DOCUMENT_VERSION` stays at 1.** Proven by
`a_caption_keeps_its_link_to_the_clip_it_came_from`, which round-trips a document
through save → load → save → load and also loads a document with no `captionOf`
key at all.

**2. `Timeline::captions_of(clip_id)`** — the ids of a clip's captions, in timeline
order. This is what a drag, a trim and a ripple delete each call to take the
captions along. Mirrors the filter at `concat/src/studio.rs:10312` that gathers a
video clip's detached sound.

**3. `Timeline::captions_with_no_clip()`** — captions whose clip is gone, so a
delete path can clear the dangling links.

**4. `concat_media::silence`** — real silence detection, decoding through the
existing `AudioDecoder` and measuring RMS in Rust. No FFmpeg process is spawned,
which keeps Concat's "nothing is spawned at run time" rule. Returns
`Vec<(f64, f64)>` — **exactly the signature `concat_project::sections::merge`
takes**, which is the two halves of an autotrim finally meeting.

```rust
let spans = concat_media::silence::detect(
    path,
    concat_media::silence::DEFAULT_NOISE_DB,     // -45 dB
    concat_media::silence::DEFAULT_MIN_DURATION, // 0.5 s
    None,                                        // first audio stream
)?;
let kept = concat_project::sections::complement(
    &concat_project::sections::merge(&spans, 0.1),
    total_seconds,
    concat_project::sections::MIN_KEEP,
);
```

### Don't have yet

**Nothing is wired to the UI.** There is still no button, no panel, and no command
that sets `caption_of`. Two deliberate reasons:

- Setting it from a caption run means editing `concat/src/panes/captions.rs`,
  which is in the **window crate** — 724 dependencies, Slint, GPU. I cannot compile
  that in my sandbox, so I would be handing you unverified code. That is how the
  last six errors happened and I am not doing it again.
- `AddTextClip` has **18 construction sites** across five crates. Adding a field to
  it touches all of them. Worth doing, but it should be done where it can be
  compiled.

So: the foundation is in, tested, and cheap to test. The wiring is next and needs
your machine.

---

## Next steps, smallest first

**A. Set the link from a caption run** — `concat/src/panes/captions.rs`.
`CaptionsPane.clip: Option<String>` already holds the id; `caption_clip()` at
line 383 already builds the `Command::AddTextClip`. Either add a `caption_of`
field to `AddTextClip` (18 call sites, all `caption_of: None`) or apply an
`UpdateClip` patch straight after in the same `Batch`.

**B. Captions follow a drag** — `concat/src/studio.rs`, beside line 10312 where
detached sound is already collected. Call `timeline.captions_of(&clip.id)` and add
those ids to the same `MoveClips` batch.

**C. A "detect silences" job** — `concat-host`, reporting spans the way the
transcriber reports percent. `concat_media::silence::detect` is the work.

**D. Autotrim** — `SplitClips` + `RemoveClips { ripple: true }` +
`Command::Batch` for one undo step + `captionmap::remap` for the survivors. All
four already exist and are tested.

---

## Design notes worth knowing

**Why RMS and not peak.** A peak meter reads one stray sample and calls the whole
10 ms window sound. `rms_reads_a_click_far_quieter_than_a_peak_would` pins the
real numbers: a full-scale click in a 480-sample window reads **-26.8 dB** on RMS
against **0 dB** on peak — 26.8 dB of forgiveness. It is still above the -45 dB
floor, which is correct: a click *is* a sound. `noise_under_the_floor_stays_silent_where_a_peak_meter_would_not` covers the case RMS is actually for, where the two
meters disagree and RMS is right.

**Why -45 dB and 0.5 s, not FFmpeg's -60 dB and 2 s.** FFmpeg's defaults are for
finding gaps between scenes. For trimming the pauses inside a talking head, -60 dB
is quieter than a real room and 2 s misses every sentence gap. Both are arguments
to `detect()`, so nothing is baked in.

**Why 10 ms windows.** `sections::MIN_SECTION_MS` is 120 ms, so a span this returns
is always several windows wide and never below the shortest section the list shows.

**Why a delete does not rewrite the captions.** `captions_are_found_by_the_clip_they_came_from`
asserts this on purpose: a command that removes one clip has no business editing
others, and a `Batch` that did would not be one undo step. The links go dangling
and the caller clears them via `captions_with_no_clip()`. That is the contract the
delete path relies on, so it is written down as a test.

**Why the split between `spans_from_windows` and `detect`.** Same shape as your own
`sections.rs`: the rule is pure and testable with no file, no decoder and no
FFmpeg; only the streaming is I/O. All 13 tests run against `spans_from_windows`
and need nothing but arithmetic.
