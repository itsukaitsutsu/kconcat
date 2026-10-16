# kconcat vs CapCut — the actual feature list

*16 Oct 2026. Compiled from reading your tree, upstream `jub0t/Concat` at HEAD, and CapCut's 2026 feature set. This is the inventory you asked for, not a strategy essay.*

---

## First: you were right, twice

**On the repo.** You said "who cares about the repo" — and for personal use, you're correct. Green CI, cleanup, README, releases: that list only exists to help *strangers* trust your code. If nobody's installing it, it's pure overhead. Ignore it. (The one thing worth keeping even solo: a backup and a build you can reproduce after a Windows reinstall. That's not hygiene, that's not losing your own work.)

**On my roadmap being too simple.** Fair. I gave you phases with "done-when" gates and skipped the part you actually wanted: *here is everything CapCut has, here's what you have, here's what's missing, here's what each one costs.* That's below.

---

## How to read the tables

- **Status in kconcat** = verified by grepping your tree + upstream, not guessed
- **Effort**: S = a weekend · M = a few weeks · L = months
- **Worth it for you?** = judged against *your* stated use (personal, offline, simple talking-head/screen content, not a CapCut rival)

---

## Option B — your 15-item list, verified item by item

*Your numbering, from the 16 Oct discussion. "kconcat today" re-checked against
`1aece2d`; where a row differs from an earlier claim, the corrections block below
is the audit trail. T/C/E/V are plan items in `02-map/kconcat-roadmap.md`.*

| # | Your item | CapCut | kconcat today | To close the gap | Plan home |
|---|---|---|---|---|---|
| 1 | Word-level timing | ✅ solid | 🟡 ~±200–500 ms — Whisper cross-attention stamps (`concat-speech/src/transcribe.rs`) | precise word times; targets in `03-manuals/A-word-timing-level1.md` | T2 · T3 |
| 2 | Chunking, 1–3 words a cue | ✅ | ✅ **exists** — four layout modes incl. Burst (`crates/concat/ui/dialogs/captions.slint:78`) + Re-split (`:615`); policy `crates/concat-project/src/captionmap.rs:597–627` | nothing to build — re-validate once #1 lands | C1 (validate) |
| 3 | Caption style presets | ✅ 100s | 🟡 32 presets ship (`crates/concat/text-presets/`), all title looks; none caption-shaped | write caption-shaped preset files | C2 |
| 4 | Karaoke per-word reveal | ✅ | 🟡 renderer wired — one highlight overlay per heard word (`concat-host/src/titles.rs:390–402`); **0 of 32 presets** set `highlight_color` | one karaoke preset (a preset file); animated pops wait on the motion schema | C3 |
| 5 | Uppercase / case transform | ✅ | ❌ none — every `to_uppercase` hit in the tree is UI labelling or first-letter capitalisation, never a style transform | a case transform in the style model | C4 |
| 6 | Caption motion | ✅ | ❌ no schema — and **0 of 32 presets** mention motion | decide where motion lives, then build | C5 |
| 7 | Edit through the transcript (text → cut) | ❌ CapCut too | ❌ | build — `captionmap.rs` is ~80% of the machinery | E1 |
| 8 | Filler-word removal | ✅ English-only | ❌ | Indonesian list — your corpus already gives: gak 29 · ya 20 · gitu 15 · jadi 14 · itu 14 | E2 |
| 9 | Waveform + draggable word edges | ✅ | 🟡 waveform **exists** (`crates/concat/ui/timeline/lanes.slint:184`); nothing drags a *word* edge | word-edge nudge in the caption lane | E3 |
| 10 | Auto-reframe 16:9 → 9:16 | ✅ Pro-only since Apr 2026 | ❌ (`centroid` appears nowhere in the tree) | RVM masks exist → centroid-driven crop track | V1 |
| 11 | Export presets + loudness | ✅ | 🟡 three quality cards only (`crates/concat/ui/dialogs/export.slint:137`); `loudnorm` runs pre-ASR only (`concat-speech/src/transcribe.rs:484`) | platform presets + −14 LUFS at export | V2 |
| 12 | Auto-ducking | ✅ | ❌ | duck music under speech | V3 |
| 13 | Batch export | ✅ | 🟡 API jobs only (`crates/concat-server/src/lib.rs:20`), nothing in the export UI | a queue in the export UI | V4 |
| 14 | Stabilisation | ✅ | ❌ | L — skip unless you film handheld | V5 |
| 15 | Indonesian ASR tuning | ✅ cloud, generic | 🟡 model ladder + **Prompt field ships end to end** (`crates/concat/ui/dialogs/captions.slint:60`, `:104`, `:332–341` → `concat-speech/src/transcribe.rs:607–608`, `:1160–1161`, `:1315–1316`); no Indonesian fine-tune exists (checked 16 Oct) | measure (WER), bake off the ladder, tune VAD/params; own fine-tune = standalone project (L) | **proposed T8 — pending your call** (sub-points 15a–e) |

