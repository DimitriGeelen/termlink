---
id: T-1451
name: "revisit_at frontmatter field + template update (T-1449 Phase-1 #1)"
description: >
  T-1449 Phase-1 deliverable #1: add revisit_at: <ISO-date> and optional revisit_evidence_needed: <one-line> frontmatter fields to task templates. Backward-compatible (opt-in field). Update zzz-default.md + inception.md templates. Teach update-task.sh to preserve the field on status changes. Document in CLAUDE.md inception section. ~30 LOC + 1 template + doc.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [framework, governance, T-1449, phase-1, channel-1-mirror]
components: []
related_tasks: [T-1449, T-1428]
created: 2026-05-02T22:21:29Z
last_update: 2026-05-15T18:42:29Z
date_finished: null
---

# T-1451: revisit_at frontmatter field + template update (T-1449 Phase-1 #1)

## Context

Prerequisite slice for T-1452 (cron + handover banner). G-053 documents
that DEFER inceptions have no structural revisit mechanism — sentinel-task
prose is the only reminder. This task adds the data field; T-1452 adds the
scanner that consumes it.

Scope is small (~30 LOC + 2 template edits + doc paragraph). Backward
compatible: field is optional; existing tasks without it are unaffected.
Channel-1 mirror to upstream framework required.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/.tasks/templates/default.md` frontmatter includes a commented opt-in hint for `revisit_at: YYYY-MM-DD` with explanation "Set on DEFER decisions to enable G-053 daily revisit scan". Commented (not active) so new tasks don't carry empty placeholders that confuse YAML parsers.
- [x] `.agentic-framework/.tasks/templates/default.md` frontmatter includes a paired commented opt-in hint for `revisit_evidence_needed:` (one-line string); comment: "What evidence makes the revisit actionable"
- [x] `.agentic-framework/.tasks/templates/inception.md` frontmatter includes the same two commented hints
- [x] `.agentic-framework/agents/task-create/update-task.sh` preserves both fields on status / horizon / owner / tags updates (regression test in `.agentic-framework/agents/task-create/tests/revisit-at-preservation-test.sh`)
- [x] CLAUDE.md "Inception Discipline" section documents the field: "When choosing DEFER, set `revisit_at: <ISO-date>` to enable the G-053 daily scan"
- [x] Channel-1 mirror: same patch pushed upstream to `agentic-engineering-framework` master at commit `aaf7f69b` (rebased onto remote 2e5f3cfb / T-077 handover head); cloned to /tmp/aef-channel1, patched, tested, pushed. The `termlink dispatch --workdir` path no longer works because the project-boundary hook (T-559/T-1702) blocks `/opt/999-AEF` references that aren't on the read-side allowlist — used /tmp clone-and-push instead.

## Verification

# Templates carry the opt-in commented hint for revisit_at + revisit_evidence_needed.
# Commented form is intentional — new tasks should NOT have an empty revisit_at
# line that would confuse parsers; only DEFER outcomes uncomment + fill it.
grep -q "revisit_at: YYYY-MM-DD" .agentic-framework/.tasks/templates/default.md
grep -q "revisit_at: YYYY-MM-DD" .agentic-framework/.tasks/templates/inception.md
grep -q "revisit_evidence_needed:" .agentic-framework/.tasks/templates/default.md
grep -q "revisit_evidence_needed:" .agentic-framework/.tasks/templates/inception.md
# Documentation present in CLAUDE.md (Inception Discipline section #8)
grep -q "revisit_at" CLAUDE.md
# Preservation test asserts update-task.sh never touches the field
test -x .agentic-framework/agents/task-create/tests/revisit-at-preservation-test.sh
.agentic-framework/agents/task-create/tests/revisit-at-preservation-test.sh
# Channel-1 mirror verification (G-002 fast-exit) — assert commit landed on upstream master
# (clone at /tmp/aef-channel1 may not persist; ls-remote is the canonical check)
git ls-remote https://onedev.docker.ring20.geelenandcompany.com/agentic-engineering-framework master 2>/dev/null | grep -q aaf7f69b

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

### 2026-05-02T22:21:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1451-revisitat-frontmatter-field--template-up.md
- **Context:** Initial task creation

### 2026-05-15T18:39:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
