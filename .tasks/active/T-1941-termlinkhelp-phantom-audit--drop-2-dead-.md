---
id: T-1941
name: "termlink_help phantom audit — drop 2 dead entries + add regression test"
description: >
  Two help-registry entries reference tools that do not exist as real MCP tools — LLM consumers calling them get tool-not-found errors. Drop the phantoms and add a unit test that walks the registry against the real tool name table so this class of bug cannot recur.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T15:40:30Z
last_update: 2026-06-03T15:40:30Z
date_finished: null
---

# T-1941: termlink_help phantom audit — drop 2 dead entries + add regression test

## Context

Audit of `termlink_help` registry against the real `#[tool(name = ...)]` macro
table found 2 phantom entries that point to tools that do not exist:

- `termlink_agent_forward` — likely a stale name; real tool is `termlink_agent_forwards_of`
  (reads, not writes — "Re-publish a post" description does not match)
- `termlink_agent_recent_dm` — never implemented; users should use `/recent-dm` skill or
  `termlink_channel_subscribe` against a dm:* topic

An LLM consumer that discovers these via `termlink_help` and tries to call them
gets a tool-not-found error from the MCP router. Drop the phantoms and add a
unit test that asserts every help entry exists in the real tool name registry,
so this class of bug cannot regress.

## Acceptance Criteria

### Agent
- [ ] Drop the 2 phantom entries (`termlink_agent_forward`, `termlink_agent_recent_dm`) from the help registry in `tools.rs`
- [ ] Add a unit test in `mod tests` that walks the full help registry (via `help_categories()` or equivalent) and asserts every `(name, _)` tuple is present in a fixture set of real tool names
- [ ] `cargo test -p termlink-mcp --lib` passes (all pre-existing + new test green)
- [ ] `cargo build -p termlink-mcp` is warning-free

## Verification

cd /opt/termlink && cargo test -p termlink-mcp --lib help_ 2>&1 | tail -20 | grep -E "test result.*0 failed" 

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

### 2026-06-03T15:40:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1941-termlinkhelp-phantom-audit--drop-2-dead-.md
- **Context:** Initial task creation
