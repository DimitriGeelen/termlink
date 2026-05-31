---
id: T-1876
name: "agent-send.sh + agent-respond.sh — replace whoami self_fp routing with channel info read (PL-195 hot-path fix)"
description: >
  Both agent-send.sh and agent-respond.sh resolve self_fp via 'whoami | jq .session.identity_fingerprint' on the --peer-fp routing path (lines 146 + 67). The field is structurally null; the die call aborts. Operators using --peer-fp instead of --topic cannot send/respond on doorbell+mail. Apply the same channel info agent-presence path that closed T-1874 + T-1875.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [scripts/agent-respond.sh]
related_tasks: []
created: 2026-05-30T08:24:15Z
last_update: 2026-05-30T08:26:43Z
date_finished: 2026-05-30T08:26:43Z
---

# T-1876: agent-send.sh + agent-respond.sh — replace whoami self_fp routing with channel info read (PL-195 hot-path fix)

## Context

Hot-path follow-on to T-1874 (/check-arc) and T-1875 (/agent-handoff). While those two fixed skill-level identifier resolution, the underlying workhorse scripts that the skills delegate to — `scripts/agent-send.sh` (T-1804) and `scripts/agent-respond.sh` (T-1805) — have the SAME broken self_fp resolution at the routing layer, used when callers pass `--peer-fp` instead of `--topic`:

- `agent-send.sh:146`: `self_fp="$($TERMLINK whoami --json 2>/dev/null | jq -r '.session.identity_fingerprint // empty')"` then `[ -n "$self_fp" ] || die "could not resolve own identity_fingerprint"`
- `agent-respond.sh:67`: same shape, same `die` on failure

Because `.session.identity_fingerprint` is structurally null in whoami's output (probed 22/22 candidates on .107), the die fires unconditionally and the script aborts. Result: any operator passing `--peer-fp <fp>` cannot send a doorbell ring nor post a receipt. Routing-layer failure, not just logging.

Fix: replace the whoami-based resolution with the same `termlink channel info agent-presence --json | jq -r .senders[0].sender_id` path established in T-1874/T-1875. Inline-replace in both scripts (no shared helper — 3 lines × 2 sites; helper sourcing would add complexity for no win).

## Acceptance Criteria

### Agent
- [x] `scripts/agent-send.sh` line ~146 reads `self_fp` from `termlink channel info agent-presence --json | jq -r '.senders[0].sender_id // empty'` (with `agent-chat-arc` fallback)
- [x] `scripts/agent-respond.sh` line ~67 reads `self_fp` from the same path
- [x] Both scripts' die message points at `/be-reachable` (or "post once to agent-chat-arc") as the remediation
- [x] Both scripts pass `bash -n` (syntax check)
- [x] Smoke test: manually exec the resolution snippet in isolation, returns 16-hex fp matching what T-1874/T-1875 resolve to

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

bash -n scripts/agent-send.sh
bash -n scripts/agent-respond.sh
grep -q "channel info agent-presence" scripts/agent-send.sh
grep -q "channel info agent-presence" scripts/agent-respond.sh
! grep -qE 'self_fp="\$\(.+whoami' scripts/agent-send.sh
! grep -qE 'self_fp="\$\(.+whoami' scripts/agent-respond.sh
HF=$(timeout 8 termlink channel info agent-presence --json 2>/dev/null | jq -r '.senders[0].sender_id // empty'); test -n "$HF"

## RCA

**Symptom:** `agent-send.sh --to-session X --peer-fp Y "msg"` and `agent-respond.sh --peer-fp Y --reply "..."` abort with `could not resolve own identity_fingerprint (run inside a termlink session, or pass --topic)`. Operators using these scripts directly (not via the `--topic` path) cannot ring a doorbell nor post a receipt. Routing fails outright, not just logging.

**Root cause:** Same PL-195 identifier conflation as T-1874 and T-1875. Both scripts read `self_fp` from `whoami --json | jq .session.identity_fingerprint`, but that field is `null` for every candidate on every host probed. The die-on-failure is correct given the contract — but the contract reads from a field that never holds the right value, so it dies always.

**Why structurally allowed:** Three layers. (1) Same as T-1874/T-1875 — the right path (channel info `senders[]`) didn't exist or wasn't known at authoring time. (2) The scripts have a hot escape hatch (`--topic <topic>`) that bypasses the broken path entirely, so the wider doorbell+mail loop appeared to work via skill-level callers that always knew the topic. (3) No integration test exercises the `--peer-fp` mode — it's documented but unused in any harness, so the breakage went latent.

**Prevention:** (1) This fix replaces the broken path with the same one T-1874/T-1875 established. Identical at all three sites now (skill #1, skill #2, both scripts) — cross-arc consistency. (2) PL-195 covers the failure class. (3) The verification commands in this task assert by string-search that `self_fp="$(...whoami` never reappears — so a future regression to whoami would block --status work-completed. No new test harness for `--peer-fp` mode added — out of scope for this fix, but worth a future T-XXXX if `--peer-fp` becomes a documented advanced path.

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

### 2026-05-30T08:24:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1876-agent-sendsh--agent-respondsh--replace-w.md
- **Context:** Initial task creation

### 2026-05-30T08:24:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-b8edb01e
- **Timestamp:** 2026-05-30T08:26:43Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-30T08:26:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
