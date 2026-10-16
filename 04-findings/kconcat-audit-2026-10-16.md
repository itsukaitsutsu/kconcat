# kconcat — proper audit

**Repo:** `itsukaitsutsu/kconcat` at `1aece2d` ("plan to get closer", 16 Oct 2026 07:11 +0700)
**Method:** fresh clone, the project's own pinned toolchain installed (Rust 1.93.1), then **actually built and ran the tests** — plus new tests written to probe your newest change. Every claim below is a command I ran or a `file:line` you can open.

---

## 1. What checks out (verified, not read)

| Claim | Command | Result |
|---|---|---|
| `concat-project` tests pass | `cargo test -p concat-project` | **252 passed, 0 failed** ✅ |
| Lints clean | `cargo clippy -p concat-project --all-targets -- -D warnings` | clean ✅ |
| Formatting clean | `cargo fmt -p concat-project -- --check` | clean ✅ |
| `concat-core` builds | `cargo test -p concat-core` | builds, 2 doctests pass ✅ |

**Your numbers are honest.** The crate holding your own logic is clean by the project's own standard, and the test count matches what your checkpoints claim. That's worth saying plainly, because most of what follows is criticism and the baseline is good.

## 2. What I could not verify — read this before trusting any end-to-end claim

`concat-media`, `concat-render`, `concat-export`, `concat-host` and the `concat` GUI all link **FFmpeg 7+**, and this sandbox has no `libavutil`. Build attempt:

```
The system library `libavutil` required by crate `ffmpeg-sys-the-third` was not found.
```

So the `95/95`, `443/443` figures in your checkpoints for *other* crates are **unverified by me** — not doubted, just untested. It also means the only crate whose behaviour I can prove anything about is `concat-project`.

---

## 3. Your newest commit is good work — and that's why the bug below matters

`1aece2d` loosened a core invariant: **captions may now overlap on a caption lane**, while titles keep the old no-overlap rule. Changing a rule that the whole timeline engine assumes is a real decision, and you did it properly:

- `nearest_start_where` factors out the shared candidate search instead of duplicating it
- `is_caption_free` / `nearest_caption_start` are separate from the ordinary pair, so titles can't inherit the looser rule by accident
- The module doc comment was rewritten to state the new rule exactly (`placement.rs:5-11`)
- A test covers it: `captions_can_overlap_on_one_track_without_allowing_titles_to_cover_them`

I also tested two cases expecting them to be broken and found they were handled correctly:

- **caption entirely inside a removed region** → deleted, no orphan ✅
- **cut through the middle of a caption** → split into two pieces, each keeping *its own* words, text rewritten to the surviving words ("halo" / "dunia"), both still captions ✅

That second one is genuinely nice behaviour and I didn't expect it.

---

## 4. The bug I did find — reproduced, with a failing test

### Reproduction

`crates/concat-project/tests/caption_identity_through_a_trim.rs` (included with this audit)

An imported caption (`caption_of: None`, identity resting on `words`) where the words sit in the second half of the cue — normal for real .srt files. Keep `[0, 3)`:

```
--- BEFORE ---
kind=Text  start=2.00 dur=2.00  caption_of=None  words=Some(2)  is_caption()=true

--- AFTER  ---
kind=Text  start=2.00 dur=1.00  caption_of=None  words=None     is_caption()=false
           text is still "halo dunia"
```

The clue survives the trim, but **stops being a caption**.

### Root cause

Two lines, each reasonable alone:

```rust
// model.rs:1509 — identity is DERIVED
self.kind == ClipKind::Text && (self.caption_of.is_some() || self.words.is_some())

// commands/clips.rs:883 — words cleared when a surviving piece keeps none
} else {
    piece.words = None;
}
```

For an imported cue there is no `caption_of`, so `words` is the *only* thing holding its identity. Clear the words and the identity goes with them. `rewindow_captions` does the same thing at `clips.rs:1129`.

### Three concrete symptoms

1. **Export — the worst one.** `is_caption_clip()` at `concat/src/panes/export.rs:835` is the same rule, and the filter at `export.rs:643` is:
   ```rust
   .filter(|title| burn_captions || !caption_clip_ids.contains(&title.clip_id))
   ```
   A ghost is no longer in `caption_clip_ids`, so with **"Burn captions" switched OFF, the ghost gets burned into the video anyway.** The switch silently leaks a caption — the exact complaint the switch was added to fix.

2. **Placement.** The ghost loses the caption↔caption overlap allowance and starts acting as an obstacle: `first_caption_track_above` (`clips.rs:1282`) counts non-caption text as blocking, so later captions get pushed to a higher lane for no reason. Drag the ghost afterwards and it snaps to a gap instead of staying where you put it.

3. **Bulk caption edits.** `studio.rs:3532` returns early for non-captions, so "apply to all captions" and sibling styling skip the ghost.

### The deeper problem: this rule is copy-pasted six times

