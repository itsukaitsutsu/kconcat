# kconcat vs CapCut — Option B list, v2 (16 Oct 2026)

*Discussion only. No application code was read into a change, and nothing in the repo was modified.*
*Base: `itsukaitsutsu/kconcat` main, latest app-source commit `1aece2d`. Supersedes the Option B table in `kconcat-feature-gap.md`.*
*Goal: get closer to CapCut Pro for long-form podcast editing, with the output short-form clips. Reference: prof.zulys-style CapCut edits.*

---

## 0. Corrections made in this version

1. **Path prefix.** Every `crates/...` path in the feature-gap doc and the roadmap is relative to `src/`. The real path is `src/crates/...`. Example: `src/crates/concat/ui/dialogs/captions.slint`. The line numbers I checked against the captions dialog matched, so the prefix is the only thing that was wrong.
2. **Caption motion is confirmed absent.** The caption dialog (`src/crates/concat/ui/dialogs/captions.slint`, 866 lines) has no motion, animation, karaoke, or case controls. None of the 32 presets in `src/crates/concat/text-presets/` sets `highlight_color`, motion, or a case transform. The "pop", "slide", and "zoom" matches in `src/crates/concat-effects/packages/` are video effects and transitions, not caption motion.
3. **Caption-shaped presets are not zero.** Earlier I wrote "none caption-shaped." Two presets are close: `concat.bold-caption` (Montserrat 900, lowered position) and `concat.lower-third`. Neither sets a highlight or motion.
4. **Animation and B-roll reported by the owner.** Captions with animation and multi-clip / B-roll are reported to exist in kconcat. I have not verified either one. Both are treated as 🟡 untested and lower priority, as you asked.

---

## 1. The list, fixed and ranked for podcasts

Status key: ✅ exists · 🟡 partial or untested · ❌ missing · ✗ skip.
Effort: S = a weekend · M = a few weeks · L = months.

### Tier A — long-form podcast core

| # | Feature | CapCut Pro | kconcat | Effort | Notes |
|---|---|---|---|---|---|
| A1 | Word-level timing accuracy | ✅ | 🟡 ±200–500 ms | S–M | Blocks transcript editing and karaoke. Roadmap T1 → T2/T3. |
| A2 | Transcript editing (delete word → cut) | ❌ (gestured at) | ❌ | M | You're ~80% there (`captionmap.rs` does the inverse). Needs A1 first. Roadmap E1. |
| A3 | Speaker labels / diarisation | ✅ Pro | ❌ | L | Two-person shows need per-speaker names or colors. Scope to decide. |
| A4 | Loudness to −14 LUFS at export | 🟡 implicit | 🟡 `loudnorm` only before transcription | S | Roadmap V2 (loudness half). |
| A5 | Platform export presets (TikTok / Reels / Shorts / YouTube) | ✅ | ❌ | S | Export dialog has three quality cards only. Roadmap V2. |
| A6 | Auto-ducking (music under speech) | ✅ | ❌ | S–M | Intro music beds. Roadmap V3. |
| A7 | Indonesian filler removal (eee, anu, kayak, gitu, apa ya, terus) | ✅ English-shaped | ❌ | S | Depends on E1 machinery. Roadmap E2. |
| A8 | Enter splits a caption at the caret | ✅ CapCut · ✅ VTS | ❌ | M | Roadmap E4. |
| A9 | Caption style presets (caption-shaped) | ✅ 100s | 🟡 2 close, 0 full | S–M | Roadmap C2. |
| A10 | Karaoke highlight wired to a preset | ✅ | 🟡 renderer works, 0 presets set it | S | Roadmap C3. |
| A11 | Batch / queue export UI | ✅ | 🟡 API jobs only | M | Roadmap V4. |
| A12 | Screen / webcam recording | ✅ | 🟡 microphone only | S | Matters for remote-guest podcasts. |

### Tier B — short-clip extraction (podcast → 45–90 s vertical)

| # | Feature | CapCut Pro | kconcat | Effort | Notes |
|---|---|---|---|---|---|
| B1 | Auto-reframe 16:9 → 9:16 | ✅ Pro since Apr 2026 | ❌ | M | RVM masks exist. Roadmap V1. Biggest win for clip output. |
| B2 | Multi-clip / B-roll | ✅ | 🟡 reported to exist, **untested** | — | Verify before planning. |

### Tier C — caption look (CapCut-style)

| # | Feature | CapCut Pro | kconcat | Effort | Notes |
|---|---|---|---|---|---|
| C1 | Uppercase / case transform in style | ✅ | ❌ | S (per roadmap C4) | Preset can uppercase without changing source text. |
| C2 | Caption motion (pop / typewriter / word-reveal) | ✅ | ❌ confirmed absent | M | Blocked on motion-schema decision (roadmap C5). |
| C3 | Waveform under caption row + word-edge drag | ✅ | ❌ | M | Roadmap E3. |
| C4 | Animation (reported as existing) | ✅ | 🟡 **unverified** | — | Owner reports it; repo check found no caption motion. Needs a visual check in the app. |

### Tier D — later / lower priority

| # | Feature | Status | Effort | Notes |
|---|---|---|---|---|
| D1 | Vocal isolation | ❌ | L | Helps noisy recording rooms. Bump up if your room is noisy. |
| D2 | Transition and LUT depth | 🟡 packages exist, thin library | M | — |
| D3 | Caption translation beyond English | 🟡 translate-to-English only | M | — |
| D4 | Video upscaling | 🟡 `enhance.rs` exists | M | — |
| D5 | Camera tracking, flicker removal, AI colour correction, AI text remover, beat sync | ❌ | M–L | Low value for podcasts. |

### Skip

Stabilisation (unless you film handheld), stickers and overlays, templates, music / sticker / font libraries, cloud storage, cross-device sync, team features, AI video generation, retouch, 130-language counts.

---

## 2. Build order (podcast-first, consistent with roadmap spine)

1. **A1** word timing (T1 harness → T2 → T3)
2. **T5** caption identity (stored, not derived), so cuts stop orphaning captions
3. **T7** one writer for text and words (added 16 Oct; needed before A8 and A2)
4. **C1 validation** on the same episode with Burst mode
5. **A8** Enter-split (E4)
6. **A2** transcript editing (E1), with A7 filler removal on the same machinery
6. **A4 + A5** loudness and platform presets (cheap, high visible value)
7. **A9 + A10** caption presets and karaoke wiring (C2, C3)
8. **A6** auto-ducking (V3)
9. **A11** batch export queue (V4)
10. **A3** speaker labels (scope first; largest unknown)
11. **B1** auto-reframe 9:16 (V1), when the clip pipeline is needed
12. **C1, C2, C3** case transform, caption motion (after the schema decision), waveform and word-edge drag
13. **A12** screen / webcam recording
14. **B2** verify multi-clip / B-roll, and **C4** verify animation, before any planning

---

## 3. Open questions for the owner

- Which of your episodes are two-person, and do they need speaker labels?
- Is your room noisy enough that vocal isolation (D1) moves up?
- Do you record remote guests, which makes A12 a priority?
- Show me one prof.zulys frame and say which element to match: word highlight, box or font, or animation.
