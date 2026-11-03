// SPDX-License-Identifier: AGPL-3.0-only
//! Session, commands, jobs and history. Typed error envelopes exist; session
//! ownership, registry execution and history remain unimplemented. Contracts are in
//! `docs/port-plan/ARCHITECTURE.md` ("Commands, history and background work").

pub mod error;

pub mod schema;
pub mod schema_value;

pub mod registry;

pub mod history_record;

/// Local editor history selection and dispatch.
pub mod history_local;

/// History travel blocking and editor-work predicates.
pub mod history_guards;

/// Serialized import/restore jobs and source-compatible listener notification.
pub mod import_queue;

/// Command deadline races and abort/timer cleanup at the owned event boundary.
pub mod command_deadline;

/// Deferred-work settlement over stable ordered callbacks.
pub mod settle;

/// Function identity and live registration for deferred-work callbacks.
pub mod settle_registry;

/// Command guard identity, configuration and explicit cleanup.
pub mod activity_guard;

/// Session watched-store source identity and conditional release.
pub mod watch_stores;
