---
id: T-1918
name: "termlink_list_sessions MCP shape — align to CLI envelope {ok,sessions} (parity v0.3 catch)"
description: >
  termlink_list_sessions MCP shape — align to CLI envelope {ok,sessions} (parity v0.3 catch)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T19:51:26Z
last_update: 2026-06-01T19:51:26Z
date_finished: null
---

# T-1918: termlink_list_sessions MCP shape — align to CLI envelope {ok,sessions} (parity v0.3 catch)

## Context

Manual diff between MCP and CLI:
- **CLI** `termlink list --json` returns: `{"ok": true, "sessions": [{...}, {...}]}`
- **MCP** `termlink_list_sessions` returns: `[{...}, {...}]` (bare array, no envelope)

Same shape-divergence class as T-1910 (topics), T-1912 (version). Operator asking "what sessions exist" via MCP gets a different JSON shape than via CLI. Tools/clients that consume both surfaces break or need conditional unwrapping.

Convergence direction: align MCP to CLI envelope. CLI's `{ok, sessions}` matches the ecosystem-wide hub-RPC envelope convention. Bare-array MCP return is the outlier.

## Acceptance Criteria

### Agent
- [x] MCP `termlink_list_sessions` (tools.rs ~line 7858) wraps its return in `{"ok": true, "sessions": [...]}` instead of returning the bare array.
- [x] Error path (line 7896 `json_err(e)`) unchanged — already returns `{ok: false, error: ...}` envelope.
- [x] New parity test `parity_list_sessions` added to `crates/termlink-mcp/tests/parity.rs`. Includes explicit `ok==true` + `sessions.is_array()` asserts plus structural diff.
- [x] `cargo build -p termlink --release` succeeds (6m 49s).
- [x] `cargo test --release --test parity -p termlink-mcp -- --test-threads=1`: `test result: ok. 8 passed; 0 failed; 1 ignored` (was 7 + 1 ignored; parity_list_sessions is the new green).

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

cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -3 | grep -qE "test result: ok\. [0-9]+ passed; 0 failed"

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

### 2026-06-01T19:51:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1918-termlinklistsessions-mcp-shape--align-to.md
- **Context:** Initial task creation
