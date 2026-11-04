// SPDX-License-Identifier: AGPL-3.0-only
//! DEFLATE and ZIP reading that reproduces the original's `fflate` 0.8.3 `unzipSync` (the pinned
//! lockfile version), including its lenient behaviour on damaged input. The original's 3MF reader
//! calls it with a name filter (`src/core/parsers/threemf.ts`). Only reading is ported.
//!
//! Faithful quirks, all observed in `fflate/lib/index.cjs`:
//! - bytes beyond the input read as zero; stored blocks ignore `NLEN`;
//! - Huffman tables are built without validation, so over- or under-subscribed trees do not fail;
//! - the output buffer has the entry's declared size; bytes written past it are dropped, and the
//!   result is the first `min(written, declared)` bytes (an empty stream yields all zeros);
//! - distance symbol 31 copies zeros, and a stored block that does not fit is a JS `RangeError`.
//!
//! Rust-side host limits are reported as [`InflateError::ResourceLimit`] or
//! [`InflateError::AggregateLimit`], never as a source error.

use std::sync::OnceLock;

/// Errors: the original's own, plus an explicit host boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflateError {
    /// An `fflate` error: its numeric code and message.
    Fflate {
        /// `fflate` error code (0 unexpected EOF, 1 block type, 2 literal, 3 distance, 13 zip, 14 method).
        code: u8,
        /// Exact message text.
        message: String,
    },
    /// JS `RangeError: offset is out of bounds` from a stored block larger than the output.
    OffsetOutOfBounds,
    /// The caller's output or entry budget was exceeded.
    ResourceLimit,
    /// Cumulative retained-entry allocation budget exceeded before allocating the next entry.
    AggregateLimit,
}

impl std::fmt::Display for InflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fflate { message, .. } => f.write_str(message),
            Self::OffsetOutOfBounds => f.write_str("offset is out of bounds"),
            Self::ResourceLimit | Self::AggregateLimit => {
                f.write_str("Input exceeds the caller's resource budget.")
            }
        }
    }
}

fn fflate(code: u8) -> InflateError {
    let message = match code {
        0 => "unexpected EOF",
        1 => "invalid block type",
        2 => "invalid length/literal",
        3 => "invalid distance",
        13 => "invalid zip data",
        _ => "unknown error",
    };
    InflateError::Fflate {
        code,
        message: message.to_string(),
    }
}

/// Host budgets for [`unzip_filtered`].
#[derive(Debug, Clone, Copy)]
pub struct ZipLimits {
    /// Largest declared or stored size of one entry.
    pub max_entry_bytes: usize,
    /// Largest entry count read from the archive.
    pub max_entries: usize,
}

const FLEB: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 0, 0, 0,
];
const FDEB: [u8; 32] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 0, 0,
];
const CLIM: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

struct Tables {
    fl: [u32; 31],
    fd: [u32; 31],
    rev: Vec<u16>,
    flrm: Vec<u16>,
    fdrm: Vec<u16>,
}

/// `freb`: base values from extra-bit counts; index -1 reads as shift 0.
fn freb(eb: &[u8; 32], start: u32) -> [u32; 31] {
    let mut b = [0u32; 31];
    let mut s = start;
    for (i, slot) in b.iter_mut().enumerate() {
        let bits = if i == 0 { 0 } else { u32::from(eb[i - 1]) };
        s += 1 << bits;
        *slot = s & 0xffff;
    }
    b
}

