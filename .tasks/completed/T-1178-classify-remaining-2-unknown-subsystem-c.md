---
id: T-1178
name: "Classify remaining 2 unknown-subsystem cards (install.sh, check-mirror-freshness.sh)"
description: >
  Classify remaining 2 unknown-subsystem cards (install.sh, check-mirror-freshness.sh)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-21T10:38:42Z
last_update: 2026-04-21T10:39:49Z
date_finished: 2026-04-21T10:39:49Z
---

# T-1178: Classify remaining 2 unknown-subsystem cards (install.sh, check-mirror-freshness.sh)

## Context

After T-1177 landed subsystem rules + backfill, fabric has 104 cards with only 2 still carrying `subsystem: unknown`: `install.sh` (root-level bootstrap installer) and `scripts/check-mirror-freshness.sh` (OneDev→GitHub mirror drift detector, T-1140, G-007 mitigation). Both are standalone shell scripts. T-1177's rules file only covers `crates/termlink-*/**` Rust crates. Extending the rules to classify these two scripts closes the long tail: 104 cards, 0 unknown, trivially grep-able subsystems on every card.

## Acceptance Criteria

### Agent
- [x] `.fabric/subsystem-rules.yaml` extended with two rules: `install.sh` → `distribution`, `scripts/check-mirror-freshness.sh` → `operations` (to match `scripts/watchdog.sh` which is already `operations`)
- [x] Both card files updated from `subsystem: unknown` to the new value
- [x] `fw fabric overview` lists `distribution` as a subsystem
- [x] No card has `subsystem: unknown`

## Verification

test -f .fabric/subsystem-rules.yaml
grep -q "install.sh" .fabric/subsystem-rules.yaml
grep -q "check-mirror-freshness.sh" .fabric/subsystem-rules.yaml
python3 -c "import yaml; [exit(1) for c in __import__('os').listdir('.fabric/components') if c.endswith('.yaml') and yaml.safe_load(open(f'.fabric/components/{c}')).get('subsystem') == 'unknown']"

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

### 2026-04-21T10:38:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1178-classify-remaining-2-unknown-subsystem-c.md
- **Context:** Initial task creation

### 2026-04-21T10:39:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
