// SPDX-License-Identifier: AGPL-3.0-only
//! `scanruler-rs`: the small native front end used for differential runs and headless work.

fn main() {
    println!(
        "scanruler-rs {}: ScanRuler {} library port ({}); scan/fit/undo/save CLI is not implemented.",
        env!("CARGO_PKG_VERSION"),
        port_core::REFERENCE_APP_VERSION,
        &port_core::REFERENCE_SHA[..7],
    );
}
