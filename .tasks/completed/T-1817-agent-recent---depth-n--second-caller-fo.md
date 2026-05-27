---
id: T-1817
name: "agent recent --depth N — second caller for T-1796 helper (T-1816 follow-up #1)"
description: >
  agent recent --depth N — second caller for T-1796 helper (T-1816 follow-up #1)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-27T23:20:10Z
last_update: 2026-05-27T23:30:02Z
date_finished: 2026-05-27T23:30:02Z
---

# T-1817: agent recent --depth N — second caller for T-1796 helper (T-1816 follow-up #1)

## Context

T-1816 wired `--depth N` for `agent on-thread` as the first caller of the T-1796 paginated
helper. T-1817 applies the same pattern to `agent recent <peer>` — same one-flag-plus-call-site
shape, direct termlink value: operators can read deeper than 1000 envelopes of single-peer
history on busy fleets where the most-recent 1000 are dominated by other peers.

## Acceptance Criteria

### Agent
- [x] `--depth` flag added to the `AgentAction::Recent` clap variant (default 1000, clamped [1, 100000]) with doc explaining single-round-trip vs multi-page tradeoff
- [x] `main.rs` destructures `depth` and threads it into `cmd_agent_recent` as the final arg
- [x] `cmd_agent_recent` signature gains `depth: u64`; clamped to [1, 100000] as `clamped_depth`
- [x] Both `fetch_recent_chat_arc_msgs(hub, 1000)` call sites in `cmd_agent_recent` (watch loop + one-shot) replaced with `super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth)`
- [x] `cargo check -p termlink` clean
- [x] `cargo test -p termlink --bin termlink paginated_tail_start` still passes (regression sentinel for T-1796 helper) — 5/5
- [x] Release build succeeds (`cargo build -p termlink --release --bin termlink`) — operator gets a binary that actually carries the flag
- [x] `target/release/termlink agent recent --help` shows `--depth` with `[default: 1000]`

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
cargo check -p termlink
cargo test -p termlink --bin termlink paginated_tail_start

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

[GO] Second caller of T-1796 `fetch_topic_msgs_paginated` shipped. `agent recent <peer>` now
accepts `--depth N` (default 1000 = pre-T-1817 behavior; up to 100000 walks bounded multi-page).
Same one-flag-plus-call-site shape as T-1816 — proves the pattern generalizes cleanly.

Follow-up candidates (file as needed): apply identical pattern to `agent presence`, `agent
overview`, `agent timeline` (each is one flag + two call-site swaps of the same shape). MCP
parity (`termlink_agent_recent`) currently caps at 1000 via its own walk_topic_full — separate
code path, deferred unless operators report needing it.

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

### 2026-05-27T23:20:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1817-agent-recent---depth-n--second-caller-fo.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7952d541
- **Timestamp:** 2026-05-27T23:30:26Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-27T23:30:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
