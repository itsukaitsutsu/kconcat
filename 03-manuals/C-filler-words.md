# C. Filler words (Indonesian) — the list and the rules

*Record for E2 (roadmap §4, owner decision A7, 16 Oct 2026). The lists live
in code as the one fact: `src/crates/concat-project/src/filler.rs`. Changing
a list means changing that file; this manual records what ships and how the
matching behaves.*

## The two tiers — both opt-in, nothing is removed unless ticked

| Tier | Words | Why it is its own tier |
|---|---|---|
| **Strong** | `eee`, `anu`, `ehm` | Sounds that carry no meaning of their own; safe to cut whenever they are heard |
| **Weak** | `kayak`, `gitu`, `terus`, `apa ya` | Real words that are *often* filler but can be meaningful ("terus" = continue); the user ticks this tier knowingly |

The owner may add or remove words; the tier of each word is the owner's call
and is recorded here when it changes.

## Matching rules (as built)

- **Whole words only.** "begitu" is never "gitu"; "kayaknya" is never "kayak".
- **Case and edge punctuation are ignored.** "Terus," matches `terus`; "Eee"
  matches `eee`.
- **A multi-word entry needs its consecutive run.** "apa ya" matches only the
  two words back to back; "apa" alone and "ya" alone are not filler.
- Longer entries win over shorter ones sharing a prefix, so the "apa ya" run
  is taken as one filler, not as two accidents.

## What removal does (owner decisions A2/A7)

- **Text only** (checkbox "Potong video saat kata dihapus" off): the filler's
  text leaves the caption and its timing slot goes with it — the single
  writer's rule (T7). Every surviving word keeps its heard time; picture and
  sound stand still.
- **Cut** (checkbox on): each filler's heard span comes out of every lane
  through the remove-selection machinery (E1's path): audio, picture and
  captions cut at the same moments, the gap closes, and a caption left with
  no words at all goes with them — no wordless husks stay on the lane.
- A pass that finds no ticked filler is **not an edit**: nothing is written
  and the undo history keeps no empty step. One pass is one undo.
