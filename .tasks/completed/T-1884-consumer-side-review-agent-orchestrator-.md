---
id: T-1884
name: "Consumer-side review-agent orchestrator — drain review-waiting queue (fw reviewer + ux-review + shell-validate + auto-followup)"
description: >
  Inception: Consumer-side review-agent orchestrator — drain review-waiting queue (fw reviewer + ux-review + shell-validate + auto-followup)

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T20:30:58Z
last_update: 2026-05-30T21:57:25Z
date_finished: 2026-05-30T21:57:25Z
---

# T-1884: Consumer-side review-agent orchestrator — drain review-waiting queue (fw reviewer + ux-review + shell-validate + auto-followup)

## Problem Statement

**The 47-task review-queue is a structural backlog.** Agent ACs are done, Human
ACs pending. The queue is unchanged for ~2 weeks despite three discovery skills
landing in the same window (T-1880/1881/1883). Growth rate from soak/agent-skill
work is ~3-5/week; without intervention the queue passes 100 in 10 weeks.

The Human ACs partition into three classes by content:

- `[REVIEW]` rendering — "watch view is steady", "output reads naturally", "table
  is operator-readable" → UI/CLI render surface, taste call still belongs to operator
- `[REVIEW]` CLI-output — "error messages name the failing input", "output is
  operator-scannable" → could be captured via `script -c` + grepped to surface
  evidence without ticking
- `[RUBBER-STAMP]` mechanical — "cron entry installed in /etc/cron.d", "MCP
  listing shows the three new tools", "GitHub Release published with macOS+Linux
  binaries" → shell/HTTP-validatable, no judgment needed

Upstream framework already ships two reviewer surfaces:
- `fw reviewer` (T-1443 v1.5) — static-scan + AUTO-TICK on PASS, but scope is
  **`[REVIEWER]`-prefixed Agent ACs only** (T-1950 Decisions 36/113/213 explicitly
  forbid auto-ticking `### Human`)
- `agents/ux-review/` (T-2002) — Playwright-driving review for interactive render
  surfaces, "informs, never decides"

Neither alone drains the 47. The constitutional rail (never tick Human ACs) is
preserved — but no consumer-side verb routes a per-task Human-AC by class to
the right validator, surfaces evidence in `## Updates` + Watchtower's verdict
block, and auto-files `investigate-and-fix` on FAIL.

**For whom:** the operator, draining a queue that grows faster than it shrinks.

**Why now:** doorbell+mail interactive arc shipped this session — the rail is
unblocked, but the review backlog negates 47 weeks of agent-ACs-passing work
because the operator click never happens.

## Assumptions

A1. AC class is detectable by **prefix** (`[REVIEW]` / `[RUBBER-STAMP]` /
    unprefixed) with high enough accuracy that mis-routing is rare. Register:
    `fw assumption add ...`

A2. Most `[RUBBER-STAMP]` Human ACs have **mechanical Steps** (curl/grep/
    file-exists/cron-grep/MCP-list) — shell-validatable without UI interaction
    or judgment.

A3. Most `[REVIEW]` Human ACs partition into (a) render-surface verifiable via
    ux-review's Playwright path, OR (b) CLI-output verifiable via `script -c`
    capture + content grep + flicker/regression check — both produce **evidence
    a human can rubber-stamp**, neither auto-ticks.

A4. Auto-filing `investigate-and-fix` follow-ups on FAIL doesn't generate noise
    — most ACs will PASS or be **inconclusive** (→ "needs human eyes" surfaced,
    NOT filed; filing is only triggered by definite regression evidence).

