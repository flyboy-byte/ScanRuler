// SPDX-License-Identifier: AGPL-3.0-only
//! Native host lifetime/scheduling tests; not original command-runtime parity.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::tasks::{Error, Limits, Owner};
use std::{
    cell::{Cell, RefCell},
    future::{Future, poll_fn},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
fn owner(tasks: usize, depth: usize, polls_per_turn: usize) -> Owner<'static> {
    Owner::new(Limits {
        tasks,
        depth,
        polls_per_turn,
    })
    .unwrap()
}
#[derive(Clone, Default)]
struct Gate {
    ready: Rc<Cell<bool>>,
    wake: Rc<RefCell<Option<Waker>>>,
}
impl Gate {
    fn open(&self) {
        self.ready.set(true);
        if let Some(wake) = self.wake.borrow_mut().take() {
            wake.wake()
        }
    }
}
impl Future for Gate {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.ready.get() {
            Poll::Ready(())
        } else {
            *self.wake.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
#[test]
fn dropped_caller_ticket_does_not_cancel_late_effects_or_fault_handling() {
    #[derive(Debug)]
    struct Opaque {
        _opaque: bool,
    }
    let owner = owner(4, 4, 10);
    let gate = Gate::default();
    let body_gate = gate.clone();
    let effects = Rc::new(RefCell::new(vec![]));
    let log = effects.clone();
    let fault = Rc::new(Opaque { _opaque: true });
    let actual = fault.clone();
    let caught = Rc::new(RefCell::new(None));
    let retained = caught.clone();
    let ticket = owner
        .spawn(async move {
            log.borrow_mut().push("start");
            body_gate.await;
            log.borrow_mut().push("late mutation");
            let result: Result<(), _> = Err(actual);
            *retained.borrow_mut() = result.err();
            log.borrow_mut().push("late fault consumed")
        })
        .unwrap();
    assert_eq!(&*effects.borrow(), &["start"]);
    assert!(!ticket.complete());
    drop(ticket);
    let log = effects.clone();
    assert!(
        owner
            .spawn(async move { log.borrow_mut().push("caller timeout") })
            .unwrap()
            .complete()
    );
    assert_eq!(owner.active(), 1);
    assert_eq!(owner.finish(), Err(Error::Busy));
    assert_eq!(owner.turn().unwrap().polls, 0);
    gate.open();
    let turn = owner.turn().unwrap();
    assert_eq!((turn.polls, turn.active, turn.ready), (1, 0, 0));
    assert!(Rc::ptr_eq(caught.borrow().as_ref().unwrap(), &fault));
    assert_eq!(
        &*effects.borrow(),
        &[
            "start",
            "caller timeout",
            "late mutation",
            "late fault consumed"
        ]
    );
    assert_eq!(owner.finish(), Ok(()));
}
#[test]
fn fifo_wakes_coalesce_and_turns_are_finite_and_fair() {
    let owner = owner(2, 4, 2);
    let log = Rc::new(RefCell::new(vec![]));
    let counters = [Rc::new(Cell::new(0)), Rc::new(Cell::new(0))];
    let mut tickets = vec![];
    for (index, count) in counters.iter().enumerate() {
        let count = count.clone();
        let log = log.clone();
        tickets.push(
            owner
                .spawn(poll_fn(move |cx| {
                    log.borrow_mut().push(index);
                    let n = count.get();
                    count.set(n + 1);
                    if n == 3 {
                        Poll::Ready(())
                    } else {
                        cx.waker().wake_by_ref();
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }))
                .unwrap(),
        )
    }
    assert_eq!(&*log.borrow(), &[0, 1]);
    for remaining in [2, 2, 0] {
        let turn = owner.turn().unwrap();
        assert_eq!(turn.polls, 2);
        assert_eq!(turn.ready, remaining);
        assert_eq!(turn.active, remaining)
    }
    assert_eq!(&*log.borrow(), &[0, 1, 0, 1, 0, 1, 0, 1]);
    assert!(tickets.iter().all(|t| t.complete()));
}
#[test]
fn external_wake_order_and_stale_wakers_cannot_target_reused_slots() {
    let owner = owner(2, 4, 10);
    let log = Rc::new(RefCell::new(vec![]));
    let ready = Rc::new(Cell::new(false));
    let wakes = Rc::new(RefCell::new(vec![]));
    for index in 0..2 {
        let log = log.clone();
        let ready = ready.clone();
        let wakes = wakes.clone();
        drop(
            owner
                .spawn(poll_fn(move |cx| {
                    if ready.get() {
                        log.borrow_mut().push(index);
                        Poll::Ready(())
                    } else {
                        wakes.borrow_mut().push(cx.waker().clone());
                        Poll::Pending
                    }
                }))
                .unwrap(),
        )
    }
    ready.set(true);
    wakes.borrow()[1].wake_by_ref();
    wakes.borrow()[1].wake_by_ref();
    wakes.borrow()[0].wake_by_ref();
    assert_eq!(owner.turn().unwrap().polls, 2);
    assert_eq!(&*log.borrow(), &[1, 0]);
    let gate = Gate::default();
    let ticket = owner.spawn(gate.clone()).unwrap();
    for wake in wakes.borrow().iter() {
        wake.wake_by_ref()
    }
    assert_eq!(owner.turn().unwrap().polls, 0);
    assert!(!ticket.complete());
    gate.open();
    assert_eq!(owner.turn().unwrap().polls, 1);
    assert!(ticket.complete());
}
#[test]
fn cross_thread_wakes_only_deliver_tokens_to_the_owner() {
    let owner = owner(1, 2, 10);
    let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal = ready.clone();
    let wake = Rc::new(RefCell::new(None));
    let saved = wake.clone();
    let seen = Rc::new(Cell::new(0));
    let completed = seen.clone();
    let ticket = owner
        .spawn(poll_fn(move |cx| {
            if signal.load(std::sync::atomic::Ordering::Acquire) {
                completed.set(37);
                Poll::Ready(())
            } else {
                *saved.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
        }))
        .unwrap();
    let waker = wake.borrow_mut().take().unwrap();
    std::thread::spawn(move || {
        ready.store(true, std::sync::atomic::Ordering::Release);
        for _ in 0..100 {
            waker.wake_by_ref()
        }
    })
    .join()
    .unwrap();
    assert_eq!(seen.get(), 0);
    assert!(!ticket.complete());
    assert_eq!(owner.turn().unwrap().polls, 1);
    assert_eq!(seen.get(), 37);
    assert!(ticket.complete());
}
#[test]
fn reentrant_spawn_has_no_held_owner_borrow_and_respects_depth() {
    let owner = owner(4, 2, 10);
    let log = Rc::new(RefCell::new(vec![]));
    let nested = owner.clone();
    let output = log.clone();
    let ticket = owner
        .spawn(async move {
            output.borrow_mut().push("outer");
            assert_eq!(nested.turn(), Err(Error::Reentrant));
            let deeper = nested.clone();
            let output = output.clone();
            let nested_ticket = nested
                .spawn(async move {
                    output.borrow_mut().push("inner");
                    assert!(matches!(deeper.spawn(async {}), Err(Error::Limit)))
                })
                .unwrap();
            assert!(nested_ticket.complete());
        })
        .unwrap();
    assert!(ticket.complete());
    assert_eq!(&*log.borrow(), &["outer", "inner"]);
    assert_eq!(owner.active(), 0);
    assert!(owner.spawn(async {}).unwrap().complete());
}
#[test]
fn bounded_admission_does_not_poll_refused_work_and_finish_never_cancels() {
    let owner = owner(1, 2, 1);
    let gate = Gate::default();
    let ticket = owner.spawn(gate.clone()).unwrap();
    let polled = Rc::new(Cell::new(false));
    let observed = polled.clone();
    assert!(matches!(
        owner.spawn(poll_fn(move |_| {
            observed.set(true);
            Poll::Ready(())
        })),
        Err(Error::Limit)
    ));
    assert!(!polled.get());
    assert_eq!(owner.finish(), Err(Error::Busy));
    gate.open();
    assert_eq!(owner.turn().unwrap().polls, 1);
    assert!(ticket.complete());
    assert_eq!(owner.finish(), Ok(()));
    assert!(matches!(owner.spawn(async {}), Err(Error::Closed)));
    assert_eq!(owner.turn().unwrap().active, 0);
    assert!(matches!(
        Owner::new(Limits {
            tasks: 1,
            depth: 0,
            polls_per_turn: 1
        }),
        Err(Error::Limit)
    ));
    assert!(matches!(
        Owner::new(Limits {
            tasks: 1,
            depth: 1,
            polls_per_turn: 0
        }),
        Err(Error::Limit)
    ));
}
#[test]
fn task_future_can_borrow_the_session_without_send_or_static() {
    let seen = Cell::new(0);
    let owner = Owner::new(Limits {
        tasks: 2,
        depth: 2,
        polls_per_turn: 2,
    })
    .unwrap();
    let gate = Gate::default();
    let wait = gate.clone();
    let shared = &seen;
    let ticket = owner
        .spawn(async move {
            wait.await;
            shared.set(41)
        })
        .unwrap();
    assert_eq!(seen.get(), 0);
    gate.open();
    owner.turn().unwrap();
    assert!(ticket.complete());
    assert_eq!(seen.get(), 41);
}

#[test]
fn ready_self_wakes_are_retired_before_reentrant_slot_reuse() {
    let owner = owner(1, 3, 2);
    let old = Rc::new(RefCell::new(vec![]));
    for _ in 0..128 {
        let saved = old.clone();
        let ticket = owner
            .spawn(poll_fn(move |cx| {
                saved.borrow_mut().push(cx.waker().clone());
                cx.waker().wake_by_ref();
                Poll::Ready(())
            }))
            .unwrap();
        assert!(ticket.complete());
        for waker in old.borrow().iter() {
            waker.wake_by_ref()
        }
        let turn = owner.turn().unwrap();
        assert_eq!((turn.polls, turn.active, turn.ready), (0, 0, 0));
    }
    assert_eq!(owner.finish(), Ok(()));
}

#[test]
fn completed_future_destructor_can_submit_into_the_freed_slot() {
    struct OnDrop<'a> {
        owner: Owner<'a>,
        seen: Rc<Cell<usize>>,
    }
    impl Future for OnDrop<'_> {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
            Poll::Ready(())
        }
    }
    impl Drop for OnDrop<'_> {
        fn drop(&mut self) {
            let seen = self.seen.clone();
            assert!(
                self.owner
                    .spawn(async move { seen.set(37) })
                    .unwrap()
                    .complete()
            )
        }
    }
    let owner = owner(1, 2, 2);
    let seen = Rc::new(Cell::new(0));
    assert!(
        owner
            .spawn(OnDrop {
                owner: owner.clone(),
                seen: seen.clone()
            })
            .unwrap()
            .complete()
    );
    assert_eq!(seen.get(), 37);
    assert_eq!(owner.active(), 0);
    assert_eq!(owner.turn().unwrap().ready, 0);
}
