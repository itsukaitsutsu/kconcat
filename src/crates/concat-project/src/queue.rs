// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! V4: the batch export queue's state (roadmap V4, owner A11, 16 Oct 2026).
//!
//! A11's three decisions, as invariants of this module:
//!
//! - **The queue runs as a separate process, like Adobe Media Encoder.**
//!   Editing continues while the queue runs. What lives here is the state
//!   the app keeps *about* that runner: what is queued, in whose order,
//!   what is encoding now, and how each item ended. The runner itself -
//!   spawning, feeding it, watching it - lives with the export machinery,
//!   which this crate does not touch.
//! - **Failure: the queue continues.** A failed item is marked and stays
//!   visible with its reason; [`ExportQueue::start_next`] steps over it to
//!   the next pending item, whatever came before.
//! - **Order: the user chooses.** Items can be moved before or during the
//!   run - while one encodes, the pending ones behind it are still the
//!   user's to reorder. Only pending items move: the running one is in the
//!   runner's hands, and a finished one is history.
//!
//! One item runs at a time - the runner is one encoder process - so
//! [`ExportQueue::start_next`] refuses while anything is running.
//!
//! This is session state, not project state: it never enters the document,
//! so it carries no `Command`, no undo, and no `DOCUMENT_VERSION` change.

use std::fmt;

/// How one queued export stands.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QueueState {
    /// Waiting for the runner.
    Pending,
    /// Encoding now; at most one item is ever in this state.
    Running,
    /// Finished; the file is where the item asked.
    Done,
    /// Finished badly; the reason is the runner's, kept for the user.
    Failed(String),
}

/// One queued export. The payload - what to encode - stays with the app
/// that enqueued it, keyed by `id`; the queue holds the state and the
/// order, and nothing that needs FFmpeg to name.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueuedExport {
    /// The queue's own id, never reused within one queue's life.
    pub id: u64,
    /// What the user sees: the file's stem, the platform, their note.
    pub name: String,
    /// Where it stands.
    pub state: QueueState,
}

/// The queue itself: a list in the user's order, and the id counter that
/// makes `id` stable while items move.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ExportQueue {
    items: Vec<QueuedExport>,
    next_id: u64,
}

/// Which way a move goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    /// Earlier in the run.
    Up,
    /// Later in the run.
    Down,
}

impl ExportQueue {
    /// An empty queue.
    pub fn new() -> ExportQueue {
        ExportQueue::default()
    }

    /// Adds an export at the end of the run and returns its id.
    pub fn enqueue(&mut self, name: impl Into<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(QueuedExport {
            id,
            name: name.into(),
            state: QueueState::Pending,
        });
        id
    }

    /// The items in run order.
    pub fn items(&self) -> &[QueuedExport] {
        &self.items
    }

    /// Moves one pending item one place, within the pending run. Refused -
    /// `false`, nothing moved - when the id is unknown, the item is not
    /// pending, or it is already at the edge of the pending stretch: the
    /// running item's place is the runner's, and finished items are
    /// history the list keeps in place.
    pub fn move_item(&mut self, id: u64, direction: Direction) -> bool {
        let Some(index) = self.items.iter().position(|item| item.id == id) else {
            return false;
        };
        if self.items[index].state != QueueState::Pending {
            return false;
        }
        // A pending item swaps with a pending neighbour, or does not move:
        // the running item is the runner's, and finished items are history
        // the list keeps in place, so the pending stretch stays contiguous
        // and the run order stays readable top to bottom.
        let neighbour = match direction {
            Direction::Up => index.checked_sub(1),
            Direction::Down => Some(index + 1).filter(|next| *next < self.items.len()),
        };
        let Some(neighbour) = neighbour else {
            return false;
        };
        if self.items[neighbour].state != QueueState::Pending {
            return false;
        }
        self.items.swap(index, neighbour);
        true
    }

    /// Hands the first pending item to the runner: it becomes the running
    /// one, and its id comes back. `None` when an item is already running
    /// (one encoder at a time) or nothing is pending - including when what
    /// is left is only failed and done items, which is the queue's
    /// "finished" as much as an empty one is.
    pub fn start_next(&mut self) -> Option<u64> {
        if self
            .items
            .iter()
            .any(|item| item.state == QueueState::Running)
        {
            return None;
        }
        let next = self
            .items
            .iter_mut()
            .find(|item| item.state == QueueState::Pending)?;
        next.state = QueueState::Running;
        Some(next.id)
    }

    /// The runner reports how the item it was given ended. An id that is
    /// not the running one's is refused - `false` - because a late report
    /// from a killed runner must not rewrite the item that replaced it.
    pub fn finish(&mut self, id: u64, result: Result<(), String>) -> bool {
        let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
            return false;
        };
        if item.state != QueueState::Running {
            return false;
        }
        item.state = match result {
            Ok(()) => QueueState::Done,
            Err(reason) => QueueState::Failed(reason),
        };
        true
    }

    /// The items that failed, in run order, reasons with them.
    pub fn failed(&self) -> Vec<(u64, String)> {
        self.items
            .iter()
            .filter_map(|item| match &item.state {
                QueueState::Failed(reason) => Some((item.id, reason.clone())),
                _ => None,
            })
            .collect()
    }

    /// Whether the run is over: nothing pending, nothing running. A queue
    /// with failures is still over - the failures are the point of
    /// `failed`, not of another pass.
    pub fn is_complete(&self) -> bool {
        self.items
            .iter()
            .all(|item| !matches!(item.state, QueueState::Pending | QueueState::Running))
    }
}

impl fmt::Display for QueueState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueState::Pending => write!(f, "pending"),
            QueueState::Running => write!(f, "running"),
            QueueState::Done => write!(f, "done"),
            QueueState::Failed(reason) => write!(f, "failed: {reason}"),
        }
    }
}