/// `hMap` with `r = true` (the only form inflate uses): index -> `(symbol << 4) | bit length`.
/// Written with the original's unchecked arithmetic, so bad trees collide instead of failing.
fn hmap(cd: &[u8], mb: usize, rev: &[u16]) -> Vec<u16> {
    let mut l = vec![0u16; mb];
    for &c in cd {
        if c != 0 && usize::from(c) - 1 < mb {
            l[usize::from(c) - 1] = l[usize::from(c) - 1].wrapping_add(1);
        }
    }
    let mut le = vec![0u16; mb];
    for i in 1..mb {
        le[i] = (le[i - 1].wrapping_add(l[i - 1])).wrapping_shl(1);
    }
    let mut co = vec![0u16; 1usize << mb];
    let rvb = 15 - mb;
    for (i, &c) in cd.iter().enumerate() {
        if c == 0 {
            continue;
        }
        let c = usize::from(c);
        let sv = ((i << 4) | c) as u16;
        let r1 = mb - c;
        let old = le[c - 1];
        le[c - 1] = old.wrapping_add(1);
        let mut v = (usize::from(old)) << r1;
        let m = v | ((1usize << r1) - 1);
        while v <= m {
            let ix = if v < 32768 {
                usize::from(rev[v]) >> rvb
            } else {
                0
            };
            co[ix] = sv;
            v += 1;
        }
    }
    co
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut rev = vec![0u16; 32768];
        for (i, slot) in rev.iter_mut().enumerate() {
            let i = i as u32;
            let mut x = ((i & 0xAAAA) >> 1) | ((i & 0x5555) << 1);
            x = ((x & 0xCCCC) >> 2) | ((x & 0x3333) << 2);
            x = ((x & 0xF0F0) >> 4) | ((x & 0x0F0F) << 4);
            *slot = ((((x & 0xFF00) >> 8) | ((x & 0x00FF) << 8)) >> 1) as u16;
        }
        let mut fl = freb(&FLEB, 2);
        fl[28] = 258;
        let fd = freb(&FDEB, 0);
        let mut flt = [0u8; 288];
        for (i, v) in flt.iter_mut().enumerate() {
            *v = match i {
                0..=143 => 8,
                144..=255 => 9,
                256..=279 => 7,
                _ => 8,
            };
        }
        let fdt = [5u8; 32];
        let flrm = hmap(&flt, 9, &rev);
        let fdrm = hmap(&fdt, 5, &rev);
        Tables {
            fl,
            fd,
            rev,
            flrm,
            fdrm,
        }
    })
}

fn byte(d: &[u8], i: usize) -> u32 {
    d.get(i).copied().map_or(0, u32::from)
}

fn bits(d: &[u8], p: usize, m: u32) -> u32 {
    let o = p / 8;
    ((byte(d, o) | (byte(d, o + 1) << 8)) >> (p & 7)) & m
}

fn bits16(d: &[u8], p: usize) -> u32 {
    let o = p / 8;
    (byte(d, o) | (byte(d, o + 1) << 8) | (byte(d, o + 2) << 16)) >> (p & 7)
}

fn max_of(a: &[u8]) -> usize {
    usize::from(a.iter().copied().max().unwrap_or(0))
}

