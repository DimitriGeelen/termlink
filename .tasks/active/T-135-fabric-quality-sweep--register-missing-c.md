---
id: T-135
name: "Fabric quality sweep — register missing cards, fix deps, sync subsystems"
description: >
  Fabric quality sweep — register missing cards, fix deps, sync subsystems

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-14T16:24:11Z
last_update: 2026-03-14T16:24:11Z
date_finished: null
---

# T-135: Fabric quality sweep — register missing cards, fix deps, sync subsystems

## Context

Fabric quality investigation found: 2 missing transport.rs cards, 4 lib cards with incomplete deps, subsystems.yaml out of sync with 5+ missing components.

## Acceptance Criteria

### Agent
- [x] transport.rs cards created for protocol and session crates
- [x] lib.yaml cards updated with complete depends_on (protocol: +events +transport, session: +auth +transport, hub: +pidfile +supervisor)
- [x] client.yaml updated with control.rs and transport.rs deps
- [x] subsystems.yaml synced with all registered components
- [x] All YAML files parse correctly

## Verification

python3 -c "import yaml; yaml.safe_load(open('.fabric/subsystems.yaml'))"
python3 -c "import yaml; yaml.safe_load(open('.fabric/components/crates-termlink-protocol-src-transport.yaml'))"
python3 -c "import yaml; yaml.safe_load(open('.fabric/components/crates-termlink-session-src-transport.yaml'))"
test -f .fabric/components/crates-termlink-protocol-src-transport.yaml
test -f .fabric/components/crates-termlink-session-src-transport.yaml

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

### 2026-03-14T16:24:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-135-fabric-quality-sweep--register-missing-c.md
- **Context:** Initial task creation
