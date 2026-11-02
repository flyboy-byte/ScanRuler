// SPDX-License-Identifier: AGPL-3.0-only
//! File formats and the `.scanruler` project reader/writer. Parsers return owned data and
//! never touch a live session. STL binary/ASCII decoding lives here (io.stl.binary/io.stl.dispatch, under P06).

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
