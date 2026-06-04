---
id: T-1988
name: "PL-200 fleet coverage validation + .141 closure (T-1987 sibling)"
description: >
  PL-200 prevention follow-up. (1) End-to-end validate the .122 doorbell+mail rail (T-1987 cron) by sending a test DM from this .107 session and confirming it lands in /var/log/dm-inbox.log on .122 within 2 minutes. (2) Apply the same presence-heartbeat cron pattern to .141 laptop-141 — last fleet host with the PL-200 gap (per memory, ~4d old). On success, peers can /agent-handoff to laptop-141-agent and the fleet listener-discovery is complete. Both deliverables are autonomous-safe (additive, recoverable cron entries; no source code changes; no destructive ops).

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: [pl-200, doorbell-mail, ring20, fleet]
components: []
related_tasks: []
created: 2026-06-04T19:57:51Z
last_update: 2026-06-04T19:57:51Z
date_finished: null
---

# T-1988: PL-200 fleet coverage validation + .141 closure (T-1987 sibling)

## Context

T-1987 closed the .122 dm-poller cron (every 2 min, surfaces inbound DMs to /var/log/dm-inbox.log). T-1985 closed the .122 presence-heartbeat cron (every minute, makes ring20-management-agent discoverable). Both verified LIVE post-handover (listener age 26s, dm-poller log clean every 2 min). Two open follow-ups: (1) the .122 rail has never had an end-to-end DM-from-peer test — only the historical 36-envelope backlog has been surfaced. (2) PL-200 documents that post-binary-swap presence-emitter is not auto-restored — .141 laptop-141 still has the gap per memory (~4d old).

## Acceptance Criteria

### Agent
- [ ] Test DM posted from this .107 session to ring20-management-agent's DM topic via `--hub 192.168.10.122:9100` (or local hub with federation gated)
- [ ] Within 4 minutes (2 dm-poller cycles), the test DM payload appears in /var/log/dm-inbox.log on .122 (verified via remote exec)
- [ ] .141 laptop-141 has /root/termlink/scripts/presence-heartbeat.sh installed (matches .122 template; agent_id=laptop-141-agent, listen_topics include dm:<141-fp>:*,agent-chat-arc)
- [ ] .141 has `* * * * * /root/termlink/scripts/presence-heartbeat.sh ...` in its crontab (idempotent — checked before append)
- [ ] Within 90 seconds of cron install, laptop-141-agent appears LIVE in `bash scripts/agent-listeners-fleet.sh` output
- [ ] PL-200 learning gets a status-update entry confirming `.122 + .141` fleet coverage achieved

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
