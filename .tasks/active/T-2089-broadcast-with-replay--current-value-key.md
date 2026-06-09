---
id: T-2089
name: "Broadcast-with-replay / current-value key — substrate primitive 9"
description: >
  Inception: Broadcast-with-replay / current-value key — substrate primitive 9

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-09T14:25:32Z
last_update: 2026-06-09T14:28:53Z
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

# T-2089: Broadcast-with-replay / current-value key — substrate primitive 9

## Problem Statement

ADR §6 #9 calls for "broadcast-with-replay / current-value key surfaced to a subscriber on registration — for late-joiner 'room state' without replaying an entire log." Today every subscribe-then-state query walks the entire envelope log (`channel.subscribe cursor=0` or `channel.snapshot`). For `agent-presence` (10K+ heartbeats, ~10 distinct agent_ids) the operator's "who's around right now?" question costs O(N_heartbeats) to produce O(N_agents) answers. The substrate needs an O(K)-bounded "current value per key" path.

See `docs/reports/T-2089-broadcast-with-replay-inception.md` for full design analysis (4 alternatives, recommendation, slice plan).

## Assumptions

- A1: Posters can opt-in by adding `metadata.cv_key=<string>` to relevant posts (the same opt-in pattern as `metadata.client_msg_id` for T-2049 dedupe).
- A2: Distinct cv_keys per topic stay bounded (cap at 1000; LOUD-refuse like T-2049 dedupe).
- A3: Rebuilding the cv_index on hub startup from one O(N) scan per topic is acceptable (already done for agent-presence retention bookkeeping per T-1991).

## Open Questions

- **IW-1: Does cv_index need persistence beyond the startup scan?**
  confidence: 2
  disposition: answered
  rationale: SQLite is already the durability layer for envelopes; the index is a derivative view. Rebuilding on startup is the same model as the existing presence-derivation and claim-index patterns. Persisted sidecar table doubles the write path without functional gain. See docs/reports/T-2089-broadcast-with-replay-inception.md §6.

- **IW-2: What's the semantics when a second poster updates `cv_key=X`?**
  confidence: 3
  disposition: answered
  rationale: Last-write-wins on offset — symmetric with agent-presence LIVE/STALE/OFFLINE (latest heartbeat per agent_id wins). See docs/reports/T-2089-broadcast-with-replay-inception.md §6.

- **IW-3: Does `cv_key` namespace overlap with existing metadata fields?**
  confidence: 2
  disposition: answered
  rationale: No collision — existing keys (`conversation_id`, `in_reply_to`, `cv`-claim, `redacts`, `capabilities`, `agent_id`) are distinct from new `cv_key`. Schema doc to land in Slice 1.

- **IW-4: Does a later-redacted envelope still surface as current value?**
  confidence: 2
  disposition: answered
  rationale: Hub MUST filter redacted offsets out of the cv-prefix delivery on subscribe — redaction collapse is already implemented in `compute_state`. Slice 1 includes the filter + unit test.

- **IW-5: Should cv_index be bounded per-topic?**
  confidence: 2
  disposition: answered
  rationale: Cap at TERMLINK_CV_INDEX_MAX_KEYS_PER_TOPIC=1000 with LOUD-refuse (mirror of T-2049 dedupe cap pattern, PL-204 invariant). For known use-cases (presence: agent_id, DM: read_receipt:<sender>, chat: pinned), K stays small.

## Exploration Plan

Inception artifact (docs/reports/T-2089-broadcast-with-replay-inception.md):
- §1 Problem (10K-row cost on agent-presence)
- §2 Four alternatives (A: tagged-post cv_index, B: last-N tail, C: snapshot+subscribe recipe, D: KV sidecar)
- §3 Recommendation rationale (A wins)
- §4 5-slice build plan (T-2090..T-2094) mirroring T-2078..T-2087 observability arc pattern
- §5..§9 ACs, IWs, related primitives, dialogue, refs

No code spike needed — the design is analytically bounded by existing claim-index pattern (T-2019) and existing presence-derivation pattern. Recommendation moves DEFER → GO.

## Technical Constraints

