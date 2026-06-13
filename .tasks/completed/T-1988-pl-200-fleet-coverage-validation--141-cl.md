---
id: T-1988
name: "PL-200 fleet coverage validation + .141 closure (T-1987 sibling)"
description: >
  PL-200 prevention follow-up. (1) End-to-end validate the .122 doorbell+mail rail (T-1987 cron) by sending a test DM from this .107 session and confirming it lands in /var/log/dm-inbox.log on .122 within 2 minutes. (2) Apply the same presence-heartbeat cron pattern to .141 laptop-141 — last fleet host with the PL-200 gap (per memory, ~4d old). On success, peers can /agent-handoff to laptop-141-agent and the fleet listener-discovery is complete. Both deliverables are autonomous-safe (additive, recoverable cron entries; no source code changes; no destructive ops).

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: [pl-200, doorbell-mail, ring20, fleet]
components: []
related_tasks: []
created: 2026-06-04T19:57:51Z
last_update: 2026-06-04T19:58:34Z
date_finished: 2026-06-04T20:16:25Z
---

# T-1988: PL-200 fleet coverage validation + .141 closure (T-1987 sibling)

## Context

T-1987 closed the .122 dm-poller cron (every 2 min, surfaces inbound DMs to /var/log/dm-inbox.log). T-1985 closed the .122 presence-heartbeat cron (every minute, makes ring20-management-agent discoverable). Both verified LIVE post-handover (listener age 26s, dm-poller log clean every 2 min). Two open follow-ups: (1) the .122 rail has never had an end-to-end DM-from-peer test — only the historical 36-envelope backlog has been surfaced. (2) PL-200 documents that post-binary-swap presence-emitter is not auto-restored — .141 laptop-141 still has the gap per memory (~4d old).

## Acceptance Criteria

### Agent
- [x] Test DM posted from this .107 session to ring20-management-agent's DM topic via `--hub 192.168.10.122:9100` (offset=30, ts=1780603129417, nonce=T1988-end2end-20260604-195849Z)
- [x] Within 4 minutes (2 dm-poller cycles), the test DM payload appears in /var/log/dm-inbox.log on .122 (poller fired at 20:00:01Z, new_envelopes=1, payload + nonce surfaced with sender attribution `d1993c2c3ec44c94 (010-termlink)`)
- [x] .141 laptop-141 has presence-heartbeat.sh installed (at `/mnt/c/ntb-acd-plugin/termlink/scripts/presence-heartbeat.sh` — script path differs from .122 due to .141's WSL2 layout, but agent_id=laptop-141-agent, listen_topics=dm:6604a2af482f0cf7:*,agent-chat-arc both correct)
- [x] .141 has `* * * * * PATH=... /mnt/c/ntb-acd-plugin/termlink/scripts/presence-heartbeat.sh ...` in dimitri's crontab (idempotent: `grep -qF "presence-heartbeat.sh"` guard before append)
- [x] laptop-141-agent appears LIVE in `bash scripts/agent-listeners-fleet.sh` output (90s after PL-146 fix landed; first attempt with .122-template-clone failed silently due to PL-146)
- [x] PL-200 learning got a status-update entry confirming `.122 + .141` fleet coverage + PL-146 cron-env gotcha + .121 flagged for follow-up

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
# Verify laptop-141-agent is LIVE (presence cron firing post-install)
set -o pipefail; bash scripts/agent-listeners-fleet.sh --include-offline --json 2>/dev/null | python3 -c "import sys,json; raw=sys.stdin.read(); lines=raw.strip().split('\n'); data=json.loads('\n'.join(lines[next(i for i,ln in enumerate(lines) if ln.startswith('{') or ln.startswith('[')):])); print('OK' if any(l.get('agent_id')=='laptop-141-agent' and l.get('status')=='LIVE' for l in data.get('listeners',[])) else 'MISSING')" | grep -q "^OK$"

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

### 2026-06-04T19:57:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1988-pl-200-fleet-coverage-validation--141-cl.md
- **Context:** Initial task creation

### 2026-06-04T20:00Z — .122 end-to-end rail proven
- **Posted:** `dm:9219671e28054458:d1993c2c3ec44c94` --hub 192.168.10.122:9100 offset=30 nonce=T1988-end2end-20260604-195849Z
- **Surfaced:** /var/log/dm-inbox.log on .122 via dm-poller fire at 20:00:01Z (`new_envelopes=1`)
- **Sender attribution:** d1993c2c3ec44c94 (010-termlink) — confirms PL-195 shared-host signing
- **T-1987 status:** end-to-end validated (was only verified for historical 36-envelope backlog before)

### 2026-06-04T20:13Z — .141 install + PL-146 gotcha
- **Discovered:** .122 template doesn't port cleanly to .141 — cron runs as dimitri (UID 1000), `${HOME:-/root}/.termlink/runtime/hub.sock` resolution needed (PL-146). Cloning .122 script verbatim caused silent post-queueing with `Queued to agent-presence — queue_id=22 (hub unreachable; will flush on next reconnect)` — every cron fire enqueued locally instead of delivering.
- **Fix:** mirrored the runtime-dir fallback block from `scripts/vendored-arc-heartbeat.sh` (the working T-1438 hourly chat-arc heartbeat). One sim-cron test drained 4 queued posts + posted offset=5. Next live cron cycle landed clean (laptop-141-agent LIVE, age=18s).
- **Files touched on .141:** `/mnt/c/ntb-acd-plugin/termlink/scripts/presence-heartbeat.sh` + dimitri crontab append (T-1988 comment + cron line)
- **PL-200 updated:** status entry with PL-146 gotcha + fleet snapshot + .121 flagged for T-1989 follow-up.

### 2026-06-04T20:14Z — fleet status snapshot
- 2 LIVE: ring20-management-agent (.122, T-1985) + laptop-141-agent (.141, T-1988)
- 1 OFFLINE: root-claude-dimitrimintdev (.107, age ~1.4d — be-reachable not active in this session, ephemeral; not PL-200)
- .121 ring20-dashboard: no listener entry at all — same PL-200 gap as .141 was, T-1989 candidate
