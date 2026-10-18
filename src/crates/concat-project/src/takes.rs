// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Laying a finished recording session on the timeline (roadmap P4).
//!
//! One take per microphone, and the owner's rule is one track per
//! microphone: the session that arrives here is already recorded, probed
//! and timed - each take carries the moment its own microphone heard its
//! first sample, as an offset in seconds from the moment the picture
//! started rolling. Placing is the existing commands in sequence
//! (`AddMedia`, `AddTrack`, `AddClip`), folded into ONE undo step under
//! one gesture, so a session that lands wrong comes back off in one move.
//!
//! The sync contract: every take lands at `playhead + offset`, clamped
//! never before zero, so the DISTANCE between two takes' starts is
//! exactly the distance between their first samples - the mics stay in
//! sync because nothing here moves one without the other. The five-minute
//! acoustic check (two mics, one take, clap at both ends) is the owner's
//! machine; what is provable here is that the placement cannot break it.

use crate::commands::{Command, CommandError, NewMedia};
use crate::editor::Editor;

/// One microphone's finished take, as the host hands it over.
pub struct SessionTake {
    /// The microphone's own name, as the system lists it. The document
    /// has nowhere to put it - tracks carry no name - so it rides along
    /// for the host's log and the session UI's meter list, and the
    /// placement itself ignores it.
    pub device: String,
    /// The take's WAV as probed: the media the clip will play.
    pub item: NewMedia,
    /// Seconds from the moment the picture started rolling to the moment
    /// THIS microphone heard its first sample. May be a hair negative -
    /// a fast device can hear before the roll is reported - and the
    /// landing clamps at zero rather than before the timeline.
    pub offset: f64,
}

/// Places a whole session: one new track per take, each take's clip at
/// its own landing. All or nothing - a time that is not a number refuses
/// the session before the first command runs, so a refused session
/// changes no document. Returns the placed clips' ids, in the order the
/// takes arrived, which is the order the mics were listed.
pub fn place_session(
    editor: &mut Editor,
    playhead: f64,
    takes: Vec<SessionTake>,
) -> Result<Vec<String>, CommandError> {
    // All or nothing: a session whose playhead or any of whose offsets is
    // not a number is refused before the first command runs, so a refused
    // session changes no document. (The commands' own predicate would
    // refuse them one at a time, mid-session, with half the takes already
    // on the timeline - which is exactly what a session must not do.)
    if !playhead.is_finite() || takes.iter().any(|take| !take.offset.is_finite()) {
        return Err(CommandError::NotANumber);
    }
    // One gesture name for the whole session, so the editor folds every
    // command below into ONE undo step: a session that lands wrong comes
    // back off in one move, tracks, clips and bin items together.
    let gesture = Some("place-session");
    let mut clip_ids = Vec::with_capacity(takes.len());
    for take in takes {
        let path = take.item.path.clone();
        // The WAV goes into the bin like any import. A path already in
        // the bin mints nothing, and the item that is there gets used.
        let media_id = match editor
            .apply_within(gesture, Command::AddMedia { item: take.item })?
            .created_id
        {
            Some(id) => id,
            None => editor
                .project()
                .media
                .iter()
                .find(|item| item.path == path)
                .map(|item| item.id.clone())
                .ok_or(CommandError::MediaGone)?,
        };
        // One track per microphone: a new lane of its own, under the
        // stack like the voiceover's lowest-free-lane habit - not the
        // first free lane, which could put two people on one track and
        // lose the separation P3's speaker lanes rely on.
        let track_id = editor
            .apply_within(gesture, Command::AddTrack { bottom: true })?
            .created_id
            .expect("AddTrack mints the track it creates");
        // The take's own landing: the playhead plus the moment this
        // microphone heard its first sample, never before the timeline.
        // On the fresh empty lane the clip starts exactly here - there is
        // nothing for the nearest-free-start to move it off.
        let start = (playhead + take.offset).max(0.0);
        let clip_id = editor
            .apply_within(
                gesture,
                Command::AddClip {
                    media_id,
                    track_id,
                    start,
                    ripple: false,
                },
            )?
            .created_id
            .expect("AddClip mints the clip it creates");
        clip_ids.push(clip_id);
    }
    Ok(clip_ids)
}
