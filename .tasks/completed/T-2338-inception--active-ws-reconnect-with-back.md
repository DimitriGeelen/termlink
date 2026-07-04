---
id: T-2338
name: "Inception — active WS reconnect-with-backoff for push consumer"
description: >
  arc-004 follow-on. Question (one, go/no-go): should v2 add an active WS reconnect-with-backoff loop that restores push when the hub returns? Explore: reconnect trigger, backoff policy, avoiding missed posts during the gap (durable offset replay on reconnect), interaction with the degrade-to-poll floor. See docs/operations/push-transport-recipe.md §4 + docs/reports/T-2303-push-transport-inception.md.

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-07-03T19:09:44Z
last_update: 2026-07-04T22:02:52Z
date_finished: 2026-07-04T22:02:52Z
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

After a hub blip, the arc-004 push consumer degrades to polling (by design — the durable
layer stays authoritative). This inception was filed to answer: does v2 need an **active
reconnect-with-backoff loop** so push is restored when the hub returns, instead of the
consumer staying on the poll floor indefinitely? For: any live agent relying on sub-second
push wake — a consumer stuck on the ~15s poll floor after a transient blip silently loses
the arc's headline latency win. Filed from `push-transport-recipe.md` §4, which at filing
time framed reconnect as an open v2 item.

## Assumptions

- A1 (implicit in the filing): "the shipped consumer has no reconnect loop" — **REFUTED by
  source verification.** T-2314 already built it; the raw CLI `--push` path and the
  push-waker share the same loop (`channel.rs:8568-8611`).
- A2: "if a loop exists, it covers all failure sequences" — **REFUTED, narrowly.** After 6
  consecutive sub-5s connect failures the consumer broke to steady poll permanently and
  never re-probed WS (`channel.rs:8597-8601`). That single gap was carved out to build task
  T-2340 (shipped 2026-07-03; E2E-proven by T-2341's reproducer).

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

Executed (source-verification, no spikes needed): (1) read the shipped `--push` consumer
loop in `channel.rs` end-to-end; (2) trace the push-waker's invocation path
(`be-reachable-pushwaker.sh` → `channel subscribe … --push`) to confirm both consumers
share one reconnect implementation; (3) enumerate the failure sequences the loop does NOT
cover. Result: IW-1 dissolved, IW-2 answered (see Open Questions); residual gap scoped to
T-2340 rather than expanding this inception.

## Technical Constraints

None new — the reconnect loop rides the existing WS transport constraints documented in
`docs/operations/push-transport-recipe.md` (backoff clamp, `WS_HEALTHY_SESSION_MS` = 5s
healthy-session reset, durable-cursor catch-up on reconnect so no posts are missed during
the gap).

## Scope Fence

**IN:** whether a v2 reconnect loop is needed (one go/no-go question). **OUT:** building
anything under this inception ID (the residual re-probe gap became T-2340, its E2E proof
T-2341); webhook transport (Candidate B, separate); receipts-through-WS (S4).

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
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

**Decision**: NO-GO

**Rationale**: Recommendation: NO-GO (superseded by T-2314)

Rationale (revised 2026-07-04 after code investigation): The proposed "v2 active
WS reconnect-with-backoff loop" was ALREADY BUILT AND SHIPPED by T-2314. The original
GO rationale below described the pre-T-2314 behavior; the gap it names no longer
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

Original GO rationale (now stale — retained for audit): ~~Real reliability gap: v1 push
consumer degrades to poll on socket loss and STAYS on poll until process restart …~~ — this
was the state T-2314 fixed.

Narrow residual carved out → T-2340: after `WS_RECONNECT_MAX_ATTEMPTS` = 6 consecutive
sub-5s reconnect failures (hub hard-down for a few seconds), the raw consumer `break`s to the
steady poll loop (`channel.rs:8597-8601`) and stays on poll until process restart — the poll
loop never re-probes WS. The waker path self-heals via its outer re-subscribe
(`be-reachable-pushwaker.sh`), but a long-lived raw `--push` invocation does not. This is a
small, scoped, bounded follow-up (periodic WS re-probe from steady poll), NOT the full rebuild
this inception assumed. Filed as build task T-2340. Human records the final no-go via
`fw inception decide T-2338 no-go`.

**Date**: 2026-07-04T22:02:52Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-03T22:32:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-07-04T22:02:52Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Recommendation: NO-GO (superseded by T-2314)

Rationale (revised 2026-07-04 after code investigation): The proposed "v2 active
WS reconnect-with-backoff loop" was ALREADY BUILT AND SHIPPED by T-2314. The original
GO rationale below described the pre-T-2314 behavior; the gap it names no longer
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

Original GO rationale (now stale — retained for audit): ~~Real reliability gap: v1 push
consumer degrades to poll on socket loss and STAYS on poll until process restart …~~ — this
was the state T-2314 fixed.

Narrow residual carved out → T-2340: after `WS_RECONNECT_MAX_ATTEMPTS` = 6 consecutive
sub-5s reconnect failures (hub hard-down for a few seconds), the raw consumer `break`s to the
steady poll loop (`channel.rs:8597-8601`) and stays on poll until process restart — the poll
loop never re-probes WS. The waker path self-heals via its outer re-subscribe
(`be-reachable-pushwaker.sh`), but a long-lived raw `--push` invocation does not. This is a
small, scoped, bounded follow-up (periodic WS re-probe from steady poll), NOT the full rebuild
this inception assumed. Filed as build task T-2340. Human records the final no-go via
`fw inception decide T-2338 no-go`.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-6aff3442
- **Timestamp:** 2026-07-04T22:02:53Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`
  2. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-2
     - evidence: `IW-2 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

### 2026-07-04T22:02:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
