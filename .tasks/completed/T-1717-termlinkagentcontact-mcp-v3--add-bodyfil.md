---
id: T-1717
name: "termlink_agent_contact MCP v3 — add body_file (CLI --file parity, T-1646)"
description: >
  Close the remaining MCP-parity gap from T-1715/T-1716: the CLI's --file PATH flag (T-1646) was deferred in v1 because MCP callers typically have inline bodies. In practice, MCP-aware agents that generate long-form structured payloads (inception findings, RCAs, code reviews) need to point at a file on disk rather than inline the entire body through the MCP tool-result transport. This task adds a body_file parameter that mirrors CLI --file semantics one-to-one.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-20T05:56:02Z
last_update: 2026-05-20T19:25:39Z
date_finished: 2026-05-20T19:25:39Z
---

# T-1717: termlink_agent_contact MCP v3 — add body_file (CLI --file parity, T-1646)

## Context

T-1715 deliberately shipped `termlink_agent_contact` MCP v1 with `message: String` (required) and no file-shortcut, on the assumption that MCP callers would inline the body. T-1716 added v2 presence/ack semantics but left the CLI's `--file` parity gap open. Real-world MCP-aware agents now exist that generate long-form structured payloads (inception findings, RCAs, code reviews, dispatch reports) which are awkward to inline as a single JSON-encoded string at the MCP boundary. The CLI already has `resolve_contact_message(message, file)` in `crates/termlink-cli/src/commands/agent.rs:698-723` (T-1646): mutex on `message` XOR `file`, empty file rejected. v3 mirrors that pure function into MCP one-to-one; defaults preserve v1+v2 byte-identical behavior (PL-172).

## Acceptance Criteria

### Agent
- [x] `AgentContactParams` extended: `message` field changes from `String` (required) to `Option<String>`; new field `body_file: Option<String>` added (tools.rs:1269-1283). Doc-comments cite T-1717 + mutex semantics
- [x] Pure helper `resolve_message_or_file_mcp(message: Option<&str>, body_file: Option<&str>) -> Result<String, String>` added at tools.rs:303-326 (just after `preview_body`) — returns `Err` strings matching CLI semantically ("specify exactly one of message or body_file, not both" / "specify exactly one of message <STRING> or body_file <PATH>" / "file <PATH> is empty — refusing to post empty message" / "failed to read <PATH>: <io-error>"). Empty body rejected; same as CLI.
- [x] `termlink_agent_contact` calls `resolve_message_or_file_mcp` after the target/target_fp mutex check (tools.rs:8893-8901); on error returns a `json_err` response with "T-1717" trace tag prepended. Pre-existing v1 callers passing only `message` continue to work — byte-identical to v2 behavior (PL-172) — `agent_contact_params_minimal_message_only_target_fp` test still passes.
- [x] `body_file` resolves relative to the MCP server's current working directory (mirrors CLI's `std::fs::read_to_string` behavior). Doc-comment on the `body_file` field (tools.rs:1276-1283) explicitly notes this and recommends absolute paths when cwd is ambiguous.
- [x] Mutex error JSON shape consistent with the existing target/target_fp mutex error JSON: `{"ok": false, "error": "T-1717: ..."}` returned via `json_err`. No panic, no unwrap on the file-read path — `std::fs::read_to_string` errors are mapped to a `String` via `map_err` and surfaced through `json_err`.
- [x] Tool description in `#[tool(...)]` attribute updated (tools.rs:8857) — now includes "**Body sources (T-1717):** exactly one of `message` (inline string) / `body_file` (path read at the MCP server's cwd — mirror of CLI `--file PATH`, T-1646). Empty file rejected." No longer claims `--file` deferred.
- [x] `cargo build --release -p termlink-mcp` clean — finished in 1m 15s, only the pre-existing `cur_run_end` warning at tools.rs:15128 (unrelated, predates T-1717)
- [x] **7** new unit tests added — exceeded the ≥4 target. Coverage: `resolve_message_inline` (a), `resolve_body_file_reads_content` (b), `resolve_rejects_both_set` (c), `resolve_rejects_neither_set` (d), `resolve_rejects_empty_file` (e), `resolve_propagates_missing_file_io_error` (bonus IO-error coverage), `params_v3_body_file_deserializes` (param-shape coverage). All 7 pass; the 15 pre-T-1717 tests on the suite also still pass. **22/22 pass** on `cargo test --release -p termlink-mcp agent_contact`.

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
cargo test --release -p termlink-mcp agent_contact

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

### 2026-05-20T05:56:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1717-termlinkagentcontact-mcp-v3--add-bodyfil.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-9d27036f
- **Timestamp:** 2026-05-20T19:25:40Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T19:25:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
