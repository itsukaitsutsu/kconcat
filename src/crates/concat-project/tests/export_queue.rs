//! V4: the batch export queue (owner A11, 16 Oct 2026).
//!
//! Done-when, in the sandbox's reach: five shorts export in one pass
//! without babysitting, with the order the user chose, and a failed item
//! does not stop the rest. The pass itself runs on the owner's machine -
//! the runner is a separate process beside FFmpeg - so what is tested
//! here is every promise the queue makes about that run: the order, the
//! one-at-a-time hand-off, the failure that does not stop the queue, and
//! the reordering that stays the user's while the run is on.

use concat_project::queue::{Direction, ExportQueue, QueueState};

#[test]
fn five_shorts_run_in_the_order_the_user_chose() {
    let mut queue = ExportQueue::new();
    let ids: Vec<u64> = (1..=5)
        .map(|n| queue.enqueue(format!("short {n}")))
        .collect();

    // The user's choice: short 3 first, then the rest as queued. A11: the
    // order is the user's, before or during the run.
    assert!(queue.move_item(ids[2], Direction::Up));
    assert!(queue.move_item(ids[2], Direction::Up));

    let mut ran: Vec<u64> = Vec::new();
    while let Some(id) = queue.start_next() {
        ran.push(id);
        assert!(queue.finish(id, Ok(())), "the runner's own item finishes");
    }
    assert_eq!(ran, vec![ids[2], ids[0], ids[1], ids[3], ids[4]]);
    assert!(queue.is_complete());
    assert!(queue.failed().is_empty());
    assert!(
        queue
            .items()
            .iter()
            .all(|item| item.state == QueueState::Done),
        "{:?}",
        queue.items()
    );
}

#[test]
fn a_failed_item_does_not_stop_the_rest() {
    let mut queue = ExportQueue::new();
    let first = queue.enqueue("short 1");
    let second = queue.enqueue("short 2");
    let third = queue.enqueue("short 3");

    assert_eq!(queue.start_next(), Some(first));
    assert!(queue.finish(first, Err("encoder died".to_owned())));

    // The queue continues: the failure is marked, and the next item runs.
    assert_eq!(queue.start_next(), Some(second));
    assert!(queue.finish(second, Ok(())));
    assert_eq!(queue.start_next(), Some(third));
    assert!(queue.finish(third, Ok(())));

    assert!(queue.is_complete(), "failures do not hold the queue open");
    assert_eq!(queue.failed(), vec![(first, "encoder died".to_owned())]);
    let states: Vec<&QueueState> = queue.items().iter().map(|item| &item.state).collect();
    assert_eq!(
        states,
        vec![
            &QueueState::Failed("encoder died".to_owned()),
            &QueueState::Done,
            &QueueState::Done
        ]
    );
}

#[test]
fn one_item_runs_at_a_time() {
    let mut queue = ExportQueue::new();
    let first = queue.enqueue("short 1");
    queue.enqueue("short 2");

    assert_eq!(queue.start_next(), Some(first));
    assert_eq!(
        queue.start_next(),
        None,
        "the runner is one encoder process: nothing else starts while one runs"
    );
    assert!(queue.finish(first, Ok(())));
    assert!(queue.start_next().is_some(), "and the queue goes on after");
}

#[test]
fn reordering_stays_the_users_while_the_run_is_on() {
    let mut queue = ExportQueue::new();
    let first = queue.enqueue("short 1");
    let second = queue.enqueue("short 2");
    let third = queue.enqueue("short 3");

    assert_eq!(queue.start_next(), Some(first));

    // A11: items move "before or during the run" - the pending ones.
    assert!(queue.move_item(third, Direction::Up), "during the run");
    assert_eq!(
        queue.items().iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![first, third, second]
    );

    // The running item does not move - it is in the runner's hands.
    assert!(!queue.move_item(first, Direction::Down));
    // The edge of the list does not wrap.
    assert!(!queue.move_item(third, Direction::Up));
    assert!(queue.finish(first, Ok(())));

    // A finished item is history: it stays where it ended.
    assert!(!queue.move_item(first, Direction::Down));
    assert_eq!(queue.items()[0].id, first);
    // An unknown id is refused, not a panic.
    assert!(!queue.move_item(999, Direction::Up));
}

#[test]
fn a_late_report_does_not_rewrite_the_item_that_replaced_it() {
    let mut queue = ExportQueue::new();
    let first = queue.enqueue("short 1");
    let second = queue.enqueue("short 2");

    assert_eq!(queue.start_next(), Some(first));
    assert!(queue.finish(first, Err("killed".to_owned())));
    assert_eq!(queue.start_next(), Some(second));

    // A runner that reports late for the item it was killed on must not
    // touch the one now encoding.
    assert!(!queue.finish(first, Ok(())));
    assert_eq!(
        queue
            .items()
            .iter()
            .find(|item| item.id == first)
            .map(|item| &item.state),
        Some(&QueueState::Failed("killed".to_owned()))
    );
    assert_eq!(
        queue
            .items()
            .iter()
            .find(|item| item.id == second)
            .map(|item| &item.state),
        Some(&QueueState::Running)
    );
}
