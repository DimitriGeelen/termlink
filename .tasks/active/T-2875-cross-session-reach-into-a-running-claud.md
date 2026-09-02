---
id: T-2875
name: "Cross-session reach into a running Claude session: is ListAgents/SendMessage
  already the answer, and where does TermLink actually fit"
description: >
  Inception: Cross-session reach into a running Claude session: is ListAgents/SendMessage
  already the answer, and where does TermLink actually fit

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-01T11:56:19Z
last_update: 2026-09-02T06:16:35Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-09-01T11:57:37Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2875: Cross-session reach into a running Claude session: is ListAgents/SendMessage already the answer, and where does TermLink actually fit

## Problem Statement

Can a running Claude Code TUI session on this host be reached from outside it, and
if so by what mechanism — and where does TermLink genuinely fit rather than
duplicate what already works?

Raised because a peer agent on .122 reported the reach as **structurally
impossible** and recommended building a TermLink-based path to close it. If that
report is right, a real capability gap exists and TermLink is chartered to close
it. If it is wrong, building anything is breadth accretion (T-2483) against a rail
that already runs.

For: any agent or operator trying to hand work to a session they are not sitting in.

## Assumptions

Four assumptions were carried into the exploration, all now tested. They are
tracked as IW-1..IW-4 under **Open Questions** below rather than duplicated here,
because each one's disposition and confidence is what matters and the gate reads
them there.

- **A1** — `ListAgents` is account-wide (tested → **false**, same-machine-only, IW-1).
- **A2** — `SendMessage` to an idle session merely queues until a human types
  (tested → **false**, it delivers and wakes the target, IW-2).
- **A3** — `termlink register --shell` would expose the caller's conversation
  (tested → **false**, it starts a sibling PTY, IW-3).
- **A4** — a cross-host path still needs building (tested → **false**, one already
  works across three hosts; the gap is enrolment, IW-4).

Every assumption carried in was disproved by measurement. That is the single most
useful fact this exploration produced.

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Is `ListAgents` scope same-machine-only, or account-wide?**
  confidence: 3
  disposition: answered
  rationale: Same-machine-only here. Disproof by absence: TermLink's cross-host rail shows `ring20-concierge` (.122) and `ring20-dashboard-agent` (.121) LIVE, and **neither appears** in this session's 14-row `ListAgents`, which lists only `interactive`/`bg`/`shell` kinds with no Remote-Control label. So the peer on .122 genuinely cannot see .107 sessions — its claim is correct *for its vantage point* and wrong as a general statement.

- **IW-2: Does `SendMessage` to an idle *interactive* session actually deliver into its conversation, or only queue until a human types?**
  confidence: 3
  disposition: answered
  rationale: It delivers. Tested end-to-end on a session I spawned and owned (no live session touched): `claude --bg` target, `SendMessage` carrying sentinel `HARNESS-PROBE-T2875-9f3c1a`, then asserted independently of the sender's `success:true` — the sentinel appears in the target's own transcript JSONL twice, once as `queue-operation` and once as a **`user` turn**, and the target woke from `state: done` and acted on it. **The false negative is the finding:** the file it was told to write never appeared, because it blocked on a permission prompt (`waitingFor: "permission prompt"`) — from the sender's side that is byte-identical to non-delivery.

- **IW-3: Does `termlink register --shell` from inside a Claude session expose that session's *conversation*, or only a sibling shell?**
  confidence: 2
  disposition: answered
  rationale: Only a sibling shell. `register --help`: `--shell` = "Start a PTY-backed session (full bidirectional I/O)" — it *starts* a PTY, it does not attach to the caller's terminal; `--self` is explicitly "event-only endpoint (no PTY)". T-2873 measured what injection into such a session actually does: the bytes landed at a bash prompt. So the peer's recommended fix grants remote shell access on that host — a materially different and more dangerous thing than a chat channel — and would not have solved the stated problem.

- **IW-4: Given a human can act from any .107 session, is there value in a cross-host path at all?**
  confidence: 2
  disposition: answered
  rationale: The mechanism already exists and works cross-host — 3 LIVE listeners across 3 hubs prove it. The gap is **enrollment, not mechanism**: 14 Claude sessions are live on .107 and exactly 1 is on the TermLink rail. Building new transport would be breadth accretion (T-2483) against a rail that already runs; the real question is why 13 of 14 sessions never enroll.

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

**Measured 2026-09-01. Two rails exist, and they see different worlds.**

| | population | scope | enrolment |
|---|---|---|---|
| Claude Code `ListAgents` | **14** sessions | **.107 only** (measured) | automatic |
| TermLink `agent-presence` | **3** listeners | **3 hosts** (.107/.122/.121) | opt-in at launch |

The target, `dimitri-mint-dev-2c`, is in the first and absent from the second.

**What the peer got right** (verified independently here, not taken on trust):
`dev.tty.legacy_tiocsti = 0`, so tty keystroke injection is genuinely dead.
`claude --help` has no `send`/`message`/`peer` subcommand. `--remote-control` is a
**launch-time flag**, so its "you cannot retrofit this onto a running session" is
correct — and the same limit applies to TermLink's own doorbell (PL-237: "running
headless claudes cannot be retrofitted — arm at relaunch"). Three mechanisms, one
shared structural truth.

