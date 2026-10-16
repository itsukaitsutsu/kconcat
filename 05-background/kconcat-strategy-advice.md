# kconcat: should you add AI to get closer to CapCut?

*Advice based on a source audit of both your tree (`e028a65`) and current upstream `jub0t/Concat` (16 Oct 2026).*

## The short answer

**Don't add AI. Add workflow.** You already have AI — a lot of it, inherited from upstream: local Whisper transcription, three TTS engines (Kokoro/Pocket/Chatterbox) with voice cloning, and a full segmentation stack (MediaPipe selfie, Robust Video Matting, IS-Net) for background removal. "Adding AI" would mean building the one thing you can't win — CapCut's cloud generative suite — while ignoring the four things you can build in a weekend that nobody else has.

**Let it be** is also wrong. Not because you need features, but because you have `0` releases and red CI. Nothing you build matters until someone can install it.

---

## 1. "AI" is three different things. Only one of them is yours.

| Tier | What it is | Examples | Verdict for you |
|---|---|---|---|
| **A. Local, small, deterministic** | Models that run on the user's machine in seconds, offline | Whisper ASR, forced alignment, VAD, TTS, voice cloning, matting, upscaling, denoise, scene/silence detect | **Doubles down. This is your moat.** Already in the tree — go deeper, not wider |
| **B. Local, generative** | Diffusion/LLMs on-device | Local LLM "cut me 3 shorts", small video models | **Watch.** Still too heavy for a laptop at usable quality. Revisit in a year |
| **C. Cloud, generative** | CapCut's Magic Studio | Image/video generation, style transfer, AI upscale, avatars | **Never build in-house.** Integrate via the API you already have |

### Why Tier C is a bad trade
- **You'd pay per use and cap your own users.** Your single best differentiator is *"no upload, no account, no cost, works on a plane."* In-house cloud AI destroys all four in one commit.
- **You can't outspend ByteDance on GPU inference.** This is a capex race, not a cleverness race.
- **You already have the right answer.** Upstream ships **JSON-RPC + gRPC + MCP**. Any user who wants generative AI can point Claude, or a fal.ai/Replicate pipeline, at your editor through MCP. That's *better* than an in-app AI button: it's opt-in, it's their key, it's their choice, and your app stays offline by default. OpenCut took fal.ai money and still hasn't shipped their MCP server — you have one.

**So: Tier A deep, Tier C by integration, Tier B on the horizon. Never Tier C in-house.**

---

## 2. What actually closes the CapCut gap (and what never will)

I checked what's in the tree. This list is grounded, not guessed.

### Already there — you don't need to build it
- **Text presets** (`concat/src/presets.rs`): a real, data-driven preset system — `text-presets/*.toml`, user-extensible, **shareable as a folder with its font inside**, per-language (`language = "Latin"`), ordering, sample glyph for the card. This is *exactly* the machinery a caption-style library needs, and it already exists. You are not starting from zero on caption styling.
- **Segmentation + tracking primitives** (`concat-vision/src/segment.rs`): RVM carries **recurrent state frame to frame "so an outline does not flicker."** That's the hard part of subject tracking, already solved and already ordered correctly.
- **The API**: every edit the GUI can make, a script can make.

### Missing, cheap, high-leverage (no AI required)
| Gap | Effort | Why it matters |
|---|---|---|
| **Text-based editing** — delete words in the transcript, video ripples to match | Medium — you're ~80% there | See §3. This is the flagship |
| **Caption-preset files** — 6–10 caption-shaped preset folders (the per-track "one-click apply" already ships, verified) | Small | CapCut's most-used caption feature; the preset system is already built — only the caption-shaped *data* is missing |
| **Auto-reframe** — 16:9 → 9:16 following the speaker | Small-ish | Feed the RVM mask centroid per frame, smooth it, drive crop keyframes. CapCut **removed this from its free plan in April 2026** |
| **Platform export presets** — TikTok/Reels/Shorts/YouTube with aspect, bitrate, loudness targets | Very small | There are none in the tree. Removes the last reason to open CapCut for a simple export |
| **Waveform word-boundary dragging** | Small | The correction UX that makes people *trust* the timing |

### Missing, expensive, and not worth it
- 500k templates, 300k licensed music tracks — that's a legal and per-asset cost centre, not an engineering problem.
- Cloud storage and cross-device sync.
- Generative AI suite.
- Mobile parity with CapCut's phone app. You have an Android target, but CapCut's mobile is a decade of polish and it's where its real moat lives. Don't fight there yet.

---

## 3. The one feature I'd build next: text-based editing

