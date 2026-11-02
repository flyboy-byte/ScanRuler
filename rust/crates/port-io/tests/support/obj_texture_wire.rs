// SPDX-License-Identifier: AGPL-3.0-only
// Test-only input/output framing. No original result is decoded as a Rust input.
use port_io::obj_texture::*;
pub struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }
    fn take(&mut self, n: usize) -> &'a [u8] {
        let b = &self.bytes[self.at..self.at + n];
        self.at += n;
        b
    }
    pub fn byte(&mut self) -> u8 {
        self.take(1)[0]
    }
    pub fn word(&mut self) -> usize {
        u32::from_le_bytes(self.take(4).try_into().unwrap()) as usize
    }
    pub fn string(&mut self) -> String {
        let n = self.word();
        String::from_utf8(self.take(n).to_vec()).unwrap()
    }
    pub fn strings(&mut self) -> Vec<String> {
        let n = self.word();
        (0..n).map(|_| self.string()).collect()
    }
    fn floats(&mut self) -> Vec<f64> {
        let n = self.word();
        (0..n)
            .map(|_| f64::from_le_bytes(self.take(8).try_into().unwrap()))
            .collect()
    }
    fn ints(&mut self) -> Vec<i32> {
        let n = self.word();
        (0..n)
            .map(|_| i32::from_le_bytes(self.take(4).try_into().unwrap()))
            .collect()
    }
    pub fn image(&mut self) -> TextureImage {
        let name = self.string();
        let width = self.word() as u32;
        let height = self.word() as u32;
        let n = self.word();
        let rgba = self.take(n).to_vec();
        TextureImage {
            name,
            width,
            height,
            rgba,
        }
    }
    pub fn refs(&mut self) -> ObjTextureRefs {
        ObjTextureRefs {
            uvs: self.floats(),
            vertex_uv: self.ints(),
            vertex_material: self.ints(),
            material_names: self.strings(),
            libraries: self.strings(),
        }
    }
    pub fn companions(&mut self) -> Option<ObjCompanions> {
        if self.byte() == 0 {
            return None;
        }
        let n = self.word();
        let materials = (0..n)
            .map(|_| MaterialLibrary {
                name: self.string(),
                text: self.string(),
            })
            .collect();
        let n = self.word();
        let images = (0..n).map(|_| self.image()).collect();
        Some(ObjCompanions { materials, images })
    }
}
pub fn word(out: &mut Vec<u8>, n: usize) {
    out.extend((n as u32).to_le_bytes());
}
pub fn string(out: &mut Vec<u8>, s: &str) {
    word(out, s.len());
    out.extend(s.as_bytes());
}
pub fn strings(out: &mut Vec<u8>, v: &[String]) {
    word(out, v.len());
    for s in v {
        string(out, s);
    }
}
pub fn baked(out: &mut Vec<u8>, v: Option<BakedTexture>) {
    match v {
        None => out.push(0),
        Some(BakedTexture {
            colors: None,
            note: TextureNote::Missing(files),
        }) => {
            out.push(1);
            strings(out, &files);
        }
        Some(BakedTexture {
            colors: Some(c),
            note: TextureNote::Applied(images),
        }) => {
            out.push(2);
            word(out, c.len());
            out.extend(c);
            strings(out, &images);
        }
        other => panic!("invalid Rust result shape {other:?}"),
    }
}
pub fn limits() -> TextureLimits {
    TextureLimits {
        max_bytes: 16 * 1024 * 1024,
        max_vertices: 1_000_000,
        max_files: 64,
        max_materials: 4096,
        max_name_bytes: 4096,
        max_work: 64_000_000,
    }
}
pub fn run(bytes: &[u8]) -> Result<Vec<u8>, TextureError> {
    let mut r = Reader::new(bytes);
    let mode = r.byte();
    let mut out = b"OTX1".to_vec();
    out.push(mode);
    match mode {
        1 => string(&mut out, &base_name(&r.string(), limits())?),
        2 => {
            let map = parse_mtl(&r.string(), limits())?;
            word(&mut out, map.len());
            for (k, v) in map {
                string(&mut out, &k);
                string(&mut out, &v);
            }
        }
        3 => strings(&mut out, &mtllib_names(&r.string(), limits())?),
        4 => {
            let img = r.image();
            let u = f64::from_le_bytes(r.take(8).try_into().unwrap());
            let v = f64::from_le_bytes(r.take(8).try_into().unwrap());
            let offset = r.word();
            let n = r.word();
            let mut pixels = r.take(n).to_vec();
            sample_texture(&img, u, v, &mut pixels, offset)?;
            word(&mut out, pixels.len());
            out.extend(pixels);
        }
        5 => {
            let refs = r.refs();
            let given = r.companions();
            baked(&mut out, bake_obj_texture(&refs, given.as_ref(), limits())?);
        }
        _ => panic!("unknown fixed test operation"),
    };
    assert_eq!(r.at, bytes.len());
    Ok(out)
}
