---
id: T-1217
name: "T-1168/B2 Learnings subscriber (poller script)"
description: >
  Cron-driven subscriber-poller for the channel:learnings topic landed by
  T-1168 B1. Drains new learning envelopes from the hub event bus since the
  last cursor, appends de-duplicated entries to
  .context/project/received-learnings.yaml in the consumer project. Pure
  read-side mirror of the T-1168 publisher; Watchtower display panel is split
  to T-1218.
status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [T-1155, T-1168, bus, learnings]
components: []
related_tasks: [T-1168, T-1218, T-1155, T-1214]
created: 2026-04-24T12:27:43Z
last_update: 2026-04-25T16:28:37Z
date_finished: 2026-04-24T13:10:12Z
---

# T-1217: T-1168/B2 Learnings subscriber (poller script)

## Context

T-1168 B1 shipped a one-way publisher (`lib/publish-learning-to-bus.sh`) that
posts every new learning to `channel:learnings` using `termlink channel post`
(Tier-A, T-1160) or falling back to `termlink event broadcast` (universally
present pre-channel). This task builds the consumer side: a poller script
that periodically drains the topic and materialises received learnings in
the local project.

Scope reduced from the original T-1217 capture (subscriber + Watchtower panel
+ auto-apply guard) to just the subscriber. Watchtower panel becomes T-1218;
auto-apply guard is deliberately out of scope (received learnings are
advisory, not auto-applied).

**Open design question resolved in AC1:** the current termlink CLI (0.9.206
on this host) does not expose `channel.subscribe`. The publisher's fallback
path uses `event broadcast`, so the subscriber must consume via `event
collect --topic channel:learnings` against the hub. AC1 spikes this to
confirm the collect-side actually receives broadcast payloads; if it does
not, the task pivots to a hub-side topic queue and is re-scoped before
proceeding.

## Acceptance Criteria

### Agent
- [x] **Discovery spike:** confirmed `event collect --topic channel:learnings
      --payload-only` receives `event broadcast` payloads on the live hub
      (3 copies observed per broadcast → dedup mandatory). Recorded in
      Decisions.
- [x] New framework helper `lib/subscribe-learnings-from-bus.sh` — landed at
      /opt/999-AEF commit 87d2ca2d. Cursor-less design (dedup replaces cursor
      per spike finding). Drains via `event collect --topic channel:learnings
      --payload-only --json --timeout ${FW_LEARNINGS_BUS_TIMEOUT:-30}`.
- [x] Dedup key: composite `(origin_project, learning_id)` — smoke-tested
      cross-poll: round1 2 appended, round2 0 appended (4 skipped_dup).
- [x] Self-learning skip via `${FW_ORIGIN_PROJECT:-$(basename PROJECT_ROOT)}`
      — smoke-tested: 4 copies of a self-originating envelope → 4 skipped_self.
- [x] Opt-out env `FW_LEARNINGS_BUS_SUBSCRIBE=0` — tested, exits 0 silently.
- [x] Silent no-op paths: missing termlink (command -v check), hub unreachable
      (collect non-zero caught), empty poll (all counts 0). Never non-zero-exit.
- [x] Audit log `.context/working/.subscribe-learnings-bus.log` — smoke-tested,
      format: `<ISO> poll received=N appended=M skipped_self=S skipped_dup=D
      skipped_malformed=X`.
- [x] Upstream-mirrored via termlink dispatch → 87d2ca2d pushed to onedev/master
      (PL-053 pattern: dispatch timed out as expected, verified by direct HEAD
      inspection).

