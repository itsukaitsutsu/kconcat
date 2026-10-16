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

## 1. Captions & subtitles — your home turf

| Feature | CapCut | kconcat | Gap | Effort | Worth it? |
|---|---|---|---|---|---|
| Auto-caption (speech → text) | ✅ cloud only | ✅ offline, multi-engine | **you're ahead** | — | — |
| Word-level timing accuracy | ✅ good | 🟡 ~±200–500 ms | **real gap** | S | ★★★ **first** |
| Animated caption presets (karaoke, pop, typewriter) | ✅ large library | 🟡 engine supports per-word reveal, no presets shipped | big visual gap | M | ★★★ |
| Caption style library (fonts, stroke, box, position) | ✅ 100s | 🟡 preset system exists, empty | medium | S–M | ★★★ |
| Apply style to whole track in one click | ✅ | ❌ | medium | S | ★★★ |
| Split / merge caption cues by hand | ✅ fast | 🟡 | small | S | ★★ |
| Waveform view to nudge cue timing | ✅ | ❌ | medium | M | ★★★ |
| Edit caption text inline | ✅ | ✅ | — | — | — |
| Import .srt / .vtt | ✅ | ✅ **yours only** | — | — | — |
| Export .srt / .vtt | ✅ | ✅ **yours only** | — | — | — |
| Multi-language caption tracks | 🟡 translate track | ✅ **yours only** | — | — | — |
| Captions stay in sync through cuts | ❌ | ✅ **yours only** | — | — | — |
| Delete word → video cuts to match (text-based editing) | ❌ | ❌ | **nobody has it** | M | ★★★ |
| Filler word removal | ✅ English-centric | ❌ | medium | S | ★★ **Indonesian version = your edge** |
| Speaker labels / diarisation | ✅ Pro | ❌ | big | L | ★ |
| Translate captions to another language | ✅ | ❌ | medium (via API/agent) | M | ★ |
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
| Loudness normalisation to platform targets | 🟡 implicit | 🟡 `loudnorm` exists, no presets | small | S | ★★★ |

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
| Screen / webcam recording | ✅ | 🟡 `record` exists in host | small | S | ★★ |
| Transition library | ✅ 100s | ✅ 123 effect packages | 🟡 depth | M | ★★ |
| Filter / LUT library | ✅ 100s | ✅ (`.cube` support) | 🟡 depth | M | ★★ |
| Stickers, emojis, overlays | ✅ huge | 🟡 minimal | large | L | ✗ |
| Multi-project / batch export | ✅ | 🟡 jobs exist | medium | M | ★★ |

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
| 3 | **One-click apply style to track** | CapCut's most-used caption action | S |
| 4 | **Auto reframe 9:16** | RVM masks already exist; smooth centroid → crop keyframes. CapCut charges Pro for this | M |
| 5 | **Waveform nudge for cues** | Lets you fix the last 5% of timing by hand in seconds | M |
| 6 | **Platform export presets** (TikTok/Reels/Shorts) + −14 LUFS | Removes the last reason to open CapCut for a simple export | S |
| 7 | **Indonesian ASR tuning** | Test Whisper large-v3 + Indonesian fine-tunes against your own voice/audio. CapCut's cloud is generic; yours can be tuned to how you actually speak | S |
| 8 | **Indonesian filler-word removal** — "eee", "anu", "kayak", "gitu", "apa ya" | CapCut's filler removal is English-shaped. This is a genuine local edge, and it's a small list | S |
| 9 | **Transcript editing** (delete word → cut) | Nobody has it. Your `captionmap.rs` is 80% of the machinery | M |
| 10 | **Auto ducking** for music beds | Small, and you'll notice its absence immediately | S |
| 11 | **Batch/queue export** | You'll want it the first time you make 5 shorts in a sitting | M |
| 12 | **Stabilisation** | Only if you film handheld. Otherwise skip | L |

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
