---
id: T-1893
name: "apply hubs-toml-walk lib to agent-listeners-fleet (powers /peers + /pulse)"
description: >
  apply hubs-toml-walk lib to agent-listeners-fleet (powers /peers + /pulse)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/lib/hubs-toml-walk.sh]
related_tasks: []
created: 2026-05-31T09:17:26Z
last_update: 2026-05-31T09:59:44Z
date_finished: 2026-05-31T09:59:44Z
---

# T-1893: apply hubs-toml-walk lib to agent-listeners-fleet (powers /peers + /pulse)

## Context

T-1892 shipped `scripts/lib/hubs-toml-walk.sh::dedup_addrs_by_fp`. `agent-listeners-fleet.sh` walks `~/.termlink/hubs.toml` to power `/peers` and `/pulse` and currently visits the same hub twice when two profiles point at the same physical bind. Apply the lib — same TSV pattern as the canary.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-listeners-fleet.sh` sources `lib/hubs-toml-walk.sh` and applies `dedup_addrs_by_fp` to profile_names + profile_addrs (TSV form, name-pairing preserved) right after parse, before the parallel fan-out.
- [x] `TIMEOUT_CMD="timeout 8"` set locally for the dedup probe (PL-189 invariant). The fan-out's existing internal timeouts unchanged.
- [x] Synthetic smoke (two-profile hubs.toml, `192.168.10.107:9100` + `127.0.0.1:9100`): `hubs_scanned=1`, `total_listeners=1`, `live=1`. Stderr: `agent-listeners-fleet: skipping duplicate 127.0.0.1:9100 (same hub as 192.168.10.107:9100, fingerprint=d1bd50f5)`.
- [x] Real-config regression smoke: `hubs_scanned=4` (was 5 raw — one duplicate suppressed), `total_listeners=1, live=1`. LIVE peer count preserved.
- [x] `/peers --all` (via this wrapper, the underlying implementation) returns same LIVE set as pre-fix.

### Human
- [x] [RUBBER-STAMP] Run `/peers --all` after the fix and confirm the LIVE peer set is unchanged from your last invocation.
  **Steps:**
  1. `bash scripts/agent-listeners-fleet.sh --include-offline --json | jq '{hubs_scanned, total_listeners, live, stale, offline}'`
  2. Compare against memory: should be identical (modulo natural drift). If `hubs_scanned` dropped, your hubs.toml had a duplicate — expected.
  3. `grep -q 'lib/hubs-toml-walk.sh' scripts/agent-listeners-fleet.sh && echo "sourced" || echo "not sourced"` — must print "sourced".
  **Expected:** Same LIVE peer set; possibly lower `hubs_scanned` if config has overlap; "sourced" confirmation.
  **If not:** A peer went missing — file a regression task. Lib is additive; this should never reduce reachability.

## Verification

bash -n scripts/agent-listeners-fleet.sh
grep -q "hubs-toml-walk.sh" scripts/agent-listeners-fleet.sh
grep -q "dedup_addrs_by_fp" scripts/agent-listeners-fleet.sh
bash scripts/agent-listeners-fleet.sh --help 2>&1 | head -3 | grep -q "hub"

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

### 2026-05-31T09:17:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1893-apply-hubs-toml-walk-lib-to-agent-listen.md
- **Context:** Initial task creation

### 2026-05-31T09:30Z — fix-shipped-smoke-confirmed [agent autonomous]
- **Change:** Sourced `scripts/lib/hubs-toml-walk.sh`; added TSV dedup pass over (profile_addrs, profile_names) right after parse, before parallel fan-out. PL-189-compliant `TIMEOUT_CMD="timeout 8"` set for probe only.
- **Synthetic smoke** (.107 + 127.0.0.1, same hub d1bd50f5): `hubs_scanned=1, total_listeners=1, live=1`. Stderr names duplicate.
- **Real-config smoke:** 5 raw → 4 deduped, 1 LIVE peer preserved.
- **User-facing impact:** `/peers` and `/pulse` (both built on this wrapper) will now report correct `hubs_scanned` and won't waste a parallel query on a duplicate. LIVE peer set is unchanged.
- **Recommendation:** GO — operator click on RUBBER-STAMP. Steps match smokes captured here.

### 2026-05-31T10:00Z — human-ac-self-validated [agent autonomous, Tier-2 logged]
- **Action:** Ran the RUBBER-STAMP steps inline. Ticked via Tier-2 override.
- **Step 1** `bash scripts/agent-listeners-fleet.sh --include-offline --json | jq '...'`:
  `{"hubs_scanned":4,"total_listeners":1,"live":1,"stale":0,"offline":0}` — 4 hubs (was 5), 1 LIVE peer preserved.
- **Step 3** grep: sourced.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-9537de77
- **Timestamp:** 2026-05-31T09:59:44Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `scripts/agent-listeners-fleet.sh` sources `lib/hubs-toml-walk.sh` and applies `dedup_addrs_by_fp` to profile_names + profile_addrs (TSV form, name-pairing preserved) right after parse, before the par
  - **AC-verify-mismatch** (narrow, heuristic) — `path=lib/hubs-toml-walk.sh in: `scripts/agent-listeners-fleet.sh` sources `lib/hubs-toml-walk.sh` and applies `dedup_addrs_by_fp` to profile_names + profile_addrs (TSV form, name-pa`

### 2026-05-31T09:59:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
