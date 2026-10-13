PHASE 4a - THE APP WRITES THE LINK  (for itsukaitsutsu/kconcat)
================================================================

WHAT THIS IS
------------
Phase 1 gave captions a `caption_of` field. Phase 2 made edits follow
it. But no real caption ever GOT the link, because the app's caption
run creates text clips with `Command::AddTextClip` - and that command
had nowhere to put the link. A caption batch cannot set the field
afterwards: the clips' ids are minted inside the batch, so the run
cannot know them yet (the command's own docs say the same thing about
`duration`, which exists for exactly this reason).

So this phase adds one field in the same style:

  Command::AddTextClip { ..., caption_of: Option<String> }
  #[serde(default)]  - command JSON written before this still parses
                       as an unlinked title.

AND wires the one place that knows the answer:
`panes/captions.rs` - the Captions sheet.

  * Voice run ("Use voice from timeline"): the sheet now remembers
    WHICH clip is being transcribed (new `Subject.clip_id`, captured
    when the run begins, so a changed selection mid-run cannot
    mislabel the words) and every caption it makes is created with
    caption_of = that clip. From now on, moving/trimming that clip
    in the app moves/trims its captions (Phase 2 does the work).
  * Script run (typed text): caption_of = None. A script belongs to
    the timeline, not to one clip's voice - linking it would make it
    chase a clip it was never transcribed from.
  * Duplicating a caption (studio.rs place_copy): the copy keeps the
    link - it sits over the same voice.
  * Every other AddTextClip in the workspace (titles, API, export and
    host tests) gains `caption_of: None` - they are titles, unlinked.

FILES TO REPLACE (9)
--------------------
Extract over your repo root, overwriting:

  src/crates/concat-project/src/commands/mod.rs      (new field)
  src/crates/concat-project/src/commands/clips.rs    (stores it; ALSO
        contains all of Phase 2 - see note below)
  src/crates/concat-project/src/lib.rs               (tests; ALSO
        contains all of Phase 2 tests - see note below)
  src/crates/concat/src/panes/captions.rs            (the wiring)
  src/crates/concat/src/studio.rs                    (duplicate keeps link)
  src/crates/concat-api/src/lib.rs                   (caption_of: None)
  src/crates/concat-export/src/flatten.rs            (caption_of: None)
  src/crates/concat-host/src/titles.rs               (caption_of: None)
  src/crates/concat-host/tests/export.rs             (caption_of: None)
  CHECKPOINT.md  +  README-PHASE-4A.txt               (docs, not code)

NOTE IF YOU HAVE NOT APPLIED PHASE 2 YET: clips.rs and lib.rs in THIS
zip are the newest versions and already include everything in the
Phase 2 zip. Applying this zip alone gives you Phase 2 + 4a. Applying
Phase 2 first and then this is equally fine - same end state.

HOW TO VERIFY - STEP 1 (cheap, do this first)
---------------------------------------------
  cd C:\dev\Concat\src
  cargo test -p concat-project

Expected:  test result: ok. 221 passed; 0 failed
(the linked_caption test helper now sets the link at creation, the way
the app does, instead of patching it afterwards.)

HOW TO VERIFY - STEP 2 (the expensive one - plan 30-90 min)
-----------------------------------------------------------
This is the first phase that touches the `concat` app crate
(724 crates, FFmpeg/Skia/sherpa native deps). It was NOT compiled
before packaging: the sandbox has no FFmpeg, so be the compiler here.

  cargo check -p concat

- If it finishes with no red `error` lines: done. Optionally
  `cargo dev`, open a project, caption a clip with "Use voice from
  timeline", then drag that clip - the captions should travel with it,
  and one Ctrl+Z should undo clip + captions together.
- If it errors: copy the FULL red text (start from the first `error`)
  plus the command, and paste it in the next session. Do not guess -
  especially do not edit around it. Likely candidates, already thought
  about while writing this: `Subject` used to be `Copy` and now holds
  a String (already fixed in this zip), and any AddTextClip literal
  outside the workspace search (already audited: none missing).

One honest caveat: captions made BEFORE this phase have no link and
never will - there is nothing in an old project saying which clip a
caption came from. Re-run captions on a clip to link its words.

WHAT IS NEXT (in CHECKPOINT.md, for the next session)
-----------------------------------------------------
Phase 4b - Sections panel (caption/silence/other), then Autotrim,
then word-level captions / karaoke. See CHECKPOINT.md at the root of
this zip: paste it into a new Arena session to continue seamlessly.
