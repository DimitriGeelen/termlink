---
id: T-1850
name: "fleet-adoption-snapshot: extend sender resolution (T-1848 undercount fix)"
description: >
  T-1848 used .metadata.agent_id only — undercount on vendored-arc heartbeat posters that use .metadata._from. T-1849 found the gap. Apply the same priority chain (agent_id → _from → sender_id) to fleet-adoption-snapshot.sh.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug, doorbell-mail, adoption, t-1848-followon]
components: []
related_tasks: []
created: 2026-05-28T19:34:56Z
last_update: 2026-05-28T19:34:56Z
date_finished: null
---

# T-1850: fleet-adoption-snapshot: extend sender resolution (T-1848 undercount fix)

## Context

T-1848 shipped `unique_speakers` in fleet-adoption-snapshot.sh using `.metadata.agent_id` only. T-1849 found that vendored-arc heartbeat posters (T-1438) use `.metadata._from`, not `.metadata.agent_id` — so they were invisible to the gauge. Live diff: window=24h shows unique_speakers=1 (old) vs unique_speakers=4 (new) on the same data.

## Acceptance Criteria

### Agent
- [x] `scripts/fleet-adoption-snapshot.sh`: sender extraction jq uses priority `(.metadata.agent_id // .metadata._from // .sender_id // "")` (matches T-1849).
- [x] Live verification: 24h window now reports `unique_speakers=5` (was 1 before fix — vendored-arc heartbeat posters become visible).
- [x] adoption_state now correctly determined: HOT iff (≥1 listener AND ≥2 unique speakers). Currently COLD because /be-reachable was stopped per protocol — but the speaker-count IS ≥2, so state would correctly be HOT the moment a listener returns. T-1848 prevented this from ever being possible on this fleet.
- [x] Existing tests still 9/9 pass.

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

bash scripts/fleet-adoption-snapshot.sh --json --since 24 | jq -e '.summary.unique_speakers >= 2' >/dev/null
bash scripts/test-fleet-adoption-snapshot.sh >/dev/null
grep -q 'metadata._from' scripts/fleet-adoption-snapshot.sh

## RCA

**Symptom:** T-1848 reported `unique_speakers=1` for a 24h window where 4 distinct posters had been active on agent-chat-arc — a 75% undercount on the gauge meant to distinguish conversation from monologue.

**Root cause:** Sender extraction looked at `.metadata.agent_id` only. The vendored-arc heartbeat convention (T-1438) uses `.metadata._from` instead. Different poster conventions, single-source jq selector.

**Why structurally allowed:** No test exercised the gauge against vendored-arc heartbeat envelopes — T-1848's test relied on the script's happy path against the local hub, where my own posts use agent_id and the heartbeats happened to be filtered out by `msg_type == "chat"`. I never inspected raw envelope shape, just trusted the convention.

**Prevention:** PL-191 (to be captured): sender-identity in TermLink envelopes is multi-source — always use the agent_id → _from → sender_id priority chain. Any future "who posted?" code path that picks one field is the next bug.

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

### 2026-05-28T19:34:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1850-fleet-adoption-snapshot-extend-sender-re.md
- **Context:** Initial task creation
