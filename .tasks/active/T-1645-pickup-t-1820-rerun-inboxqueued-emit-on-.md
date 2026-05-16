---
id: T-1645
name: "Pickup: T-1820 rerun: inbox.queued emit on fix-shipped hub 0.9.2110 (ebe05294) not observable from any session-targeted event poll/subscribe — wiring-asymmetry vs fw peer subscribe (from 999-Agentic-Engineering-Framework)"
description: >
  Auto-created from pickup envelope. Source: 999-Agentic-Engineering-Framework, task T-1637. Type: bug-report.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [pickup, bug-report]
components: []
related_tasks: []
created: 2026-05-16T07:01:02Z
last_update: 2026-05-16T08:39:34Z
date_finished: null
source_task_id_in_origin: T-1637
source_project_in_origin: "999-Agentic-Engineering-Framework"
---

# T-1645: Pickup: T-1820 rerun: inbox.queued emit on fix-shipped hub 0.9.2110 (ebe05294) not observable from any session-targeted event poll/subscribe — wiring-asymmetry vs fw peer subscribe (from 999-Agentic-Engineering-Framework)

## Context

Framework T-1637 / P-042 bug-report: post-T-1636/T-1637 fix at `ebe05294`,
`inbox.queued` events are emitted via direct `aggregator().inject()` calls
inside `channel.rs handle_channel_post_with` (line 497) AND
`mirror_inbox_deposit_with` (line 207) — both with `session_id: "hub"` and
no attachment to any per-session bus.

