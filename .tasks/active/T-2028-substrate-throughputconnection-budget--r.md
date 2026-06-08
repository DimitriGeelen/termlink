---
id: T-2028
name: "Substrate: throughput/connection budget + retention/compaction policy"
description: >
  §6 primitive 10 (Supporting). No connection cap, rate limiter, or backpressure governor
  exists. T-1991 (agent-presence bloat) was found in PRODUCTION, not predicted. The
  coordination/announcement pattern AEF wants generates exactly that traffic class,
  so retention/compaction must be designed in from the start, not bolted on.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:55Z
last_update: 2026-06-08T07:39:51Z
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

# T-2028: Substrate: throughput/connection budget + retention/compaction policy

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Supporting. **§9 boundary:** cross-cutting policy — touches every primitive's design.

**Role per ADR §6:** No connection cap, rate limiter, or backpressure governor exists in code. T-1991 (agent-presence bloat to ~1800 envelopes) was found in PRODUCTION, not predicted. The coordination/announcement pattern AEF wants generates exactly that traffic class — retention/compaction must be designed in from the start, not bolted on.

**Why captured now:** Capture-while-fresh while T-1991's precedent is sharp. Likely the LAST primitive to actually build (it's a cross-cutting review), but filing it last would risk losing the policy context.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Per-topic retention — Days(N) / Forever / message-count — confirm the existing API or add it?**
  confidence: 4
  disposition: answered
  rationale: ALREADY EXISTS. `crates/termlink-bus/src/retention.rs` defines `enum Retention { Forever, Days(N), Messages(N) }`; `Bus::create_topic(name, retention)` and `Bus::topic_retention(topic)` are in `lib.rs:92-103`; compaction logic at `lib.rs:375-385`. Per-topic policy, set at creation time. One small addition worth shipping: `Retention::Latest` as T-2027's compaction-side sibling. See docs/reports/T-2028-throughput-retention-inception.md §2, §4.A.

- **IW-2: Compaction trigger — time-based, size-based, both? Per-topic policy?**
  confidence: 4
  disposition: answered
  rationale: BOTH MODES EXIST, per-topic. `Retention::Days(N)` = time-based; `Retention::Messages(N)` = size-based. Compaction enforces whichever policy the topic was created with. T-1991 was a case where the bounded-policy was available but `agent-presence` had been set to `Forever` — operator awareness gap, not API gap. See artifact §3.

- **IW-3: Connection cap — per-process, per-host, per-hub? Behavior when hit — queue or refuse?**
  confidence: 3
  disposition: answered
  rationale: PER-PROCESS, REFUSE with structured error. Per-process matches the deployment shape (typically one hub per host); refuse-with-structured-error is loud per IW-3 hint and aligns with G-058 silent-failure precedent. Concrete: `code=-32029 OVERLOADED`, `retry_after_ms` in error data, surfaced in CLI as "hub at capacity (retry in 2.3s)". See artifact §4.B.

- **IW-4: Rate limit — per-sender, per-topic, per-RPC? Budget visible to clients in `topic info`?**
  confidence: 3
  disposition: answered
  rationale: PER-SENDER. Per-topic adds policy complexity for limited gain; per-RPC is too granular. Per-sender bucket aligns with the trust-model (HMAC identifies the sender). Observability: surface via `hub status` (top senders, hit counts) + per-RPC response headers (X-RateLimit-style). Visible via Track C (separate small build task). See artifact §4.B-C.

- **IW-5: T-1991 precedent — what was the would-have-helped policy?**
  confidence: 4
  disposition: answered
  rationale: TWO-PRONGED. The actual fix was subscribe-path resilience (per-binary-version slowdown regression). But topic-size bounding via `Retention::Messages(200)` on agent-presence was ALWAYS available — just not applied. The deeper miss was observability: had `channel info agent-presence` surfaced "growing 60 envelopes/min, retention=Forever, runway-to-pain ~30 min", an operator would have set retention before the wedge. Hence Track C (observability) is core to preventing T-1991 recurrence. See artifact §3.

## Exploration Plan

Treat as a cross-cutting review. After Foundation lands, review each Foundation/Resilience primitive's design against the budget. Then build the missing policy primitives.

## Technical Constraints

**Dependencies (upstream):** None directly

**Dependencies (downstream):** Every other primitive should respect the budget at design — review each Foundation/Resilience primitive's design against this

