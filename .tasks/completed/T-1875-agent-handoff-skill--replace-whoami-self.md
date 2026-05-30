---
id: T-1875
name: "/agent-handoff skill — replace whoami self-fp path with channel info read (PL-195 parallel fix)"
description: >
  PL-195 parallel: /agent-handoff Step 2 reads sender_id from whoami --json but on every host (shared or single) candidates[].sender_id is null. Skill currently logs 'unknown' on every handoff and Step 3.5/end-of-skill subscribe instructions inherit the same bad path. Apply the same channel info agent-presence fix that closed T-1874.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T08:19:40Z
last_update: 2026-05-30T08:19:55Z
date_finished: null
---

# T-1875: /agent-handoff skill — replace whoami self-fp path with channel info read (PL-195 parallel fix)

## Context

Same PL-195 root cause as T-1874 (which fixed `/check-arc`). Probed today on host .107: `whoami --json` returns 22 candidates and EVERY candidate has `sender_id: null`. So `/agent-handoff` Step 2's "If single candidate: capture `sender_id`" path can never succeed — the field is structurally null, not just on shared hosts.

Impact: less severe than `/check-arc` because the resolved sender_id is used only for log visibility (Task Updates section + the final 4-line summary), not for routing. So the skill still works end-to-end — but every handoff log says `Self: unknown`, defeating the audit trail.

Scope: edit `.claude/commands/agent-handoff.md` Step 2 to use the same `termlink channel info agent-presence --json | jq -r .senders[0].sender_id` path T-1874 established. Also update the closing-section advice ("derive it from `termlink whoami` (self fingerprint)") to point at the working path.

## Acceptance Criteria

### Agent
- [x] `.claude/commands/agent-handoff.md` Step 2 reads `sender_id` from `termlink channel info agent-presence --json | jq -r '.senders[0].sender_id'` (with fallback to `agent-chat-arc`), NOT from `whoami --json`
- [x] Step 2 preserves the "fall back to `unknown` if resolution fails" path so the skill still proceeds (handoff routing doesn't depend on self-fp, only logging does)
- [x] The trailing "derive it from `termlink whoami` (self fingerprint)" hint (line ~107) is updated to the working path
- [x] Step 2 documents shared-host semantics consistent with T-1874 (the resolved fp is the host signing key; T-1693 will give per-agent keys)
- [x] The skill's "Rules" or "Related" section references PL-195 and T-1874 (predecessor fix)
- [x] Smoke test: manually run the new Step 2 sequence and confirm it returns the same 16-hex fp as `/check-arc`'s Step 1 (consistency across SEND and RECEIVE sides)

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

grep -qE "channel info agent-presence" .claude/commands/agent-handoff.md
grep -qE "PL-195" .claude/commands/agent-handoff.md
grep -qE "T-1874" .claude/commands/agent-handoff.md
HF=$(timeout 8 termlink channel info agent-presence --json 2>/dev/null | jq -r '.senders[0].sender_id // empty'); test -n "$HF"

## RCA

**Symptom:** Every `/agent-handoff` invocation logs `Self: unknown` in the Task Updates entry and final summary. Audit trail loses the "who sent this handoff" attribution that the skill's own contract promised. Surfaced today (2026-05-30) while inspecting `/check-arc` (T-1874) for shared-host blindness — discovered the parallel skill had the same broken read path.

**Root cause:** Same as T-1874 — Step 2 reads `sender_id` from `whoami --json`'s `candidates[].sender_id`, but that field is structurally `null` (not just on shared hosts: probed and confirmed null across all 22 candidates on .107). `whoami` simply does not expose the wire envelope `sender_id`. The Step 2 contract ("If single candidate: capture `sender_id`") could never succeed even on a single-session host.

**Why structurally allowed:** The skill was authored when the right path (channel info `senders[]` array) didn't exist or wasn't known. The failure mode is silent ("log says unknown") so it never raised an exception that pushed anyone to look. No automated audit ever checked "does this skill's resolved identifier match what envelopes carry."

**Prevention:** (1) This fix replaces the broken path with the same one T-1874 established for `/check-arc` — cross-skill consistency now enforces itself: both SEND and RECEIVE sides read the same way from the same source. (2) PL-195 carries the failure class for future skill authors. (3) Verification commands in this task and T-1874 compare-string-test the working path on every completion, so a future regression to whoami would fail the gate. No new framework-level audit added — the cost of generalizing "skill identifier consistency" outweighs the benefit at N=2 skills.

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

### 2026-05-30T08:19:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1875-agent-handoff-skill--replace-whoami-self.md
- **Context:** Initial task creation

### 2026-05-30T08:19:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
