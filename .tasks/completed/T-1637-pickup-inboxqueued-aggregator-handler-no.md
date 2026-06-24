---
id: T-1637
name: "Pickup: inbox.queued aggregator handler not registered at hub boot — emit unreachable from CLI (from agentic-engineering-framework)"
description: >
  Auto-created from pickup envelope. Source: agentic-engineering-framework, task T-1636. Type: bug-report.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [pickup, bug-report, arc:peer-consult, cross-repo]
components: [crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs]
related_tasks: [T-1636, T-1820, T-1821, T-1166]
created: 2026-05-14T05:55:01Z
last_update: 2026-05-15T09:29:07Z
date_finished: 2026-05-15T09:29:07Z
source_task_id_in_origin: T-1636
source_project_in_origin: "agentic-engineering-framework"
---

# T-1637: Pickup: inbox.queued aggregator handler not registered at hub boot — emit unreachable from CLI (from agentic-engineering-framework)

## Context

Framework-agent (P-041 bug report) found that three `channel post inbox:<id> --msg-type file.init` calls did NOT fire `inbox.queued` on their live subscriber, even though T-1636 unit tests pass. Their working hypothesis was "router::init_aggregator not called at hub startup". **That hypothesis is false** — `server.rs:225` calls `router::init_aggregator()` in the production boot path. The actual gap: T-1636 emit lives in `mirror_inbox_deposit_with`, called only from `event.emit_to` → offline target → file-class topic spool (`router.rs:517`). `channel.post` to `inbox:<id>` is a separate RPC handler (`handle_channel_post_with`) that writes the bus topic but never injects.

This is resolution path (C) from the bug report: bundle the emit with the delivery-path change. Post-T-1166 (legacy primitive retirement), `event.emit_to`/`inbox.push` go away and `channel.post` becomes the only inbox-delivery RPC — making the channel.post path the right long-term home for the emit.

**Scope:** Extend `handle_channel_post_with` to inject `inbox.queued` when the posted topic starts with `inbox:` and bus.post succeeds. Payload contract matches T-1636 exactly.

## Acceptance Criteria

### Agent
- [x] `handle_channel_post_with` injects `inbox.queued` via `EventAggregator::inject` when the post topic starts with `inbox:` and `bus.post` returns `Ok(offset)`
- [x] Payload shape exactly matches T-1636 contract: `{schema_version, addressee_session_id, channel, message_offset, enqueued_at}` — addressee_session_id derived from topic suffix after `inbox:`
- [x] No emit when topic does not start with `inbox:` (existing channel.post behaviour unchanged for non-inbox topics)
- [x] Unit test pins both: channel.post to `inbox:<id>` fires; channel.post to a non-inbox topic does NOT fire
- [x] `cargo build --release --bin termlink` clean; existing T-1636 tests still pass; `cargo test -p termlink-hub inbox_queued` exits 0
- [x] Framework-agent notified via `framework:pickup` channel with fix.shipped envelope including commit hash + version

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo build --release --bin termlink 2>&1 | tail -3 | grep -q "Finished\|Compiling"
cargo test -p termlink-hub --lib -- inbox_queued channel_post_inbox channel_post_non_inbox --test-threads=1 2>&1 | grep -qE "4 passed; 0 failed"
cargo check --all 2>&1 | grep -q "Finished"

## RCA

**Symptom:** Framework-agent's live smoke for T-1820 expected `inbox.queued` on their subscriber stream after `channel.post inbox:<id> --msg-type file.init`. Event never fired; subscriber `next_seq` stuck.

**Root cause:** T-1636 attached the emit to `mirror_inbox_deposit_with` — the offline-target file-event spool path reached only from `event.emit_to` (router.rs:517). `channel.post` is a different RPC (`handle_channel_post_with`) that bypasses inbox/deposit entirely and writes directly to bus topics. The emit point and the natural delivery path were misaligned.

**Why structurally allowed:** T-1636's unit tests exercised `mirror_inbox_deposit_with` directly with synthetic bus. No integration test exercised the channel.post → inbox.queued path end-to-end. Spec ambiguity in T-1804 inception too: "inbox delivery" was implicit about which RPC path. Framework-agent's misdiagnosis ("aggregator handler not registered") was an artifact of unfamiliarity with the hub's two-RPC inbox model.

**Prevention:** Unit test added here pins emit firing from channel.post; the negative case (non-inbox topic) is also pinned, preventing accidental over-emit. Post-T-1166, only the channel.post path remains (event.emit_to retires), so the seam converges naturally. Learning: when shipping cross-repo seams, exercise the live consumer's RPC path, not the most convenient internal helper.

## Evolution

### 2026-05-15 — diagnosis pivot
- **What changed:** Framework-agent's hypothesis ("init_aggregator not called at hub boot") was investigated and disproven — `server.rs:225` already calls it in the production path. The actual gap was the emit being attached only to `mirror_inbox_deposit_with`, reached via `event.emit_to`, never via `channel.post`. Bug report path (A) was a red herring; path (C) ("bundle with next delivery-path change") was the correct framing.
- **Plan impact:** Scope contained — single-site addition in `handle_channel_post_with` mirroring the T-1636 emit shape exactly. No new CLI surface needed, no protocol revisit.
- **Triggered:** None new. T-1166 retirement path simplifies post-fix (channel.post becomes the sole inbox delivery RPC, so the emit lives in the right place by default).

### 2026-05-15 — test-isolation tax
- **What changed:** Adding a second concurrent aggregator-injecting test exposed a pre-existing flakiness in T-1636's `inbox_queued_fires_for_no_consumer` and an unrelated `hub_subscribe_returns_events_structure` test — both relied on the singleton aggregator broadcast being empty during their window. With more concurrent injectors, that assumption is no longer safe.
- **Plan impact:** Both tests required addressee-filter or shape-only relaxation. Tiny change but worth flagging: any future test that injects through `crate::router::aggregator()` shares the same singleton — design for cross-test pollution from day one.
- **Triggered:** None. Single-pass robustness fix in this task.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-14T05:55:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1637-pickup-inboxqueued-aggregator-handler-no.md
- **Context:** Initial task creation

### 2026-05-15T09:29:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
