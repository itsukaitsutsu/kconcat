# UI-1 investigation — panels do not scroll when resized smaller

*16 Oct 2026. Scope: code reading only. The app was NOT run. Every runtime claim below is marked "to verify". No code was changed.*

## 1. Symptom (owner report)

- In VTS, each panel scrolls when made smaller.
- In kconcat, the scroll is not visible. To see a whole panel, the user resets it to its default size.

## 2. What the code has (verified by reading `src/crates/concat/ui/`)

| Item | Where | Fact |
|---|---|---|
| Scroll component | `workspace/pane.slint` `PaneScroll` | Wheel, finger drag, and an 8 px `ThinScrollbar`. Overflow = `content.preferred-height − root.height`. |
| Scrollbar visibility | `primitives/scrollbar.slint` | `visible: overflow > 0`. No overflow means no bar, and the wheel does nothing. |
| Scrollbar colour | `primitives/scrollbar.slint` | Thumb `Theme.line-strong`, which turns `fg-dim` on hover. Contrast not checked. |
| Users of `PaneScroll` | `config-pane.slint` (inspector, transitions), `media-pane.slint` (pages), `dialogs/sections.slint` (list), `pane-switcher.slint` (menus), `dialogs/captions.slint` | Yes. |
| Panes with no `PaneScroll` | `workspace/keyframes-pane.slint`, `workspace/meters-pane.slint` | No scroll at all. Preview has one hit only; not checked further. |
| Timeline | `workspace/timeline-pane.slint` | Its own `ThinScrollbar` / `ThinHScrollbar`. |
| Recent change | commit `f1426c9` (15 Oct 2026, "can you fix this"), `workspace/pane.slint` | Added `callback user-scrolled`, made `offset` `in-out` and `at` `out`, for the Sections ↔ Timeline sync (`CHECKPOINT.md` Phase 8). Reading the diff: it does not remove scrolling. Not a confirmed cause. |
| Design note | `CVTS.md` item 5 | `SectionsBody` rows table: `PaneScroll` with `vertical-stretch: 1`, "eliminating nested scrollbars". |

## 3. Mechanism to test

`PaneScroll` measures the content with `content.preferred-height`, which is the sum of its children's preferred heights. If that number is smaller than the content actually drawn, then:
- overflow = 0,
- the scrollbar stays hidden,
- the wheel does nothing,
- the lower part of the panel is cut off by `clip: true`.

That matches the symptom exactly. Children that report no preferred height (plain rectangles, conditional children that are off, text that wraps) are the usual reason.

## 4. Hypotheses (ranked by likelihood from the code)

| # | Hypothesis | Predicts | Test |
|---|---|---|---|
| H1 | **Panes with no `PaneScroll`** (Keyframes, Meters): the feature is missing, not broken. | These panes clip with no bar at any size. | Resize Keyframes and Meters to minimum. |
| H2 | **Preferred height under-measured** inside `PaneScroll` (overflow = 0 while content is taller). | Bar hidden, wheel does nothing, bottom cut off, in a `PaneScroll` pane. | Temporarily show `content.preferred-height` and `root.height` as text inside `PaneScroll`. If preferred < drawn, H2 is confirmed. |
| H3 | **Wheel captured by inner controls** (number fields, knobs, sliders) so the wheel never reaches `PaneScroll`. | Wheel works over empty space in the panel, not over controls. | Wheel over empty space, then over a control. |
| H4 | **Scrollbar hard to see** (`line-strong` thumb, 4 px). The bar exists but the owner does not notice it. | Bar appears when hovered or dragged. | Check the bar's colour against the panel background. |
| H5 | **Regression from `f1426c9`**. | Only in panes that used to scroll. | Compare a build from before 15 Oct with the current one, same pane, same size. |

## 5. Runtime protocol for the owner (Windows build)

1. Build and run `cargo run -p concat` from `src/`. Record the commit (`git rev-parse --short HEAD`).
2. For each pane (Inspector with a clip selected, Inspector with nothing selected, Media, Sections, Keyframes, Meters, Preview, Timeline): drag its border until it is at its smallest. Fill in the table below.

| Pane | Clipped at min size? | Bar visible? | Wheel over empty space scrolls? | Wheel over a control scrolls? |
|---|---|---|---|---|
| Inspector (clip selected) | | | | |
| Inspector (no selection) | | | | |
| Media | | | | |
| Sections | | | | |
| Keyframes | | | | |
| Meters | | | | |
| Preview | | | | |
| Timeline | | | | |

3. Optional (debug only, do not commit): put a `Text { text: "pref " + content.preferred-height / 1px + " h " + root.height / 1px; }` inside `PaneScroll`. That is H2's test.
4. Send the filled table back. The fix depends on which hypothesis it confirms.

## 6. Likely fix shape (after the protocol)

- H1: wrap the Keyframes and Meters content in `PaneScroll` (controls only, not the canvas).
- H2: measure real height from the rendered content (for example a `min-height` on children, or an explicit height for the content), not from `preferred-height` alone. Keep `PaneScroll`; do not add a second scroller.
- H3: let the wheel pass through controls that do not use it, or give the controls an explicit `scroll-event` pass-through.
- H4: a brighter thumb colour and a wider hit area. Design call.
- H5: revert or adjust `f1426c9` only if the test shows the regression.

## 7. Code change (19 Oct 2026) — runtime acceptance still pending

The missing-scroller path is now implemented without reverting the existing `PaneScroll` work:

- `workspace/pane.slint`: `PaneScroll` accepts an optional `min-content-height` and uses it for both content layout height and overflow. Default is `0px`, so existing users keep their previous behavior.
- `workspace/keyframes-pane.slint`: the keyframe body is wrapped in a finger- and wheel-scrollable `PaneScroll`. The dopesheet gets a content height based on its row count; curves keep a 280px minimum drawing area. Header and tools remain fixed.
- `workspace/meters-pane.slint`: the meters body is wrapped in `PaneScroll` with a 220px minimum content extent. Header remains fixed; the bars and readouts can be reached in a short panel.

This addresses H1 for Keyframes/Meters and supplies explicit extent for these panes rather than assuming every layout reports its drawn size correctly. It does **not** prove all other `PaneScroll` users have correct preferred-height values. The runtime protocol below is still needed to check wheel/finger gesture routing, scrollbar visibility, and any H2 cases in other panes.

## 8. Not verified

- The Slint UI was not compiled or run in this environment. H2's behavior in the remaining panes is still a code-level hypothesis.
- The VTS behaviour is the owner's observation; VTS was not checked.
