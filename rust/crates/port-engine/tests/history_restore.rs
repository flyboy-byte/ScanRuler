// SPDX-License-Identifier: AGPL-3.0-only
//! Full original global control-flow traces at the stable typed participant boundary.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::history_record::restore::{self, Hook, RestoreFuture, RestoreHost, View};
use port_engine::history_record::{
    Cell, Compare, Entry, Guards, HistoryError, Patch, Recorder, Section, Snapshot,
};
use std::{
    future::Future,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
    task::{Context, Poll, Waker},
};
type Text = Vec<u16>;
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/history-restore/cases.tsv");
fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn hex(s: &[u16]) -> String {
    s.iter()
        .flat_map(|u| u.to_le_bytes())
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn string(s: &[u16]) -> String {
    String::from_utf16(s).unwrap()
}
fn ready<T>(future: impl Future<Output = T>) -> T {
    let mut cx = Context::from_waker(Waker::noop());
    let mut f = std::pin::pin!(future);
    match f.as_mut().poll(&mut cx) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("fixture future unexpectedly pending"),
    }
}
struct Document {
    pool: Vec<Cell>,
    alpha: Cell,
    beta: Cell,
    log: Vec<String>,
    workspace: Text,
    busy: bool,
    status: Text,
    error: Option<Text>,
    fault: String,
    enabled: bool,
    view: View,
}
impl Document {
    fn new() -> Self {
        let pool: Vec<_> = [0., 1., 2., 10., 11., 12.]
            .iter()
            .map(|&v| Cell::Object(Arc::new(vec![(text("v"), Cell::Number(v))])))
            .collect();
        Self {
            alpha: pool[0].clone(),
            beta: pool[3].clone(),
            pool,
            log: vec![],
            workspace: text("measure"),
            busy: false,
            status: text("initial"),
            error: None,
            fault: String::new(),
            enabled: false,
            view: View {
                restoring: false,
                past: 0,
                future: 0,
                revision: 0,
                importing: false,
            },
        }
    }
    fn cell(&self, c: &Cell) -> String {
        if matches!(c, Cell::Null) {
            return "null".into();
        }
        let id = self
            .pool
            .iter()
            .position(|p| p.strict_equal(c))
            .expect("unknown reconstructed cell");
        let Cell::Object(o) = c else { panic!("object") };
        let Cell::Number(v) = o[0].1 else {
            panic!("number")
        };
        format!("{id}/{v}")
    }
    fn patch(&self, p: &Patch) -> String {
        p.iter()
            .map(|(k, v)| format!("{}={}", string(k), self.cell(v)))
            .collect::<Vec<_>>()
            .join(",")
    }
    fn snapshot(&self, s: &Snapshot) -> String {
        s.sections
            .iter()
            .map(|s| format!("{}={}", string(&s.key), self.cell(&s.value)))
            .collect::<Vec<_>>()
            .join(",")
    }
    fn view(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}/{}/{}",
            usize::from(self.view.restoring),
            self.view.past,
            self.view.future,
            self.view.revision,
            string(&self.workspace),
            usize::from(self.busy),
            usize::from(self.view.importing)
        )
    }
    fn event(&mut self, name: &str, p: Option<&Patch>, previous: Option<&Snapshot>, view: View) {
        self.view = view;
        self.log.push(format!(
            "{name}:{}:{},{}{}{}",
            self.view(),
            self.cell(&self.alpha),
            self.cell(&self.beta),
            p.map(|p| format!(":{}", self.patch(p))).unwrap_or_default(),
            previous
                .map(|p| format!(":{}", self.snapshot(p)))
                .unwrap_or_default()
        ));
    }
    fn fail(&self, name: &str) -> Result<(), Text> {
        if self.enabled && (self.fault == name || self.fault == format!("swallow-{name}")) {
            let mut e = text(&format!("fault:{name}"));
            e.push(0xd800);
            Err(e)
        } else {
            Ok(())
        }
    }
    fn entries(&self, entries: &[Entry]) -> String {
        entries
            .iter()
            .map(|e| {
                format!(
                    "{}~{}~{}~{}~{}",
                    self.patch(&e.before),
                    self.patch(&e.after),
                    hex(&e.label),
                    string(&e.workspace),
                    e.group
                )
            })
            .collect::<Vec<_>>()
            .join(";")
    }
}
impl RestoreHost for Document {
    type Error = Text;
    fn participants(&self) -> usize {
        2
    }
    fn capture(&mut self, view: View) -> Result<Snapshot, Text> {
        self.event("scope", None, None, view);
        self.fail("scope")?;
        self.event("scan", None, None, view);
        self.fail("scan")?;
        self.event("capture-alpha", None, None, view);
        self.fail("capture-alpha")?;
        let alpha = self.alpha.clone();
        self.event("capture-beta", None, None, view);
        self.fail("capture-beta")?;
        Ok(Snapshot {
            sections: vec![
                Section {
                    key: text("scope"),
                    value: Cell::Null,
                    compare: Compare::Identity,
                },
                Section {
                    key: text("scan"),
                    value: Cell::Null,
                    compare: Compare::Identity,
                },
                Section {
                    key: text("alpha"),
                    value: alpha,
                    compare: Compare::Fields,
                },
                Section {
                    key: text("beta"),
                    value: self.beta.clone(),
                    compare: Compare::Fields,
                },
            ],
        })
    }
    fn hook<'a>(
        &'a mut self,
        hook: Hook,
        patch: &'a Patch,
        previous: &'a Snapshot,
        view: View,
    ) -> RestoreFuture<'a, Result<(), Text>> {
        Box::pin(async move {
            let name = match hook {
                Hook::BeforeCore => "before-core",
                Hook::BeforeParticipant(0) => "before-alpha",
                Hook::BeforeParticipant(1) => "before-beta",
                Hook::AfterCore => "after-core",
                Hook::AfterParticipant(0) => "after-alpha",
                Hook::AfterParticipant(1) => "after-beta",
                _ => panic!("participant"),
            };
            self.event(
                name,
                Some(patch),
                if matches!(hook, Hook::AfterCore | Hook::AfterParticipant(_)) {
                    Some(previous)
                } else {
                    None
                },
                view,
            );
            if matches!(hook, Hook::BeforeCore) {
                self.busy = true
            }
            self.fail(name)?;
            if matches!(hook, Hook::AfterCore) {
                self.busy = false
            }
            Ok(())
        })
    }
    fn apply(&mut self, patch: &Patch, view: View) -> Result<(), Text> {
        for key in ["alpha", "beta"] {
            if let Some((_, value)) = patch.iter().find(|(k, _)| string(k) == key) {
                let name = format!("apply-{key}");
                self.event(&name, None, None, view);
                self.fail(&name)?;
                if key == "alpha" {
                    self.alpha = value.clone()
                } else {
                    self.beta = value.clone()
                }
            }
        }
        Ok(())
    }
    fn workspace(&mut self, w: &[u16]) -> Result<(), Text> {
        self.workspace = w.to_vec();
        Ok(())
    }
    fn status(&mut self, s: Text) -> Result<(), Text> {
        self.status = s;
        Ok(())
    }
    fn clear_busy(&mut self) {
        self.busy = false
    }
    fn error(&mut self, e: Text) {
        self.error = Some(e)
    }
}
struct Host(std::rc::Rc<std::cell::RefCell<Document>>);
impl RestoreHost for Host {
    type Error = Text;
    fn participants(&self) -> usize {
        2
    }
    fn capture(&mut self, v: View) -> Result<Snapshot, Text> {
        self.0.borrow_mut().capture(v)
    }
    fn run_restore<'a>(
        &'a mut self,
        view: View,
        job: impl FnOnce(&'a mut Self) -> RestoreFuture<'a, Result<(), Text>> + 'a,
    ) -> RestoreFuture<'a, Result<(), Text>> {
        let shared = self.0.clone();
        Box::pin(async move {
            {
                let mut d = shared.borrow_mut();
                d.event("run:start", None, None, view);
                d.fail("run-before")?;
                if d.enabled && d.fault == "skip" {
                    return Ok(());
                }
            }
            let swallow = {
                let d = shared.borrow();
                d.enabled && d.fault.starts_with("swallow-")
            };
            let result = job(self).await;
            if !swallow {
                result?;
            }
            let mut d = shared.borrow_mut();
            let view = d.view;
            d.event("run:end", None, None, view);
            d.fail("run-after")?;
            Ok(())
        })
    }
    fn hook<'a>(
        &'a mut self,
        hook: Hook,
        p: &'a Patch,
        s: &'a Snapshot,
        v: View,
    ) -> RestoreFuture<'a, Result<(), Text>> {
        let result = ready(self.0.borrow_mut().hook(hook, p, s, v));
        Box::pin(async move { result })
    }
    fn apply(&mut self, p: &Patch, v: View) -> Result<(), Text> {
        self.0.borrow_mut().apply(p, v)
    }
    fn workspace(&mut self, w: &[u16]) -> Result<(), Text> {
        self.0.borrow_mut().workspace(w)
    }
    fn status(&mut self, s: Text) -> Result<(), Text> {
        self.0.borrow_mut().status(s)
    }
    fn clear_busy(&mut self) {
        self.0.borrow_mut().clear_busy()
    }
    fn error(&mut self, e: Text) {
        self.0.borrow_mut().error(e)
    }
}
fn observe(line: &str) -> String {
    let parts: Vec<_> = line.split('\t').collect();
    let records: usize = parts[2].parse().unwrap();
    let shared = std::rc::Rc::new(std::cell::RefCell::new(Document::new()));
    let mut host = Host(shared.clone());
    let mut r = Recorder::default();
    {
        let mut h = shared.borrow_mut();
        h.workspace = text(parts[3]);
        for i in 1..=records {
            let mut label = text(&format!("change{i}"));
            label.push(0xd800);
            let workspace = h.workspace.clone();
            let token = r
                .begin_sync(&label, &workspace, Guards::default(), |r| {
                    h.capture(View::from(&r.state))
                })
                .unwrap()
                .unwrap();
            h.alpha = h.pool[i].clone();
            h.beta = h.pool[i + 3].clone();
            r.finish(token, |r| h.capture(View::from(&r.state)))
                .unwrap();
        }
    }
    if parts[1] == "redo" {
        ready(restore::travel(&mut r, &mut host, false)).unwrap();
    }
    {
        let mut h = shared.borrow_mut();
        h.workspace = text("measure");
        h.busy = false;
        h.status = text("initial");
        h.error = None;
        h.log.clear();
        h.fault = parts[4].to_string();
        h.enabled = true;
    }
    let error = match ready(restore::travel(&mut r, &mut host, parts[1] == "redo")) {
        Err(HistoryError::Capture(e)) => hex(&e),
        Ok(_) => "-".into(),
        Err(e) => panic!("{e:?}"),
    };
    let h = shared.borrow();
    format!(
        "{}\t{error}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        parts[0],
        h.cell(&h.alpha),
        h.cell(&h.beta),
        h.entries(&r.state.past),
        h.entries(&r.state.future),
        r.state.revision,
        usize::from(r.state.restoring),
        usize::from(r.state.importing),
        string(&h.workspace),
        usize::from(h.busy),
        hex(&h.status),
        h.error
            .as_ref()
            .map(|e| hex(e))
            .unwrap_or_else(|| "-".into()),
        h.log.join(";")
    )
}
#[test]
fn typed_restore_traces_are_bounded() {
    assert_eq!(INPUT.lines().count(), 216);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "full original global restore comparison requires Node22 and pinned git object"]
fn pinned_history_restore_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-history-restore.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/history-restore.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(INPUT.as_bytes())
        .unwrap();
    let original = child.wait_with_output().unwrap();
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    assert!(original.stderr.is_empty());
    let output = String::from_utf8(original.stdout).unwrap();
    let originals: Vec<_> = output.split_inclusive('\n').collect();
    assert_eq!(originals.len(), 216);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/history-restore-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    let candidate = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), &candidate.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&candidate.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    for (input, expected) in INPUT.lines().zip(originals) {
        let id = input.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        let actual = observe(input);
        for (name, data) in [
            ("input.bin", input.as_bytes()),
            ("original.bin", expected.as_bytes()),
            ("rust.bin", actual.as_bytes()),
            ("original.stderr.txt", b"".as_slice()),
        ] {
            std::fs::write(case.join(name), data).unwrap();
        }
        assert_eq!(actual, expected, "{id}");
    }
    println!(
        "216 complete global restore observations; {}",
        dir.display()
    );
}
