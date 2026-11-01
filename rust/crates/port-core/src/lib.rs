// SPDX-License-Identifier: AGPL-3.0-only
//! Pure core of the Rust port: units, identifiers and rigid transforms.
//!
//! No DOM, UI, filesystem or process state lives here (ARCHITECTURE.md, "Keep the first
//! workspace small"). Measurement mathematics lands in this crate as units P05 onward are
//! compared against the pinned original; until then only contracts that every later unit
//! shares are defined.

pub mod ids;
pub mod transform;
pub mod units;

/// The upstream commit this port is pinned to. `cargo xtask pin` checks it against
/// `docs/port-plan/reference-pin.json`.
pub const REFERENCE_SHA: &str = "2cd89c6769a10c27ee7754b477ada521c9ef5cb7";

/// The upstream application version at [`REFERENCE_SHA`].
pub const REFERENCE_APP_VERSION: &str = "0.9.2";
pub mod linear;
pub mod normals;
pub mod orientation;
pub mod weld;
