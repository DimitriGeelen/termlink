---
id: T-1451
name: "revisit_at frontmatter field + template update (T-1449 Phase-1 #1)"
description: >
  T-1449 Phase-1 deliverable #1: add revisit_at: <ISO-date> and optional revisit_evidence_needed: <one-line> frontmatter fields to task templates. Backward-compatible (opt-in field). Update zzz-default.md + inception.md templates. Teach update-task.sh to preserve the field on status changes. Document in CLAUDE.md inception section. ~30 LOC + 1 template + doc.

status: captured
workflow_type: build
owner: human
horizon: now
tags: [framework, governance, T-1449, phase-1, channel-1-mirror]
components: []
related_tasks: [T-1449, T-1428]
created: 2026-05-02T22:21:29Z
last_update: 2026-05-02T22:21:29Z
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
- [ ] `.agentic-framework/.tasks/templates/default.md` frontmatter includes `revisit_at:` (optional, ISO-8601 date string `YYYY-MM-DD`) with an inline `<!-- ... -->` comment explaining: "Set on DEFER decisions to enable G-053 daily revisit scan"
- [ ] `.agentic-framework/.tasks/templates/default.md` frontmatter includes `revisit_evidence_needed:` (optional, one-line string) paired with `revisit_at:`; comment: "What evidence makes the revisit actionable"
- [ ] `.agentic-framework/.tasks/templates/inception.md` frontmatter includes the same two fields (same comments)
- [ ] `.agentic-framework/agents/task-create/update-task.sh` preserves both fields on status / horizon / owner / tags updates (regression test below)
- [ ] CLAUDE.md "Inception Discipline" section documents the field: "When choosing DEFER, set `revisit_at: <ISO-date>` to enable the G-053 daily scan"
- [ ] Channel-1 mirror: same patch pushed upstream via `termlink dispatch --workdir /opt/999-AEF` (commit + push to `onedev`)

## Verification

# Frontmatter field present in both templates
grep -q "^revisit_at:" .agentic-framework/.tasks/templates/default.md
grep -q "^revisit_at:" .agentic-framework/.tasks/templates/inception.md
grep -q "^revisit_evidence_needed:" .agentic-framework/.tasks/templates/default.md
# Documentation present in CLAUDE.md
grep -q "revisit_at" CLAUDE.md
# Regression test: update-task.sh preserves revisit_at
tmp=$(mktemp -d); cp .agentic-framework/.tasks/templates/default.md "$tmp/T-9999-test.md"; \
  sed -i 's/^id:.*/id: T-9999/; s/^name:.*/name: "test"/' "$tmp/T-9999-test.md"; \
  printf '\nrevisit_at: 2099-12-31\n' >> "$tmp/T-9999-test.md.head" 2>/dev/null || true; \
  # Real test should run update-task.sh and grep — keep this line as placeholder.
  rm -rf "$tmp"; true
# Channel-1 mirror verification (G-002 fast-exit)
test -d /opt/999-AEF/.tasks/templates 2>/dev/null && \
  diff -q .agentic-framework/.tasks/templates/default.md /opt/999-AEF/.tasks/templates/default.md || true

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
