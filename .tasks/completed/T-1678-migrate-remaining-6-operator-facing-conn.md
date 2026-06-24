---
id: T-1678
name: "Migrate remaining 6 operator-facing connect_addr sites to connect_addr_with_timeout (T-1677 follow-up)"
description: >
  Migrate remaining 6 operator-facing connect_addr sites to connect_addr_with_timeout (T-1677 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-17T21:57:34Z
last_update: 2026-05-17T22:06:49Z
date_finished: 2026-05-17T22:06:49Z
---

# T-1678: Migrate remaining 6 operator-facing connect_addr sites to connect_addr_with_timeout (T-1677 follow-up)

## Context

T-1677 added `Client::connect_addr_with_timeout` and migrated `connect_remote_hub` (the entry point used by `termlink remote *`). Survey of the other six `connect_addr` call sites revealed that only TCP-target sites actually benefit (Unix-socket connects don't trigger the OS TCP retry budget — they fail immediately if the path doesn't exist). Reduced scope to the two real-impact sites.

Migrated:
- `crates/termlink-cli/src/commands/channel.rs:250` — `rpc_call_authed` (chat-arc post/list, TCP path; Unix path already short-circuits to `rpc_call_addr`)
- `crates/termlink-mcp/src/tools.rs:301` — MCP `termlink_remote_call`/equivalent TCP-hub connect

Out of scope (no value):
- `crates/termlink-cli/src/commands/file.rs:114, 424` — both Unix sockets
- `crates/termlink-mcp/src/tools.rs:4208, 4386` — both Unix sockets
- `crates/termlink-session/src/inbox_channel.rs` — internal helpers; defer

## Acceptance Criteria

### Agent
- [x] The 2 TCP-facing sites use `Client::connect_addr_with_timeout(addr, Duration::from_secs(10))`
- [x] `cargo check --workspace` passes
- [x] No remaining unbounded `Client::connect_addr(&addr)` where `addr` is TCP at the four audited paths (channel.rs:250 and tools.rs:301); Unix-socket sites left as-is by design
- [x] Live smoke: chat-arc command against blackhole hub exits in <12s with timeout error

## Verification

cargo check --workspace
bash -c "grep -q 'connect_addr_with_timeout' crates/termlink-cli/src/commands/channel.rs"
bash -c "grep -q 'connect_addr_with_timeout' crates/termlink-mcp/src/tools.rs"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-05-17T21:57:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1678-migrate-remaining-6-operator-facing-conn.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-20867553
- **Timestamp:** 2026-05-17T22:06:57Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T22:06:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
