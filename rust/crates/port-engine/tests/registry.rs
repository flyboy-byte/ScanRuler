// SPDX-License-Identifier: AGPL-3.0-only
//! Full original registration/admission operation states, not fake app commands.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::registry::{Command, Limits, Registry, RegistryError};
include!("support/value_wire.rs");
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/registry_cases.rs")
}
fn get<'a>(v: &'a Value, k: &str) -> &'a Value {
    if let Value::Object(a) = v
        && let Some((_, v)) = a
            .iter()
            .find(|(n, _)| n.iter().copied().eq(k.encode_utf16()))
    {
        return v;
    }
    &Value::Undefined
}
fn list(v: &Value) -> &[Value] {
    if let Value::Array(a) = v {
        a
    } else {
        panic!("fixture list")
    }
}
fn text(v: &Value) -> &[u16] {
    if let Value::String(s) = v {
        s
    } else {
        panic!("fixture text")
    }
}
fn integer(v: &Value) -> usize {
    if let Value::Number(n) = v {
        *n as usize
    } else {
        panic!("fixture integer")
    }
}
fn obj(fields: Vec<(&str, Value)>) -> Value {
    Value::Object(
        fields
            .into_iter()
            .map(|(k, v)| (k.encode_utf16().collect(), v))
            .collect(),
    )
}
fn str_(v: &str) -> Value {
    Value::String(v.encode_utf16().collect())
}
fn err(e: RegistryError) -> Value {
    match e {
        RegistryError::Registration(message) => obj(vec![
            ("name", str_("Error")),
            ("message", Value::String(message)),
        ]),
        RegistryError::Command(e) => obj(vec![
            ("name", str_(e.name())),
            ("message", Value::String(e.message_units().to_vec())),
            ("code", str_(e.code().as_str())),
        ]),
        RegistryError::ValidationFault(_) => obj(vec![
            ("name", str_("TypeError")),
            (
                "message",
                str_("Reduce of empty array with no initial value"),
            ),
        ]),
        e => panic!("unexpected host refusal {e:?}"),
    }
}
fn input(b: &[u8]) -> Value {
    let mut r = Reader { b, at: 0 };
    let v = r.value();
    assert_eq!(r.at, b.len());
    v
}
fn payload(b: &[u8]) -> Vec<u8> {
    let case = input(b);
    let defs = list(get(&case, "defs"));
    let mut registry = Registry::new();
    let mut handles = Vec::new();
    let mut observations = Vec::new();
    for op in list(get(&case, "ops")) {
        let kind = String::from_utf16(text(get(op, "op"))).unwrap();
        let mut value = Value::Undefined;
        let mut error = Value::Undefined;
        let mut after = Value::Undefined;
        let mut target = Value::Undefined;
        let result: Result<(), RegistryError> = match kind.as_str() {
            "register" => {
                let commands: Vec<_> = list(get(op, "indices"))
                    .iter()
                    .map(|v| {
                        let i = integer(v);
                        Command {
                            definition: defs[i].clone(),
                            body: i,
                        }
                    })
                    .collect();
                registry
                    .register(text(get(op, "namespace")), &commands, Limits::default())
                    .map(|h| {
                        value = Value::Number(handles.len() as f64);
                        handles.push(h);
                    })
            }
            "cleanup" => registry.cleanup(handles[integer(get(op, "handle"))]),
            "admit" => {
                let mut given = get(op, "input").clone();
                let result = registry.admit(text(get(op, "name")), &mut given, Limits::default());
                after = given;
                result.map(|a| {
                    value = a.input;
                    target = Value::Number(a.body as f64);
                })
            }
            "headless" => {
                value = Value::Boolean(registry.runs_headless(text(get(op, "name"))));
                Ok(())
            }
            "list" => Ok(()),
            _ => panic!("fixture op"),
        };
        if let Err(e) = result {
            error = err(e)
        }
        let commands = registry.list().unwrap();
        let headless = Value::Array(
            list(&commands)
                .iter()
                .map(|c| Value::Boolean(registry.runs_headless(text(get(c, "name")))))
                .collect(),
        );
        observations.push(obj(vec![
            ("value", value),
            ("error", error),
            ("afterInput", after),
            ("target", target),
            ("commands", commands),
            ("headless", headless),
        ]));
    }
    let mut b = b"REG3".to_vec();
    encode(&mut b, &Value::Array(observations));
    b
}
#[test]
fn host_limits_and_owned_definitions() {
    let c = cases()
        .into_iter()
        .find(|c| c.id == "unknown-input")
        .unwrap();
    let case = input(c.input);
    let definition = list(get(&case, "defs"))[0].clone();
    let mut registry = Registry::new();
    let mut command = Command {
        definition: definition.clone(),
        body: 42,
    };
    for limits in [
        Limits {
            commands: 0,
            ..Limits::default()
        },
        Limits {
            cleanups: 0,
            ..Limits::default()
        },
        Limits {
            units: 0,
            ..Limits::default()
        },
        Limits {
            name_units: 1,
            ..Limits::default()
        },
    ] {
        assert!(matches!(
            Registry::new().register(
                &"probe".encode_utf16().collect::<Vec<_>>(),
                std::slice::from_ref(&command),
                limits
            ),
            Err(RegistryError::Limit)
        ));
    }
    let h = registry
        .register(
            &"probe".encode_utf16().collect::<Vec<_>>(),
            std::slice::from_ref(&command),
            Limits::default(),
        )
        .unwrap();
    command.definition = Value::Null;
    assert_eq!(
        get(&list(&registry.list().unwrap())[0], "title"),
        get(&definition, "title")
    );
    let name = "probe.read".encode_utf16().collect::<Vec<_>>();
    let mut value = obj(vec![("flag", str_("true"))]);
    let accepted = registry
        .admit(&name, &mut value, Limits::default())
        .unwrap();
    assert_eq!(accepted.body, 42);
    assert_eq!(value, accepted.input);
    registry.cleanup(h).unwrap();
    registry.cleanup(h).unwrap();
    assert!(matches!(
        registry.admit(&name, &mut value, Limits::default()),
        Err(RegistryError::Command(_))
    ));
}

