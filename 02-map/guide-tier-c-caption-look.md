# Guide: Tier C, caption look (16 Oct 2026)

Status: guide for the owner and for the AI session that builds it. Plan fixed in `kconcat-roadmap-v2-execution.md` (§3 Track 2 and §4 E3). This guide puts the Tier C items in one place, with their order, decisions, tests, and exit condition. It does not replace the roadmap. If the two disagree, fix the roadmap first.

Discussion only. No repo code was changed by this guide.

---

## Goal (owner, 16 Oct 2026)

Get closer to CapCut Pro for long-form podcast editing, with short-form clips as output. Reference style: prof.zulys-style CapCut edits. Every task below must name the CapCut Pro item it serves, or the measurement that proves it. A task that does not serve this goal is not built.

---

## 0. Numbering (read this first)

Two numbering systems exist. Use the mapping below.

| Option B list (Tier C) | Roadmap task | What it is | Plan status |
|---|---|---|---|
| C1 Uppercase / case transform | **C4** | Show caption text in capitals without changing the stored text | Fixed in plan, not built |
| C2 Caption motion (pop, word-reveal, typewriter) | **C5** | Words animate in at their spoken time | Schema v1 approved, not built |
| C3 Waveform under caption row, and word-edge drag | **E3** | Shows audio under captions; drag a word edge to move its timing | Fixed in plan, not built; needs T2 |
| C4 Animation (owner report) | **none** | Owner confirmed (16 Oct 2026): no animation exists. Caption motion is the work under C5. | Closed as a separate item |

Roadmap **C1, C2, C3** are different again:
- **C1** word chunking: already built. Validation only.
- **C2** caption presets: eight presets, decided under A9.
- **C3** karaoke highlight wiring: decided under A10.

---

## 1. Order

Follow the plan order in roadmap §11. For Tier C:

1. **C1 validation** (roadmap C1). After T2, T5, T6, T7.
2. **C2 presets** (roadmap C2, A9). Eight presets, data only.
3. **C3 karaoke** (roadmap C3, A10). Highlight is on or off per user, for all presets.
4. **C4 uppercase** (roadmap C4, Option B C1).
5. **C5 motion** (roadmap C5, Option B C2). Schema v1 is approved, so this can start after C4.
6. **E3 waveform and word-edge drag** (roadmap E3, Option B C3). After E1 and T2.

Do not start any item before the roadmap's plan order allows it. Each task must name its CapCut parity item or its measurement, as the owner's scope rule requires.

---

## 2. Each item

### 2.1 C1 validation: word chunking (already built)

- **What exists:** four layout modes and Re-split, in `src/crates/concat/ui/dialogs/captions.slint`.
- **Task:** validation only. No new UI.
- **Test:** the Burst mode (3 words per caption) on the same episode, before and after T2. Compare by eye and with the Tier 1 table from T1.
- **Serves:** CapCut Pro word chunking, used for readable captions on clips. Measurement: Burst comparison before and after T2, plus Tier 1 table.
- **Done when:** the comparison is recorded in the roadmap. A regression stops the track.

### 2.2 C2 presets (roadmap C2, decided under A9)

- **What:** eight caption presets, data only, under `src/crates/concat/text-presets/`. Names and values are in `03-manuals/B-caption-styles.md` §4 (in the repo).
- **Decided (owner, 16 Oct 2026):**
  - All eight ship. The user picks one from a list.
  - No separate Indonesian variant. Caption size is a fraction of frame height, so the same preset fits any sequence.
  - No extra preset tuning UI.
- **Tests:** a preset with `highlight_color` round-trips through `preset.toml`. Applying one preset to a caption track gives a caption-shaped result without the inspector.
- **Serves:** CapCut Pro caption style presets (Option B A9: CapCut has many; kconcat has two close ones). Measurement: eight presets apply in one click.
- **Preview (mock, not browser-tested):** `05-background/caption-presets-preview.html`. Its look is an estimate only.

### 2.3 C3 karaoke highlight (roadmap C3, decided under A10)

- **What exists:** the renderer already draws one highlight per heard word.
- **Decided (owner, 16 Oct 2026; Tier C approved):**
  - Highlight is a feature the user can turn on or off, for **all** presets. It is the existing option, not new.
  - Presets that the manual lists with a highlight (pop-bold, karaoke, upper-tight) start with it on.
  - Every other preset can turn it on.
- **Test:** a karaoke preset produces one highlight per heard word. The existing test `a_word_timed_caption_gets_one_highlight_per_heard_word` covers the renderer.
- **Measurement:** karaoke frame check at 30 fps, under about 5% of frames on the wrong word, measured after T2 (roadmap §8).
- **Serves:** CapCut Pro karaoke highlight (Option B A10). Measurement: under about 5% wrong-word frames at 30 fps.
- **Dependency:** quality depends on T2. Until T2 is accepted, the highlight can land on the wrong word.

### 2.4 C4 uppercase (roadmap C4, Option B C1)

