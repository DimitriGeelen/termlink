---
id: T-1732
name: "termlink_agent_threads MCP — list thread roots on agent-chat-arc"
description: >
  termlink_agent_threads MCP — list thread roots on agent-chat-arc

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T21:22:21Z
last_update: 2026-05-20T21:29:09Z
date_finished: 2026-05-20T21:29:09Z
---

# T-1732: termlink_agent_threads MCP — list thread roots on agent-chat-arc

## Context

MCP parity for CLI `agent threads` (T-1533, ships in T-1365 family). Lists all thread roots on `agent-chat-arc` with `reply_count`, `participants`, `last_ts_ms`, `root_payload` preview. Read-only; composes `compute_threads_index` from `channel.rs:7047` (~90 LOC) over a `walk_topic_full` envelope sweep. Follows PL-172 silent-strip recipe — port pure helpers as `*_mcp` variants. Reuses `redacted_offsets_mcp` (T-1730) and `decode_payload_lossy_mcp` (T-1730) already in tools.rs.

## Acceptance Criteria

### Agent
- [x] `compute_threads_index_mcp` ported into tools.rs alongside existing `*_mcp` helpers; preserves redacted-root drop, redacted-reply drop, transitive BFS, sort-by-last_ts_ms-desc-then-offset-asc, drop rows with reply_count==0
- [x] `parent_offset_of_mcp` helper ported (reads `metadata.in_reply_to` as string, parses u64)
- [x] `ThreadIndexRowMcp` struct + `to_json_mcp` mirror CLI's `ThreadIndexRow` (`root_offset`, `reply_count`, `participants`, `last_ts_ms`, `root_payload`)
- [x] `AgentThreadsParams { top: Option<usize> }` and `termlink_agent_threads` MCP tool method added; uses `walk_topic_full_mcp` on `agent-chat-arc`; returns `{ok, topic, threads: [...]}` JSON; honors optional `top` truncation
- [x] ≥6 new unit tests in `tools::tests` covering: empty input, single-root happy path, redacted root dropped, redacted reply dropped, multi-thread sort order, top truncation
- [x] `cargo build -p termlink-mcp` clean (no new warnings beyond pre-existing `cur_run_end`)
- [x] `cargo test -p termlink-mcp` passes (all new tests + no regressions)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
cargo build -p termlink-mcp 2>&1 | grep -E '^(error|warning):' | grep -vE 'cur_run_end|generated [0-9]+ warning' | grep -q . && exit 1 || exit 0
cargo test -p termlink-mcp --quiet 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-05-20 — replace existing T-1574 `termlink_agent_threads` with T-1365 parity

- **Chose:** Delete the existing `termlink_agent_threads` MCP tool (T-1574, line ~17760) and its `AgentThreadsParams { limit }` struct; ship a fresh implementation backed by `compute_threads_index_mcp` matching CLI `agent threads` (T-1365) one-to-one.
- **Why:** The T-1574 implementation was a pre-parity sketch — string `root_offset`, no participants count, no payload preview, no redaction handling, sort-by-last_reply_ts-only. The CLI's `compute_threads_index` is the canonical implementation (redacted-root drop, redacted-reply drop, transitive BFS, sort-by-last_ts_ms-desc-then-offset-asc). CLI parity is the whole point of T-1732. CLAUDE.md §Tone: "Avoid backwards-compatibility hacks" — keeping both creates two MCP surfaces with the same name, ambiguous for callers.
- **Rejected:**
  - **Rename mine to `termlink_agent_threads_index`** — awkward (the CLI verb is just `threads`); callers would have to learn two near-identical tool names.
  - **Subprocess the CLI with `--json`** — viable (PL-172 silent-strip recipe option) but a 90-LOC pure-helper port is cheaper than a per-call subprocess, and pure helpers are unit-testable without spinning a binary.
- **Contract changes (breaking, intentional):**
  - `root_offset` was string → now u64 (matches CLI envelope contract — offsets are numbers in the hub)
  - `last_reply_ts` field renamed to `last_ts_ms` (matches CLI's `ThreadIndexRow`)
  - New fields: `participants` (distinct sender_ids), `root_payload` (lossy base64 preview)
  - Redacted roots/replies now properly excluded (old version silently double-counted)
  - `limit` param renamed to `top` (matches CLI's `--top`)

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-20T21:22:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1732-termlinkagentthreads-mcp--list-thread-ro.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-1bbc2238
- **Timestamp:** 2026-05-20T21:29:25Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T21:29:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