**Found after your list started** — *Enter splits a caption at the caret*: CapCut ✅,
VTS ✅, kconcat ❌. `crates/concat/ui/primitives/modal.slint:634` binds `edited`
only, and the row editor (`crates/concat/src/panes/sections.rs:680 edit_row_text`)
rewrites `style.content` through `UpdateClip` and stops — no split, no re-time,
`clip.words` untouched. Plan home: **E4**.

**Already matched or ahead of CapCut** (do not rebuild): offline auto-caption,
.srt/.vtt import *and* export, captions that survive cuts, multi-track caption
tracks, scripting/API. The ordered build list that comes out of this table is §6.

---

## Corrections — 16 Oct 2026, second pass through the code

Five rows below were written from module *names*, not from the code that runs.
They are fixed here, and the tables carry the fixed values.

| What I claimed earlier | What the code actually does |
|---|---|
| Word chunking (1–3 words a cue) is missing | **Exists.** Four layout modes — Engine, Readable lines, Burst (3 words), One word at a time (`crates/concat/ui/dialogs/captions.slint:78`) — plus a **Re-split** button (`:615`), with the policy in `GroupPolicy::for_preset` (`crates/concat-project/src/captionmap.rs:597–627`) applied by `resplit_drafts` (`crates/concat/src/panes/captions.rs:596`). It reads badly only because the word times it cuts on are ±200–500 ms |
| The preset system exists but is empty | **32 presets ship** (`crates/concat/text-presets/`) — all title looks, none caption-shaped. A data gap, not an engine gap |
| One-click "apply style to track" is missing | **Exists**: "Apply to all captions", checked by default, scoped per caption track (`crates/concat/src/studio.rs:963`, `3550`, `5279`) |
| Split / merge of cues is only partial | Split and merge exist **at the playhead** (`Command::SplitClips` `crates/concat-project/src/commands/mod.rs:513`, `MergeClips` `:574`). What is missing is a **text-first** split — the row added below |
| Karaoke highlight is not reachable from the UI | The renderer is wired: one highlight overlay per heard word, built from `clip.words` through `locate_words` (`crates/concat-host/src/titles.rs:367–390`, `crates/concat-project/src/captionmap.rs:241`) |

**New gap found the same way** (reported by the owner, then confirmed in the
code): pressing **Enter** while editing a caption row in **Sections** does
nothing. The row's editor is a `Field` → `TextInput` that binds only `edited`
(`crates/concat/ui/primitives/modal.slint:615`, `:634`) and never `accepted` —
`accepted` is used elsewhere in this same UI
(`crates/concat/ui/primitives/number-field.slint:138`,
`crates/concat/ui/timeline/tabs.slint:159`), so the wiring pattern exists and is
simply absent here. The handler behind the row
(`crates/concat/src/panes/sections.rs:680 edit_row_text`) writes `style.content`
and stops: no split, no re-timing, and `clip.words` is left untouched — so after
a text edit the highlight can no longer find its words (`locate_words` returns
`None` for a word whose text is gone).

**Third pass, same day — every remaining ❌ and 🟡 in these tables re-checked
against `1aece2d`.** Two more rows were wrong the same way the first five were:

- **The waveform view exists.** A sound band is drawn inside video clips
  (`ui/timeline/lanes.slint:184`, `ui/primitives/wave.slint`), and the media pane
  and speech dialog draw it too. What is missing is a waveform under the
  *caption* row and any drag on a *word* edge.
- **Caption translation is not absent.** Auto-caption ships Whisper's
  *translate to English* task (`ui/dialogs/captions.slint:57`,
  `concat-speech/src/transcribe.rs:248`). Other target languages: none.

Three rows were right in kind but vague, now made exact: voiceover recording is
microphone-only; `loudnorm` runs *before transcription*, not at export; and the
API's jobs are not an export queue.

Verified genuinely absent: stabilisation, auto-ducking, vocal isolation, beat
sync, filler-word removal, speaker diarisation, auto-reframe (`centroid` appears
nowhere in the tree), and platform export presets.

