---
id: T-1694
name: "Expose envelope metadata on termlink_channel_post MCP tool (implements T-1692 Shape 1)"
description: >
  Implementation of T-1692 GO recommendation. Add metadata: Option<serde_json::Value> to ChannelPostParams in crates/termlink-mcp/src/tools.rs, pass-through to the post envelope. Free-form, no schema enforcement at tool layer (Shape 1). Unblocks cohort-agent n8n exec 7.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-18T09:27:54Z
last_update: 2026-05-18T09:34:36Z
date_finished: 2026-05-18T09:34:36Z
---

# T-1694: Expose envelope metadata on termlink_channel_post MCP tool (implements T-1692 Shape 1)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `ChannelPostParams` gains `metadata: Option<HashMap<String, String>>` field with JsonSchema doc
- [x] `termlink_channel_post` handler pass-through: when `metadata` is Some and non-empty, inserts the `metadata` key into the JSON-RPC params sent to the hub
- [x] No-metadata callers see unchanged behavior (the key is absent from RPC params, hub treats as empty BTreeMap)
- [x] Hub-side handler at `crates/termlink-hub/src/channel.rs:466` already parses `params["metadata"]` into the envelope — no hub change needed (verified by reading the existing code)
- [x] Unit tests (4): deserializes-when-present, optional-when-omitted, empty-map-distinct-from-none, round-trip-json-shape
- [x] `cargo build --release -p termlink-mcp` clean
- [x] `cargo test --release -p termlink-mcp --lib` 123/123 passed (119 prior + 4 new — no regression)

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

cargo build --release -p termlink-mcp 2>&1 | tail -3
cargo test --release -p termlink-mcp --lib 2>&1 | grep -q "test result: ok"
grep -q "pub metadata: Option" crates/termlink-mcp/src/tools.rs

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

### 2026-05-18T09:27:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1694-expose-envelope-metadata-on-termlinkchan.md
- **Context:** Initial task creation

### 2026-05-18T09:28:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-e05a3387
- **Timestamp:** 2026-05-18T09:38:17Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#4 (Agent)** — Hub-side handler at `crates/termlink-hub/src/channel.rs:466` already parses `params["metadata"]` into the envelope — no hub change needed (verified by reading the existing code)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-hub/src/channel.rs in: Hub-side handler at `crates/termlink-hub/src/channel.rs:466` already parses `params["metadata"]` into the envelope — no hub change needed (verified by`

### 2026-05-18T09:34:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