**What it got wrong.** It concluded that reaching that session is therefore
impossible from outside. It is not: the session is addressable from any .107
session right now, with no setup. The peer generalised a property of its own
vantage point (.122, no Remote Control) into a property of the system, and its
own toolset was the evidence it reasoned from — `ListAgents` was absent for it,
so it inferred the capability does not exist. This session has it.

**The reframe.** Nothing needs building for the stated problem. The mechanism for
the general problem also already exists and demonstrably works across three hosts.
What does not work is **enrolment**: 1 of 14 local sessions is on the rail. That is
an adoption question, and this repo already instruments it (`fleet-adoption-snapshot`
cron) — which is where a follow-up belongs, if the operator wants one.

**Out of scope, deliberately:** building any new transport. Two working rails
already cover this ground; a third would be the breadth accretion T-2483 exists
to prevent.

## Technical Constraints

- `dev.tty.legacy_tiocsti = 0` — tty keystroke injection is dead at the kernel
  level. Verified independently here, not taken from the peer's report.
- `claude --help` exposes no `send` / `message` / `peer` subcommand.
- `--remote-control` is a **launch-time** flag: it cannot be retrofitted onto a
  running session. TermLink's own doorbell shares this limit (PL-237 — "running
  headless claudes cannot be retrofitted; arm at relaunch"). Three independent
  mechanisms, one shared structural truth: **reach must be arranged at launch.**
- `ListAgents` / `SendMessage` are same-machine-only (IW-1), so they cannot span
  hosts no matter how they are driven.
- A sender-side `success: true` from `SendMessage` proves queueing, not receipt
  (IW-2) — any prover built on this rail must assert on the receiver.

## Scope Fence

**IN scope:** measuring what the two existing rails can each actually do, and
disposing of the peer's structural-impossibility claim with evidence.

**OUT of scope, deliberately:** building any new transport. Two working rails
already cover this ground — `ListAgents`/`SendMessage` same-machine, TermLink
`agent-presence` cross-host — and a third would be exactly the breadth accretion
the charter-drift guard (T-2483) exists to prevent.

**OUT of scope, referred onward:** the enrolment gap. It is the real finding and
it is an adoption question, not a mechanism question, so it belongs in its own
task rather than being absorbed here.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** NO-GO on building a new transport — with one follow-up filed.

**Rationale:**

The DEFER this task filed under is spent. It deferred on one unmeasured question —
*"whether `ListAgents` scope is same-machine-only"* — and that question is now
answered at confidence 3 (IW-1). With it answered, every branch the DEFER was
holding open has resolved, and none of them lead to building anything:

- **Same-machine reach already works.** `SendMessage` delivers into a running
  session's conversation and wakes it (IW-2, proven on the receiver's own
  transcript, not on the sender's `success:true`).
- **Cross-host reach already works.** TermLink's `agent-presence` rail carries
  three LIVE listeners across .107 / .122 / .121 today.
- **The peer's recommended fix was worse than nothing.** `register --shell`
  starts a sibling PTY; it does not attach to the caller's conversation (IW-3).
  It would have granted remote **shell** access on that host — materially more
  dangerous than a chat channel — and still not solved the stated problem.

So the peer's report is correct about its own vantage point and wrong as a general
claim: `ListAgents` was absent for it, and it inferred the capability does not
exist. It reasoned from the shape of its own toolset. That is worth naming, because
it is a failure mode any agent can repeat.

**The real gap is enrolment, not mechanism.** 14 Claude sessions are live on .107
and exactly **1** is on the TermLink rail. Nothing needs building for reach; what
does not happen is sessions joining the rail that already exists. That is an
adoption question, this repo already instruments it (`fleet-adoption-snapshot`),
and it is filed separately as **T-2879** rather than absorbed here — one
inception, one question.

**Evidence:**

- **IW-1, disproof by absence:** TermLink shows `ring20-concierge` (.122) and
  `ring20-dashboard-agent` (.121) LIVE; **neither** appears in this session's
  14-row `ListAgents`, which lists only `interactive`/`bg`/`shell` kinds. The .122
  peer genuinely cannot see .107 sessions.
- **IW-2, asserted on the receiver:** sentinel `HARNESS-PROBE-T2875-9f3c1a` sent to
  a session spawned and owned by this exploration (no live session touched) appears
  in the target's own transcript JSONL as a **`user` turn**, and the target woke
  from `state: done` and acted. The **false negative is the finding**: the file it
  was told to write never appeared, because it blocked on a permission prompt —
  from the sender's side, byte-identical to non-delivery.
- **T-2876 turned that into a standing prover.** `scripts/session-message-selftest.sh`
  asserts on the receiver's transcript and splits the four outcomes a sender cannot
  distinguish (`DELIVERED` / `BLOCKED` / `ENQUEUED` / `UNDELIVERED`). Live-proven
  2026-09-01. Collapsing BLOCKED into UNDELIVERED is precisely this task's
  misdiagnosis, now pinned by a mutant test.
- **Two premises died on first execution** (T-2876): the `waitingFor` field does not
  exist (0 of 56 agent records), and `state == "blocked"` is the ordinary RESTING
  state of a finished session (43 of 56) — so keying off it would have classified
  almost every broken rail as "merely stuck", inverting the bug.
- **IW-3, from `register --help`:** `--shell` = "Start a PTY-backed session (full
  bidirectional I/O)"; `--self` = "event-only endpoint (no PTY)". T-2873
  independently measured where injected bytes land: a bash prompt.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-09-01T11:57:37Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