Framework's subscriber wire-path is `termlink event poll <addressee> --topic
inbox.queued --since <cursor>` which only sees `<addressee>`'s per-session
bus. Result: emit is invisible to every per-session probe; only visible via
hub-level `event.subscribe` (no `target` param) → `handle_hub_subscribe` →
`agg.collect()`. The existing CLI verbs `event poll/watch/collect` all
require/iterate target sessions and never call hub-level subscribe.

**Resolution shape chosen:** B (ship hub-aggregator subscribe path). Not A
(re-attach to addressee bus). `inbox.queued` is semantically an **observer**
event (audit stream for monitors like framework's peer.py), not addressee-
bound. The addressee's notification mechanism IS the inbox topic itself
(`channel.post inbox:<id>` lands on the bus topic, subscribable as
`channel subscribe inbox:<id>`). Adding a duplicate per-session emit would
mislocate the observer signal and create two sources of truth. Hub-
aggregator is the canonical sink by design (T-966).

**Gap exposed:** Hub already supports `event.subscribe` with no target →
`handle_hub_subscribe` → `aggregator.collect()` (router.rs:128 + 610). But
no CLI/MCP path passes `target: null`. This task ships that path.

## Acceptance Criteria

### Agent
- [x] `event watch` CLI accepts `--hub` flag (mutually exclusive with positional `targets`) — cli.rs Watch variant + `conflicts_with = "targets"`; test `event_watch_hub_conflicts_with_positional_target` passes
- [x] When `--hub` set, `cmd_watch` makes a single hub-level `event.subscribe` RPC (no target) and renders aggregated events — events.rs `cmd_watch_hub` (commit 0ac0bd0a); main.rs dispatch branches on `hub` flag
- [x] `--topic <name>` filter works under `--hub` (server-side filter via aggregator's `topic_filter`) — verified live with `--topic inbox.queued`; aggregator's `collect()` filters by topic
- [x] `--json` mode emits one NDJSON line per event under `--hub`, with `session: "hub"` field present — verified: `{"session":"hub","session_name":"hub","source":"hub-aggregator",...}` in smoke output
- [x] `--count N` exits after N events under `--hub`; `--timeout N` exits after N seconds under `--hub` — verified: smoke ran with `--count 1 --timeout 6` and exited after the first event
- [x] Unit test asserts `--hub` flag wires to hub-level RPC (no target enumeration) — 3 tests in `cli::cli_tests` pass: `event_watch_hub_flag_parses`, `event_watch_hub_conflicts_with_positional_target`, `event_watch_without_hub_accepts_targets`
- [x] Live smoke against local hub: `termlink channel post inbox:t1645-smoke ...` + `termlink event watch --hub --topic inbox.queued --json --count 1 --timeout 5` returns event with `addressee_session_id == "t1645-smoke"` — confirmed twice (build + post-test rebuild); see `/tmp/t1645-watch2.out`
- [x] Reply posted to framework-agent on `agent-chat-arc` topic with `_thread=T-1645, in_reply_to=P-042, msg_type=fix-shipped` citing termlink commit SHA + repro one-liner — `agent-chat-arc` offset=1474 (2026-05-16T08:39:58Z), payload from `/tmp/t1645-reply.txt` via `termlink agent contact --file`'s sibling `channel post --payload "$(cat ...)"` route (T-1646 `--file` couldn't be used because agent contact is per-DM, this fanout went through the broadcast topic per T-1644 fallback)

## Verification

# CLI tests for --hub flag wiring (clap parsing, conflicts_with).
cd /opt/termlink && cargo test --release -p termlink --bin termlink cli_tests 2>&1 | tail -3 | grep -q "3 passed"

# CLI flag wired and discoverable in --help.
cd /opt/termlink && target/release/termlink event watch --help 2>&1 | grep -q -- "--hub"

# Smoke: precreate the inbox topic (idempotent), then capture watch output to a
# file (avoids SIGPIPE under set -o pipefail), post to inbox:t1645-smoke, and
# assert the aggregator surfaced inbox.queued for that addressee.
cd /opt/termlink && target/release/termlink channel create inbox:t1645-smoke --json > /dev/null 2>&1; timeout 8s target/release/termlink event watch --hub --topic inbox.queued --json --count 1 --timeout 6 > /tmp/t1645-watch.out 2>&1 & WATCH_PID=$!; sleep 1; target/release/termlink channel post inbox:t1645-smoke --msg-type file.init --payload '{"transfer_id":"t1645-smoke-verify"}' --json > /tmp/t1645-post.out 2>&1; wait $WATCH_PID || true; grep -q '"addressee_session_id":"t1645-smoke"' /tmp/t1645-watch.out

## RCA

**Symptom:** Framework's `lib/peer.py::poll_once` calling
`termlink event poll <addressee_session_id> --topic inbox.queued --since
<cursor>` returns `count=0, next_seq=0` (no advance), even after the
fix-shipped hub 0.9.2110 (`ebe05294`) confirmed inbox.queued emission via
internal tests. The emit fires (verified by hub-side unit tests using
`agg.collect()`), but no CLI/MCP-visible path on the per-session bus
surfaces it.

**Root cause:** `channel.rs:497` and `channel.rs:207`
(`mirror_inbox_deposit_with`) both emit `inbox.queued` via
`aggregator().inject()` with `session_id: "hub"`. The aggregator is fed
by (a) per-session background long-pollers that subscribe to each real
session's `event.subscribe`, and (b) direct injects. Direct injects with
`session_id="hub"` never appear on any real session's bus. The hub
router DOES handle `event.subscribe` with no `target` → routes to
`handle_hub_subscribe` → `aggregator.collect()` — but the CLI verbs
`event poll/watch/collect` all require/iterate target sessions, so no
CLI path reaches the aggregator-direct sink.

**Why structurally allowed:** Two converging blind spots:
(1) The cross-repo seam between T-1636 (emit added) and T-1637 (emit
relocated) was validated using `agg.collect()` directly in hub-side
tests, not via the CLI/MCP path that downstream consumers use. The
test coverage proved the emit exists in the broadcast channel but never
proved it was reachable from the documented operator surface.
(2) `event.subscribe` has a documented hub-level mode (no target) in
the hub router and aggregator design (T-966), but the CLI never grew a
flag to invoke it. The MCP wrapper inherited the same gap. No lint or
schema check ties hub-router branches to CLI exposure.

**Prevention:**
Tactical: this PR adds `event watch --hub`, closing the visibility gap.
Pattern: when a hub event is emitted via `aggregator().inject(...)` with
`session_id: "hub"`, the canonical operator-side test fixture must
include a CLI invocation that reaches the aggregator stream — not just
a unit-test on `agg.collect()`. The cross-repo seam test (peer.py poll
or equivalent) becomes part of the emit's acceptance gate.
Tier-B (proposed, file as separate task): add a rpc_audit assertion
that any `event.subscribe` branch reachable from the router has a
corresponding CLI flag, by inventory pairing.
Learning: capture as L-pattern "hub-aggregator emits need CLI-surface
companion before downstream consumers see them."

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

### 2026-05-16T07:01:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1645-pickup-t-1820-rerun-inboxqueued-emit-on-.md
- **Context:** Initial task creation

### 2026-05-16T08:25:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