**Delete a word in the transcript → the video cuts to match.** That's Descript's entire business, it's what CapCut gestures at with filler-word removal, and it is the natural completion of what you already wrote.

Why it's yours to win:
- `captionmap.rs` already does **word-level remap through a cut** — split cues, rejoin halves, clip straddling words, preserve track and style. That's the *inverse* of what you need; the arithmetic is the same.
- `sections.rs` already tiles a clip into caption/silence/other and turns ticks into a trim set.
- `subtitle.rs` already round-trips timings.
- Upstream has **no** text-based editing at all — I grepped for it and `transcribe`/`delete_words`/`transcript_edit` return nothing, and neither does auto-reframe.

So the pipeline is: transcript → delete "um" or a whole sentence → command that ripples every lane → captions and audio stay in sync → export. You have every primitive. It's the flag you plant: **"the free, offline editor where the transcript *is* the timeline."**

And it's only possible *because* of the Level-1 word-timing fix from the previous doc. Which is the point: **that fix isn't cleanup, it's the unlock.** Ship timing first, then this.

---

## 4. Don't do these

1. **Don't add an AI chatbot sidebar to say you have AI.** Costly, worse than Claude/ChatGPT, and it makes the app heavier for everyone who doesn't want it. MCP already covers this market properly.
2. **Don't build a plugin marketplace** before you have users. You'd be building a distribution channel for zero participants.
3. **Don't chase CapCut's asset library.** It's a licensing business with a legal team behind it.
4. **Don't fork further.** Every commit you write on top of a 36-commits-stale base raises the merge cost.

---

## 5. The real blocker isn't AI, it's that kconcat isn't shippable

You have **0 releases, no installable build, red CI at HEAD**, a Windows-only build path of ~15 `.bat` scripts, and a repo root with 20+ `CHECKPOINT-*.md` files, a file named `Python 3.14.7`, and `src/nulrobocopy`. Before any feature conversation, that has to change:

1. **Green CI at HEAD** — the Windows `cargo nextest` step is failing; until it passes, "tests pass" is a claim, not a fact.
2. **One release artifact** — even just a zipped binary and a `README` that says "unzip, run". Zero users can exist without it.
3. **Delete the junk, add a `CHANGELOG.md`.** Agents left that mess; curate it.
4. **Decide what this *is*:**
   - **Upstream contributor** — send the three pure modules to `jub0t/Concat`, get them in front of 6.6k users, keep your fork as a dev branch. Lowest effort, highest reach. Your README already ships a signed-off patch.
   - **Hard fork with a maintainer plan** — rebase on a schedule, own releases, own roadmap, written contribution rules. Justified if you intend to own the caption/text-editing niche and upstream won't take it.
   - **Personal tool** — then skip all of the above, keep it on your machine, and just make the one workflow you use every day perfect.

   Doing none of these — drifting, 36 commits behind, with checkpoints piling up — is the only option that can't work.

---

## 6. If I had 30 days on kconcat

| Days | Work |
|---|---|
| 1–3 | Green CI at HEAD, delete the junk, tag `v0.1.0-kconcat`, publish one downloadable build, write a real README ("Concat + linked captions + silence trim") |
| 4–8 | **Word-onset snapping** (Level 1 from the sync doc) + tests. Measure: words >80 ms off, before and after |
| 9–12 | **Text-based editing** — transcript view, delete-word → ripple cut, using `captionmap` + `sections` |
| 13–16 | **Caption-preset files** — 6–10 shipped as `text-presets/` folders (per-track apply already ships). Plus platform export presets |
| 17–21 | **Auto-reframe** from the RVM mask centroid; smooth, drive crop keyframes |
| 22–25 | Open the upstream PRs: `subtitle.rs`, `sections.rs`, `captionmap.rs`, then the word-timing fix |
| 26–30 | Record one 3-minute demo: paste a 20-minute talking-head video, get a captioned, silence-trimmed vertical short, fully offline. That video is your README |

---

## 7. Bottom line

- **AI: yes, but only the kind you already have.** Deepen local perception (timing, alignment, tracking, matting). Never build cloud generation.
- **Closer to CapCut: yes, by owning one workflow end to end**, not by matching a feature list. Pick the talking-head → captions → silence → vertical short pipeline and make it the best in the world. That's the one CapCut keeps putting behind Pro.
- **Differentiate on the thing CapCut is structurally abandoning:** free, offline, no account — and *your timings are files you own* (SRT/VTT in and out, words preserved, remapped through every cut). No one else in this comparison can say that.
- **Do the unglamorous work first.** CI green, one release, one demo. A brilliant feature nobody can download has 0 users — which is exactly where you are today.
