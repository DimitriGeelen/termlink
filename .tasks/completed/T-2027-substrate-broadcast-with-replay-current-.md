---
id: T-2027
name: "Substrate: broadcast-with-replay (current-value-on-registration topic)"
description: >
  §6 primitive 9 (Supporting). For late-joiner room-state without replaying an entire
  log. Subscriber registers and receives the current value of a designated key, then
  live updates. Smaller spec than the Foundation primitives.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:50Z
last_update: 2026-06-08T07:33:29Z
date_finished: 2026-06-08T10:05:49Z
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

# T-2027: Substrate: broadcast-with-replay (current-value-on-registration topic)

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Supporting. **§9 boundary:** supporting per §9 (small scope, but still hard-dep style).

**Role per ADR §6:** For late-joiner room-state without replaying an entire log. Subscriber registers and receives the current value of a designated key, then live updates from cursor onward.

**Why captured now:** Smaller piece. Filing now while the use case is in view; may consolidate with T-2025.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Current-value as a topic-level config, or per-subscriber on registration?**
  confidence: 4
  disposition: answered
  rationale: PER-SUBSCRIBER on registration (new `--from-latest [--once|--then-live]` flag on `channel.subscribe`). More flexible than a topic config — the same topic can be read in either mode by different consumers (dashboards want from-latest, audit tools want since-offset=0). See docs/reports/T-2027-broadcast-with-replay-inception.md §4-§5.

- **IW-2: How is 'current' defined — last-written, or a separate snapshot the publisher updates?**
  confidence: 4
  disposition: answered
  rationale: LAST-WRITTEN. Adding a separate "publisher writes a snapshot" mechanism doubles the API surface and adds publisher-coordination cost. Last-written is what consumers mean by "current". Future structured-snapshot use cases are an overlay on top, not a substrate primitive. See artifact §5.IW-2.

- **IW-3: Storage — keep current value in SQLite alongside log, or a separate kv store?**
  confidence: 4
  disposition: answered
  rationale: NEITHER — no extra storage needed. The latest envelope IS what's at the topic's max offset. `--from-latest` is a read pattern over existing data. Existing `kv` primitive is session-scoped (not topic-shaped, no watch) so doesn't fit. Compaction is optional and deferred to T-2028. See artifact §5.IW-3.

- **IW-4: Cursor interaction — new subscriber starts at current-value + live, or at cursor=0?**
  confidence: 4
  disposition: answered
  rationale: AT CURRENT-VALUE + LIVE. That's the explicit point of the primitive. Old behavior (`--since-offset 0`) remains the explicit replay-everything mode for audit tools. Atomic by design: hub holds topic read-mutex while resolving "latest" and seeking the cursor, so no posts can land between. See artifact §5.IW-4.

## Exploration Plan

At promotion time: (1) consolidate with T-2025 if persistent presence already covers the use case; (2) otherwise small spec + build.

## Technical Constraints

**Dependencies (upstream):** None (independent topic-level feature)

**Dependencies (downstream):** Enables clean late-joiner UX for any topic carrying room-state (presence dashboards, status boards)

**ADR §9 boundary:** supporting per §9 (small scope, but still hard-dep style)

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
- Either consolidation with T-2025 is decided OR a small bounded spec is locked.

**NO-GO if:**
- Use case is fully covered by T-2025 — close as duplicate.

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

**Recommendation:** GO with revised scope (ship subscribe-side `--from-latest` flag; defer compaction-side `retention: keep-latest` to T-2028).

**Rationale (one-paragraph):** The gap is real. Existing `kv` is session-scoped (not topic-shaped, no watch), and `channel.subscribe` lacks atomic "latest + live" mode — the two-call workaround has a race window. The fix splits cleanly: subscribe-side adds one new flag (`--from-latest [--once|--then-live]`) with atomic semantics (hub holds topic read-mutex during latest-resolve + cursor-seek). Compaction-side (`retention: keep-latest`) is an optimization, not a correctness requirement — without it the topic grows; if growth becomes a problem T-2028 addresses it. Build is ~80 LOC across 4 vertical slices, purely additive surface, no schema migration, no conflicts with existing primitives.

**Full design + IW dispositions:** see [docs/reports/T-2027-broadcast-with-replay-inception.md](../../docs/reports/T-2027-broadcast-with-replay-inception.md).

