// SPDX-License-Identifier: AGPL-3.0-only
//! Regression for the 2026-11-02 independent fuzz audit (divergence D1): the original prints the
//! negative typed-array length with JavaScript number formatting. The expected strings are the
//! original's recorded output (Node 22, pinned parser), not values produced by this port.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use port_io::ply::{PlyError, PlyLimits, read_ply};

fn limits() -> PlyLimits {
    PlyLimits {
        max_bytes: 2_000_000,
        max_vertices: 100_000,
        max_triangles: 100_000,
        max_work: 1_000_000,
    }
}

fn header(format: &str, count: &str) -> Vec<u8> {
    format!(
        "ply\nformat {format} 1.0\nelement vertex {count}\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n"
    )
    .into_bytes()
}

fn message(format: &str, count: &str) -> String {
    let err = read_ply(&header(format, count), limits(), |_| {}).unwrap_err();
    assert!(matches!(err, PlyError::InvalidArrayLength(_)), "{err:?}");
    err.to_string()
}

#[test]
fn negative_counts_use_javascript_number_text() {
    let huge = format!("-{}", "9".repeat(400));
    for format in ["ascii", "binary_little_endian", "binary_big_endian"] {
        assert_eq!(
            message(format, "-1000000000000000000000"),
            "Invalid typed array length: -3e+21",
            "{format}"
        );
        assert_eq!(
            message(format, &huge),
            "Invalid typed array length: -Infinity",
            "{format}"
        );
        assert_eq!(
            message(format, "-5"),
            "Invalid typed array length: -15",
            "{format}"
        );
    }
}
