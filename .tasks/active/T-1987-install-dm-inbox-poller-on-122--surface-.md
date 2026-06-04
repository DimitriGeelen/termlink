---
id: T-1987
name: "Install DM-inbox poller on .122 — surface unread DMs to /var/log/dm-inbox.log (T-1985 followup)"
description: >
  T-1985 shipped the presence-emitter (peers can reach .122). Companion missing: nothing on .122 is READING the dm:* topics, so inbound messages still go unseen even though they land cleanly. This task installs a per-2-minute cron poller on .122 that walks all dm:<self-fp>:* topics, tracks per-topic last-seen offsets in /root/.termlink/dm-poller.state, and appends new envelopes to /var/log/dm-inbox.log so the operator can tail -f and see what arrived. Read-only (no auto-ack), idempotent across restarts. Closes the doorbell+mail rail for .122: T-1985 = presence (announce), T-1987 = polling (receive). Same install mechanism as T-1985 (remote exec → write script + add cron). Does NOT subscribe in real-time — that's a heavier listener-process pattern reserved for follow-up if cron-poll proves insufficient.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [fleet, ring20-management, doorbell-mail, operational]
components: []
related_tasks: []
created: 2026-06-04T08:52:05Z
last_update: 2026-06-04T08:53:16Z
date_finished: null
---

# T-1987: Install DM-inbox poller on .122 — surface unread DMs to /var/log/dm-inbox.log (T-1985 followup)

## Context

Completes the doorbell+mail rail on .122. T-1985 shipped the presence side (peers can detect ring20-management-agent is LIVE). This ships the inbox side (.122 sees what peers sent). Same install pattern as T-1985 — write script + add cron via `termlink remote exec`. PL-200 (auto-restore gap after binary swaps) is the predecessor learning. The cron approach is intentionally lightweight (every 2min, no real-time subscribe) — operator can upgrade to a long-running listener later if cron-poll proves insufficient.

## Acceptance Criteria

### Agent
- [ ] `/root/termlink/scripts/dm-poller.sh` exists on .122 (chmod +x, T-1987 design)
- [ ] Script enumerates dm:<self-fp>:* topics by listing topics with prefix `dm:` and filtering to those containing `9219671e28054458`
- [ ] Per-topic state file tracks last-seen offset (`/root/.termlink/dm-poller.state` — one JSON line per topic)
- [ ] On each poll: fetch new envelopes via `channel info <topic>` for current `posts` count; if > last_seen advance offset and write summary line
- [ ] Crontab entry `*/2 * * * *` calls the script, cron stdout/stderr → `/var/log/dm-poller.log`
- [ ] Initial state-file seed: pre-populate with current latest-offset per topic so first poll only sees NEW arrivals (the 30-msg historical backlog is in T-1985, not re-surfaced here)
- [ ] Smoke test: run script once manually; verify state file written + zero new-DM lines (post-seed)
- [ ] Cron-fired verification (T+3min): `/var/log/dm-poller.log` shows ≥1 timestamped fire
- [ ] Idempotent: running script twice produces no duplicate inbox lines (state file gates by last_offset)

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
set -o pipefail; timeout 15 termlink remote exec ring20-management tl-dorwh74y "test -x /root/termlink/scripts/dm-poller.sh && crontab -l | grep -q 'dm-poller.sh' && echo OK" 2>&1 | grep -q OK

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

### 2026-06-04T08:52:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1987-install-dm-inbox-poller-on-122--surface-.md
- **Context:** Initial task creation

### 2026-06-04T08:53:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
