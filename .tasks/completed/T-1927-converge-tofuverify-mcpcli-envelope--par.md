---
id: T-1927
name: "Converge tofu_verify MCP/CLI envelope + parity test (PL-198 follow-up)"
description: >
  MCP and CLI tofu_verify diverge in 4 fields: status=probe-fail vs probe-failed, error vs probe_error, MCP missing match:bool, MCP has ok+actions CLI lacks. Align MCP field names to CLI, add ok+actions to CLI, add parity_tofu_verify_no_pin test.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-02T14:57:41Z
last_update: 2026-06-02T14:57:41Z
date_finished: null
---

# T-1927: Converge tofu_verify MCP/CLI envelope + parity test (PL-198 follow-up)

## Context

Follow-up to T-1911 / T-1926 (PL-198 + PL-199 + parity harness). MCP
`termlink_tofu_verify` and CLI `termlink tofu verify <host> --json`
both probe a hub's TLS fingerprint and diff vs `~/.termlink/known_hubs`
— same logical operation, drifted serialization. Census:

| Field | MCP | CLI |
|---|---|---|
| `ok` | yes (`status=="match"`) | NO |
| `status` | `"match"`/`"drift"`/`"no-pin"`/**`"probe-fail"`** | `"match"`/`"drift"`/`"no-pin"`/**`"probe-failed"`** |
| `error` | yes (probe error msg) | NO |
| `probe_error` | NO | yes (same content, different name) |
| `match` (bool/null) | NO | yes |
| `actions` (heal hints) | yes ([] or 2 entries) | NO |
| `address` | yes | yes |
| `wire` | yes | yes |
| `pinned` | yes | yes |

Plan: align MCP field names to CLI (`probe-fail` → `probe-failed`,
`error` → `probe_error`), add `match` to MCP, add `ok` + `actions` to
CLI. Net result: both emit identical shape. Add `parity_tofu_verify_no_pin`
test using `127.0.0.1:1` (fast-fail ECONNREFUSED) — both sides report
`status=probe-failed` with same envelope.

## Acceptance Criteria

### Agent
- [x] MCP `termlink_tofu_verify` emits `status=probe-failed` (not `probe-fail`) — tools.rs:12581
- [x] MCP `termlink_tofu_verify` renames `error` → `probe_error` — tools.rs:12601
- [x] MCP `termlink_tofu_verify` includes `match: bool|null` field — tools.rs:12575-12582,12600
- [x] CLI `tofu verify --json` includes `ok: bool` top-level field — infrastructure.rs:1243-1252
- [x] CLI `tofu verify --json` includes `actions: []` field (populated on drift) — infrastructure.rs:1245-1250
- [x] New `parity_tofu_verify_no_pin` parity test passes — parity.rs:822-873 (1 passed; 0 failed; 0 ignored)
- [x] Full parity suite: 16 passed, 0 failed, 0 ignored (verified 2026-06-02, 383.86s wall)
- [x] `cargo build -p termlink -p termlink-mcp` succeeds with no new warnings (pre-existing tools.rs:23215 unused_assignments noted as separate)

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
cargo test --release --test parity -p termlink-mcp parity_tofu_verify -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. 1 passed; 0 failed; 0 ignored"
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

### 2026-06-02T14:57:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1927-converge-tofuverify-mcpcli-envelope--par.md
- **Context:** Initial task creation
