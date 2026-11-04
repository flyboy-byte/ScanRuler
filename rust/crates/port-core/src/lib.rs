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
pub mod adjacency;
pub mod bounds;
pub mod centroid;
pub mod clip;
pub mod closed_shells;
pub mod components;
pub mod curvature_gate;
pub mod linear;
pub mod marked_plane;
pub mod mesh_graph;
pub mod mesh_selection;
pub mod normals;
mod numeric;
pub mod orientation;
pub mod plane;
pub mod ransac;
pub mod weld;

/// Source-compatible number text shared by parser and command errors.
pub mod js_number;

/// Own Date millisecond clipping and Gregorian UTC timestamp conversion.
pub mod js_date;

/// Exact source fixed-three-decimal number text.
pub mod js_fixed;
/// Complete fixed-selection free/held-radius sphere fitting.
pub mod sphere;

/// Lossless Unicode command-label text and source-compatible provider ordering.
pub mod js_string;
/// Complete marked-sphere element output/refusal continuation.
pub mod marked_sphere;
/// Shared fixed-selection marking-size guard.
pub mod selection;

/// Prepared scan rendering and source corner-slot/crease continuation.
pub mod crease;

/// Source Math.hypot boundary shared by native query/command adapters.
/// Reuses the existing scaled compensated implementation used by the kernels.
pub fn hypot3(v: [f64; 3]) -> f64 {
    numeric::norm3(v)
}

/// Four-component source Math.hypot boundary for quaternion normalization.
pub fn hypot4(v: [f64; 4]) -> f64 {
    let abs = v.map(f64::abs);
    if abs.iter().any(|v| v.is_infinite()) {
        return f64::INFINITY;
    }
    if abs.iter().any(|v| v.is_nan()) {
        return f64::NAN;
    }
    let max = abs[0].max(abs[1]).max(abs[2]).max(abs[3]);
    if max == 0. {
        return 0.;
    }
    let mut sum = 0.;
    let mut compensation = 0.;
    for v in abs {
        let scaled = v / max;
        let summand = scaled * scaled - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}
