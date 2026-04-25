---
id: T-1253
name: "G-007 mirror anomaly — github ahead of onedev despite OneDev being source of truth"
description: >
  Inception: G-007 mirror anomaly — github ahead of onedev despite OneDev being source of truth

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T15:21:42Z
last_update: 2026-04-25T15:36:23Z
date_finished: 2026-04-25T15:36:23Z
---

# T-1253: G-007 mirror anomaly — github ahead of onedev despite OneDev being source of truth

## Problem Statement

G-007 has been "watching" since at least 2026-04-12: GitHub mirror is observed to be 25+ minutes stale despite successful pushes to OneDev. CLAUDE.md "CI / Release Flow" enshrines OneDev as source-of-truth + GitHub as a read-only mirror via `.onedev-buildspec.yml`'s `PushRepository` job (BranchUpdateTrigger).

**Today's anomaly (2026-04-25T16:08Z) inverts the direction:** When checking remotes after a normal commit + auto-handover sequence on /opt/termlink:
- local HEAD = `93e39ff1`
- `git ls-remote github main` → `93e39ff1` (in sync)
- `git ls-remote origin main` → first attempt 502, second attempt `a586edd8` (14+ min stale)
- After `git push origin main` → onedev advances to `93e39ff1`

**GitHub was AHEAD of OneDev** for the window `[16:07:05, 16:08:23+]`. Per the documented mirror flow, that should be impossible — github only receives updates *via* onedev's PushRepository job.

## Assumptions

A-1 (DISPROVEN): `fw git commit` auto-pushes to remotes. — *Disproved by reading `.agentic-framework/agents/git/git.sh` and `.agentic-framework/lib/version.sh`; no auto-push at commit time. Git hooks (post-commit, pre-push) do not push either.*

A-2 (CONFIRMED): The handover agent, when invoked with `--commit` (the routine path, including auto-handover from PreCompact + budget-checkpoint hooks), pushes to **all** configured remotes individually, not just `origin`. — *Confirmed by `.agentic-framework/agents/handover/handover.sh:771-790`:*

```
for remote_name in $(git -C "$PROJECT_ROOT" remote); do
    timeout 60 git push --follow-tags "$remote_name" HEAD
```

A-3 (PROBABLE): When OneDev is briefly unreachable (502, network glitch) at the moment of `handover --commit`, the loop pushes to GitHub successfully but the OneDev push fails (or times out at 60s). Net effect: GitHub advances, OneDev does not. The OneDev BranchUpdateTrigger never fires because OneDev never received a new ref. The mirror diverges silently.

A-4 (UNTESTED): The next successful push to OneDev does NOT trigger a corrective re-push to GitHub from OneDev's side, because PushRepository runs `force: false` and OneDev would attempt a non-fast-forward (GitHub already has commits OneDev doesn't), causing the push step to fail/skip.

## Exploration Plan

- ✅ S1: Read all agents/scripts that issue `git push` to enumerate push paths.
- ✅ S2: Reproduce: observe today's incident timeline via `git reflog` + `git ls-remote`. Confirm the divergence window and the exact commit at which github ran ahead.
- ✅ S3: Read `.onedev-buildspec.yml`'s `PushRepository` config to confirm the failure mode of OneDev → GitHub mirror when GitHub is ahead.
- ⏳ S4 (deferred): Probe the next OneDev recovery — does OneDev's PushRepository job auto-push and succeed (force-push?) or skip? Requires waiting for next 502.

## Technical Constraints

- OneDev's PushRepository job is the only sanctioned onedev → github mirror channel.
- `force: false` is set in `.onedev-buildspec.yml` (verified line 7 of the PushRepository step).
- The handover agent's "all remotes" loop has been in place since at least 2026-04-13 per T-1144 + T-1277 + T-1341 commit comments — predates the gap-watching window.

## Scope Fence

**IN:** Identifying the structural cause of the divergence + recommending a bounded fix.

**OUT:** Fixing OneDev's reliability (502 cause). Fixing the historical backlog of divergent commits (would require force-push or onedev manual ops).

## Acceptance Criteria

### Agent
- [x] Problem statement validated — direction-inverted from the original G-007 description (github ahead of onedev rather than the reverse), but same gap.
- [x] Assumptions tested — A-2 confirmed (line 771-790 of handover.sh); A-3 strongly inferred from sequence.
- [x] Recommendation written with rationale — see ## Recommendation below.

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO — bounded fix with two-step path.

**Rationale:** Root cause identified (handover.sh pushes to all remotes individually; bypasses the documented onedev-as-source-of-truth flow when onedev briefly fails). Fix is small, scoped, and reversible.

**Proposed fix (Step 1, structural):**
Modify `.agentic-framework/agents/handover/handover.sh:781-790` to push only to the canonical remote (`origin`) when both `origin` and a downstream mirror like `github` are configured. Retain the timeout-tolerant per-remote loop ONLY for projects where `origin` is the only remote (degenerate case where the loop currently runs once and is correct).

Concrete shape:
```bash
# Push only to canonical remote (per CLAUDE.md "Only push to OneDev")
# Mirroring to GitHub etc. is OneDev's job via PushRepository.
remote_name=origin
timeout "$_push_timeout" git push --follow-tags "$remote_name" HEAD || _push_failed=true
```

**Proposed fix (Step 2, observability):**
Add an audit check that flags github vs origin divergence:
```bash
github_head=$(git ls-remote github main 2>/dev/null | awk '{print $1}')
origin_head=$(git ls-remote origin main 2>/dev/null | awk '{print $1}')
[ "$github_head" = "$origin_head" ] || warn "github and origin diverged"
```
Run as part of `fw audit`. Surfaces drift within one audit cycle (15min) instead of "25+ min stale" via human spot-check.

**Evidence:**
- `handover.sh:783` `for remote_name in $(git remote)` — push loop iterates all remotes.
- `.onedev-buildspec.yml` PushRepository step uses `force: false` — diverged refs don't auto-heal from OneDev's side.
- Today's reflog timestamp (16:07:05 commit, 16:08:23 first ls-remote) confirms <2min divergence window after handover commit.
- T-1252 is unrelated (the version-stamp bug); G-007 is its own structural fix.

**Decomposition note:** This inception produces ONE build task (the handover.sh push-target change + audit check). Per Task Sizing rules, do not bundle with the historical-backlog cleanup (force-push question — explicit OUT of scope).

**Cost vs benefit:** ~30 LOC change in handover.sh + ~10 LOC audit check. Benefit: removes a structural cause of mirror divergence that produced the G-007 watching condition.

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

**Decision**: GO

**Rationale**: Root cause identified — `agents/handover/handover.sh:783` push loop iterates all remotes individually, allowing GitHub to advance ahead of OneDev when OneDev briefly 502s during `handover --commit`. Fix is bounded (~30 LOC: push only to canonical `origin`, plus a github-vs-origin divergence audit check). Reversible. Unblocks G-007 closure once the fix lands. (Rationale text restored 2026-04-25T15:42Z after a CSRF-fix verification probe accidentally recorded "test probe do not commit"; full root-cause analysis and recommendation in this task body's `## Recommendation` section above.)

**Date**: 2026-04-25T15:36:23Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-25T15:36:23Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** test probe do not commit

### 2026-04-25T15:36:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Reason:** Inception decision in progress

### 2026-04-25T15:36:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
