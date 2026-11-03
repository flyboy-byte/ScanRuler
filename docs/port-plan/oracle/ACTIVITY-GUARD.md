# Command guard identity and cleanup

2026-11-02. PR #70 ports configureCommands, commandGuard and FREE.
Configuration stores the supplied guard object without invoking it. Reading
returns that same object; two objects sharing a busy callback remain distinct.
Cleanup unconditionally restores the original FREE object, including when an old
cleanup follows newer configuration. Dropping cleanup does not invoke it.

Busy results remain raw null/empty/text and opaque exceptions; this unit does
not perform truthiness filtering or normalize an error. The slot borrow ends
before a returned guard's callback can reconfigure/reset the slot. Weak handles
permit reentrant host callbacks without guard/slot ownership cycles. The slot
owns only current/default guard metadata, with constant work per operation.
Callback output allocation/budget belongs to its host owner; ordinary Rc
allocation is not process-wide OOM isolation.

192 source-first scenarios compare3072 full states: guard object identity,
null/empty/UTF-16 returns, exception identity/message, callback order, reentrant
configure/reset, old/repeated cleanup and dropped cleanup. All values match.
The complete pinned activity module is compiled without rewritten bodies or
behavioral dependency stubs. Full graph/map/provenance, exact module source
proof, independent generator and complete paired raw artifacts are retained.
Ordinary Rust tests cover borrowed host state and non-Clone faults.

Owned stable guard callbacks exclude arbitrary JS busy-property getters,
replacement and prototype mutation. Session owners control lifetime. Activity
running/progress, plugin busy/watch/stop, full registry execution and actual
browser guard runtime remain required separate work. Native roster on this
stack:37 suites /3,923 identities. Run cargo xtask ci and cargo xtask oracle
from rust/. Wasm compile is required, not whole-session acceptance.