A5. Memory `[Validate-don't-punt]` ("when AC Steps are mechanical, RUN them and
    tick the box") is a project-local override of T-1950's constitutional rail.
    The orchestrator's policy on `[RUBBER-STAMP]` validation outcome — surface
    vs tick — is the load-bearing decision of this inception.

## Exploration Plan

Three time-boxed spikes (each ≤45 min):

### S1: AC-class classifier prototype

Read all 47 task files, extract `### Human` AC text + Steps blocks, classify each
into `{REVIEWER-AGENT-MISFILE, REVIEW-RENDER, REVIEW-CLI, RUBBER-STAMP-MECHANICAL,
RUBBER-STAMP-RELEASE, OTHER}` via prefix + content heuristics. Output: distribution
table (count per class) + 5 representative ACs per class for sanity check.

**Validates:** A1, partially A2.

### S2: Mechanical-Step extractor + dry-run validator

For each `RUBBER-STAMP-MECHANICAL` AC from S1, parse the **Steps:** block,
synthesize shell commands, dry-run them on a representative sample (5-10 ACs).
Compare actual exit code + output vs the **Expected:** block. Verdict per AC:
`PASS-ROBUST` (clean match), `PASS-LOOSE` (matches with hint), `FAIL`,
`INCONCLUSIVE` (Steps missing or non-deterministic).

**Validates:** A2, A4. Tests the actual mechanical-validatability claim against
real data, not memory.

### S3: ux-review wireup spike (one render-surface AC)

Pick T-1486 ("agent presence --watch view is steady — no flicker / no row
jumping") and drive `fw ux-review` (or a CLI proxy) to capture: screenshot,
console scan, multi-frame compare for flicker. Surface evidence into the task's
`## Updates` block. Verify operator can rubber-stamp from the evidence alone.

**Validates:** A3. Tests integration cost — if ux-review needs per-task config
that nullifies "one verb" UX, this spike fails the inception.

## Technical Constraints

- **Constitutional rail (T-1950, Decisions 36/113/213):** auto-tick is for
  `[REVIEWER]`-Agent ACs only. The orchestrator MUST NOT tick `### Human` ACs
  on its own authority. Memory `[Validate-don't-punt]` is operator-blessed but
  Tier-2 — pre-condition: operator selects this path explicitly per session,
  not as default.
- **Watchtower on :3003** (read `.context/working/watchtower.url`, never hardcode)
- **Consumer fw path** — `.agentic-framework/bin/fw`, not `bin/fw`
- **47-task batch is operator-owned** (`owner: human`) — agent INITIATIVE
  allows file-follow-ups + surface-evidence in Updates; agent AUTHORITY does
  NOT extend to ticking
- **G-019 RCA gate** — if any review FAILs and gets a follow-up `T-XXXX
  investigate-and-fix`, that task must include an RCA section before close
- **No upstream modification** — orchestrator wraps `fw reviewer` + `fw
  ux-review`, does not patch them
- **Reviewer prompt is already authoritative** — `agents/ux-review/AGENT.md`
  + `docs/reports/T-1950-reviewer-auto-tick-inception.md` upstream are the
  source of truth; this inception reads them, doesn't redefine them

## Scope Fence

**IN scope:**
- Consumer-side verb (working title `fw drain-review` or `fw review-queue` —
  decide during build) that batches over `partial-complete` tasks with unchecked
  `### Human` ACs
- Per-AC classifier + router
- Evidence surfacing into `## Updates` block of each task file (and Watchtower
  verdict block if appropriate)
- Auto-filing `T-XXXX investigate-and-fix-T-<src>` on FAIL with G-019 RCA stub
- Operator UX: one verb prints a per-task verdict line, links to evidence

**OUT of scope:**
- Auto-ticking `### Human` ACs (constitutional)
- Fleet-coordination across hosts (single-host operator UX)
- Modifying upstream `fw reviewer` / `agents/ux-review/`
- Replacing operator judgment on `[REVIEW]` taste calls (output reads naturally,
  watch view feels right) — agent surfaces evidence, operator clicks
- Bulk-batch retroactive over completed tasks (only operates on currently
  active `partial-complete`)

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
- S1 classifier achieves ≥80% confident routing on the 47 (i.e. ≤9 ACs land
  in OTHER bucket needing manual sort)
- S2 dry-run finds ≥15 of the 47 mechanically validatable (PASS-ROBUST or
  PASS-LOOSE) — i.e. the orchestrator can drain at least a third of the
  queue without operator click
- S3 confirms ux-review wireup needs ≤1 line of per-task config (i.e. just
  the URL or page-id), preserving the "one verb drains them" UX promise
- Operator-policy decision on `[Validate-don't-punt]` memory vs T-1950
  constitutional rail is recorded with explicit operator selection mechanism
  (flag default + per-session opt-in)

**NO-GO if:**
- AC classification ambiguity exceeds 50% (operator wins by sorting manually)
- <5 of 47 mechanically validatable (wrapper saves less than it costs)
- ux-review wireup needs per-task config (nullifies one-verb UX)
- Constitutional/memory tension cannot be resolved without surfacing it on
  every invocation (operator friction exceeds queue-drain benefit)

**DEFER if:**
- Spike S2 reveals the `[RUBBER-STAMP]` Steps blocks are inconsistently
  formatted across the 47 (preprocessing problem becomes the actual problem)
  — DEFER on building the orchestrator, file a follow-up to standardize Steps
  format first

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

**Recommendation:** GO

**Rationale:**

47 review-waiting tasks unchanged for 2+ weeks despite three discovery skills (recent-dm, check-arc, reply) shipping in same session. Upstream T-1950 + T-1443 already designed reviewer auto-tick for [REVIEWER]-Agent ACs and ux-review for [REVIEW] UI ACs — no constitutional new ground. Gap is a consumer-side orchestrator that routes by AC class (REVIEWER → static scan + auto-tick, REVIEW → ux-review surface, RUBBER-STAMP → shell-validate Steps), surfaces evidence in Updates + Watchtower, and auto-files investigate-and-fix on FAIL. Modest design surface (~3-spike inception): which path for mechanical RUBBER-STAMP under ### Human (constitutional surface vs memory-blessed tick), batch vs per-task UX, follow-up filing semantics. Direct value: drain a 47-deep queue with one operator verb instead of 47 manual click-throughs.

**Evidence:**

- **S1 (classifier):** 87.5% confident routing on 72 unchecked Human ACs across 63 tasks. 9-class taxonomy validates A-024.
  - `docs/reports/T-1884-S1-results.md` — full output
  - `scripts/T-1884-S1-classify.py` — 165-line classifier
- **S2 (mechanical-Step dry-run):** 25% first-round PASS-LOOSE; gap is parser+remote-exec, not structural NO-GO. Real bug surfaced as proof-of-design-value: T-1696 cron drift.
  - `docs/reports/T-1884-S2-results.md` — 16 ACs, 223 lines
  - `scripts/T-1884-S2-dryrun.py` — safety-classifier + executor
- **S3 (CLI-watch validator):** PASS-LOOSE on T-1486 — frame-capture technique works. Reframed: A-026 was about ux-review, but the 8 REVIEW-RENDER ACs are actually `--watch` CLI not browser UI. CLI-watch validator is correct tool.
  - `docs/reports/T-1884-S3-results.md`
  - `scripts/T-1884-S3-cli-watch.py`
- **Synthesis:** `docs/reports/T-1884-review-queue-orchestrator-inception.md` (final form with Recommendation section)

**Refined MVP scope (replaces filing-time framing):**
- v0.1 local-only ships REVIEW-CLI (32) + CLI-WATCH (8) + RUBBER-STAMP-RELEASE (1) = 41 ACs / 56% of queue
- v0.2 adds remote-exec → +RUBBER-STAMP-MECHANICAL (9) + OBSERVE-INFRA (7) = 57 ACs / 79% of queue
- Surface-only: OPERATOR-ACTION (6) + TIME-GATED (3) + OTHER (6) = 15 ACs / 21% — no validator possible by definition

**Assumption results:** A-024 ✓, A-025 ✓ (with caveats), A-026 ✓ (reframed), A-027 ✓.

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

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale:

47 review-waiting tasks unchanged for 2+ weeks despite three discovery skills (recent-dm, check-arc, reply) shipping in same session. Upstream T-1950 + T-1443 already designed reviewer auto-tick for [REVIEWER]-Agent ACs and ux-review for [REVIEW] UI ACs — no constitutional new ground. Gap is a consumer-side orchestrator that routes by AC class (REVIEWER → static scan + auto-tick, REVIEW → ux-review surface, RUBBER-STAMP → shell-validate Steps), surfaces evidence in Updates + Watchtower, and auto-files investigate-and-fix on FAIL. Modest design surface (~3-spike inception): which path for mechanical RUBBER-STAMP under ### Human (constitutional surface vs memory-blessed tick), batch vs per-task UX, follow-up filing semantics. Direct value: drain a 47-deep queue with one operator verb instead of 47 manual click-throughs.

Evidence:

- S1 (classifier): 87.5% confident routing on 72 unchecked Human ACs across 63 tasks. 9-class taxonomy validates A-024.
  - `docs/reports/T-1884-S1-results.md` — full output
  - `scripts/T-1884-S1-classify.py` — 165-line classifier
- S2 (mechanical-Step dry-run): 25% first-round PASS-LOOSE; gap is parser+remote-exec, not structural NO-GO. Real bug surfaced as proof-of-design-value: T-1696 cron drift.
  - `docs/reports/T-1884-S2-results.md` — 16 ACs, 223 lines
  - `scripts/T-1884-S2-dryrun.py` — safety-classifier + executor
- S3 (CLI-watch validator): PASS-LOOSE on T-1486 — frame-capture technique works. Reframed: A-026 was about ux-review, but the 8 REVIEW-RENDER ACs are actually `--watch` CLI not browser UI. CLI-watch validator is correct tool.
  - `docs/reports/T-1884-S3-results.md`
  - `scripts/T-1884-S3-cli-watch.py`
- Synthesis: `docs/reports/T-1884-review-queue-orchestrator-inception.md` (final form with Recommendation section)

Refined MVP scope (replaces filing-time framing):
- v0.1 local-only ships REVIEW-CLI (32) + CLI-WATCH (8) + RUBBER-STAMP-RELEASE (1) = 41 ACs / 56% of queue
- v0.2 adds remote-exec → +RUBBER-STAMP-MECHANICAL (9) + OBSERVE-INFRA (7) = 57 ACs / 79% of queue
- Surface-only: OPERATOR-ACTION (6) + TIME-GATED (3) + OTHER (6) = 15 ACs / 21% — no validator possible by definition

Assumption results: A-024 ✓, A-025 ✓ (with caveats), A-026 ✓ (reframed), A-027 ✓.

**Date**: 2026-05-30T21:57:25Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-30T20:33:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-30T21:57:25Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

47 review-waiting tasks unchanged for 2+ weeks despite three discovery skills (recent-dm, check-arc, reply) shipping in same session. Upstream T-1950 + T-1443 already designed reviewer auto-tick for [REVIEWER]-Agent ACs and ux-review for [REVIEW] UI ACs — no constitutional new ground. Gap is a consumer-side orchestrator that routes by AC class (REVIEWER → static scan + auto-tick, REVIEW → ux-review surface, RUBBER-STAMP → shell-validate Steps), surfaces evidence in Updates + Watchtower, and auto-files investigate-and-fix on FAIL. Modest design surface (~3-spike inception): which path for mechanical RUBBER-STAMP under ### Human (constitutional surface vs memory-blessed tick), batch vs per-task UX, follow-up filing semantics. Direct value: drain a 47-deep queue with one operator verb instead of 47 manual click-throughs.

Evidence:

- S1 (classifier): 87.5% confident routing on 72 unchecked Human ACs across 63 tasks. 9-class taxonomy validates A-024.
  - `docs/reports/T-1884-S1-results.md` — full output
  - `scripts/T-1884-S1-classify.py` — 165-line classifier
- S2 (mechanical-Step dry-run): 25% first-round PASS-LOOSE; gap is parser+remote-exec, not structural NO-GO. Real bug surfaced as proof-of-design-value: T-1696 cron drift.
  - `docs/reports/T-1884-S2-results.md` — 16 ACs, 223 lines
  - `scripts/T-1884-S2-dryrun.py` — safety-classifier + executor
- S3 (CLI-watch validator): PASS-LOOSE on T-1486 — frame-capture technique works. Reframed: A-026 was about ux-review, but the 8 REVIEW-RENDER ACs are actually `--watch` CLI not browser UI. CLI-watch validator is correct tool.
  - `docs/reports/T-1884-S3-results.md`
  - `scripts/T-1884-S3-cli-watch.py`
- Synthesis: `docs/reports/T-1884-review-queue-orchestrator-inception.md` (final form with Recommendation section)

Refined MVP scope (replaces filing-time framing):
- v0.1 local-only ships REVIEW-CLI (32) + CLI-WATCH (8) + RUBBER-STAMP-RELEASE (1) = 41 ACs / 56% of queue
- v0.2 adds remote-exec → +RUBBER-STAMP-MECHANICAL (9) + OBSERVE-INFRA (7) = 57 ACs / 79% of queue
- Surface-only: OPERATOR-ACTION (6) + TIME-GATED (3) + OTHER (6) = 15 ACs / 21% — no validator possible by definition

Assumption results: A-024 ✓, A-025 ✓ (with caveats), A-026 ✓ (reframed), A-027 ✓.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7f338264
- **Timestamp:** 2026-05-30T21:57:25Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-30T21:57:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
