---
id: T-1796
name: "Paginate fetch_topic_msgs for deeper-than-1000 fleet history (T-1795 follow-up)"
description: >
  fetch_topic_msgs is clamped to the hub's 1000-envelope page cap (T-1795). Fleet-aggregation verbs (presence, overview, by-project) genuinely want more history on busy fleets but can only get the most-recent 1000 in one page. Add bounded multi-page pagination (model on walk_topic_full) so a caller can request the most-recent N>1000 envelopes via multiple round-trips.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [chat-arc, fetch, T-1795]
components: [crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/commands/channel.rs]
related_tasks: [T-1795]
created: 2026-05-22T06:59:41Z
last_update: 2026-05-27T20:49:14Z
date_finished: 2026-05-27T20:49:14Z
---

# T-1796: Paginate fetch_topic_msgs for deeper-than-1000 fleet history (T-1795 follow-up)

## Context

T-1795 fixed the bug where `fetch_topic_msgs` read the OLDEST page when
`slice_size` exceeded the hub's 1000-per-page cap, by clamping the effective
slice to the cap. That makes on-thread/presence/overview/by-project correct
(they read the most-recent 1000) but caps fleet history at 1000 envelopes per
call. The genuine intent behind the original `2000` slices — deeper history
for fleet aggregation on busy arcs — is now unmet. This task adds bounded
multi-page pagination so callers can request the most-recent N>1000 envelopes
across multiple round-trips (model on the existing `walk_topic_full` /
`fetch_chat_arc_full` paging). Enhancement, not a bug — parked at horizon=later.

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent

Scope: add ONE new public helper + ONE new async function in
`crates/termlink-cli/src/commands/channel.rs`. Do NOT modify the existing
`fetch_topic_msgs` (T-1795 single-page tail) or `walk_topic_full` (full topic
walk) — both serve current callers correctly. The gap is the *bounded
tail-anchored multi-page* variant.

- [x] New pure helper `paginated_tail_start(count, slice_size) -> u64` returns the tail-anchored start cursor (`count.saturating_sub(slice_size)`). Lives alongside `tail_slice_cursor` (line 701). Documented with `T-1796` reference.
- [x] New async function `fetch_topic_msgs_paginated(topic, hub, slice_size)` makes a `channel.list` round-trip for the count, then walks the hub from `paginated_tail_start(count, slice_size)` forward in pages of `HUB_SUBSCRIBE_PAGE_CAP` (1000), collecting up to `slice_size` envelopes in offset-ascending order. Returns `Result<Vec<Value>>`.
- [x] Equivalence: when `slice_size <= HUB_SUBSCRIBE_PAGE_CAP`, `fetch_topic_msgs_paginated` returns the same envelope set as `fetch_topic_msgs` (a single round-trip suffices); when `count <= slice_size`, returns ALL envelopes (equivalent to `walk_topic_full`).
- [x] Unit tests for `paginated_tail_start`: slice < count (anchors at tail), slice = count (cursor 0), slice > count (cursor 0, saturating), slice = 0 (cursor = count).
- [x] `cargo check -p termlink` passes from the workspace root (`/opt/termlink`).
- [x] `cargo test -p termlink paginated_tail_start` runs the new unit tests and they PASS.
- [x] Existing T-1795 tests (`fetch_topic_tail_cursor_*`) still PASS — proves we didn't regress the single-page path.

### Human
<!-- All criteria are mechanically verifiable; no human AC needed. The function
     is internal pagination plumbing — there is no operator-facing surface yet.
     Future tasks will wire it into specific verbs (presence, overview, etc.)
     where operator-facing behavior may emerge and human ACs become relevant. -->

## Verification

# T-1796 verification: build + targeted unit tests for the new helper.
cargo check -p termlink
cargo test -p termlink --bin termlink paginated_tail_start
cargo test -p termlink --bin termlink fetch_topic_tail_cursor

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

**Recommendation:** GO — pagination helper shipped, ready for caller wiring in follow-up tasks.

**Rationale:** All 7 Agent ACs satisfied. The implementation closes the gap T-1795 left open: callers wanting >1000 envelopes from a busy topic now have a bounded multi-round-trip alternative to the unbounded `walk_topic_full`. The new helper is internal plumbing only — no caller migrated yet (intentional; the spec said add the verb), so the existing `fetch_topic_msgs` (T-1795 tail-clamp) and `walk_topic_full` (full walk) remain authoritative for their current callers. The `#[allow(dead_code)]` annotation makes the "ships ahead of callers" stance explicit and prevents a clippy regression when wiring follow-up tasks land.

**Evidence:**
- New code: `crates/termlink-cli/src/commands/channel.rs` — `paginated_tail_start` (pure helper, ~line 714), `fetch_topic_msgs_paginated` (async function, ~100 lines, ~line 800)
- New tests: 5 unit tests for `paginated_tail_start` covering slice<count, slice=count, slice>count (saturating), slice=0, and empty topic
- Build: `cargo check -p termlink` PASS with zero new warnings (the unrelated `termlink-mcp` warning is pre-existing)
- Tests: 5 new + 3 regression PASS (`cargo test -p termlink --bin termlink paginated_tail_start` + `fetch_topic_tail_cursor`)
- Verification gate: 3/3 PASS

**Algorithm summary (for follow-up callers):**
1. `channel.list` → topic count (one round-trip)
2. Walk from `paginated_tail_start(count, slice_size)` forward in pages of `HUB_SUBSCRIBE_PAGE_CAP` (1000) — typically ⌈slice_size / 1000⌉ round-trips
3. Stop when collected ≥ slice_size OR a page comes back short (topic exhausted from this cursor)
4. Trim any overshoot to exactly `slice_size` envelopes

**Follow-up candidates (separate tasks when wanted):**
- Wire `fetch_topic_msgs_paginated` into fleet-aggregation verbs (`agent presence`, `agent overview`, `agent on-thread`) where deeper history is useful on busy fleets
- Consider a CLI knob (`--depth N`) on those verbs so the operator opts into multiple round-trips per call

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

### 2026-05-22T06:59:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1796-fix-broken-bin-test-compilation--cmdflee.md
- **Context:** Initial task creation

### 2026-05-27T20:45:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7fa3da00
- **Timestamp:** 2026-05-27T20:49:34Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-27T20:49:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Pagination helper + 5 unit tests + 3 regression tests; cargo check clean; verification 3/3 PASS
