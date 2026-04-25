---
id: T-1255
name: "Implement T-1253 GO: handover.sh push only to origin + audit github-vs-origin drift"
description: >
  Implement T-1253 GO: handover.sh push only to origin + audit github-vs-origin drift

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T16:00:58Z
last_update: 2026-04-25T16:07:24Z
date_finished: 2026-04-25T16:07:24Z
---

# T-1255: Implement T-1253 GO: handover.sh push only to origin + audit github-vs-origin drift

## Context

T-1253 GO decision: `agents/handover/handover.sh:781-794` push loop iterates ALL git remotes individually
on every `handover --commit` invocation (auto-handover from PreCompact + budget-checkpoint hooks).
When OneDev briefly 502s, GitHub gets the push but origin doesn't, leaving GitHub AHEAD of OneDev.
The PushRepository job in `.onedev-buildspec.yml` uses `force: false`, so the divergence cannot
self-heal from OneDev's side. Documented mirror flow (CLAUDE.md "CI / Release Flow") says
**GitHub is read-only mirror** — only OneDev should be pushed directly.

PL-036 (from T-1140) already noted "fw handover --commit pushes directly to both origin and github,
bypassing the OneDev → GitHub mirror flow" but no fix was scoped at the time. T-1253 inception now
provides the bounded fix path.

**Step 1:** Modify push loop to push ONLY to canonical `origin` when other remotes
exist; preserve current behaviour for projects with `origin` as the only remote.
**Step 2:** Add `fw audit` divergence check (`git ls-remote github main` vs
`git ls-remote origin main`) so future drift is detected within one audit cycle (15min).

**Source-of-truth rule.** Edits must land in `/opt/999-Agentic-Engineering-Framework`
(upstream) and then mirror to consumer `/opt/termlink/.agentic-framework/`. Use
Channel 1 dispatch pattern (workflow_channel1_upstream_mirror memory).

## Acceptance Criteria

### Agent
- [x] Upstream `agents/handover/handover.sh:781-794` modified: push loop only pushes
      to `origin` when more than one remote is configured. Upstream commit 7f84a3ec
      adds `_remote_count=$(git remote | wc -l)` and skips non-origin remotes when
      count > 1. Single-remote behaviour preserved.
- [x] Skip emits "Skipping <remote> (mirrored from origin via PushRepository)" in
      cyan (matches existing handover output style). See upstream handover.sh
      around the new T-1255 anchor block.
- [x] Upstream `agents/audit/audit.sh` (in GIT TRACEABILITY section) compares
      `git ls-remote origin main` vs `git ls-remote github main` when both remotes
      exist, with `timeout 10` per probe. PASS msg shows matching short SHA;
      WARN msg names origin and github SHAs separately and points at T-1255.
- [x] Mirrored to consumer `.agentic-framework/agents/handover/handover.sh`,
      `.agentic-framework/agents/audit/audit.sh`, and the new
      `.agentic-framework/tests/handover-push-target.sh` via direct file copy
      from upstream. Hermetic test passes locally.
- [x] PL-036 in `.context/project/learnings.yaml` got a `status: closed` field
      and a `resolution:` line pointing at T-1255 + upstream commit 7f84a3ec.
      Body of the original `learning:` left untouched.
- [x] G-007 in `.context/project/concerns.yaml` flipped `status: watching` →
      `status: resolved`, added `resolved: 2026-04-25`, full resolution prose,
      and appended T-1253 + T-1255 to `related_tasks`.

### Human
- [ ] [REVIEW] Verify the next auto-handover (PreCompact or budget-checkpoint) emits
      "Skipping github" and only pushes to origin. Then `git ls-remote github main`
      should match `git ls-remote origin main` within ~5min of OneDev's PushRepository
      job firing.
      **Steps:**
      1. Wait for next auto-handover commit (or invoke handover via the framework CLI in /opt/termlink)
      2. Inspect handover output for "Pushing to remotes..." section
      3. Run: `git -C /opt/termlink ls-remote github main && git -C /opt/termlink ls-remote origin main`
      **Expected:** Output shows ONLY origin push (not github); ls-remote shows matching SHAs.
      **If not:** check handover.sh logic preserved correct behavior for single-remote case.

      **Agent evidence (2026-04-25T18:25Z, post-S-2026-0425-1958 handover):**
      - Auto-handover commit `e0c4b131` ran at 17:58Z. Captured output verbatim:
        `Pushing to remotes...` followed by `Skipping github (mirrored from origin via PushRepository)` and `Pushed to origin ✓`. No direct github push attempted.
      - Mirror sync verified at 18:25Z: `git ls-remote github main` and `git ls-remote origin main` both report `7ecce13c…` (HEAD) — zero drift.
      - Audit GIT TRACEABILITY check now reports `OneDev → GitHub mirror in sync (origin=github=…)` on every run.
      - **Pipeline behaving as designed.** Human can rubber-stamp.

## Verification

grep -q "Skipping" /opt/termlink/.agentic-framework/agents/handover/handover.sh
grep -q "OneDev.*GitHub mirror" /opt/termlink/.agentic-framework/agents/audit/audit.sh
grep -q "T-1255" /opt/termlink/.context/project/learnings.yaml
grep -q "T-1255" /opt/termlink/.context/project/concerns.yaml

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

### 2026-04-25T16:00:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1255-implement-t-1253-go-handoversh-push-only.md
- **Context:** Initial task creation

### 2026-04-25T16:07:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
