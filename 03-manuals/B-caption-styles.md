# B. Caption styles — what already works, what's actually missing

*Verified against your tree at `1aece2d`. This doc corrects three of my own earlier claims — two items on the shortlist are already done. The third correction (word chunking) landed on a second pass the same day; see §1 and §3.*

---

## 1. Stop here: four things I listed as gaps already exist

I checked before writing. Correcting myself:

| I said | Reality | Evidence |
|---|---|---|
| **"Caption style library missing"** | ❌ wrong — **32 text presets already ship** (`concat.atomic`, `concat.bold-caption`, `concat.clean`, `concat.impact`, `concat.lower-third`, `concat.pixel`, `concat.retro`, `concat.blackletter`, …) | `crates/concat/text-presets/` |
| **"One-click apply style to track missing"** | ❌ wrong — it exists, and defaults **on** | `studio.rs:963` `apply_all_captions`, `studio.rs:3550`, `studio.rs:5279` |
| **"No karaoke/highlight"** | ❌ wrong — the renderer is built and tested: one overlay title per heard word, each with its own picture | `titles.rs:367`, `titles.rs:520`, test `a_word_timed_caption_gets_one_highlight_per_heard_word` |
| **"No word-chunking"** | ❌ wrong — **four layout modes ship**: Engine (as transcribed), Readable lines, Burst (3 words), One word at a time — plus a **Re-split** button | `ui/dialogs/captions.slint:78`, `:615`; `GroupPolicy::for_preset` `captionmap.rs:597–627`; `resplit_drafts` `panes/captions.rs:596` |

And the style system itself is far richer than I assumed — `TextStyle` (`model.rs:980`) already carries:

```
font_family, font_size, font_weight (100–900), italic, color, align, opacity,
stroke_width, stroke_color, shadow,
background (plate) + background_radius + padding_x + padding_y,
line_height, tracking, max_width, height,
highlight_color                          ← the karaoke colour
```

That is CapCut's caption style vocabulary, essentially complete. **So B is not "build styles" — it's "aim the existing machinery at captions".**

---

## 2. The real gap, in one sentence

The 32 presets are **title looks** (Western, blackletter, comic, stencil, poster, handwritten…). None of them is a **caption style**: none sets `highlight_color`, none sets caption position, none carries motion.

Verified: `grep -rl "highlight_color" crates/concat/text-presets/` → **nothing**. And the preset schema (`preset.toml`) is:

```toml
id, name, font, offsetY, language, order, sample
[style]   # any TextStyle field
```

`highlight_color` **is** a TextStyle field — so a preset can set it today, with no schema change and no Rust. **The mechanism is already capable; nothing uses it.**

---

## 3. The six concrete gaps

| # | Gap | Effort | Notes |
|---|---|---|---|
| 1 | No caption presets (with highlight colour + position) | **small** | pure data + a test that the field loads from TOML |
| 2 | Presets carry no motion (pop, typewriter, bounce, word-reveal) | small–medium | needs one schema field + something to apply it |
| 3 | No uppercase/transform option | small | CapCut styles are usually ALL CAPS; not in `TextStyle` |
| 4 | Presets are one flat list of 32; captions deserve a group | small | add a `kind`/`group` tag and a tab |
| 5 | ~~No word-chunking~~ — **wrong, it exists**: four layout modes + Re-split (§1). It only reads badly while the word times it cuts on are off | — | **validation after A**, not a build: re-split the same video as *Burst* and compare |
| 6 | Preset card shows `sample = "Aa"` — fine for titles, weak for captions | small | a caption sample line reads better |

**What deserves emphasis is the opposite of what I first wrote.** Chunking is not a gap — it ships (§1): Burst already limits a cue to 3 words / 18 characters / 2.5 s and breaks on a 0.35 s pause (`captionmap.rs:597–627`). What is missing is the *timing* those boundaries cut on — with estimated words the cut lands in the wrong place, and on a 0.4–2.5 s cue that error is 15–60 % of what the viewer sees. So the highest-value item in B is not a build at all: it is **running A and watching Burst become readable**. The next thing to *build* here is item 1 of the table above — the caption-shaped presets.