---

## 1. Captions & subtitles — your home turf

| Feature | CapCut | kconcat | Gap | Effort | Worth it? |
|---|---|---|---|---|---|
| Auto-caption (speech → text) | ✅ cloud only | ✅ offline, multi-engine | **you're ahead** | — | — |
| Word chunking — 1–3 words per cue | ✅ | ✅ **four layout modes, incl. Burst (3 words)** — `crates/concat/ui/dialogs/captions.slint:78`, `crates/concat-project/src/captionmap.rs:597–627` | **not a gap** — reads badly only while word timing is off | — | — |
| Word-level timing accuracy | ✅ good | 🟡 ~±200–500 ms | **real gap** | S | ★★★ **first** |
| Animated caption presets (karaoke, pop, typewriter) | ✅ large library | 🟡 renderer wired — one highlight per heard word (`concat-host/src/titles.rs:367–390`); **0 of 32 presets** set `highlight_color`; motion has no schema | big visual gap | M | ★★★ |
| Caption style library (fonts, stroke, box, position) | ✅ 100s | 🟡 **32 presets ship — all title looks**; caption-shaped ones: none | medium | **S** | ★★★ |
| Apply style to whole track in one click | ✅ | ✅ "Apply to all captions", **on by default**, scoped per caption track (`crates/concat/src/studio.rs:963`) | — | — | — |
| Split / merge clips at the playhead | ✅ | ✅ `Command::SplitClips` / `MergeClips` (`crates/concat-project/src/commands/mod.rs:513`, `:574`) | — | — | — |
| **Enter splits a caption at the caret** into a new cue | ✅ CapCut · ✅ VTS | ❌ Enter is unwired (`crates/concat/ui/primitives/modal.slint:634` binds `edited` only); inline edit writes text and stops (`crates/concat/src/panes/sections.rs:680`) | **real gap** | M | ★★★ |
| Waveform on clips | ✅ | ✅ **already drawn** — a sound band inside video clips (`ui/timeline/lanes.slint:184`), plus the media pane and the speech dialog | — | — | — |
| Waveform under the **caption** row + draggable word edges | ✅ | ❌ nothing drags a *word* edge (grep: no word-drag code) | medium | M | ★★★ |
| Edit caption text inline | ✅ | ✅ | — | — | — |
| Import .srt / .vtt | ✅ | ✅ **yours only** | — | — | — |
| Export .srt / .vtt | ✅ | ✅ **yours only** | — | — | — |
| Multi-language caption tracks | 🟡 translate track | ✅ **yours only** | — | — | — |
| Captions stay in sync through cuts | ❌ | ✅ **yours only** | — | — | — |
| Delete word → video cuts to match (text-based editing) | ❌ | ❌ | **nobody has it** | M | ★★★ |
| Filler word removal | ✅ English-centric | ❌ | medium | S | ★★ **Indonesian version = your edge** |
| Speaker labels / diarisation | ✅ Pro | ❌ | big | L | ★ |
| Translate captions to another language | ✅ many | 🟡 **translate-to-English ships** — Whisper's translate task (`ui/dialogs/captions.slint:57`, `concat-speech/src/transcribe.rs:248`); every other target language: none | medium | M | ★ |
| Burn captions / soft subs / sidecar | ✅ | ✅ | — | — | — |
| 130+ languages | ✅ | 🟡 Whisper supports ~99 locally | not a real gap | — | — |

**Note:** CapCut's auto-caption is cloud-only [1](https://caption-x.com/blog/how-to-add-captions-capcut). Yours isn't. That row is the whole reason kconcat exists.

---

## 2. Audio

| Feature | CapCut | kconcat | Gap | Effort | Worth it? |
|---|---|---|---|---|---|
| Voice cleanup (denoise / enhance / level) | ✅ | ✅ | — | — | — |
| Voice changer / robot / telephone | ✅ | ✅ | — | — | — |
| Text-to-speech | ✅ cloud | ✅ local (3 engines) | — | — | — |
| Voice cloning | ✅ Pro | ✅ local | **you're ahead** | — | — |
| Auto silence removal | ✅ desktop | ✅ **plus captions follow the cut** | you're ahead | — | — |
| Auto ducking (music under speech) | ✅ | ❌ | small | S | ★★ |
| Auto beat sync (edit to music) | ✅ | ❌ | large | L | ★ (skip — not your content type) |
| Music / SFX library | ✅ 300k tracks | ❌ | n/a | — | ✗ licensing business |
| Vocal isolation (split voice from music) | ✅ Pro | ❌ | large | L | ★ |
| Manual keyframes on volume | ✅ | ✅ | — | — | — |
| Loudness normalisation to platform targets | 🟡 implicit | 🟡 `loudnorm` runs only in the transcription path (`concat-speech/src/transcribe.rs:484`); nothing at export | small | S | ★★★ |