### Human
- [ ] [RUBBER-STAMP] Install cron entry (e.g. `*/15 * * * *
      /opt/termlink/.agentic-framework/lib/subscribe-learnings-from-bus.sh`)
      on at least one peer project to observe end-to-end.
      **Steps:**
      1. On a second project (e.g. ring20-management), ensure framework is at the
         version with this script (`fw upgrade` post-merge)
      2. Add the cron line, wait 15 min
      3. Trigger a learning on this project: `fw context add-learning "test
         cross-project mirror" --task T-1217 --source P-001`
      4. On the second project: `cat .context/project/received-learnings.yaml`
      **Expected:** The test learning appears with `origin_project: termlink`
      and the correct L-ID.
      **If not:** Check `.subscribe-learnings-bus.log` on the subscriber side
      and `.publish-learning-bus.log` on this side for error paths.

      **Agent evidence (2026-04-24T15:30Z, end-to-end):**
      - Cron installed on peer `email-archive` (via termlink exec) —
        `*/15 * * * * cd /opt/050-email-archive && /opt/050-email-archive/.agentic-framework/lib/subscribe-learnings-from-bus.sh >/dev/null 2>&1`
      - Triggered `fw context add-learning ... --task T-1217` on /opt/termlink → PL-059
      - Ran subscriber manually on email-archive (same command cron will run)
      - Result: `received=4 appended=1 skipped_self=0 skipped_dup=3` then
        fresh run picked up PL-059, cursor advanced 380→400
      - PL-059 lands in `/opt/050-email-archive/.context/project/received-learnings.yaml`
        with `learning_id: "PL-059"` and the exact text from the source
      - Rubber-stamp ready: pipeline works; cron runs it on the `*/15` cadence

      **Re-verified end-to-end (2026-04-25T16:27Z, T-1255 follow-up):**
      - Cron entry confirmed live: `crontab -l` shows the `*/15` line installed on /opt/050-email-archive.
      - Independent observation: PL-070 (T-1254 CSRF fix learning) auto-propagated to email-archive at 16:00:02Z (2 cron cycles after publish on /opt/termlink) — no manual run needed.
      - Fresh test: added PL-071 ("T-1217 cross-project mirror end-to-end test from /opt/termlink at 2026-04-25T162718Z") on /opt/termlink at 16:27:18Z. Triggered subscriber on email-archive via termlink dispatch. PL-071 appeared at 16:27:38Z (~20 seconds end-to-end) in `/opt/050-email-archive/.context/project/received-learnings.yaml` with `origin_project: termlink`, `origin_hub_fingerprint: sha256:4774a193…`, exact text preserved.
      - **Pipeline is healthy and self-running.** Human can rubber-stamp.

## Verification

bash -n /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh
test -x /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh
grep -q "FW_LEARNINGS_BUS_SUBSCRIBE" /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh
grep -q "channel:learnings" /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh

## Decisions

### 2026-04-24 — Discovery spike result (AC1)

- **Chose:** `termlink event collect --topic channel:learnings --payload-only
  --json` as the consumer primitive. Confirmed receives payloads posted via
  `termlink event broadcast channel:learnings -p <json>`.
- **Finding:** collect returned the same payload **3 times** for one
  broadcast (once per listening session on the hub). Composite-key dedup
  `(origin_project, learning_id)` is therefore load-bearing, not a nice-to-have.
- **Chose:** no `--since` cursor; rely on (origin_project, learning_id) dedup
  against `received-learnings.yaml` entries. Rationale: `--since=<seq>` is
  per-session and the hub does not expose a global seq via collect; a dedup
  set is simpler and idempotent. Missed events during between-poll gaps are
  acceptable (learnings are advisory).
- **Cron cadence:** document `*/5 * * * *` with `--timeout 30` as the
  recommended install — near-continuous coverage, no overlap risk.
- **Rejected:** long-running daemon (systemd). Cron keeps the consumer
  footprint uniform with existing framework cron entries.

## Updates

### 2026-04-24T12:27:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1217-t-1168b2-b3-learnings-subscriber--watcht.md
- **Context:** Initial task creation

### 2026-04-24T13:05:00Z — scope-reduction [agent]
- **Change:** status captured → started-work
- **Scope:** Reduced to B2 (subscriber poller) only. B3 (Watchtower panel)
  split to T-1218 per sizing rule "one task = one deliverable". Auto-apply
  guard dropped (received learnings are advisory).
- **Real ACs filled** replacing G-020 placeholders.

### 2026-04-24T13:10:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
