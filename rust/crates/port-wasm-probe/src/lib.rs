// SPDX-License-Identifier: AGPL-3.0-only
//! Private safe test ABI for independent browser execution of frozen input packets.
//! Not a product interface: linker exports use checked v0 symbol suffixes.
use std::sync::Mutex;
mod files;
mod obj;

const MAX_INPUT: usize = 1 << 20;
const MAX_OUTPUT: usize = 1 << 20;
const MAX_SCALARS: usize = 30_000;
const MAX_POINTS: usize = 10_000;
struct State {
    kind: u32,
    failed: bool,
    input: Vec<u8>,
    output: Vec<u8>,
}
static STATE: Mutex<State> = Mutex::new(State {
    kind: 0,
    failed: true,
    input: Vec::new(),
    output: Vec::new(),
});

/// Reset state for mode 1 (TLS), 2 (mesh graph), 3 (marked plane), 4 (raw PLY), 5 (OBJ), 6 (OBJ helpers), or 7 (companion file states).
pub extern "C" fn scanruler_wasm_reset(kind: u32) -> u32 {
    let Ok(mut s) = STATE.lock() else {
        return 255;
    };
    s.input.clear();
    s.output.clear();
    s.kind = kind;
    s.failed = !(1..=7).contains(&kind);
    u32::from(s.failed)
}
/// Append one to four little-endian input bytes; a failed write poisons this run.
pub extern "C" fn scanruler_wasm_push(word: u32, count: u32) -> u32 {
    let Ok(mut s) = STATE.lock() else {
        return 255;
    };
    if s.failed {
        return 1;
    }
    if !(1..=4).contains(&count) || s.input.len() + count as usize > MAX_INPUT {
        s.failed = true;
        s.output.clear();
        return 2;
    }
    if s.input.try_reserve(count as usize).is_err() {
        s.failed = true;
        s.output.clear();
        return 3;
    }
    s.input
        .extend_from_slice(&word.to_le_bytes()[..count as usize]);
    0
}
/// Execute geometry using input only; failures clear output instead of reusing it.
pub extern "C" fn scanruler_wasm_run() -> u32 {
    let Ok(mut s) = STATE.lock() else {
        return 255;
    };
    s.output.clear();
    if s.failed {
        return 1;
    }
    match execute(s.kind, &s.input) {
        Ok(output) => {
            s.output = output;
            0
        }
        Err(()) => {
            s.failed = true;
            2
        }
    }
}
/// Number of completed output bytes, zero after reset or a host/codec failure.
pub extern "C" fn scanruler_wasm_length() -> u32 {
    let Ok(s) = STATE.lock() else {
        return 0;
    };
    s.output.len() as u32
}
/// Read four little-endian result bytes (zero padding past the final byte).
pub extern "C" fn scanruler_wasm_word(offset: u32) -> u32 {
    let Ok(s) = STATE.lock() else {
        return 0;
    };
    let mut b = [0; 4];
    for (i, v) in b.iter_mut().enumerate() {
        *v = (offset as usize)
            .checked_add(i)
            .and_then(|at| s.output.get(at))
            .copied()
            .unwrap_or(0);
    }
    u32::from_le_bytes(b)
}

