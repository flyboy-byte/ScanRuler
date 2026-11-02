// SPDX-License-Identifier: AGPL-3.0-only
//! Full parser outputs, texture continuation, progress and exact source refusals.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_io::obj::{ObjError, ObjLimits, ObjMesh, read_obj};
use port_io::obj_texture::*;
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let x = &self.b[self.at..self.at + n];
        self.at += n;
        x
    }
    fn byte(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn word(&mut self) -> usize {
        u32::from_le_bytes(self.take(4).try_into().unwrap()) as usize
    }
    fn string(&mut self) -> String {
        let n = self.word();
        String::from_utf8(self.take(n).to_vec()).unwrap()
    }
    fn image(&mut self) -> TextureImage {
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
    fn companions(&mut self) -> Option<ObjCompanions> {
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
struct Case {
    id: &'static str,
    input: &'static [u8],
    success: bool,
}
fn cases() -> Vec<Case> {
    include!("support/obj_mesh_cases.rs")
}
fn limits() -> ObjLimits {
    ObjLimits {
        max_bytes: 16 * 1024 * 1024,
        max_vertices: 2_000_000,
        max_uvs: 2_000_000,
        max_indices: 6_000_000,
        max_face_vertices: 1_000_000,
        max_materials: 4096,
        max_libraries: 64,
        max_work: 64_000_000,
        texture: TextureLimits {
            max_bytes: 64 * 1024 * 1024,
            max_vertices: 2_000_000,
            max_files: 64,
            max_materials: 4096,
            max_name_bytes: 4096,
            max_work: 64_000_000,
        },
    }
}
fn word(b: &mut Vec<u8>, n: usize) {
    b.extend((n as u32).to_le_bytes());
}
fn string(b: &mut Vec<u8>, s: &str) {
    word(b, s.len());
    b.extend(s.as_bytes());
}
fn strings(b: &mut Vec<u8>, v: &[String]) {
    word(b, v.len());
    for s in v {
        string(b, s);
    }
}
fn payload(result: &Result<ObjMesh, ObjError>, progress: &[String]) -> Vec<u8> {
    let mut b = b"OBJ1".to_vec();
    match result {
        Ok(mesh) => {
            b.push(0);
            word(&mut b, mesh.positions.len());
            b.extend(
                mesh.positions
                    .iter()
                    .flat_map(|x| x.to_bits().to_le_bytes()),
            );
            word(&mut b, mesh.indices.len());
            b.extend(mesh.indices.iter().flat_map(|i| i.to_le_bytes()));
            if let Some(c) = &mesh.colors {
                b.push(1);
                word(&mut b, c.len());
                b.extend(c);
            } else {
                b.push(0);
            }
            match &mesh.texture {
                None => b.push(0),
                Some(TextureNote::Missing(v)) => {
                    b.push(1);
                    strings(&mut b, v);
                }
                Some(TextureNote::Applied(v)) => {
                    b.push(2);
                    strings(&mut b, v);
                }
            }
        }
        Err(e) => {
            b.push(2);
            string(&mut b, &e.to_string());
        }
    }
    strings(&mut b, progress);
    b
}
fn run(input: &[u8]) -> (Result<ObjMesh, ObjError>, Vec<String>) {
    let mut r = Reader { b: input, at: 0 };
    let n = r.word();
    let bytes = r.take(n);
    let given = r.companions();
    assert_eq!(r.at, input.len());
    let mut progress = Vec::new();
    let result = read_obj(
        bytes,
        limits(),
        |s| progress.push(s.to_owned()),
        given.as_ref(),
    );
    (result, progress)
}
#[test]
fn parser_cases_and_host_limits() {
    for c in cases() {
        let (result, _) = run(c.input);
        assert_eq!(result.is_ok(), c.success, "{}: {result:?}", c.id);
    }
    let triangle = b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
    for l in [
        ObjLimits {
            max_bytes: 0,
            ..limits()
        },
        ObjLimits {
            max_vertices: 2,
            ..limits()
        },
        ObjLimits {
            max_face_vertices: 2,
            ..limits()
        },
        ObjLimits {
            max_indices: 2,
            ..limits()
        },
        ObjLimits {
            max_work: 0,
            ..limits()
        },
    ] {
        assert_eq!(read_obj(triangle, l, |_| {}, None), Err(ObjError::Limit));
    }
    assert_eq!(
        read_obj(
            b"vt 0 0\n",
            ObjLimits {
                max_uvs: 0,
                ..limits()
            },
            |_| {},
            None
        ),
        Err(ObjError::Limit)
    );
    assert_eq!(
        read_obj(
            b"usemtl a\n",
            ObjLimits {
                max_materials: 0,
                ..limits()
            },
            |_| {},
            None
        ),
        Err(ObjError::Limit)
    );
    assert_eq!(
        read_obj(
            b"mtllib a.mtl\n",
            ObjLimits {
                max_libraries: 0,
                ..limits()
            },
            |_| {},
            None
        ),
        Err(ObjError::Limit)
    );
}
#[test]
#[ignore = "live complete OBJ parser comparison requires Node 22 and frozen source modules"]
fn pinned_obj_mesh_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run_dir = root
        .join("rust/target/obj-mesh-diff")
        .join(format!("run-{stamp}"));
    std::fs::create_dir_all(&run_dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(run_dir.join("candidate.txt"), sha.stdout).unwrap();
    let mut identity = String::from(
        "contract: io.obj.mesh.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: complete positions/indices/colors/notes, progress/refusals exact\nseeds: 1,7,99,1039\n",
    );
    for (label, p, args) in [
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["--version"]),
    ] {
        let o = Command::new(p)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(o.status.success());
        identity.push_str(&format!(
            "{label}: {}\n",
            String::from_utf8_lossy(&o.stdout).trim()
        ));
    }
    std::fs::write(run_dir.join("identity.txt"), identity).unwrap();
    let cases = cases();
    assert_eq!(cases.len(), 81);
    let mut successes = 0;
    for c in cases {
        let (result, progress) = run(c.input);
        let candidate = payload(&result, &progress);
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/obj-mesh.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(c.input).unwrap();
        let original = child.wait_with_output().unwrap();
        let dir = run_dir.join(c.id);
        std::fs::create_dir_all(&dir).unwrap();
        for (name, b) in [
            ("input.bin", c.input),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", candidate.as_slice()),
            ("original.stderr.txt", original.stderr.as_slice()),
        ] {
            std::fs::write(dir.join(name), b).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert!(original.stderr.is_empty(), "{}", c.id);
        assert_eq!(result.is_ok(), c.success, "{}: {result:?}", c.id);
        assert_eq!(original.stdout[4] == 0, c.success, "{}", c.id);
        if let Err(e) = &result {
            assert!(
                matches!(
                    e,
                    ObjError::InvalidVertex(_)
                        | ObjError::InvalidFaceIndex(_)
                        | ObjError::NoVertices
                        | ObjError::NoFaces
                ),
                "{}: host failure cannot pass {e}",
                c.id
            );
        } else {
            successes += 1;
        }
        assert_eq!(
            candidate, original.stdout,
            "{}: complete mesh/state/progress/error",
            c.id
        );
        if c.id == "texture-applied" {
            let mut changed = candidate.clone();
            changed[9] ^= 1;
            assert_ne!(changed, original.stdout, "changed coordinate must fail");
            assert_ne!(
                &candidate[..candidate.len() - 1],
                original.stdout,
                "missing progress/note cannot pass"
            );
        }
        println!("{}: complete mesh/state/progress/refusal match", c.id);
    }
    assert_eq!(successes, 53);
    println!(
        "OBJ: 53 complete outputs /28 refusals; raw {}",
        run_dir.display()
    );
}
