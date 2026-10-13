# VTS → Concat, Tranche A: three pure modules

Ported from `itsukaitsutsu/video-trim-studio` into `jub0t/Concat`, targeting
`concat-project` v0.2.7 (branch `main` at `2d3a43e`).

These are the parts of Video Trim Studio that are **pure logic** — no files, no
FFmpeg, no GPU, no engine type in any signature. That is deliberate: it is the
only layer that is cheap to move, cheap to review, and cheap to test, and it is
the foundation every later piece (word-level captions, autotrim, karaoke) is
built on.

**Status: complete and verified.** 101 new tests, 215 in the crate, all passing.
`cargo clippy --all-targets -- -D warnings` clean. `cargo fmt --check` clean.

---

## What to copy

Four files, all under `src/crates/concat-project/src/`:

| File | Lines | Status | Tests |
|---|---:|---|---:|
| `subtitle.rs` | 515 | **new** | 24 |
| `sections.rs` | 783 | **new** | 33 |
| `captionmap.rs` | 1086 | **new** | 44 |
| `lib.rs` | 4297 | **modified — 3 lines added** | — |

The `lib.rs` change is only the module list; nothing else in it moves:

```diff
+pub mod captionmap;
 pub mod commands;
 pub mod doc;
 pub mod editor;
 pub mod model;
 pub mod placement;
+pub mod sections;
 pub mod speed;
+pub mod subtitle;
```

If you would rather not overwrite your `lib.rs`, add those three `pub mod` lines
by hand and copy only the three new files.

No `Cargo.toml` change is needed — `serde`, `serde_json` and `thiserror` are
already dependencies of the crate.

---

## Two ways to install

**Option 1 — copy the files.** Drop the three new files in and add the three
`pub mod` lines to `lib.rs`.

**Option 2 — apply the patch** (keeps authorship and the commit message, and is
what you want if this goes upstream):

```sh
cd Concat
git checkout -b vts-tranche-a
git am patch/0001-tranche-a.patch
```

The patch is signed off (`-s`), as `CONTRIBUTING.md` asks. **Amend the author
before pushing anywhere** — it was committed here as `VTS port
<vts-port@local>`:

```sh
git commit --amend --author="Your Name <you@example.com>" --no-edit
```

---

## Verify

```sh
cd src
cargo test -p concat-project
cargo fmt -p concat-project -- --check
cargo clippy -p concat-project --all-targets -- -D warnings
```

Expected: `test result: ok. 215 passed; 0 failed`.

**One thing that will happen and is not this patch's fault.** The workspace pins
Rust `1.93.1` in `src/rust-toolchain.toml`. On a newer toolchain (this was
verified on 1.99.0), clippy fires `chunks_exact_to_as_chunks` in three
**pre-existing** places in `concat-core`:

```
crates/concat-core/src/frame.rs:130
crates/concat-core/src/frame.rs:298
crates/concat-core/src/shader.rs:162
```

That lint does not exist in 1.93.1. Either build with the pinned toolchain, or
add `-A clippy::chunks_exact_to_as_chunks` to see your own result. None of the
three sites are touched by this patch.

---

## What each module does

### `subtitle.rs` — SubRip and WebVTT, in and out

Answers the open user request in `ARCHITECTURE.md` §15 for SRT import and export.

One tolerant pass reads both formats, because they differ in their header and
their decimal mark and in almost nothing else. Handled, with a test each: a
byte-order mark, CRLF and lone-CR endings, optional cue indices, a `WEBVTT`
header carrying a title, `NOTE` / `STYLE` / `REGION` blocks, **cue settings
trailing the timing line** (`line:90% position:50%`), inline tags
(`<v Roger>`, `<b>`, `<00:00:01.000>`), an unclosed `<` treated as a character,
`MM:SS.mmm` stamps with the hours left off, one-to-three-digit fractions, cues
out of order, zero-length and backwards cues, and a four-digit fraction refused
rather than misread.

Writes both formats back, with cues round-tripping exactly in any script —
Indonesian, Japanese, Arabic and Thai all have a test, and the writer escapes
nothing, because the point of a local editor is that nobody's language is
special-cased.

