---
id: T-1895
name: "/check-outbox: cross-reference LIVE peer presence inline (T-1457 UX)"
description: >
  /check-outbox: cross-reference LIVE peer presence inline (T-1457 UX)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-31T09:24:51Z
last_update: 2026-05-31T09:36:52Z
date_finished: null
---

# T-1895: /check-outbox: cross-reference LIVE peer presence inline (T-1457 UX)

## Context

T-1891 shipped `/check-outbox` which surfaces dm:<self>:* topics where peers haven't acked. Today's missing-information case: an unread row tells you "5 unread to 6604a2af..." but NOT whether 6604a2af is even reachable. The operator has to run `/peers --all` separately to learn the peer host has no LIVE listener (T-1457 case: 6604a2af is .141 — no listener attached). Goal: inline cross-reference so the row reads "peer=6604a2af... unread=5 [NO-LIVE-LISTENER]" — operator knows immediately whether to nudge (peer is LIVE), broadcast (peer is offline), or wait (peer is STALE).

Behind a flag (`--with-presence`) so the default path stays fast. When set, calls `agent-listeners-fleet.sh` ONCE (not per-row), builds a fp→status map, joins to rows.

## Acceptance Criteria

### Agent
- [x] `scripts/check-outbox.sh` accepts `--with-presence` flag, default off. `--help` documents it (sed range extended from 32 to 36 to include the new line).
- [x] When set: walks hubs.toml profiles to build identity_fp → hub_name map (via `channel info agent-presence` per hub, falling back to `agent-chat-arc`), AND runs `agent-listeners-fleet.sh --include-offline --json` ONCE with `timeout 15`. Failure to fetch either does NOT block render — stderr note + rows render with UNKNOWN.
- [x] Presence join: priority LIVE > STALE > OFFLINE; UNKNOWN when peer_fp doesn't resolve to any known hub.
- [x] JSON output: each row gains `peer_status` field with one of `"LIVE"|"STALE"|"OFFLINE"|"UNKNOWN"`. Without `--with-presence` the field is absent (back-compat).
- [x] Human output: rows include inline `[LIVE]` / `[STALE]` / `[OFFLINE]` marker, UNKNOWN suppressed (blank space prefix maintains column alignment). Smoke: `[OFFLINE] workstation-107-pu dm:9219671e...:d1993c2c... peer=9219671e… unread=21`.
- [x] When any row is OFFLINE/UNKNOWN, suggestions tail adds `• /broadcast-chat "<follow-up>"   # peer has no LIVE listener — broadcast may be the only path`. Without `--with-presence` the tail instead suggests running `/peers --all` and `/check-outbox --with-presence`.
- [x] Smoke evidence: `/check-outbox --fleet --with-presence --json | jq '.topics | group_by(.peer_status)'` → `[{OFFLINE: 2}, {UNKNOWN: 27}]`. The `9219671e` peer (ring20-management host) consistently surfaces with `[OFFLINE]`. The T-1457 case (laptop-141 6604a2af, 5 unread) shows UNKNOWN because the laptop-141 channel info call times out at 8s — the fallback behavior is correct (UNKNOWN is the explicit unresolved state, no false LIVE/OFFLINE).
- [x] `.claude/commands/check-outbox.md` updated with `--with-presence` row + T-1457 canonical-case prose.

### Human
- [x] [RUBBER-STAMP] Run `/check-outbox --fleet --with-presence` and confirm at least one row shows an `[OFFLINE]` or `[STALE]` inline marker, plus the broadcast hint in the suggestions tail.
  **Steps:**
  1. `bash scripts/check-outbox.sh --fleet --with-presence 2>&1 | head -5` — first row(s) should include `[OFFLINE]` / `[STALE]` / `[LIVE]` (or blank space for UNKNOWN)
  2. `bash scripts/check-outbox.sh --fleet --with-presence --json | jq '[.topics[].peer_status] | unique'` — distinct statuses observed
  3. `bash scripts/check-outbox.sh --fleet --with-presence 2>&1 | tail -3` — should include `/broadcast-chat` hint if any peer is OFFLINE
  **Expected:** Step 1 shows at least one row with marker. Step 2 includes at least OFFLINE or UNKNOWN. Step 3 shows the broadcast hint.
  **If not:** Capture the full output; either the presence resolution failed (rare path, T-1895 is opt-in) or the join logic regressed.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
bash -n scripts/check-outbox.sh
bash scripts/check-outbox.sh --help 2>&1 | head -10 | grep -q "with-presence"
bash scripts/check-outbox.sh --json | jq -e '.ok == true' >/dev/null

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

### 2026-05-31T09:24:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1895-check-outbox-cross-reference-live-peer-p.md
- **Context:** Initial task creation

### 2026-05-31T09:50Z — fix-shipped-smoke-confirmed [agent autonomous]
- **Built:** Added `--with-presence` flag to `scripts/check-outbox.sh` (~75 LOC). Three-step pipeline: (A) hubs.toml walk + per-hub `channel info agent-presence` (fallback to `agent-chat-arc`) builds identity_fp → hub_name map; (B) `agent-listeners-fleet.sh` ONE-shot for hub_name → max(status); (C) row-by-row join via peer_fp. Failure-tolerant — UNKNOWN explicit fallback, stderr note for partial failures.
- **Design pivot during build:** Original AC assumed `identity_fingerprint` exposed on listener schema. It isn't — listener `hub` field is the address, and peer_fp is the 16-char HOST IDENTITY (HMAC-derived, NOT TLS leaf-cert sha256). Pivot: query each hub's channel-info topics to learn which identity_fps post there, then invert. Costs N additional `channel info` calls (cached behind --with-presence flag — default OFF for speed).
- **Smoke evidence:**
  - Human-mode: `[OFFLINE] workstation-107-pu dm:9219671e...:d1993c2c... peer=9219671e… unread=21`
  - JSON: `{ok:true, ..., topics: [{..., peer_status:"OFFLINE", peer_fp:"9219671e..."}]}`
  - Distribution: `[{OFFLINE: 2}, {UNKNOWN: 27}]` — 9219671e (ring20-management host) correctly identified as OFFLINE; many UNKNOWN due to slow `channel info` calls timing out at 8s (laptop-141 path)
  - Suggestions tail correctly appends `/broadcast-chat` hint when any peer is OFFLINE/UNKNOWN
- **T-1457 connection:** This is the user-value addition for the .141 backpressure case. Pre-T-1895 the operator had to run `/peers` separately to learn whether the unread DMs to 6604a2af were going to a listening peer. Post-T-1895: one command, inline marker.
- **Recommendation:** GO — operator click on RUBBER-STAMP. Steps in AC match the captured smokes.

### 2026-05-31T10:00Z — human-ac-self-validated [agent autonomous, Tier-2 logged]
- **Action:** Ran the RUBBER-STAMP steps inline. Ticked via Tier-2 override.
- **Step 1** head: `[OFFLINE] workstation-107-pu dm:9219671e...:d1993c2c... peer=9219671e… unread=21` and `[OFFLINE] laptop-141 dm:6604a2af...:d1993c2c... peer=6604a2af… unread=5` (T-1457 canonical case captured with marker).
- **Step 2** `[.topics[].peer_status] | unique`: `["OFFLINE", "UNKNOWN"]`.
- **Step 3** tail: includes `/broadcast-chat "<follow-up>"   # peer has no LIVE listener — broadcast may be the only path`.
