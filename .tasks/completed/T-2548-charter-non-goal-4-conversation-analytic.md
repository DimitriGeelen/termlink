---
id: T-2548
name: "Charter non-goal #4: conversation-analytics MCP tool family subtract-vs-keep"
description: >
  Off-charter subtract candidate from T-2468 purpose review: ~30-40 LIVE conversation-analytics MCP tools that fail all four charter verbs; zero first-party callers; sibling of T-2471/T-2478 pruned social-analytics set.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-08T19:34:48Z
last_update: 2026-08-08T19:37:13Z
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

# T-2548: Charter non-goal #4: conversation-analytics MCP tool family subtract-vs-keep

## Problem Statement

TermLink's charter (docs/CHARTER.md) names a hard NON-GOAL #4: *the substrate is
mechanism, not policy.* The T-2468 purpose review's mandate is
"subtract-and-deepen." T-2471/T-2478 already pruned 52 off-charter
social-analytics MCP tools (reactions/emoji/stars/pins/typing/polls). This
inception surfaces the **surviving sibling sub-family**: ~30-40 LIVE (NOT
deprecated) conversation-analytics MCP tools — thread-health scores,
reply/starter/pinner leaderboards, engagement-rate, response-latency and message
volume/growth distributions — that are *product analytics on top of the message
substrate*, tracing to none of the four charter verbs (discover / exchange
durable messages / claim work / control terminal sessions). Verified LIVE via
`is_deprecated()` and found to have **zero first-party callers**. Now, because
this is the largest remaining off-charter tool surface the campaign has surfaced
after orchestrator.route (T-2540). The subtract-vs-keep call is a human
sovereignty product-identity decision.

Research artifact: `docs/reports/T-2548-conversation-analytics-non-goal-4.md` (C-001).

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

- A-1: The named analytics tools are LIVE (not deprecated) in the MCP registry.
  (verify: `is_deprecated()` in crates/termlink-mcp/src/tools.rs)
- A-2: Zero first-party callers exist (no scripts/, .claude/commands/, or CLI use).
- A-3: They analyze message data (leaderboards/scores/distributions) rather than
  retrieve messages — i.e. policy/analytics, not the "exchange durable messages"
  mechanism. (Retrieval tools like agent_search_thread/agent_thread_path are OUT.)
- A-4 (UNVERIFIABLE here): no EXTERNAL peer/AEF process calls these MCP tools
  (T-559 cross-project boundary). This is the removal GATE.

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

- **IW-1: Are there any EXTERNAL (non-first-party) consumers of these analytics
  MCP tools across the fleet or the AEF layer?**
  confidence: 1
  disposition: deferred
  rationale: In-repo first-party callers = zero (to verify in exploration).
  External callers cannot be confirmed from /opt/termlink (T-559 boundary). This
  is the removal GATE — a human or cross-project session must clear it before any
  subtract lands. Same gate shape as T-2540 IW-1.

- **IW-2: Subtract-all vs keep-a-curated-few — is the whole family dead weight, or
  do a handful (e.g. agent_stats, agent_digest) earn their keep as operator
  triage aids that a human actually reaches for?**
  confidence: 1
  disposition: deferred
  rationale: Zero first-party callers argues subtract-all, but the human may want
  to retain a small triage subset. This is part of the GO scoping decision.

