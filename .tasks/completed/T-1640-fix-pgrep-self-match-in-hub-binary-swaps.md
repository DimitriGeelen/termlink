---
id: T-1640
name: "Fix pgrep self-match in hub-binary-swap.sh — bracket trick"
description: >
  scripts/hub-binary-swap.sh:111 uses 'pgrep -f "termlink hub start" | head -1' to find the running hub PID. The pattern matches the remote-exec shell's own argv (which contains the pgrep command itself, including the search string), so head -1 can return a transient PID instead of the long-running hub. 2026-05-15 T-1632/T-1633 deploy on .122 hit this: pgrep returned PID 2274098 (transient) instead of 3067203 (real hub), kill missed, script reported 'hub did not exit within 5s' and exited without rollback — leaving binary swapped on disk but old process still serving. Fix: bracket trick '[t]ermlink hub start' so pgrep's own argv (which contains 't' followed by ']' not 'e') no longer self-matches. Apply same to fleet-deploy-binary.sh:101+118+127+142 where 'pgrep -f' patterns also appear. Add a regression test: spawn a fake remote-exec wrapper that contains 'termlink hub start' in its argv, run the script's PID-resolution against a known-PID hub, assert the right one is picked. Pre-existing latent since T-1438 (2026-05-01). Related: T-1632, T-1633, T-1438.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [bug, T-1438, hub-binary-swap, deploy-tooling]
components: []
related_tasks: [T-1632, T-1633, T-1438]
created: 2026-05-15T20:14:13Z
last_update: 2026-05-15T20:19:24Z
date_finished: 2026-05-15T20:19:24Z
---

# T-1640: Fix pgrep self-match in hub-binary-swap.sh — bracket trick

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] **A1** All `pgrep -f` and `pkill -f` patterns in `scripts/hub-binary-swap.sh` use the bracket trick (e.g. `'[t]ermlink hub start'` instead of `'termlink hub start'`) so the search pattern in the running shell's argv does not self-match. — 7 callsites converted.
- [x] **A2** Same applied to `scripts/fleet-deploy-binary.sh` (5 callsites converted: lines 102, 184, 198, 201, 224).
- [x] **A3** Static test `tests/test_t1640_pgrep_self_match.sh` asserts every `pgrep -f` / `pkill -f` literal in both scripts uses a bracket-class prefix. — passes; fails if any callsite regresses.
- [x] **A4** Functional test: bug reproduced on decoy A (old pattern matches decoy PID), fix verified on decoy B (new pattern does NOT match the bracket-tricked decoy). — PASS in same test file.
- [x] **A5** `bash -n` clean on both scripts.
- [x] **A6** RCA section populated below (bug-class).

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

bash -n scripts/hub-binary-swap.sh
bash -n scripts/fleet-deploy-binary.sh
bash tests/test_t1640_pgrep_self_match.sh

## RCA

**Symptom:** 2026-05-15 deploy on .122. `scripts/hub-binary-swap.sh` reported `ERR: hub did not exit within 5s` and exited without rollback, even though the binary mv had already happened on disk. Post-incident inspection showed the kill landed on a transient PID (2274098), not the long-running hub (3067203, started May 12). The actual hub kept running with the old binary mapped in memory (`/proc/3067203/exe -> /usr/local/bin/termlink (deleted)`), still serving 0.9.2093 while disk had 0.9.2127. Manual SIGTERM + nohup relaunch was required to complete the deploy.

**Root cause:** `pgrep -f 'termlink hub start' | head -1` self-matches. When the script runs via `termlink remote exec`, the search pattern `termlink hub start` is part of the remote shell's own argv (the shell's command-line includes the pgrep invocation, which includes the literal pattern). pgrep finds the long-running hub AND the shell, then `head -1` picks whichever pgrep emits first — often the shorter-lived shell PID. The kill misses, no rollback fires, the script's "5s timeout" message masks a procedural mismatch.

**Why structurally allowed:**
1. **Pre-existing since T-1438 (~2 weeks).** The scripts have shipped this pattern across 12 callsites without anyone noticing — most prior deploys happened to pick the right PID (or used systemd-launched hubs where the kill target was unambiguous). The bug was latent until .122 (no systemd, orphan-under-init hub) made the multi-match real.
2. **No regression test for "script picks the correct PID under multi-match."** The bracket-trick (`'[t]ermlink hub start'`) is a well-known pgrep idiom but wasn't applied at write-time; no static lint or test enforced it.
3. **The error message ("hub did not exit within 5s") leads operators to suspect the hub, not the script.** Misleading framing slowed diagnosis today.

**Prevention:**
- **Static test (this task):** `tests/test_t1640_pgrep_self_match.sh` greps both scripts and fails if any `pgrep -f` / `pkill -f` literal doesn't start with `[X]`. Catches future regressions at test-time before they hit a hub.
- **Functional test (same file):** Spawns two decoy shells whose argvs contain the OLD and NEW pattern forms. Asserts the OLD pattern hits the decoy (bug reproduced) AND the NEW pattern does NOT (fix verified). Pinned behavior, not vibes.
- **The fix itself is mechanical (`'X'` → `'[X]'`).** No surface-area change for callers; ops procedure unchanged.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-15T20:14:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1640-fix-pgrep-self-match-in-hub-binary-swaps.md
- **Context:** Initial task creation

### 2026-05-15T20:15:01Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-15T20:19:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
