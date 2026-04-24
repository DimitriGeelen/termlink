---
id: T-1168
name: "T-1074 follow-up: channel:learnings publisher + subscriber on T-1155 bus"
description: >
  Cross-agent learning exchange on top of T-1155 channel bus. Publish learnings to channel:learnings on fw context add-learning; subscribe daemon writes to received-learnings.yaml; Watchtower fleet-insights panel. Depends on T-1158 (bus crate), T-1159 (ed25519 identity), T-1160 (channel API). Replaces the 15-min cron design from T-1074 inception — see docs/reports/T-1074-cross-agent-learning-exchange-inception.md for rationale.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [T-1074, T-1155, bus, learnings-exchange]
components: []
related_tasks: [T-1074, T-1155, T-1158, T-1159, T-1160, T-1161]
created: 2026-04-20T14:43:26Z
last_update: 2026-04-22T04:52:49Z
date_finished: null
---

# T-1168: T-1074 follow-up: channel:learnings publisher + subscriber on T-1155 bus

## Context

Cross-agent learning exchange on top of the T-1155 channel bus. Replaces the 15-min cron design from T-1074 inception — see `docs/reports/T-1074-cross-agent-learning-exchange-inception.md` for the full spike evidence and the bus-pivot rationale.

**Dependencies:** T-1158 (bus crate), T-1159 (ed25519 identity), T-1160 (channel API). This task cannot start until those three land.

**Scope:** one topic + one publisher hook + one subscriber daemon + one Watchtower panel. Designed to fit one session once dependencies are in place.

**Dependency + hook-point audit (2026-04-24):**

*All three deps landed:* `T-1158`, `T-1159`, `T-1160` are in `.tasks/completed/`. This task is unblocked.

*Same install gap as T-1165:* `termlink channel {post,subscribe,list,create}` verbs exist in termlink source (v0.9.385) but installed CLI is v0.9.206. Both the publisher hook AND the subscriber daemon need `termlink ≥ 0.9.380` deployed to every participating project.

*Publisher hook point (framework side):* `agents/context/lib/learning.sh::do_add_learning @97-103` — after the `mv "$temp_file" "$learnings_file"` that persists the entry, before `return 0`. One-liner addition: `publish_learning_to_bus "$id" "$learning" "$task" "$source" "$date" 2>/dev/null || true` (graceful degradation mirrors T-1165 pattern).

*Subscriber daemon (new component):* runs `termlink channel subscribe channel:learnings --follow` (assuming `--follow` exists; otherwise polled subscribe with cursor). Appends each envelope to `<project>/.context/project/received-learnings.yaml`. Dedup on `(origin_project, L-id)` per AC. Deployment options: (a) systemd service per project, (b) framework-scoped watchtower background thread, (c) cron polling. T-1074 inception rejected cron — pick (a) or (b).

*Watchtower panel (framework side):* new blueprint `web/blueprints/fleet_insights.py` + template `web/templates/fleet_insights.html`. Follow pattern of `fleet.py` (just mirrored upstream in T-1206). Register in `web/blueprints/__init__.py` alongside `fleet_bp`. Reads `.context/project/received-learnings.yaml`, groups by origin_project, renders table.

*Schema / envelope design needed:*
```yaml
# channel:learnings envelope payload
origin_project: termlink            # identity string (per PL-021)
origin_hub_fingerprint: sha256:... # T-1052 R1 — pre-rotation detection
learning_id: L-052                  # scoped to origin_project
learning: "text"
source: "P-001"
task: "T-1064"
date: "2026-04-24"
```

*Boundary:* framework-side files (learning.sh, new blueprint, new daemon script) live in `/opt/999-Agentic-Engineering-Framework/`. T-559 hook blocks direct Write — need `/tmp` staging + `cp` + cross-repo commit pattern (T-1206 established).

**Recommended build split:**
- B1: publisher hook (smallest; just a bash helper function call in learning.sh)
- B2: subscriber daemon + received-learnings.yaml schema
- B3: Watchtower fleet_insights panel
- B4: install-bump coordination (every participating project upgrades termlink)

B1 alone gives asymmetric observability: any framework with termlink ≥ 0.9.380 can see learnings from this project land on the bus. Subscribers follow later.

## Acceptance Criteria

### Agent
- [ ] `channel:learnings` topic defined in bus schema with `(origin_project, L-id)` dedup key
- [ ] `fw context add-learning` publishes the new entry to `channel:learnings` via `channel.post` (T-1160 API)
- [ ] Subscriber daemon writes incoming entries to `.context/project/received-learnings.yaml` (separate file to preserve origin authorship per T-1074 scope fence)
- [ ] Dedup on `(origin_project, id)` — re-receiving the same entry is idempotent
- [ ] Envelope includes `origin_project` + `origin_hub_fingerprint` (T-1052 R1 — lets receivers spot pre-rotation learnings)
- [ ] Watchtower "fleet insights" panel renders `received-learnings.yaml`
- [ ] Never auto-applies received learnings — humans decide promotion to local rules

### Human
- [x] [REVIEW] Verify the Watchtower "fleet insights" panel surfaces cross-agent learnings — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — channel:learnings publisher/subscriber design approved; Watchtower fleet insights panel deferred to follow-up.
  **Steps:**
  1. After deploy, add a test learning via `fw context add-learning "test from <project>"`
  2. Open the target project's Watchtower `/fleet-insights` page
  3. Confirm the learning appears within one subscribe-poll cycle
  4. Confirm it's stored in `.context/project/received-learnings.yaml` (not `learnings.yaml`)
  **Expected:** Visible + origin attribution preserved
  **If not:** Check subscriber daemon is running; check bus connectivity

## Verification

cargo build
cargo test -p termlink-watchtower learnings
bash -n agents/context/context.sh
grep -q "channel:learnings" agents/context/context.sh
grep -q "received-learnings" agents/context/context.sh
test -f .context/project/received-learnings.yaml || echo "will be created on first receive"

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

### 2026-04-20T14:43:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1168-t-1074-follow-up-channellearnings-publis.md
- **Context:** Initial task creation

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next
