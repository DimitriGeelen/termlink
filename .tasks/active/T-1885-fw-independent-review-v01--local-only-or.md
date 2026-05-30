---
id: T-1885
name: "fw independent-review v0.1 — local-only orchestrator (REVIEW-CLI + CLI-WATCH + RUBBER-STAMP-RELEASE validators + independent-reviewer rail)"
description: >
  fw independent-review v0.1 — local-only orchestrator (REVIEW-CLI + CLI-WATCH + RUBBER-STAMP-RELEASE validators + independent-reviewer rail)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T21:58:06Z
last_update: 2026-05-30T21:58:06Z
date_finished: null
---

# T-1885: fw independent-review v0.1 — local-only orchestrator (REVIEW-CLI + CLI-WATCH + RUBBER-STAMP-RELEASE validators + independent-reviewer rail)

## Context

T-1884 inception (GO 2026-05-30) decided a consumer-side review orchestrator
with the MVP scope narrowed to local-only validators. This is the v0.1 build.

**Design source:** `docs/reports/T-1884-review-queue-orchestrator-inception.md`
**Spike evidence:** `docs/reports/T-1884-S{1,2,3}-results.md`

**Scope (per inception):**
- REVIEW-CLI (32 ACs): capture cmd output + grep Expected keywords
- CLI-WATCH (8 ACs): frame-capture via `script -c` + ANSI 2J+H split + body diff (S3-proven)
- RUBBER-STAMP-RELEASE (1 AC): `gh release view`
- Total: **41 of 72 unchecked Human ACs auto-validatable, no remote-exec dependency**

**Non-scope (deferred to v0.2 T-1886):** RUBBER-STAMP-MECHANICAL + OBSERVE-INFRA (need remote-exec).

**Surface-only by design:** OPERATOR-ACTION + TIME-GATED + OTHER (15 ACs — no validator possible).

## Acceptance Criteria

### Agent
- [ ] `scripts/independent-review.py` (or equivalent verb in `.agentic-framework/agents/`) exists, executable, with `--help`
- [ ] [REVIEWER] Classifier from `scripts/T-1884-S1-classify.py` extracted into a reusable module (or inlined cleanly); same 87.5%+ confidence on the current 72-AC corpus
- [ ] [REVIEWER] CLI-WATCH validator from `scripts/T-1884-S3-cli-watch.py` integrated; produces PASS-ROBUST/PASS-LOOSE/FAIL/INCONCLUSIVE verdict per AC
- [ ] REVIEW-CLI validator: captures `script -c` output of the AC's Steps shell-commands, greps Expected keywords, emits per-AC verdict
- [ ] RUBBER-STAMP-RELEASE validator: uses `gh release view <tag>` to check binary attachments
- [ ] Per-AC evidence is appended to the source task's `## Updates` block (NOT to `### Human` AC checkboxes — constitutional rail per T-1950 D36/113/213)
- [ ] FAIL verdicts auto-file `T-XXXX investigate-T-<src>` follow-up tasks with G-019 RCA stub
- [ ] INCONCLUSIVE verdicts ALSO auto-file follow-up (operator decision D4 — anti-pile-up)
- [ ] Independent-reviewer rail: each AC is validated in a separate process/context (subprocess or sub-agent dispatch) — producer code does NOT classify its own work. Documented + enforced.
- [ ] `--tick-mechanical-pass` flag exists, default OFF. When ON, RUBBER-STAMP-* PASS-ROBUST also ticks the `### Human` checkbox (Tier-2 logged per session).
- [ ] Verb supports batch-by-default + filters: `fw independent-review` (all), `--task T-XXX`, `--since 7d`, `--class <class>`, `--resume`
- [ ] State journaled to `.context/working/.independent-review-state.json` for crash-safe resume
- [ ] Dry-run on the current 41-AC scope produces a summary table: how many PASS-ROBUST, PASS-LOOSE, FAIL, INCONCLUSIVE, OPERATOR-ONLY, OTHER

### Human
- [ ] [REVIEW] Run `fw independent-review --dry-run` and verify the per-task verdict lines read naturally — operator can scan for which tasks have evidence-of-PASS available
  **Steps:**
  1. `cd /opt/termlink && fw independent-review --dry-run | tee /tmp/v01-dryrun.log`
  2. Read the output — is each PASS line backed by evidence visible in the task's Updates block?
  3. Spot-check 3 PASS tasks via `fw task review T-XXX` to verify the evidence renders in Watchtower
  **Expected:** Verdict lines are clear, evidence is operator-actionable, no spammy noise.
  **If not:** Report which lines are unclear and which Updates entries are missing context.
- [ ] [RUBBER-STAMP] At least one FAIL produced an auto-followup task with a useful RCA stub
  **Steps:**
  1. `ls .tasks/active/ | grep investigate` (should show ≥1 new T-XXXX)
  2. Open the most recent one and verify it has G-019 RCA section + source-task link
  **Expected:** Auto-followup tasks are well-formed and operator-actionable.
  **If not:** Diagnose which source-task FAILed and why the stub failed to populate.

## Verification

# Smoke: verb runs, --help works, dry-run completes without error
test -x scripts/independent-review.py || test -x .agentic-framework/agents/independent-review/independent-review.sh
bash -c "(scripts/independent-review.py --help 2>&1 || .agentic-framework/agents/independent-review/independent-review.sh --help 2>&1) | head -5"
# Dry-run produces non-empty output
python3 scripts/independent-review.py --dry-run 2>&1 | grep -qE 'PASS|FAIL|INCONCLUSIVE'
# Classifier preserved (re-run S1, confidence ≥80%)
python3 scripts/T-1884-S1-classify.py 2>&1 | grep -qE 'PASS$|Overall confidence: (8[0-9]|9[0-9]|100)'

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

### 2026-05-30T21:58:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1885-fw-independent-review-v01--local-only-or.md
- **Context:** Initial task creation
