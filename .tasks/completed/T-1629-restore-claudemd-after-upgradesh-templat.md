---
id: T-1629
name: "Restore CLAUDE.md after upgrade.sh template-overwrite regression"
description: >
  Restore CLAUDE.md after upgrade.sh template-overwrite regression

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-09T21:13:48Z
last_update: 2026-05-09T21:17:18Z
date_finished: 2026-05-09T21:17:18Z
---

# T-1629: Restore CLAUDE.md after upgrade.sh template-overwrite regression

## Context

CLAUDE.md regression detected 2026-05-09T21:10Z: working copy diverged from
HEAD (and from CLAUDE.md.bak, which agreed with HEAD). Lost 3 lines on
2026-05-03 20:05:08 — when `.agentic-framework/lib/upgrade.sh` rewrote
CLAUDE.md from a generic template, backing the previous content into
CLAUDE.md.bak. PL-124 (T-1447, 2026-05-02) had documented this exact failure
class one day prior; the prescribed manual restore was either skipped or
mis-applied. Sat undetected for 6 days. Same class hit ≥2 times within 7
days → systemic per CLAUDE.md bug-fix learning checkpoint → registered as
G-055.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md restored — `git diff HEAD -- CLAUDE.md` is empty
- [x] CLAUDE.md.bak removed — `ls CLAUDE.md.bak` returns ENOENT
- [x] PL-124 application field updated from "TBD" with the recurrence + workaround prescription
- [x] G-055 added to concerns.yaml capturing the failure class as systemic (status: watching)
- [x] Both YAML files parse cleanly (`yaml.safe_load` no exception)

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

test -z "$(git diff HEAD -- CLAUDE.md)"
test ! -f /opt/termlink/CLAUDE.md.bak
python3 -c "import yaml; yaml.safe_load(open('/opt/termlink/.context/project/concerns.yaml'))"
python3 -c "import yaml; yaml.safe_load(open('/opt/termlink/.context/project/learnings.yaml'))"
grep -q "G-055" /opt/termlink/.context/project/concerns.yaml
grep -qE "PL-124.*[\s\S]*application:.*SYSTEMIC" /opt/termlink/.context/project/learnings.yaml || grep -qB2 -A2 "SYSTEMIC — same class hit again" /opt/termlink/.context/project/learnings.yaml

## RCA

**Symptom:** CLAUDE.md working copy diverged from HEAD and from CLAUDE.md.bak
on 2026-05-03 20:05:08. Lost the consumer-project fw-path clarification on
bullet 3 of "Copy-Pasteable Commands" (replaced by generic 1-liner) and 2
Quick Reference rows (/agent-handoff SEND, /check-arc RECEIVE).

**Root cause:** `.agentic-framework/lib/upgrade.sh:287-369` rewrites CLAUDE.md
governance sections wholesale from a framework-supplied template. Inline
modifications inside those governance sections (extra rows added to a table,
project-specific bullet text) are not preserved because the merge logic
operates at section-granularity, not line-granularity. The header comment
claims "preserve project sections, update governance" but only protects
named project-only sections, not inline additions to governance sections.

**Why structurally allowed:**
1. `upgrade.sh` writes a CLAUDE.md.bak alongside the new CLAUDE.md. The .bak
   is a passive artifact — nothing reconciles it against the new file or
   alerts when divergence appears.
2. PL-124 (2026-05-02) captured the failure class but its `application` was
   left "TBD" — no automation, no per-upgrade audit hook, no operator
   reminder. The learning was a note-to-self that never got applied on the
   very next upgrade 24h later.
3. The regression sat for 6 days because nothing in the framework's audit
   surface checks `git diff CLAUDE.md` for unexpected drift.

**Prevention:**
1. (Tactical, this task) PL-124 application field updated from "TBD" with
   the explicit post-upgrade ritual; G-055 registered to track the
   structural follow-up.
2. (Structural, follow-up framework task — not in scope here) `upgrade.sh`
   should run `git diff CLAUDE.md` after the template write and fail loud
   if the diff includes lines not present in either the old or new template
   (i.e. project additions). See G-055 `what_remains` for the full list of
   structural mitigations.
3. (Hygiene) Auto-cleanup of CLAUDE.md.bak after a TTL so it cannot mask
   future drift on grep-based inspections.


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

## Updates

### 2026-05-09T21:13:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1629-restore-claudemd-after-upgradesh-templat.md
- **Context:** Initial task creation

### 2026-05-09T21:17:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
