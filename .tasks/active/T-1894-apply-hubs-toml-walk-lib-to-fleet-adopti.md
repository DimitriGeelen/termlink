---
id: T-1894
name: "apply hubs-toml-walk lib to fleet-adoption-snapshot (T-1892 followup)"
description: >
  apply hubs-toml-walk lib to fleet-adoption-snapshot (T-1892 followup)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-31T09:20:54Z
last_update: 2026-05-31T09:20:54Z
date_finished: null
---

# T-1894: apply hubs-toml-walk lib to fleet-adoption-snapshot (T-1892 followup)

## Context

T-1892 lib applied to canary (T-1892) and listeners-fleet (T-1893). Same pattern remaining in `scripts/fleet-adoption-snapshot.sh` — feeds the daily adoption-metric snapshot cron, so duplicate-counted hubs inflate `total_hubs`, `fleet_reachable_hubs`, `fleet_listeners`, `fleet_chat_arc`, `fleet_dm_topics`. Apply the lib.

T-1848's UNION-via-`sort -u` on `fleet_speakers_tmp` already handles speaker dedup correctly across hubs; only the per-hub-counted metrics need the lib.

## Acceptance Criteria

### Agent
- [x] `scripts/fleet-adoption-snapshot.sh` sources `lib/hubs-toml-walk.sh` and applies `dedup_addrs_by_fp` to (profile_addrs, profile_names) TSV right after parse, before the per-hub probe loop.
- [x] `TIMEOUT_CMD="timeout 8"` set locally (PL-189) for the probe.
- [x] Synthetic two-profile smoke (`.107` + `127.0.0.1` → same hub d1bd50f5): `{hubs:1, reachable_hubs:1, live_listeners:1, chat_arc_posts:67, unique_speakers:4, dm_topics_active:127, adoption_state:HOT}`. Stderr: `fleet-adoption-snapshot: skipping duplicate 127.0.0.1:9100 (same hub as 192.168.10.107:9100, fingerprint=d1bd50f5)`.
- [x] Real-config regression smoke: `{hubs:4, reachable_hubs:4, live_listeners:1, chat_arc_posts:95, unique_speakers:5, dm_topics_active:131, adoption_state:HOT}`. `hubs` dropped from 5 raw to 4 deduped — the local hub no longer counts twice. unique_speakers UNION (T-1848) already de-duped correctly pre-fix.

### Human
- [ ] [RUBBER-STAMP] Confirm the cron snapshot reads correctly post-fix.
  **Steps:**
  1. `bash scripts/fleet-adoption-snapshot.sh --json 2>/dev/null | jq '{total_hubs, fleet_reachable_hubs, fleet_listeners, fleet_chat_arc, fleet_dm_topics}'`
  2. Compare against `.context/working/.fleet-adoption-snapshot.log` last entry; should be consistent (total_hubs may have dropped by the number of duplicate profiles in your hubs.toml).
  3. `grep -q 'lib/hubs-toml-walk.sh' scripts/fleet-adoption-snapshot.sh && echo "sourced" || echo "not sourced"`
  **Expected:** Step 1 succeeds; Step 3 prints "sourced".
  **If not:** Capture the snapshot output and file a regression task.

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

bash -n scripts/fleet-adoption-snapshot.sh
grep -q "lib/hubs-toml-walk.sh" scripts/fleet-adoption-snapshot.sh
grep -q "dedup_addrs_by_fp" scripts/fleet-adoption-snapshot.sh

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

### 2026-05-31T09:20:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1894-apply-hubs-toml-walk-lib-to-fleet-adopti.md
- **Context:** Initial task creation

### 2026-05-31T09:35Z — fix-shipped-smoke-confirmed [agent autonomous]
- **Change:** Sourced T-1892 lib; TSV-form dedup over (profile_addrs, profile_names) right after parse, before per-hub probe loop. `TIMEOUT_CMD="timeout 8"` (PL-189) for probe only.
- **Synthetic smoke:** `{hubs:1, reachable_hubs:1, live_listeners:1, chat_arc_posts:67, unique_speakers:4, dm_topics_active:127}` — was 2 hubs pre-fix.
- **Real-config smoke:** `{hubs:4, reachable_hubs:4, live_listeners:1, chat_arc_posts:95, unique_speakers:5, dm_topics_active:131, adoption_state:HOT}` — `hubs` dropped from 5 raw to 4 deduped.
- **User-facing impact:** Daily adoption-metric snapshot cron no longer reports inflated hub count when hubs.toml has overlapping profiles. Downstream G-060 alerting (which gates on `adoption_state`) sees clean data.
- **Recommendation:** GO — RUBBER-STAMP steps match captured smokes.