#[test]
#[ignore = "live original registry operation comparison requires Node 22 and pinned TypeScript dependency"]
fn pinned_registry_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/registry-diff")
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
        "contract: engine.session.registry.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: full results/errors/caller mutation/body target/catalog/headless exact UTF16 values\nseeds: 1,7,99,1039\n",
    );
    for (label, program, args) in [
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["--version"]),
    ] {
        let r = Command::new(program)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(r.status.success());
        identity.push_str(&format!(
            "{label}: {}\n",
            String::from_utf8_lossy(&r.stdout).trim()
        ));
    }
    std::fs::write(run.join("identity.txt"), identity).unwrap();
    let prepared = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-session-registry.mjs"))
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(run.join("oracle-build.stdout.txt"), &prepared.stdout).unwrap();
    std::fs::write(run.join("oracle-build.stderr.txt"), &prepared.stderr).unwrap();
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let cases = cases();
    assert_eq!(cases.len(), 210);
    let mut input = Vec::new();
    for c in &cases {
        n(&mut input, c.input.len());
        input.extend(c.input);
    }
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/registry.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Read stdout while writing: large outputs cannot deadlock the pipe.
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        stdin.write_all(&input).unwrap();
    });
    let original = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    assert!(original.stderr.is_empty());
    let mut offset = 0;
    for c in cases {
        let candidate = payload(c.input);
        let len =
            u32::from_le_bytes(original.stdout[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let observed = &original.stdout[offset..offset + len];
        offset += len;
        let d = run.join(c.id);
        std::fs::create_dir_all(&d).unwrap();
        for (name, b) in [
            ("input.bin", c.input),
            ("original.bin", observed),
            ("rust.bin", candidate.as_slice()),
            ("original.stderr.txt", original.stderr.as_slice()),
        ] {
            std::fs::write(d.join(name), b).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert!(
            original.stderr.is_empty(),
            "{}: unexpected source diagnostics",
            c.id
        );
        assert_eq!(
            candidate, observed,
            "{}: complete registration/catalog/admission state",
            c.id
        );
        if c.id == "unknown-input" {
            let title: Vec<_> = "Probe 🦀"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect();
            let mut metadata = candidate.clone();
            let at = metadata
                .windows(title.len())
                .position(|b| b == title)
                .unwrap();
            metadata[at] ^= 1;
            assert_ne!(metadata, observed, "changed metadata must fail");
            let key: Vec<_> = "flag".encode_utf16().flat_map(u16::to_le_bytes).collect();
            let mut coerced = candidate.clone();
            let at = coerced
                .windows(key.len() + 1)
                .position(|b| b[..key.len()] == key && b[key.len()] == 3)
                .unwrap()
                + key.len();
            coerced[at] = 2;
            assert_ne!(coerced, observed, "changed coerced boolean must fail");
            let mut changed = candidate.clone();
            let needle: Vec<_> = "There is no command"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect();
            let at = changed
                .windows(needle.len())
                .position(|b| b == needle)
                .unwrap();
            changed[at] ^= 1;
            assert_ne!(
                changed, observed,
                "a deliberately changed error line must fail"
            );
            assert_ne!(
                &candidate[..candidate.len() - 1],
                observed,
                "missing operation-state data must fail"
            );
        }
        println!("{}: all original operation state compared exactly", c.id);
    }
    assert_eq!(offset, original.stdout.len());
    println!(
        "210 complete registry-operation scenarios; raw {}",
        run.display()
    );
}
