// SPDX-License-Identifier: AGPL-3.0-only
//! PNG decoding as the original's `decodeImage` (`src/app/scanFiles.ts`) observes it in Chrome:
//! `createImageBitmap(blob, { colorSpaceConversion: 'none', premultiplyAlpha: 'none' })`, then
//! `drawImage` onto a 2D canvas and `getImageData`. The original catches every failure and leaves
//! the image out, so a source refusal has no message: [`PngError::Omitted`].
//!
//! Behaviour was derived from actual Chrome 156 outputs (fixtures in
//! `docs/port-plan/fixtures/image-audit`, probes and fuzz in `tests/png.rs`), not from a codec:
//! - critical chunks need a valid CRC, ancillary chunks with a bad CRC are skipped;
//! - the zlib header is checked, its Adler-32 is not; data after the image's bytes is ignored;
//! - IDAT chunks must be consecutive; the image is complete once the 8-byte header of the next
//!   chunk after them is present, whatever its type, length or CRC (nothing after it is read);
//! - 16-bit samples keep their high byte; lower depths scale by bit replication; colour
//!   management (gAMA, sRGB, iCCP) is not applied;
//! - palette indices past `PLTE` take its last entry and that entry's alpha (black if empty);
//! - APNG: with a valid `acTL` and no `fcTL` before IDAT, the image is frame 0 (the first
//!   `fcTL` and its `fdAT` run, placed on a transparent canvas) and IDAT is never decoded;
//!   `fcTL` length, CRC, sequence and region are checked, `fdAT` CRCs are not;
//! - the canvas stores premultiplied 8-bit colour and `getImageData` un-premultiplies in float32
//!   ([`canvas_round_trip`]), which changes colour wherever alpha is neither 0 nor 255.
//!
//! Host limits ([`PngLimits`]) are reported as [`PngError::Limit`], never as an omission.

/// Caller budgets. These are host safeguards, distinct from the original's own failures.
#[derive(Debug, Clone, Copy)]
pub struct PngLimits {
    /// Maximum input bytes.
    pub max_input: usize,
    /// Maximum `width * height` pixels.
    pub max_pixels: u64,
}

impl Default for PngLimits {
    fn default() -> Self {
        Self {
            max_input: 256 << 20,
            max_pixels: 1 << 26,
        }
    }
}

/// Why no image was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngError {
    /// The browser could not decode it; the original leaves the image out.
    Omitted,
    /// A caller budget was exceeded (host boundary, not a source result).
    Limit,
    /// A buffer reservation failed.
    Allocation,
}

/// Decoded straight (non-premultiplied) RGBA, rows top to bottom, after the canvas round trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Four bytes per pixel.
    pub rgba: Vec<u8>,
}

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];

/// True when the bytes start with the PNG signature.
pub fn is_png(bytes: &[u8]) -> bool {
    bytes.starts_with(&SIGNATURE)
}

/// What one canvas `drawImage` then `getImageData` does to straight RGBA: premultiply to 8 bits
/// with integer rounding, then un-premultiply in float32 and round half to even. Exact for all
/// 65,536 colour/alpha pairs on Chrome 156 (the `png-alpha-atlas` fixture).
pub fn canvas_round_trip(rgba: &mut [u8]) {
    let table = round_trip_table();
    for px in rgba.as_chunks_mut::<4>().0 {
        let a = usize::from(px[3]);
        for c in &mut px[..3] {
            *c = table[a * 256 + usize::from(*c)];
        }
    }
}

fn round_trip_table() -> &'static [u8; 65536] {
    static TABLE: std::sync::OnceLock<Box<[u8; 65536]>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = Box::new([0u8; 65536]);
        let inv255 = 1.0f32 / 255.0;
        for a in 0..256u32 {
            for c in 0..256u32 {
                let out = match a {
                    0 => 0,
                    255 => c,
                    _ => {
                        let p = (c * a + 127) / 255;
                        let r = p as f32 * inv255;
                        let s = 1.0 / (a as f32 * inv255);
                        let v = (r * s * 255.0).round_ties_even();
                        if v >= 255.0 { 255 } else { v as u32 }
                    }
                };
                t[(a * 256 + c) as usize] = out as u8;
            }
        }
        t
    })
}

