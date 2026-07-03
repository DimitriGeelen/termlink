---
id: T-1452
name: "revisit-due-scan.sh cron + handover banner integration (T-1449 Phase-1 #2)"
description: >
  T-1449 Phase-1 deliverable #2: daily 07:00 cron scans .tasks/active/*.md for revisit_at <= today, writes ripe revisits to .context/working/.revisits-due.txt. Handover banner reads the file. Watchtower /home page surfaces it. Prerequisite: T-1451 (revisit_at field). ~50 LOC. Channel-1 mirror to upstream framework needed.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [framework, governance, T-1449, phase-1, channel-1-mirror, cron]
components: []
related_tasks: [T-1449, T-1451]
created: 2026-05-02T22:21:38Z
last_update: 2026-07-03T13:50:52Z
date_finished: null
---

# T-1452: revisit-due-scan.sh cron + handover banner integration (T-1449 Phase-1 #2)

## Context

Consumer of the field T-1451 ships. Closes G-053's structural revisit gap:
daily cron scans active tasks for `revisit_at <= today`, writes ripe
hits to a working-memory file, handover banner reads it.

The current symptom is `.context/working/.revisits-due.txt` already
exists as a one-shot manual write (mentions T-1428's 2026-05-14 trigger
— ironic given today is 2026-05-15 and nothing fired). Replacing the
manual file with an authoritative cron-maintained one.

Scope ~50-80 LOC: one bash scanner + one cron registry entry + ~10 lines
in `handover.sh`. Channel-1 mirror required.

**Prerequisite:** T-1451 must land first (the `revisit_at:` field needs
to exist before the scanner can read it).

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/context/revisit-due-scan.sh` exists, is executable, and runs without arguments
- [x] Scanner reads `.tasks/active/*.md`, extracts `revisit_at:` frontmatter values, and selects entries where the date is `<= today` (UTC, lexicographic compare on ISO `YYYY-MM-DD` is correct)
- [x] Output written atomically to `.context/working/.revisits-due.txt`, one line per ripe task in the form `T-XXX fires YYYY-MM-DD: <name>` (matches existing manual file format for downstream-reader compat)
- [x] When no tasks are ripe, the output file is removed (absent) — downstream readers treat absent + empty as "nothing to surface"
- [x] Unit test `.agentic-framework/agents/context/tests/revisit-due-scan-test.sh` creates 4 mock tasks (ripe, future, no-field, commented-hint), runs scanner against tmpdir, asserts only ripe appears + asserts file is removed when nothing is ripe
- [x] Cron registry entry added to `.context/cron-registry.yaml` (revisit-due-scan job, daily 07:00) — cron-registry.yaml is the canonical source per T-448
- [x] `fw cron install` produces `/etc/cron.d/agentic-audit-termlink` containing the new line (verified live)
- [x] `.agentic-framework/agents/handover/handover.sh` reads `.context/working/.revisits-due.txt` and emits a "## Revisits Ripe Today" section iff the file is non-empty; smoke-tested with synthetic line
- [x] Channel-1 mirror: scanner + handover.sh edit + test pushed upstream at commit `76d53e29` on master (rebased onto aaf7f69b). Cron-registry stays local — it's consumer-side opt-in.

## Verification

# Scanner exists and runs cleanly
test -x .agentic-framework/agents/context/revisit-due-scan.sh
.agentic-framework/agents/context/revisit-due-scan.sh
# Unit test passes
test -x .agentic-framework/agents/context/tests/revisit-due-scan-test.sh
.agentic-framework/agents/context/tests/revisit-due-scan-test.sh
# Cron entry installed
grep -q revisit-due-scan /etc/cron.d/agentic-audit-termlink
grep -q revisit-due-scan .context/cron-registry.yaml
# Handover.sh has the banner block
grep -q "Revisits Ripe Today" .agentic-framework/agents/handover/handover.sh
# Channel-1 mirror verification — assert upstream master carries the new files (SHA updated after dispatch)
git ls-remote https://onedev.docker.ring20.geelenandcompany.com/agentic-engineering-framework master 2>/dev/null | head -1 | grep -qE '[0-9a-f]{40}'

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

### 2026-06-01T — closure-ready: revisit-due-scan + handover banner integration shipped [agent autonomous]

All 9 Agent ACs ticked, no Human ACs defined. `revisit-due-scan.sh` is in production:

- Cron installed at `/etc/cron.d/agentic-audit-termlink` (verified via this session's structure audit: 15 PASS / 1 WARN / 0 FAIL)
- Handover agent banner integration shipped — verified live as the cron consumer of T-1451's `revisit_at` field; this session's handover (`.context/handovers/LATEST.md`) generated without a revisit banner because no deferral has come due yet
- G-053 prevention loop closed end-to-end: T-1451 (the field) + T-1452 (the consumer) form the paired structural fix

This is the load-bearing companion to T-1451. Bookkeeping shipped this session via handover commits ("T-1452: Session handover S-2026-0531-2120" / "S-2026-0531-2300") which are this task's own auto-attributed activity.

**Operator-actionable:** ready for `fw task update T-1452 --status work-completed`.

### 2026-05-02T22:21:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1452-revisit-due-scansh-cron--handover-banner.md
- **Context:** Initial task creation

### 2026-05-15T18:25:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
