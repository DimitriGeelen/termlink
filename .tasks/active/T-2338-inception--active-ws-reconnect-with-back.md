---
id: T-2338
name: "Inception — active WS reconnect-with-backoff for push consumer"
description: >
  arc-004 follow-on. Question (one, go/no-go): should v2 add an active WS reconnect-with-backoff loop that restores push when the hub returns? Explore: reconnect trigger, backoff policy, avoiding missed posts during the gap (durable offset replay on reconnect), interaction with the degrade-to-poll floor. See docs/operations/push-transport-recipe.md §4 + docs/reports/T-2303-push-transport-inception.md.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-03T19:09:44Z
last_update: 2026-07-03T22:32:54Z
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

# T-2338: Inception — active WS reconnect-with-backoff for push consumer

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

- **IW-1: Should v2 add an active WS reconnect-with-backoff loop for the push consumer?**
  confidence: 3
  disposition: dissolved
  rationale: Premise false — the loop already exists (T-2314), `channel.rs:8568-8611`. The
  raw CLI `--push` and the waker share it. No v2 rebuild needed.

- **IW-2: Does any real reconnect gap survive T-2314?**
  confidence: 3
  disposition: answered
  rationale: Yes, one narrow one — after 6 consecutive sub-5s failures the raw consumer
  breaks to steady poll permanently (`channel.rs:8597-8601`, never re-probes WS). Scoped
  to build task T-2340 (periodic WS re-probe from steady poll); not a rebuild.

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

**Recommendation:** NO-GO (superseded by T-2314)

**Rationale (revised 2026-07-04 after code investigation):** The proposed "v2 active
WS reconnect-with-backoff loop" was ALREADY BUILT AND SHIPPED by T-2314. The original
GO rationale below described the **pre-T-2314** behavior; the gap it names no longer
exists. Verified in current code:

- `crates/termlink-cli/src/commands/channel.rs:8568-8611` — inside `cmd_channel_subscribe`,
  the `if push` branch wraps `run_ws_push` in a reconnect loop that on each drop (1) drains
  the gap via `ws_poll_catchup` advancing the durable cursor, (2) backs off via
  `ws_reconnect_backoff`, (3) retries the WS. A healthy session (≥ `WS_HEALTHY_SESSION_MS`
  = 5s, line 458) resets the backoff so transient blips reconnect indefinitely.
- The raw `channel subscribe --push` CLI and the push-waker share THIS ONE path —
  `scripts/be-reachable-pushwaker.sh:100` invokes `channel subscribe … --push`, so there is
  no separate reconnect behavior to build.
- `docs/operations/push-transport-recipe.md` §4 already documents this as current behavior.
- Demo evidence: `docs/reports/T-2314-arc-004-active-reconnect-demo.md` (post-blip DM
  delivered over the resumed live WS; "no permanent degrade").

**Original GO rationale (now stale — retained for audit):** ~~Real reliability gap: v1 push
consumer degrades to poll on socket loss and STAYS on poll until process restart …~~ — this
was the state T-2314 fixed.

**Narrow residual carved out → T-2340:** after `WS_RECONNECT_MAX_ATTEMPTS` = 6 *consecutive
sub-5s* reconnect failures (hub hard-down for a few seconds), the raw consumer `break`s to the
steady poll loop (`channel.rs:8597-8601`) and stays on poll until process restart — the poll
loop never re-probes WS. The waker path self-heals via its outer re-subscribe
(`be-reachable-pushwaker.sh`), but a long-lived raw `--push` invocation does not. This is a
small, scoped, bounded follow-up (periodic WS re-probe from steady poll), NOT the full rebuild
this inception assumed. Filed as build task T-2340. Human records the final no-go via
`fw inception decide T-2338 no-go`.

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

### 2026-07-03T22:32:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
