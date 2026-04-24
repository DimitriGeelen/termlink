---
id: T-1218
name: "Watchtower fleet-learnings panel (T-1168 B3)"
description: >
  Watchtower panel that reads the cross-project learnings mirror written by T-1217's subscriber. Displays origin_project, learning_id, learning text, task, source, date, received_at per entry. Read-only UI over .context/project/received-learnings.yaml. Split from T-1217 B3 per sizing rule 'one task = one deliverable'. Open question: should this be a new /fleet-learnings page, or integrated into the existing fleet overview? Warrants inception before build.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-24T13:09:59Z
last_update: 2026-04-24T13:17:42Z
date_finished: 2026-04-24T13:17:42Z
---

# T-1218: Watchtower fleet-learnings panel (T-1168 B3)

## Context

Companion to T-1217 (subscriber poller) — Watchtower needs to surface the
cross-project learnings accumulating in `.context/project/received-learnings.yaml`.
Watchtower already has `/learnings` (own learnings + patterns + practices) via
`web/blueprints/discovery.py`. Simplest integration: extend the existing
`/learnings` page with a "Received from peers" section at the bottom, rather
than adding a new route. New route is overkill for a read-only list.

## Acceptance Criteria

### Agent
- [x] New `load_received_learnings()` helper in `web/context_loader.py` —
      landed in 9bfdc5d5; reads `.context/project/received-learnings.yaml`,
      returns `received:` list (empty on missing/malformed per load_yaml).
- [x] `/learnings` route in `web/blueprints/discovery.py` passes
      `received_learnings=received` — landed in 9bfdc5d5.
- [x] `templates/learnings.html` renders "Received from peers" `<details>`
      section with origin, id, learning, task, source, date, received_at
      columns. Wrapped in `{% if received_learnings %}` so empty list hides
      the section entirely — landed in 9bfdc5d5.
- [x] Direct render verification (Jinja invoked with seeded 2-entry yaml):
      section present with both rows, both L-IDs, both origins. With empty
      list: section absent. (Running gunicorn on :3000 predates the commit
      and requires restart to pick up Python code — HTTP-layer smoke test
      deferred to the Human RUBBER-STAMP where the UI will be live.)
- [x] Upstream-mirrored to `/opt/999-Agentic-Engineering-Framework/web/*` —
      commit 9bfdc5d5 pushed to onedev/master via termlink dispatch (PL-053).

### Human
- [ ] [REVIEW] Render quality of the Received section.
      **Prerequisites (agent-verified 2026-04-24T16:55Z):**
      - T-1218 template/blueprint changes landed in /opt/999-AEF (commit 9bfdc5d5)
        but are NOT yet in termlink's vendored `.agentic-framework/web/`.
      - `grep -c "received_learnings" .agentic-framework/web/context_loader.py` returns 0.
      - Running Watchtower (pid 3689280, started 2026-04-23 20:53, port 3100) serves
        the stale vendored copy. `curl http://localhost:3100/learnings | grep -c "from peers"` returns 0.
      - `.context/project/received-learnings.yaml` has 1 seeded entry (L-0001 from peer-alpha)
        ready to render once the upgrade+restart happens.
      **Steps (updated for this project):**
      1. Sync vendored framework: `fw upgrade` (brings in T-1218's web/ changes)
      2. Restart Watchtower: kill pid 3689280 → `PROJECT_ROOT=/opt/termlink python3 -m web.app --port 3100 &`
      3. Open http://localhost:3100/learnings in browser (not :3000 — that's the AEF-framework Watchtower)
      4. Look at the "Received from peers" section (the seeded L-0001 should appear)
      **Expected:** Table renders cleanly, aligns with existing learnings
      table style, long learning text wraps sensibly
      **If not:** Screenshot + note what's misaligned

## Verification

python3 -c "import yaml; yaml.safe_load(open('/opt/999-Agentic-Engineering-Framework/web/context_loader.py'))" 2>/dev/null || true
grep -q "load_received_learnings" /opt/999-Agentic-Engineering-Framework/web/context_loader.py
grep -q "received_learnings" /opt/999-Agentic-Engineering-Framework/web/blueprints/discovery.py
grep -q "Received from peers" /opt/999-Agentic-Engineering-Framework/web/templates/learnings.html

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

### 2026-04-24T13:09:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1218-watchtower-fleet-learnings-panel-t-1168-.md
- **Context:** Initial task creation

### 2026-04-24T13:11:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-24T13:17:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