---

## 3. Picture / AI

| Feature | CapCut | kconcat | Gap | Effort | Worth it? |
|---|---|---|---|---|---|
| Background removal | ✅ cloud | ✅ local (3 models, incl. hair detail) | **you're ahead** | — | — |
| Auto reframe 16:9 → 9:16 | ✅ **Pro only since Apr 2026** | ❌ | medium | M | ★★★ **biggest single win** |
| Manual crop + masks | ✅ | ✅ | — | — | — |
| Video upscaling | ✅ Pro | 🟡 `vision/enhance.rs` exists | medium | M | ★ |
| Video stabilisation | ✅ | ❌ | large | L | ★★ if you film handheld |
| Camera tracking | ✅ Pro | ❌ | large | L | ★ |
| Remove flicker | ✅ Pro | ❌ | medium | M | ★ |
| Chroma key / green screen | ✅ | ✅ | — | — | — |
| AI colour correction | ✅ Pro | ❌ | medium | M | ★ |
| Retouch / beauty | 🟡 removed in US 2024 | ❌ | — | — | ✗ |
| AI text remover (paint out text in footage) | ✅ | ❌ | medium | M | ★ |
| AI video generation | ✅ Magic Studio | ❌ | n/a | — | ✗ never build in-house |

---

## 4. Editing & timeline

| Feature | CapCut | kconcat | Gap | Effort | Worth it? |
|---|---|---|---|---|---|
| Multi-track, multi-timeline | ✅ | ✅ | — | — | — |
| Split, trim, ripple, merge | ✅ | ✅ | — | — | — |
| Magnetic timeline | ✅ | ✅ | — | — | — |
| Speed control / ramping | ✅ | ✅ (`core/retime.rs`) | — | — | — |
| Freeze frame, reverse | ✅ | ✅ | — | — | — |
| Keyframes + curve editor | ✅ | ✅ | — | — | — |
| Blend modes, crop, flip | ✅ | ✅ | — | — | — |
| Proxies for 4K smooth scrubbing | ✅ | ✅ | — | — | — |
| Screen / webcam recording | ✅ | 🟡 microphone voiceover only (`concat-host/src/record.rs:4`); no screen or webcam capture | small | S | ★★ |
| Transition library | ✅ 100s | ✅ 123 effect packages | 🟡 depth | M | ★★ |
| Filter / LUT library | ✅ 100s | ✅ (`.cube` support) | 🟡 depth | M | ★★ |
| Stickers, emojis, overlays | ✅ huge | 🟡 minimal | large | L | ✗ |
| Multi-project / batch export | ✅ | 🟡 the API has jobs (`concat-server/src/lib.rs:20`); nothing in the export UI | medium | M | ★★ |

---

## 5. Assets, export, platform (the parts not to chase)

| Feature | CapCut | kconcat | Worth it? |
|---|---|---|---|
| Template library (500k+) | ✅ | ❌ | ✗ — licensing business, not engineering |
| Font library | ✅ | 🟡 system fonts + user files | ✗ |
| Cloud storage 100 GB | ✅ | ❌ | ✗ — opposite of your identity |
| Cross-device sync | ✅ | ❌ | ✗ |
| Direct share to TikTok/IG | ✅ | ❌ | ★ manual upload is fine |
| Team collaboration / brand kit | ✅ | ❌ | ✗ |
| Export 4K / HEVC / AV1 / 10-bit | ✅ Pro | ✅ free | — |
| Export presets per platform | ✅ | ❌ | ★★★ cheap, high value |
| Scripting / API / MCP | ❌ | ✅ **yours only** | — |

---

## 6. The shortlist — 12 things worth adding, in order

Judged for *your* use, not for beating CapCut.

