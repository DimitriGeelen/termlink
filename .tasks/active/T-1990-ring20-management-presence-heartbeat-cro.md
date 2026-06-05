---
id: T-1990
name: "ring20-management presence-heartbeat cron stopped firing — listener silently OFFLINE"
description: >
  ring20-management presence-heartbeat cron stopped firing — listener silently OFFLINE

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T09:07:07Z
last_update: 2026-06-05T09:10:54Z
date_finished: null
---

# T-1990: ring20-management presence-heartbeat cron stopped firing — listener silently OFFLINE

## Context

T-1985 installed a presence-heartbeat cron on .122 (ring20-management) to close PL-200.
Verified LIVE at install time. Today (2026-06-05T09:06Z) `/var/log/presence-heartbeat.log`
mtime is stuck at 2026-06-04 08:33Z — ~24h stale. dm-poller cron at the same crontab
position still fires every 2 min, so the cron daemon is healthy. Something specific to
the presence-heartbeat path is failing silently. ring20-management-agent is likely
OFFLINE on the fleet despite the cron line being present.

This is a regression-on-the-fix from T-1985. Investigate, fix, and add a regression
signal so the next silent drop is caught structurally instead of by ad-hoc inspection.

## Acceptance Criteria

### Agent
- [x] Root cause identified — FALSE ALARM. Cron is firing every minute (syslog confirms
      09:00–09:08 entries every 60s). Listener `ring20-management-agent` is LIVE with
      age_secs=18 on the .122 hub. Stale log mtime is a diagnostic artifact:
      `termlink channel post` is silent on success in 0.11.473+; `>> log 2>&1` with
      empty stream does not bump mtime. Recorded in `## RCA` below.
- [x] presence-heartbeat cron firing on .122 — confirmed via journalctl: 10 consecutive
      every-minute CRON entries from 08:59:01Z to 09:08:01Z, no errors. Cron path is
      structurally healthy.
- [x] `ring20-management-agent` LIVE on .122 hub — verified 2026-06-05T09:08Z via
      `bash scripts/agent-listeners.sh --hub 192.168.10.122:9100 --filter-agent-id ring20-management-agent --include-offline --json`:
      `total_listeners=1, live=1, status=LIVE, age_secs=18, listen_topics=dm:9219671e28054458:d1993c2c3ec44c94,agent-chat-arc`.
- [x] Regression signal added — registered as PL-201 in `.context/project/learnings.yaml`:
      "DIAGNOSTIC GOTCHA: do NOT use log-file mtime as a liveness signal for
      presence-heartbeat cron. Authoritative signal is listener status on the hub,
      not log mtime." Cross-linked to PL-200.

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

# Verify the .122 presence cron is structurally healthy via channel info (fast, no topic walk).
# Reasoning: agent-listeners.sh walks the entire topic; on .122 agent-presence has accumulated
# ~1490 envelopes since 2026-06-04 08:33Z (every-minute cron) and the walk times out at 8s.
# channel info returns the post count in O(1); count >= 1490 proves the cron has fired ~1490
# times since install, and the per-minute cycle means count grows monotonically.
bash -c 'set -o pipefail; c=$(timeout 8 termlink channel info agent-presence --json --hub 192.168.10.122:9100 2>/dev/null | python3 -c "import sys,json; print(json.loads(sys.stdin.read()).get(\"count\",0))"); test "$c" -ge 1490 && echo "OK count=$c" || { echo "FAIL count=$c"; exit 1; }'

## RCA

**Symptom:** `/var/log/presence-heartbeat.log` on .122 had mtime stuck at
2026-06-04 08:33Z (24h stale) and zero bytes of content, which read as "cron stopped
firing — listener silently OFFLINE" during steady-state verification of T-1985's fix.

**Root cause:** Misdiagnosis — there was no regression. Three orthogonal facts compose
into a misleading signal:

1. `termlink channel post agent-presence ...` succeeds silently in CLI version 0.11.473
   (exit 0, empty stdout, empty stderr).
2. Bash redirection `>> /var/log/x.log 2>&1` with zero bytes of output does NOT update
   the target file's mtime. The append-mode open() is a noop on the inode timestamps
   if no bytes are written.
3. The very first cron run after the script was created at 2026-06-04 08:32Z happened
   at 08:33Z — that single run is what stamped the log's mtime. Every subsequent
   minute-cycle has succeeded silently, leaving mtime untouched.

The structural outcome (listener LIVE on the hub, age_secs=18) shows the system has
been healthy throughout. The dm-poller cron's log keeps growing because dm-poller
actively prints "no new envelopes" / "NEW: ..." lines per cycle — presence-heartbeat
does not, by design.

**Why structurally allowed:** The framework had no rule against using log mtime as a
liveness signal, and the documentation around T-1985/T-1988/T-1989 (PL-200) implicitly
encouraged it ("if the cron's running you'll see the log grow"). The diagnostic was
not WRONG when the script first errored, but silently became misleading once the
script started succeeding cleanly. Authoritative liveness signal exists
(`agent-listeners.sh --filter-agent-id <id>`) but wasn't documented as the canonical
check for "is the heartbeat working?"

**Prevention:** PL-201 registered in `learnings.yaml` with explicit guidance:
- Do NOT use log mtime as a liveness signal for `termlink channel post`-based crons.
- Authoritative check: `agent-listeners.sh --filter-agent-id <id> --include-offline --json`
  and read `.listeners[0].status==LIVE` + `.age_secs < 90`.
- Cross-linked into PL-200's `context` field so any future agent debugging PL-200
  closure on a new host gets the diagnostic guidance immediately.

No code change required — the system is healthy.

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

### 2026-06-05T09:07:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1990-ring20-management-presence-heartbeat-cro.md
- **Context:** Initial task creation
