// SPDX-License-Identifier: AGPL-3.0-only
//! File formats and the `.scanruler` project reader/writer. Parsers return owned data and
//! never touch a live session. First resident: binary STL triangle decoding (io.stl.binary, under P06).

pub mod stl;
