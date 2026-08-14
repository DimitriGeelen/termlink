---
id: T-2719
name: "agent inbox unread count mixes retention-relative index with absolute cursor"
description: >
  termlink agent inbox subtracts an absolute cursor from a retention-window-relative
  latest index, over-reporting unread by 19x on a topic past its retention cap;
  agent unread reports the true count from absolute offsets
status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-14T21:20:00Z
last_update: 2026-08-14T21:20:00Z
date_finished: null
---

# T-2719: agent inbox unread count mixes retention-relative index with absolute cursor

## Context

Reported by the 999-AEF peer session (uds:/tmp/cc-socks/124667.sock) on
2026-08-14, homed here because the fix lives in this repo. Reproduced on
consecutive invocations with no posts in between.

**Two verbs disagree about the same topic in the same second:**

```
$ termlink agent inbox  | grep agent-chat-arc
  agent-chat-arc — 389 unread (latest=2000, cursor=1611)

$ termlink agent unread | grep agent-chat-arc
Topic 'agent-chat-arc': 20 unread for d1993c2c3ec44c94 (first new offset 11838, last 11857, last receipt up_to=11836)
```

389 vs 20 — a 19× disagreement, with nothing indicating which to trust.

**Diagnosis (peer's, and it is self-consistent from the numbers alone).**
`termlink agent info` reports:

```
Topic: agent-chat-arc
Retention: messages:2000
Posts: 2001
```

`latest=2000` in the inbox line is a **retention-window-relative** index — it is
the window size, not an absolute offset. The cursor `1611` is then subtracted from
it as though both were in the same coordinate space: `2000 - 1611 = 389`, exactly
the printed figure. `agent unread` uses **absolute** offsets (11838 / 11857 /
11836) and arrives at 20, the true count.

The arithmetic reproducing the printed number exactly is strong evidence on its
own — this does not need a live hub to be credible, though it does need one to be
confirmed (see the prediction below, which is the actual test).

**Falsifiable prediction to run BEFORE patching** (peer's, and the right shape —
it can disprove the diagnosis rather than merely illustrate it):

> `inbox`'s number should be wrong only on topics whose post count has **exceeded**
> the retention cap, and should agree with `unread` on any topic still under it.

If that holds, the fix is one coordinate conversion. If it does not hold, the
diagnosis is wrong and the cause is elsewhere. **Run this first** — patching on an
unconfirmed read is how a coordinate bug gets moved rather than fixed.

**Why this matters more than the arithmetic.** The peer hit it while diagnosing a
cross-session coordination failure: another agent believed it had sent ten
unanswered messages and asked whether the peer's read cursor had wedged. That
question could not be answered by inspection, because the two verbs that exist to
answer exactly it disagreed 19×.

`inbox` is the summary view — the one an agent reaches for **first** — and it is
the one that is wrong. It over-reports, so it fails toward alarm rather than
silence. That is the better failure direction, but a permanently-alarming unread
count is functionally equivalent to no count at all, because you stop reading it.
This is the same class as the guard-layer findings from this session's audit
(T-2680, T-2709): a signal whose value does not correspond to the state it claims
to describe, which trains its reader to ignore it.

The underlying coordination failure turned out to be neither party's cursor — the
sending peer was writing into a DM topic the recipient's fingerprint is not a party
to. **So this bug did not cause that incident**; it made it undiagnosable for
longer than it should have been. Worth stating plainly so the fix is not
over-credited.

**Environment:** hub PID 3080, runtime `/var/lib/termlink`, local hub only,
identity `d1993c2c3ec44c94`, topic `agent-chat-arc` (Senders 3, Posts 2001,
Retention messages:2000).

**Status of verification at filing:** NOT yet reproduced in this repo. This session
hit the context-budget gate, which blocks Bash, so neither the live verbs nor the
source could be inspected. Everything above is the peer's evidence plus arithmetic
that checks out on its face. First action next session is the prediction test, then
locating the subtraction site.

## Acceptance Criteria

### Agent
- [ ] The prediction is tested first: `inbox` and `unread` agree on a topic UNDER its retention cap, and disagree on one OVER it — recorded with both outputs
- [ ] If the prediction fails, the task is re-scoped to the real cause rather than patched to match the peer's read
- [ ] The subtraction site is located and cited by `file:line`, showing which operand is retention-relative and which is absolute
- [ ] The fix converts coordinates rather than special-casing the over-cap topic
- [ ] A regression test covers a topic whose post count exceeds its retention cap — the only configuration in which the bug is visible
- [ ] `agent inbox` and `agent unread` are shown agreeing on the reporter's exact case (agent-chat-arc, Posts 2001, Retention messages:2000)
- [ ] The peer (999-AEF) gets the outcome, including the result of the prediction test

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Fill in once the fix lands — the regression test is the load-bearing entry.
cargo test --workspace

## RCA

**Symptom:** `agent inbox` reports 389 unread where `agent unread` reports 20, on
the same topic in the same second, with no posts in between.

**Root cause (proposed, unconfirmed):** the unread count is computed as
`latest - cursor` where `latest` is an index into the retention window and
`cursor` is an absolute offset. The two are only equal while a topic has never
exceeded its retention cap, which is why this went unnoticed — every topic under
its cap reports correctly.

**Why structurally allowed:** the repo has retention-relative and absolute offsets
as distinct concepts but not as distinct *types*, so a subtraction across the two
is well-formed and silently wrong. Nothing tests a topic past its retention cap
against a second verb computing the same quantity a different way — and the
divergence is invisible until a topic crosses the cap, which on a low-traffic
topic can be months after the code ships.

**Prevention (to confirm during the fix):** the durable guard is a test asserting
`inbox` and `unread` agree for a topic seeded past its retention cap. The stronger
form is a newtype separating the two coordinate spaces so the subtraction cannot
compile — worth assessing once the site is located, since if there is one such
subtraction there are likely others.

## Separate findings — NEED THEIR OWN TASKS

The peer reported two usability items alongside the bug. Recorded here so they are
not lost, but they are **distinct deliverables** and compounding them into this
task would violate the one-bug-one-task rule. File as their own tasks next session:

1. **`termlink agent recent <topic>` resolves its positional as a session.**
   Passing a DM topic yields `Session '<topic>' not found`, which reads as "that
   topic does not exist" rather than "wrong argument kind". Cost the reporter a
   hypothesis. Same class as T-2663 (a refusal that does not name what happened) —
   the message is not wrong, it is answering a different question than the one the
   operator asked.

2. **No verb maps a fingerprint to a name.** `agent who` and `agent who-is` both
   reject a fingerprint argument. The reporter ended up identifying a peer by
   reading a DM and recognising the sign-off. Needs confirming that no such verb
   exists before building one.

## Updates

### 2026-08-14T21:20:00Z — filed from peer report
- **Source:** 999-AEF session, cross-session message
- **Action:** filed without local reproduction; context-budget gate blocked Bash
- **Next:** run the falsifiable prediction before any patching
