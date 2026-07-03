---
id: T-2327
name: "pushwaker trap reaps only direct children — subscribe grandchild can orphan"
description: >
  pushwaker trap reaps only direct children — subscribe grandchild can orphan on non-pgroup stop
status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-07-03T08:40:00Z
last_update: 2026-07-03T08:40:00Z
date_finished: null
---

# T-2327: pushwaker trap reaps only direct children — subscribe grandchild can orphan

## Context

Found by the arc-004 code-review agent (2026-07-03, SHIP-WITH-NITS verdict) on the
T-2324 S2 refactor of `scripts/be-reachable-pushwaker.sh`.

Before S2, `run_waker` launched the `channel subscribe … --push` process as a
DIRECT child of the waker. After S2, the waker launches per-rail
`pushwaker_rail_loop` subshells in the background, and each subshell launches its
own `channel subscribe … --push` — so the subscribe process is now a
**grandchild** of the waker.

The `_pw_reap_children` trap (TERM/INT/EXIT) reaps only DIRECT children:
`kids="$(pgrep -P $$)"; kill $kids`. It therefore kills the rail subshells but
NOT their subscribe grandchildren. On the defense-in-depth trap path — a
standalone `Ctrl-C` or an EXIT that is NOT a process-group kill — a killed rail
subshell can orphan its `channel subscribe --push` grandchild (it reparents to
init and keeps holding the WS).

**Impact: LOW / MINOR.** The two paths that dominate real usage already handle
grandchildren: `be-reachable.sh cmd_stop` kills the whole setsid process group
(hits grandchildren directly — the PRIMARY reaper per T-2319), and a terminal
`Ctrl-C` delivers SIGINT to the entire foreground process group. The gap is only
the rare "trap fires but not via pgroup kill" case. Not a happy-path or
normal-stop bug — the review verdict was SHIP-WITH-NITS.

## Acceptance Criteria

### Agent
- [ ] `_pw_reap_children` reaps the full subtree, not just direct children —
  e.g. recurse (`pgrep -P` per child, or kill the process group), so a killed
  rail subshell cannot orphan its `channel subscribe --push` grandchild on the
  INT/EXIT trap path.
- [ ] `be-reachable.sh cmd_stop` (pgroup kill) still fully reaps (no regression);
  a standalone-`Ctrl-C` of the waker leaves NO lingering `channel subscribe
  --push` process.
- [ ] `bash -n scripts/be-reachable-pushwaker.sh` clean; `bash
  scripts/test-pushwaker-filter.sh` still green.

## Verification

bash -n scripts/be-reachable-pushwaker.sh
bash scripts/test-pushwaker-filter.sh

## Updates

### 2026-07-03 — captured from code-review
- Filed from the arc-004 review agent's one MINOR finding. Budget gate blocked
  fixing it in-session; captured for a fresh session. One-line fix expected.