/// Decode a PNG as the original sees it in Chrome.
pub fn decode_png(bytes: &[u8], limits: PngLimits) -> Result<DecodedImage, PngError> {
    if bytes.len() > limits.max_input {
        return Err(PngError::Limit);
    }
    let parsed = parse_chunks(bytes)?;
    let h = parsed.header;
    let pixels = u64::from(h.width) * u64::from(h.height);
    if pixels > limits.max_pixels {
        return Err(PngError::Limit);
    }
    let size = usize::try_from(pixels * 4).map_err(|_| PngError::Limit)?;
    // Frame 0's own header: the IDAT image, or an APNG frame placed on the full canvas.
    let fh = parsed.frame.map_or(h, |f| Header {
        width: f.width,
        height: f.height,
        ..h
    });
    // Inflate before allocating pixels, so a header that claims more than the data holds costs
    // only what the data decodes to.
    let raw = inflate_zlib(
        &parsed.idat,
        &parsed.ends,
        raw_size(&fh).ok_or(PngError::Limit)?,
    )?;
    let mut rgba = zeroed(size)?;
    match parsed.frame {
        None => unfilter_and_convert(&h, &parsed, &raw, &mut rgba)?,
        Some(f) => {
            let mut sub = zeroed(f.width as usize * f.height as usize * 4)?;
            unfilter_and_convert(&fh, &parsed, &raw, &mut sub)?;
            let (fw, iw) = (f.width as usize * 4, h.width as usize * 4);
            for (r, row) in sub.chunks_exact(fw).enumerate() {
                let o = (f.y as usize + r) * iw + f.x as usize * 4;
                rgba[o..o + fw].copy_from_slice(row);
            }
        }
    }
    canvas_round_trip(&mut rgba);
    Ok(DecodedImage {
        width: h.width,
        height: h.height,
        rgba,
    })
}

fn zeroed(size: usize) -> Result<Vec<u8>, PngError> {
    let mut v = Vec::new();
    v.try_reserve_exact(size)
        .map_err(|_| PngError::Allocation)?;
    v.resize(size, 0);
    Ok(v)
}

#[derive(Debug, Clone, Copy)]
struct Header {
    width: u32,
    height: u32,
    depth: u8,
    color: u8,
    interlace: bool,
}

impl Header {
    fn channels(&self) -> usize {
        match self.color {
            0 | 3 => 1,
            2 => 3,
            4 => 2,
            _ => 4,
        }
    }
    fn bits_per_pixel(&self) -> usize {
        self.channels() * usize::from(self.depth)
    }
}

struct Parsed {
    header: Header,
    palette: Vec<[u8; 3]>,
    trns: Option<Vec<u8>>,
    /// Image data of the decoded frame: IDAT, or frame 0's fdAT payloads.
    idat: Vec<u8>,
    /// Cumulative end offsets of each payload within `idat`.
    ends: Vec<usize>,
    /// Frame 0's region when it is not the IDAT image.
    frame: Option<Frame>,
}

fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn crc32(data: &[&[u8]]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let t = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (n, e) in t.iter_mut().enumerate() {
            let mut c = n as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            *e = c;
        }
        t
    });
    let mut c = 0xffff_ffffu32;
    for part in data {
        for &b in *part {
            c = t[((c ^ u32::from(b)) & 0xff) as usize] ^ (c >> 8);
        }
    }
    c ^ 0xffff_ffff
}

/// One chunk read in full: its type, data, and whether its CRC matched.
struct Chunk<'a> {
    data: &'a [u8],
    crc_ok: bool,
}

/// The type in the complete 8-byte header at `at`, before its length is judged.
fn chunk_kind(bytes: &[u8], at: usize) -> Result<[u8; 4], PngError> {
    let head = bytes.get(at..at + 8).ok_or(PngError::Omitted)?;
    Ok([head[4], head[5], head[6], head[7]])
}

/// The 8-byte header at `at`, if complete and its length is at most 2^31 - 1: (length, type).
fn chunk_head(bytes: &[u8], at: usize) -> Result<(usize, [u8; 4]), PngError> {
    let kind = chunk_kind(bytes, at)?;
    let len = be32(&bytes[at..at + 4]);
    if len > 0x7fff_ffff {
        return Err(PngError::Omitted);
    }
    Ok((len as usize, kind))
}

fn chunk_body(bytes: &[u8], at: usize, len: usize, kind: [u8; 4]) -> Result<Chunk<'_>, PngError> {
    let data = bytes.get(at + 8..at + 8 + len).ok_or(PngError::Omitted)?;
    let crc = bytes
        .get(at + 8 + len..at + 12 + len)
        .ok_or(PngError::Omitted)?;
    Ok(Chunk {
        data,
        crc_ok: crc32(&[&kind, data]) == be32(crc),
    })
}

