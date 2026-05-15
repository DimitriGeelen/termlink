---
id: T-1639
name: "fleet doctor: surface sender-side pickup-queue stall (T-1827 follow-up)"
description: >
  Framework-agent T-1827 offset-14 follow-up. When termlink-agent posts to framework:pickup, the local hub returns offset+ts immediately but the message can sit queued (the offset-9/10/12 stretch took ~19h to surface to framework-agent on same local hub). fleet doctor currently shows hub health (reachable, version, latency) but NOT sender-side queue depth or stall age. Operator and agent both blind to outbound queue stalls until destination acks. Ask: add a queue-status block to fleet doctor that surfaces per-topic outbound-queue depth, oldest-unacked-age, and last-acked offset. Threshold-warn when oldest-unacked-age > 5min. Out of scope: fixing the queue stall itself — that's a different issue. This task is the visibility layer.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [ops, fleet-doctor, visibility, arc:queue-health]
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-15T07:18:00Z
last_update: 2026-05-15T07:47:48Z
date_finished: 2026-05-15T07:47:48Z
---

# T-1639: fleet doctor: surface sender-side pickup-queue stall (T-1827 follow-up)

## Context

Framework-agent surfaced this ask on `framework:pickup` offset 14 (2026-05-14)
after the offset-9/10/12 stretch took ~19h to surface across the same local
hub. `fleet doctor` shows hub reachability + version + latency, but NOT
sender-side outbound-queue state. `OfflineQueue` (~/.termlink/outbound.sqlite,
T-1161) holds buffered posts when the local hub can't ack — already surfaced
by the standalone `termlink channel queue-status` command but invisible in
the fleet-wide view operators actually run.

Goal: integrate the existing offline-queue read into `cmd_fleet_doctor`'s
output. Tiny scope; pure additive surface, no new RPCs.

## Acceptance Criteria

### Agent
- [x] `cmd_fleet_doctor` in `crates/termlink-cli/src/commands/remote.rs` emits a `Outbound queue: N pending, oldest topic=X age=Ys` line (or `0 pending` / `0 pending (no queue file)`) BEFORE the per-hub probe loop, so it's the first thing the operator reads.
- [x] When `pending > 0 AND oldest_age_secs > 300` the line is prefixed `[WARN]` and `total_warn` is incremented; a follow-up `hint:` line names the diagnostic command (`termlink channel queue-status`).
- [x] JSON output (`fleet doctor --json`) carries a top-level `queue_status` object with fields: `queue_path`, `exists`, `pending`, `oldest_age_secs`, `oldest_topic`, `warn`.
- [x] No new dependencies; reuses `termlink_session::offline_queue::{default_queue_path, OfflineQueue}` already used by `cmd_channel_queue_status`.
- [x] `cargo build --release --bin termlink` clean (no new warnings in changed file).
- [x] Manual smoke: `fleet doctor` on a healthy fleet shows `Outbound queue: 0 pending` first, then existing per-hub blocks; `fleet doctor --json | jq .queue_status` returns the object.

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
cargo build --release --bin termlink 2>&1 | tail -3 | grep -q "Finished"
cargo check --all 2>&1 | tail -2 | grep -q "Finished"
./target/release/termlink fleet doctor --json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); assert 'queue_status' in d, 'missing queue_status'; qs=d['queue_status']; assert all(k in qs for k in ('queue_path','exists','pending','oldest_age_secs','warn')), f'missing fields in {qs}'; print('queue_status fields ok:', list(qs.keys()))"

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

### 2026-05-15 — scope held to additive surface
- **What changed:** Original task description mentioned "per-topic" depth and "last-acked offset". Implementation surfaces a single overall depth + oldest-topic + oldest-age rather than a per-topic table. Reason: the local `OfflineQueue` (sqlite) keys posts by ordinal queue_id not topic; computing per-topic counts means a full table scan on every fleet-doctor run. For the visibility goal ("operator sees the stall at all"), the oldest-topic field plus the existing standalone `termlink channel queue-status` (which already shows head topic) is sufficient. If per-topic-grouped output becomes a real ask later, file as a follow-up.
- **Plan impact:** Smaller surface than initially scoped; fewer joins; same operator value for the stall case.
- **Triggered:** None — no scope cut warrants its own task. Recorded for arc traceability.

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

### 2026-05-15T07:18:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1639-fleet-doctor-surface-sender-side-pickup-.md
- **Context:** Initial task creation

### 2026-05-15T07:38:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-05-15T07:47:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
