// SPDX-License-Identifier: AGPL-3.0-only
//! Bounded input-only codec for complete owned ingress file state comparisons.
use super::{Input, MAX_INPUT, Output};
use port_io::scan_files::{FORMAT_CONSTANTS, FileId, FileLimits, FileSet, classify};
const MAX_FILES: usize = 64;
const MAX_OPERATIONS: usize = 64;
impl<'a> Input<'a> {
    fn file_take(&mut self, n: usize) -> Result<&'a [u8], ()> {
        let end = self.offset.checked_add(n).ok_or(())?;
        let b = self.bytes.get(self.offset..end).ok_or(())?;
        self.offset = end;
        Ok(b)
    }
    fn file_byte(&mut self) -> Result<u8, ()> {
        Ok(self.file_take(1)?[0])
    }
    fn file_count(&mut self, max: usize, width: usize) -> Result<usize, ()> {
        let n = self.word()? as usize;
        if n > max || n > self.bytes.len().saturating_sub(self.offset) / width {
            return Err(());
        }
        Ok(n)
    }
    fn file_blob(&mut self, max: usize) -> Result<&'a [u8], ()> {
        let n = self.file_count(max, 1)?;
        self.file_take(n)
    }
    fn file_text(&mut self) -> Result<&'a str, ()> {
        std::str::from_utf8(self.file_blob(4096)?).map_err(|_| ())
    }
    fn file_ids(&mut self, files: &[FileId]) -> Result<Vec<FileId>, ()> {
        let n = self.file_count(MAX_FILES, 4)?;
        let mut v = Vec::new();
        v.try_reserve_exact(n).map_err(|_| ())?;
        for _ in 0..n {
            v.push(*files.get(self.word()? as usize).ok_or(())?);
        }
        Ok(v)
    }
}
fn ids(out: &mut Output, v: &[FileId]) -> Result<(), ()> {
    out.word(v.len())?;
    for id in v {
        out.word(id.index())?;
    }
    Ok(())
}
pub(super) fn execute(bytes: &[u8]) -> Result<Vec<u8>, ()> {
    if bytes.len() > MAX_INPUT {
        return Err(());
    }
    let limits = FileLimits {
        max_files: MAX_FILES,
        max_bytes: MAX_INPUT,
        max_name_bytes: 4096,
        max_links: 4096,
        max_work: 4_000_000,
    };
    let mut r = Input::new(bytes);
    let mode = r.file_byte()?;
    let mut out = Output(Vec::new());
    out.bytes(b"FLS1")?;
    out.bytes(&[mode])?;
    match mode {
        0 => {
            let info = classify(r.file_text()?, limits).map_err(|_| ())?;
            out.text(&info.extension)?;
            out.bytes(&info.flags.map(u8::from))?;
            for text in FORMAT_CONSTANTS {
                out.text(text)?;
            }
        }
        1 => {
            let n = r.file_count(MAX_FILES, 8)?;
            let mut set = FileSet::new(limits);
            let mut files = Vec::new();
            files.try_reserve_exact(n).map_err(|_| ())?;
            for _ in 0..n {
                let name = r.file_text()?;
                let bytes = r.file_blob(MAX_INPUT)?;
                files.push(set.insert(name, bytes).map_err(|_| ())?);
            }
            let n = r.file_count(MAX_OPERATIONS, 5)?;
            out.word(n)?;
            for _ in 0..n {
                let op = r.file_byte()?;
                let args = r.file_ids(&files)?;
                out.bytes(&[op])?;
                match op {
                    1 => out.word(
                        set.bundle(&args)
                            .map_err(|_| ())?
                            .map_or(u32::MAX as usize, FileId::index),
                    )?,
                    2 => {
                        if args.len() != 1 {
                            return Err(());
                        }
                        ids(&mut out, set.companions(args[0]).map_err(|_| ())?)?;
                    }
                    3 => {
                        let copies = set.companion_bytes(&args).map_err(|_| ())?;
                        out.word(copies.len())?;
                        for file in copies {
                            out.text(&file.name)?;
                            out.word(file.bytes.len())?;
                            out.bytes(&file.bytes)?;
                        }
                    }
                    _ => return Err(()),
                }
                out.word(files.len())?;
                for &file in &files {
                    ids(&mut out, set.companions(file).map_err(|_| ())?)?;
                }
            }
        }
        _ => return Err(()),
    }
    if r.offset != bytes.len() {
        return Err(());
    }
    Ok(out.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_utf8_ids_and_trailing_data_fail_without_source_outcomes() {
        for packet in [
            &[][..],
            &[255][..],
            &[0, 255, 255, 255, 255][..],
            &[0, 1, 0, 0, 0, 255][..],
            &[1, 255, 255, 255, 255][..],
        ] {
            assert!(execute(packet).is_err());
        }
        let valid = [0; 5];
        assert!(execute(&valid).is_ok());
        let mut trailing = valid.to_vec();
        trailing.push(0);
        assert!(execute(&trailing).is_err());
        // One empty file, one bundle operation referencing a non-existent file.
        let mut packet = vec![1];
        for n in [1u32, 0, 0, 1] {
            packet.extend(n.to_le_bytes());
        }
        packet.push(1);
        for n in [1u32, 1] {
            packet.extend(n.to_le_bytes());
        }
        assert!(execute(&packet).is_err());
        let mut too_many = vec![1];
        for n in [0u32, 65] {
            too_many.extend(n.to_le_bytes());
        }
        too_many.extend([0; 325]);
        assert!(execute(&too_many).is_err());
    }
}