/// An APNG frame control: region and its validity against the image.
#[derive(Debug, Clone, Copy)]
struct Frame {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

/// `fcTL`: Chrome fails the image on a bad length, CRC, sequence number, region, dispose or blend.
fn read_fctl(c: &Chunk<'_>, image: &Header, seq: &mut u32) -> Result<Frame, PngError> {
    use PngError::Omitted;
    if c.data.len() != 26 || !c.crc_ok || be32(&c.data[0..4]) != *seq {
        return Err(Omitted);
    }
    *seq = seq.wrapping_add(1);
    let f = Frame {
        width: be32(&c.data[4..8]),
        height: be32(&c.data[8..12]),
        x: be32(&c.data[12..16]),
        y: be32(&c.data[16..20]),
    };
    let (dispose, blend) = (c.data[24], c.data[25]);
    let fits = f.width > 0
        && f.height > 0
        && u64::from(f.x) + u64::from(f.width) <= u64::from(image.width)
        && u64::from(f.y) + u64::from(f.height) <= u64::from(image.height);
    if !fits || dispose > 2 || blend > 1 {
        return Err(Omitted);
    }
    Ok(f)
}

fn parse_chunks(bytes: &[u8]) -> Result<Parsed, PngError> {
    use PngError::Omitted;
    if !is_png(bytes) {
        return Err(Omitted);
    }
    let mut at = 8usize;
    let mut header: Option<Header> = None;
    let mut palette: Option<Vec<[u8; 3]>> = None;
    let mut trns: Option<Vec<u8>> = None;
    let mut idat = Vec::new();
    let mut ends = Vec::new();
    let mut animated = false;
    let mut idat_is_frame = false;
    let mut seq = 0u32;
    // Once IDAT has started, the first complete header of any other chunk ends the image:
    // Chrome reads nothing after it (its type, length and CRC included).
    loop {
        let (len, kind) = chunk_head(bytes, at)?;
        if header.is_none() && &kind != b"IHDR" {
            return Err(Omitted);
        }
        if &kind == b"IEND" {
            return Err(Omitted);
        }
        let c = chunk_body(bytes, at, len, kind)?;
        let critical = kind[0] & 0x20 == 0;
        if !c.crc_ok && critical {
            return Err(Omitted);
        }
        if &kind == b"IDAT" {
            if header.is_some_and(|h| h.color == 3) && palette.is_none() {
                return Err(Omitted);
            }
            idat.extend_from_slice(c.data);
            ends.push(idat.len());
            at += 12 + len;
            while &chunk_kind(bytes, at)? == b"IDAT" {
                let (len, kind) = chunk_head(bytes, at)?;
                let c = chunk_body(bytes, at, len, kind)?;
                if !c.crc_ok {
                    return Err(Omitted);
                }
                idat.extend_from_slice(c.data);
                ends.push(idat.len());
                at += 12 + len;
            }
            break;
        }
        at += 12 + len;
        let image = header;
        match &kind {
            b"IHDR" => {
                if header.is_some() || len != 13 {
                    return Err(Omitted);
                }
                header = Some(read_header(c.data)?);
            }
            b"PLTE" => {
                if palette.is_some() || !(3..=768).contains(&len) {
                    return Err(Omitted);
                }
                palette = Some(c.data.as_chunks::<3>().0.to_vec());
            }
            b"tRNS" if c.crc_ok => {
                let color = header.map_or(0, |h| h.color);
                if trns.is_none() && (color != 3 || palette.is_some()) {
                    trns = Some(c.data.to_vec());
                }
            }
            b"acTL" if c.crc_ok => {
                if len == 8
                    && (1..=0x7fff_ffff).contains(&be32(&c.data[0..4]))
                    && be32(&c.data[4..8]) <= 0x7fff_ffff
                {
                    animated = true;
                }
            }
            b"fcTL" => {
                let image = image.ok_or(Omitted)?;
                let f = read_fctl(&c, &image, &mut seq)?;
                if f.width != image.width || f.height != image.height || f.x != 0 || f.y != 0 {
                    return Err(Omitted);
                }
                idat_is_frame = true;
            }
            _ if critical => return Err(Omitted),
            _ => {}
        }
    }
    let header = header.ok_or(Omitted)?;
    let palette = palette.unwrap_or_default();
    if !animated || idat_is_frame {
        return Ok(Parsed {
            header,
            palette,
            trns,
            idat,
            ends,
            frame: None,
        });
    }
    // Animated, with IDAT not part of the animation: frame 0 is the first fcTL and its fdAT run.
    let mut frame: Option<Frame> = None;
    let mut data = Vec::new();
    let mut fdat_ends = Vec::new();
    let mut in_fdat = false;
    loop {
        if in_fdat && &chunk_kind(bytes, at)? != b"fdAT" {
            break;
        }
        let (len, kind) = chunk_head(bytes, at)?;
        if &kind == b"IEND" {
            return Err(Omitted);
        }
        let c = chunk_body(bytes, at, len, kind)?;
        at += 12 + len;
        match &kind {
            b"fcTL" => frame = Some(read_fctl(&c, &header, &mut seq)?),
            b"fdAT" => {
                if frame.is_none() || len < 4 || be32(&c.data[0..4]) != seq {
                    return Err(Omitted);
                }
                seq = seq.wrapping_add(1);
                data.extend_from_slice(&c.data[4..]);
                fdat_ends.push(data.len());
                in_fdat = true;
            }
            _ if kind[0] & 0x20 == 0 && (&kind != b"PLTE" || !c.crc_ok) => return Err(Omitted),
            _ => {}
        }
    }
    Ok(Parsed {
        header,
        palette,
        trns,
        idat: data,
        ends: fdat_ends,
        frame,
    })
}

fn read_header(d: &[u8]) -> Result<Header, PngError> {
    let width = be32(&d[0..4]);
    let height = be32(&d[4..8]);
    let (depth, color, comp, filter, interlace) = (d[8], d[9], d[10], d[11], d[12]);
    let ok_depth = match color {
        0 => matches!(depth, 1 | 2 | 4 | 8 | 16),
        3 => matches!(depth, 1 | 2 | 4 | 8),
        2 | 4 | 6 => matches!(depth, 8 | 16),
        _ => false,
    };
    if width == 0
        || height == 0
        || width > 0x7fff_ffff
        || height > 0x7fff_ffff
        || !ok_depth
        || comp != 0
        || filter != 0
        || interlace > 1
    {
        return Err(PngError::Omitted);
    }
    Ok(Header {
        width,
        height,
        depth,
        color,
        interlace: interlace == 1,
    })
}

const PASSES: [(usize, usize, usize, usize); 7] = [
    (0, 0, 8, 8),
    (4, 0, 8, 8),
    (0, 4, 4, 8),
    (2, 0, 4, 4),
    (0, 2, 2, 4),
    (1, 0, 2, 2),
    (0, 1, 1, 2),
];

/// Sub-images as (x0, y0, dx, dy, columns, rows); one pass for a non-interlaced image.
fn passes(h: &Header) -> Vec<(usize, usize, usize, usize, usize, usize)> {
    let (w, ht) = (h.width as usize, h.height as usize);
    if !h.interlace {
        return vec![(0, 0, 1, 1, w, ht)];
    }
    PASSES
        .iter()
        .map(|&(x0, y0, dx, dy)| {
            (
                x0,
                y0,
                dx,
                dy,
                w.saturating_sub(x0).div_ceil(dx),
                ht.saturating_sub(y0).div_ceil(dy),
            )
        })
        .filter(|p| p.4 > 0 && p.5 > 0)
        .collect()
}

fn raw_size(h: &Header) -> Option<usize> {
    let bpp = h.bits_per_pixel();
    let mut total = 0usize;
    for p in passes(h) {
        let stride = p.4.checked_mul(bpp)?.div_ceil(8);
        total = total.checked_add(stride.checked_add(1)?.checked_mul(p.5)?)?;
    }
    Some(total)
}

fn unfilter_and_convert(
    h: &Header,
    parsed: &Parsed,
    raw: &[u8],
    rgba: &mut [u8],
) -> Result<(), PngError> {
    let bpp_bits = h.bits_per_pixel();
    let bpp = bpp_bits.div_ceil(8);
    let width = h.width as usize;
    let mut at = 0usize;
    for (x0, y0, dx, dy, cols, rows) in passes(h) {
        let stride = (cols * bpp_bits).div_ceil(8);
        let mut prev = vec![0u8; stride];
        let mut line = vec![0u8; stride];
        for r in 0..rows {
            let filter = raw[at];
            line.copy_from_slice(&raw[at + 1..at + 1 + stride]);
            at += 1 + stride;
            for i in 0..stride {
                let a = if i >= bpp { line[i - bpp] } else { 0 };
                let b = prev[i];
                let c = if i >= bpp { prev[i - bpp] } else { 0 };
                let add = match filter {
                    0 => 0,
                    1 => a,
                    2 => b,
                    3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                    4 => paeth(a, b, c),
                    _ => return Err(PngError::Omitted),
                };
                line[i] = line[i].wrapping_add(add);
            }
            let y = y0 + r * dy;
            for col in 0..cols {
                let x = x0 + col * dx;
                let px = pixel(h, parsed, &line, col);
                let o = (y * width + x) * 4;
                rgba[o..o + 4].copy_from_slice(&px);
            }
            std::mem::swap(&mut prev, &mut line);
        }
    }
    Ok(())
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (ia, ib, ic) = (i16::from(a), i16::from(b), i16::from(c));
    let p = ia + ib - ic;
    let (pa, pb, pc) = ((p - ia).abs(), (p - ib).abs(), (p - ic).abs());
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// The raw sample `index` (0-based within the line) at the header's bit depth.
fn sample(line: &[u8], depth: u8, index: usize) -> u16 {
    match depth {
        16 => u16::from_be_bytes([line[index * 2], line[index * 2 + 1]]),
        8 => u16::from(line[index]),
        d => {
            let bit = index * usize::from(d);
            let byte = line[bit / 8];
            let shift = 8 - usize::from(d) - bit % 8;
            u16::from((byte >> shift) & ((1u8 << d) - 1))
        }
    }
}

fn to8(v: u16, depth: u8) -> u8 {
    match depth {
        16 => (v >> 8) as u8,
        8 => v as u8,
        4 => (v * 0x11) as u8,
        2 => (v * 0x55) as u8,
        _ => (v * 0xff) as u8,
    }
}

fn pixel(h: &Header, parsed: &Parsed, line: &[u8], col: usize) -> [u8; 4] {
    let d = h.depth;
    let trns = parsed.trns.as_deref();
    // tRNS colour keys: at 16 bits the chunk must be exactly the key and compares in full; at 8
    // bits or fewer a longer chunk is accepted and each sample compares with the key's low byte.
    let key_ok = |t: &[u8], n: usize| if d == 16 { t.len() == n } else { t.len() >= n };
    let key = |t: &[u8], i: usize| {
        if d == 16 {
            u16::from_be_bytes([t[i], t[i + 1]])
        } else {
            u16::from(t[i + 1])
        }
    };
    match h.color {
        0 => {
            let v = sample(line, d, col);
            let g = to8(v, d);
            let a = match trns {
                Some(t) if key_ok(t, 2) && key(t, 0) == v => 0,
                _ => 255,
            };
            [g, g, g, a]
        }
        2 => {
            let (r, g, b) = (
                sample(line, d, col * 3),
                sample(line, d, col * 3 + 1),
                sample(line, d, col * 3 + 2),
            );
            let a = match trns {
                Some(t) if key_ok(t, 6) && key(t, 0) == r && key(t, 2) == g && key(t, 4) == b => 0,
                _ => 255,
            };
            [to8(r, d), to8(g, d), to8(b, d), a]
        }
        3 => {
            let i = usize::from(sample(line, d, col));
            // Indices past PLTE take its last entry (with that entry's alpha), or opaque black.
            let n = parsed.palette.len();
            let i = if i >= n { n.wrapping_sub(1) } else { i };
            match parsed.palette.get(i) {
                Some(&[r, g, b]) => [r, g, b, trns.and_then(|t| t.get(i).copied()).unwrap_or(255)],
                None => [0, 0, 0, 255],
            }
        }
        4 => {
            let g = to8(sample(line, d, col * 2), d);
            [g, g, g, to8(sample(line, d, col * 2 + 1), d)]
        }
        _ => [
            to8(sample(line, d, col * 4), d),
            to8(sample(line, d, col * 4 + 1), d),
            to8(sample(line, d, col * 4 + 2), d),
            to8(sample(line, d, col * 4 + 3), d),
        ],
    }
}

// ---- strict zlib (RFC 1950/1951 with zlib's own validity rules) ----

// ---- zlib as Chrome's PNG decoder runs it ----
//
// Chrome 156 decodes PNG with the Rust `png` crate over `fdeflate`. Their observable rules,
// reproduced here (behaviour, not code): before the image's bytes are complete every stream error
// and running out of data fail the image. Once they are complete the decoder writes nothing more
// and only follows structure, with the bits left in the IDAT chunk where the image finished (later
// chunks are never fed): a block end is taken only when at least 15 bits remain, a block header
// needs 10, a stored header its full length, a dynamic header 17 bits, then 3 per code-length
// code and at least 7 per code-length symbol. Any of these that does not fit stops quietly; an
// invalid one fails the image. Huffman sets must be complete, except that a distance set may be
// empty or a single one-bit code. The Adler-32 is never checked.

/// Why inflation stopped early: the data ran out, or the stream is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Z {
    Eof,
    Bad,
    /// A buffer reservation failed (host boundary).
    Alloc,
}

/// Append one byte, growing the buffer fallibly in step with what the data actually decodes to.
fn put(out: &mut Vec<u8>, b: u8, need: usize) -> Result<(), Z> {
    if out.len() == out.capacity() {
        let more = out.len().max(1 << 16).min(need - out.len());
        out.try_reserve_exact(more).map_err(|_| Z::Alloc)?;
    }
    out.push(b);
    Ok(())
}

/// LSB-first bit reader over `data`, never reading at or past bit `limit`.
struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    limit: usize,
}

