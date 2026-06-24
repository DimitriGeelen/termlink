---
id: T-1734
name: "termlink_agent_replies_of MCP — list all replies by a given sender"
description: >
  termlink_agent_replies_of MCP — list all replies by a given sender

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-20T21:36:31Z
last_update: 2026-05-20T21:39:45Z
date_finished: 2026-05-20T21:39:45Z
---

# T-1734: termlink_agent_replies_of MCP — list all replies by a given sender

## Context

MCP parity for CLI `agent replies-of [SENDER]` (T-1370). Lists every non-redacted, non-reaction reply by a given sender on `agent-chat-arc`, with parent context (sender + payload preview, best-effort when parent is present and not redacted). Defaults `sender` to caller's local identity fingerprint when omitted — same as CLI. Read-only; ports `compute_replies_of` (channel.rs:5102, ~55 LOC) along with `RepliesOfRow` struct. Reuses existing `*_mcp` helpers (`redacted_offsets_mcp`, `parent_offset_of_mcp` from T-1732, `decode_payload_lossy_mcp` from T-1730).

## Acceptance Criteria

### Agent
- [x] `compute_replies_of_mcp` ported alongside existing `*_mcp` helpers; preserves filters (sender match, drop redacted reply offset, drop msg_type=reaction, require non-null parent), best-effort parent_sender/parent_payload when parent absent or redacted, sort by `reply_offset` descending
- [x] `RepliesOfRowMcp` struct + `to_json_mcp` mirror CLI's `RepliesOfRow` (`reply_offset`, `parent_offset`, `parent_sender`, `parent_payload`, `reply_payload`, `ts_ms`)
- [x] `AgentRepliesOfParams { sender: Option<String> }` and `termlink_agent_replies_of` tool method added; sender defaults to caller's local Identity fingerprint when omitted (load via `Identity::load_or_create_default`)
- [x] Returns `{ok, topic, sender, replies: [...]}` JSON
- [x] ≥6 new unit tests in `tools::tests` covering: empty input, single reply happy path, redacted reply dropped, reactions filtered out, parent redacted → empty parent_payload, sort by reply_offset descending
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

### 2026-05-20T21:36:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1734-termlinkagentrepliesof-mcp--list-all-rep.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-fe0da026
- **Timestamp:** 2026-05-20T21:40:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T21:39:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
