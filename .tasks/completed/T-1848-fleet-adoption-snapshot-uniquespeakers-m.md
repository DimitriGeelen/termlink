---
id: T-1848
name: "fleet-adoption-snapshot: unique_speakers metric — distinguish conversation from monologue"
description: >
  Add unique_speakers (per-hub + fleet) to fleet-adoption-snapshot.sh. Refine HOT semantics: HOT requires >=2 unique speakers. Closes the gap where 178 chat_arc_posts from a single agent reports HOT but is actually a solo-monologue with no active conversation — the exact symptom the user's directive calls out.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [doorbell-mail, adoption, t-1843-followon]
components: [scripts/fleet-adoption-snapshot.sh]
related_tasks: []
created: 2026-05-28T18:58:30Z
last_update: 2026-05-28T19:02:16Z
date_finished: 2026-05-28T19:02:16Z
---

# T-1848: fleet-adoption-snapshot: unique_speakers metric — distinguish conversation from monologue

## Context

Current adoption_state semantics: HOT = ≥1 listener AND ≥1 chat_arc post. Today the fleet reports HOT with 178 posts/24h — but those posts are predominantly from a single agent (me). That's not a "conversation"; it's a monologue. The user's directive explicitly calls this out: "focus on **no active doorbell+mail conversations arc**". A real conversation needs ≥2 distinct speakers. This task adds a `unique_speakers` derived metric per-hub and refines HOT to require ≥2 speakers, so the gauge tells operators the real story.

## Acceptance Criteria

### Agent
- [x] Per-hub envelope grows a `unique_speakers` field: integer count of distinct `.metadata.agent_id` values in the windowed `agent-chat-arc` scan.
- [x] Summary envelope grows a `unique_speakers` field: count of distinct agent_ids across the union of all hubs' chat_arc envelopes in the window (NOT a sum — a single agent posting on 3 hubs counts as 1).
- [x] `adoption_state` reclassified: HOT requires `unique_speakers ≥ 2`; WARM covers (≥1 listener AND (0 posts OR 1 unique speaker)); COLD unchanged (0 listeners).
- [x] Human output adds a `unique_speakers:` summary row and a `SPEAKERS` column in the per-hub table.
- [x] Existing test `scripts/test-fleet-adoption-snapshot.sh` still 9/9 pass; T5 + T6 still validate JSON shape + state-membership.
- [x] Live verification shows current fleet now correctly reports WARM (single-speaker monologue) instead of misleadingly HOT, until a real second speaker posts.

      Live (2026-05-28): `state=WARM, live_listeners=2, chat_arc_posts=180, unique_speakers=1, dm_topics_active=258`. The directive's "no active doorbell+mail conversations arc" assertion is now MEASURABLE, not just observed.

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

bash scripts/fleet-adoption-snapshot.sh --json | jq -e '.summary.unique_speakers != null'
bash scripts/fleet-adoption-snapshot.sh --json | jq -e '.profiles | all(.unique_speakers != null)'
bash scripts/test-fleet-adoption-snapshot.sh >/dev/null

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

### 2026-05-28T18:58:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1848-fleet-adoption-snapshot-uniquespeakers-m.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-970e4b4b
- **Timestamp:** 2026-05-28T19:02:47Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 3
     - evidence: `bash scripts/test-fleet-adoption-snapshot.sh >/dev/null`

### 2026-05-28T19:02:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