- In-process HashMap (per-topic, per-cv_key → latest offset). No SQLite migration; no new RPC.
- Backward-compatible wire protocol: extend `channel.subscribe` with optional `include_current_value: bool=false` param; old callers unaffected.
- Restart safety: O(N) per-topic startup scan to rebuild — bounded by retention (T-1991 caps agent-presence at 200; other topics by operator-configured retention).
- Redaction-aware: hub filters redacted offsets out of cv-prefix delivery on subscribe.
- Per-topic cv_key cap (LOUD-refuse pattern, PL-204): TERMLINK_CV_INDEX_MAX_KEYS_PER_TOPIC default 1000.

## Scope Fence

**IN:** Inception decision (GO/NO-GO/DEFER), design artifact, slice plan, AC structure for build tasks.

**OUT (build artifacts — only after GO decision):**
- Slice 1 (T-2090): Hub-side `cv_index` HashMap + post-side wire + startup scan + unit tests
- Slice 2 (T-2091): `channel.subscribe --include-current-value` wire
- Slice 3 (T-2092): CLI flag + MCP parity
- Slice 4 (T-2093): `channel cv-keys` inspection verb
- Slice 5 (T-2094): `/peers` ↔ heartbeat cv_key=agent_id end-to-end smoke

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

**GO if:**
- Design A (tagged-post current-value index) is bounded — in-memory HashMap per topic, one optional `include_current_value` param on `channel.subscribe`, no schema migration.
- Slice plan converges into 5 independently shippable build tasks (T-2090..T-2094) each fitting one session, mirroring the proven T-2078..T-2087 observability arc pattern.
- Existing claim-index pattern (T-2019, channel.rs:923) confirms the storage shape is already proven inside the codebase — this generalizes it.
- Storage size is negligible (agent-presence with 50 agents = ~2.5KB; topics without cv_keys = 0 bytes).
- Restart safety: O(N) startup scan per topic is the same model presence-derivation already uses.

**NO-GO if:**
- Persisting cv_index to SQLite is required (would mean schema migration + sidecar table — strictly larger than design A with no functional gain → reject design D).
- Backward compatibility breaks for existing subscribers — but it doesn't: opt-in by both poster (cv_key metadata) and subscriber (flag), old callers unaffected.
- Per-topic cv_key cardinality is unbounded — mitigated by TERMLINK_CV_INDEX_MAX_KEYS_PER_TOPIC=1000 LOUD-refuse cap (PL-204 invariant, mirror of T-2049 dedupe cap).

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

**Recommendation:** GO (Design A — tagged-post cv_index)

**Rationale:**

Research artifact at `docs/reports/T-2089-broadcast-with-replay-inception.md` analyzed four design alternatives (A: tagged-post cv_index, B: last-N tail replay, C: snapshot+subscribe recipe, D: KV sidecar) and recommends Design A. The design is bounded by an existing in-codebase pattern (claim-index, T-2019), backward compatible, restart-safe, and directly unlocks the operator pain point on `agent-presence` (`/peers` becomes O(N_agents) instead of O(N_heartbeats)). Build decomposes into 5 independently shippable slices (T-2090..T-2094) mirroring the proven T-2078..T-2087 substrate observability arc pattern. All five IW questions have been answered (see Open Questions section). Filing GO.

**Evidence:**

- ADR §6 #9 explicit primitive requirement: `docs/architecture/parallel-execution-substrate.md:267-268`
- Existing claim-index pattern (proof of storage shape): `crates/termlink-hub/src/channel.rs:923` (`channel.claim`)
- Existing channel.subscribe wire (extension point): `crates/termlink-hub/src/channel.rs:594`
- Existing snapshot walk-cost pattern this addresses: `crates/termlink-cli/src/commands/channel.rs:6644` (`cmd_channel_snapshot`)
- Agent-presence high-rate context (target use case): `crates/termlink-hub/src/channel.rs:327` + T-1991/G-058 retention cap
- Build slice plan: docs/reports/T-2089-broadcast-with-replay-inception.md §4 (5 tasks T-2090..T-2094)

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

### 2026-06-09T14:28:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