/// Raw DEFLATE into an output of the declared size `su` (the caller has checked `su` against its
/// budget). Returns the first `min(written, su)` bytes, as `inflateSync(data, { out: new Uint8Array(su) })`.
pub fn inflate_into(dat: &[u8], su: usize) -> Result<Vec<u8>, InflateError> {
    let mut buf = vec![0u8; su];
    if dat.is_empty() {
        return Ok(buf);
    }
    let t = tables();
    let sl = dat.len();
    let tbts = sl * 8;
    let mut pos = 0usize;
    let mut bt = 0usize;
    loop {
        let fin = bits(dat, pos, 1);
        let kind = bits(dat, pos + 1, 3);
        pos += 3;
        let (lm, dm, lbt, dbt);
        match kind {
            0 => {
                let s = pos.div_ceil(8) + 4;
                let l = byte(dat, s - 4) as usize | ((byte(dat, s - 3) as usize) << 8);
                let end = s + l;
                if end > sl {
                    return Err(fflate(0));
                }
                if bt + l > su {
                    return Err(InflateError::OffsetOutOfBounds);
                }
                buf[bt..bt + l].copy_from_slice(&dat[s..end]);
                bt += l;
                pos = end * 8;
                if fin == 1 {
                    break;
                }
                continue;
            }
            1 => {
                lm = t.flrm.clone();
                dm = t.fdrm.clone();
                lbt = 9;
                dbt = 5;
            }
            2 => {
                let h_lit = bits(dat, pos, 31) as usize + 257;
                let hc_len = bits(dat, pos + 10, 15) as usize + 4;
                let tl = h_lit + bits(dat, pos + 5, 31) as usize + 1;
                pos += 14;
                let mut ldt = vec![0u8; tl];
                let mut clt = [0u8; 19];
                for (i, &ix) in CLIM.iter().enumerate().take(hc_len) {
                    clt[ix] = bits(dat, pos + i * 3, 7) as u8;
                }
                pos += hc_len * 3;
                let clb = max_of(&clt);
                let clbmsk = (1u32 << clb) - 1;
                let clm = hmap(&clt, clb, &t.rev);
                let mut i = 0usize;
                while i < tl {
                    let r = clm[bits(dat, pos, clbmsk) as usize];
                    pos += usize::from(r & 15);
                    let s = r >> 4;
                    if s < 16 {
                        ldt[i] = s as u8;
                        i += 1;
                    } else {
                        let (mut n, c) = match s {
                            16 => {
                                let n = 3 + bits(dat, pos, 3) as usize;
                                pos += 2;
                                (n, if i > 0 { ldt[i - 1] } else { 0 })
                            }
                            17 => {
                                let n = 3 + bits(dat, pos, 7) as usize;
                                pos += 3;
                                (n, 0)
                            }
                            _ => {
                                let n = 11 + bits(dat, pos, 127) as usize;
                                pos += 7;
                                (n, 0)
                            }
                        };
                        while n > 0 {
                            if i < tl {
                                ldt[i] = c;
                            }
                            i += 1;
                            n -= 1;
                        }
                    }
                }
                let (lt, dt) = ldt.split_at(h_lit);
                lbt = max_of(lt);
                dbt = max_of(dt);
                lm = hmap(lt, lbt, &t.rev);
                dm = hmap(dt, dbt, &t.rev);
            }
            _ => return Err(fflate(1)),
        }
        if pos > tbts {
            return Err(fflate(0));
        }
        let lms = (1u32 << lbt) - 1;
        let dms = (1u32 << dbt) - 1;
        loop {
            let c = lm[(bits16(dat, pos) & lms) as usize];
            let sym = usize::from(c >> 4);
            pos += usize::from(c & 15);
            if pos > tbts {
                return Err(fflate(0));
            }
            if c == 0 {
                return Err(fflate(2));
            }
            if sym < 256 {
                if bt < su {
                    buf[bt] = sym as u8;
                }
                bt += 1;
            } else if sym == 256 {
                break;
            } else {
                let add = if sym > 264 {
                    let i = sym - 257;
                    let b = u32::from(FLEB[i]);
                    let a = bits(dat, pos, (1 << b) - 1) as usize + t.fl[i] as usize;
                    pos += b as usize;
                    a
                } else {
                    sym - 254
                };
                let d = dm[(bits16(dat, pos) & dms) as usize];
                let dsym = usize::from(d >> 4);
                if d == 0 {
                    return Err(fflate(3));
                }
                pos += usize::from(d & 15);
                // fd has 31 entries; symbol 31 reads undefined, so the copy below writes zeros.
                let mut dist = t.fd.get(dsym).copied().map(|v| v as usize);
                if dsym > 3 {
                    let b = u32::from(FDEB[dsym]);
                    let extra = (bits16(dat, pos) & ((1 << b) - 1)) as usize;
                    dist = dist.map(|v| v + extra);
                    pos += b as usize;
                }
                if pos > tbts {
                    return Err(fflate(0));
                }
                let end = bt + add;
                match dist {
                    Some(dt) => {
                        if bt < dt {
                            return Err(fflate(3));
                        }
                        while bt < end {
                            if bt < su {
                                let src = bt - dt;
                                buf[bt] = if src < su { buf[src] } else { 0 };
                            }
                            bt += 1;
                        }
                    }
                    None => {
                        while bt < end {
                            if bt < su {
                                buf[bt] = 0;
                            }
                            bt += 1;
                        }
                    }
                }
            }
        }
        if fin == 1 {
            break;
        }
    }
    buf.truncate(bt.min(su));
    Ok(buf)
}

fn rd(d: &[u8], i: i128) -> u32 {
    usize::try_from(i)
        .ok()
        .and_then(|i| d.get(i))
        .copied()
        .map_or(0, u32::from)
}
fn b2(d: &[u8], i: i128) -> u32 {
    rd(d, i) | (rd(d, i + 1) << 8)
}
fn b4(d: &[u8], i: i128) -> u32 {
    rd(d, i) | (rd(d, i + 1) << 8) | (rd(d, i + 2) << 16) | (rd(d, i + 3) << 24)
}
fn b8(d: &[u8], i: i128) -> i128 {
    i128::from(b4(d, i)) + i128::from(b4(d, i + 4)) * 4_294_967_296
}

/// `fflate.strFromU8`: Latin-1 when the UTF-8 flag is clear, else UTF-8 with replacement and no BOM.
fn name_of(bytes: &[u8], latin1: bool) -> String {
    if latin1 {
        bytes.iter().map(|&b| char::from(b)).collect()
    } else {
        let b = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
        String::from_utf8_lossy(b).into_owned()
    }
}

fn subarray(d: &[u8], start: i128, end: i128) -> &[u8] {
    let len = d.len() as i128;
    let s = start.clamp(0, len);
    let e = end.clamp(0, len);
    if s >= e {
        &[]
    } else {
        &d[s as usize..e as usize]
    }
}

