// SPDX-License-Identifier: AGPL-3.0-only
//! File formats and the `.scanruler` project reader/writer. Parsers return owned data and
//! never touch a live session. STL binary/ASCII decoding lives here (io.stl.binary/io.stl.dispatch, under P06).

pub mod stl;
pub mod stl_dispatch;
