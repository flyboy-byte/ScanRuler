// SPDX-License-Identifier: AGPL-3.0-only
//! `scanruler-rs`: the small native front end used for differential runs and headless work.

fn main() {
    println!(
        "scanruler-rs {}: port of ScanRuler {} ({}); nothing is ported yet.",
        env!("CARGO_PKG_VERSION"),
        port_core::REFERENCE_APP_VERSION,
        &port_core::REFERENCE_SHA[..7],
    );
}
