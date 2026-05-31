---
id: T-1870
name: "agent-chat-arc-recent: surface failed hub names in JSON + human output (PL-189 visibility follow-on)"
description: >
  agent-chat-arc-recent: surface failed hub names in JSON + human output (PL-189 visibility follow-on)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [scripts/agent-chat-arc-recent.sh]
related_tasks: [T-1845, T-1861, T-1851, T-1860]
created: 2026-05-29T23:15:58Z
last_update: 2026-05-29T23:21:33Z
date_finished: 2026-05-29T23:21:33Z
---

# T-1870: agent-chat-arc-recent: surface failed hub names in JSON + human output (PL-189 visibility follow-on)

## Context

`scripts/agent-chat-arc-recent.sh` (T-1851 read-side) and its wrapper paths
(`/recent-chat`, `/pulse`) currently report `hubs_failed: N` as a count
only — the *names* of the failed hubs are discarded. Observed today on
.107: 5-hub fleet, 2 hubs (.121 ring20-dashboard, .141 laptop) timed
out at the PL-189 8s ceiling on `channel info agent-chat-arc`, but the
recent-chat output read `"failed: 2"` with no indication which two.
Operator can't tell whether it's recoverable (slow hub with big backlog)
or genuine (hub down) without running an out-of-band fleet probe.

Same opacity exists in `--json` envelope — `summary.hubs_failed: 2`
but no `failed_hubs: [...]` array next to it. Compare with
`scripts/agent-listeners-fleet.sh` which exposes `hubs_failed: [...]`
as an array of names — that's the right shape for actionability.

Bounded reliability fix: track failure as `{hub, reason}` pair, expose
in JSON envelope, render one extra line in human format. No behavior
change — only output enrichment. Related learning: PL-192 (discovery
RPCs silently swallow failures) — same class of opacity.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-chat-arc-recent.sh` tracks failures as `{name, reason}` pairs, not just a counter
- [x] `--json` envelope grows a `summary.failed_hubs: [{hub: "<name>", reason: "<short>"}]` array (parallel to existing `hubs_failed` count, NOT a rename — preserve backward-compat for `/pulse` JSON consumers)
- [x] Human format prints an extra line `  failed: <name> (<reason>), <name2> (<reason2>)` when failures occurred (omit line when zero failures, do not print "failed: " stub)
- [x] Reason captures the structural cause: `timeout` (rc=124 from `timeout` wrapper), `network` (non-124 non-zero rc), `unknown-topic` (current -32013/not-found path — still counts as `hubs_scanned`, not failed; this path is already correct)
- [x] `bash scripts/agent-chat-arc-recent.sh --json --limit 1 --since 24 | jq '.summary.failed_hubs'` returns a JSON array (possibly empty)
- [x] When all hubs succeed, `failed_hubs` is `[]` in JSON and no line is printed in human format
- [x] Existing `summary.hubs_failed` integer field preserved as length of `failed_hubs` array

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

bash scripts/agent-chat-arc-recent.sh --json --limit 1 --since 1 | jq -e '.summary.failed_hubs | type == "array"'
bash scripts/agent-chat-arc-recent.sh --json --limit 1 --since 1 | jq -e '.summary.hubs_failed == (.summary.failed_hubs | length)'
bash scripts/agent-chat-arc-recent.sh --help >/dev/null

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

### 2026-05-29T23:15:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1870-agent-chat-arc-recent-surface-failed-hub.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-02738e1d
- **Timestamp:** 2026-05-29T23:22:08Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 3
     - evidence: `bash scripts/agent-chat-arc-recent.sh --help >/dev/null`

### 2026-05-29T23:21:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
