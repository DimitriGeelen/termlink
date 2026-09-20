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

### F1 — Producer: a heartbeat asserts one thing, and it is not reachability

`scripts/listener-heartbeat.sh` emits ten metadata fields: `agent_id`, `role`,
`listen_topics`, `started_at`, `interval_secs`, `host`, and conditionally
`pty_session`, `capabilities`, `addr`, `cv_key`.

**Every one of them is bound before the loop starts.** `started_at`, `host` and
`listen_csv` are computed at lines 150-157, above `post_one()`; the rest come straight
from argv or the environment. `post_one` interpolates the same variables on every
iteration. Nothing is re-evaluated per beat.

So a heartbeat is a launch-time declaration, replayed. The single fact it establishes
is *this bash loop is still iterating*. `pty_session` — which T-2387 treats as "the
waker-running signal" — is a string captured at arm time and repeated verbatim
forever; the loop never re-checks that the PTY exists or that the waker still runs.
(T-2405 exists precisely because a waker can be alive on stale code, and T-2387 class
(b) because its pid can be dead. Both are detected *outside* the heartbeat, by
canaries, because the heartbeat itself cannot see it.)

### F2 — Consumers: LIVE is a recency test on that replay

`scripts/agent-listeners.sh:13-16` defines the classification every consumer inherits:

    age = now - last_seen_ts
    age <= 2*interval        => LIVE
    2*interval < age <= 5*x  => STALE
    age > 5*interval         => OFFLINE

`interval` is the agent's own declared `interval_secs`, so each agent sets the window
it is judged by. More importantly the predicate is *purely* recency. Combined with F1,
**LIVE means exactly: a bash loop iterated within the last two intervals.**

Consumers then read it as dispatchability — `/peers` ("who's around to DM?"),
`agent find-idle` (the DISPATCH primitive: `LIVE(agent-presence) \ DISTINCT(claimer)`),
and doorbell discovery via `cv_index`. Twenty-five files under `scripts/` and eight
under `crates/` read this topic.

### F3 — The gap is structural, not accidental: the beater is not the agent

`scripts/be-reachable.sh:253-264` spawns the heartbeat as
`nohup setsid "$LH_SCRIPT" ... & disown`, with the comment *"so it survives this shell
exit"*. The loop is deliberately its own session leader, independent of the Claude
session it represents. CLAUDE.md says the same thing from the operator's side: stop it
explicitly, "the heartbeat is detached via `nohup setsid` so it will otherwise outlive
the session".

**So presence measures a different process from the one that would act on a message.**
The agent can be blocked at a prompt, finished, crashed, or waiting on a tool
approval, and presence still reads LIVE for as long as the detached loop keeps beating.

Nothing closes this. The `pid` in `~/.termlink/be-reachable-<id>.state` is `$!` from
that detached spawn — the heartbeat loop's pid — and `pid_alive()` (line 135) validates
the beater. No local state, no hub field, and no canary records whether the agent
behind the heartbeat can act:

- **T-2239 frozen-husk** — live process, frozen heartbeat. The converse case.
- **T-2387 waker-liveness** — LIVE without `pty_session`, or a dead waker pid. Covers
  "nothing can ring the PTY", not "the PTY rang and nobody came".
- **T-2405 stale-waker-code** — alive waker on old code.

All three ask about the *delivery apparatus*. None asks whether the recipient is in a
state to act, which is precisely T-2876's BLOCKED.

### F4 — Vocabulary fit: the discriminating half transfers exactly, the other half does not

T-2876's verdicts are properties of a *specific message's fate*, asserted on the
receiver. Presence asks a different question — "can this agent act on a message I have
not sent yet?" — so the set does not transfer wholesale. It splits cleanly:

| T-2876 verdict | presence analogue | status today |
|---|---|---|
| UNDELIVERED | heartbeat absent | **already visible** — OFFLINE |
| DELIVERED | none (message-scoped) | n/a |
| BLOCKED | beating, agent at rest and not draining | **invisible — collapsed into LIVE** |
| ENQUEUED | beating, agent busy | **invisible — collapsed into LIVE** |

The two verdicts that exist *specifically to separate "reachable" from "will act"* map
onto the two states presence cannot see. That is the finding: LIVE is not merely
imprecise, it is the union of the exact pair T-2876 was built to tell apart.

What transfers with full force is T-2876's **principle** — assert on the receiver,
never the sender; emit only what the emitter can observe. Under that rule the current
field is misnamed: the beater can observe that it is beating, so the honest claim is
BEATING. Whether the agent can act is observable only by the agent.

### F5 — Ownership: local to fix, vendored to propagate

Every surface a fix would touch is **local**: `scripts/listener-heartbeat.sh`,
`scripts/be-reachable.sh`, `scripts/agent-listeners.sh`, and the hub-side readers under
`crates/` (`agent.rs`, `cv_index.rs`, `fleet_presence.rs`). G-062 does not block the fix.

But the vendored framework carries **template copies** of the same scripts at
`.agentic-framework/lib/templates/scripts/{listener-heartbeat,be-reachable,agent-listeners,agent-listeners-fleet,agent-send}.sh`,
and these seed every project bootstrapped from the framework. The template
`listener-heartbeat.sh` has the identical shape — `started_at` computed once at line
123, emitted at line 137 — so it carries the same defect.

A purely local fix therefore leaves the class alive in every other project and
re-imports it here on the next bootstrap. The template half is upstream's (G-062) and
must be filed, not patched.

## Disposition summary

- **IW-1** answered (3) — ten fields, all launch-time-bound; LIVE = recency on a replay. F1/F2.
- **IW-2** answered (3) — yes, and by construction: `nohup setsid` makes the beater a
  different process from the agent; nothing validates the agent. F3.
- **IW-3** answered (2) — partial transfer, measured states + reasoned mapping: BLOCKED
  and ENQUEUED map exactly onto what LIVE collapses; DELIVERED/UNDELIVERED are
  message-scoped. The principle transfers fully. F4.
- **IW-4** answered (3) — fix is local; the same defect is replicated in vendored
  templates that seed other projects, so the template half is an upstream filing. F5.

## Recommendation

**GO** — but the pre-filled rationale was wrong about what the work is.

The section arrived pre-filled with "GO — presence is the remaining surface that
asserts delivery it cannot see". Presence does not assert delivery at all; delivery is
message-scoped and presence is prior to it. What presence asserts is *dispatchability*,
and the measured defect is sharper than the pre-filled text: LIVE is the union of
T-2876's BLOCKED and ENQUEUED, which is exactly the distinction that vocabulary exists
to draw.

**Scope, dependency-ordered:**

1. **Rename the claim before changing it.** LIVE -> BEATING at the producer/classifier
   boundary is truthful under F1 with no new signal required. Cheapest correct step.
2. **Decide who observes agent-side readiness.** The beater structurally cannot (F3).
   Either the agent emits its own state, or consumers stop inferring dispatchability
   from presence. This is the real design question and it is *not* settled by this
   exploration.
3. **File the template half upstream** (F5) — otherwise the fix does not travel.
4. **Only then** touch `find-idle`, whose anti-join currently treats LIVE as
   dispatchable.

**Do not start (1) before (2) is decided.** Renaming the field while consumers still
infer dispatchability from it relocates the untruth instead of removing it.

## Decision

Not mine to make. `### Human [REVIEW]` owns the go/no-go; producer-not-judge forbids me
ratifying my own exploration. No decision is recorded on this task.
