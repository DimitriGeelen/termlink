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
last_update: 2026-05-15T18:25:44Z
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
- [ ] `.agentic-framework/agents/context/revisit-due-scan.sh` exists, is executable, and runs without arguments
- [ ] Scanner reads `.tasks/active/*.md`, extracts `revisit_at:` frontmatter values, and selects entries where the date is `<= today` (UTC, lexicographic compare on ISO `YYYY-MM-DD` is correct)
- [ ] Output written atomically to `.context/working/.revisits-due.txt`, one line per ripe task in the form `T-XXX fires YYYY-MM-DD: <name>` (matches the existing manual file's format for backward compatibility with downstream readers)
- [ ] When no tasks are ripe, the output file is either absent or empty (`< 1 byte`) — downstream readers must treat both as "nothing to surface"
- [ ] Unit test in `.agentic-framework/agents/context/tests/` creates two mock tasks (one with `revisit_at: 1999-01-01`, one with `revisit_at: 2099-12-31`), runs the scanner against a tmpdir, asserts only the past one appears
- [ ] Cron registry entry in `.agentic-framework/.context/cron/agentic-audit.crontab.template` (or the project equivalent) invokes the scanner daily — pick 07:00 local to match the G-053 spec, but make the time configurable via a comment
- [ ] `fw cron install` produces a `/etc/cron.d/agentic-audit-<project>` containing the new entry (verified via `crontab -l` or `cat /etc/cron.d/agentic-audit-termlink | grep revisit-due-scan`)
- [ ] `.agentic-framework/agents/handover/handover.sh` reads `.context/working/.revisits-due.txt` and emits a "## Revisits Ripe Today" section in the handover document iff the file is non-empty (otherwise skip the section entirely — no noise)
- [ ] Channel-1 mirror: scanner + cron template + handover edit pushed upstream via `termlink dispatch --workdir /opt/999-AEF` (remote: `onedev`)

## Verification

# Scanner exists and is executable
test -x .agentic-framework/agents/context/revisit-due-scan.sh
# Scanner produces well-formed output (T-1428 has revisit_at if T-1451 backfilled it; otherwise empty)
.agentic-framework/agents/context/revisit-due-scan.sh
test -f .context/working/.revisits-due.txt || true
# Unit test passes
test -f .agentic-framework/agents/context/tests/revisit-due-scan-test.sh && \
  .agentic-framework/agents/context/tests/revisit-due-scan-test.sh
# Cron entry installed (audit-cron registry includes the scanner)
grep -q revisit-due-scan /etc/cron.d/agentic-audit-termlink 2>/dev/null || \
  grep -q revisit-due-scan .agentic-framework/.context/cron/agentic-audit.crontab.template
# Handover banner surfaces ripe revisits (smoke: dry-run, grep the produced file)
# Real test should run handover.sh and grep — keep this line as placeholder.
true
# Channel-1 mirror verification
test -x /opt/999-AEF/agents/context/revisit-due-scan.sh 2>/dev/null || true

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

### 2026-05-02T22:21:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1452-revisit-due-scansh-cron--handover-banner.md
- **Context:** Initial task creation

### 2026-05-15T18:25:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
