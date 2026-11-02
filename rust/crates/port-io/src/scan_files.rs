// SPDX-License-Identifier: AGPL-3.0-only
//! Complete file classification and companion selection/byte preservation.
//!
//! Owned ingress data only: no image codec, browser drop traversal or engine state.
//! File IDs are local to one set; files are never removed/reused within that set.

/// Picker/display constants, in the original exported order.
pub const FORMAT_CONSTANTS: [&str; 6] = [
    ".stl,.ply,.obj,.3mf,.step,.stp,.mtl,.png,.jpg,.jpeg,.bmp,.webp",
    ".stl,.ply,.obj,.3mf,.step,.stp",
    ".png,.jpg,.jpeg",
    "STL, PLY, OBJ, 3MF or STEP · mm",
    "STL, PLY, OBJ, 3MF or STEP",
    "PNG or JPEG",
];
const MODELS: [&str; 6] = ["stl", "ply", "obj", "3mf", "step", "stp"];
const TEXTURES: [&str; 6] = ["mtl", "png", "jpg", "jpeg", "bmp", "webp"];

/// Explicit host limits; these are not source refusals or numerical tolerances.
#[derive(Clone, Copy, Debug)]
pub struct FileLimits {
    /// Files owned and maximum references in a single operation.
    pub max_files: usize,
    /// Aggregate owned name/data bytes and bytes copied by one snapshot operation.
    pub max_bytes: usize,
    /// UTF-8 bytes in one filename.
    pub max_name_bytes: usize,
    /// Total saved companion references across every file.
    pub max_links: usize,
    /// Traversal/name/byte-copy work allowed per operation.
    pub max_work: usize,
}
/// Host-only failures. No invalid file identifier can become a source outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileError {
    /// Explicit count/byte/name/link budget exceeded.
    Limit,
    /// Per-operation work budget exceeded.
    Work,
    /// A reference is outside this ingress set.
    InvalidId,
    /// A fallible allocation failed.
    Allocation,
}
fn own_bytes(b: &[u8]) -> Result<Vec<u8>, FileError> {
    let mut v = Vec::new();
    v.try_reserve_exact(b.len())
        .map_err(|_| FileError::Allocation)?;
    v.extend_from_slice(b);
    Ok(v)
}
fn own_text(s: &str) -> Result<String, FileError> {
    let mut v = String::new();
    v.try_reserve_exact(s.len())
        .map_err(|_| FileError::Allocation)?;
    v.push_str(s);
    Ok(v)
}
fn spend(work: &mut usize, n: usize, limits: FileLimits) -> Result<(), FileError> {
    *work = work.checked_add(n).ok_or(FileError::Work)?;
    if *work > limits.max_work {
        return Err(FileError::Work);
    }
    Ok(())
}
fn extension(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}
fn matches(name: &str, choices: &[&str]) -> bool {
    // Every recognized extension is ASCII and none contains K (the Unicode
    // Kelvin sign is the only non-ASCII lowercase-to-ASCII mapping). Unknown
    // extensions remain unknown. Full lowercasing is retained in classify.
    choices
        .iter()
        .any(|s| extension(name).eq_ignore_ascii_case(s))
}
/// Full extension and six source classifier flags, in exported source order.
#[derive(Debug, PartialEq, Eq)]
pub struct FileFormat {
    /// Last dot-separated segment, Unicode lowercase; no path stripping/trim.
    pub extension: String,
    /// Model, texture, 3MF, STEP, reference, flat-image membership.
    pub flags: [bool; 6],
}
/// Classify one valid UTF-8 filename with the original registry semantics.
pub fn classify(name: &str, limits: FileLimits) -> Result<FileFormat, FileError> {
    if name.len() > limits.max_name_bytes {
        return Err(FileError::Limit);
    }
    let mut work = 0;
    spend(&mut work, name.len(), limits)?;
    // Standard lowercase preserves contextual final sigma. Its temporary is
    // bounded by the independently checked filename length, as in obj_texture.
    let ext = extension(name).to_lowercase();
    let model = MODELS.contains(&ext.as_str());
    Ok(FileFormat {
        flags: [
            model,
            TEXTURES.contains(&ext.as_str()),
            ext == "3mf",
            ext == "step" || ext == "stp",
            model,
            ["png", "jpg", "jpeg"].contains(&ext.as_str()),
        ],
        extension: ext,
    })
}
/// A stable index owned by exactly one ingress set; never serialized as engine ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(usize);
impl FileId {
    /// Input insertion order, for platform bookkeeping and test observations.
    pub fn index(self) -> usize {
        self.0
    }
}
/// Complete preserved file data, independent of any returned byte snapshot.
#[derive(Debug, PartialEq, Eq)]
pub struct OwnedFile {
    /// Original UTF-8 filename, unmodified.
    pub name: String,
    /// Original file data, including malformed image/MTL bytes.
    pub bytes: Vec<u8>,
}
/// Source WeakMap bookkeeping represented within a bounded owned file lifetime.
/// This is neither an engine session nor a second application-state store.
#[derive(Debug)]
pub struct FileSet {
    files: Vec<OwnedFile>,
    links: Vec<Vec<FileId>>,
    bytes: usize,
    link_count: usize,
    limits: FileLimits,
}
impl FileSet {
    /// Empty ingress owner; dropping it releases files and companion associations.
    pub fn new(limits: FileLimits) -> Self {
        Self {
            files: Vec::new(),
            links: Vec::new(),
            bytes: 0,
            link_count: 0,
            limits,
        }
    }
    /// Own a new, distinct file identity; failure leaves observable state intact.
    pub fn insert(&mut self, name: &str, bytes: &[u8]) -> Result<FileId, FileError> {
        if self.files.len() >= self.limits.max_files || name.len() > self.limits.max_name_bytes {
            return Err(FileError::Limit);
        }
        let added = name
            .len()
            .checked_add(bytes.len())
            .ok_or(FileError::Limit)?;
        let total = self.bytes.checked_add(added).ok_or(FileError::Limit)?;
        if total > self.limits.max_bytes {
            return Err(FileError::Limit);
        }
        let mut work = 0;
        spend(&mut work, added, self.limits)?;
        let file = OwnedFile {
            name: own_text(name)?,
            bytes: own_bytes(bytes)?,
        };
        self.files
            .try_reserve(1)
            .map_err(|_| FileError::Allocation)?;
        self.links
            .try_reserve(1)
            .map_err(|_| FileError::Allocation)?;
        let id = FileId(self.files.len());
        self.files.push(file);
        self.links.push(Vec::new());
        self.bytes = total;
        Ok(id)
    }
    /// Read original data without giving a caller mutable access to owned bytes.
    pub fn file(&self, id: FileId) -> Result<&OwnedFile, FileError> {
        self.files.get(id.0).ok_or(FileError::InvalidId)
    }
    fn validate(&self, ids: &[FileId]) -> Result<usize, FileError> {
        if ids.len() > self.limits.max_files {
            return Err(FileError::Limit);
        }
        let mut work = 0;
        for &id in ids {
            spend(
                &mut work,
                self.file(id)?.name.len().saturating_add(1),
                self.limits,
            )?;
        }
        Ok(work)
    }
    /// Complete bundleFiles: first model, texture companions, or first-file fallback.
    /// A later bundle with no texture companions preserves the previous association.
    pub fn bundle(&mut self, ids: &[FileId]) -> Result<Option<FileId>, FileError> {
        let mut work = self.validate(ids)?;
        let model = ids
            .iter()
            .copied()
            .find(|id| matches(&self.files[id.0].name, &MODELS));
        let Some(model) = model else {
            return Ok(ids.first().copied());
        };
        let mut rest = Vec::new();
        for &id in ids {
            spend(
                &mut work,
                self.files[id.0].name.len().saturating_add(1),
                self.limits,
            )?;
            if id != model && matches(&self.files[id.0].name, &TEXTURES) {
                if self.link_count - self.links[model.0].len() + rest.len() >= self.limits.max_links
                {
                    return Err(FileError::Limit);
                }
                rest.try_reserve(1).map_err(|_| FileError::Allocation)?;
                rest.push(id);
            }
        }
        if !rest.is_empty() {
            self.link_count = self.link_count - self.links[model.0].len() + rest.len();
            self.links[model.0] = rest;
        }
        Ok(Some(model))
    }
    /// Complete companionsOf for a file, retaining reference order and duplicates.
    pub fn companions(&self, id: FileId) -> Result<&[FileId], FileError> {
        self.links
            .get(id.0)
            .map(Vec::as_slice)
            .ok_or(FileError::InvalidId)
    }
    /// Complete companionBytes over already read owned input data: ordered fresh
    /// name/data copies, including duplicates. No image parsing or filtering.
    pub fn companion_bytes(&self, ids: &[FileId]) -> Result<Vec<OwnedFile>, FileError> {
        let mut work = self.validate(ids)?;
        let mut bytes: usize = 0;
        for &id in ids {
            let file = self.file(id)?;
            bytes = bytes
                .checked_add(file.name.len())
                .and_then(|n| n.checked_add(file.bytes.len()))
                .ok_or(FileError::Limit)?;
            if bytes > self.limits.max_bytes {
                return Err(FileError::Limit);
            }
            spend(&mut work, file.bytes.len(), self.limits)?;
        }
        let mut out = Vec::new();
        out.try_reserve_exact(ids.len())
            .map_err(|_| FileError::Allocation)?;
        for &id in ids {
            let f = self.file(id)?;
            out.push(OwnedFile {
                name: own_text(&f.name)?,
                bytes: own_bytes(&f.bytes)?,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_ids_cannot_read_or_change_live_links() -> Result<(), FileError> {
        let mut set = FileSet::new(FileLimits {
            max_files: 8,
            max_bytes: 1000,
            max_name_bytes: 100,
            max_links: 8,
            max_work: 1000,
        });
        let model = set.insert("a.obj", b"")?;
        let image = set.insert("a.png", b"data")?;
        set.bundle(&[model, image])?;
        let invalid = FileId(usize::MAX);
        assert_eq!(set.file(invalid), Err(FileError::InvalidId));
        assert_eq!(set.companions(invalid), Err(FileError::InvalidId));
        assert_eq!(set.bundle(&[model, invalid]), Err(FileError::InvalidId));
        assert_eq!(
            set.companion_bytes(&[image, invalid]),
            Err(FileError::InvalidId)
        );
        assert_eq!(set.companions(model)?, [image]);
        Ok(())
    }
}
