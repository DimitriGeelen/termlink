---
id: T-3323
name: "Upgrade .121 ring20-dashboard hub via runme now that it has a remote-exec foothold"
description: >
  Operator 2026-10-02: how do we get .121 upgraded? It serves 0.12.39 (fleet doctor), has no T-3310 retention/forever-owner fields. T-3290 left .121 out of runme action 6 for lack of a foothold; a root remote-exec session now exists (tl-cl4jd2gx, ring20-dashboard). Read-only probe: x86_64, /usr/local/bin/termlink 0.12.39, hub pid detached (ppid 1, no systemd unit, no watchdog found), TERMLINK_RUNTIME_DIR=/var/lib/termlink (persistent; hub.secret + May cert present). Operator authorised forced upgrades of .122 and .121 on 2026-09-30 (runme.sh action 6 header). Supersedes the T-2379 delegation (sent 2026-07-07, never answered).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [fleet, arc:arc-012]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-10-02T16:52:01Z
last_update: 2026-10-02T16:56:09Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-3323: Upgrade .121 ring20-dashboard hub via runme now that it has a remote-exec foothold

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/fleet-deploy-binary.sh --swap-restart` relaunches a detached hub with the RUNNING hub's TERMLINK_RUNTIME_DIR (read from /proc/<pid>/environ), falling back to the exec session's env only when unreadable — so a restart can never move the hub to a fresh runtime dir and rotate its secret/cert (PL-021)
- [x] runme.sh action 6 includes ring20-dashboard (.121) alongside ring20-management; comments and the action heading no longer say ".121 has no foothold"
- [x] runme fixtures still pass (action count derived, no real host touched)
- [x] After the operator's runme run: fleet doctor shows .121 serving the deployed version with status ok (secret still authenticates) and `tofu verify` passes (cert unchanged); its governor reports the T-3310 retention fields
- [x] `.context/cron/fleet-version-floors.conf` note for ring20-dashboard updated to the measured state (0.12.39 before, foothold exists)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
-->

## Verification

grep -q "RUN_RT=" scripts/fleet-deploy-binary.sh && grep -q 'RUNTIME=\${RUN_RT:-' scripts/fleet-deploy-binary.sh
grep -q 'FLEET_HUBS="${RUNME_FLEET_HUBS:-ring20-management ring20-dashboard}"' runme.sh
bash tests/runme-fixtures.sh > /tmp/.t3323-runme 2>&1 && grep -q "0 failed" /tmp/.t3323-runme
grep -q "^ring20-dashboard 0.12.103" .context/cron/fleet-version-floors.conf
bash scripts/check-fleet-binary-freshness.sh --no-heartbeat > /tmp/.t3323-fleet 2>&1 && grep -q "ring20-dashboard: served=0.12.103" /tmp/.t3323-fleet
termlink tofu verify 192.168.10.121:9100 > /tmp/.t3323-tofu 2>&1

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

## Recommendation

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
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

### 2026-10-02T17:10Z — deployed and verified live [claude]
- runme rc=0 (runme-20261002T165923Z-4075926.log): ring20-dashboard 0.12.39 -> 0.12.103 via --swap-restart; fleet doctor ok, tofu verify ok.
- Live probe after: hub relaunched as `/usr/local/bin/termlink hub start --tcp 0.0.0.0:9100` with TERMLINK_RUNTIME_DIR=/var/lib/termlink; hub.secret + hub.cert.pem still dated 2026-05-02 (no rotation); session tl-cl4jd2gx reconnected (same pid); governor reports T-3310 retention fields.
- Floor set: ring20-dashboard 0.12.103; fleet binary canary rc 0.
- Left open (their host, their call): .121's hub has no supervisor (no systemd unit, no watchdog) — it will not come back after a crash or reboot.

### 2026-10-02T16:52:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3323-upgrade-121-ring20-dashboard-hub-via-run.md
- **Context:** Initial task creation