impl Bits<'_> {
    fn avail(&self) -> usize {
        self.limit.saturating_sub(self.pos)
    }
    /// The next `n` (at most 25) bits; bits at or past the limit read as zero.
    fn peek(&self, n: u32) -> u32 {
        let byte = self.pos / 8;
        let mut word = 0u64;
        for (i, &b) in self.data.iter().skip(byte).take(5).enumerate() {
            word |= u64::from(b) << (8 * i);
        }
        let avail = self.avail().min(n as usize) as u32;
        ((word >> (self.pos % 8)) & ((1u64 << avail) - 1)) as u32
    }
    fn need(&mut self, n: u32) -> Result<u32, Z> {
        if self.avail() < n as usize {
            return Err(Z::Eof);
        }
        let v = self.peek(n);
        self.pos += n as usize;
        Ok(v)
    }
    fn align(&mut self) {
        self.pos = self.pos.div_ceil(8) * 8;
    }
}

struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
    /// Codes of at most `FAST` bits by their next `FAST` stream bits: `length << 12 | symbol`.
    fast: Vec<u16>,
}

const FAST: u32 = 9;

/// Build a canonical code; `None` for a set the decoder rejects: anything but a complete set,
/// except that a distance set may be empty or hold a single one-bit code.
fn huffman(lengths: &[u8], distance: bool) -> Option<Huffman> {
    let mut counts = [0u16; 16];
    for &l in lengths {
        counts[usize::from(l)] += 1;
    }
    let max = (1..16).rev().find(|&l| counts[l] != 0).unwrap_or(0);
    let single = distance && (max == 0 || (max == 1 && counts[1] == 1));
    if !single {
        let mut used: u32 = 0;
        for &c in &counts[1..=max.max(1)] {
            used = (used << 1) + u32::from(c);
        }
        if max == 0 || used != 1 << max {
            return None;
        }
    }
    let mut offs = [0u16; 16];
    for l in 1..15 {
        offs[l + 1] = offs[l] + counts[l];
    }
    let mut symbols = vec![0u16; lengths.len()];
    for (s, &l) in lengths.iter().enumerate() {
        if l != 0 {
            symbols[usize::from(offs[usize::from(l)])] = s as u16;
            offs[usize::from(l)] += 1;
        }
    }
    counts[0] = 0;
    // Canonical codes, MSB first; the stream holds them bit-reversed.
    let mut fast = vec![0u16; 1 << FAST];
    let mut code = 0u32;
    let mut index = 0usize;
    for (len, &count) in counts.iter().enumerate().take(FAST as usize + 1).skip(1) {
        for _ in 0..count {
            let rev = (0..len).fold(0u32, |r, i| r | ((code >> i) & 1) << (len - 1 - i));
            let entry = (len as u16) << 12 | symbols[index];
            let mut k = rev as usize;
            while k < fast.len() {
                fast[k] = entry;
                k += 1 << len;
            }
            code += 1;
            index += 1;
        }
        code <<= 1;
    }
    Some(Huffman {
        counts,
        symbols,
        fast,
    })
}