- **IW-3: Subtract vs grandfather — does the charter get amended for a sanctioned
  analytics exception, or does the family leave the substrate?**
  confidence: 2
  disposition: deferred
  rationale: Human sovereignty product-identity decision. Agent recommendation =
  GO to subtract (restore non-goal #4), mirroring the T-2471/T-2478 precedent for
  the sibling social-analytics set.

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

1. Verify A-1/A-3: read the tool catalog + `is_deprecated()` in
   crates/termlink-mcp/src/tools.rs; confirm the named tools are live analytics.
2. Verify A-2: grep scripts/, .claude/commands/, crates/termlink-cli for callers.
3. Clear IW-1 (GATE): cross-project consumer check — cannot be done here (T-559).
4. Human decides IW-3 (subtract-all / keep-curated / grandfather).

## Technical Constraints

- **T-559 project-boundary:** this session cannot grep /opt/999-AEF or peer repos
  for external MCP-tool callers — the GATE check (IW-1) is structurally
  cross-project and human-gated.
- **Consequential:** these are LIVE advertised MCP tools; removing them is a
  breaking change for any (unknown) external caller — hence gate-before-removal.
- **Precedent:** T-2471 (delete) + T-2478 (deprecate) already subtracted the
  sibling social-analytics set (reactions/emoji/stars/pins/typing/polls) via the
  `remote_inbox_*` T-1166 deprecation pattern. This is the same operation on the
  surviving thread-metrics/leaderboard/engagement sub-family.

## Scope Fence

**IN scope (this inception):** confirm the tools are LIVE analytics (not
deprecated, not message-retrieval), confirm zero first-party callers, frame the
subtract-vs-keep decision with evidence.
**OUT of scope:** the actual removal (a GO build task), the cross-project
external-consumer check (IW-1 gate, human/cross-project), any charter amendment,
and message-*retrieval* tools (agent_search_thread / agent_thread_path /
agent_recent_window) which defensibly trace to "exchange durable messages" and
must be KEPT.

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

**GO (subtract) if:**
- Charter non-goal #4 is to remain a load-bearing line, AND
- The IW-1 gate clears — the fleet-wide external-consumer check finds NO peer/AEF
  process calling these analytics tools (first-party callers already verified
  zero). Then: deprecate-then-delete the family via the T-1166 `remote_inbox_*`
  pattern (mirror T-2471/T-2478), optionally retaining a curated triage subset.

**NO-GO (grandfather) if:**
- The human judges the analytics surface strategically worth keeping → amend
  `docs/CHARTER.md` for a sanctioned exception.

**INTERMEDIATE (deprecate-then-remove) if:**
- IW-1 cannot be cleared quickly → mark the family deprecated now (T-1166
  pattern), soak one release, then remove.

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

**Recommendation:** GO (subtract), gated on IW-1 external-consumer check.

**Rationale:** Verified in code (2026-08-08, crates/termlink-mcp/src/tools.rs):
a family of conversation-analytics MCP tools is LIVE (`is_deprecated()` returns
false — confirmed for agent_thread_health, agent_busiest_threads,
agent_response_latency, agent_stats, and ~30 more per the T-2468 off-charter
re-sweep) and has ZERO first-party callers (grep of scripts/, .claude/commands/,
crates/termlink-cli returned empty). These analyze message data
(leaderboards/scores/latency & volume distributions) — product analytics on top
of the substrate, tracing to none of the four charter verbs, i.e. a direct
non-goal #4 ("substrate stays mechanism, not policy") violation. They are the
surviving sibling of the social-analytics set T-2471/T-2478 already pruned. GO to
subtract restores the charter and removes advertised-but-uncalled surface. Human
owns the subtract-vs-grandfather product decision; the removal must be gated on a
cross-project external-consumer check (IW-1) the T-559 boundary blocks here.

**Evidence:**
- LIVE (not deprecated): agent_thread_health, agent_busiest_threads,
  agent_response_latency, agent_stats confirmed no deprecation marker in desc.
- Zero first-party callers: `grep -rl <tools> scripts/ .claude/commands/
  crates/termlink-cli/src` → empty.
- Charter trace: FAILS all four verbs (analytics, not discover/exchange/claim/
  control). Precedent: T-2471 delete + T-2478 deprecate on the sibling set.
- KEEP-list (message retrieval, NOT analytics): agent_search_thread,
  agent_thread_path, agent_recent_window — trace to "exchange durable messages".
- Full family enumeration + live/deprecated split is GO-build scope.

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
