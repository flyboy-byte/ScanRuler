// SPDX-License-Identifier: AGPL-3.0-only
//! Tool-only capture adapters: preserve existing corpus/comparator bodies and fail
//! compilation if their observation points change. Product sources are untouched.
#![allow(clippy::unwrap_used)]
use std::{env, fs, path::PathBuf};
fn replace(source: &mut String, from: &str, to: &str) {
    assert_eq!(
        source.matches(from).count(),
        1,
        "capture anchor changed: {from}"
    );
    *source = source.replacen(from, to, 1);
}
fn main() {
    let base = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for (name, path) in [
        ("zip", "zip_inflate.rs"),
        ("threemf", "support/threemf_corpus.rs"),
        ("png", "png.rs"),
    ] {
        let input = base.join("../crates/port-io/tests").join(path);
        println!("cargo:rerun-if-changed={}", input.display());
        let text = fs::read_to_string(input).unwrap();
        let mut source = text
            .lines()
            .filter(|l| {
                !l.starts_with("//!")
                    && !l.starts_with("#![")
                    && *l != "#[test]"
                    && !l.starts_with("#[ignore")
            })
            .collect::<Vec<_>>()
            .join("\n")
            .replace(".join(\"../../..\")", ".join(\"../..\")");
        if name == "threemf" {
            let parent = base.join("../crates/port-io/tests/threemf.rs");
            println!("cargo:rerun-if-changed={}", parent.display());
            let helpers = fs::read_to_string(parent).unwrap();
            let a = helpers.find("use port_io::inflate::ZipLimits;").unwrap();
            let b = helpers.find("\nfn model(").unwrap();
            source = source.replace("use super::{limits, stored_zip};", "");
            source = format!("{}\n{}", helpers[a..b].replace("use port_io::obj_texture::{ObjCompanions, TextureImage, TextureLimits, TextureNote};", "use port_io::obj_texture::TextureLimits;").replace("use port_io::threemf::{LimitKind, ThreeMfError, ThreeMfLimits, read_3mf};", "use port_io::threemf::ThreeMfLimits;"), source);
        }
        if name == "zip" {
            replace(
                &mut source,
                "fn pinned_zip_inflate_comparison() {",
                "pub fn capture() {\nlet recording = crate::Recording::new(\"zip-inflate-diff\");",
            );
            replace(
                &mut source,
                "            let matched = match (&rust, orig) {",
                "            recording.case(&format!(\"seed{seed}-case{k}\"), input, format!(\"{orig:?}\").as_bytes(), format!(\"{rust:?}\").as_bytes());\n            let matched = match (&rust, orig) {",
            );
        } else if name == "threemf" {
            replace(
                &mut source,
                "pub fn run_comparison() {",
                "pub fn capture() {\nlet recording = crate::Recording::new(\"threemf-diff\");",
            );
            replace(
                &mut source,
                "            match compare(&rust, &progress, original, sabotage) {",
                "            let mut capture_input = zip.clone();\n            capture_input.extend(format!(\"\\nimages={:?}\", companions.images).as_bytes());\n            recording.case(&format!(\"seed{seed}-case{k}\"), &capture_input, format!(\"{original:?}\").as_bytes(), format!(\"{rust:?} progress={progress:?}\").as_bytes());\n            match compare(&rust, &progress, original, sabotage) {",
            );
        } else {
            source.push_str(r#"
pub fn capture() {
    let recording = crate::Recording::new("png-fixed-diff");
    let sabotage = std::env::var_os("SCANRULER_PNG_FIXED_SABOTAGE").is_some();
    for set in ["named", "sweep-s1", "struct-s11"] {
        let cases = read_batch(&fixture(&format!("{set}.bin.gz")));
        let chrome = read_results(&fixture(&format!("{set}.chrome.bin.gz")));
        assert!(!cases.is_empty());
        for (k, (input, expected)) in cases.iter().zip(&chrome).enumerate() {
            recording.case(&format!("{set}-{k}"), input, format!("{expected:?}").as_bytes(), format!("{:?}", rust_outcome(input)).as_bytes());
        }
        assert!(compare(&cases, &chrome, sabotage).mismatches.is_empty(), "wrong PNG pixels: {set}");
    }
    for (id, expected) in audit_capture().into_iter().filter(|(id, _)| id.contains("png")) {
        let input = std::fs::read(root().join(format!("docs/port-plan/fixtures/image-audit/{id}.bin"))).unwrap();
        let got = rust_outcome(&input);
        recording.case(&format!("audit-{id}"), &input, format!("{expected:?}").as_bytes(), format!("{got:?}").as_bytes());
        assert_eq!(got, Ok(expected));
    }
    comparison_rejects_a_wrong_pixel();
}
"#);
        }
        fs::write(out.join(format!("{name}.rs")), source).unwrap();
    }
}
