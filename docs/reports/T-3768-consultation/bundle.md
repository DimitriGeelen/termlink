# Decision under review: what a "waker-liveness" canary should flag on one host

## System, in brief
- TermLink is a cross-terminal messaging substrate. Several AI agent sessions (one per project) run on host ".107" and exchange durable messages through a local hub.
- A daily **canary** (scheduled check; empty output = healthy, any line = alarm) watches the old "push-wake" delivery route: each agent was expected to run a small **waker** process that rings its terminal (PTY) when mail arrives. The signal is `pty_session` on the agent's presence heartbeat.
- The canary has three firing classes:
  - (a) LIVE-no-waker: a live agent whose heartbeat has no `pty_session`.
  - (b) dead-waker: a waker state file whose process is dead (the waker died silently after being armed).
  - (c) rail-dark: zero live agents carry `pty_session` (opt-in flag; already switched off on this host).
- A newer delivery route (arc-011) is replacing push-wake. Mail reaches each agent through a per-project **sidecar** daemon and a prompt hook. The arc-011 "doorbell" stage (W2), which replaces the waker's ring, is still being built.

## The standing operator ruling (2026-10-17, "PD-234")
> B — stop expecting the legacy push-wake rail on this host until arc-011's doorbell (W2): `--expect-armed` dropped from the waker crontab, and agents not expected armed are named with a reason in the git-tracked `.context/checks/waker-not-expected-allowlist` (`penelope`, `aef-greenfield-test`), counted and reported but not firing; a dead waker on an armed agent and any new unarmed agent still fire. Revisit at W2; the rail-dark expectation retires at W4.

Current allowlist contents: the two names above.

## What is observed now (latest run, 2026-11-14)
- FIRING: 13 LIVE agents with no `pty_session` (class a), 0 dead wakers (class b).
- The 13 are every agent on the host: this project's own sessions, the framework project, and about ten other projects' agents. All of them now advertise presence through the arc-011 sidecar, and none uses push-wake.
- The canary has fired daily since about 2026-11-13. The escalation tooling auto-files a task for a canary that stays red, and refiles 24h after a close if it is still red.
- Other existing checks: a "notify-sidecar" canary checks that each *declared* agent's sidecar is heartbeating (can this host still hear?). A separate on-demand prover checks that a message sent over the TermLink rail lands in a receiving session's transcript. Neither checks waker processes.

## Operator rules that bind the choice
1. Nothing working is switched off until its replacement is proven in the field, and then only with explicit documented operator approval.
2. Reliability thresholds measure whether things work, not how often something is used.
3. An always-on alarm is treated as no detection at all (alert fatigue is a known failure class here).
4. Adding or removing a check must stay within a cap: one guard in, one out.

## Options on the table
- **A.** Make class (a) non-firing on this host until W2: still counted and reported, but no alarm. Classes (b) and (c) unchanged. One config line in the canary, citing the ruling; it lapses at W2.
- **B.** Add all 13 current agents to the allowlist by name, each with a reason. New agents keep firing until someone adds them.
- **C.** Change nothing: the canary stays red.
- **D.** Rebuild class (a) to test the new route instead (does each live agent have a working sidecar receiver?). This is a build task, and it partly overlaps the notify-sidecar canary and the prover.