- **Code status:** absent. `TextStyle` has no transform field. The `to_uppercase` hits in the code are UI labels only.
- **Change:** add a transform field to the text style. Apply it at render time only. The stored caption text is never changed, so the transcript keeps its casing.
- **Test first:** a preset with uppercase renders uppercase, and the stored caption text is unchanged after save and reload.
- **Done when:** the test passes, and the caption design of the other presets is unchanged.
- **Serves:** CapCut Pro case transform (Option B C1). Measurement: preset uppercase renders uppercase; stored text unchanged.
- **Do not:** change the stored text, or change the look of presets that do not ask for uppercase.

### 2.5 C5 motion (roadmap C5, Option B C2)

This is the work the owner called "animation". Owner confirmed on 16 Oct 2026 that no animation exists yet, so this item builds it.

- **Decided (owner, 16 Oct 2026):**
  - Option (a): a `[motion]` block in `preset.toml`. Rejected: Inspector-only (presets would stop being one-click), and an effect package named by the preset (needs a caption package type that does not exist).
  - Four modes in v1: `none`, `pop`, `word-reveal`, `typewriter`. Typewriter is in v1.
  - The Motion drawer is shown for **captions and titles**, under Inspector → Text tab. It is not gated by `highlightable`.
  - Default `duration` = 0.18 s. Accepted for now. Tune after T2 is accepted.
- **Modes (agreed definition):**
  - `none`: the whole cue is on screen from its start. Only the highlight moves.
  - `word-reveal`: the cue starts empty. Each word appears at its spoken time and stays.
  - `pop`: each word appears at its spoken time and scales from `scale_from` (default 1.15) to 1 over `duration`.
  - `typewriter`: letters of each word appear one by one, across the word's spoken start-to-end. `duration` is not used.
- **Schema (v1):**
  ```toml
  [motion]
  mode = "none"       # none | pop | word-reveal | typewriter
  duration = 0.18     # seconds; pop and word-reveal
  scale_from = 1.15   # pop only
  ```
- **Storage:** a field on `TextStyle` with `serde(default)`. Older projects load as `mode = "none"`. The preset copies its block into the clip on apply.
- **Clamp:** `duration` and `scale_from` are clamped in the one place where `TextStyle` is clamped (`AGENTS.md` invariant 6). Nowhere else.
- **Old builds:** `PresetFile` has no `deny_unknown_fields`, so an unknown `[motion]` block is ignored by older builds. Test this.
- **Facts checked (16 Oct 2026):** `TextStyle` has no motion field today. The per-word reveal exists in the engine (`RevealMap`, `src/crates/concat-core/src/shader.rs`). What drives `progress` for a caption track today is not verified.
- **Serves:** CapCut Pro caption motion, pop / typewriter / word-reveal (Option B C2). Measurement: each mode on the test clip, and choice survives reload.
- **Tests:**
  - Applying a preset with motion gives that motion on the clip.
  - The motion choice survives a project save and reload.
  - Typewriter: letters appear in order within each word, and only inside its spoken start-to-end.
  - An old preset file with no `[motion]` block loads as `none`.
- **Preview (mock):** `05-background/motion-drawer-preview.html`. Mock only, sample timings. Browser check by the owner is pending.
- **Done when:** a caption preset with motion gives the motion without opening the inspector, and the choice survives reload.

### 2.6 E3 waveform under caption row and word-edge drag (roadmap E3, Option B C3)

- **Code status:** the timeline waveform exists (`src/crates/concat/ui/primitives/wave.slint`). A waveform under the caption row does not exist. Word-edge drag does not exist.
- **Needs:** T2 (timing must be right before dragging edges is useful). E1 is recommended first.
- **Change:** show the waveform under the caption row. Let the user drag a word's edge to change its timing.
- **Serves:** Option B C3 (waveform under captions, word-edge drag). Measurement: edge drag moves highlight and timing, and survives export.
- **Done when:** dragging a word edge moves its highlight and its timing, and the edit survives export.
- **Test:** the same edit, exported, keeps the new timing on the word.

---

## 3. Animation (old Option B C4)

Owner confirmed on 16 Oct 2026: animation does not exist yet. There is no separate animation task. The caption motion work is in §2.5 (C5). If the owner finds a different animation later, it becomes a new task with its own parity item.

---

## 4. Rules for every Tier C item

- **Caption design does not change** unless the owner asks. The owner rejected per-word colour: "jangan berubah seperti itu". Colour belongs to the speaker and applies to all of that speaker's captions.
- **No new UI** beyond the Motion drawer (C5) and the existing caption controls.
- **Do not change working paths** without need (owner rule).
- **Tests come first.** Each item lists its tests above. Write the failing test, then the change.
- **Verify in the app** before calling an item done. The mock previews are not proof.

---

## 5. Exit for Tier C

Track 2 exit (roadmap §3): a caption track that looks like the CapCut Pro reference, without the inspector, once T2 is accepted. Tier C is done when C1 to C5 and E3 meet their "done when" lines, and the karaoke measurement in §2.3 is met.

Why Tier C serves the goal: the reference is a prof.zulys caption look. Tier C is the set of items that make kconcat's captions look like that reference, so the exit test compares the output with the reference frame.
