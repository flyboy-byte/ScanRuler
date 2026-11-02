// SPDX-License-Identifier: AGPL-3.0-only
// Authored input/output framing only. Expected/source results are never inputs.
use port_io::scan_files::*;
pub fn limits() -> FileLimits {
    FileLimits {
        max_files: 64,
        max_bytes: 1 << 20,
        max_name_bytes: 4096,
        max_links: 4096,
        max_work: 4_000_000,
    }
}
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let b = &self.b[self.at..self.at + n];
        self.at += n;
        b
    }
    fn byte(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn word(&mut self) -> usize {
        u32::from_le_bytes(self.take(4).try_into().unwrap()) as usize
    }
    fn blob(&mut self) -> &'a [u8] {
        let n = self.word();
        self.take(n)
    }
    fn text(&mut self) -> &'a str {
        std::str::from_utf8(self.blob()).unwrap()
    }
}
fn word(b: &mut Vec<u8>, n: usize) {
    b.extend((n as u32).to_le_bytes());
}
fn blob(b: &mut Vec<u8>, v: &[u8]) {
    word(b, v.len());
    b.extend(v);
}
fn ids(b: &mut Vec<u8>, v: &[FileId]) {
    word(b, v.len());
    for id in v {
        word(b, id.index());
    }
}
pub fn run(input: &[u8]) -> Vec<u8> {
    let mut r = Reader { b: input, at: 0 };
    let mode = r.byte();
    let mut out = b"FLS1".to_vec();
    out.push(mode);
    match mode {
        0 => {
            let f = classify(r.text(), limits()).unwrap();
            blob(&mut out, f.extension.as_bytes());
            out.extend(f.flags.map(u8::from));
            for s in FORMAT_CONSTANTS {
                blob(&mut out, s.as_bytes());
            }
        }
        1 => {
            let n = r.word();
            let mut set = FileSet::new(limits());
            let mut files = Vec::new();
            for _ in 0..n {
                let name = r.text();
                files.push(set.insert(name, r.blob()).unwrap());
            }
            let n = r.word();
            word(&mut out, n);
            for _ in 0..n {
                let op = r.byte();
                let n = r.word();
                let args: Vec<_> = (0..n).map(|_| files[r.word()]).collect();
                out.push(op);
                match op {
                    1 => word(
                        &mut out,
                        set.bundle(&args)
                            .unwrap()
                            .map_or(u32::MAX as usize, FileId::index),
                    ),
                    2 => {
                        assert_eq!(args.len(), 1);
                        ids(&mut out, set.companions(args[0]).unwrap());
                    }
                    3 => {
                        let v = set.companion_bytes(&args).unwrap();
                        word(&mut out, v.len());
                        for f in v {
                            blob(&mut out, f.name.as_bytes());
                            blob(&mut out, &f.bytes);
                        }
                    }
                    _ => panic!("bad fixed op"),
                }
                word(&mut out, files.len());
                for &id in &files {
                    ids(&mut out, set.companions(id).unwrap());
                }
            }
        }
        _ => panic!("bad fixed mode"),
    }
    assert_eq!(r.at, input.len());
    out
}
