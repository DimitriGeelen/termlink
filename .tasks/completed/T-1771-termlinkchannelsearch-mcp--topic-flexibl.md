---
id: T-1771
name: "termlink_channel_search MCP — topic-flexible regex search"
description: >
  Port CLI cmd_channel_search (channel.rs:8535) to MCP. Four value-adds over termlink_agent_search: (1) topic-flexibility; (2) regex support; (3) all-flag to include meta msg-types; (4) richer row shape including msg_type. Reuses existing walk_topic_full_mcp helper.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-21T13:32:36Z
last_update: 2026-05-21T13:36:21Z
date_finished: 2026-05-21T13:36:21Z
---

# T-1771: termlink_channel_search MCP — topic-flexible regex search

## Context

Port `cmd_channel_search` (channel.rs:8535) to MCP as `termlink_channel_search`. Four value-adds over `termlink_agent_search`: (1) topic-flexibility (any topic vs chat-arc only); (2) regex support (CLI has it; agent_search lacks it); (3) `all` flag to include meta msg-types (UNREAD_META_TYPES filter); (4) richer row shape including `msg_type`. Implementation: mirror CLI's `payload_matches` as `payload_matches_mcp` pure helper, then tool method walks topic via `walk_topic_full_mcp`, filters per CLI semantics, returns `{ok, topic, pattern, hits, count}`.

## Acceptance Criteria

### Agent
- [x] `payload_matches_mcp(text, pattern, regex, case_sensitive)` pure helper mirrors CLI `payload_matches` 1:1 (regex with `(?i)` prefix when case_insensitive; non-regex falls back to lowercase substring).
- [x] Returns `Result<bool, String>` (string error for invalid regex — mirrored from CLI's anyhow error stringified).
- [x] `ChannelSearchParams { topic, pattern, regex?: Option<bool>, case_sensitive?: Option<bool>, all?: Option<bool>, limit?: Option<u64> }` params with sensible defaults (regex=false, case_sensitive=false, all=false, limit=100).
- [x] `termlink_channel_search` tool method: fail-fast validates regex once up-front; walks topic; filters per CLI semantics (skip meta msg-types unless `all=true`; skip empty payloads; substring/regex match); applies limit (`0 = unlimited`); returns rows `[{offset, sender_id, ts, msg_type, payload}]`.
- [x] Unit tests cover helper: substring case-insensitive default, substring case-sensitive, regex match, regex case-insensitive via `(?i)`, invalid regex error. Params deserialize tests cover defaults + all-flags-set. (7 tests, all pass)
- [x] `cargo build -p termlink-mcp` clean.
- [x] `cargo test -p termlink-mcp payload_matches_mcp` all pass.

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
cargo build -p termlink-mcp 2>&1 | tail -3
cargo test -p termlink-mcp --lib payload_matches_mcp 2>&1 | tail -10
cargo test -p termlink-mcp --lib channel_search 2>&1 | tail -10

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

### 2026-05-21T13:32:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1771-termlinkchannelsearch-mcp--topic-flexibl.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-df9d3c52
- **Timestamp:** 2026-05-21T13:36:21Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T13:36:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
