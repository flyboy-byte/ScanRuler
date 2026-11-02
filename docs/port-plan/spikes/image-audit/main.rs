// SPDX-License-Identifier: AGPL-3.0-only
//! Temporary native candidate probe; no workspace dependency or product command.
#![forbid(unsafe_code)]
mod decoder;
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args_os()
        .nth(1)
        .ok_or("expected authored input path")?;
    let bytes = std::fs::read(file)?;
    std::io::stdout().write_all(&decoder::decode(&bytes))?;
    Ok(())
}