Word timings do **not** survive this format: neither SRT nor VTT has a place for
them. That is why `captionmap` exists, and why a `From` conversion between the
two cue types is provided in both directions with a test showing exactly what is
lost.

### `sections.rs` — the arithmetic a trim by content is made of

Four pieces, in the order they are used:

- **`silence_spans(text, total)`** reads the lines an FFmpeg `silencedetect`
  filter prints and returns the spans. Text in, spans out — who ran the filter,
  and on what, is somebody else's business, which is why this crate can hold it.
  Handles the case that matters most: **a silence running to the end of the file
  never prints `silence_end`**, so `total` supplies it. Without that, the last
  pause of every recording is the one that goes undetected.
- **`merge(spans, pad)`** folds overlaps *and* adjacencies, optionally keeping a
  pad of air at each end of every cut. The pad is the difference between a tight
  read and a chopped one, and is the equivalent of Resolve's Pre Head / Post Tail.
- **`complement(spans, total, min_length)`** returns the kept ranges — the whole
  of a trim expressed as arithmetic. Ranges too short to hold are dropped rather
  than becoming a stutter.
- **`build_sections(duration, silences, cues, min_section_ms)`** tiles the source
  into `caption` / `silence` / `other` pieces so what is about to be deleted can
  be read first. **Boundaries come from both sets**, so no piece is half silence
  and half speech — which is what makes ticking one row mean exactly what it says.
  Neighbours agreeing on kind *and* words are merged back; slivers are folded.

Plus `summarize` for the one line a panel says above its list, and a `TimedCue`
trait so a transcription's own segments, a file's cues and a document's caption
clips can all be tiled by the same function.

A test asserts the two routes to a trim agree: ticking every non-silence section
and taking the complement of the silences produce the same total. If those ever
diverge, the list a user ticks and the ranges an export keeps are different
edits.

### `captionmap.rs` — a cue's words, and what a cut does to them

The piece that makes linked caption↔video trimming possible.

**`clean_words`** puts whatever a recogniser emitted into the shape everything
downstream assumes: sorted, inside the cue, **never overlapping a neighbour**,
with a punctuation-only token glued onto the word before it rather than standing
alone (a token engine emits `.` as its own token starting exactly where the
previous word ends, and a caption reading `cerah .` is wrong). Padding, gluing
and de-overlapping are reported back as a change, which is what turns a cue's
`WordSource` into `Edited` instead of leaving it claiming to be what the
recogniser said.

The alphanumeric test is `is_alphanumeric`, **not** `is_ascii_alphanumeric` — if
it were the other question, every word in a Japanese, Arabic or Thai caption
would read as punctuation and glue into one. There is a test that asserts exactly
that.

**`remap(cues, kept, min_piece)`** moves a cue list through a cut: shifted by
everything removed ahead of it, split where a cut went through one, **joined
again where the two halves of one sentence landed back to back** (a cut inside
one sentence should not leave two captions both saying it), and gone where
nothing was kept — which is the whole product: the caption goes and the picture
under it goes with it.

The words travel with their cue and are **clipped at the cut**, not dropped: a
word straddling the join is heard on both sides, and losing it loses a syllable
from the caption and a beat from the karaoke timing. There is a test for the
subtle case — kept ranges that only touch, where a word on the boundary would
arrive from both halves.

`Cue` carries `id`, `track` and an opaque `style` map, all of which survive a
remap with a test each. `track` is what keeps **stacked bilingual caption lanes**
from collapsing into one: cues on two lanes are never joined, even when their
words match and their times do. Serde keys are camelCase, matching the document's
spelling, and `Word` writes as `{"w","s","e"}` — three letters, because a
ten-minute video carries a few thousand of these into a document and back out.

---

## Provenance and licence

Ported from Video Trim Studio:

| Concat | VTS |
|---|---|
| `subtitle::parse` | `vts/detect.py:parse_subtitles` |
| `subtitle::to_srt` / `to_vtt` / stamps | `vts/transcribe.py:write_srt` / `write_vtt` / `to_srt_time` / `to_vtt_time` |
| `sections::silence_spans` | `vts/detect.py:detect_silences` (the parsing half) |
| `sections::merge` / `complement` | `vts/detect.py:_merge` / `complement` |
| `sections::build_sections` / `summarize` | `vts/detect.py:build_sections` / `summary` |
| `captionmap::clean_words` | `vts/segmenter.py:clean_words` |
| `captionmap::remap` / `clip_words` | `vts/captionmap.py:remap_cues` / `_clip_words` |

Every function was **verified against the Python original** rather than ported
from reading. Where this port and VTS disagreed, VTS was treated as the
specification and the Rust was corrected — that happened once, on a WebVTT
timing line with trailing cue settings, where the first Rust version looked for
the last decimal mark on the line and found the one inside `line:90%`.

**Licence — action needed before this goes upstream.** Video Trim Studio
currently has **no LICENSE file**, which under Berne means all rights reserved.
Since it is your own repository, add one before offering any of this to Concat:
**AGPL-3.0-or-later** matches Concat and keeps the two from drifting apart;
MIT or Apache-2.0 also work, since Concat can take permissive code in. Concat
additionally requires a signed CLA (`CLA.md`) and the SPDX header on every new
file, which all three files here carry:

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
```

Add your own copyright line beside it rather than replacing that one.

`CONTRIBUTING.md` also asks that anything beyond a small fix be raised as an
issue or in Discord first, because "large areas are already in progress or
intentionally deferred". SRT import/export is an open user request, so it is
worth naming in the issue — and so is the fact that this patch touches no
document format and no `DOCUMENT_VERSION`, which is one of the deferred areas.

---

## What this does not do

Stated plainly, so nothing is assumed:

- **No FFmpeg is called.** `silence_spans` reads the *output* of a
  `silencedetect` pass. Running the pass belongs in `concat-media`, which links
  FFmpeg; `concat-project` deliberately does not.
- **No transcription.** That is `concat-speech`, which needs FFmpeg's dev
  libraries to build.
- **No UI, no commands, no document change.** Nothing here is wired into
  `Command`, `Editor` or the window. These are the functions those will call.
- **No word grouping yet.** `segmenter.py`'s `group_words` — the
  engine/line/burst/word/custom policies that turn words into cues — is the next
  module and the biggest remaining piece of pure logic.

---

## What comes next

In the order the dependencies run:

1. **`group_words`** (from `vts/segmenter.py`, 709 lines) → `concat-project` or
   `concat-speech`. The short-form caption policies; also pure, also testable
   here. This is the differentiator: no other free editor groups captions by word
   count with pause-aware breaks.
2. **Word timings out of whisper** → `concat-speech/src/transcribe.rs`.
   `set_token_timestamps(true)`, then `segment.n_tokens()` / `get_token(i)` /
   `token_data()` giving `t0`, `t1`, `t_dtw`, `pt`, `ptsum`. Needs FFmpeg to build.
3. **Translate to English** → `params.set_translate(true)`. Already in
   whisper-rs 0.16 and unused. Nearly free.
4. **Silence detection** → `concat-media/src/silence.rs`, decoding through the
   existing `AudioDecoder` and computing energy in Rust, which keeps Concat's
   "nothing is spawned at run time" rule. Hands its spans to `sections::merge`.
5. **Autotrim** → assembled from verbs that already exist and are already tested:
   `SplitClips`, `RemoveClips { ripple: true }`, `Command::Batch` for one undo
   step, and `captionmap::remap` for the survivors. The missing piece is a link
   field on the caption clip — and `concat/src/panes/captions.rs` already
   computes exactly that as `Subject { start, speed, look }` and throws it away.
6. **Vulkan** → `whisper-rs-sys 0.15.0` (the version already pinned) has a
   `vulkan` feature, and `whisper_rs::vulkan::list_devices()` returns name and
   VRAM. Small Rust, large build cost across three platforms.

Item 5 is the one to raise as an issue first, because it needs a decision about
`DOCUMENT_VERSION`, which `ARCHITECTURE.md` §15 lists as a known deferred defect.