| Where | What it decides |
|---|---|
| `concat-project/src/model.rs:1509` | `Clip::is_caption()` — the canonical one |
| `concat-project/src/commands/clips.rs:85` | caption-ness at creation |
| `concat-project/src/sectionlist.rs:189` | which rows are caption sections |
| `concat/src/panes/export.rs:835` | what "burn captions" excludes |
| `concat/src/panes/sections.rs:1449` | section rows in the UI |
| `concat/src/studio.rs:3532` | caption track siblings |

One rule, six copies, three crates — and four of them re-implement it inline instead of calling `Clip::is_caption()`. Any change to what counts as a caption must be made six times, and a single miss produces exactly this class of bug. That's the real finding here.

### Fix, in order of preference

1. **Store the identity instead of deriving it.** A field set once at creation — e.g. `Clip.origin: TextOrigin::{Title, Caption}` — that `is_caption()` reads. Then no trim, edit or word-clearing can flip it. This kills the whole class.
2. **Minimal patch:** at `clips.rs:883`, don't clear `words` when the piece came from a caption — keep `Some(vec![])`. You already distinguish "never had words" from "lost them" in `captionmap` (`a_cue_that_had_no_words_does_not_gain_an_empty_claim_to_have_them`), so the distinction is available to you.
3. Whichever you choose, make the other five sites call the one function.

---

## 5. Smaller findings

- `src/]` and `src/nulrobocopy` are still committed (both dated 16 Oct). Harmless, in every clone.
- Your checkpoints name branch `arena/fix-sync-export-checkpoints`; origin has only `main` + two dependabot branches. Push it or stop naming it — a checkpoint that points at a branch nobody can check out is a dead end later.
- Nine `CHECKPOINT-*.md` / `README-PHASE-*.txt` files at the root, plus the committed `caption-sync-fix-plan.md`. Fine for personal use — but you now have three overlapping descriptions of the same work, and they disagree in places.
- `is_caption()`'s doc says "generated or imported caption rather than a hand-made title". A title that ever acquires word timings (the word-reveal path sets `words`) takes the caption branch. Worth a test if that path can reach a title.

## 6. Verdict

In better shape than your repo surface suggests: **tests pass, lints are clean, and the newest architectural change is documented and tested** — including two tricky caption/trim cases I expected to fail. One real bug, reproduced with a test to keep, caused by derived identity and a rule copied six times. Everything downstream of `concat-project` stays unverified here; the honest confidence test is a Windows build plus one real edit — import → caption → remove a silence → export with **Burn captions off** — and check whether a caption you trimmed came back burned in.

---

## 7. Penjelasan sederhana (Bahasa Indonesia)

### Masalahnya: "KTP" caption-nya hilang

Setiap blok caption di timeline punya **tanda pengenal**, supaya aplikasi tahu blok itu caption (subtitle), bukan tulisan/judul biasa. Ada dua jenis tanda pengenal:

1. **Link ke klip video** — "caption ini milik klip A". Ini ada pada caption hasil **Auto-Caption**.
2. **Data waktu tiap kata** — timestamp tiap kata (dipakai untuk karaoke / highlight per kata).

Caption hasil **import .srt/.vtt** cuma punya nomor 2. Dia tidak punya link, karena asalnya dari file, bukan dari fitur Auto-Caption.

Aturan di dalam kode:

```
ini_caption = (punya link) ATAU (punya data waktu kata)
```

Jadi untuk caption import, **data waktu kata itu satu-satunya tanda pengenal**. Anggap saja itu KTP-nya.

### Apa yang terjadi waktu kamu buang bagian video

Contoh: video 10 detik, ada caption di detik 2–4, isinya "halo dunia".

```
detik:     0     1     2     3     4     5 ..... 10
video:     [──────────────────────────────────────]
caption:               [2 ─────────── 4]
kata:                            [3.5 .. 3.9]
                                 ↑
                    "halo" dan "dunia" ada di sini
```

Kamu potong di detik 3 (buang semua setelah detik 3):

```
yang disimpan:  [0 ──── 3]
yang dibuang:                  [3 ──────────── 10]
```

Yang terjadi:

- Sisa caption dari detik 2 ke 3 **masih ada** di timeline.
- Tapi semua katanya ada di detik 3.5–3.9 — **ikut terbuang**.
- Kode lalu berpikir: "blok ini sudah tidak punya kata lagi, bersihkan saja" → data kata dihapus.
- **Padahal data kata itu KTP-nya.** Jadi KTP-nya ikut terhapus.

Hasilnya: blok teks masih ada di timeline, teksnya masih "halo dunia", kelihatannya normal. Tapi aplikasi sudah **tidak menganggapnya caption lagi** — dianggap tulisan biasa. Ini yang saya sebut "ghost" (hantu).

### Kenapa ini merugikan (3 akibat)

1. **Paling parah — waktu export.** Pas kamu matikan opsi "Burn captions", aplikasi justru memasukkan semua teks yang *bukan* caption ke video. Karena si hantu sudah dianggap bukan caption, dia malah **ikut terbakar ke video**. Kebalikan dari yang kamu mau.
2. **Penempatan caption jadi aneh.** Caption lain jadi terdorong ke track lebih atas, karena si hantu dianggap penghalang. Padahal dia caption.
3. **"Apply to all captions" melewatinya.** Fitur edit massal tidak menyentuh si hantu.

