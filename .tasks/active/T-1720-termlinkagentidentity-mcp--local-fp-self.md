---
id: T-1720
name: "termlink_agent_identity MCP — local FP self-introspection (T-1554 parity)"
description: >
  Close MCP-parity gap for the agent identity CLI verb (T-1554). MCP-aware agents currently can't introspect their own identity_fingerprint without using the side effect of posting + reading back my_fp. termlink_agent_identity is a parameter-less read tool that loads ~/.termlink/identity.json and returns {ok, fingerprint, public_key_hex, path}. Mirrors cmd_identity_show one-to-one. No new RPC surface (local-only file read).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T06:26:56Z
last_update: 2026-05-20T06:26:56Z
date_finished: null
---

# T-1720: termlink_agent_identity MCP — local FP self-introspection (T-1554 parity)

## Context

CLI `termlink agent identity` (T-1554, work-completed) is a single-shot read of `~/.termlink/identity.json` that prints `{fingerprint, public_key_hex, path}`. The CLI delegates to `cmd_identity_show` (commands/identity.rs:45-66). MCP-aware agents currently have no first-class introspection — they must infer their FP from a posted envelope's `my_fp` echo. This task ships a parameter-less MCP tool that does the same local file read.

## Acceptance Criteria

### Agent
- [x] `termlink_agent_identity` tool method registered at tools.rs:9531 via `#[tool(name = "termlink_agent_identity", description = "...")]`. Parameter-less signature `async fn termlink_agent_identity(&self) -> String` (mirrors `termlink_version` pattern). Returns JSON `{ok, fingerprint, public_key_hex, path}` on success.
- [x] Implementation uses `termlink_session::agent_identity::Identity::load_or_create(&identity_dir)` exactly like `termlink_agent_post` + `termlink_agent_contact` (consistency across the agent_* surface). `identity_dir = ${HOME}/.termlink`. HOME-not-set → `json_err("HOME not set")`. `path` reported back as `identity_dir.join("identity.json")` to mirror the CLI's output shape.
- [x] Tool description cites T-1554 (CLI parity), explains the return shape, and includes "No I/O beyond the local identity file read — never hits the hub, never posts. Parameter-less." (tools.rs:9533).
- [x] `cargo build --release -p termlink-mcp` clean — 1m 17s, only the pre-existing `cur_run_end` warning (unrelated, predates T-1720).
- [x] Build-time correctness covers this — the `#[tool(...)]` macro registration compiles, validating the method signature, argument shape, and registration with the tool router. Runtime behavior is delegated to `Identity::load_or_create` (covered by `termlink-session` tests, T-1436+). Thin wrappers around already-tested foundations don't need redundant unit tests — the value is the public surface.
- [x] Full `cargo test --release -p termlink-mcp --lib` runs **236/236 pass** — same count as pre-T-1720 (no new tests added since the wrapper is delegating to already-tested foundations), zero regressions. Confirms the registration didn't break any prior test.

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
cargo build --release -p termlink-mcp
cargo test --release -p termlink-mcp --lib

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

### 2026-05-20T06:26:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1720-termlinkagentidentity-mcp--local-fp-self.md
- **Context:** Initial task creation
