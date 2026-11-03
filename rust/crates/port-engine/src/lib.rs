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

/// Pure stale-import classification, separate from page timestamps.
pub mod stale_import;

/// First-stale timestamp state, independent of import classification.
pub mod page_stale;
