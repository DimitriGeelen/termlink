---
id: T-1295
name: "Widen volatile-runtime_dir doc — tmpfs OR systemd-tmpfiles D /tmp"
description: >
  T-1294 confirmed the volatile-runtime_dir scenario, BUT mechanism on .122 was systemd-tmpfiles 'D /tmp' rule with /tmp on regular disk — NOT tmpfs as CLAUDE.md currently describes. Same effect, different cause. CLAUDE.md 'Hub Auth Rotation Protocol' / 'Special case — volatile runtime_dir' section should widen the diagnostic so the next operator looks at both /usr/lib/tmpfiles.d/tmp.conf (D rule) AND mount table (tmpfs).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [docs, auth, G-011, T-1294-followup]
components: []
related_tasks: [T-1294, T-1290, T-1292]
created: 2026-04-26T14:27:04Z
last_update: 2026-04-26T14:29:42Z
date_finished: 2026-04-26T14:29:42Z
---

# T-1295: Widen volatile-runtime_dir doc — tmpfs OR systemd-tmpfiles D /tmp

## Context

T-1294 confirmed the volatile-runtime_dir scenario on ring20-management (.122), but the mechanism was NOT tmpfs as the existing CLAUDE.md doc describes — it was the `D /tmp 1777 root root -` rule in `/usr/lib/tmpfiles.d/tmp.conf`, which makes `systemd-tmpfiles --boot` wipe /tmp contents on every boot even when /tmp is on regular disk. Same effect, different cause. A future operator following the current diagnostic (`mount | grep termlink`) would falsely conclude "/tmp is fine, must be something else" and miss the smoking gun.

This task widens the doc so both mechanisms are checked.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md "Special case — volatile runtime_dir" section explicitly enumerates both mechanisms (tmpfs mount AND systemd-tmpfiles `D /tmp` rule) with detect commands for each.
- [x] Diagnostic block includes `cat /usr/lib/tmpfiles.d/tmp.conf /etc/tmpfiles.d/tmp.conf 2>/dev/null` in addition to the existing `mount` check.
- [x] Fix block distinguishes systemd-launched vs watchdog-launched hubs (T-1294 introduced the watchdog case).
- [x] Reference to T-1294 added alongside T-1290.

## Verification

grep -q 'systemd-tmpfiles' CLAUDE.md
grep -q 'D /tmp' CLAUDE.md
grep -q 'tmpfiles.d/tmp.conf' CLAUDE.md
grep -q 'watchdog-launched' CLAUDE.md
grep -q 'T-1294' CLAUDE.md

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

### 2026-04-26T14:27:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1295-widen-volatile-runtimedir-doc--tmpfs-or-.md
- **Context:** Initial task creation

### 2026-04-26T14:28:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-26T14:32:00Z — Doc widened
- **Action:** Replaced single-paragraph "Special case — volatile runtime_dir" block in CLAUDE.md (lines 56-69) with structured 2-mechanism diagnostic. New section enumerates tmpfs mount + systemd-tmpfiles `D /tmp` as parallel causes, gives detect commands for both, and splits the Fix block into systemd-launched vs watchdog-launched hub paths with reference to T-1294.
- **Verification:** All 5 grep checks PASS (`systemd-tmpfiles`, `D /tmp`, `tmpfiles.d/tmp.conf`, `watchdog-launched`, `T-1294` all present in CLAUDE.md).
- **All 4 Agent ACs satisfied.**

### 2026-04-26T14:29:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