/// Decode one symbol; `Bad` for a code outside the set.
fn decode_sym(bits: &mut Bits<'_>, h: &Huffman) -> Result<u16, Z> {
    let entry = h.fast[bits.peek(FAST) as usize];
    let len = usize::from(entry >> 12);
    if len != 0 && len <= bits.avail() {
        bits.pos += len;
        return Ok(entry & 0xfff);
    }
    let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
    for len in 1..16 {
        code |= bits.need(1)? as i32;
        let count = i32::from(h.counts[len]);
        if code - count < first {
            return Ok(h.symbols[(index + (code - first)) as usize]);
        }
        index += count;
        first += count;
        first <<= 1;
        code <<= 1;
    }
    Err(Z::Bad)
}

/// The next symbol without consuming it, if its bits are all present.
fn peek_sym(bits: &Bits<'_>, h: &Huffman) -> Option<u16> {
    let mut probe = Bits {
        data: bits.data,
        pos: bits.pos,
        limit: bits.limit,
    };
    decode_sym(&mut probe, h).ok()
}

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn fixed_tables() -> (Huffman, Huffman) {
    let mut l = [0u8; 288];
    l[..144].fill(8);
    l[144..256].fill(9);
    l[256..280].fill(7);
    l[280..].fill(8);
    // Both sets are complete as built; symbols 286/287 and distances 30/31 are rejected on use.
    match (huffman(&l, false), huffman(&[5u8; 32], true)) {
        (Some(a), Some(b)) => (a, b),
        _ => unreachable_tables(),
    }
}