**ADR §9 boundary:** cross-cutting policy — touches every primitive's design

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
- Cross-cutting review surfaced concrete budget; each primitive's design respects it; missing policy primitives bounded and small.

**NO-GO if:**
- Budget review reveals fundamental scale mismatch — substrate cannot support intended fleet size without redesign of channel storage.

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

**Recommendation:** PARTIAL GO with three sub-tracks (A: retention audit + Retention::Latest; B: connection cap + per-sender rate limit; C: budget observability). Each track is a small independent build task.

**Rationale (one-paragraph):** The §6 framing bundles three sub-problems with very different statuses. Retention/compaction ALREADY EXISTS as per-topic policy with both time-based (Days) and size-based (Messages) modes — T-1991 was an operator-awareness gap, not an API gap, since the bounded policy was always available for agent-presence. One small additive surface (`Retention::Latest` to complete T-2027's broadcast-with-replay story) closes the retention half. Connection cap + per-sender rate limit ARE genuinely missing — standard governor pattern, ~150 LOC, refuse-with-structured-error semantics keep failures loud. Budget observability (surface state in `channel info` + `hub status`) is the T-1991-prevention piece: had operators seen "agent-presence growing 60 env/min, retention=Forever, runway=30 min" they would have set retention before the wedge. Three small independent tasks ship cleanly in order without bundling.

**Full design + IW dispositions:** see [docs/reports/T-2028-throughput-retention-inception.md](../../docs/reports/T-2028-throughput-retention-inception.md).

**Build slice plan (three independent tracks):**

**Track A — Retention audit + `Retention::Latest` (~30 LOC):**
- Audit topics created by substrate code; ensure each sets a retention.
- Add `Retention::Latest` enum variant + compaction case (keep most recent envelope only).
- Bundle with T-2027 build task if T-2027 goes (subscribe-side + compaction-side complete the broadcast-with-replay story together).

**Track B — Connection cap + per-sender rate limit (~150 LOC):**
- Per-process connection governor (`MAX_CLIENT_CONNECTIONS=64` configurable).
- Per-sender token bucket (e.g. 100 RPCs/s/sender).
- Refuse with `code=-32029 OVERLOADED`, `retry_after_ms` in error data.
- CLI surfacing: "hub at capacity (retry in 2.3s)".

**Track C — Budget observability (~80 LOC):**
- Surface in `channel info <topic>`: current size, retention policy, growth rate over last hour.
- Surface in `hub status`: connection count, rate-limit hits, top senders by RPC count.
- CLI + MCP read paths.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ✅ "Cross-cutting review surfaced concrete budget" — three tracks, each scoped and sized.
- ✅ "Each primitive's design respects it" — retention already does; connection/rate gets per-sender bucket; observability surfaces it.
- ✅ "Missing policy primitives bounded and small" — 30 + 150 + 80 LOC.

**Open follow-up tasks to file on GO:**
- Track A audit + `Retention::Latest` build task (consider bundling with T-2027 Slice 1-4).
- Track B governor build task.
- Track C observability build task.
- *(Operator)* Set `agent-presence` retention to `Messages(200)` (or measure-informed N) — small operator action once Track A lands, prevents T-1991 recurrence.

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

**Rationale**: Recommendation: PARTIAL GO with three sub-tracks (A: retention audit + Retention::Latest; B: connection cap + per-sender rate limit; C: budget observability). Each track is a small independent build task.

Rationale (one-paragraph): The §6 framing bundles three sub-problems with very different statuses. Retention/compaction ALREADY EXISTS as per-topic policy with both time-based (Days) and size-based (Messages) modes — T-1991 was an operator-awareness gap, not an API gap, since the bounded policy was always available for agent-presence. One small additive surface (`Retention::Latest` to complete T-2027's broadcast-with-replay story) closes the retention half. Connection cap + per-sender rate limit ARE genuinely missing — standard governor pattern, ~150 LOC, refuse-with-structured-error semantics keep failures loud. Budget observability (surface state in `channel info` + `hub status`) is the T-1991-prevention piece: had operators seen "agent-presence growing 60 env/min, retention=Forever, runway=30 min" they would have set retention before the wedge. Three small independent tasks ship cleanly in order without bundling.

Full design + IW dispositions: see [docs/reports/T-2028-throughput-retention-inception.md](../../docs/reports/T-2028-throughput-retention-inception.md).

Build slice plan (three independent tracks):

Track A — Retention audit + `Retention::Latest` (~30 LOC):
- Audit topics created by substrate code; ensure each sets a retention.
- Add `Retention::Latest` enum variant + compaction case (keep most recent envelope only).
- Bundle with T-2027 build task if T-2027 goes (subscribe-side + compaction-side complete the broadcast-with-replay story together).

Track B — Connection cap + per-sender rate limit (~150 LOC):
- Per-process connection governor (`MAX_CLIENT_CONNECTIONS=64` configurable).
- Per-sender token bucket (e.g. 100 RPCs/s/sender).
- Refuse with `code=-32029 OVERLOADED`, `retry_after_ms` in error data.
- CLI surfacing: "hub at capacity (retry in 2.3s)".

Track C — Budget observability (~80 LOC):
- Surface in `channel info <topic>`: current size, retention policy, growth rate over last hour.
- Surface in `hub status`: connection count, rate-limit hits, top senders by RPC count.
- CLI + MCP read paths.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ "Cross-cutting review surfaced concrete budget" — three tracks, each scoped and sized.
- ✅ "Each primitive's design respects it" — retention already does; connection/rate gets per-sender bucket; observability surfaces it.
- ✅ "Missing policy primitives bounded and small" — 30 + 150 + 80 LOC.

Open follow-up tasks to file on GO:
- Track A audit + `Retention::Latest` build task (consider bundling with T-2027 Slice 1-4).
- Track B governor build task.
- Track C observability build task.
- (Operator) Set `agent-presence` retention to `Messages(200)` (or measure-informed N) — small operator action once Track A lands, prevents T-1991 recurrence.

**Date**: 2026-06-08T10:01:26Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:37:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T10:01:26Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: PARTIAL GO with three sub-tracks (A: retention audit + Retention::Latest; B: connection cap + per-sender rate limit; C: budget observability). Each track is a small independent build task.

Rationale (one-paragraph): The §6 framing bundles three sub-problems with very different statuses. Retention/compaction ALREADY EXISTS as per-topic policy with both time-based (Days) and size-based (Messages) modes — T-1991 was an operator-awareness gap, not an API gap, since the bounded policy was always available for agent-presence. One small additive surface (`Retention::Latest` to complete T-2027's broadcast-with-replay story) closes the retention half. Connection cap + per-sender rate limit ARE genuinely missing — standard governor pattern, ~150 LOC, refuse-with-structured-error semantics keep failures loud. Budget observability (surface state in `channel info` + `hub status`) is the T-1991-prevention piece: had operators seen "agent-presence growing 60 env/min, retention=Forever, runway=30 min" they would have set retention before the wedge. Three small independent tasks ship cleanly in order without bundling.

Full design + IW dispositions: see [docs/reports/T-2028-throughput-retention-inception.md](../../docs/reports/T-2028-throughput-retention-inception.md).

Build slice plan (three independent tracks):

Track A — Retention audit + `Retention::Latest` (~30 LOC):
- Audit topics created by substrate code; ensure each sets a retention.
- Add `Retention::Latest` enum variant + compaction case (keep most recent envelope only).
- Bundle with T-2027 build task if T-2027 goes (subscribe-side + compaction-side complete the broadcast-with-replay story together).

Track B — Connection cap + per-sender rate limit (~150 LOC):
- Per-process connection governor (`MAX_CLIENT_CONNECTIONS=64` configurable).
- Per-sender token bucket (e.g. 100 RPCs/s/sender).
- Refuse with `code=-32029 OVERLOADED`, `retry_after_ms` in error data.
- CLI surfacing: "hub at capacity (retry in 2.3s)".

Track C — Budget observability (~80 LOC):
- Surface in `channel info <topic>`: current size, retention policy, growth rate over last hour.
- Surface in `hub status`: connection count, rate-limit hits, top senders by RPC count.
- CLI + MCP read paths.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ "Cross-cutting review surfaced concrete budget" — three tracks, each scoped and sized.
- ✅ "Each primitive's design respects it" — retention already does; connection/rate gets per-sender bucket; observability surfaces it.
- ✅ "Missing policy primitives bounded and small" — 30 + 150 + 80 LOC.

Open follow-up tasks to file on GO:
- Track A audit + `Retention::Latest` build task (consider bundling with T-2027 Slice 1-4).
- Track B governor build task.
- Track C observability build task.
- (Operator) Set `agent-presence` retention to `Messages(200)` (or measure-informed N) — small operator action once Track A lands, prevents T-1991 recurrence.
