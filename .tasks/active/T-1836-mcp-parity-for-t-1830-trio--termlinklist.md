---
id: T-1836
name: "MCP parity for T-1830 trio — termlink_listener_heartbeat / termlink_agent_listeners / termlink_agent_send_auto_discover (shell-out)"
description: >
  MCP parity for T-1830 trio — termlink_listener_heartbeat / termlink_agent_listeners / termlink_agent_send_auto_discover (shell-out)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T13:32:41Z
last_update: 2026-05-28T13:32:41Z
date_finished: null
---

# T-1836: MCP parity for T-1830 trio — termlink_listener_heartbeat / termlink_agent_listeners / termlink_agent_send_auto_discover (shell-out)

## Context

PL-185 named the decision point: shell-script verbs (T-1832/33/34) have no MCP wrappers, so LLM-driven agents (cohort-agent, penelope, claude-code) can only call them via Bash. This task picks option (b) — MCP shells out to the bash scripts via tokio::process — matching the T-1689 precedent (`termlink_fleet_bootstrap_check` subprocesses `fleet bootstrap-check --json`). Three new tools: `termlink_listener_heartbeat` (emit one heartbeat, --once mode), `termlink_agent_listeners` (discovery reader), `termlink_agent_send_auto_discover` (auto-resolve agent_id → topic+pty_session, supports --dry-run). All read existing scripts under `${TERMLINK_SCRIPTS_DIR:-/opt/termlink/scripts}`; no Rust CLI changes.

## Acceptance Criteria

### Agent
- [ ] Three new MCP tools registered: `termlink_listener_heartbeat`, `termlink_agent_listeners`, `termlink_agent_send_auto_discover`
- [ ] Each subprocesses the corresponding `scripts/*.sh` via `tokio::process::Command` with `kill_on_drop(true)` + `stdin(Stdio::null())` + bounded `timeout_secs` (default 10, clamped 1..=120)
- [ ] Script path resolution: env var `TERMLINK_SCRIPTS_DIR` overrides; default `/opt/termlink/scripts`; missing script returns `json_err` with the path tried
- [ ] `termlink_listener_heartbeat`: always passes `--once` (never loops); params `agent_id` (required), `role`, `listen_topics` (Vec<String>), `pty_session`, `topic`, `interval_secs`, `hub`; returns stdout/stderr + exit_code wrapped in JSON
- [ ] `termlink_agent_listeners`: params `topic` (default `agent-presence`), `hub`, `filter_agent_id`, `include_offline` (bool), `min_age_secs`/`max_age_secs`; passes `--json`; returns parsed envelope decorated with `ok` + `exit_code`
- [ ] `termlink_agent_send_auto_discover`: params `to_agent_id` (required), `message`, `dry_run` (bool), `hub`; always passes `--to <id>` and refuses if to-session/topic/peer-fp would be needed; returns subprocess output
- [ ] Tests under `#[cfg(test)]` cover: missing script error path, timeout handling, dry-run happy path with synthetic env, parameter forwarding
- [ ] `cargo build -p termlink-mcp` passes
- [ ] `cargo test -p termlink-mcp t1836` passes
- [ ] Tool descriptions reference T-1830/T-1836 and PL-185 so MCP introspection surfaces the lineage

### Human
- [ ] [RUBBER-STAMP] MCP listing shows the three new tools
  **Steps:**
  1. `termlink mcp` or a connected MCP client shows tools containing `agent_listeners` / `listener_heartbeat` / `agent_send_auto_discover`
  2. Or grep the built binary: `cargo build -p termlink-mcp --release && ./target/release/termlink mcp --list-tools 2>&1 | grep -E 'listener_heartbeat|agent_listeners|agent_send_auto_discover'`
  **Expected:** All three names appear
  **If not:** Build failed or registration missing — re-run cargo build with -vv

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

cargo build -p termlink-mcp 2>&1 | tail -5
cargo test -p termlink-mcp t1836 2>&1 | tail -10
grep -q "termlink_listener_heartbeat" crates/termlink-mcp/src/tools.rs
grep -q "termlink_agent_listeners" crates/termlink-mcp/src/tools.rs
grep -q "termlink_agent_send_auto_discover" crates/termlink-mcp/src/tools.rs

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

### 2026-05-28T13:32:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1836-mcp-parity-for-t-1830-trio--termlinklist.md
- **Context:** Initial task creation
