// SPDX-License-Identifier: AGPL-3.0-only
//! Bounded input-only private codecs for complete OBJ parser/helper execution.
use super::{MAX_INPUT, MAX_OUTPUT, MAX_POINTS, MAX_SCALARS, Output};
use port_io::obj::{ObjError, ObjLimits, read_obj};
use port_io::obj_texture::*;

fn texture_limits() -> TextureLimits {
    TextureLimits {
        max_bytes: MAX_INPUT,
        max_vertices: MAX_POINTS,
        max_files: 64,
        max_materials: 1024,
        max_name_bytes: 4096,
        max_work: 4_000_000,
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ()> {
        let end = self.at.checked_add(n).ok_or(())?;
        let b = self.bytes.get(self.at..end).ok_or(())?;
        self.at = end;
        Ok(b)
    }
    fn byte(&mut self) -> Result<u8, ()> {
        Ok(self.take(1)?[0])
    }
    fn word(&mut self) -> Result<u32, ()> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| ())?,
        ))
    }
    fn number(&mut self) -> Result<f64, ()> {
        Ok(f64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| ())?,
        ))
    }
    // Check both a semantic cap and minimum remaining packet bytes before reserving.
    fn count(&mut self, max: usize, width: usize) -> Result<usize, ()> {
        let n = self.word()? as usize;
        if n > max || n > self.bytes.len().saturating_sub(self.at) / width {
            return Err(());
        }
        Ok(n)
    }
    fn blob(&mut self) -> Result<Vec<u8>, ()> {
        let n = self.count(MAX_INPUT, 1)?;
        let bytes = self.take(n)?;
        let mut v = Vec::new();
        v.try_reserve_exact(n).map_err(|_| ())?;
        v.extend_from_slice(bytes);
        Ok(v)
    }
    fn text(&mut self, max: usize) -> Result<String, ()> {
        let n = self.count(max, 1)?;
        let text = std::str::from_utf8(self.take(n)?).map_err(|_| ())?;
        let mut s = String::new();
        s.try_reserve_exact(n).map_err(|_| ())?;
        s.push_str(text);
        Ok(s)
    }
    fn array<T>(
        &mut self,
        max: usize,
        width: usize,
        mut read: impl FnMut(&mut Self) -> Result<T, ()>,
    ) -> Result<Vec<T>, ()> {
        let n = self.count(max, width)?;
        let mut v = Vec::new();
        v.try_reserve_exact(n).map_err(|_| ())?;
        for _ in 0..n {
            v.push(read(self)?);
        }
        Ok(v)
    }
    fn strings(&mut self, max: usize) -> Result<Vec<String>, ()> {
        self.array(max, 4, |r| r.text(4096))
    }
    fn image(&mut self) -> Result<TextureImage, ()> {
        Ok(TextureImage {
            name: self.text(4096)?,
            width: self.word()?,
            height: self.word()?,
            rgba: self.blob()?,
        })
    }
    fn companions(&mut self) -> Result<Option<ObjCompanions>, ()> {
        match self.byte()? {
            0 => Ok(None),
            1 => {
                let materials = self.array(64, 8, |r| {
                    Ok(MaterialLibrary {
                        name: r.text(4096)?,
                        text: r.text(MAX_INPUT)?,
                    })
                })?;
                let images = self.array(64 - materials.len(), 16, Self::image)?;
                Ok(Some(ObjCompanions { materials, images }))
            }
            _ => Err(()),
        }
    }
    fn refs(&mut self) -> Result<ObjTextureRefs, ()> {
        Ok(ObjTextureRefs {
            uvs: self.array(MAX_POINTS * 2, 8, Self::number)?,
            vertex_uv: self.array(MAX_POINTS, 4, |r| Ok(r.word()? as i32))?,
            vertex_material: self.array(MAX_POINTS, 4, |r| Ok(r.word()? as i32))?,
            material_names: self.strings(1024)?,
            libraries: self.strings(64)?,
        })
    }
}
fn strings(o: &mut Output, v: &[String]) -> Result<(), ()> {
    o.word(v.len())?;
    for s in v {
        o.text(s)?;
    }
    Ok(())
}
fn note(o: &mut Output, v: Option<&TextureNote>) -> Result<(), ()> {
    match v {
        None => o.bytes(&[0]),
        Some(TextureNote::Missing(v)) => {
            o.bytes(&[1])?;
            strings(o, v)
        }
        Some(TextureNote::Applied(v)) => {
            o.bytes(&[2])?;
            strings(o, v)
        }
    }
}
pub(super) fn execute(kind: u32, bytes: &[u8]) -> Result<Vec<u8>, ()> {
    if bytes.len() > MAX_INPUT {
        return Err(());
    }
    let mut r = Reader { bytes, at: 0 };
    let mut o = Output(Vec::new());
    if kind == 5 {
        let n = r.count(MAX_INPUT, 1)?;
        let obj = r.take(n)?;
        let companions = r.companions()?;
        if r.at != bytes.len() {
            return Err(());
        }
        let mut progress = Vec::new();
        let mut progress_bytes: usize = 0;
        let mut failed = false;
        let result = read_obj(
            obj,
            ObjLimits {
                max_bytes: MAX_INPUT,
                max_vertices: MAX_POINTS,
                max_uvs: MAX_POINTS,
                max_indices: MAX_SCALARS,
                max_face_vertices: MAX_POINTS,
                max_materials: 1024,
                max_libraries: 64,
                max_work: 4_000_000,
                texture: texture_limits(),
            },
            |s| {
                let mut text = String::new();
                if s.len() + 4 > MAX_OUTPUT.saturating_sub(progress_bytes)
                    || progress.try_reserve(1).is_err()
                    || text.try_reserve_exact(s.len()).is_err()
                {
                    failed = true;
                } else {
                    text.push_str(s);
                    progress_bytes += s.len() + 4;
                    progress.push(text);
                }
            },
            companions.as_ref(),
        );
        if failed {
            return Err(());
        }
        o.bytes(b"OBJ1")?;
        match result {
            Ok(mesh) => {
                o.bytes(&[0])?;
                o.floats(&mesh.positions)?;
                o.indices(&mesh.indices)?;
                if let Some(c) = mesh.colors {
                    o.bytes(&[1])?;
                    o.word(c.len())?;
                    o.bytes(&c)?;
                } else {
                    o.bytes(&[0])?;
                }
                note(&mut o, mesh.texture.as_ref())?;
            }
            Err(
                e @ (ObjError::InvalidVertex(_)
                | ObjError::InvalidFaceIndex(_)
                | ObjError::NoVertices
                | ObjError::NoFaces),
            ) => {
                o.bytes(&[2])?;
                o.text(&e.to_string())?;
            }
            Err(_) => return Err(()),
        }
        strings(&mut o, &progress)?;
    } else if kind == 6 {
        let op = r.byte()?;
        o.bytes(b"OTX1")?;
        o.bytes(&[op])?;
        match op {
            1 => o.text(&base_name(&r.text(4096)?, texture_limits()).map_err(|_| ())?)?,
            2 => {
                let map = parse_mtl(&r.text(MAX_INPUT)?, texture_limits()).map_err(|_| ())?;
                o.word(map.len())?;
                for (key, value) in map {
                    o.text(&key)?;
                    o.text(&value)?;
                }
            }
            3 => strings(
                &mut o,
                &mtllib_names(&r.text(MAX_INPUT)?, texture_limits()).map_err(|_| ())?,
            )?,
            4 => {
                let image = r.image()?;
                let u = r.number()?;
                let v = r.number()?;
                let offset = r.word()? as usize;
                let mut pixels = r.blob()?;
                sample_texture(&image, u, v, &mut pixels, offset).map_err(|_| ())?;
                o.word(pixels.len())?;
                o.bytes(&pixels)?;
            }
            5 => {
                let refs = r.refs()?;
                let companions = r.companions()?;
                match bake_obj_texture(&refs, companions.as_ref(), texture_limits())
                    .map_err(|_| ())?
                {
                    None => o.bytes(&[0])?,
                    Some(BakedTexture {
                        colors: None,
                        note: TextureNote::Missing(v),
                    }) => {
                        o.bytes(&[1])?;
                        strings(&mut o, &v)?;
                    }
                    Some(BakedTexture {
                        colors: Some(c),
                        note: TextureNote::Applied(v),
                    }) => {
                        o.bytes(&[2])?;
                        o.word(c.len())?;
                        o.bytes(&c)?;
                        strings(&mut o, &v)?;
                    }
                    Some(_) => return Err(()),
                }
            }
            _ => return Err(()),
        }
    } else {
        return Err(());
    }
    if r.at != bytes.len() {
        return Err(());
    }
    Ok(o.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hostile_packets_cannot_allocate_from_declarations_or_reuse_trailing_data() {
        for kind in [5, 6] {
            assert!(execute(kind, &[]).is_err());
            assert!(execute(kind, &[255; 32]).is_err());
        }
        let mut packet = vec![5]; // helper refs: oversized UV declaration
        packet.extend(u32::MAX.to_le_bytes());
        assert!(execute(6, &packet).is_err());
        let mut empty_obj = vec![0; 5];
        let source = execute(5, &empty_obj);
        assert!(source.as_ref().is_ok_and(|v| &v[..5] == b"OBJ1\x02"));
        empty_obj.push(0);
        assert!(execute(5, &empty_obj).is_err());
        let invalid_utf8 = [1, 1, 0, 0, 0, 255];
        assert!(execute(6, &invalid_utf8).is_err());
    }
    #[test]
    fn sampler_output_guard_and_file_budget_are_host_failures() {
        // Empty name, 1x1 image, no RGBA; UV zero; offset 0, only two output bytes.
        let mut packet = vec![4];
        for n in [0u32, 1, 1, 0] {
            packet.extend(n.to_le_bytes());
        }
        packet.extend([0; 16]);
        for n in [0u32, 2] {
            packet.extend(n.to_le_bytes());
        }
        packet.extend([9, 9]);
        assert!(execute(6, &packet).is_err());
        let mut r = Reader {
            bytes: &[1, 255, 255, 255, 255],
            at: 0,
        };
        assert!(r.companions().is_err());
    }
}