struct Input<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Input<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn word(&mut self) -> Result<u32, ()> {
        let end = self.offset.checked_add(4).ok_or(())?;
        let chunk = self.bytes.get(self.offset..end).ok_or(())?;
        let mut b = [0; 4];
        b.copy_from_slice(chunk);
        self.offset = end;
        Ok(u32::from_le_bytes(b))
    }
    fn number(&mut self) -> Result<f64, ()> {
        let lo = self.word()?;
        let hi = self.word()?;
        Ok(f64::from_bits(u64::from(lo) | (u64::from(hi) << 32)))
    }
    fn floats(&mut self, n: usize) -> Result<Vec<f32>, ()> {
        if n > MAX_SCALARS || n * 4 > self.bytes.len().saturating_sub(self.offset) {
            return Err(());
        }
        let mut out = Vec::new();
        out.try_reserve_exact(n).map_err(|_| ())?;
        for _ in 0..n {
            out.push(f32::from_bits(self.word()?));
        }
        Ok(out)
    }
    fn indices(&mut self) -> Result<Vec<u32>, ()> {
        let bytes = self.bytes.len().saturating_sub(self.offset);
        let n = bytes / 4;
        if !bytes.is_multiple_of(4) || n > MAX_POINTS {
            return Err(());
        }
        let mut out = Vec::new();
        out.try_reserve_exact(n).map_err(|_| ())?;
        for _ in 0..n {
            out.push(self.word()?);
        }
        Ok(out)
    }
}
struct Output(Vec<u8>);
impl Output {
    fn bytes(&mut self, b: &[u8]) -> Result<(), ()> {
        if b.len() > MAX_OUTPUT.saturating_sub(self.0.len()) {
            return Err(());
        }
        self.0.try_reserve(b.len()).map_err(|_| ())?;
        self.0.extend_from_slice(b);
        Ok(())
    }
    fn word(&mut self, n: usize) -> Result<(), ()> {
        self.bytes(&(n as u32).to_le_bytes())
    }
    fn number(&mut self, v: f64) -> Result<(), ()> {
        self.bytes(&v.to_le_bytes())
    }
    fn text(&mut self, s: &str) -> Result<(), ()> {
        self.word(s.len())?;
        self.bytes(s.as_bytes())
    }
    fn floats(&mut self, v: &[f32]) -> Result<(), ()> {
        self.word(v.len())?;
        for x in v {
            self.bytes(&x.to_le_bytes())?;
        }
        Ok(())
    }
    fn indices(&mut self, v: &[u32]) -> Result<(), ()> {
        self.word(v.len())?;
        for x in v {
            self.bytes(&x.to_le_bytes())?;
        }
        Ok(())
    }
}
fn execute(kind: u32, bytes: &[u8]) -> Result<Vec<u8>, ()> {
    let mut i = Input::new(bytes);
    let mut o = Output(Vec::new());
    match kind {
        1 => {
            let n = i.word()? as usize;
            let p = i.floats(n)?;
            let idx = i.indices()?;
            let fit = port_core::plane::fit_plane_tls(&p, &idx).map_err(|_| ())?;
            o.bytes(&[u8::from(fit.is_some())])?;
            if let Some(fit) = fit {
                for v in fit.normal.into_iter().chain([fit.d]) {
                    o.number(v)?;
                }
                for j in idx {
                    let j = j as usize * 3;
                    o.number(fit.residual(
                        f64::from(p[j]),
                        f64::from(p[j + 1]),
                        f64::from(p[j + 2]),
                    ))?;
                }
            }
        }
        2 => {
            let flags = i.word()?;
            if flags & 0xff000000 != 0 {
                return Err(());
            }
            let n = i.word()? as usize;
            let p = i.floats(n)?;
            let indices = if flags & 0xff0000 != 0 {
                Some(i.indices()?)
            } else {
                if i.offset != bytes.len() {
                    return Err(());
                }
                None
            };
            let mut progress = Vec::new();
            let result = port_core::mesh_graph::build_mesh_graph(
                port_core::mesh_graph::ParsedMesh {
                    positions: p,
                    indices,
                    soup: flags & 255 != 0,
                    keep_winding: flags & 0xff00 != 0,
                },
                port_core::mesh_graph::GraphLimits {
                    vertices: MAX_POINTS,
                    indices: MAX_POINTS,
                    neighbors: MAX_POINTS * 2,
                },
                |s| progress.push(s.to_owned()),
            );
            match result {
                Ok(g) => {
                    o.bytes(&[0])?;
                    o.word(g.vertex_count)?;
                    o.word(g.triangle_count)?;
                    o.number(g.bbox_diag)?;
                    o.floats(&g.positions)?;
                    o.indices(&g.indices)?;
                    o.floats(&g.normals)?;
                    o.indices(&g.adjacency.offsets)?;
                    o.indices(&g.adjacency.list)?;
                }
                Err(port_core::mesh_graph::GraphError::Source(s)) => {
                    o.bytes(&[1])?;
                    o.text(s)?;
                }
                Err(_) => return Err(()),
            }
            o.word(progress.len())?;
            for s in progress {
                o.text(&s)?;
            }
        }
        3 => {
            let n = i.word()? as usize;
            let sigma = i.number()?;
            let p = i.floats(n)?;
            let normals = i.floats(n)?;
            let idx = i.indices()?;
            match port_core::marked_plane::fit_plane_on_selection(
                &p, &normals, &idx, sigma, MAX_POINTS,
            ) {
                Ok(r) => {
                    o.bytes(&[0])?;
                    o.text("plane")?;
                    for x in r
                        .center
                        .into_iter()
                        .chain(r.normal)
                        .chain(r.basis_u)
                        .chain(r.basis_v)
                        .chain([
                            r.extent_u,
                            r.extent_v,
                            r.sigma,
                            r.form_error,
                            r.residual_min,
                            r.residual_max,
                        ])
                    {
                        o.number(x)?;
                    }
                    o.word(r.used_points)?;
                    o.word(r.region_size)?;
                    o.indices(&r.region)?;
                }
                Err(port_core::marked_plane::MarkedPlaneError::Source(s)) => {
                    o.bytes(&[1])?;
                    o.text(&s)?;
                }
                Err(_) => return Err(()),
            }
            for t in 0..45 {
                let a = (t as f64 / 45.) * (std::f64::consts::PI / 2.);
                o.number(a.cos())?;
                o.number(a.sin())?;
            }
        }
        4 => {
            let mut events = String::new();
            let mut event_failure = false;
            let parsed = port_io::ply::read_ply(
                bytes,
                port_io::ply::PlyLimits {
                    max_bytes: MAX_INPUT,
                    max_vertices: MAX_POINTS,
                    max_triangles: MAX_POINTS,
                    max_work: 4_000_000,
                },
                |s| {
                    if s.len() + 1 > MAX_OUTPUT.saturating_sub(events.len())
                        || events.try_reserve(s.len() + 1).is_err()
                    {
                        event_failure = true;
                    } else {
                        events.push_str(s);
                        events.push('\n');
                    }
                },
            );
            if event_failure {
                return Err(());
            }
            match parsed {
                Ok(mesh) => {
                    o.bytes(&[0])?;
                    o.word(16 + 4 * (mesh.positions.len() + mesh.indices.len()))?;
                    o.bytes(b"PLY1")?;
                    o.word(usize::from(mesh.keep_winding))?;
                    o.word(mesh.positions.len())?;
                    o.word(mesh.indices.len())?;
                    for v in mesh.positions {
                        o.bytes(&v.to_bits().to_le_bytes())?;
                    }
                    for v in mesh.indices {
                        o.bytes(&v.to_le_bytes())?;
                    }
                }
                Err(
                    port_io::ply::PlyError::ResourceLimit
                    | port_io::ply::PlyError::Allocation
                    | port_io::ply::PlyError::BinaryNotPorted,
                ) => return Err(()),
                Err(error) => {
                    o.bytes(&[2])?;
                    o.word(0)?;
                    let message = format!("{error}\n");
                    events.try_reserve(message.len()).map_err(|_| ())?;
                    events.push_str(&message);
                }
            }
            o.text(&events)?;
        }
        5 | 6 => return obj::execute(kind, bytes),
        7 => return files::execute(bytes),
        _ => return Err(()),
    }
    Ok(o.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codec_and_output_are_bounded() {
        assert!(execute(0, &[]).is_err());
        assert!(execute(1, &[]).is_err());
        assert!(execute(1, &u32::MAX.to_le_bytes()).is_err());
        let out = execute(1, &0u32.to_le_bytes());
        assert_eq!(out, Ok(vec![0]));
        let mut o = Output(Vec::new());
        assert!(o.bytes(&vec![0; MAX_OUTPUT + 1]).is_err());
    }
    #[test]
    fn ply_source_refusal_and_host_failure_are_distinct() {
        let source = execute(4, &[]).expect("source refusal is a completed comparison result");
        assert_eq!(source[0], 2);
        assert_eq!(&source[1..5], &[0; 4]);
        assert_eq!(&source[9..], b"Not a valid PLY file.\n");
        let huge = b"ply\nformat ascii 1.0\nelement vertex 10001\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nend_header\n";
        assert!(
            execute(4, huge).is_err(),
            "host budget must not become a source refusal"
        );
    }
    #[test]
    fn failed_write_clears_output_until_reset() {
        assert_eq!(scanruler_wasm_reset(1), 0);
        assert_eq!(scanruler_wasm_push(0, 4), 0);
        assert_eq!(scanruler_wasm_run(), 0);
        assert_eq!(scanruler_wasm_length(), 1);
        assert_ne!(scanruler_wasm_push(0, 0), 0);
        assert_eq!(scanruler_wasm_length(), 0);
        assert_ne!(scanruler_wasm_run(), 0);
        assert_ne!(scanruler_wasm_reset(9), 0);
        assert_ne!(scanruler_wasm_run(), 0);
        assert_eq!(scanruler_wasm_length(), 0);
        assert_eq!(scanruler_wasm_reset(1), 0);
        assert_ne!(scanruler_wasm_run(), 0);
        assert_eq!(scanruler_wasm_length(), 0);
    }
}
