# AGENTS.md — rules for working in this repo

Read this before your first change in a session. It is short on purpose:
these are the rules that a test or a linter cannot enforce.

Everything committed — identifiers, comments, doc comments, commit messages,
this file — is in **English**, like upstream Concat. Notes kept outside the
repo may be in any language.

## Where things run

The git root and the Cargo workspace root are different directories: **every
cargo command runs from `src/`**. Build, run and verification commands are in
`.claude/skills/run-concat/SKILL.md` — read it instead of rediscovering them.

## Invariants

Each of these has already been broken once. Treat them as hard rules.

1. **What a clip *is*, is stored — never derived.**
   A caption is a caption because it was created as one, not because it
   currently holds word timings or a source link. Deriving identity from data
   another feature may clear is how a caption silently becomes a title.
   *(A trim once cleared `words`; the clip survived, stopped being a caption,
   and the export then burned it with "Burn captions" switched off.)*

2. **One predicate decides caption-ness: `Clip::is_caption()`.**
   Do not re-implement the test inline. It is currently duplicated across
   three crates — check at least:
   `concat-project/src/model.rs`, `concat-project/src/commands/clips.rs`,
   `concat-project/src/sectionlist.rs`,
   `concat/src/panes/export.rs`, `concat/src/panes/sections.rs`,
   `concat/src/studio.rs`.
   Fixing one copy is how the same bug comes back somewhere else. If you
   change what a caption is, change it in one place and make every site call it.

3. **No two clips share a span on a lane — except caption↔caption.**
   Ordinary clips (title, shape, picture, sound, layer) may never cover
   another clip on their lane. A caption may cover another caption, and may
   never cover an ordinary clip. This rule lives in
   `concat-project/src/placement.rs`; every command that places, moves or
   lengthens a clip asks it, and so must anything new.

4. **Words are clipped at an edit, never dropped.**
   A word straddling a cut is heard on the kept side and must stay on the kept
   side, at the right offset. Losing it loses a syllable from the caption and a
   beat from the karaoke timing.

5. **Never split words on ASCII whitespace alone.**
   Thai, Japanese, Chinese, Lao and Khmer have no spaces between words; an
   ASCII-only split collapses a whole sentence into one "word". Test the same
   way `captionmap` does — `is_alphanumeric`, not `is_ascii_alphanumeric`.

6. **Every number a user can move is clamped in one place.**
   Time fields go through `Clip::tidy` (`MAX_TIME`), frame sizes through
   `VideoSettings::is_sane`. Do not clamp at a second site; a value that
   survives one path and not another is a bug waiting to be reported.

## Before you change anything

- **Grep for the rule you are about to change.** If it exists in more than one
  file, the fix is "make them one", not "fix the one in front of me".
- **Do not widen a rule to fix one case.** Loosening an invariant for caption
  overlap is fine *because it was made a separate, named path* — copy that
  pattern: new path, new doc comment, its own test.

## Before you say a feature is missing

Five "missing" claims in one session turned out to be "already built". A module
name is not evidence, and neither is the absence of the word you happened to
search for. Say *missing* only after reading all three:

1. **The UI path** — the widget and the callback that would carry the action.
   (`Field` in `ui/primitives/modal.slint` binds `edited` and never declares
   `accepted`, so Enter on a caption row has nothing to fire.)
2. **The handler** — what the callback writes, to the end of the function. One
   that stops after `style.content` is not a split feature.
3. **A comparator grep** — the sibling machinery, before concluding: the command
   enum in `commands/mod.rs`, and words like `split`, `caret`,
   `cursor-position`, `accepted`. "Split exists for *time*" is not "split exists
   for *text*" — direction and granularity decide whether it is the same thing.

Miss one of the three and the honest label is **"not verified"** — never
"missing". And when a note and the code disagree, the code is right; fix the
note in the same pass.

## Definition of done

A change is done when all of these hold:

1. **A test exists that fails without your change.** Write it first and show it
   failing. A fix with no test is a guess, however confident it reads.
2. **You can name the test that covers the reported path.** If nothing covers
   it, say so plainly instead of reporting the fix as verified.
3. `cargo test -p <crates you touched>` passes, and `cargo clippy` is clean
   with `-D warnings`.
4. `cargo check -p concat` passes for anything touching the Slint UI — the UI
   bindings are generated at compile time and a narrower check will not see
   the wiring.
5. **A failure that changes what something *is* silently counts as a bug.**
   No crash and green tests are not evidence. State loss, a caption that stops
   being a caption, a clip that changes lane — none of these fail a build.

## Known traps

- **Silent state loss is the most expensive bug class in this repo.** Before
  clearing a field, ask what else reads it to decide *what something is*.
- **The window crate is huge** (`studio.rs` alone is ~10k lines) and the same
  predicate is copied across crates. A session editing one file cannot see the
  copy in another. Grep first.
- **A caption hidden from a list is still on the timeline.** If the Sections
  panel or export no longer sees a caption, the user sees "sync broke again" —
  the clip is right there in the picture.
- **Checkpoints and phase notes describe work; the code decides.** When they
  disagree, the code is right — and the note should be fixed.
