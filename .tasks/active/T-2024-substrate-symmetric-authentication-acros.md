---
id: T-2024
name: "Substrate: symmetric authentication across UDS + TCP transports"
description: >
  §6 primitive 6 (Supporting, hard-dep) + §7 transport unification. Same-host UDS
  today is auth-bypassed (UID trust); cross-host is HMAC+cert-pinning. Unify on one
  authenticated path (loopback TCP same-host, TCP cross-host, both HMAC + cert pinning)
  to retire the privileged sidecar UDS listener.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:37Z
last_update: 2026-06-08T11:20:55Z
date_finished:
revisit_at: 2026-09-08            # T-1451: DEFER pending measurement spike; revisit in 90d (gives Foundation primitives time to ship)
revisit_evidence_needed: "Latency-spike numbers under concurrent-agent load (≥10 simultaneous clients), or a concrete UID-trust incident, or operator decision to retire the privileged sidecar."
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-06-07T11:41:30Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal)
    rubric_sha: missing
---

# T-2024: Substrate: symmetric authentication across UDS + TCP transports

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Supporting. **§9 boundary:** hard-dep per §9. Also implements §7 transport unification..

**Role per ADR §6:** Same-host UDS today is auth-bypassed (UID trust); cross-host is HMAC + cert pinning. Two code paths, two trust models, and a long-lived privileged sidecar UDS listener. §7 decision: unify on one authenticated path — loopback TCP same-host, TCP cross-host, both HMAC + cert pinning.

**Why captured now:** Trust-model surface reduction is a precondition for opening up the substrate to more concurrent agents (T-2019..T-2021 increase the attack surface if UID-trust UDS stays).

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Loopback TCP latency vs UDS at homelab scale — confirmed negligible per §7, but measure under concurrent-agent load to be sure.**
  confidence: 1
  disposition: open
  rationale: UNRESOLVED — this is the GATE for the whole decision. §7 asserts the answer without measurement, and T-2019..T-2021 increase the concurrent-client load on the local hub well above prior measurement points. Same shape as T-1991 "agent-presence bloat" (asserted, then turned out to be wrong reason). Recommended action: spike before deciding. See docs/reports/T-2024-symmetric-auth-inception.md §3, §6 Track A.

- **IW-2: Cert pinning store location for loopback case — same KnownHubStore or separate?**
  confidence: 4
  disposition: resolved
  rationale: SAME KnownHubStore. Loopback hubs are just hubs at `127.0.0.1:port` — `KnownHubStore` is already keyed by address. No new store needed. See artifact §5.IW-2.

- **IW-3: Migration — how does an existing UDS-only deployment upgrade without downtime?**
  confidence: 3
  disposition: resolved
  rationale: STAGED COEXISTENCE — 4 phases. Phase-1: ship TCP path alongside UDS, no behavior change. Phase-2: clients opt into TCP via config flag, default still UDS. Phase-3: flip the default, UDS deprecation warning. Phase-4: remove UDS. Each phase = ≥1 release; shape is clear, per-phase task scoping at GO time. See artifact §4, §5.IW-3.

## Exploration Plan

At promotion time: (1) measure loopback TCP latency under concurrent-agent load; (2) design cert-pinning approach; (3) write migration recipe; (4) build + cutover.

## Technical Constraints

**Dependencies (upstream):** None

**Dependencies (downstream):** Retires the UID-trust UDS path; reduces trust-model surface to one path.

**ADR §9 boundary:** hard-dep per §9. Also implements §7 transport unification.

## Scope Fence

**IN scope (this inception):** Validate that §6's description still holds in light of what's been learned from earlier primitives. Refine open questions into a design proposal. Recommend GO / NO-GO / DEFER with rationale. Surface any newly-discovered sub-decomposition.

**OUT of scope (this inception):** Build/code work — that's a follow-on task created on GO. Other primitives' shapes — they have their own tasks. AEF orchestration layer integration — that's the §9 collaboration seam, owned at the boundary.

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

**GO if:**
- Latency measurement confirms §7's assumption; migration recipe is reversible; UDS path can be retired cleanly.

**NO-GO if:**
- Loopback TCP shows non-trivial latency at scale; OR migration path is unreversible.

