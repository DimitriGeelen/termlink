---
id: T-1821
name: "termlink_fleet_secrets_audit MCP parity — agent-callable security hygiene scan (T-1820 follow-up #2)"
description: >
  termlink_fleet_secrets_audit MCP parity — agent-callable security hygiene scan (T-1820 follow-up #2)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T07:45:28Z
last_update: 2026-05-28T07:48:52Z
date_finished: 2026-05-28T07:48:52Z
---

# T-1821: termlink_fleet_secrets_audit MCP parity — agent-callable security hygiene scan (T-1820 follow-up #2)

## Context

T-1820 shipped `termlink fleet secrets-audit` as a CLI verb (audits `~/.termlink/secrets/*.hex` for perms/format/orphan issues, closes G-011 item 4). This task adds the MCP companion `termlink_fleet_secrets_audit` so agents can run the same surveillance without shelling out — same pattern as T-1689 (`termlink_fleet_bootstrap_check`): subprocess the resolved `termlink` binary with `--json`, under `tokio::time::timeout` + `kill_on_drop=true` + null stdin, return the CLI's JSON envelope decorated with `ok` and `exit_code`.

## Acceptance Criteria

### Agent
- [x] `FleetSecretsAuditParams` struct added with `dir: Option<String>` and `timeout_secs: Option<u64>` (default 10, clamped 1..=120)
- [x] `termlink_fleet_secrets_audit` MCP tool added — subprocesses `termlink fleet secrets-audit --json [--dir <dir>]` under tokio timeout, kill_on_drop, null stdin
- [x] Returns the CLI JSON envelope decorated with `exit_code: i32` — the CLI's own `ok` field (true when zero warn-perms AND zero warn-format) is preserved verbatim from stdout; only parse/subprocess/timeout failures return `ok: false`
- [x] Timeout returns `{ok: false, verdict: "timeout", error: "..."}` shape (matches T-1689 pattern)
- [x] Tool registered in service trait (`#[tool(name = "termlink_fleet_secrets_audit", ...)]`)
- [x] 4 unit tests: defaults, dir override, timeout clamp, subprocess always returns parseable JSON
- [x] `cargo check -p termlink-mcp` passes
- [x] `cargo test -p termlink-mcp fleet_secrets_audit` passes (4/4)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
cd /opt/termlink && cargo check -p termlink-mcp 2>&1 | tail -5
cd /opt/termlink && cargo test -p termlink-mcp fleet_secrets_audit 2>&1 | tail -10

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

## Recommendation

**GO** — ship as-is.

Mechanical T-1820 follow-up. Same pattern as T-1689 (fleet_bootstrap_check):
subprocess the resolved binary with `--json`, kill_on_drop + null stdin,
forward the envelope decorated with `exit_code`. 4/4 unit tests pass.
Live-validated the underlying CLI JSON shape on the .107 secrets cache
(4 files: 3 ok + 1 info-orphan proxmox4.hex — the G-011 incident residue).

Agent-callable surveillance is the structural payoff: a watchtower agent
or scheduled MCP poller can now ask `termlink_fleet_secrets_audit` and
get a typed JSON answer about whether any secret file has slipped to
chmod 0o644 (G-011 item 4) — without shelling out, without an SSH hop,
without re-implementing the classifier. Mirrors the CLI verb 1:1.

## Updates

### 2026-05-28T07:45:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1821-termlinkfleetsecretsaudit-mcp-parity--ag.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-bc474de4
- **Timestamp:** 2026-05-28T07:49:04Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T07:48:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
