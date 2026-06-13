---
id: T-1928
name: "Converge hub_probe MCP/CLI envelope + parity test (PL-198 follow-up)"
description: >
  MCP hub_probe emits {ok, address, fingerprint, error}; CLI emits {address, fingerprint} on success and non-JSON on failure. Add ok+error to CLI, structure CLI failure path as JSON, add parity_hub_probe test.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-02T15:28:13Z
last_update: 2026-06-02T15:28:13Z
date_finished: 2026-06-02T18:25:08Z
---

# T-1928: Converge hub_probe MCP/CLI envelope + parity test (PL-198 follow-up)

## Context

Follow-up to T-1927 (tofu_verify convergence). Same arc: another
read-only TLS probe with CLI/MCP envelope drift.

| Field | MCP success | MCP fail | CLI success | CLI fail |
|---|---|---|---|---|
| `ok` | `true` | `false` | MISSING | non-JSON (anyhow bail) |
| `address` | yes | yes | yes | n/a |
| `fingerprint` | `sha256:<hex>` | `null` | `sha256:<hex>` | n/a |
| `error` | `null` | `"<msg>"` | MISSING | n/a |

CLI failure path violates the `--json` contract — anyhow::bail prints
non-JSON to stderr and exits non-zero. Test rigs and agent consumers
can't parse the error.

Plan: align CLI to MCP shape — add `ok` + `error` fields, route the
failure path through `super::json_error_exit` (same pattern used in
cmd_inbox_status). Then add `parity_hub_probe` test using
`127.0.0.1:1` (fast-fail ECONNREFUSED) — both sides emit
`{ok: false, address, fingerprint: null, error: "<msg>"}`.

## Acceptance Criteria

### Agent
- [x] CLI `hub probe --json` emits `ok: bool` top-level field — infrastructure.rs:1007 (success) / 1023 (failure)
- [x] CLI `hub probe --json` emits `error: null|"<msg>"` field — infrastructure.rs:1010 / 1026
- [x] CLI `hub probe --json` failure path emits JSON (not anyhow bail) — uses super::json_error_exit
- [x] New `parity_hub_probe` parity test passes — parity.rs:822-877 (1 passed; 0 failed; 0 ignored, 376s incl. fresh build)
- [x] Full parity suite: 17 passed, 0 failed, 0 ignored (verified 2026-06-02, 342.21s wall)
- [x] `cargo build -p termlink -p termlink-mcp` succeeds with no new warnings (the existing tools.rs:23215 unused_assignments is pre-existing, not introduced here)

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
cargo test --release --test parity -p termlink-mcp parity_hub_probe -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. 1 passed; 0 failed; 0 ignored"
cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. [0-9]+ passed; 0 failed; 0 ignored"

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

### 2026-06-02T15:28:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1928-converge-hubprobe-mcpcli-envelope--parit.md
- **Context:** Initial task creation