---

## 4. A concrete set of 8 caption presets to ship

Written as they would sit in `text-presets/`, all with `kind = "caption"` if you add that tag. Values are starting points — the point is that they are *caption-shaped*: bigger, heavier, positioned low, usually with a stroke or plate to survive footage behind them.

| Preset | weight | size | stroke | plate | highlight | position |
|---|---|---|---|---|---|---|
| `kconcat.pop-bold` | 900 | 0.075 | 0.014 `#000` | none | `#ffd21f` | y 0.42 |
| `kconcat.boxed` | 800 | 0.068 | 0 | `#000000cc`, radius 0.02 | none | y 0.44 |
| `kconcat.karaoke` | 900 | 0.078 | 0.012 `#000` | none | `#39ff88` | y 0.42 |
| `kconcat.minimal` | 600 | 0.055 | 0 | none | none | y 0.45 |
| `kconcat.hard-shadow` | 900 | 0.074 | 0.010 `#000` + shadow | none | none | y 0.42 |
| `kconcat.upper-tight` | 800 | 0.062 | 0.008 `#000` | none | `#ffffff` (inverse) | y 0.40 |
| `kconcat.low-third-cap` | 700 | 0.050 | 0 | `#00000099` | none | y 0.62 |
| `kconcat.clean-bilingual` | 600 | 0.052 | 0.006 `#000` | none | none | y 0.50 (stacked) |

Notes that matter more than the numbers:

- **Stroke or plate, always.** Captions sit over moving footage; anything without one becomes unreadable in the exact frame where it matters.
- **Bilingual stacking is your differentiator** — `offset_y` is a fraction of frame height, so the second language is the same preset with +0.10 and no other change. Your Stage-2 work already stacks tracks; the presets should make it one click.
- **`language` field exists** — ship a `kconcat.indonesian` variant with tracking/size tuned for Indonesian (longer words, more syllables) instead of inheriting Latin defaults.
- **Do not chase a large library.** Eight good ones beat sixty; CapCut's 100s exist because it is a template business, and you are not.

---

## 5. The one schema decision

Motion has to live somewhere. Three options:

1. **Add a `[motion]` block to `preset.toml`** (`in = "pop"`, `perWord = true`). Presets then carry a whole look, which matches how CapCut's caption styles behave.
2. **Keep presets style-only**, motion chosen in the inspector. Less coupling, but a preset stops being "one click for a look".
3. **Ride the existing reveal path** — `reveal_map` is already baked per title (`concat-core/src/shader.rs:222`), and the effect packages are folders discovered at compile time. A `concat.caption-pop` effect package could carry the motion, and the preset names it.

Option 1 is the closest to CapCut and the smallest user-facing change. Option 3 reuses machinery you already have and keeps presets data-only. Either is defensible — but decide it *before* writing preset files, because it changes their shape.

---

## 6. Order of work, and why

```
A  (word timing)  ──►  karaoke becomes readable
                            │   and Burst / One-word chunking becomes readable
                            │   (validation only — those modes ship today, §1)
                            ▼
B1 (8 caption presets + kind tag)   the next visual difference people notice
B3 (uppercase transform)
B2 (motion — after the schema decision in §5)
```

**There is no B5 left to schedule** — chunking ships, so all it needs is A (§3). B1 is next: with Burst and One-word layouts already available, a caption-shaped preset is the remaining visual difference between kconcat and CapCut, and it is pure data. The one ordering rule that survives: **validate Burst immediately after A**, before styling, because the chunk boundary is what a preset is styling.

---

## 7. Exit criteria

- `kconcat.pop-bold` applied to a generated caption track produces a CapCut-shaped caption with **no inspector use**
- after A, the same video re-split as **Burst** reads correctly and karaoke highlights stay aligned through the split — those modes ship today (§1); this is a check, not a build
- a second language stacks with one click, no manual `offset_y`
- a test asserts that `highlightColor` round-trips through `preset.toml` (this is the one field the whole karaoke path hangs on)
