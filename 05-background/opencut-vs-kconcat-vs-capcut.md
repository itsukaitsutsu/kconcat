# OpenCut vs kconcat vs CapCut — an honest comparison

*Checked on 16 October 2026. Facts below were pulled live from the GitHub API, the repos' own code, and their CI runs — not from marketing pages.*

---

## 1. The 30-second answer

| Question | Answer |
|---|---|
| **Best editor for most people, today** | **CapCut** — by a wide margin on polish, templates, AI and mobile. You pay for it with your data, a subscription tier and paywall creep. |
| **Best free + private + actually shipping** | **Concat** (`jub0t/Concat`) — a native Rust editor, v0.2.7, ~39k downloads on that release, no watermark/account/upload. Still beta. |
| **Best "open it in a browser, no install"** | **OpenCut classic** (`opencut.app`) — MIT, local-first, but a *basic* editor. The repo you linked is a rewrite shell, not the editor. |
| **Where does kconcat fit?** | It isn't a competitor to CapCut — it's **a personal fork of Concat** with a genuinely interesting feature set (silence-trim + linked captions). 22 commits old, 0 stars, 0 releases, red CI, no installable build. Excellent lab. Not a product yet. |

**The single most important thing to know:** `itsukaitsutsu/kconcat` is not an independent editor. Its own README says the code was *"Ported from `itsukaitsutsu/video-trim-studio` into `jub0t/Concat`"*, its `ARCHITECTURE.md` is Concat's, and its license is Concat's AGPL-3.0. So the real comparison is **OpenCut vs Concat vs CapCut**, with kconcat sitting on the Concat branch of that tree.

---

## 2. What each one actually is (context most comparisons miss)

