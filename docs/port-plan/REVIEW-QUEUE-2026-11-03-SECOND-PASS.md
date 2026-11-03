# Second-pass port review queue

Observed 2026-11-03 17:03:41 UTC. Main: `6f3dc45f614007c4537ec55e9597e32b1000883e`.
This finite snapshot covers 21 open PRs and excludes its own handoff PR #84. Refresh live heads/checks before merging. No merge or acceptance is authorized by this table.

The [earlier dated handoff](REVIEW-HANDOFF-2026-11-03.md) remains historical. This snapshot retains every required check identity; missing, pending, skipped or red checks cannot be displayed as green. Raw data: [review snapshot](evidence/2026-11-03-pr-review-refresh.json).

| PR | Dependencies | Exact head | Draft | Observed required checks | Scope |
| --- | --- | --- | --- | --- | --- |
| [#58](https://github.com/flyboy-byte/ScanRuler/pull/58) | main | `cc5e49ac861c6958239bc57a425ae3062bbf8688` | no | red: source-tests,test-and-build | io.threemf: port 3MF import (zip container, units, textures) |
| [#60](https://github.com/flyboy-byte/ScanRuler/pull/60) | main | `a68e1b7e93612fe261fbbcd157ee7028782c83a0` | no | 11 green | engine.session.stale.detect: port stale-import classification |
| [#61](https://github.com/flyboy-byte/ScanRuler/pull/61) | #60 | `9ab46c699bdc4f1f14f7914194983a72c339ec2d` | no | red: source-tests,test-and-build | engine.session.stale.clock: port first-stale timestamp and reset |
| [#63](https://github.com/flyboy-byte/ScanRuler/pull/63) | #61 | `71898cc8804703ef8d4c23de2d5135647a59e487` | no | 11 green | engine.session.stale.error: port stale-page envelope construction |
| [#64](https://github.com/flyboy-byte/ScanRuler/pull/64) | main | `62e3d48180b5611ff8b67c66471f71ec5560f494` | no | pending/missing: source-tests,test-and-build | test: derive history trace seed labels from actual fixtures |
| [#65](https://github.com/flyboy-byte/ScanRuler/pull/65) | #63 | `038cdff575504880932ea328540f741feb3c8e5c` | no | 11 green | engine.session.error.normalize: port command fault classification |
| [#68](https://github.com/flyboy-byte/ScanRuler/pull/68) | #65 | `0b59d60d9e2e160752eaf06c25302bce3598e710` | no | 11 green | engine.session.clock.utc: port pure-Rust UTC timestamp conversion |
| [#70](https://github.com/flyboy-byte/ScanRuler/pull/70) | main | `ba8577d5c4299e35360956587829034ec2031bae` | no | pending/missing: browser-wasm,gate,source-tests,test-and-build | engine.session.activity.guard: port guard configuration and reset |
| [#71](https://github.com/flyboy-byte/ScanRuler/pull/71) | #70 | `3597a32831e746db3172ea1f9c8617e46ad6160b` | no | 11 green | engine.session.activity.stores: port watched-store source identity |
| [#72](https://github.com/flyboy-byte/ScanRuler/pull/72) | #71 | `f6b6892cb867726ba02a25081f632d73f2888762` | no | 11 green | engine.session.activity.watch: port subscription snapshot and teardown |
| [#73](https://github.com/flyboy-byte/ScanRuler/pull/73) | #72 | `b0f14c207ad0134475440082eeb7ec1022ec6b51` | no | 11 green | docs: record port review order and next execution boundaries |
| [#74](https://github.com/flyboy-byte/ScanRuler/pull/74) | #68 | `048cda28c838842e360c9f75253d203e12baea41` | no | 11 green | test: compare a complete negative Gregorian period against Date |
| [#75](https://github.com/flyboy-byte/ScanRuler/pull/75) | #64, #69 (merged), #73, #74 | `60bf04b91d8ee0e304dcb4c22e27e91b9c1b63b8` | no | 11 green | integration: validate the complete pending Rust port stacks |
| [#76](https://github.com/flyboy-byte/ScanRuler/pull/76) | #75 | `20986f5bc821d0dc467c05f42a33a01705de0f0e` | no | 11 green | harness.verification.levels: distinguish implemented from verified |
| [#77](https://github.com/flyboy-byte/ScanRuler/pull/77) | #76 | `a9e36b7537d29b95478f3215f2331ea32d3e2791` | no | 11 green | engine.session.activity.plugins: port plugin busy registration and lookup |
| [#78](https://github.com/flyboy-byte/ScanRuler/pull/78) | #77 | `15dbda0175d15f069cd16f1c0a14b62347c6543a` | no | 11 green | engine.session.activity.stop: port live plugin stopping and waits |
| [#79](https://github.com/flyboy-byte/ScanRuler/pull/79) | #78 | `52554dae9e92304c0904b256e6456f8ed528b919` | no | 11 green | engine.session.scene.queue: port direct routing and viewport serialization |
| [#80](https://github.com/flyboy-byte/ScanRuler/pull/80) | #79 | `72279dd21ce3f3ba7dea455a270ce590c3437323` | no | 11 green | engine.session.execute.lifecycle: port preflight, wait ordering and done lifetime |
| [#81](https://github.com/flyboy-byte/ScanRuler/pull/81) | #80 | `eb18227d18fc6617200e98efcfa04df4546c9841` | no | 11 green | engine.session.recent.result.data: port retained JSON data and size limits |
| [#82](https://github.com/flyboy-byte/ScanRuler/pull/82) | #81 | `93161108d1b59fe6928f4fb035df837936bf9877` | no | 11 green | engine.session.recent.store: port record, completion and shallow readouts |
| [#83](https://github.com/flyboy-byte/ScanRuler/pull/83) | #82 | `ccd5a4b89c0ac527483349cc5c34289728cae82e` | yes | pending/missing: source-tests,source-browser,test-and-build | engine.state.workspaces: port workspace mapping and installed names |

Merged integration dependencies are retained with exact identities and checked ancestry in the observed main:

- [#69](https://github.com/flyboy-byte/ScanRuler/pull/69): `506f29e27654d514bebe5592f098859cb6253b75`, merged 2026-11-03T16:48:49Z.