fn unreachable_tables() -> (Huffman, Huffman) {
    let empty = || Huffman {
        counts: [0; 16],
        symbols: Vec::new(),
        fast: vec![0; 1 << FAST],
    };
    (empty(), empty())
}

/// Where the decoder is between steps.
enum St {
    Header,
    Stored(usize),
    Codes(Huffman, Huffman),
    Done,
}

/// How the image's last byte was produced, which decides what the decoder does next.
enum Full {
    /// A copy ran past the image end and is still queued: nothing more is read.
    Queued,
    /// The decoder is at a step boundary in state `St`; the flag says the block is the final one.
    At(St, bool),
}

/// Inflate a zlib stream (the concatenated payloads, chunk `ends` as cumulative byte offsets)
/// for exactly `need` bytes.
fn inflate_zlib(data: &[u8], ends: &[usize], need: usize) -> Result<Vec<u8>, PngError> {
    use PngError::Omitted;
    if data.len() < 2 {
        return Err(Omitted);
    }
    let (cmf, flg) = (data[0], data[1]);
    if (u16::from(cmf) << 8 | u16::from(flg)) % 31 != 0
        || cmf & 0x0f != 8
        || cmf >> 4 > 7
        || flg & 0x20 != 0
    {
        return Err(Omitted);
    }
    let mut out = Vec::new();
    let mut bits = Bits {
        data,
        pos: 16,
        limit: data.len() * 8,
    };
    let full = fill(&mut bits, &mut out, need).map_err(|e| {
        if e == Z::Alloc {
            PngError::Allocation
        } else {
            Omitted
        }
    })?;
    if let Full::At(state, last) = full {
        // Only the rest of the chunk holding the last consumed bit is ever fed.
        let last_byte = bits.pos.saturating_sub(1) / 8;
        let end = ends
            .iter()
            .copied()
            .find(|&e| e > last_byte)
            .unwrap_or(data.len());
        bits.limit = end * 8;
        if after_full(&mut bits, state, last) == Err(Z::Bad) {
            return Err(Omitted);
        }
    }
    Ok(out)
}

