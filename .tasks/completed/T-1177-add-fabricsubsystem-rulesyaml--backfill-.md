---
id: T-1177
name: "Add .fabric/subsystem-rules.yaml + backfill 14 unknown Rust cards"
description: >
  Add .fabric/subsystem-rules.yaml + backfill 14 unknown Rust cards

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-21T07:55:14Z
last_update: 2026-04-21T07:56:39Z
date_finished: 2026-04-21T07:56:39Z
---

# T-1177: Add .fabric/subsystem-rules.yaml + backfill 14 unknown Rust cards

## Context

Follow-up from T-1175 enricher fix: the post-enrich summary showed 25 edges still going to/from cards with `subsystem: unknown`. Root cause: `_do_register_file` in `register.sh` has fallback path-based subsystem inference (lines 221-236) but doesn't know about the `crates/termlink-*` Rust layout, so every Rust card gets `subsystem: unknown`. T-369 introduced a project-local hook (`.fabric/subsystem-rules.yaml`) precisely for this case — but termlink doesn't have one.

Scope: create the rules file mapping `crates/termlink-<X>/**` → subsystem `<X>`, plus backfill the 14 existing unknown-subsystem Rust cards. Framework code untouched (the hook is already wired at register.sh:185). Future cards created under any termlink-* crate will classify correctly automatically.

## Acceptance Criteria

### Agent
- [x] `.fabric/subsystem-rules.yaml` created with one rule per existing workspace crate (`termlink-bus`, `termlink-cli`, `termlink-hub`, `termlink-mcp`, `termlink-protocol`, `termlink-session`, `termlink-test-utils`)
- [x] Backfill script updates the 14 existing unknown-subsystem Rust card files in-place so their `subsystem:` field matches the crate name pattern
- [x] Unknown-subsystem count of Rust cards drops from 14 to 0
- [x] Total subsystem count for `bus` rises from 0 (didn't exist before) — new subsystem is populated with the 6 bus cards + derived entries; other existing subsystems (`cli`, `session`, `hub`) unchanged

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

test -f .fabric/subsystem-rules.yaml
python3 -c "import yaml; d = yaml.safe_load(open('.fabric/subsystem-rules.yaml')); assert d and d.get('rules'), 'empty or missing rules'"
python3 -c "import glob, yaml; bad=[p for p in glob.glob('.fabric/components/crates-termlink-*.yaml') if (yaml.safe_load(open(p)) or {}).get('subsystem')=='unknown']; exit(1 if bad else 0)"

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

### 2026-04-21T07:55:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1177-add-fabricsubsystem-rulesyaml--backfill-.md
- **Context:** Initial task creation

### 2026-04-21T07:56:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
