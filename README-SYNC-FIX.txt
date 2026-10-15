Sections / Timeline / Caption sync fixes (kconcat, 2026-10-15)

WHAT WAS WRONG (found by reading the code; each one is a path you can check)

1. Ticks reset on every edit.
   Every document change re-tiles the Sections rows from scratch, and every
   fresh row starts at its default (silences unticked). So after you kept a
   pause, then changed a caption word, a volume, anything, the pause was
   removed again. Now the rebuilt rows carry the ticks of the rows they
   replace (matched by kind and span, to the millisecond).
   Files: concat-project/src/sectionlist.rs (carry_ticks + 5 unit tests),
          concat/src/panes/sections.rs (populate_sections uses it).

2. The Sections list went stale after its own edits.
   While the Sections pane handles a message it is moved out of the studio,
   so the studio's "after an edit, refresh the sections" step saw a blank pane
   and skipped it. Captions added from Sections, and row edits, did not show
   up in the list until something else refreshed it.
   Now Studio::handle re-tiles the list once the pane is back, if the document
   changed while it was handling the message.
   Files: concat/src/studio.rs (Msg::Sections arm),
          concat/src/panes/sections.rs (a rebuild no longer forces the list to
          scroll to the playhead; a finished detection still does, as before).

3. Editing a row overwrote other caption tracks.
   Changing a row's words updated every caption clip overlapping that row's
   time, including a translation on another track. Now only the caption whose
   words are the row's old words is changed.
   File: concat/src/panes/sections.rs (edit_row_text).

4. The Timeline's section strip (on the ruler) did not follow captions.
   It was published only with the full window update. Dragging, trimming or
   splitting a caption uses the lanes-only update, so the strip stayed where
   the caption had been until another full update.
   Now the strip, its filter pills and "Remove selected" state are published
   with the lanes.
   File: concat/src/studio.rs (publish_lanes; the same lines removed from
   publish_chrome).

5. Scrolling the timeline and the list used different conventions.
   Sections -> timeline puts a row a quarter of the lanes' width from the left.
   Timeline -> Sections read the position back with a lead term that only
   matched that above half a screen, so the list could land on a different row
   from the one you scrolled to. TimelinePane::focus_time now is the exact
   inverse of scroll_to_time (centred phone view: the middle).
   File: concat/src/panes/timeline.rs.

6. "Remove selected" in Sections put the sound and the other lanes out of step.
   Removing a whole section sent a per-lane RemoveClips with ripple. That
   closed the gap only on the section's own lane: its detached sound stayed
   where it was and was not removed, and a caption or music bed on another
   lane stayed at its old time. A cut inside a clip also trimmed a second
   piece's sound wrongly, because that sound is linked to the first piece.
   Now a whole-section removal is AutotrimClip with an empty kept list, and
   the model rebuilds every clip that overlaps the section on every lane:
   the sound, the captions, and the bed. Time before the cut stays put, and
   time after it moves left on every lane. A clip that runs past the cut
   keeps its tail. A piece keeps its id, so a second cut still finds the
   sound linked to it.
   Files: concat-project/src/commands/clips.rs (AutotrimClip),
          concat-project/src/commands/mod.rs (doc),
          concat/src/panes/sections.rs (apply_trim).
   Tests (concat-project/src/lib.rs):
     a_whole_clip_removal_takes_its_sound_and_closes_every_lane
     a_second_cut_takes_the_sound_of_the_piece_it_cuts
     a_cut_takes_a_bed_on_another_lane_with_the_picture_and_keeps_its_tail
   All three fail on the old code and pass on this one.
   The existing test that expected an empty kept list to be a no-op now
   checks only the non-empty no-op cases.

NOT CHANGED (checked, left as-is)
- Locked lanes: the Sections trim does not check the lane lock, and it did
  not before. The lock is UI state (lane_view.locked), not part of the
  model. Left for a later decision.
- Stream-copy export ("Same as copy") ignores gaps, speed and effects on
  the timeline, and cuts sound and video at different points. This is not in
  this patch; it needs a decision on whether to warn, refuse, or re-encode.
- AddCaptionsToTimeline and AddCaptionsNewTrack both create a new caption
  track, as the Phase 8 notes say. The doc comment on AddCaptionsToTimeline
  still says "replacing existing captions"; that comment is stale.

VERIFY
Windows, from C:\dev\kconcat\src:
  cargo test -p concat-project
    expect: 250 passed (was 242; 5 are carry_ticks tests, 3 are the
            Sections model tests in item 6).
  cargo dev
    then, with a video that has speech and a few captions:
    a) Sections: Detect, untick a silence row (keep that pause). Edit a caption
       word in the list. The pause must still be unticked.
    b) Sections: Add captions. The new rows appear in the list straight away.
    c) Two caption tracks (e.g. English + Japanese) on the same span. Edit an
       English row. The Japanese caption must not change.
    d) Drag a caption on the timeline. The section strip on the ruler moves
       with it.
    e) Scroll the timeline: the list follows. Scroll the list: the timeline
       follows. Neither should jump on its own.

BUILD FIX (after the first Windows build): studio.rs:2750 in the previous commit
read studio.playhead while studio.sections was mutably borrowed through a RefMut
(E0502). It is now read into a local first. That line came from commit f1426c9,
not from these changes.

NOT COMPILED HERE: the concat crate (the window) needs Slint, Skia, FFmpeg and
whisper; the sandbox this was written in cannot build them. concat-project was
built and tested. Please run `cargo check -p concat` and tell me the first
error verbatim if there is one.
