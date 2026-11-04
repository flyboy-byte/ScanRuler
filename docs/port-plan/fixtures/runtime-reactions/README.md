<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Retained queue/deadline reaction inputs

Hand-written complete Cartesian profile: three abort-listener policies (ignore,
resolve fallback, reject the identical reason), two late outcomes (success/error),
and two native observation-ticket lifetimes (retained/dropped). Twelve stable
case IDs; no random generator, private files or recorded source outputs are inputs.

Each fresh session queues a limited pending body, a synchronous failing command,
and a successful command. A controlled timer fires, later commands answer before
an ignored-abort body finishes, the body settles late, and a cleared timer is
fired again. Compare all five frames: every caller/body value or full UTF-16
error and reason identity, document effects, timer/abort state, ordered factories,
cleanup and replies, and absence of unhandled rejection. The original executes
the full pinned `limited` and `inScene` bodies, not a promise/queue substitute.

Rust uses `Owner`, `SceneQueue`, `Limited` and its owned caller future. Ticket
drop cannot cancel work. All earlier deadline/execute/scene inputs are retained.
`SCANRULER_REACTIONS_SABOTAGE=1` changes one resulting document value from 42 to
43; the real comparison must fail and retain the failed artifact directory.
Platform wall-clock timers and the complete guarded business-session executor
remain separate work; controlled time here is evidence for reaction ordering.
