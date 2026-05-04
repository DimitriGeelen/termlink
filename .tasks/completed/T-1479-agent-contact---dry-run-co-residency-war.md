---
id: T-1479
name: "agent contact --dry-run: co-residency warning for peer_fp"
description: >
  agent contact --dry-run: co-residency warning for peer_fp

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/agent.rs]
related_tasks: []
created: 2026-05-04T11:19:23Z
last_update: 2026-05-04T11:21:56Z
date_finished: 2026-05-04T11:21:56Z
---

# T-1479: agent contact --dry-run: co-residency warning for peer_fp

## Context

T-1478 added `agent contact --dry-run` which previews the resolved dm
topic + metadata. T-1448's central insight: when multiple co-resident
sessions on the same host share a single identity_fingerprint, posting
to `dm:<a>:<peer_fp>` reaches ALL of them. The dry-run preview can
proactively detect this and surface a warning — directly serving the
disambiguation campaign's purpose.

Detection is cheap: enumerate local sessions, count those whose
metadata.identity_fingerprint matches the peer FP. If > 1 (excluding
the caller's own session), flag co-residency.

When co-residency is detected AND no `to_project` is in the metadata
block, escalate to a sharper warning: "co-resident peers detected and
no to_project qualifier — message will reach all N sessions". This
directly nudges the operator toward the `<name>:<project>` syntax.

## Acceptance Criteria

### Agent
- [x] `co_residency` block emitted when N > 1, absent otherwise (verified)
- [x] Co-residency count computed via `manager::list_sessions` filtered by `identity_fingerprint == peer_fp`
- [x] No-to_project warning text matches AC verbatim including "pass <name>:<project>" guidance (verified live: "co-resident peers detected (50 sessions share this FP locally) and no to_project qualifier — post will reach all of them; pass <name>:<project> to target one")
- [x] With-to_project softer warning text uses "self-filter" verbiage and echoes the to_project value (verified: "to_project=test-T-1479 will let receivers self-filter")
- [x] N <= 1 silent — verified by unit tests `no_co_residency_block_when_count_one` and `no_co_residency_block_when_count_zero`
- [x] 4 new unit tests in contact_tests cover: count_zero/count_one (silent), warns_no_to_project, softer_warning_with_to_project — all pass
- [x] `cargo build -p termlink` succeeds (7.37s); `cargo test ... contact_tests` 15 passed; 0 failed
- [x] Smoke (no to_project): live preview shows local_session_count=50 + first warning variant ✓
- [x] Smoke (with to_project): live preview shows local_session_count=50 + second warning variant ✓

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

cargo build -p termlink
cargo test -p termlink --bin termlink contact_tests > /tmp/t1479-test.txt 2>&1; grep -q "0 failed" /tmp/t1479-test.txt
target/debug/termlink agent contact termlink-agent --message x --dry-run > /tmp/t1479-dr.json 2>&1; python3 -c "import json; d=json.load(open('/tmp/t1479-dr.json')); cr=d.get('co_residency'); assert cr and cr.get('local_session_count',0) > 1 and 'no to_project' in cr.get('warning',''), d"
target/debug/termlink agent contact termlink-agent:test --message x --dry-run > /tmp/t1479-dr2.json 2>&1; python3 -c "import json; d=json.load(open('/tmp/t1479-dr2.json')); cr=d.get('co_residency'); assert cr and 'self-filter' in cr.get('warning',''), d"

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

### 2026-05-04T11:19:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1479-agent-contact---dry-run-co-residency-war.md
- **Context:** Initial task creation

### 2026-05-04T11:21:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
