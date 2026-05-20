---
id: T-1733
name: "termlink_agent_thread MCP — read full thread tree by root offset"
description: >
  termlink_agent_thread MCP — read full thread tree by root offset

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T21:32:12Z
last_update: 2026-05-20T21:35:14Z
date_finished: 2026-05-20T21:35:14Z
---

# T-1733: termlink_agent_thread MCP — read full thread tree by root offset

## Context

MCP parity for CLI `agent thread <ROOT>` (T-1328, ships in T-1365 family). Reads the full conversation tree rooted at a specific offset on `agent-chat-arc`. Companion to T-1732 `termlink_agent_threads` (lists all roots) — answers "show me the conversation that started at offset N". Read-only; ports `build_thread` (channel.rs:2330) — a tiny pre-order DFS helper (~20 LOC) — and orchestrates a topic walk + parent→children indexing + root-exists check.

## Acceptance Criteria

### Agent
- [x] `build_thread_mcp` ported alongside existing `*_mcp` helpers; preserves pre-order DFS, ascending-offset child sort for determinism, depth tracking
- [x] `AgentThreadParams { root: u64 }` and `termlink_agent_thread` tool method added; walks `agent-chat-arc` via `walk_topic_full_mcp`; returns error JSON if root offset is not found
- [x] Returns `{ok, topic, root, thread: [{offset, depth, sender_id, msg_type, payload}, ...]}` JSON with payload base64-decoded lossy (via `decode_payload_lossy_mcp`)
- [x] ≥5 new unit tests in `tools::tests` covering: empty parents map, root-only no children, single chain depth 0/1/2, branching tree depth order, ascending-offset child sort
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

### 2026-05-20T21:32:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1733-termlinkagentthread-mcp--read-full-threa.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-b2de88b3
- **Timestamp:** 2026-05-20T21:35:30Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T21:35:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
