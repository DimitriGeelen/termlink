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
last_update: 2026-09-01T11:57:37Z
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

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

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
  confidence: 1
  disposition: deferred
  rationale: Being listed is not the same as being deliverable — the T-2873 lesson exactly (a hub reporting "injected" was not proof the PTY received it). Settling this means messaging a real session someone is using, which is the operator's call, not mine. Deferred pending that consent.

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

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

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

**Recommendation:** DEFER

**Rationale:**

A peer agent on .122 reported that reaching a running Claude TUI session on .107 is structurally impossible from outside, citing four verified facts (no claude send subcommand, cc-daemon control.sock is internal, a spawned claude -p lacks ListAgents/SendMessage, TIOCSTI disabled). Three of four check out here. But ListAgents run from a .107 session lists dimitri-mint-dev-2c and 107 dimitri-mint-dev as addressable peers right now, so the capability exists and the peer generalised its own vantage point into a structural claim. DEFER rather than NO-GO because the decisive question is unmeasured: whether ListAgents scope is same-machine-only, which would make the peer correct for cross-host .122 to .107 and leave a real gap that TermLink, not Remote Control, is chartered to close.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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
