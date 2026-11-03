// SPDX-License-Identifier: AGPL-3.0-only
//! limited() from pinned registry.ts, at the owned timer/body event boundary.
//! The runtime retains and drives body work after caller timeout. Timer backend
//! scheduling/normalization and task driving remain separate host responsibilities.
use crate::{
    error::{CommandError, CommandErrorCode, EnvelopeError},
    schema_value::Text,
};
use std::sync::Arc;
/// Original default timeout, ten minutes in milliseconds.
pub const COMMAND_TIMEOUT_MS: u64 = 10 * 60_000;
const MIDDLE: &str = " did not finish in ";
const END: &str = " s and was given up on. It may have changed the session in part — read session.state (history.undo says the last step) before running it again.";
/// Host protocol/metadata failures are distinct from original command errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlineError {
    /// Error-envelope metadata budget/allocation failure.
    Envelope(EnvelopeError),
    /// Event arrived in the wrong owned phase.
    InvalidState,
    /// Timer completion token belongs to another deadline or was repeated.
    InvalidToken,
}
/// Caller result. Body faults retain their original value, while timeout and
/// abort share one immutable CommandError instance.
#[derive(Debug)]
pub enum Completion<T, E> {
    /// The command body won the race, successfully or with its own fault.
    Body(Result<T, E>),
    /// The timer rejection won the race.
    Timeout(Arc<CommandError>),
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Register,
    Factory,
    Racing,
    Won,
    Clearing,
    Complete,
}
/// Capability for the synchronous abort dispatch inside the timer callback.
/// Finish this dispatch before allowing microtask cleanup to run.
#[derive(Debug, Clone)]
pub struct AbortEvent {
    domain: Arc<()>,
    reason: Arc<CommandError>,
}
impl AbortEvent {
    /// Exact reason shared with the signal and any timeout caller response.
    pub fn reason(&self) -> &Arc<CommandError> {
        &self.reason
    }
}
/// Dependency-free deadline race state. A serial clock owner registers one
/// timer before invoking the body factory. Body results are delivered in their
/// actual reaction order, with factory registration retaining array-order ties.
pub struct Deadline<T, E> {
    name: Text,
    ms: f64,
    max_units: usize,
    message_capacity: usize,
    domain: Arc<()>,
    timer: Option<u64>,
    phase: Phase,
    body_done: bool,
    body: Option<Result<T, E>>,
    late_ready: bool,
    winner: Option<Completion<T, E>>,
    fired: bool,
    cleared: bool,
    firing: bool,
    reason: Option<Arc<CommandError>>,
}
impl<T, E> Deadline<T, E> {
    /// Admit metadata before acquiring a host timer. The conservative 32-unit
    /// number bound covers all IEEE Number text; no timeout arithmetic moves
    /// ahead of the original timer callback.
    pub fn new(name: &[u16], ms: f64, max_units: usize) -> Result<Self, DeadlineError> {
        let bound = name
            .len()
            .checked_add(MIDDLE.encode_utf16().count())
            .and_then(|n| n.checked_add(END.encode_utf16().count()))
            .and_then(|n| n.checked_add(32))
            .ok_or(DeadlineError::Envelope(EnvelopeError::LengthOverflow))?;
        if bound > max_units {
            return Err(DeadlineError::Envelope(EnvelopeError::MessageLimit));
        }
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(name.len())
            .map_err(|_| DeadlineError::Envelope(EnvelopeError::Allocation))?;
        owned.extend_from_slice(name);
        Ok(Self {
            name: owned,
            ms,
            max_units,
            message_capacity: bound,
            domain: Arc::new(()),
            timer: None,
            phase: Phase::Register,
            body_done: false,
            body: None,
            late_ready: false,
            winner: None,
            fired: false,
            cleared: false,
            firing: false,
            reason: None,
        })
    }
    /// Original raw milliseconds passed to the timer backend, without inventing
    /// a cross-platform clamp policy.
    pub fn milliseconds(&self) -> f64 {
        self.ms
    }
    /// A trusted serial timer has been registered; the body factory may now run.
    pub fn registered(&mut self, timer: u64) -> Result<(), DeadlineError> {
        if self.phase != Phase::Register {
            return Err(DeadlineError::InvalidState);
        }
        self.timer = Some(timer);
        self.phase = Phase::Factory;
        Ok(())
    }
    /// Body promise settled, even if the caller already has its answer. Returns
    /// true only when the value participates in the race; late/duplicate results
    /// are consumed without becoming another answer or an unhandled rejection.
    pub fn body_settled(&mut self, result: Result<T, E>) -> bool {
        if self.body_done {
            return false;
        }
        self.body_done = true;
        if self.phase == Phase::Register || self.phase == Phase::Factory {
            self.body = Some(result);
            true
        } else if self.phase == Phase::Racing {
            self.winner = Some(Completion::Body(result));
            self.phase = Phase::Won;
            true
        } else {
            false
        }
    }
    /// run() returned its ordinary promise (or threw and was converted to a
    /// rejected promise). Attach body/race handlers. Already-settled body wins
    /// the [ran, late] array tie, even if the timer fired during the factory.
    pub fn factory_returned(&mut self) -> Result<(), DeadlineError> {
        if self.phase != Phase::Factory {
            return Err(DeadlineError::InvalidState);
        }
        if let Some(result) = self.body.take() {
            self.winner = Some(Completion::Body(result));
            self.phase = Phase::Won
        } else if self.late_ready {
            self.winner = Some(Completion::Timeout(
                self.reason
                    .as_ref()
                    .ok_or(DeadlineError::InvalidState)?
                    .clone(),
            ));
            self.phase = Phase::Won
        } else {
            self.phase = Phase::Racing
        }
        Ok(())
    }
    /// Timer callback begins: construct the original error and abort first.
    /// Dispatch synchronous abort listeners using the returned reason before
    /// completing the timer rejection. Ignored/cleared timers do nothing.
    pub fn fire_begin(&mut self) -> Result<Option<AbortEvent>, DeadlineError> {
        if self.timer.is_none() || self.fired || self.cleared {
            return Ok(None);
        }
        self.fired = true;
        let mut message = Vec::new();
        message
            .try_reserve_exact(self.message_capacity)
            .map_err(|_| DeadlineError::Envelope(EnvelopeError::Allocation))?;
        message.extend_from_slice(&self.name);
        message.extend(MIDDLE.encode_utf16());
        let seconds = port_core::js_number::number_string(self.ms / 1000.);
        message.extend(seconds.encode_utf16());
        message.extend(END.encode_utf16());
        let reason = Arc::new(
            CommandError::from_utf16(CommandErrorCode::Timeout, &message, self.max_units)
                .map_err(DeadlineError::Envelope)?,
        );
        self.reason = Some(reason.clone());
        self.firing = true;
        Ok(Some(AbortEvent {
            domain: self.domain.clone(),
            reason,
        }))
    }
    /// Finish synchronous abort delivery and reject late. A body settled by an
    /// abort listener before this point retains the original reaction priority.
    pub fn fire_end(&mut self, event: AbortEvent) -> Result<(), DeadlineError> {
        if !Arc::ptr_eq(&event.domain, &self.domain) || !self.firing {
            return Err(DeadlineError::InvalidToken);
        }
        self.firing = false;
        self.late_ready = true;
        if self.phase == Phase::Racing {
            self.winner = Some(Completion::Timeout(event.reason));
            self.phase = Phase::Won
        }
        Ok(())
    }
    /// Promise.finally cleanup, after queued race reactions and after the timer
    /// callback returns. The backend clears even an already-fired timer handle.
    pub fn begin_clear(&mut self) -> Option<u64> {
        if self.phase != Phase::Won || self.firing {
            return None;
        }
        self.phase = Phase::Clearing;
        self.timer
    }
    /// Trusted synchronous clearTimeout completed; publish the caller's result.
    pub fn cleared(&mut self) -> Result<(), DeadlineError> {
        if self.phase != Phase::Clearing {
            return Err(DeadlineError::InvalidState);
        }
        self.cleared = true;
        self.phase = Phase::Complete;
        Ok(())
    }
    /// Signal reason, including a timeout whose rejection lost the race.
    pub fn abort_reason(&self) -> Option<&Arc<CommandError>> {
        self.reason.as_ref()
    }
    /// Whether the timer could still fire.
    pub fn timer_active(&self) -> bool {
        self.timer.is_some() && !self.fired && !self.cleared
    }
    /// Completed caller result; body ownership is independent of this readout.
    pub fn outcome(&self) -> Option<&Completion<T, E>> {
        if self.phase == Phase::Complete {
            self.winner.as_ref()
        } else {
            None
        }
    }
    /// Transfer the single owned caller result. Late body results remain heard.
    pub fn take_outcome(&mut self) -> Option<Completion<T, E>> {
        if self.phase == Phase::Complete {
            self.winner.take()
        } else {
            None
        }
    }
}
