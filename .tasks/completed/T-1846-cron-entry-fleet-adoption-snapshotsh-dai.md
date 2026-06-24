---
id: T-1846
name: "Cron entry: fleet-adoption-snapshot.sh daily HOT/WARM/COLD log (T-1843 follow-on)"
description: >
  Persist daily adoption_state log via cron, symmetric to T-1831 doorbell+mail-canary. Adoption is a positive signal so capture ALL output (not just drift). Operators get historical visibility into COLD/WARM/HOT transitions.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [doorbell-mail, cron, adoption, t-1843-followon]
components: []
related_tasks: []
created: 2026-05-28T18:36:48Z
last_update: 2026-05-28T18:41:48Z
date_finished: 2026-05-28T18:41:48Z
---

# T-1846: Cron entry: fleet-adoption-snapshot.sh daily HOT/WARM/COLD log (T-1843 follow-on)

## Context

T-1843 shipped `scripts/fleet-adoption-snapshot.sh` — a distinct gauge from the T-1831 health canary. The canary asks "is the rail healthy?" (drift-only log: empty=healthy); this script asks "is the rail being USED?" (HOT/WARM/COLD adoption_state). Without persistent history, an operator can't see drift in adoption over time — only the current snapshot. This task wires the snapshot into cron daily, symmetric to T-1831's installed cron entry.

## Acceptance Criteria

### Agent
- [x] `.context/cron/fleet-adoption-snapshot.crontab` (NEW) — single daily cron entry under USER-field syntax (`root` user). Window default 24h. Non-colliding with existing schedules (:13 mirror-canary, :17 heartbeat, :23 doorbell-mail-canary, :33 meta-canary, :43 rollout audit) — pick a fresh minute slot.
- [x] Installed copy at `/etc/cron.d/termlink-fleet-adoption-snapshot` matches the source.
- [x] Log target: `.context/working/.fleet-adoption-snapshot.log` — append-mode, full output (NOT --quiet — adoption is a positive signal so all output captured). Heartbeat file: `.context/working/.fleet-adoption-snapshot.heartbeat`.
- [x] Fabric: cron entry registered in fw cron registry (or audit passes — `fw audit` reports no new cron drift).
- [x] First-run smoke: manually invoke the cron command line; expect single-line HOT/WARM/COLD summary appended to log.

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

test -f .context/cron/fleet-adoption-snapshot.crontab
test -f /etc/cron.d/termlink-fleet-adoption-snapshot
diff -q .context/cron/fleet-adoption-snapshot.crontab /etc/cron.d/termlink-fleet-adoption-snapshot
grep -qE '^[0-9]+ [0-9]+ \* \* \* root' /etc/cron.d/termlink-fleet-adoption-snapshot
test -s .context/working/.fleet-adoption-snapshot.log

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

### 2026-05-28T18:36:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1846-cron-entry-fleet-adoption-snapshotsh-dai.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-220ae633
- **Timestamp:** 2026-05-28T18:41:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T18:41:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