/// `fflate.unzipSync(data, { filter: name => keep(name) })`: entries that pass `keep`, in the
/// order a JS object lists them (a repeated name keeps its first position and its last value).
pub fn unzip_filtered(
    data: &[u8],
    keep: impl Fn(&str) -> bool,
    limits: ZipLimits,
) -> Result<Vec<(String, Vec<u8>)>, InflateError> {
    unzip_filtered_bounded(data, keep, limits, usize::MAX)
}

/// Filtered ZIP read with a cumulative output-allocation budget.
///
/// Charges each kept entry before copying or inflating: actual stored bytes or declared DEFLATE
/// output capacity, including duplicate names. Truncating an inflated vector does not release its
/// capacity. Charging duplicates also bounds cumulative decompression work rather than allowing a
/// last-value-wins archive to reuse the budget. Filtered entries spend nothing. This is an explicit
/// host limit, absent from the source; [`InflateError::AggregateLimit`] distinguishes it from the
/// per-entry and entry-count limits. No entry is allocated before both checks pass.
pub fn unzip_filtered_bounded(
    data: &[u8],
    keep: impl Fn(&str) -> bool,
    limits: ZipLimits,
    max_total_bytes: usize,
) -> Result<Vec<(String, Vec<u8>)>, InflateError> {
    let mut remaining = max_total_bytes;
    let len = data.len() as i128;
    let mut e = len - 22;
    while b4(data, e) != 0x06054B50 {
        if e == 0 || len - e > 65558 {
            return Err(fflate(13));
        }
        e -= 1;
    }
    let mut c = i128::from(b2(data, e + 8));
    if c == 0 {
        return Ok(Vec::new());
    }
    let mut o = i128::from(b4(data, e + 16));
    let mut z = b4(data, e - 20) == 0x07064B50;
    if z {
        let ze = i128::from(b4(data, e - 12));
        z = b4(data, ze) == 0x06064B50;
        if z {
            c = i128::from(b4(data, ze + 32));
            o = i128::from(b4(data, ze + 48));
        }
    }
    if c > limits.max_entries as i128 {
        return Err(InflateError::ResourceLimit);
    }
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for _ in 0..c {
        let fnl = i128::from(b2(data, o + 28));
        let efl = i128::from(b2(data, o + 30));
        let latin1 = b2(data, o + 8) & 2048 == 0;
        let name = name_of(subarray(data, o + 46, o + 46 + fnl), latin1);
        let es = o + 46 + fnl;
        let (mut sc, mut su, mut off) = (
            i128::from(b4(data, o + 20)),
            i128::from(b4(data, o + 24)),
            i128::from(b4(data, o + 42)),
        );
        let (nsc, nsu, noff) = (sc == 0xFFFF_FFFF, su == 0xFFFF_FFFF, off == 0xFFFF_FFFF);
        if z && (nsc || nsu || noff) {
            let end = es + efl;
            let mut p = es;
            let mut found = false;
            while p + 4 < end {
                if b2(data, p) == 1 {
                    let (a, b, cc) = (
                        if nsc {
                            b8(data, p + 4 + 8 * i128::from(nsu))
                        } else {
                            sc
                        },
                        if nsu { b8(data, p + 4) } else { su },
                        if noff {
                            b8(data, p + 4 + 8 * (i128::from(nsu) + i128::from(nsc)))
                        } else {
                            off
                        },
                    );
                    sc = a;
                    su = b;
                    off = cc;
                    found = true;
                    break;
                }
                p += 4 + i128::from(b2(data, p + 2));
            }
            if !found {
                return Err(fflate(13));
            }
        }
        let method = b2(data, o + 10);
        o = es + efl + i128::from(b2(data, o + 32));
        let body = off + 30 + i128::from(b2(data, off + 26)) + i128::from(b2(data, off + 28));
        if !keep(&name) {
            continue;
        }
        let input = subarray(data, body, body + sc);
        let allocation = match method {
            0 => input.len() as i128,
            8 => su,
            _ => 0, // Preserve the source's unknown-method error below.
        };
        if allocation > limits.max_entry_bytes as i128 {
            return Err(InflateError::ResourceLimit);
        }
        if allocation > remaining as i128 {
            return Err(InflateError::AggregateLimit);
        }
        remaining -= allocation as usize;
        let bytes = match method {
            0 => input.to_vec(),
            8 => inflate_into(input, su as usize)?,
            m => {
                return Err(InflateError::Fflate {
                    code: 14,
                    message: format!("unknown compression type {m}"),
                });
            }
        };
        match files.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = bytes,
            None => files.push((name, bytes)),
        }
    }
    Ok(files)
}

/// The 3MF reader's filter: `/\.(model|rels|config)$/i`.
#[must_use]
pub fn is_threemf_part(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [".model", ".rels", ".config"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}
