PHASE 2 - CAPTIONS FOLLOW MOVE / TRIM  (for itsukaitsutsu/kconcat)
==================================================================

WHAT THIS IS
------------
Phase 1 (already in your repo, commit "update3") gave every caption a
`caption_of` link to the clip it was transcribed from - but nothing used
that link yet. You could move a video and its captions stayed behind.

Phase 2 makes the link DO something, entirely inside `concat-project`
(the cheap crate: ~18 crates to build, no FFmpeg, no app build):

  1. MOVE   - MoveClips now shifts a clip's linked captions by the same
              amount, in the SAME command, so it is one undo step.
              A caption you dragged explicitly in the same move is not
              shifted twice (its explicit move wins).
  2. TRIM   - TrimClip re-windows the clip's captions into its new range:
              a caption partly trimmed is clamped, a caption whose source
              was trimmed away completely is removed (same undo step).
              A ripple head-trim slides captions left with the content;
              a plain head-trim keeps the remaining picture put, and the
              captions clamp to the new head.
  3. RIPPLE - When a ripple trim or ripple delete pulls later clips left,
              their captions come with them (captions live on a different
              lane, so the old per-track ripple never touched them).
  4. SPLIT  - SplitClips re-links a caption that starts at/after the cut
              to the new tail piece, so a later move drags the right half.

Deliberately NOT changed, because a Phase 1 test pins the contract:
deleting a clip (RemoveClips) does not delete or rewrite its captions.
Their links dangle and the caller clears them via captions_with_no_clip.

WHAT TO REPLACE (2 files, nothing else)
---------------------------------------
Copy these over the same paths in your repo, overwriting:

  src/crates/concat-project/src/commands/clips.rs   (the behaviour)
  src/crates/concat-project/src/lib.rs              (4 new tests only)

No Cargo.toml change. No model.rs change (Phase 1 already added the
field). No app (`concat`) change - the app calls MoveClips/TrimClip
already, so it gets this behaviour for free when you next build it.

HOW TO VERIFY (one line at a time, in your build terminal)
----------------------------------------------------------
  cd C:\dev\Concat\src
  cargo test -p concat-project

Expected, near the end:

  test result: ok. 221 passed; 0 failed

That is your previous 217 plus these 4 new ones (search the output):
  captions_follow_their_clip_when_it_moves
  a_caption_dragged_explicitly_is_not_moved_twice
  captions_are_clamped_and_dropped_by_a_trim
  captions_follow_a_ripple_delete_and_relink_on_split

Then the tidy checks (both should print nothing / say Finished):

  cargo fmt -p concat-project -- --check
  cargo clippy -p concat-project --all-targets -- -D warnings

Verified before packaging on Rust 1.93.1 (your pinned toolchain):
221 passed, clippy clean, fmt clean.

IF IT FAILS
-----------
Copy the exact red text + the exact command and ask. Do not guess.
The most likely non-code cause is still a stale terminal (close VSCode
fully, reopen, use open-a-build-terminal.bat).

WHERE YOU ARE NOW (the whole map)
---------------------------------
  Phase 1  link field .............. DONE (in your repo)
  Phase 2  captions follow edits ... THIS ZIP
  Phase 3  silence detection ....... DONE (in your repo: concat-media/src/silence.rs)
  Phase 4  see it in the app ....... NOT DONE - the expensive one (724 crates)

Still open after this, in a sensible order for a next session:
  a) Wire the app's caption run to actually SET caption_of when it creates
     caption clips (Phase 1 field + this logic are useless until the app
     writes the link). This touches the `concat` crate - the expensive build.
  b) Sections panel (caption/silence/other) - the math is already in
     sections.rs; it needs a UI panel in the app.
  c) Autotrim - assemble silence::detect + sections::merge/complement +
     a RemoveClips ripple Batch.
  d) Word-level captions / karaoke - captionmap::group_words port +
     whisper token timestamps.

Suggested next step: (a), because without it no real caption in the app
is linked, so Phase 2 has nothing to follow. Say "continue with (a)"
and you will get the next zip.
