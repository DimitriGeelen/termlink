---
id: T-1816
name: "agent on-thread --depth N — wire T-1796 helper as first caller"
description: >
  agent on-thread --depth N — wire T-1796 helper as first caller

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-27T21:53:40Z
last_update: 2026-05-27T22:03:40Z
date_finished: 2026-05-27T22:03:40Z
---

# T-1816: agent on-thread --depth N — wire T-1796 helper as first caller

## Context

T-1796 shipped `fetch_topic_msgs_paginated` as internal plumbing with
`#[allow(dead_code)]` — the spec deliberately did not wire any caller. This
task wires it into `agent on-thread` as the first caller, behind a new
`--depth N` CLI flag. Default depth = 1000 envelopes (`HUB_SUBSCRIBE_PAGE_CAP`),
which preserves current behavior; setting `--depth 5000` lets the operator
pull deeper history for filtering on stale threads where the most-recent
1000 envelopes contain few matching posts.

Why `agent on-thread` first: it was the exact bug-symptom that motivated
T-1795 (operator requested slice 2000, got empty results). It's a single
file change with two call sites (one-shot path + watch loop), no MCP-side
work needed (the MCP companion `termlink_agent_on_thread` already does a
full topic walk and has different semantics).

## Acceptance Criteria

### Agent
- [x] Add `#[arg(long = "depth", default_value_t = 1000)] depth: u64` to the `OnThread` clap variant in `crates/termlink-cli/src/cli.rs`. Doc-comment names T-1816 and explains: clamped to [1, 100000], default 1000 (the hub page cap), values >1000 make multiple round-trips.
- [x] Thread `depth` through the dispatch site in `crates/termlink-cli/src/main.rs` to `cmd_agent_on_thread`.
- [x] Update `cmd_agent_on_thread` signature in `crates/termlink-cli/src/commands/agent.rs` to accept `depth: u64`, clamp it to `1..=100_000`, and replace both call sites of `fetch_recent_chat_arc_msgs(hub, HUB_SUBSCRIBE_PAGE_CAP)` (one-shot fetch + watch-loop fetch) with `fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth)`. Tracking comment names T-1816.
- [x] Remove `#[allow(dead_code)]` on `fetch_topic_msgs_paginated` and `paginated_tail_start` in `crates/termlink-cli/src/commands/channel.rs` — they now have a real caller.
- [x] `cargo check -p termlink` PASS with zero new warnings.
- [x] `cargo test -p termlink --bin termlink paginated_tail_start` PASS (regression — proves the T-1796 unit tests still hold under the new wiring).
- [x] `cargo build --release -p termlink` PASS (the bin actually links) — wiring the verb means the dead-code removal must hold.
- [x] `target/release/termlink agent on-thread --help 2>&1 | grep -q -- '--depth'` PASS — the new flag is documented in --help output.

### Human
<!-- All Agent ACs are mechanically verifiable. The one operator-facing surface
     (--depth flag behavior on a real busy thread) doesn't exist as a unit-testable
     concern; live exercise can come later if needed. -->

## Verification

cargo check -p termlink
cargo test -p termlink --bin termlink paginated_tail_start
cargo build --release -p termlink
target/release/termlink agent on-thread --help 2>&1 | grep -q -- '--depth'

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

## Recommendation

**Recommendation:** GO — first caller wired, dead-code annotation removed.

**Rationale:** All 8 Agent ACs satisfied. The T-1796 helper now has a real
caller, `agent on-thread`, behind the `--depth N` flag. Default 1000
preserves backward-compatible single-page behavior; the operator opts into
deeper history via `--depth 5000` etc. Clamp at 100k caps the worst-case
round-trip count at ~100. The MCP companion was deliberately left alone
because it uses different semantics (full topic walk → BFS tree
construction) and doesn't need the new helper.

**Evidence:**
- `crates/termlink-cli/src/cli.rs` — new `depth: u64` arg on the `OnThread`
  clap variant (default 1000)
- `crates/termlink-cli/src/main.rs` — `depth` threaded through dispatch
- `crates/termlink-cli/src/commands/agent.rs` — signature updated, two
  call sites (watch loop + one-shot) now use `fetch_topic_msgs_paginated`
- `crates/termlink-cli/src/commands/channel.rs` — `#[allow(dead_code)]`
  removed on both `paginated_tail_start` and `fetch_topic_msgs_paginated`
- Verification: 4/4 PASS (`cargo check` clean, regression unit tests pass,
  release build links, `--depth` flag visible in `--help`)

**Live help output:**
```
      --depth <DEPTH>
          T-1816: history depth — number of recent chat-arc envelopes to
          fetch before filtering by thread/window/peer. Default 1000 ...
          Clamped to [1, 100000] [default: 1000]
```

**Follow-up candidates:**
- Apply the same `--depth` pattern to `agent recent`, `agent presence`,
  `agent overview`, `agent timeline` — each is a one-flag-plus-call-site
  change of similar size
- Add MCP parity if a busy thread observation reveals the CLI surface
  is insufficient for agents using `termlink_agent_on_thread`

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

### 2026-05-27T21:53:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1816-agent-on-thread---depth-n--wire-t-1796-h.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f2f4f91d
- **Timestamp:** 2026-05-27T22:10:30Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 4

**Per-AC findings:**

- **AC#1 (Agent)** — Add `#[arg(long = "depth", default_value_t = 1000)] depth: u64` to the `OnThread` clap variant in `crates/termlink-cli/src/cli.rs`. Doc-comment names T-1816 and explains: clamped to [1, 100000], defau
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/cli.rs in: Add `#[arg(long = "depth", default_value_t = 1000)] depth: u64` to the `OnThread` clap variant in `crates/termlink-cli/src/cli.rs`. Doc-comment names `
- **AC#2 (Agent)** — Thread `depth` through the dispatch site in `crates/termlink-cli/src/main.rs` to `cmd_agent_on_thread`.
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/main.rs in: Thread `depth` through the dispatch site in `crates/termlink-cli/src/main.rs` to `cmd_agent_on_thread`.`
- **AC#3 (Agent)** — Update `cmd_agent_on_thread` signature in `crates/termlink-cli/src/commands/agent.rs` to accept `depth: u64`, clamp it to `1..=100_000`, and replace both call sites of `fetch_recent_chat_arc_msgs(hub,
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/commands/agent.rs in: Update `cmd_agent_on_thread` signature in `crates/termlink-cli/src/commands/agent.rs` to accept `depth: u64`, clamp it to `1..=100_000`, and replace b`
- **AC#4 (Agent)** — Remove `#[allow(dead_code)]` on `fetch_topic_msgs_paginated` and `paginated_tail_start` in `crates/termlink-cli/src/commands/channel.rs` — they now have a real caller.
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/commands/channel.rs in: Remove `#[allow(dead_code)]` on `fetch_topic_msgs_paginated` and `paginated_tail_start` in `crates/termlink-cli/src/commands/channel.rs` — they now ha`
### 2026-05-27T22:03:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** First caller wired; dead-code annotation removed; verification 4/4 PASS