### Kenapa ini bisa kejadian

Karena "apakah ini caption?" selalu **dihitung ulang** dari data yang ada, bukan **disimpan** sejak awal. Ibaratnya KTP-nya digambar ulang tiap kali dari ingatan — begitu ingatannya hilang, KTP-nya hilang.

Solusi paling rapi: **simpan tanda pengenal itu permanen saat blok dibuat** (misalnya field yang isinya "ini caption" / "ini judul"), jadi tidak bisa hilang gara-gara potong video.

### Cara cek sendiri

Bikin caption dari import .srt, lalu buang bagian video yang jatuh di *tengah* caption, tapi cari yang kata-katanya ada di ujung. Setelah itu export dengan **Burn captions dimatikan**. Kalau caption yang kamu potong muncul di video, berarti kena bug ini.

*(Catatan: tulisan `[0, 3)` di penjelasan bahasa Inggris sebelumnya cuma cara singkat menulis "disimpan dari detik 0 sampai detik 3". Maaf, itu notasi teknis yang bikin bingung.)*

---

## 8. Follow-up: this explains the "repeated unsync" — and why AI keeps leaving it

I tested three more scenarios after you said the unsync keeps coming back. Results:

| Question | Result |
|---|---|
| Does the ghost still follow a **second** trim? | ✅ **Yes.** Tested: it lands exactly where it should (1.00s, 1.00s duration). Your rebuild path is solid — the ghost does not drift on trims. |
| Does the Sections panel still **see** the ghost as a caption? | ❌ **No.** `clip_sections_with_cues` counts caption sections using the same rule (`sectionlist.rs:189`). Measured: **1 caption section before the trim → 0 after.** |
| Is that the sync path? | Very likely. See below. |

### The mechanism

`sectionlist.rs:186-189` decides which timeline clips are caption cues:

```rust
&& (linked_ids.contains(&c.id)
    || (c.caption_of.is_none() && c.words.is_some()))
```

A ghost has `words = None` and no link, so it drops out of that list. That list is what `carve_silences_around_cues` and the section tiling use to know **where a caption sits**. Lose the entry, and the trim planner no longer knows a caption is there — so a later "Remove selected" can cut straight through it. The caption is still on the timeline; the *tool* stopped treating it as one. That reads to a user as "sync broke again".

So there are **two different** sync problems, and they need different fixes:

| What you see | Cause | Fix |
|---|---|---|
| Words inside a caption land early/late; the block is roughly right | Whisper attention timings | `caption-sync-fix-plan.md` |
| A caption gets ignored/clipped/leaked by a feature after a trim | Identity bug (this doc) | Store the identity; stop deriving it |

### Why the AI left this behind (the useful part)

Not because the AI is bad at Rust — your modules are well written. Four specific reasons, all fixable in how you brief it:

1. **It fixes the instance you showed it, not the class.** You reported a sync symptom; it patched the path you pointed at. The rule lives in **six places**; the other five kept the old behaviour, so the symptom came back somewhere else. That *is* the "repeated" feeling.
2. **This failure is silent.** Nothing crashes, no test fails, no error prints — a caption just quietly stops being a caption. Agents verify by "does it build, do tests pass", and silent state loss passes both.
3. **Green tests give false confidence.** 252 tests pass because they test the model layer. The damage happens at the integration points (export, sections panel, studio) which have no tests. An agent that checks the suite concludes "done".
4. **The repo is bigger than one session's attention.** `studio.rs` alone is ~10k lines, and the same predicate is copied across three crates. A session editing `export.rs` cannot see that `sectionlist.rs` has the same bug.

### How to brief your agent so this stops happening

1. **Ask for the rule, not the symptom.** *"Find every place that decides whether a text clip is a caption. Make `Clip::is_caption()` the only one. Replace all inline copies."* This turns a whack-a-mole fix into a one-shot fix.
2. **Grep before fixing.** *"Before changing anything, list every occurrence of this predicate across the repo."* Cheap, and it prevents partial fixes.
3. **Demand a failing test first.** *"Write a test that reproduces this and show it failing, then fix it."* Forces the agent to find the real cause instead of patching the visible surface.
4. **Keep an invariants file.** A short `ARCHITECTURE-INVARIANTS.md` with rules like *"a caption is identified by a stored flag set at creation — never derived"* and *"no two clips share a span on a lane, except caption↔caption"*. Hand it to the agent at the start of every session. Agents follow stated rules far better than they infer unstated ones.
5. **Never accept "tests pass".** Ask: *"which test covers the path I reported?"* If the answer is none, the fix is unverified — that's the moment to ask for one.
6. **General principle:** if the app must remember *what something is*, store it. Don't recompute it from data another feature is allowed to clear.

---

### Files with this audit
- `caption-identity-bug.patch` — 3 tests, applies in your repo root: `git apply caption-identity-bug.patch`
  (1 expected failure — the identity bug; 1 expected pass proving the ghost still follows trims; 1 expected failure — the caption vanishing from the Sections panel)
