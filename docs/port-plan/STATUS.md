# Current Rust port status

Generated from canonical unit files and ci/native-suites.json; do not hand-edit.

Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

Native coverage inventory: **23 suites / 1191 case identities including refusals**. This counts declared cases, not successful measurements or feature completion.

No aggregate percentage establishes parity. Unit states describe evidence scope; hosted run outcomes are in [AUDIT-QUEUE.md](AUDIT-QUEUE.md). Bounded [browser-Wasm coverage](oracle/BROWSER-WASM.md) is separate from engine/session/UI integration and the still-open exact-base numerical ratchet.

| State | Units |
| --- | ---: |
| baseline-captured | 2 |
| implemented | 28 |
| planned | 47 |
| specified | 1 |

| Unit | State | Tested implementation | PR |
| --- | --- | --- | --- |
| [P00](plan/units/P00.json) | planned | — | — |
| [P01](plan/units/P01.json) | planned | — | — |
| [P02](plan/units/P02.json) | planned | — | — |
| [P03](plan/units/P03.json) | planned | — | — |
| [P04](plan/units/P04.json) | planned | — | — |
| [P05](plan/units/P05.json) | planned | — | — |
| [P06](plan/units/P06.json) | planned | — | — |
| [P07](plan/units/P07.json) | planned | — | — |
| [P08](plan/units/P08.json) | planned | — | — |
| [P09](plan/units/P09.json) | planned | — | — |
| [P10](plan/units/P10.json) | planned | — | — |
| [P11](plan/units/P11.json) | planned | — | — |
| [P12](plan/units/P12.json) | planned | — | — |
| [P13](plan/units/P13.json) | planned | — | — |
| [P14](plan/units/P14.json) | planned | — | — |
| [P15](plan/units/P15.json) | planned | — | — |
| [P16](plan/units/P16.json) | planned | — | — |
| [P17](plan/units/P17.json) | planned | — | — |
| [P18](plan/units/P18.json) | planned | — | — |
| [P19](plan/units/P19.json) | planned | — | — |
| [P20](plan/units/P20.json) | planned | — | — |
| [P21](plan/units/P21.json) | planned | — | — |
| [P22](plan/units/P22.json) | planned | — | — |
| [P23](plan/units/P23.json) | planned | — | — |
| [P24](plan/units/P24.json) | planned | — | — |
| [P25](plan/units/P25.json) | planned | — | — |
| [P26](plan/units/P26.json) | planned | — | — |
| [P27](plan/units/P27.json) | planned | — | — |
| [P28](plan/units/P28.json) | planned | — | — |
| [P29](plan/units/P29.json) | planned | — | — |
| [P30](plan/units/P30.json) | planned | — | — |
| [P31](plan/units/P31.json) | planned | — | — |
| [P32](plan/units/P32.json) | planned | — | — |
| [P33](plan/units/P33.json) | planned | — | — |
| [P34](plan/units/P34.json) | planned | — | — |
| [P35](plan/units/P35.json) | planned | — | — |
| [P36](plan/units/P36.json) | planned | — | — |
| [P37](plan/units/P37.json) | planned | — | — |
| [P38](plan/units/P38.json) | planned | — | — |
| [P39](plan/units/P39.json) | planned | — | — |
| [baseline.reproducibility](plan/units/baseline.reproducibility.json) | implemented | `0a42dca` | [#26](https://github.com/flyboy-byte/ScanRuler/pull/26) |
| [catalog.browser](plan/units/catalog.browser.json) | baseline-captured | `8f327ee` | [#14](https://github.com/flyboy-byte/ScanRuler/pull/14) |
| [catalog.headless](plan/units/catalog.headless.json) | baseline-captured | `8b9084f` | [#13](https://github.com/flyboy-byte/ScanRuler/pull/13) |
| [ci.integration](plan/units/ci.integration.json) | implemented | `f43ffae` | [#22](https://github.com/flyboy-byte/ScanRuler/pull/22) |
| [engine.error.envelope](plan/units/engine.error.envelope.json) | implemented | `f509ae2` | [#30](https://github.com/flyboy-byte/ScanRuler/pull/30) |
| [engine.measurement.slice](plan/units/engine.measurement.slice.json) | planned | — | — |
| [engine.session.contract](plan/units/engine.session.contract.json) | planned | — | — |
| [fit.curvature.gate](plan/units/fit.curvature.gate.json) | implemented | `f46eeb1` | [#16](https://github.com/flyboy-byte/ScanRuler/pull/16) |
| [fit.linalg.solve](plan/units/fit.linalg.solve.json) | implemented | `46e0992` | [#7](https://github.com/flyboy-byte/ScanRuler/pull/7) |
| [fit.plane.clip](plan/units/fit.plane.clip.json) | implemented | `4c8b4cf` | [#9](https://github.com/flyboy-byte/ScanRuler/pull/9) |
| [fit.plane.ransac](plan/units/fit.plane.ransac.json) | implemented | `2490952` | [#12](https://github.com/flyboy-byte/ScanRuler/pull/12) |
| [fit.plane.selection](plan/units/fit.plane.selection.json) | implemented | `e5c342c` | [#23](https://github.com/flyboy-byte/ScanRuler/pull/23) |
| [fit.plane.tls](plan/units/fit.plane.tls.json) | implemented | `4c8b4cf` | [#8](https://github.com/flyboy-byte/ScanRuler/pull/8) |
| [geometry.adjacency](plan/units/geometry.adjacency.json) | implemented | `74dd926` | [#17](https://github.com/flyboy-byte/ScanRuler/pull/17) |
| [geometry.bounds](plan/units/geometry.bounds.json) | implemented | `4c8b4cf` | [#10](https://github.com/flyboy-byte/ScanRuler/pull/10) |
| [geometry.centroid](plan/units/geometry.centroid.json) | implemented | `1473fe4` | [#11](https://github.com/flyboy-byte/ScanRuler/pull/11) |
| [geometry.closed.shells](plan/units/geometry.closed.shells.json) | implemented | `fd11f98` | [#21](https://github.com/flyboy-byte/ScanRuler/pull/21) |
| [geometry.components](plan/units/geometry.components.json) | implemented | `66997f8` | [#18](https://github.com/flyboy-byte/ScanRuler/pull/18) |
| [geometry.mesh.graph](plan/units/geometry.mesh.graph.json) | implemented | `6cfda5b` | [#20](https://github.com/flyboy-byte/ScanRuler/pull/20) |
| [geometry.normals.compute](plan/units/geometry.normals.compute.json) | implemented | `4a5aac4` | [#5](https://github.com/flyboy-byte/ScanRuler/pull/5) |
| [geometry.normals.orient](plan/units/geometry.normals.orient.json) | implemented | `4c8b4cf` | [#6](https://github.com/flyboy-byte/ScanRuler/pull/6) |
| [geometry.selection](plan/units/geometry.selection.json) | implemented | `3f56d3e` | [#19](https://github.com/flyboy-byte/ScanRuler/pull/19) |
| [harness.oracle.gate](plan/units/harness.oracle.gate.json) | implemented | `eeb2136` | [#15](https://github.com/flyboy-byte/ScanRuler/pull/15) |
| [harness.wasm.obj](plan/units/harness.wasm.obj.json) | specified | — | [#34](https://github.com/flyboy-byte/ScanRuler/pull/34) |
| [harness.wasm.ply](plan/units/harness.wasm.ply.json) | implemented | `8bc7363` | [#29](https://github.com/flyboy-byte/ScanRuler/pull/29) |
| [harness.wasm.runtime](plan/units/harness.wasm.runtime.json) | implemented | `00fd1a6` | [#24](https://github.com/flyboy-byte/ScanRuler/pull/24) |
| [io.mesh.weld](plan/units/io.mesh.weld.json) | implemented | `53d6408` | [#4](https://github.com/flyboy-byte/ScanRuler/pull/4) |
| [io.obj.companions](plan/units/io.obj.companions.json) | planned | — | — |
| [io.obj](plan/units/io.obj.json) | planned | — | — |
| [io.obj.mesh](plan/units/io.obj.mesh.json) | implemented | `58aeacb` | [#32](https://github.com/flyboy-byte/ScanRuler/pull/32) |
| [io.obj.texture](plan/units/io.obj.texture.json) | implemented | `219059c` | [#31](https://github.com/flyboy-byte/ScanRuler/pull/31) |
| [io.ply.ascii](plan/units/io.ply.ascii.json) | implemented | `737d3c3` | [#27](https://github.com/flyboy-byte/ScanRuler/pull/27) |
| [io.ply.binary](plan/units/io.ply.binary.json) | implemented | `98d39fc` | [#28](https://github.com/flyboy-byte/ScanRuler/pull/28) |
| [io.ply](plan/units/io.ply.json) | planned | — | — |
| [io.stl.binary](plan/units/io.stl.binary.json) | implemented | `5ee3881` | [#2](https://github.com/flyboy-byte/ScanRuler/pull/2) |
| [io.stl.dispatch](plan/units/io.stl.dispatch.json) | implemented | `77683bf` | [#3](https://github.com/flyboy-byte/ScanRuler/pull/3) |
| [io.threemf](plan/units/io.threemf.json) | planned | — | — |
| [project.measurement.compatibility](plan/units/project.measurement.compatibility.json) | planned | — | — |