/// Decode until `out` holds `need` bytes; any error or end of data first is a failure.
fn fill(bits: &mut Bits<'_>, out: &mut Vec<u8>, need: usize) -> Result<Full, Z> {
    let mut state = St::Header;
    let mut last = false;
    if need == 0 {
        return Ok(Full::At(state, false));
    }
    loop {
        state = match state {
            St::Done => return Err(Z::Bad),
            St::Header => {
                last = bits.need(1)? == 1;
                match bits.need(2)? {
                    0 => {
                        bits.align();
                        let len = bits.need(16)?;
                        let nlen = bits.need(16)?;
                        if len != !nlen & 0xffff {
                            return Err(Z::Bad);
                        }
                        St::Stored(len as usize)
                    }
                    1 => {
                        let (l, d) = fixed_tables();
                        St::Codes(l, d)
                    }
                    2 => {
                        let (l, d) = dynamic_header(bits, false)?.ok_or(Z::Eof)?;
                        St::Codes(l, d)
                    }
                    _ => return Err(Z::Bad),
                }
            }
            St::Stored(mut left) => {
                while left > 0 {
                    put(out, bits.need(8)? as u8, need)?;
                    left -= 1;
                    if out.len() == need {
                        return Ok(Full::At(
                            if left > 0 {
                                St::Stored(left)
                            } else {
                                next(last)
                            },
                            last,
                        ));
                    }
                }
                next(last)
            }
            St::Codes(lit, dist) => loop {
                let sym = decode_sym(bits, &lit)?;
                if sym < 256 {
                    put(out, sym as u8, need)?;
                    if out.len() == need {
                        return Ok(Full::At(St::Codes(lit, dist), last));
                    }
                    continue;
                }
                if sym == 256 {
                    break next(last);
                }
                let s = usize::from(sym - 257);
                if s >= 29 {
                    return Err(Z::Bad);
                }
                let len = usize::from(LEN_BASE[s]) + bits.need(u32::from(LEN_EXTRA[s]))? as usize;
                let ds = usize::from(decode_sym(bits, &dist)?);
                if ds >= 30 {
                    return Err(Z::Bad);
                }
                let d = usize::from(DIST_BASE[ds]) + bits.need(u32::from(DIST_EXTRA[ds]))? as usize;
                if d > out.len() {
                    return Err(Z::Bad);
                }
                let room = need - out.len();
                for _ in 0..len.min(room) {
                    put(out, out[out.len() - d], need)?;
                }
                if len > room {
                    return Ok(Full::Queued);
                }
                if out.len() == need {
                    return Ok(Full::At(St::Codes(lit, dist), last));
                }
            },
        };
        if let St::Done = state {
            return Err(Z::Bad);
        }
    }
}

fn next(last: bool) -> St {
    if last { St::Done } else { St::Header }
}

