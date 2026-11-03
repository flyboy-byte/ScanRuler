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

/// Session subscription snapshots, listener identity and ordered teardown.
pub mod watch_session;

/// Pure stale-import classification, separate from page timestamps.
pub mod stale_import;

/// First-stale timestamp state, independent of import classification.
pub mod page_stale;

/// Stale-page envelope construction with source-ordered host effects.
pub mod stale_error;

/// Command fault normalization preserving typed error identity and stale state.
pub mod error_normalize;

/// Ordered plugin busy registration and live first-busy lookup.
pub mod plugin_busy;

/// Live asynchronous plugin stopping, shared done capability and owned waits.
pub mod plugin_stop;

/// Direct command routing and serialized viewport turns.
pub mod scene_queue;

/// Execute preflight, provider wait ordering and shared completion lifetime.
pub mod execute;

/// Retained owned JSON-data copies, byte summaries and exact size thresholds.
pub mod recent_result;

/// Recent record/reaction state, bounded eviction and shallow readout snapshots.
pub mod recent_store;

/// Public/internal workspace names over current installed-plugin metadata.
pub mod workspaces;

/// Lossless Unicode command-label text and source-compatible provider ordering.
pub mod command_label;
/// Command-driven activity publication and live listeners.
pub mod activity_state;
