---
id: T-2322
name: "Inception extend push-wake to dm rail for non-live-sender dm posts"
description: >
  Verify+design whether to extend the arc-004 push-waker to ring on direct dm posts absent a live-sender ring

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: ["push-transport", "reliable-comms", "inception"]
components: []
related_tasks: [T-2316, T-2318, T-2320, T-2303]
created: 2026-07-02T23:45:13Z
last_update: 2026-07-02T23:45:13Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2322: Inception extend push-wake to dm rail for non-live-sender dm posts

## Problem Statement

**CAPTURED / decision-ready — awaiting a human GO before exploration proceeds.**
This is a `horizon: later` inception scoped (not explored) during the T-2320/T-2321
arc-004 wrap-up. The GAP was verified but the go/no-go dialogue is deliberately
deferred to the human (inception decide is sovereignty-gated).

**Verified gap.** The shipped arc-004 push-waker (T-2316) rings the receiver on
`inbox.queued` frames, which the hub emits ONLY for `channel.post → inbox:<id>`
topics (`crates/termlink-hub/src/channel.rs:752`). A post to a `dm:<self>:<peer>`
topic is a *non-inbox* topic and does **not** fire `inbox.queued` — proven by the
hub's own negative test `channel_post_non_inbox_topic_does_not_fire`
(channel.rs:3038). Today a direct `dm:` post only wakes the receiver if the
SENDER performs the ring-1 inject (`scripts/agent-send.sh`). Therefore a
`dm:<self>:<peer>` post from a poster that does NOT ring — raw `termlink channel
post`, a cron/automation, a remote peer, or the MCP `channel_post` tool — leaves
the receiver un-woken until their next `/check-arc` poll. That is a reliable-comms
hole in push coverage: the push story covers inbox deposits but not direct dm
posts absent a live-sender ring.

**Why now / why it matters.** arc-004's headline is "a live agent receives a DM
the instant it is posted." That holds for the inbox-deposit path and the
live-sender path, but not the direct-dm-post-by-a-non-live-sender path — the exact
case cross-host automation and MCP-driven peers hit. Worth a go/no-go before the
next cross-host integration relies on it.

**The one inception question (one inception = one question).** Should we extend the
push-waker to also ring on direct `dm:<self>:*` posts — and if so, by what
live-topic mechanism? The non-trivial design sub-question is topic DISCOVERY:
`inbox.queued` is a single aggregator topic, but `dm:<self>:*` is a dynamic
per-peer topic set. Candidate mechanisms to weigh at inception time:
  - **A. Hub-side dm aggregator** — emit a `dm.queued` frame (mirror of
    `inbox.queued`) for `dm:` posts; waker subscribes one topic. Cleanest for the
    waker; requires a hub change.
  - **B. Client-side wildcard/prefix subscribe** — if/when the hub supports a
    `dm:<self>:*` prefix push subscription; waker subscribes one prefix.
  - **C. Client-side discovery loop** — waker periodically lists `dm:<self>:*`
    topics and (re)subscribes each with `--push`; no hub change, more client
    complexity + a discovery-cadence latency floor.
Trade-offs: A adds hub state/emit (like T-1637 did for inbox); B depends on a push
feature that may not exist; C is pure-client but reintroduces a poll floor on
topic discovery (partially defeating the point). Portability (D4) favours A or B.

**Scope guard.** This is INCEPTION — no build artifacts before
`fw inception decide T-2322 go`. On GO, decompose into build slices (hub emit /
waker subscribe / E2E) under fresh task IDs, mirroring the T-2316→T-2320 arc.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->
- Direct `dm:` posts by a non-live-sender do not wake the receiver until the next
  poll (VERIFIED via channel.rs:752 + the `channel_post_non_inbox_topic_does_not_fire`
  negative test — high confidence; the go/no-go still needs a live demand check:
  do real cross-host/MCP posters actually hit this path in practice?).

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

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

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

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
-->

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
