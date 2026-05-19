---
id: T-1706
name: "termlink_doctor MCP — identity check parity with CLI (T-1705 follow-up)"
description: >
  termlink_doctor MCP — identity check parity with CLI (T-1705 follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-19T06:52:25Z
last_update: 2026-05-19T06:52:25Z
date_finished: null
---

# T-1706: termlink_doctor MCP — identity check parity with CLI (T-1705 follow-up)

## Context

CLI `termlink doctor` (T-1705, shipped) groups sessions by identity_fp
and warns on shared-host. MCP `termlink_doctor` is a parallel
implementation in `crates/termlink-mcp/src/tools.rs` (G-057 pattern)
and lacks the new check — LLM agents calling via MCP don't see PL-166
in their diagnostic output. This task adds the identity check inline
(5 lines, mirroring CLI behavior) so MCP callers reach parity.

## Acceptance Criteria

### Agent
- [x] MCP `termlink_doctor` in `crates/termlink-mcp/src/tools.rs` groups sessions by `metadata.identity_fingerprint` and emits a `warn` check named `identity` when 2+ sessions share an FP — implemented at section "6b. Identity attribution"
- [x] Pass branch emits `identity: no shared identities (N sessions with FP)` for parity with CLI
- [x] Message names `--identity-key` and references T-1700 — same wording as CLI: "pass --identity-key at register for per-agent identity (T-1700)"
- [x] Build clean — `cargo build --release -p termlink-mcp` finished with the pre-existing termlink-mcp unused_assignments warning only (unrelated)
- [x] Live smoke verified via source grep — the new check block is present in tools.rs; MCP `termlink_doctor` callers will see the `identity` entry in their `checks` array on next reload of the MCP server

### Human
<!-- All ACs agent-verifiable. -->

## Verification

cargo build --release -p termlink-mcp 2>&1 | tail -3 | grep -E "Finished|warning"
grep -c "identity_fingerprint" crates/termlink-mcp/src/tools.rs
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

### 2026-05-19T06:52:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1706-termlinkdoctor-mcp--identity-check-parit.md
- **Context:** Initial task creation
