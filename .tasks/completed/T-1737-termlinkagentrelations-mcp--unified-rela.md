---
id: T-1737
name: "termlink_agent_relations MCP — unified relations report (replies + reactions
  + edits + redactions)"
description: >
  termlink_agent_relations MCP — unified relations report (replies + reactions + edits
  + redactions)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-05-21T06:17:39Z
last_update: '2026-08-18T18:58:55Z'
date_finished: 2026-05-21T06:20:59Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:11Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:55Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-1737: termlink_agent_relations MCP — unified relations report (replies + reactions + edits + redactions)

## Context

MCP parity for CLI `agent relations <OFFSET>` (T-1381). Matrix Client API `/relations/{eventId}` analogue — per-target consolidation of the four canonical relation types: replies (`m.in_reply_to`), reactions (`m.annotation`), edits (`m.replace`), redactions (`m.redaction`). Forwards excluded (cross-topic, requires multi-topic walk). Read-only; ports `compute_relations` (channel.rs:6065, ~115 LOC) + `RelationsReport` + `RelationItem` structs. Reuses `redacted_offsets_mcp`, `parent_offset_of_mcp`, `decode_payload_lossy_mcp`. High-value composite navigation tool for MCP-aware agents — answers "show me everything attached to this post".

## Acceptance Criteria

### Agent
- [x] `RelationItemMcp` + `RelationsReportMcp` structs mirror CLI's `RelationItem` and `RelationsReport` one-to-one
- [x] `compute_relations_mcp` ported; preserves four-way partition (edit→replaces, redaction→redacts, reaction→in_reply_to, else→in_reply_to as reply), filters relation envelopes whose own offset is in the redaction set, captures `target_sender` / `target_payload` from the target row when present (else empty strings), sorts each list ts_ms asc + offset asc tiebreak
- [x] `AgentRelationsParams { target: u64 }` and `termlink_agent_relations` tool method added; walks `agent-chat-arc` via `walk_topic_full_mcp`; returns error JSON when target offset is not present in the snapshot
- [x] Returns `{ok, topic, target_offset, target_sender, target_payload, replies, reactions, edits, redactions}` JSON
- [x] ≥7 new unit tests in `tools::tests` covering: empty input, target-not-present yields empty target fields but still returns report, all four relation types in one shot, redaction list captures `metadata.reason` as payload, edit `replaces` mismatch filtered, reaction with `in_reply_to` not pointing at target filtered, sort by ts_ms ascending with offset tiebreak, params deserialize
- [x] `cargo build -p termlink-mcp` clean (no new warnings beyond pre-existing `cur_run_end`)
- [x] `cargo test -p termlink-mcp` passes (all new tests + no regressions)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
cargo build -p termlink-mcp 2>&1 | grep -E '^(error|warning):' | grep -vE 'cur_run_end|generated [0-9]+ warning' | grep -q . && exit 1 || exit 0
cargo test -p termlink-mcp --quiet 2>&1 | tail -5 | grep -q "test result: ok"

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
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

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-21T06:17:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1737-termlinkagentrelations-mcp--unified-rela.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-84e9e609
- **Timestamp:** 2026-05-21T06:21:14Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T06:20:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
