// SPDX-License-Identifier: AGPL-3.0-only
//! Candidate-only dependency audit. Not an application decoder or source port.
#![forbid(unsafe_code)]
use image::{ImageError, ImageReader, Limits};
use std::io::Cursor;
fn failure(tag: u8, message: &str) -> Vec<u8> {
    let mut out = b"IMG1".to_vec();
    out.push(tag);
    out.extend((message.len() as u32).to_le_bytes());
    out.extend(message.as_bytes());
    out
}
/// Decode fixed small authored audit fixtures. This helper deliberately does not
/// simulate browser canvas, apply source-derived answer tables or claim parity.
/// Resource limits here are prototype safeguards, not product acceptance.
pub fn decode(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() > 1 << 20 {
        return failure(2, "audit input limit");
    }
    let mut reader = match ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
        Ok(reader) => reader,
        Err(error) => return failure(0, &error.to_string()),
    };
    let mut limits = Limits::default();
    limits.max_image_width = Some(1024);
    limits.max_image_height = Some(1024);
    limits.max_alloc = Some(16 << 20);
    reader.limits(limits);
    match reader.decode() {
        Ok(image) => {
            let rgba = image.to_rgba8();
            let mut out = b"IMG1".to_vec();
            out.push(1);
            out.extend(rgba.width().to_le_bytes());
            out.extend(rgba.height().to_le_bytes());
            out.extend((rgba.len() as u32).to_le_bytes());
            out.extend(rgba.as_raw());
            out
        }
        Err(error @ ImageError::Limits(_)) => failure(2, &error.to_string()),
        Err(error) => failure(0, &error.to_string()),
    }
}
