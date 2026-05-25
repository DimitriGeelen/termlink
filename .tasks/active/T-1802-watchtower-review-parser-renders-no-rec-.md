---
id: T-1802
name: "Watchtower review parser renders NO-REC for tasks that have a Recommendation block (T-1612)"
description: >
  Watchtower review parser renders NO-REC for tasks that have a Recommendation block (T-1612)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-25T15:35:39Z
last_update: 2026-05-25T15:35:39Z
date_finished: null
---

# T-1802: Watchtower review parser renders NO-REC for tasks that have a Recommendation block (T-1612)

## Context

**Filed title is the initial (wrong) hypothesis.** Operator reported that the
Watchtower review page for **T-1612** showed NO-REC ("agent has not yet written
a `## Recommendation` block") despite termlink's completed
`T-1612-g-054-...md` clearly having a GO Recommendation block. Initial guess
was a parser bug in `web.shared.extract_recommendation`.

**Actual root cause: cross-project Watchtower port drift on shared host .107.**
- The operator opened termlink's advertised `watchtower.url` = `http://192.168.10.107:3001`.
- But **:3001 is NOT termlink's Watchtower** — it is pid 1245149 running from
  `/opt/050-email-archive/.agentic-framework` (`PROJECT_ROOT=/opt/050-email-archive`).
- 050-email-archive has its **own** `T-1612` (`inbox keyboard navigation`, an
  active task with no Recommendation block → correctly NO-REC).
- termlink's real Watchtower (pid 1711352, `PROJECT_ROOT=/opt/termlink`) runs on
  **:3003** and renders termlink's T-1612 (the G-054 task) correctly as **GO**.
- termlink's `watchtower.pid` correctly pointed at the :3003 process, but
  `watchtower.url`/`watchtower.port` had drifted to **:3001** — so the advertised
  URL led to a neighbor project's dashboard. The recommendation parser was never
  involved.

**Why it drifted:** termlink had **no configured `PORT`** (defaulted to 3000),
so its Watchtower port was non-deterministic. The shared launcher
(`bin/watchtower.sh`) writes the triple with whatever port it ended up binding;
on a host where multiple AEF projects share the port space, termlink's triple
ended up advertising a port a different project now owns.

## Acceptance Criteria

### Agent
- [x] Proved the parser is NOT the bug: termlink's `extract_recommendation` returns `verdict=="GO"` + non-empty raw for the exact T-1612 body, via the server's own import path (`PROJECT_ROOT=/opt/termlink`, `.agentic-framework/web/shared.py`, `_find_task_file`)
- [x] Identified true cause: termlink `watchtower.url`/`watchtower.port` advertised `:3001`, which is the 050-email-archive Watchtower (`PROJECT_ROOT=/opt/050-email-archive`, pid 1245149); termlink's real instance is `:3003` (pid 1711352)
- [x] Corrected termlink's `.context/working/watchtower.url` and `watchtower.port` to `:3003` (matching the actually-bound port; `pid` was already correct)
- [x] Verified the corrected URL renders termlink's T-1612 (G-054 task) as `Recommendation — GO`, not 050's inbox-nav NO-REC
- [x] Pinned termlink `PORT=3003` (`fw config set PORT 3003`) so future `watchtower restart` is deterministic and the triple cannot drift onto a neighbor's port

### Human
<!-- No human ACs — the fix is mechanically verified above (URL now resolves to
     termlink's own dashboard rendering T-1612 GO). -->

## Verification
test "$(cat .context/working/watchtower.port)" = "3003"
grep -q ':3003' .context/working/watchtower.url
test "$(.agentic-framework/bin/fw config get PORT)" = "3003"
curl -sf "$(cat .context/working/watchtower.url)/review/T-1612" | grep -q 'Recommendation — GO'

## RCA

**Symptom:** Watchtower review page for "T-1612" showed NO-REC ("agent has not
written a Recommendation block") even though termlink's completed T-1612 has a
GO Recommendation block. Operator reached the page via termlink's advertised
`watchtower.url`.

**Root cause:** termlink's `watchtower.url`/`watchtower.port` pointer files had
drifted to `:3001`, which on shared host .107 is a **different project's**
Watchtower (050-email-archive, `PROJECT_ROOT=/opt/050-email-archive`). That
instance served *its own* T-1612 ("inbox keyboard navigation", an active task
with no Recommendation block → legitimately NO-REC). termlink's actual
Watchtower runs on `:3003` and renders termlink's T-1612 as GO. The
recommendation parser (`web.shared.extract_recommendation`) was correct
throughout — proven by reproducing the server's exact import/find/parse path,
which returns verdict GO + non-empty raw.

**Why structurally allowed:** (1) termlink had no configured `PORT`, so its
Watchtower port was non-deterministic and could land on / drift to a port a
co-hosted project owns. (2) The triple `watchtower.{pid,port,url}` can become
internally inconsistent — `pid` pointed at the real :3003 process while
`url`/`port` advertised :3001 — and nothing validates that the advertised port
actually belongs to *this* project's running Watchtower. (3) Same task IDs
(T-1612) exist across projects, so a wrong-dashboard hit renders a plausible
page instead of a 404, masking the misdirection.

**Prevention:** (a) Pinned termlink `PORT=3003` so the port is deterministic
across restarts (this task). (b) Corrected the drifted triple. (c) Filed
follow-up T-1803 for the shared launcher hazard: on a multi-project host
`bin/watchtower.sh` should not advertise/kill a foreign port holder, and should
validate that `watchtower.url`/`port` point at this project's own running
instance (e.g. cross-check the served `PROJECT_ROOT`). Until that lands, the
deterministic per-project `PORT` is the operative guard.

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

### 2026-05-25T15:35:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1802-watchtower-review-parser-renders-no-rec-.md
- **Context:** Initial task creation
