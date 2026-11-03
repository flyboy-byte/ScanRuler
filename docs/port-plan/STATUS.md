# Current Rust port status

Generated from canonical unit files and ci/native-suites.json; do not hand-edit.

Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

Native coverage inventory: **51 suites / 9578 case identities including refusals**. This counts declared cases, not successful measurements or feature completion.

No aggregate percentage establishes parity. Unit states describe evidence scope; hosted run outcomes are in [AUDIT-QUEUE.md](AUDIT-QUEUE.md). Bounded [browser-Wasm coverage](oracle/BROWSER-WASM.md) is separate from engine/session/UI integration and the still-open exact-base numerical ratchet.

| State | Units |
| --- | ---: |
| baseline-captured | 2 |
| implemented | 65 |
| planned | 47 |
| specified | 2 |

Verification labels describe scoped evidence snapshots, not whole-feature/runtime acceptance.

| Verification | Units |
| --- | ---: |
| fixed-corpus | 48 |
| fuzzed | 4 |
| none | 64 |

| Unit | State | Verification | Tested implementation | Verification snapshot | PR |
| --- | --- | --- | --- | --- | --- |
| [P00](plan/units/P00.json) | planned | none | — | — | — |
| [P01](plan/units/P01.json) | planned | none | — | — | — |
| [P02](plan/units/P02.json) | planned | none | — | — | — |
| [P03](plan/units/P03.json) | planned | none | — | — | — |
| [P04](plan/units/P04.json) | planned | none | — | — | — |
| [P05](plan/units/P05.json) | planned | none | — | — | — |
| [P06](plan/units/P06.json) | planned | none | — | — | — |
| [P07](plan/units/P07.json) | planned | none | — | — | — |
| [P08](plan/units/P08.json) | planned | none | — | — | — |
| [P09](plan/units/P09.json) | planned | none | — | — | — |
| [P10](plan/units/P10.json) | planned | none | — | — | — |
| [P11](plan/units/P11.json) | planned | none | — | — | — |
| [P12](plan/units/P12.json) | planned | none | — | — | — |
| [P13](plan/units/P13.json) | planned | none | — | — | — |
| [P14](plan/units/P14.json) | planned | none | — | — | — |
| [P15](plan/units/P15.json) | planned | none | — | — | — |
| [P16](plan/units/P16.json) | planned | none | — | — | — |
| [P17](plan/units/P17.json) | planned | none | — | — | — |
| [P18](plan/units/P18.json) | planned | none | — | — | — |
| [P19](plan/units/P19.json) | planned | none | — | — | — |
| [P20](plan/units/P20.json) | planned | none | — | — | — |
| [P21](plan/units/P21.json) | planned | none | — | — | — |
| [P22](plan/units/P22.json) | planned | none | — | — | — |
| [P23](plan/units/P23.json) | planned | none | — | — | — |
| [P24](plan/units/P24.json) | planned | none | — | — | — |
| [P25](plan/units/P25.json) | planned | none | — | — | — |
| [P26](plan/units/P26.json) | planned | none | — | — | — |
| [P27](plan/units/P27.json) | planned | none | — | — | — |
| [P28](plan/units/P28.json) | planned | none | — | — | — |
| [P29](plan/units/P29.json) | planned | none | — | — | — |
| [P30](plan/units/P30.json) | planned | none | — | — | — |
| [P31](plan/units/P31.json) | planned | none | — | — | — |
| [P32](plan/units/P32.json) | planned | none | — | — | — |
| [P33](plan/units/P33.json) | planned | none | — | — | — |
| [P34](plan/units/P34.json) | planned | none | — | — | — |
| [P35](plan/units/P35.json) | planned | none | — | — | — |
| [P36](plan/units/P36.json) | planned | none | — | — | — |
| [P37](plan/units/P37.json) | planned | none | — | — | — |
| [P38](plan/units/P38.json) | planned | none | — | — | — |
| [P39](plan/units/P39.json) | planned | none | — | — | — |
| [baseline.reproducibility](plan/units/baseline.reproducibility.json) | implemented | none | `0a42dca` | — | [#26](https://github.com/flyboy-byte/ScanRuler/pull/26) |
| [catalog.browser](plan/units/catalog.browser.json) | baseline-captured | none | `8f327ee` | — | [#14](https://github.com/flyboy-byte/ScanRuler/pull/14) |
| [catalog.headless](plan/units/catalog.headless.json) | baseline-captured | none | `8b9084f` | — | [#13](https://github.com/flyboy-byte/ScanRuler/pull/13) |
| [ci.integration](plan/units/ci.integration.json) | implemented | none | `f43ffae` | — | [#22](https://github.com/flyboy-byte/ScanRuler/pull/22) |
| [engine.error.envelope](plan/units/engine.error.envelope.json) | implemented | fixed-corpus | `f509ae2` | `f509ae2` | [#30](https://github.com/flyboy-byte/ScanRuler/pull/30) |
| [engine.measurement.slice](plan/units/engine.measurement.slice.json) | planned | none | — | — | — |
| [engine.schema.validate](plan/units/engine.schema.validate.json) | implemented | fixed-corpus | `d3f9945` | `d3f9945` | [#42](https://github.com/flyboy-byte/ScanRuler/pull/42) |
| [engine.session.activity.guard](plan/units/engine.session.activity.guard.json) | implemented | fixed-corpus | `c458f7a` | `c458f7a` | [#70](https://github.com/flyboy-byte/ScanRuler/pull/70) |
| [engine.session.activity.plugins](plan/units/engine.session.activity.plugins.json) | implemented | fixed-corpus | `d353297` | `d353297` | [#77](https://github.com/flyboy-byte/ScanRuler/pull/77) |
| [engine.session.activity.state](plan/units/engine.session.activity.state.json) | specified | none | — | — | [#86](https://github.com/flyboy-byte/ScanRuler/pull/86) |
| [engine.session.activity.stop](plan/units/engine.session.activity.stop.json) | implemented | fixed-corpus | `5a7b036` | `5a7b036` | [#78](https://github.com/flyboy-byte/ScanRuler/pull/78) |
| [engine.session.activity.stores](plan/units/engine.session.activity.stores.json) | implemented | fixed-corpus | `97dcd7f` | `97dcd7f` | [#71](https://github.com/flyboy-byte/ScanRuler/pull/71) |
| [engine.session.activity.watch](plan/units/engine.session.activity.watch.json) | implemented | fixed-corpus | `7f2c709` | `7f2c709` | [#72](https://github.com/flyboy-byte/ScanRuler/pull/72) |
| [engine.session.clock.utc](plan/units/engine.session.clock.utc.json) | implemented | fixed-corpus | `cf0543a` | `cf0543a` | [#68](https://github.com/flyboy-byte/ScanRuler/pull/68) |
| [engine.session.command.deadline](plan/units/engine.session.command.deadline.json) | implemented | fixed-corpus | `b3566ac` | `b3566ac` | [#59](https://github.com/flyboy-byte/ScanRuler/pull/59) |
| [engine.session.contract](plan/units/engine.session.contract.json) | specified | none | — | — | [#45](https://github.com/flyboy-byte/ScanRuler/pull/45) |
| [engine.session.error.normalize](plan/units/engine.session.error.normalize.json) | implemented | fixed-corpus | `6dcf191` | `6dcf191` | [#65](https://github.com/flyboy-byte/ScanRuler/pull/65) |
| [engine.session.execute.lifecycle](plan/units/engine.session.execute.lifecycle.json) | implemented | fixed-corpus | `e6dce90` | `e6dce90` | [#80](https://github.com/flyboy-byte/ScanRuler/pull/80) |
| [engine.session.history.core](plan/units/engine.session.history.core.json) | implemented | fixed-corpus | `c2795e1` | `c2795e1` | [#55](https://github.com/flyboy-byte/ScanRuler/pull/55) |
| [engine.session.history.guards](plan/units/engine.session.history.guards.json) | implemented | fixed-corpus | `1504212` | `1504212` | [#53](https://github.com/flyboy-byte/ScanRuler/pull/53) |
| [engine.session.history.local](plan/units/engine.session.history.local.json) | implemented | fixed-corpus | `123e43e` | `123e43e` | [#52](https://github.com/flyboy-byte/ScanRuler/pull/52) |
| [engine.session.history.record](plan/units/engine.session.history.record.json) | implemented | fixed-corpus | `b5c2c40` | `b5c2c40` | [#46](https://github.com/flyboy-byte/ScanRuler/pull/46) |
| [engine.session.history.restore](plan/units/engine.session.history.restore.json) | implemented | fixed-corpus | `84e49dd` | `84e49dd` | [#54](https://github.com/flyboy-byte/ScanRuler/pull/54) |
| [engine.session.import.queue](plan/units/engine.session.import.queue.json) | implemented | fixed-corpus | `f239035` | `f239035` | [#57](https://github.com/flyboy-byte/ScanRuler/pull/57) |
| [engine.session.recent.result.data](plan/units/engine.session.recent.result.data.json) | implemented | fixed-corpus | `4fb6a53` | `4fb6a53` | [#81](https://github.com/flyboy-byte/ScanRuler/pull/81) |
| [engine.session.recent.store](plan/units/engine.session.recent.store.json) | implemented | fixed-corpus | `e61e7f8` | `e61e7f8` | [#82](https://github.com/flyboy-byte/ScanRuler/pull/82) |
| [engine.session.registry](plan/units/engine.session.registry.json) | implemented | fixed-corpus | `bdc24fe` | `bdc24fe` | [#45](https://github.com/flyboy-byte/ScanRuler/pull/45) |
| [engine.session.scene.queue](plan/units/engine.session.scene.queue.json) | implemented | fixed-corpus | `ac221ed` | `ac221ed` | [#79](https://github.com/flyboy-byte/ScanRuler/pull/79) |
| [engine.session.settle.registry](plan/units/engine.session.settle.registry.json) | implemented | fixed-corpus | `84e083e` | `84e083e` | [#67](https://github.com/flyboy-byte/ScanRuler/pull/67) |
| [engine.session.settle.run](plan/units/engine.session.settle.run.json) | implemented | fixed-corpus | `9691406` | `9691406` | [#66](https://github.com/flyboy-byte/ScanRuler/pull/66) |
| [engine.session.stale.clock](plan/units/engine.session.stale.clock.json) | implemented | fixed-corpus | `8a6d137` | `8a6d137` | [#61](https://github.com/flyboy-byte/ScanRuler/pull/61) |
| [engine.session.stale.detect](plan/units/engine.session.stale.detect.json) | implemented | fixed-corpus | `7cd55a7` | `7cd55a7` | [#60](https://github.com/flyboy-byte/ScanRuler/pull/60) |
| [engine.session.stale.error](plan/units/engine.session.stale.error.json) | implemented | fixed-corpus | `4ce6c9d` | `4ce6c9d` | [#63](https://github.com/flyboy-byte/ScanRuler/pull/63) |
| [engine.state.workspaces](plan/units/engine.state.workspaces.json) | implemented | fixed-corpus | `af44b28` | `af44b28` | [#83](https://github.com/flyboy-byte/ScanRuler/pull/83) |
| [fit.curvature.gate](plan/units/fit.curvature.gate.json) | implemented | fixed-corpus | `f46eeb1` | `f46eeb1` | [#16](https://github.com/flyboy-byte/ScanRuler/pull/16) |
| [fit.linalg.solve](plan/units/fit.linalg.solve.json) | implemented | fixed-corpus | `46e0992` | `46e0992` | [#7](https://github.com/flyboy-byte/ScanRuler/pull/7) |
| [fit.plane.clip](plan/units/fit.plane.clip.json) | implemented | fixed-corpus | `4c8b4cf` | `4c8b4cf` | [#9](https://github.com/flyboy-byte/ScanRuler/pull/9) |
| [fit.plane.ransac](plan/units/fit.plane.ransac.json) | implemented | fixed-corpus | `2490952` | `2490952` | [#12](https://github.com/flyboy-byte/ScanRuler/pull/12) |
| [fit.plane.selection](plan/units/fit.plane.selection.json) | implemented | fixed-corpus | `e5c342c` | `e5c342c` | [#23](https://github.com/flyboy-byte/ScanRuler/pull/23) |
| [fit.plane.tls](plan/units/fit.plane.tls.json) | implemented | fixed-corpus | `4c8b4cf` | `4c8b4cf` | [#8](https://github.com/flyboy-byte/ScanRuler/pull/8) |
| [geometry.adjacency](plan/units/geometry.adjacency.json) | implemented | fixed-corpus | `74dd926` | `74dd926` | [#17](https://github.com/flyboy-byte/ScanRuler/pull/17) |
| [geometry.bounds](plan/units/geometry.bounds.json) | implemented | fixed-corpus | `4c8b4cf` | `4c8b4cf` | [#10](https://github.com/flyboy-byte/ScanRuler/pull/10) |
| [geometry.centroid](plan/units/geometry.centroid.json) | implemented | fixed-corpus | `1473fe4` | `1473fe4` | [#11](https://github.com/flyboy-byte/ScanRuler/pull/11) |
| [geometry.closed.shells](plan/units/geometry.closed.shells.json) | implemented | fixed-corpus | `fd11f98` | `fd11f98` | [#21](https://github.com/flyboy-byte/ScanRuler/pull/21) |
| [geometry.components](plan/units/geometry.components.json) | implemented | fixed-corpus | `66997f8` | `66997f8` | [#18](https://github.com/flyboy-byte/ScanRuler/pull/18) |
| [geometry.mesh.graph](plan/units/geometry.mesh.graph.json) | implemented | fixed-corpus | `6cfda5b` | `6cfda5b` | [#20](https://github.com/flyboy-byte/ScanRuler/pull/20) |
| [geometry.normals.compute](plan/units/geometry.normals.compute.json) | implemented | fixed-corpus | `4a5aac4` | `4a5aac4` | [#5](https://github.com/flyboy-byte/ScanRuler/pull/5) |
| [geometry.normals.orient](plan/units/geometry.normals.orient.json) | implemented | fixed-corpus | `4c8b4cf` | `4c8b4cf` | [#6](https://github.com/flyboy-byte/ScanRuler/pull/6) |
| [geometry.selection](plan/units/geometry.selection.json) | implemented | fixed-corpus | `3f56d3e` | `3f56d3e` | [#19](https://github.com/flyboy-byte/ScanRuler/pull/19) |
| [harness.coverage.original](plan/units/harness.coverage.original.json) | implemented | none | `96c776c` | — | [#43](https://github.com/flyboy-byte/ScanRuler/pull/43) |
| [harness.coverage.rust](plan/units/harness.coverage.rust.json) | implemented | none | `be43ab8` | — | [#44](https://github.com/flyboy-byte/ScanRuler/pull/44) |
| [harness.fuzz.minimize](plan/units/harness.fuzz.minimize.json) | implemented | none | `32c45a9` | — | [#51](https://github.com/flyboy-byte/ScanRuler/pull/51) |
| [harness.fuzz.standing](plan/units/harness.fuzz.standing.json) | implemented | none | — | — | — |
| [harness.oracle.gate](plan/units/harness.oracle.gate.json) | implemented | none | `eeb2136` | — | [#15](https://github.com/flyboy-byte/ScanRuler/pull/15) |
| [harness.verification.levels](plan/units/harness.verification.levels.json) | implemented | fixed-corpus | `7402ddf` | `7402ddf` | [#76](https://github.com/flyboy-byte/ScanRuler/pull/76) |
| [harness.wasm.files](plan/units/harness.wasm.files.json) | implemented | none | `dbe8fcc` | — | [#37](https://github.com/flyboy-byte/ScanRuler/pull/37) |
| [harness.wasm.obj](plan/units/harness.wasm.obj.json) | implemented | none | `239012c` | — | [#34](https://github.com/flyboy-byte/ScanRuler/pull/34) |
| [harness.wasm.ply](plan/units/harness.wasm.ply.json) | implemented | none | `8bc7363` | — | [#29](https://github.com/flyboy-byte/ScanRuler/pull/29) |
| [harness.wasm.runtime](plan/units/harness.wasm.runtime.json) | implemented | none | `00fd1a6` | — | [#24](https://github.com/flyboy-byte/ScanRuler/pull/24) |
| [io.inflate](plan/units/io.inflate.json) | implemented | none | — | — | — |
| [io.mesh.weld](plan/units/io.mesh.weld.json) | implemented | fixed-corpus | `53d6408` | `53d6408` | [#4](https://github.com/flyboy-byte/ScanRuler/pull/4) |
| [io.obj.companions.drop](plan/units/io.obj.companions.drop.json) | implemented | fixed-corpus | `a4cb8de` | `a4cb8de` | [#40](https://github.com/flyboy-byte/ScanRuler/pull/40) |
| [io.obj.companions.files](plan/units/io.obj.companions.files.json) | implemented | fixed-corpus | `f0a3e9e` | `f0a3e9e` | [#36](https://github.com/flyboy-byte/ScanRuler/pull/36) |
| [io.obj.companions.image_audit](plan/units/io.obj.companions.image_audit.json) | implemented | none | `87d2318` | — | [#39](https://github.com/flyboy-byte/ScanRuler/pull/39) |
| [io.obj.companions.images](plan/units/io.obj.companions.images.json) | planned | none | — | — | — |
| [io.obj.companions](plan/units/io.obj.companions.json) | planned | none | — | — | — |
| [io.obj](plan/units/io.obj.json) | planned | none | — | — | — |
| [io.obj.mesh](plan/units/io.obj.mesh.json) | implemented | fixed-corpus | `58aeacb` | `58aeacb` | [#32](https://github.com/flyboy-byte/ScanRuler/pull/32) |
| [io.obj.texture](plan/units/io.obj.texture.json) | implemented | fixed-corpus | `219059c` | `219059c` | [#31](https://github.com/flyboy-byte/ScanRuler/pull/31) |
| [io.ply.ascii](plan/units/io.ply.ascii.json) | implemented | fuzzed | `737d3c3` | `60bf04b` | [#27](https://github.com/flyboy-byte/ScanRuler/pull/27) |
| [io.ply.binary](plan/units/io.ply.binary.json) | implemented | fuzzed | `98d39fc` | `60bf04b` | [#28](https://github.com/flyboy-byte/ScanRuler/pull/28) |
| [io.ply](plan/units/io.ply.json) | planned | none | — | — | — |
| [io.stl.binary](plan/units/io.stl.binary.json) | implemented | fuzzed | `5ee3881` | `60bf04b` | [#2](https://github.com/flyboy-byte/ScanRuler/pull/2) |
| [io.stl.dispatch](plan/units/io.stl.dispatch.json) | implemented | fuzzed | `77683bf` | `60bf04b` | [#3](https://github.com/flyboy-byte/ScanRuler/pull/3) |
| [io.threemf](plan/units/io.threemf.json) | planned | none | — | — | — |
| [project.measurement.compatibility](plan/units/project.measurement.compatibility.json) | planned | none | — | — | — |