**Build slice plan:**
- Slice 1: `Bus::subscribe_from_latest` library function + unit tests (happy path, empty topic, concurrent-post race).
- Slice 2: Hub handler — extend `channel.subscribe` parameter parsing to accept `--from-latest` mode; route through the new bus function.
- Slice 3: CLI flag `--from-latest [--once|--then-live]` on `termlink channel subscribe`; session-client wrapper.
- Slice 4: MCP tool `termlink_channel_subscribe_from_latest` + help-registry entry + docs showing the late-joiner-dashboard recipe.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ✅ "A small bounded spec is locked" — 80 LOC, 4 slices, additive only.
- ❌ "Use case is fully covered by T-2025" — T-2025 went NO-GO covering presence durability; T-2027 covers a different read pattern.

**Open follow-up tasks to file on GO:**
- Build task: Slices 1-4 (`channel.subscribe --from-latest`).
- *(Pre-existing)* T-2028 inception for `retention: keep-latest` compaction policy — independent track.

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

**Rationale**: Recommendation: GO with revised scope (ship subscribe-side `--from-latest` flag; defer compaction-side `retention: keep-latest` to T-2028).

Rationale (one-paragraph): The gap is real. Existing `kv` is session-scoped (not topic-shaped, no watch), and `channel.subscribe` lacks atomic "latest + live" mode — the two-call workaround has a race window. The fix splits cleanly: subscribe-side adds one new flag (`--from-latest [--once|--then-live]`) with atomic semantics (hub holds topic read-mutex during latest-resolve + cursor-seek). Compaction-side (`retention: keep-latest`) is an optimization, not a correctness requirement — without it the topic grows; if growth becomes a problem T-2028 addresses it. Build is ~80 LOC across 4 vertical slices, purely additive surface, no schema migration, no conflicts with existing primitives.

Full design + IW dispositions: see [docs/reports/T-2027-broadcast-with-replay-inception.md](../../docs/reports/T-2027-broadcast-with-replay-inception.md).

Build slice plan:
- Slice 1: `Bus::subscribe_from_latest` library function + unit tests (happy path, empty topic, concurrent-post race).
- Slice 2: Hub handler — extend `channel.subscribe` parameter parsing to accept `--from-latest` mode; route through the new bus function.
- Slice 3: CLI flag `--from-latest [--once|--then-live]` on `termlink channel subscribe`; session-client wrapper.
- Slice 4: MCP tool `termlink_channel_subscribe_from_latest` + help-registry entry + docs showing the late-joiner-dashboard recipe.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ "A small bounded spec is locked" — 80 LOC, 4 slices, additive only.
- ❌ "Use case is fully covered by T-2025" — T-2025 went NO-GO covering presence durability; T-2027 covers a different read pattern.

Open follow-up tasks to file on GO:
- Build task: Slices 1-4 (`channel.subscribe --from-latest`).
- (Pre-existing) T-2028 inception for `retention: keep-latest` compaction policy — independent track.

**Date**: 2026-06-08T10:01:17Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:31:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T10:01:17Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO with revised scope (ship subscribe-side `--from-latest` flag; defer compaction-side `retention: keep-latest` to T-2028).

Rationale (one-paragraph): The gap is real. Existing `kv` is session-scoped (not topic-shaped, no watch), and `channel.subscribe` lacks atomic "latest + live" mode — the two-call workaround has a race window. The fix splits cleanly: subscribe-side adds one new flag (`--from-latest [--once|--then-live]`) with atomic semantics (hub holds topic read-mutex during latest-resolve + cursor-seek). Compaction-side (`retention: keep-latest`) is an optimization, not a correctness requirement — without it the topic grows; if growth becomes a problem T-2028 addresses it. Build is ~80 LOC across 4 vertical slices, purely additive surface, no schema migration, no conflicts with existing primitives.

Full design + IW dispositions: see [docs/reports/T-2027-broadcast-with-replay-inception.md](../../docs/reports/T-2027-broadcast-with-replay-inception.md).

Build slice plan:
- Slice 1: `Bus::subscribe_from_latest` library function + unit tests (happy path, empty topic, concurrent-post race).
- Slice 2: Hub handler — extend `channel.subscribe` parameter parsing to accept `--from-latest` mode; route through the new bus function.
- Slice 3: CLI flag `--from-latest [--once|--then-live]` on `termlink channel subscribe`; session-client wrapper.
- Slice 4: MCP tool `termlink_channel_subscribe_from_latest` + help-registry entry + docs showing the late-joiner-dashboard recipe.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ "A small bounded spec is locked" — 80 LOC, 4 slices, additive only.
- ❌ "Use case is fully covered by T-2025" — T-2025 went NO-GO covering presence durability; T-2027 covers a different read pattern.

Open follow-up tasks to file on GO:
- Build task: Slices 1-4 (`channel.subscribe --from-latest`).
- (Pre-existing) T-2028 inception for `retention: keep-latest` compaction policy — independent track.