### CapCut (ByteDance) — the commercial baseline
- Closed-source, freemium. Free tier: capped at 1080p, watermarks on some template/effect exports, AI features limited per day, ~1 GB cloud.
- Pro: roughly **$9.99/mo or $89.99/yr** through CapCut's site, up to **$19.99/mo / $179.99/yr** via app stores, depending on region and channel.
- Strongest assets: 500k+ templates, 300k+ licensed tracks, mobile apps that are genuinely better than the desktop app, best-in-class auto-captions/background removal, and the "make a Reel in 3 minutes" workflow nobody else matches.
- Structural weaknesses: paywall creep (features that were free now aren't), AI features are cloud-processed, and ByteDance ownership. PCMag's 2026 review headline is blunt: *"Great for Social Videos, Risky for Personal Data"*, noting EU regulatory action over data transfers to China. It's banned in India. Mobile/web require a login; desktop doesn't.

### OpenCut — two different things wearing one name
| | The repo you linked (`OpenCut-app/OpenCut`) | The editor people actually use |
|---|---|---|
| What | Ground-up **rewrite** (Rust core + GPUI desktop + plugin API + MCP + headless) | **opencut.app**, the "classic" Next.js editor, source archived at `opencut-app/opencut-classic` |
| Stars | 94.7k | 278 (archived May 2026) |
| Status | Last commit **24 Sep 2026** (~3 weeks idle), 379 open issues, **"not set up to take outside contributions yet"**, no release timeline | Live, v0.3.0 (Apr 2026), no longer maintained |
| Feature depth | Promises: Editor API, plugins, mobile, MCP, scripting tab | Reality: timeline, multi-track, masks, keyframes, speed, canvas background, effects. **No auto-captions, no AI suite, no template library.** Filters/transitions were still "expected by v0.4.0". |

So OpenCut's 94.7k stars measure *excitement*, not shipped capability. For an actual edit today, classic OpenCut is closer to "a clean browser timeline" than to "a CapCut alternative".

### Concat (`jub0t/Concat`) — the real open-source CapCut alternative
- **6.6k stars, 716 forks, 108 open issues, AGPL-3.0, v0.2.7 (13 Oct 2026)**, ~39k downloads on that release. Windows/macOS/Linux/Android (iOS sideloaded), setup installers published.
- ~92k lines of Rust in 16 crates: exact rational time, arena timeline, wgpu GPU compositor, FFmpeg linked in, whisper.cpp + sherpa-onnx compiled in, models downloaded locally.
- Ships what CapCut charges for: **auto-captions (local Whisper), TTS + voice cloning, background removal, keyframes + curve editor, 170+ effects/filters/transitions/text animations, multi-track, titles, voice cleanup, H.264/HEVC/AV1 up to 4K60 10-bit**.
- **JSON-RPC + gRPC + MCP API and a CLI** — every edit the GUI can make, a script or an agent can make. This is a real, shipped differentiator.
- Caveats: it's **beta** ("it works, and it still has edges"), and its own CI is currently red on some jobs, so "green build" isn't a promise either project can make.

### kconcat (`itsukaitsutsu/kconcat`) — a lab, not a product (yet)
- Created **13 Oct 2026**, **22 commits**, **0 stars**, **0 releases**, **no build artifacts**, **2 open dependabot issues**.
- Built on an upstream snapshot from **12 Oct 2026**; upstream is already **36 commits ahead** (115 files changed).
- Windows-centric build: a pile of `.bat`/`.ps1` scripts, `FFMPEG_DIR` pointing at a BtbN `win64-gpl-shared` build, and a ~22 GB `target/` for the window crate.

---

## 3. Scorecard

Legend: ✅ strong · 🟡 partial/coming · ❌ missing · ⚠️ with caveats

| | **CapCut** | **OpenCut (classic)** | **Concat** | **kconcat** |
|---|---|---|---|---|
| Price | Free tier + $9.99–19.99/mo | Free (MIT) | Free (AGPL-3.0) | Free (AGPL-3.0 fork) |
| Watermark / account | ⚠️ watermark on some free exports; login on mobile/web | ✅ none / no account | ✅ none / no account | ✅ none / no account |
| Runs | Mobile, desktop, web | **Browser only** | Native desktop + Android | Native desktop (build from source) |
| Offline / privacy | ⚠️ AI is cloud-side; ByteDance data practices | ✅ in-browser | ✅ fully local, models local | ✅ fully local |
| Auto-captions | ✅ (limited free) | ❌ | ✅ local Whisper | ✅ + engine picker (faster-whisper / whisper.cpp Vulkan / CPU) |
| Silence auto-cut | ✅ desktop (some sources say Pro-gated) | ❌ | ✅ (silence detection module) | ✅ **+ captions stay in sync with the cut** |
| Templates / asset library | ✅ 500k+ | ❌ | 🟡 templates supported | 🟡 |
| Effects | ✅ huge | 🟡 basic, growing | ✅ 170+ | ✅ (inherited) |
| TTS / voice cloning | ✅ cloud | ❌ | ✅ local | ✅ (inherited) |
| Background removal | ✅ | ❌ | ✅ | ✅ (inherited) |
| Multi-track | ✅ | ✅ (basic) | ✅ multi-track, multi-timeline | ✅ |
| Max export | 1080p free / 4K Pro | browser resolutions | **4K60 10-bit, H.264/HEVC/AV1** | same as Concat **+ stream-copy remux** |
| Automation / API | ❌ (manual only) | 🟡 MCP *planned*, not shipped | ✅ JSON-RPC, gRPC, MCP, CLI — shipping | ✅ inherited |
| Installability for a normal person | ✅ app stores | ✅ URL | ✅ installers | ❌ build from source |
| Green CI / release discipline | ✅ | 🟡 stale | ⚠️ beta, red jobs | ❌ red CI, no releases |
| Community / contributor base | huge | 101 contributors | 24+ contributors, active PRs | 1 person + an AI agent |

---

## 4. What kconcat actually adds — and it's not nothing

Against its upstream snapshot, kconcat has **~8,300 added and ~2,600 removed lines** of Rust/Slint (excluding locales), touching 100+ files. The substantive work:

1. **`subtitle.rs`** — tolerant SRT/VTT reader + writer (BOM, CRLF, cue settings, inline tags, Thai/Arabic/Japanese round-trips). 24 tests.
2. **`sections.rs` / `sectionlist.rs`** — the arithmetic of trimming by content: silence spans → merge/complement → tiling a clip into caption/silence/other sections you can tick. A test asserts that "tick every non-silence section" and "complement of the silences" agree.
3. **`captionmap.rs`** — word-level caption remapping through a cut: split, join, clip at the boundary, preserve track/style; the mechanism that makes **subtitles move with the video when you delete a silence**.
4. **UI**: a dockable `Sections` pane with filters, a section strip under the timeline, timeline↔sections scroll sync, caption import pane, multi-track bilingual caption stacking, `+ Add as new caption track`.
5. **Export**: `Same as copy` stream-copy remux for trimmed timelines, with **synchronized `.srt` sidecars** and optional `mov_text` soft subs; a "Burn captions into the video" switch that targets captions only (not titles).
6. **Speech**: engine selector (faster-whisper vs whisper.cpp Vulkan/CPU), device + compute-type + VAD controls, live ETA, and post-transcription VAD refinement on the media clock to avoid drift.
7. **i18n**: 24 locales kept in sync.

That combination — **local transcription + silence cutting + captions that survive the cut, in a fully offline native editor** — is something neither CapCut nor OpenCut does cleanly. CapCut's silence removal bakes into the export and doesn't carry subtitles with the cut; OpenCut has no captions at all.

### The honest counter-list
- It's a **fork of a fast-moving beta**, already 36 commits behind. Upstream has since split `studio.rs` into a `studio/` module, added `pace.rs`, `crash.rs`, `consolidate.rs`, `machine.rs` — kconcat carries none of it, so merging back gets harder every day.
- **No release, no CI-on-green, no install path.** The last completed CI run failed including the Windows `cargo nextest run --workspace` step (the report artifact exists but needs auth to read); two other jobs died earlier on environment setup. Upstream's CI also has red runs, so this is partly inheritance, partly unverified local fixes.
- Their own README-INSTALL admits only `concat-project` and `concat-media` were built/tested in the sandbox — "the Slint changes are not compiled" in that environment; the claimed 251/443 test numbers come from partial builds.
- **Repo hygiene**: a root littered with 20+ `CHECKPOINT-*.md` / `README-PHASE-*.txt` files plus stray junk (`src/]`, `src/nulrobocopy`, a file named `Python 3.14.7`, `not`). Commit messages read `more bug more fug`, `tryna`, `cargo`. This is what an AI-agent workflow leaves behind if you don't curate it — and it signals "not ready for users or contributors".
- License: forking AGPL-3.0 is fine, but keep `LICENSE`, `LICENSE-EXCEPTIONS.md` and attribution intact, and remember §13: if you ever *host* it as a service, you must publish your modified source.

---

## 5. Verdict

**If you're a normal user who wants to edit a video tonight:** CapCut. Nothing free matches its polish, templates and mobile app; Concat is the credible free substitute if you'll accept beta bugs and no template library; OpenCut classic is fine for basic browser cuts; kconcat is not an option (there's nothing to install).

**If your priority is free + private + no subscription:** **Concat** — it's the only one shipping CapCut-class features (auto-captions, TTS, background removal, 4K) with a real API on top, at 0 cost. OpenCut is the winner only if "runs in a browser" is the requirement, because its feature set is much thinner and its main repo is mid-rewrite.

**If your priority is automation / agents / pipelines:** Concat, unquestionably — JSON-RPC + gRPC + MCP + CLI are shipped today; OpenCut's MCP server is a roadmap bullet.

**About kconcat specifically:** it is *not* "better or worse than CapCut" — it's a research fork. Measured as engineering, the VTS integration is the most interesting caption-aware trimming work in this comparison, and it's implemented as *pure, testable modules*, which is exactly the shape upstream would accept. Measured as a product, it's behind OpenCut (which at least you can open in a browser) and far behind Concat (which you can install).

### Highest-leverage next moves for kconcat
1. **Turn the three pure modules into PRs against `jub0t/Concat`** — the README already ships a signed-off patch and instructions for amending authorship. 6.6k stars of users are upstream; 0 are in the fork.
2. **Get CI green at HEAD** and publish at least one build artifact; "tests pass" claims are worthless to a user who can't install it.
3. **Rebase onto current upstream** before the divergence cost grows past a weekend's work.
4. **Replace the checkpoint pile with a `CHANGELOG.md` and delete the junk files.**
5. If the fork must live on, write a real README: *"kconcat = Concat + linked captions + silence trim"*, with a screenshot and a 60-second quickstart.

---

## 7. Caption-editing deep dive (added after a follow-up question: "is kconcat more powerful for caption editing?")

The answer splits cleanly in two, because "editing captions" means two different jobs.

### A. Captions as *data* (timing, files, sync with the cut) — **kconcat wins, and it isn't close**

Verified by reading the source of upstream `jub0t/Concat` on 16 Oct 2026:

| Capability | CapCut | OpenCut | Concat (upstream) | kconcat |
|---|---|---|---|---|
| Auto-transcribe to captions | ✅ (Pro-gated more each year) | ❌ | ✅ local Whisper | ✅ + Vulkan/CPU engine picker, VAD, ETA |
| **Import .srt / .vtt** | ✅ | ❌ | ❌ **no subtitle module at all** | ✅ `subtitle.rs` (BOM, CRLF, cue settings, inline tags, 24 tests) |
| **Export .srt / .vtt** | ✅ | ❌ | ❌ | ✅ sidecar on export + optional `mov_text` soft subs |
| **Per-word timings kept as data** | 🟡 word-level ASR, but timings aren't a portable artifact | ❌ | ❌ | ✅ `captionmap.rs` — words are a first-class field |
| **Captions remap when you cut/trim** | ❌ cuts are baked; you re-sync by hand | ❌ | ❌ | ✅ split cues at the cut, rejoin halves, clip straddling words |
| Delete a silence → captions follow | ❌ | ❌ | ❌ | ✅ sections + captionmap work as one operation |
| Multi-language caption lanes that don't overwrite each other | 🟡 translate track, stacked manually | ❌ | ❌ | ✅ own track + vertical stacking, `+ Add as new caption track` |
| Caption-only export switch (titles stay) | ✅ | ❌ | ❌ | ✅ burn toggle scoped to captions |
| Batch/scriptable caption edits | ❌ manual only | ❌ | ✅ JSON-RPC/gRPC/MCP (inherited) | ✅ same API |
| Works fully offline, no usage cap | ❌ | ✅ | ✅ | ✅ |

I checked this directly: a case-insensitive search for `srt` / `webvtt` / `.vtt` across every `.rs` and `.slint` file in current upstream Concat returns **zero matches**. The caption sheet there generates captions from a clip's audio or a typed script, drops them on the timeline as title clips, and that's the end of it — no subtitle file in, no subtitle file out, no word-level remap. So on caption *plumbing*, kconcat doesn't just beat CapCut and OpenCut; it beats its own upstream.

### B. Captions as *styling and speed* — **CapCut still wins, comfortably**

- **Animated preset library**: karaoke highlight, pop-by-word, bounce, typewriter, glow, multiline, trending — one click applies a style to the whole track [1](https://pixflow.net/blog/ai-automatic-captions-subtitles/).
- **55–130+ languages**, auto language detection, **built-in translation** into a second language track, **automatic filler-word removal** ("um", "uh"), speaker-ID captions, AI text remover [4](https://verbit.ai/resources/10-best-ai-video-captioning-tools-for-2026-2027-top-picks-for-media-business-video-creators/).
- **Inline editing, split/merge buttons** built for fast cleanup, on desktop *and* phone, with ~92–95% accuracy out of the box.
- **The catch is the trend, not the feature**: CapCut's own help centre says auto-captions now include Pro-only functionality, and Auto Reframe left the free plan in April 2026; Pro is $19.99/mo or $179.99/yr in the US, and it varies by region [3](https://cut.pro/en/blog/capcut-clips-or-clipping-tool-2026).

The Concat family has the machinery for word-level reveal (the engine bakes a per-word "reveal order" texture, `concat-core/src/shader.rs`) and a text inspector for font/stroke/shadow/plate — but there is no CapCut-style one-click animated caption library, no translation, no filler-word stripping.

### So, is kconcat "a bit powerful" for captions?
- **As a caption engine: yes.** It's the only one of the three where captions are structured edit data rather than text boxes — SRT/VTT both directions, word timings preserved, captions that ripple and re-time when you cut, multi-language lanes, offline and scriptable.
- **As a caption editor you'd recommend to a friend: no, not yet.** No release to install, red CI at HEAD, no animated style library, and its own change notes list unfixed UX problems (e.g. scrolling the Sections list can zoom the timeline). "More powerful data model" ≠ "more usable app".
- **The one-line version:** CapCut makes captions *look* better; kconcat makes captions *behave* better.

---

## 6. Data behind this (checked 16 Oct 2026)

| Metric | CapCut | OpenCut | Concat | kconcat |
|---|---|---|---|---|
| Stars | — | 94,723 | 6,658 | 0 |
| Forks | — | 9,321 | 716 | 0 |
| Open issues | — | 379 | 108 | 2 |
| Created | — | 22 Jun 2025 | 25 Aug 2026 | 13 Oct 2026 |
| Last push | — | 24 Sep 2026 | 15 Oct 2026 | 15 Oct 2026 |
| Latest release | — | v0.3.0 (Apr 2026, classic) | v0.2.7 (13 Oct 2026, ~39k downloads) | none |
| Language / licence | closed | TypeScript / MIT | Rust / AGPL-3.0 | Rust / AGPL-3.0 |
| CI at HEAD | — | n/a | red on some jobs | red (Windows test step + env-setup jobs) |

### Sources
- OpenCut repo + README: https://github.com/OpenCut-app/OpenCut
- OpenCut classic README (archived): https://github.com/opencut-app/opencut-classic
- OpenCut status/rewrite reporting: https://www.opensourcealternatives.to/item/opencut · https://note.com/ai_driven/n/n7e8a36e1d5f3 · https://dev.to/coddykit/opencut-the-open-source-capcut-alternative-72669-github-stars-2f20
- Concat repo + README + ARCHITECTURE.md: https://github.com/jub0t/Concat · code tour: https://pyshine.com/Concat-Open-Source-Capcut-Replacement-Source-Tour/
- kconcat repo, README, ARCHITECTURE.md, CHECKPOINT-*.md: https://github.com/itsukaitsutsu/kconcat
- CapCut pricing/free-tier limits: https://www.toolsurf.com/capcut-pricing-2026-official-plans-costs-free-vs-pro-comparison-2026-plans-features-best-deals-compared/ · https://www.agencyhandy.com/informative/capcut-pricing/ · https://www.eesel.ai/blog/capcut-pricing
- CapCut privacy/regulatory: https://au.pcmag.com/video-editing/89766/capcut
- CapCut silence removal: https://store.hollyland.com/blogs/creator-hub/remove-pauses-in-capcut · https://cutsio.com/blog/capcut-silence-remover-vs-cutsio-comparison
- Raw numbers: GitHub REST API (`/repos`, `/actions/runs`, `/releases`, `/compare`) queried 16 Oct 2026
