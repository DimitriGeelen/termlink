---
id: T-3371
name: "Write down the objectives of termlink"
description: >
  Seeded once by fw upgrade (T-3636): this project has no .context/project/objectives.yaml.
  Author it once, from this project's own material: a headline, a few objectives each
  with a measure, and what is out of scope. Progress is derived from the work afterwards.
status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [objectives-authoring]
components: []
related_tasks: []
created: 2026-10-06T11:19:34Z
last_update: 2026-10-20T23:41:14Z
date_finished: 2026-10-20T21:41:31Z
bvp_scores_proposed:
  - ts: '2026-10-20T19:51:39Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3371: Write down the objectives of termlink

## Context

`fw upgrade` seeded this task because termlink has no
`.context/project/objectives.yaml`. Every governed project authors its objectives once,
through its own entry point; this is that moment for a project that was onboarded before
the step existed. It is seeded once: upgrade will not add it again, and never if the file
exists.

**Draft from this project's own material only:** its README and docs, the code and its
history, its tasks and arcs, and anything the operator has said. Never copy another
project's objectives, and never the framework's own.

After this, the objectives are not rewritten to stay current. Arcs and tasks point at
objective ids (`supports: [O-n]`), progress is read from that work, and the framework's
audit flags an objective nothing serves.

### The shape to write

```yaml
# Project objectives — termlink.
#
# Authored intent; progress is derived, never written here.
# Arcs reference these ids via `supports: [O-n, ...]`.
headline: >-
  One or two sentences: who this is for and what it does for them.
objectives:
  - id: O-1
    text: >-
      An outcome a user or operator would notice, not an activity.
    measure: >-
      How you would know it holds. Say plainly if it is not measured yet.
out_of_scope:
  - "Something a reasonable person might expect this project to do, and it will not."
```

Keep objectives few (three to six). Every objective needs a `measure`, even if the honest
answer is "not measured yet".

**The flow:** the agent drafts, an independent reviewer checks it where one is available,
and the operator sees it once and ratifies it or says what to change.

## Acceptance Criteria

### Agent
- [x] `.context/project/objectives.yaml` exists in the shape above: the header line
      saying authored intent, progress is derived; headline; objectives (each with id,
      text, measure); out_of_scope
- [x] Drafted from this project's own material — never copied from another project or
      from the framework
- [x] Independent review asked for where available (`fw reviewer T-3371`, or another
      model), and its corrections applied or answered

### Human
- [ ] [REVIEW] The objectives say what termlink is for, in words you would use
  **Steps:**
  1. Read `.context/project/objectives.yaml`
  2. Check: does the headline say what termlink does and for whom? Is each
     objective an outcome you care about, with a measure you would accept? Is anything
     missing, or anything listed out of scope that should not be?
  **Expected:** You would sign the file as written, or after the changes you named
  **If not:** Tell the agent what to change; it edits the file and hands it back

## Verification

# The objectives file exists and has the shape: headline, objectives each with id/text/measure, out_of_scope list
python3 -c "import yaml; d=yaml.safe_load(open('.context/project/objectives.yaml')); o=d['objectives']; assert str(d['headline']).strip() and o and all(x.get('id') and x.get('text') and x.get('measure') for x in o) and isinstance(d['out_of_scope'], list)"
# It carries the authored-intent header (progress is derived, never written into it)
grep -qi 'authored intent; progress is derived' .context/project/objectives.yaml

## Recommendation

<!-- Fill in once the draft has been reviewed, before handing over with
     `fw task review T-3371`.

     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** why
     **Evidence:**
     - Finding 1 -->

**Recommendation:** GO (accept the draft as written; edit any wording you would not use).

**Rationale:** the headline is the charter sentence restated around its four verbs. Each objective is one
verb or one cross-cutting promise (shipped = live, failures heard), and every measure is something this
host already produces daily (canary logs, provers), so progress can be derived without new tooling. The
two not-yet-measurable parts (O-2's stage chain, O-6's in-band loudness) say so plainly instead of
borrowing a green.

