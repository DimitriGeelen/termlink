---
id: T-1696
name: "Release pipeline drift canary — alert when OneDev HEAD diverges from GitHub HEAD >24h (G-058 prevention)"
description: >
  Release pipeline drift canary — alert when OneDev HEAD diverges from GitHub HEAD >24h (G-058 prevention)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [release, observability, canary, G-058]
components: []
related_tasks: [T-1695, T-1691]
created: 2026-05-18T10:44:52Z
last_update: 2026-05-18T10:44:52Z
date_finished: null
---

# T-1696: Release pipeline drift canary (G-058 prevention)

## Context

G-058 documents that the OneDev → GitHub mirror was silently broken for 16 days.
Sibling T-1695 restores the mirror (operator action). This task closes the
**prevention** Level-D gap: G-058 ran undetected because while T-1140 built
`scripts/check-mirror-freshness.sh` (script exists today, correctly reports the
current 534-commit drift on smoke test), **the canary was never wired to a cron**.
Script-without-trigger is a classic Level-C-without-Level-D failure mode.

This task adds the missing wire: a crontab entry that fires the existing canary
daily, with output appended to `.context/working/.release-mirror-canary.log` so
the next drift produces a visible artifact within 24h instead of 16+ days.

Also extends the canary with tag-drift coverage (the most-recent local tag must
exist on github) — current script only checks HEAD, which would miss the
specific failure mode where main mirrors but tags don't.

## Acceptance Criteria

### Agent
- [x] `scripts/check-mirror-freshness.sh` extended to also check tag drift — the most-recent local tag (`git describe --tags --abbrev=0`) must exist on `github` remote; missing tag flips exit status to drift
- [x] Smoke test against current state confirms drift detection still works AND the new tag-drift path fires (v0.11.1 is on OneDev but not on GitHub) — verified: `534 commit(s) behind` + `Latest tag v0.11.1 is NOT on GitHub (tag mirror broken)`
- [x] `.context/cron/release-mirror-canary.crontab` written with a daily ~07:00Z entry running `scripts/check-mirror-freshness.sh --quiet`, appending to `.context/working/.release-mirror-canary.log`, with cwd set to `/opt/termlink`
- [x] CLAUDE.md "CI / Release Flow" section gets one new paragraph describing the canary, log location, and the action triggered on drift (open T-1695 sibling task / page operator)
- [x] Existing `cargo test`/clippy pipeline NOT touched — pure tooling addition, no Rust changes

### Human
- [ ] [RUBBER-STAMP] Cron entry merged into the active host crontab on .107
  **Steps:**
  1. Review `.context/cron/release-mirror-canary.crontab` for the schedule
  2. Append the single line to your active crontab: `crontab -l | { cat; cat .context/cron/release-mirror-canary.crontab; } | crontab -`
  3. Verify: `crontab -l | grep check-mirror-freshness`
  4. Wait 24h or run manually once to seed the log: `bash scripts/check-mirror-freshness.sh --quiet >> .context/working/.release-mirror-canary.log 2>&1`
  **Expected:** Canary entry present in crontab; log file accumulates entries on drift
  **If not:** Pick a different schedule slot or path that fits the host's crontab convention

## Verification

# Smoke-test against current state — drift IS present right now (G-058), so canary should fire
bash scripts/check-mirror-freshness.sh; test $? -eq 1
# File exists, executable
test -x scripts/check-mirror-freshness.sh
# Crontab fragment present and well-formed
test -f .context/cron/release-mirror-canary.crontab
# Crontab references the canary script
grep -q 'check-mirror-freshness.sh' .context/cron/release-mirror-canary.crontab

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### 2026-05-18T10:44:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1696-release-pipeline-drift-canary--alert-whe.md
- **Context:** Initial task creation
