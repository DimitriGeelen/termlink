---
id: T-2327
name: "pushwaker trap reaps only direct children — subscribe grandchild can orphan"
description: >
  pushwaker trap reaps only direct children — subscribe grandchild can orphan on non-pgroup stop
status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-03T08:40:00Z
last_update: 2026-07-03T08:47:40Z
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
- [x] `_pw_reap_children` reaps the full subtree, not just direct children —
  e.g. recurse (`pgrep -P` per child, or kill the process group), so a killed
  rail subshell cannot orphan its `channel subscribe --push` grandchild on the
  INT/EXIT trap path.
- [x] `be-reachable.sh cmd_stop` (pgroup kill) still fully reaps (no regression);
  a standalone-`Ctrl-C` of the waker leaves NO lingering `channel subscribe
  --push` process.
- [x] `bash -n scripts/be-reachable-pushwaker.sh` clean; `bash
  scripts/test-pushwaker-filter.sh` still green.

## Verification

bash -n scripts/be-reachable-pushwaker.sh
bash scripts/test-pushwaker-filter.sh

## Updates

### 2026-07-03 — captured from code-review
- Filed from the arc-004 review agent's one MINOR finding. Budget gate blocked
  fixing it in-session; captured for a fresh session. One-line fix expected.

### 2026-07-03 — fixed (fresh session, post-compaction)
- `_pw_reap_children` (scripts/be-reachable-pushwaker.sh) rewritten from a flat
  `pgrep -P $$` to a breadth-first descendant walk: collect every process in the
  waker's subtree (waker → rail-loop subshell → `channel subscribe --push`), then
  `kill` them all. A process tree has no cycles so the frontier drains at the
  leaves. cmd_stop's pgroup-kill path is untouched (no regression) — the change
  only strengthens the defense-in-depth INT/EXIT trap path.
- **Live proof (AC2):** built a fake `termlink` whose `channel subscribe` execs a
  tagged long-lived grandchild, ran the real waker with BOTH rails, then sent
  `SIGINT to the waker pid only` (the standalone-Ctrl-C / non-pgroup case).
  Before: 2 subscribe grandchildren alive. After trap-path stop: **0 survivors**,
  waker exited clean. The old flat reaper would have orphaned both.
- **AC1/AC3:** `bash -n` clean; `test-pushwaker-filter.sh` 11/11 PASS.

### 2026-07-03T08:47:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
