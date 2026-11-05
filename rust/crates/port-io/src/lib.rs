// SPDX-License-Identifier: AGPL-3.0-only
//! File formats and the `.scanruler` project reader/writer. Parsers return owned data and
//! never touch a live session. STL binary/ASCII decoding lives here (io.stl.binary/io.stl.dispatch, under P06).

/// DEFLATE and ZIP reading that reproduces the original's fflate behaviour.
pub mod inflate;
/// Complete OBJ geometry/decoded-texture parser.
pub mod obj;
/// Source OBJ material/decoded-image texture helpers; no image codec dependency.
pub mod obj_texture;
pub mod stl;
pub mod stl_dispatch;

/// All original PLY encodings through one dispatcher.
pub mod ply;
/// Source-faithful ASCII PLY import; binary is an explicit unported boundary.
pub mod ply_ascii;
mod ply_binary;

/// Owned file selection, companion identity and byte snapshots; no image codec.
pub mod scan_files;

/// Complete dropped-folder async callback traversal; no DOM adapter or decoder.
pub mod drop_files;

/// 3MF import (zip container, units, components, slicer parts, colour and textures).
pub mod threemf;

pub mod png;