/// What the decoder still reads once the image is complete. `Ok` when it stops quietly.
fn after_full(bits: &mut Bits<'_>, mut state: St, mut last: bool) -> Result<(), Z> {
    loop {
        state = match state {
            St::Done | St::Stored(1..) => return Ok(()),
            St::Stored(0) => next(last),
            St::Codes(lit, _) => {
                // A block end is taken only when 15 bits are buffered.
                if bits.avail() < 15 || peek_sym(bits, &lit) != Some(256) {
                    return Ok(());
                }
                decode_sym(bits, &lit)?;
                next(last)
            }
            St::Header => {
                if bits.avail() < 10 {
                    return Ok(());
                }
                last = bits.peek(1) == 1;
                match bits.peek(3) >> 1 {
                    0 => {
                        let header = 3 + ((8 - (bits.pos + 3) % 8) % 8) + 32;
                        if bits.avail() < header {
                            return Ok(());
                        }
                        bits.need(3)?;
                        bits.align();
                        let len = bits.need(16)?;
                        if len != !bits.need(16)? & 0xffff {
                            return Err(Z::Bad);
                        }
                        St::Stored(len as usize)
                    }
                    1 => {
                        bits.need(3)?;
                        if bits.peek(7) == 0 {
                            bits.need(7)?;
                            if last {
                                return Ok(());
                            }
                            while bits.avail() >= 10 && bits.peek(10) == 0b010 {
                                bits.need(10)?;
                            }
                            St::Header
                        } else {
                            let (l, d) = fixed_tables();
                            St::Codes(l, d)
                        }
                    }
                    2 => {
                        if bits.avail() < 17 {
                            return Ok(());
                        }
                        bits.need(3)?;
                        match dynamic_header(bits, true)? {
                            Some((l, d)) => St::Codes(l, d),
                            None => return Ok(()),
                        }
                    }
                    _ => return Err(Z::Bad),
                }
            }
        };
    }
}

/// A dynamic block's header after its 3 type bits. With `waiting`, a step whose bits are not all
/// present stops quietly (`Ok(None)`), with the decoder's own thresholds; otherwise that is `Eof`.
fn dynamic_header(bits: &mut Bits<'_>, waiting: bool) -> Result<Option<(Huffman, Huffman)>, Z> {
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let stop = |b: &Bits<'_>, n: usize| -> Result<bool, Z> {
        if b.avail() >= n {
            Ok(false)
        } else if waiting {
            Ok(true)
        } else {
            Err(Z::Eof)
        }
    };
    if stop(bits, 14)? {
        return Ok(None);
    }
    let nlen = bits.need(5)? as usize + 257;
    let ndist = bits.need(5)? as usize + 1;
    let ncode = bits.need(4)? as usize + 4;
    if nlen > 286 || ndist > 30 {
        return Err(Z::Bad);
    }
    if stop(bits, 3 * ncode)? {
        return Ok(None);
    }
    let mut lengths = [0u8; 19];
    for &i in ORDER.iter().take(ncode) {
        lengths[i] = bits.need(3)? as u8;
    }
    let lencode = huffman(&lengths, false).ok_or(Z::Bad)?;
    let mut ll = vec![0u8; nlen + ndist];
    let mut i = 0;
    while i < nlen + ndist {
        if stop(bits, 7)? {
            return Ok(None);
        }
        let probe = peek_sym(bits, &lencode).ok_or(Z::Bad)?;
        let extra = match probe {
            16 => 2,
            17 => 3,
            18 => 7,
            _ => 0,
        };
        let code_len = {
            let mut p = Bits {
                data: bits.data,
                pos: bits.pos,
                limit: bits.limit,
            };
            decode_sym(&mut p, &lencode)?;
            p.pos - bits.pos
        };
        if stop(bits, code_len + extra)? {
            return Ok(None);
        }
        let sym = decode_sym(bits, &lencode)?;
        if sym < 16 {
            ll[i] = sym as u8;
            i += 1;
            continue;
        }
        let (val, rep) = match sym {
            16 => {
                if i == 0 {
                    return Err(Z::Bad);
                }
                (ll[i - 1], 3 + bits.need(2)? as usize)
            }
            17 => (0, 3 + bits.need(3)? as usize),
            _ => (0, 11 + bits.need(7)? as usize),
        };
        if i + rep > nlen + ndist {
            return Err(Z::Bad);
        }
        ll[i..i + rep].fill(val);
        i += rep;
    }
    if ll[256] == 0 {
        return Err(Z::Bad);
    }
    let lit = huffman(&ll[..nlen], false).ok_or(Z::Bad)?;
    let dist = huffman(&ll[nlen..], true).ok_or(Z::Bad)?;
    Ok(Some((lit, dist)))
}
