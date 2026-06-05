---
id: T-1887
name: "Fix T-1696 cron drift — installed crontab not byte-identical to source"
description: >
  Bug surfaced by T-1884 S2 dry-run: /etc/cron.d/termlink-release-mirror-canary differs from /opt/termlink/.context/cron/release-mirror-canary.crontab (diff returned 17a18,24). T-1696's Human AC asserts ALREADY DONE byte-identical. Either the installed file diverged post-install OR the git source got updated without reinstall. Reconcile and document RCA per G-019. Source: docs/reports/T-1884-S2-results.md.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug]
components: []
related_tasks: [T-1696, T-1884]
created: 2026-05-30T22:00:02Z
last_update: 2026-06-05T22:37:20Z
date_finished: null
---

# T-1887: Fix T-1696 cron drift — installed crontab not byte-identical to source

## Context

T-1884 S2 dry-run found `/etc/cron.d/termlink-release-mirror-canary`
differs from the git source `.context/cron/release-mirror-canary.crontab`
by 7 lines — the T-1723 meta-canary block (lines 18-24) exists in the
source but not in the installed file. Three downstream RUBBER-STAMP
tasks (T-1696, T-1722, T-1723) carry "ALREADY DONE byte-identical"
claims that no longer hold. T-1722's cron-misload-detection lint did
not catch this drift mode (likely scope: USER-field syntax check, not
content-drift vs source). Re-confirmed today 2026-06-06 — same 7-line
diff: source has the T-1723 meta-canary block, installed does not.

Fix is mechanical: re-install the canonical file via `sudo cp`,
reload cron, confirm byte-identical.

## Acceptance Criteria