| # | Feature | Why | Effort |
|---|---|---|---|
| 1 | **Word-timing snapping** | Everything else (karaoke, pop, transcript editing) is built on it. Currently ~±200–500 ms | S |
| 2 | **Caption style library** — 10–15 presets | The visible thing you'd miss from CapCut. System already exists (`text-presets/`), just ship files | S–M |
| 3 | **Enter splits a caption at the caret** into a new cue | In CapCut (and VTS) Enter cuts the caption at the cursor and the tail becomes a new cue with its own timing. kconcat's Enter is unwired (`crates/concat/ui/primitives/modal.slint:634`): the caret is readable, the row editor is not. Daily caption editing — the friction you hit most often | M |
| 4 | **Auto reframe 9:16** | RVM masks already exist; smooth centroid → crop keyframes. CapCut charges Pro for this | M |
| 5 | **Waveform nudge for cues** | Lets you fix the last 5% of timing by hand in seconds | M |
| 6 | **Platform export presets** (TikTok/Reels/Shorts) + −14 LUFS | Removes the last reason to open CapCut for a simple export | S |
| 7 | **Indonesian ASR tuning** | Test Whisper large-v3 + Indonesian fine-tunes against your own voice/audio. CapCut's cloud is generic; yours can be tuned to how you actually speak | S |
| 8 | **Indonesian filler-word removal** — "eee", "anu", "kayak", "gitu", "apa ya" | CapCut's filler removal is English-shaped. This is a genuine local edge, and it's a small list | S |
| 9 | **Transcript editing** (delete word → cut) | Nobody has it. Your `captionmap.rs` is 80% of the machinery | M |
| 10 | **Auto ducking** for music beds | Small, and you'll notice its absence immediately | S |
| 11 | **Batch/queue export** | You'll want it the first time you make 5 shorts in a sitting | M |
| 12 | **Stabilisation** | Only if you film handheld. Otherwise skip | L |

*(#3 replaced on 16 Oct 2026: the earlier #3, "one-click apply style to track",
turned out to already ship and default on — see the corrections block above. The
count stays 12.)*

**Explicitly skip forever:** templates, music/sticker/font libraries, cloud AI, collaboration, mobile parity, 130 languages, colour grading depth. Every one of those is either a licensing business, an infra business, or a GPU-spend race — and none of them serve *your* workflow.

---

## 7. The money part, honestly

You mentioned you're jobless right now and thinking about this as a possible opportunity. Two things you should know before you build a business plan around kconcat.

**(a) kconcat can't be sold as closed software.** Concat is **AGPL-3.0**, and that's viral — your added modules are derivative work. If you ever *distribute* a build (sell it, give it to a client, put it on a store), you must offer the source to whoever receives it, under the same licence. You can absolutely *charge* money — AGPL doesn't stop you selling — but buyers can legally redistribute it. So a paid-binary business is structurally fragile.

Three realistic paths if money is the goal:
- **Sell the service, not the software.** Use kconcat as *your* tool to offer captioning / short-form editing / batch processing to local creators, UMKM, agencies. You keep the tool private, clients pay for output. Zero licensing problem, revenue starts sooner, and it directly fits "I don't want to be CapCut's rival."
- **If you want to sell software, change the stack.** Build on permissive deps (whisper.cpp, FFmpeg, wgpu) without forking AGPL Concat. More work up front, but then it's yours to license. This is a real fork in the road, and it's worth deciding *before* writing more kconcat code.
- **Keep it personal and free.** Also completely valid. Not every project has to pay rent.

Read `LICENSE-EXCEPTIONS.md` in the Concat tree before any commercial move — it spells out what their plugin exception does and doesn't cover.

**(b) Context on the price you're comparing against.** CapCut Pro officially runs about **Rp 129.000/month** or **~Rp 1.099.000–1.249.000/year** in Indonesia [1](https://www.literally-apps.id/blog/capcut-pro-murah-2026) [4](https://tekno.kompas.com/read/2023/06/05/15150027/cara-berlangganan-capcut-pro-keunggulan-serta-harganya-). There's a whole grey market reselling shared accounts at Rp 25.000–35.000/month [1](https://www.literally-apps.id/blog/capcut-pro-murah-2026) — which tells you the *real* willingness to pay is around a third of the official price. That's a market signal, and also a warning: at that price point, software alone is a hard living.

**The unsentimental summary:** your *skill* (Rust, agents, video pipelines, Whisper, FFmpeg, 100k-line forks that build) is worth more on the market right now than this *product* is. That's not a reason to stop — it's a reason to let the project serve two purposes: a tool that makes your own content work easier, **and** a portfolio piece that gets you paid work. If it later becomes a product, great. Don't starve waiting for it.

---

## 8. What I'd like to know to go deeper

The right build order depends on what you actually edit — and I've been assuming. Tell me and I'll rewrite the shortlist against your real workflow instead of a generic one.
