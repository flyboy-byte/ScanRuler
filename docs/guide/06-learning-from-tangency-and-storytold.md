<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 6. Learning from Tangency and Storytold

Two real, public Rust CAD projects have solved problems very close to ours. We study how they
work and adopt the patterns that fit. We do not import their code, their infrastructure or their
licence terms. Neither project has endorsed this one, and the plan assumes no acceptance or
relationship.

What is described below was read from their repositories and from this workspace's notes on
contributing to them (2026-10-31). It is a snapshot and their processes change.

## Tangency Labs: `solidcore` (and SolidCraft)

**What it is.** `solidcore` is a Rust port of NVIDIA's SMLib, a boundary-representation solid
modelling kernel (exact solids: shells, faces, edges, NURBS curves and surfaces, booleans,
fillets, offsets). It is written in safe Rust, aims for no panics on bad input, no global state,
builds for WebAssembly, and numerical agreement with the C++ original to 1e-9 relative with
identical topology. SolidCraft is the CAD application that uses it. It is the nearest relative
of what we are doing: a faithful port of a large existing program, checked unit by unit.

**What it does well, and what we borrowed**

| Pattern | How they do it | How we use it |
|---------|----------------|---------------|
| Differential testing | A `difftest` crate runs the Rust and the C++ reference on the same inputs, with seeds | The paired-runner protocol (page 4); the original TypeScript plays the part of the C++ |
| One record per unit | `plan/units/<id>.toml`, 261 units, status plus notes | `plan/units/<id>.json`; the status is in one place only |
| Generated progress | `PROGRESS.md` and a live site rebuilt on every merge; never edited by hand | `backlog.json` is generated; a progress site is a good later addition |
| Findings and upstream bugs as files | One file each, never edited later; updates are addenda | `plan/findings/` and `plan/decisions/` |
| "Correct means as upstream" | Match the C++ by default, bugs included; a fix goes behind an opt-in switch that is off by default so compared results do not move | Our reference policy: the original is authoritative; improvements are separate decisions |
| Honest partial status | Cases that reach a stubbed call are counted as *deferred* and tagged with the owning unit, kept apart from passes | "Deferred" is its own outcome state; it can never become a pass |
| Known mismatches recorded | Each has its own file and a findings entry | Same: a reproducible divergence gets a fixture and an open item |
| Claims | An empty claim commit plus a draft pull request; the lowest PR number wins | A scoped draft PR naming unit, symbols and files |
| Exact-base ratchet | Compare against reports for the exact base commit | The ratchet in page 4 |
| Pinned toolchain | A `rust-toolchain.toml` pins the compiler so new lints cannot break CI unannounced | CI may pin; locally we use the system toolchain (decision `no-rustup`) |
| Mutation testing | Deterministic seed prefixes; survivors minimised into tests | Page 4, "Testing the tests" |

**What we do not copy.** Their numerical tolerances (ours come from ScanRuler's own tests), their
infrastructure and runner assumptions, their timing numbers, and their private plan documents.
Their licence situation (Apache-2.0 upstream) differs from ours (AGPL).

**Something useful for later.** `solidcore` is a candidate pure-Rust exact kernel for our G1
gate. The plan asks for evidence, not assumption: for every operation we need, record whether it
is supported and verified, partial, missing, numerically wrong, too slow or unknown. Missing
pieces become scoped tasks, and useful gaps can be described precisely to that project, if and
when the owner chooses to, following its own rules. There must be no silent fallback to a
non-Rust kernel in anything we call complete.

## Storytold: `cadcraft`

**What it is.** A pure-Rust, AutoCAD-class 2D/3D CAD application, built in layers, with an egui
user interface and a WebAssembly build.

**Habits worth studying**

- **One gate for people and CI.** `cargo xtask ci` is the same command locally and in CI: format,
  lints with warnings as errors, tests, asset and layer checks, and the WebAssembly build. Ours is
  built the same way, with the plan checks included.
- **Every user-visible change is a command** in the engine, with tests. The UI is thin. This is
  also how ScanRuler's own agent interface works, so we keep it.
- **No `unwrap`, `expect` or `panic` outside tests.** Our workspace sets these as lint warnings
  (denied in the gate).
- **Input-derived numbers are hostile.** Cap sizes, use checked arithmetic. A file can lie about
  how big it is.
- **Never break WebAssembly.** The gate builds for it every time.
- **A shared CI workflow exists**, but it is suite-specific. The plan's advice is to adopt the
  pattern and not depend on their workflow file.

**A difference to remember.** `cadcraft` states a clean-room rule (never read GPL CAD sources).
That is a policy for that project. This port is source-informed and AGPL, so we do not claim
clean-room, and we do not paste code from either project.

## How the three fit together

```text
 ScanRuler (TypeScript, AGPL)  ──oracle──▶  this port (pure Rust, AGPL)
                                               │
                  patterns (not code)          │ exact-kernel candidate,
        solidcore · cadcraft  ───────────────▶ │ to be assessed per operation (G1)
```

## Practical reading list

1. In `solidcore`: `README.md`, `plan/STATUS.md`, one `plan/units/*.toml`, and the docs on
   completion evidence and mutation testing.
2. In `cadcraft`: its `CLAUDE.md` and the gate definition behind `cargo xtask ci`.
3. In this repo: `docs/port-plan/PORTING-PROCESS.md` ("Practices adapted, not inherited as
   commands") says exactly where we agree and diverge.