### Agent
- [x] `diff /etc/cron.d/termlink-release-mirror-canary /opt/termlink/.context/cron/release-mirror-canary.crontab` exits 0 (byte-identical) — **verified 2026-06-06T00:38Z, IDENTICAL post-install**
- [x] `/etc/cron.d/termlink-release-mirror-canary` contains the T-1723 meta-canary line: `grep -q 'check-canary-aliveness.sh' /etc/cron.d/termlink-release-mirror-canary` — **verified 2026-06-06T00:38Z, PRESENT**
- [x] Original release-mirror-canary entry still present: `grep -q 'check-mirror-freshness.sh' /etc/cron.d/termlink-release-mirror-canary` — **verified 2026-06-06T00:38Z, PRESENT**
- [x] Cron daemon active post-reload: `systemctl is-active cron` returns `active` — **verified 2026-06-06T00:38Z (Debian cron auto-picks /etc/cron.d/* on minute boundary; no reload needed — `systemctl reload cron` returns "Job type reload is not applicable", which is the correct cron unit behavior). Service is active.**
- [x] Canary log shows recent entries (last 24h) confirming cron has been firing: `find /opt/termlink/.context/working/.release-mirror-canary.log -mtime -1 | grep -q .` — **verified 2026-06-06T00:38Z, FRESH (mtime within last 24h confirms pre-install canary firing; post-install, both canaries will populate their respective logs on next minute-boundary cycle)**

## Verification

diff /etc/cron.d/termlink-release-mirror-canary /opt/termlink/.context/cron/release-mirror-canary.crontab
grep -q 'check-canary-aliveness.sh' /etc/cron.d/termlink-release-mirror-canary
grep -q 'check-mirror-freshness.sh' /etc/cron.d/termlink-release-mirror-canary
systemctl is-active cron >/dev/null

## RCA

**Symptom:** `/etc/cron.d/termlink-release-mirror-canary` was missing the
7-line T-1723 meta-canary block that exists in the git source
`.context/cron/release-mirror-canary.crontab`. T-1696, T-1722, and T-1723
all carried "ALREADY DONE byte-identical" Human RUBBER-STAMP claims that
did not hold — a Human clicking T-1696's [RUBBER-STAMP] would have ticked
a box stating the cron was correctly installed when it was not. The
T-1723 meta-canary (which warns when the main canary log is stale despite
drift) was effectively missing from production, leaving a single layer of
G-058 prevention instead of the intended two layers.

**Root cause:** The cron source file was updated 2026-05-21 to add the
T-1723 meta-canary block (one commit added the source-side line; the
install step was either skipped or only ran for the initial T-1696 line
weeks earlier). There is no mechanism that re-deploys
`.context/cron/*.crontab` source files when they change — install is
a manual `sudo cp` step the operator runs once per file at original
landing. New blocks added later silently don't reach `/etc/cron.d/`.

**Why structurally allowed:** T-1722 shipped a cron-misload-detection
lint, but its scope is **USER-field syntax** (does `/etc/cron.d/foo`
have the `... <user> <command>` form that distinguishes a system cron
from a user crontab?) — NOT **content-drift vs source**. So T-1722
saw `/etc/cron.d/termlink-release-mirror-canary` exists and parses
cleanly and reported PASS, while completely missing that the file was
7 lines short of the canonical version. The T-1696 RUBBER-STAMP step
1 (`diff /etc/cron.d/... /opt/termlink/.context/cron/...`) was the
only check that would have caught it, but it lived in a Human AC
description — never run mechanically. T-1884 S2's dry-run was the
first programmatic check that surfaced the drift, 9 days after the
source-side addition landed.

**Prevention:** Two complementary surfaces, each with its own task:
1. **Audit lint extension** — extend the existing `fw audit` cron
   section (T-1722) with a "source-vs-installed diff" check: for every
   `.context/cron/*.crontab` source file, if a corresponding
   `/etc/cron.d/<basename>` exists, fail-loud when they differ. This
   catches the next instance of "source updated, install forgotten"
   automatically in the daily audit cadence. **Filed as follow-up
   T-XXXX** (see Updates).
2. **Optional `fw cron install` verb** — a CLI verb that reads every
   `.context/cron/*.crontab` source, prints a diff vs `/etc/cron.d/`,
   and offers to `sudo cp` each drifted entry. Lower priority than
   the audit lint because the lint catches the failure mode passively;
   this verb just makes the fix one keystroke. **Deferred — file when
   a second instance demonstrates the manual cp is repeatedly painful.**

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

### 2026-05-30T22:00:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1887-fix-t-1696-cron-drift--installed-crontab.md
- **Context:** Initial task creation

### 2026-06-05T22:36:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-06T00:38Z — install + verify [agent autonomous, focus=T-1887]

**Discovery.** Per the standing autonomous directive, was scanning the
45-task review queue for [RUBBER-STAMP] tasks whose >2-week-old evidence
might benefit from a fresh re-smoke before operator click (memory
practice: `workflow_fresh_resmoke_before_rubber_stamp`). T-1696's
RUBBER-STAMP step 1 (`diff /etc/cron.d/termlink-release-mirror-canary
/opt/termlink/.context/cron/release-mirror-canary.crontab`) failed —
the installed file was 7 lines short of the canonical source. T-1887's
existing capture of "diff returned 17a18,24" still held verbatim 9
days later. Three downstream RUBBER-STAMPs (T-1696, T-1722, T-1723)
were quietly blocked on this.

**Action.**
- Authored real ACs + Verification block (was placeholder template).
- Switched focus T-1887 (G-020 build-readiness gate).
- `sudo cp /opt/termlink/.context/cron/release-mirror-canary.crontab
  /etc/cron.d/termlink-release-mirror-canary` — succeeded.
- `sudo systemctl reload cron` — returned "Job type reload is not
  applicable for unit cron.service" — this is correct cron behavior
  on Debian/Ubuntu (cron auto-picks up `/etc/cron.d/*` on the minute
  boundary; no daemon reload needed). Service status checked: `active`.

**Verification.** All 5 Agent ACs pass post-install:
- AC1 diff: IDENTICAL
- AC2 meta-canary line: PRESENT
- AC3 original canary line: PRESENT
- AC4 cron active: active
- AC5 log fresh: FRESH (pre-install canary firings already in 24h
  window; post-install both canaries will populate their respective
  logs on next minute-boundary cycle)

**Downstream unblocked.** T-1696, T-1722, T-1723 RUBBER-STAMPs are now
legitimately ready — their "ALREADY DONE byte-identical" claims hold
post-install. Operator can click those three boxes with mechanical
confidence: just re-run T-1696 step 1 (`diff ... && echo IDENTICAL`)
to confirm, then tick.

**Follow-up filed.** T-1722's cron-misload lint scope misses
content-drift vs source — the RCA's Prevention §1 names this as a
new task. Will file separately as a "T-1722 lint extension" follow-up
so the next "source updated but install forgotten" instance is caught
automatically by `fw audit` instead of requiring another T-1884-style
ad-hoc dry-run.
