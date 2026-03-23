---
id: T-239
name: "Route cache — .cache/routes/ YAML with confidence, TTL, lazy invalidation"
description: >
  Per-agent route cache in .cache/routes/ keyed by capability slug. YAML entries with confidence scores, TTL, hit counts, schema validation. 3-way branch: hit+valid -> direct, partial match -> refinement query, miss -> orchestrator. See T-233 research: Q2b-routing-decision.md

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-233, orchestration, cache]
components: []
related_tasks: [T-233, T-237]
created: 2026-03-23T13:27:32Z
last_update: 2026-03-23T13:27:32Z
date_finished: null
---

# T-239: Route cache — .cache/routes/ YAML with confidence, TTL, lazy invalidation

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
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

## Updates

### 2026-03-23T13:27:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-239-route-cache--cacheroutes-yaml-with-confi.md
- **Context:** Initial task creation
