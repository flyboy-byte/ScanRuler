// SPDX-License-Identifier: AGPL-3.0-only
//! Lifetime/reaction integration; complete original traces remain in deadline/execute suites.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    command_deadline::Completion,
    reactions::{self, Limited, StartError},
    tasks::{Error, Limits, Owner},
};
use std::{
    cell::{Cell, RefCell},
    future::poll_fn,
    rc::Rc,
    sync::Arc,
    task::{Poll, Waker},
};
fn owner(tasks: usize) -> Owner<'static> {
    Owner::new(Limits {
        tasks,
        depth: 8,
        polls_per_turn: 32,
    })
    .unwrap()
}
struct Fault(Rc<Cell<usize>>);
impl Drop for Fault {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn timeout_answers_once_but_retains_real_late_future_and_consumes_its_fault() {
    let owner = owner(4);
    let log = Rc::new(RefCell::new(Vec::new()));
    let clear_log = log.clone();
    let limited = Limited::<(), Fault>::new(&owner, &[65], 1., 1024, move |_| {
        clear_log.borrow_mut().push("clear")
    })
    .unwrap();
    limited.registered(1).unwrap();
    let producer = limited.clone();
    let ready = Rc::new(Cell::new(false));
    let wake = Rc::new(RefCell::new(None::<Waker>));
    let drops = Rc::new(Cell::new(0));
    let (body_ready, body_wake, body_log, body_drops) =
        (ready.clone(), wake.clone(), log.clone(), drops.clone());
    let factory_log = log.clone();
    // Factory starts synchronously. Its future and receiver remain owner-held after ticket drop.
    drop(
        reactions::retain(
            &owner,
            move || {
                factory_log.borrow_mut().push("factory");
                poll_fn(move |cx| {
                    if body_ready.get() {
                        body_log.borrow_mut().push("late body effect");
                        Poll::Ready(Err(Fault(body_drops.clone())))
                    } else {
                        *body_wake.borrow_mut() = Some(cx.waker().clone());
                        Poll::Pending
                    }
                })
            },
            move |result| {
                assert!(!producer.body_settled(result));
            },
        )
        .unwrap(),
    );
    assert_eq!(&*log.borrow(), &["factory"]);
    limited.factory_returned().unwrap();
    owner.turn().unwrap();
    let event = limited.fire_begin().unwrap().unwrap();
    let reason = event.reason().clone();
    limited.fire_end(event).unwrap();
    assert!(limited.take_outcome().unwrap().is_none());
    owner.turn().unwrap();
    let Some(Completion::Timeout(error)) = limited.take_outcome().unwrap() else {
        panic!("missing timeout reply")
    };
    assert!(Arc::ptr_eq(&reason, &error));
    assert!(limited.take_outcome().unwrap().is_none());
    assert_eq!(owner.active(), 2);
    assert_eq!(owner.finish(), Err(Error::Busy));
    drop(limited);
    ready.set(true);
    wake.borrow_mut().take().unwrap().wake();
    let turn = owner.turn().unwrap();
    assert_eq!(turn.active, 0);
    assert_eq!(drops.get(), 1);
    assert_eq!(&*log.borrow(), &["factory", "clear", "late body effect"]);
    owner.finish().unwrap();
}

#[test]
fn abort_resolution_wins_without_running_finally_inside_abort_dispatch() {
    let owner = owner(1);
    let clears = Rc::new(Cell::new(0));
    let count = clears.clone();
    let limited =
        Limited::<u32, ()>::new(&owner, &[65], 1., 1024, move |_| count.set(count.get() + 1))
            .unwrap();
    limited.registered(4).unwrap();
    limited.factory_returned().unwrap();
    let event = limited.fire_begin().unwrap().unwrap();
    assert!(limited.body_settled(Ok(42)));
    // Even explicit reentrant host pumping cannot enter finally during abort dispatch.
    owner.turn().unwrap();
    assert_eq!(clears.get(), 0);
    assert!(limited.take_outcome().unwrap().is_none());
    limited.fire_end(event).unwrap();
    owner.turn().unwrap();
    assert_eq!(clears.get(), 1);
    assert!(matches!(
        limited.take_outcome().unwrap(),
        Some(Completion::Body(Ok(42)))
    ));
    owner.finish().unwrap();
}

#[test]
fn factory_array_ties_keep_body_first_and_cleanup_is_asynchronous() {
    for timer_first in [false, true] {
        let owner = owner(1);
        let limited = Limited::<u32, ()>::new(&owner, &[65], 1., 1024, |_| {}).unwrap();
        limited.registered(1).unwrap();
        if !timer_first {
            limited.body_settled(Ok(7));
        }
        let event = limited.fire_begin().unwrap().unwrap();
        limited.fire_end(event).unwrap();
        if timer_first {
            limited.body_settled(Ok(7));
        }
        limited.factory_returned().unwrap();
        assert!(limited.take_outcome().unwrap().is_none());
        owner.turn().unwrap();
        assert!(matches!(
            limited.take_outcome().unwrap(),
            Some(Completion::Body(Ok(7)))
        ));
        owner.finish().unwrap();
    }
}

#[test]
fn ready_failure_is_delivered_once_to_its_captured_receiver_after_factory_returns() {
    let owner = owner(2);
    let first = Rc::new(Cell::new(0));
    let second = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    for target in [first.clone(), second.clone()] {
        let fault = Fault(drops.clone());
        drop(
            reactions::retain(
                &owner,
                move || std::future::ready(Err::<(), _>(fault)),
                move |result| {
                    assert!(result.is_err());
                    target.set(target.get() + 1);
                },
            )
            .unwrap(),
        );
    }
    assert_eq!((first.get(), second.get()), (0, 0));
    owner.turn().unwrap();
    assert_eq!((first.get(), second.get()), (1, 1));
    assert_eq!(drops.get(), 2);
    owner.turn().unwrap();
    assert_eq!((first.get(), second.get()), (1, 1));
    owner.finish().unwrap();
}

#[test]
fn exhausted_owner_cannot_start_factory_or_acquire_timer_side_effects() {
    let owner = owner(0);
    let called = Rc::new(Cell::new(false));
    let factory_called = called.clone();
    assert!(matches!(
        reactions::retain(
            &owner,
            move || {
                factory_called.set(true);
                std::future::ready(())
            },
            |_| panic!("unadmitted receiver")
        ),
        Err(Error::Limit)
    ));
    assert!(!called.get());
    assert!(matches!(
        Limited::<(), ()>::new(&owner, &[65], 1., 1024, |_| panic!("no timer")),
        Err(StartError::Owner(Error::Limit))
    ));
    assert_eq!(owner.active(), 0);
    owner.finish().unwrap();
}