**DEFER if:**
- Predecessor primitives have shifted shape in ways that change the open questions; capture the shift, update the ADR, re-file.

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

**Recommendation:** DEFER with measurement-first plan. Spike latency under concurrent-agent load before locking the migration. Resume the decision with data (revisit_at=2026-09-08).

**Rationale (one-paragraph):** Unlike T-2020/T-2021/T-2025/T-2027, this primitive is not verb-shaped — it's a transport-and-trust-model change that retires the UDS path. The cross-host TCP+HMAC+cert primitives already exist and work; nothing is missing infrastructure-wise. What IS missing is *measurement*: §7 asserts loopback TCP latency is negligible at homelab scale, but no spike has been run, and T-2019..T-2021 increase concurrent client load above prior measurement points. The same "asserted without evidence" shape as the T-1991 agent-presence bloat finding. The cost of a measurement spike (≤1 session) is small relative to the cost of cutting over and discovering a latency regression after some local-tool's UX degraded. Migration is recoverable but the risk surface is comparable to a CLI flag rename, not a verb addition.

**Full analysis:** see [docs/reports/T-2024-symmetric-auth-inception.md](../../docs/reports/T-2024-symmetric-auth-inception.md).

**Two-track unblock plan:**

**Track A — measurement spike (can run now, ≤1 session):**
- Set up local hub with both UDS and TCP listeners on loopback.
- Drive synthetic concurrent load: 10 clients × subscribe + 10 × post + 10 × claim/release cycles.
- Measure: p50/p95/p99 round-trip latency for each path; CPU/syscall overhead; connection-establish cost.
- Write `docs/reports/T-2024-latency-spike.md` with raw numbers + decision update.

**Track B — re-decide based on data (after Track A):**
- TCP-loopback p99 ≤ 2× UDS p99 → **GO**, file Phase-1 coexistence build task.
- TCP-loopback p99 > 5× UDS p99 → **NO-GO**, document gap, leave UDS in place but file audit-logging task to make trust-model gap observable.
- Between 2× and 5× → operator judgment with raw numbers in hand.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ⏸ "Latency measurement confirms §7's assumption" — UNRESOLVED, this is the gate.
- ✅ "Migration recipe is reversible" — staged coexistence pattern (4 phases) keeps UDS available throughout deprecation cycle.
- ✅ "UDS path can be retired cleanly" — yes, but only post-cutover after deprecation cycle completes.

**Open follow-up tasks to file:**
- *(Immediate, on DEFER)* Spike task — "T-2024 latency-measurement spike: TCP-loopback vs UDS under concurrent-agent load".
- *(Conditional, on Track A → GO)* Phase-1 build task — coexistence path with opt-in config flag.
- *(Conditional, on Track A → NO-GO)* Audit-logging task — add structured audit log on UDS path so trust-model gap is observable.

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

**Decision**: DEFER

**Rationale**: Recommendation: DEFER with measurement-first plan. Spike latency under concurrent-agent load before locking the migration. Resume the decision with data (revisit_at=2026-09-08).

Rationale (one-paragraph): Unlike T-2020/T-2021/T-2025/T-2027, this primitive is not verb-shaped — it's a transport-and-trust-model change that retires the UDS path. The cross-host TCP+HMAC+cert primitives already exist and work; nothing is missing infrastructure-wise. What IS missing is measurement: §7 asserts loopback TCP latency is negligible at homelab scale, but no spike has been run, and T-2019..T-2021 increase concurrent client load above prior measurement points. The same "asserted without evidence" shape as the T-1991 agent-presence bloat finding. The cost of a measurement spike (≤1 session) is small relative to the cost of cutting over and discovering a latency regression after some local-tool's UX degraded. Migration is recoverable but the risk surface is comparable to a CLI flag rename, not a verb addition.

Full analysis: see [docs/reports/T-2024-symmetric-auth-inception.md](../../docs/reports/T-2024-symmetric-auth-inception.md).

Two-track unblock plan:

Track A — measurement spike (can run now, ≤1 session):
- Set up local hub with both UDS and TCP listeners on loopback.
- Drive synthetic concurrent load: 10 clients × subscribe + 10 × post + 10 × claim/release cycles.
- Measure: p50/p95/p99 round-trip latency for each path; CPU/syscall overhead; connection-establish cost.
- Write `docs/reports/T-2024-latency-spike.md` with raw numbers + decision update.

