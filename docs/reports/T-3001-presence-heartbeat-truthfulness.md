# T-3001 — Presence-heartbeat truthfulness

**Task:** T-3001 (inception, arc-009 value-review, owner: agent)
**Status:** exploration in progress — no decision recorded, no behaviour changed
**Created:** 2026-09-20, before research (C-001)

## Why this artifact exists

C-001: the thinking trail IS the artifact. Conversations are ephemeral; this file is
permanent. It is created before the research runs and filled incrementally.

## The question

`agent-presence` heartbeats are read as "this agent is reachable". T-2876 established,
on the message rail, that a sender-side success signal is not evidence of anything on
the receiver — and built a four-verdict vocabulary that tells the cases apart:

| verdict | evidence | meaning |
|---|---|---|
| DELIVERED | sentinel is a `user` turn in the receiver's transcript | the rail works |
| BLOCKED | sentinel queued, target at rest | delivery fine; target never drained |
| ENQUEUED | sentinel queued, target still busy | may yet drain |
| UNDELIVERED | sentinel absent | the rail is broken |

This inception asks whether presence has the same defect and whether that vocabulary
transfers to it.

## Open questions (mirrors the task file)

- **IW-1** What does a heartbeat assert today — producer fields vs consumer inferences?
- **IW-2** Is LIVE-but-cannot-act reachable, and currently invisible?
- **IW-3** Does the T-2876 verdict set transfer, or does presence need its own axis?
- **IW-4** Producer-side or consumer-side enforcement — and vendored (G-062) vs local?

## Findings

_(filled as spikes run)_

### F1 — Producer: what a heartbeat emits

_pending — spike 1_

### F2 — Consumers: who reads presence and what they conclude

_pending — spike 2_

### F3 — Is LIVE-but-blocked reachable and invisible?

_pending — spike 3_

### F4 — Vocabulary fit

_pending — spike 4_

### F5 — Ownership: vendored vs local

_pending — spike 5_

## Recommendation

_Not yet written. The `## Recommendation` section of the task arrived PRE-FILLED with
"GO" before any exploration ran — the same defect T-2989 caught and corrected. It is
treated here as a hypothesis to test, not a conclusion, and will be rewritten from the
measured findings._

## Decision

Not mine to make. `### Human [REVIEW]` owns the go/no-go; producer-not-judge forbids me
ratifying my own exploration.
