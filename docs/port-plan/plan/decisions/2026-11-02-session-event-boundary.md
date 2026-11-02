<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Session ownership and bounded event boundary — 2026-11-02

Owner handed the Rust session implementation from the old Claude specification
claim to Codex after #42–#44 merged at4ff736f. `engine.session.contract` stays
specified until its complete section8 traces pass. This is not acceptance of a
new epoch policy or a fake CLI workflow.

DESIGN: implement the documented dependency-free event-driven design. One session
owns the registry, business-state sections, document/history epochs and queues.
An admitted call names an opaque registered body and owns coerced arguments.
Future scheduling receives deterministic clock/body/progress events and emits
start/answer/activity effects. Adapters do transport/timing, not policy. No second
business-state store, foreign runtime, async dependency, automatic retry or
rollback on timeout.

First bounded child `engine.session.registry` implements complete registration,
listing/headless lookup and the start-admission prefix. Its result is explicitly
`Admitted`, not command execution. Reuse merged Value/schema and CommandError.
Registered definitions and snapshots own their data; executable callbacks use a
caller-supplied body key, not dummy application commands. Subsequent session PRs
must integrate scheduling/history and actual family bodies before claiming M3.

DOCUMENTED: original registration is not atomic. A failing later item leaves
previous items registered and provides no cleanup closure. Cleanup closures are
repeatable name lists: an old closure removes a newer registration under the
same name. Admission validates synchronously before queueing; nullish inputs use
an empty object; non-nullish input keeps coercions even when refused or thrown.
All are complete original observations in the corpus, not improvements.

DESIGN: explicit caller limits bound live definitions, retained repeatable cleanup
handles, aggregate owned units and name lengths, separately from schema8000/12.
Host failures remain distinct. Definitions must be ordinary owned typed data;
shared/cyclic definition references, getters and live mutation aliases are not
represented. Full locale/platform acceptance remains open: ASCII valid-name
sorting uses the observed Node22 ICU alphabet order, compared on all140 names
and authored/seeded punctuation/digit/prefix cases. No external collation crate.

ORACLE: Vite builds the full pinned registry module and real history/activity/
schema/store dependencies from a fresh git archive in /tmp, with only compiled artifacts under target. No reached
behavioral stub. Fixture command bodies only echo input and identify their
opaque callback; they are not implementations of the catalog's140 features.
Every original result/error/input mutation/target/catalog/headless state is
compared. The immutable initial209-case capture predates Rust comparison;
extended observables,210th refused-coercion regression and140 appended true
admission refusals preserve all prior
fields/inputs. Raw source captures are compressed losslessly, with original raw
SHA256/length retained. No result enters the other engine.
