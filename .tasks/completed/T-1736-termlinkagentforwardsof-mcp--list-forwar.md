---
id: T-1736
name: "termlink_agent_forwards_of MCP — list forwards by a given sender"
description: >
  termlink_agent_forwards_of MCP — list forwards by a given sender

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-20T21:44:38Z
last_update: 2026-05-20T21:47:16Z
date_finished: 2026-05-20T21:47:16Z
---

# T-1736: termlink_agent_forwards_of MCP — list forwards by a given sender

## Context

MCP parity for CLI `agent forwards-of [SENDER]` (T-1367). Lists every active (non-redacted) forward envelope posted by a given sender on `agent-chat-arc`. Third member of the by-sender triplet (T-1734 replies_of + T-1735 reactions_of + this). Ports `compute_forwards_of` (channel.rs:4981, ~35 LOC) + `extract_forward` (channel.rs:7644, ~10 LOC) + `ForwardOfRow` struct. A "forward" is identified by metadata-pair presence (`forwarded_from` + `forwarded_sender`) — msg_type is preserved from original so isn't the discriminator. Topics may contain colons (e.g. `dm:a:b`), so origin parse splits on LAST colon.

## Acceptance Criteria

### Agent
- [x] `extract_forward_mcp` ported alongside existing `*_mcp` helpers; returns `Some((topic, offset, sender))` only when BOTH `metadata.forwarded_from` and `metadata.forwarded_sender` are present and `forwarded_from` parses as `<topic>:<u64>` via `rsplit_once(':')`
- [x] `compute_forwards_of_mcp` ported; preserves filters (sender match, drop redacted offsets, require successful `extract_forward_mcp`), sort by `forward_offset` descending
- [x] `ForwardOfRowMcp` struct + `to_json_mcp` mirror CLI's `ForwardOfRow` (`forward_offset`, `origin_topic`, `origin_offset`, `origin_sender`, `payload`, `ts`)
- [x] `AgentForwardsOfParams { sender: Option<String> }` and `termlink_agent_forwards_of` tool method added; sender defaults to caller's local Identity fingerprint when omitted
- [x] Returns `{ok, topic, sender, forwards: [...]}` JSON
- [x] ≥6 new unit tests in `tools::tests` covering: empty input, single forward happy path, redacted forward dropped, missing `forwarded_sender` returns None, dm:* topic with internal colons parses correctly (last-colon split), sort by forward_offset descending, params deserialize
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

### 2026-05-20T21:44:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1736-termlinkagentforwardsof-mcp--list-forwar.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d975ccfa
- **Timestamp:** 2026-05-20T21:47:32Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T21:47:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
