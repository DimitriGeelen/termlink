---
id: T-1878
name: "recent-dm.sh self-filter default — switch from be-reachable agent_id to envelope sender_id (PL-195 propagation)"
description: >
  recent-dm.sh defaults --self to ~/.termlink/be-reachable.state agent_id (e.g. root-claude-dimitrimintdev) but DM topics are keyed by envelope sender_id (host signing fp, e.g. d1993c2c3ec44c94). Result: default-mode silently filters away every real DM. Apply the PL-195 canonical fix (channel info agent-presence) used by check-arc/agent-handoff/agent-send/agent-respond.

status: work-completed
workflow_type: build
owner: claude-code
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-30T14:22:21Z
last_update: 2026-05-30T14:24:54Z
date_finished: 2026-05-30T14:24:54Z
---

# T-1878: recent-dm.sh self-filter default — switch from be-reachable agent_id to envelope sender_id (PL-195 propagation)

## Context

PL-195 was closed across check-arc/agent-handoff/agent-send/agent-respond in T-1874..T-1877 (commits 1a081c9c..1481d29c, 2026-05-30). The same identifier-conflation propagated to `scripts/recent-dm.sh` via its self-resolution path: it reads `agent_id` from `~/.termlink/be-reachable.state` (the presence display name, e.g. "root-claude-dimitrimintdev") and substring-matches that against `dm:*` topic names — but DM topics are keyed by envelope sender_id (host signing fingerprint on shared hosts, e.g. "d1993c2c3ec44c94"). Result: default-mode `/recent-dm <peer>` silently filters away every real DM on shared-host topology.

Observed live 2026-05-30 on .107 (this host): `bash scripts/recent-dm.sh 9219671e --since 720` → "no dm:* topics found containing both '9219671e' and self='root-claude-dimitrimintdev'". Override `--self d1993c2c3ec44c94` → 2 topics matched, 9 unread DMs surface (T-1166 close-out conversation with ring20-management-agent, 2026-05-26..29).

## Acceptance Criteria

### Agent
- [x] `scripts/recent-dm.sh` self-resolution replaces the be-reachable.state-first lookup with the PL-195 canonical chain: (1) `channel info agent-presence`, (2) `channel info agent-chat-arc` fallback, (3) be-reachable.state as last-resort (kept for forward-compat with T-1693 per-agent keys)
- [x] Inline comment block explains WHY (T-1878/PL-195 propagation, parity with check-arc/agent-handoff/agent-send/agent-respond)
- [x] `--help` text reflects the new default (no longer claims be-reachable.state is primary)
- [x] Live verification: `bash scripts/recent-dm.sh 9219671e --since 720` (no `--self` override) now returns the 2 matched topics that previously required explicit override
- [x] No new external dependencies (jq already required; `channel info` already used elsewhere)

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

grep -q "channel info agent-presence" scripts/recent-dm.sh
grep -q "PL-195" scripts/recent-dm.sh
bash scripts/recent-dm.sh --help 2>&1 | grep -qv "default: agent_id from"

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

### 2026-05-30T14:22:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1878-recent-dmsh-self-filter-default--switch-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-ba98168b
- **Timestamp:** 2026-05-30T14:24:54Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-30T14:24:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
