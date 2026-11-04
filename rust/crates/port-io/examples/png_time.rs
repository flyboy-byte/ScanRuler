// SPDX-License-Identifier: AGPL-3.0-only
//! Times one decode: `cargo run --release -p port-io --example png_time -- file.png`.
fn main() {
    let path = std::env::args().nth(1).unwrap_or_default();
    let bytes = std::fs::read(&path).unwrap_or_default();
    let t = std::time::Instant::now();
    let r = port_io::png::decode_png(&bytes, port_io::png::PngLimits::default());
    println!("{:?} in {:?}", r.map(|i| (i.width, i.height)), t.elapsed());
}
