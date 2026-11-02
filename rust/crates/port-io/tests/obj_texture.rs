// SPDX-License-Identifier: AGPL-3.0-only
//! Complete source values/state, not status-only checks for texture helpers.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "support/obj_texture_wire.rs"]
mod wire;
use port_io::obj_texture::*;
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/obj_texture_cases.rs")
}
#[test]
fn texture_inputs_limits_and_atomic_output_guards() {
    for c in cases() {
        assert!(wire::run(c.input).is_ok(), "{}", c.id);
    }
    let limits = wire::limits();
    assert_eq!(
        base_name(
            "long.png",
            TextureLimits {
                max_name_bytes: 1,
                ..limits
            }
        ),
        Err(TextureError::Limit)
    );
    assert_eq!(
        parse_mtl(
            "newmtl a\nmap_Kd x.png",
            TextureLimits {
                max_materials: 0,
                ..limits
            }
        ),
        Err(TextureError::Limit)
    );
    assert_eq!(
        parse_mtl(
            "newmtl a",
            TextureLimits {
                max_work: 0,
                ..limits
            }
        ),
        Err(TextureError::Work)
    );
    assert_eq!(
        mtllib_names(
            "a.mtl b.mtl",
            TextureLimits {
                max_files: 1,
                ..limits
            }
        ),
        Err(TextureError::Limit)
    );
    let img = TextureImage {
        name: "sample".into(),
        width: 1,
        height: 1,
        rgba: vec![255, 0, 0, 0],
    };
    let mut out = [11, 22];
    assert_eq!(
        sample_texture(&img, 0., 0., &mut out, 0),
        Err(TextureError::Output)
    );
    assert_eq!(out, [11, 22]);
    assert_eq!(
        sample_texture(&img, 0., 0., &mut out, usize::MAX),
        Err(TextureError::Output)
    );
    assert_eq!(out, [11, 22]);
    let refs = ObjTextureRefs {
        uvs: vec![0., 0.],
        vertex_uv: vec![0],
        ..ObjTextureRefs::default()
    };
    let given = ObjCompanions {
        images: vec![img],
        ..ObjCompanions::default()
    };
    assert_eq!(
        bake_obj_texture(
            &refs,
            Some(&given),
            TextureLimits {
                max_bytes: 0,
                ..limits
            }
        ),
        Err(TextureError::Limit)
    );
    assert_eq!(
        bake_obj_texture(
            &refs,
            Some(&given),
            TextureLimits {
                max_vertices: 0,
                ..limits
            }
        ),
        Err(TextureError::Limit)
    );
}
#[test]
#[ignore = "live OBJ texture helper comparison requires Node 22 and frozen TypeScript dependency"]
fn pinned_obj_texture_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/obj-texture-diff")
        .join(format!("run-{stamp}"));
    std::fs::create_dir_all(&run).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(run.join("candidate.txt"), sha.stdout).unwrap();
    let mut identity = String::from(
        "contract: io.obj.texture.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: all strings, ordered maps/notes and complete color/output bytes exact\nseeds: 1,7,99,1039\n",
    );
    for (label, program, args) in [
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["--version"]),
    ] {
        let o = Command::new(program)
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
    std::fs::write(run.join("identity.txt"), identity).unwrap();
    let cases = cases();
    assert_eq!(cases.len(), 144);
    for c in cases {
        let candidate = wire::run(c.input);
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/obj-texture.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(c.input).unwrap();
        let source = child.wait_with_output().unwrap();
        let dir = run.join(c.id);
        std::fs::create_dir_all(&dir).unwrap();
        for (name, b) in [
            ("input.bin", c.input),
            ("original.bin", source.stdout.as_slice()),
            ("original.stderr.txt", source.stderr.as_slice()),
        ] {
            std::fs::write(dir.join(name), b).unwrap();
        }
        let actual = candidate.as_ref().map(Vec::as_slice).unwrap_or(&[]);
        std::fs::write(dir.join("rust.bin"), actual).unwrap();
        assert!(
            source.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&source.stderr)
        );
        assert!(source.stderr.is_empty(), "{}", c.id);
        assert!(candidate.is_ok(), "{}: {candidate:?}", c.id);
        assert_eq!(actual, source.stdout, "{}: complete values/state", c.id);
        if c.id == "bake-applied" {
            let mut wrong = actual.to_vec();
            wrong[10] ^= 1;
            assert_ne!(wrong, source.stdout, "changed color bit must fail");
            assert_ne!(
                &actual[..actual.len() - 1],
                source.stdout,
                "missing note data must fail"
            );
        }
        println!("{}: all helper values/state match exactly", c.id);
    }
    println!("144 complete helper comparisons; raw {}", run.display());
}