**Evidence:**
1. Out-of-scope list = the five charter non-goals, verbatim in meaning (docs/CHARTER.md).
2. Measures cite existing scripts and logs: comms-selftest, substrate-smoke, session-selftest,
   arc-claim-drift, the canary logs and the release-publication canary.

## Decisions

### 2026-10-21 — objectives accepted as written (operator ruling)
- **Chose:** A, accept `.context/project/objectives.yaml` as written. The operator first answered "b"
  (accept with wording changes), then, asked which changes, said "all good": no changes.
- **Why:** the headline and O-1..O-6 restate the charter's verbs and guarantees, each with a measure
  already collected, and the out-of-scope list is the charter's non-goals.
- **Rejected:** C (rewrite: each objective maps to a real promise); D (defer: arcs keep running without a
  stated purpose).
- **Left open:** O-2's per-stage message view and O-6's in-band loudness become measurable as arc-010 and
  arc-011 deliver.
- The `### Human` review box is the operator's to tick (via Watchtower); the agent does not tick it.

### 2026-10-21 — objectives tightened and widened (operator ruling, supersedes the entry above)
- **Context:** the operator reopened the decision: "the trust system is a long road and the objectives
  should show it", and asked for a reflection against the weeks' work, not a single tweak.
- **Reflection (what the draft missed):** trust (identity, signatures, inject, supervisor token, operator
  identity); dispatched workers and the shared contract (O-3 covered only claims); "accepted means
  stored" (T-3461 ruling) and hub responsiveness under load (T-3488); arc-011's hub-as-record (gap notice);
  the operator's role (decisions only, never a courier, one command); measures that leaned on "canary log
  empty" instead of outcome numbers.
- **Chose:** the revised file, operator "yes" (2026-10-21):
  - headline "trusted … and their operator … dispatch and hand back work";
  - O-2 tightened (stored before accepted, gap notice, responsive under load, slow-line measure);
  - O-3 widened to dispatched work with one shared authoritative record and cheap isolated workspaces;
  - O-5 adds every host's daily health;
  - O-6 widened to the operator's role;
  - new O-7, trust;
  - out-of-scope item 5 clarified ("within the fleet every action is attributable and authorised").
  O-1 and O-4 are unchanged.
- **Rejected:** the as-written draft (blind to trust and to the worker direction).

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-10-20T19:51:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-10-20T20:00:00Z — draft written [w-objectives]
- Wrote `.context/project/objectives.yaml`: headline + 6 objectives (O-1 discover live
  peers, O-2 no silent message loss, O-3 claim work once / recover on worker death,
  O-4 control terminal sessions without leaks, O-5 shipped == running on Linux+macOS,
  O-6 failures loud and in one place) + 5 out-of-scope items (the charter's five
  non-goals). Sources: docs/CHARTER.md, README.md (four verbs, Guarantees), the arcs'
  headline_mechanic fields, and the T-3344 step-1 rulings summary.
- Every measure names an existing check: all 11 canaries cited have a crontab under
  `.context/cron/`; comms-selftest / session-selftest / substrate-smoke / canary-status
  scripts exist; `release.yml` has `needs: [test, test-macos]` (line 136). O-2 and O-6
  say plainly which parts are not measured yet (arc-010 stage chain, arc-011 gap notice,
  arc-007 in-band loudness).
- Review: `fw reviewer T-3371 --no-write` → PASS, no findings (it scans the task file,
  not the objectives' content). A content review by a second model was attempted and
  refused by the dispatch gate (FW_DISPATCH_LIMIT=0, T-3910); not run via
  `fw termlink dispatch` because this worker was told not to spawn anything. The
  operator's Human [REVIEW] AC is therefore the first content review.
- Verification commands (below) pass: shape check prints `6 5`, header grep OK.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-02f20589
- **Timestamp:** 2026-10-20T21:41:34Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-10-20T21:41:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