Track B — re-decide based on data (after Track A):
- TCP-loopback p99 ≤ 2× UDS p99 → GO, file Phase-1 coexistence build task.
- TCP-loopback p99 > 5× UDS p99 → NO-GO, document gap, leave UDS in place but file audit-logging task to make trust-model gap observable.
- Between 2× and 5× → operator judgment with raw numbers in hand.

GO criteria evaluation (from §Go/No-Go Criteria):
- ⏸ "Latency measurement confirms §7's assumption" — UNRESOLVED, this is the gate.
- ✅ "Migration recipe is reversible" — staged coexistence pattern (4 phases) keeps UDS available throughout deprecation cycle.
- ✅ "UDS path can be retired cleanly" — yes, but only post-cutover after deprecation cycle completes.

Open follow-up tasks to file:
- (Immediate, on DEFER) Spike task — "T-2024 latency-measurement spike: TCP-loopback vs UDS under concurrent-agent load".
- (Conditional, on Track A → GO) Phase-1 build task — coexistence path with opt-in config flag.
- (Conditional, on Track A → NO-GO) Audit-logging task — add structured audit log on UDS path so trust-model gap is observable.

**Date**: 2026-06-08T11:20:55Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:35:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T11:20:55Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** Recommendation: DEFER with measurement-first plan. Spike latency under concurrent-agent load before locking the migration. Resume the decision with data (revisit_at=2026-09-08).

Rationale (one-paragraph): Unlike T-2020/T-2021/T-2025/T-2027, this primitive is not verb-shaped — it's a transport-and-trust-model change that retires the UDS path. The cross-host TCP+HMAC+cert primitives already exist and work; nothing is missing infrastructure-wise. What IS missing is measurement: §7 asserts loopback TCP latency is negligible at homelab scale, but no spike has been run, and T-2019..T-2021 increase concurrent client load above prior measurement points. The same "asserted without evidence" shape as the T-1991 agent-presence bloat finding. The cost of a measurement spike (≤1 session) is small relative to the cost of cutting over and discovering a latency regression after some local-tool's UX degraded. Migration is recoverable but the risk surface is comparable to a CLI flag rename, not a verb addition.

Full analysis: see [docs/reports/T-2024-symmetric-auth-inception.md](../../docs/reports/T-2024-symmetric-auth-inception.md).

Two-track unblock plan:

Track A — measurement spike (can run now, ≤1 session):
- Set up local hub with both UDS and TCP listeners on loopback.
- Drive synthetic concurrent load: 10 clients × subscribe + 10 × post + 10 × claim/release cycles.
- Measure: p50/p95/p99 round-trip latency for each path; CPU/syscall overhead; connection-establish cost.
- Write `docs/reports/T-2024-latency-spike.md` with raw numbers + decision update.

Track B — re-decide based on data (after Track A):
- TCP-loopback p99 ≤ 2× UDS p99 → GO, file Phase-1 coexistence build task.
- TCP-loopback p99 > 5× UDS p99 → NO-GO, document gap, leave UDS in place but file audit-logging task to make trust-model gap observable.
- Between 2× and 5× → operator judgment with raw numbers in hand.

GO criteria evaluation (from §Go/No-Go Criteria):
- ⏸ "Latency measurement confirms §7's assumption" — UNRESOLVED, this is the gate.
- ✅ "Migration recipe is reversible" — staged coexistence pattern (4 phases) keeps UDS available throughout deprecation cycle.
- ✅ "UDS path can be retired cleanly" — yes, but only post-cutover after deprecation cycle completes.

Open follow-up tasks to file:
- (Immediate, on DEFER) Spike task — "T-2024 latency-measurement spike: TCP-loopback vs UDS under concurrent-agent load".
- (Conditional, on Track A → GO) Phase-1 build task — coexistence path with opt-in config flag.
- (Conditional, on Track A → NO-GO) Audit-logging task — add structured audit log on UDS path so trust-model gap is observable.

### 2026-06-08T11:20:55Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)
- **Reason:** Inception decision: DEFER — parking task
