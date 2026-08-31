---
id: T-2869
name: "Static-check allowlists have no stale-entry detection — a fixed site stays acknowledged forever"
description: >
  Every source-level static check acknowledges confirmed-safe sites in a git-tracked allowlist, but no check verifies its own entries still correspond to a site it would flag. When a site is fixed or deleted, its entry silently keeps acknowledging nothing — and if the same signature is ever reintroduced it is pre-acknowledged, so the guard never fires. Found in T-2669 slice 2: migrating termlink_event_poll left its allowlist line acknowledging a site that no longer existed; it was removed by hand, and nothing would have caught it otherwise.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [guard-layer, static-check, allowlist, G-019]
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
created: 2026-08-31T15:06:46Z
last_update: 2026-08-31T15:06:46Z
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

# T-2869: Static-check allowlists have no stale-entry detection — a fixed site stays acknowledged forever

## Context

Every source-level static check in this repo (alloc-sink T-2527, drain-sink T-2531,
silent-exit T-2666, busy-spin T-2672, platform-lock T-2693, error-code-emission T-2699,
version-derivation T-2746, mcp-parity-census T-2747, unbounded-rpc-call T-2669, and the
task-corpus checks) acknowledges confirmed-safe sites in a git-tracked allowlist under
`.context/checks/`. The acknowledgement mechanism is load-bearing and deliberate: an entry
is counted and reported but does not fire, so the check ratchets — a NEW site is in neither
set and fires immediately.

**Nothing verifies that an entry still corresponds to a site the check would flag.** When a
site is fixed, migrated, or deleted, its allowlist line keeps sitting there acknowledging
nothing, silently and forever.

Found in T-2669 slice 2. `termlink_event_poll` was filed CLASS 1 (intentional long-poll)
on a reason that was false for it — `event.poll` never blocks — so its single call was
migrated to the bounded variant. Its allowlist line then acknowledged a site that no longer
existed. It was removed **by hand**, because I happened to be editing that file; had I not,
the check would have reported the same clean green with a dead entry in it and nothing would
ever have said so.

**The cost is not tidiness.** A stale entry is a live pre-acknowledgement: if the same
signature is reintroduced later — the same `<relpath>::<fn>::<kind>` — the check clears it
on sight and the guard never fires for a defect it was built to catch. That is the
`check-error-code-docs` / T-2683 shape one layer down: a guard whose green is a statement
about a set that has quietly drifted out from under it.

**Measured before filing, so the scope is honest.** A throwaway probe over the six fn-keyed
allowlists found **184 entries, 0 with a missing file, 0 with a missing fn**. The tree is
clean today. This task is **prevention, not remediation** — there is no backlog to work
down, and the check should be expected to ship green.

**Feasibility confirmed, not assumed.** The general algorithm is uniform across every check
carrying an `--allowlist` seam: run the check with an **empty** allowlist to obtain its full
candidate set, then any acknowledged signature absent from that set is stale. Verified
against `check-unbounded-rpc-call.sh` — empty allowlist yields exactly 155 firing sites
against 155 acknowledged entries, a perfect correspondence, so zero stale.

Sibling in kind to the same slice's other finding (per-function keying means one CLASS 1
line exempts every call in that function) — both are blind spots in the ledger mechanism
rather than in any one check. Neither blocks the T-2669 sweep.

Origin: T-2669 slice 2 (commit `281237933`), recorded in the header note of
`.context/checks/unbounded-rpc-call-allowlist`.

## Acceptance Criteria

### Agent
- [ ] A staleness detector exists that, for each participating check, derives the full
      candidate set by invoking the check with an empty allowlist and reports every
      acknowledged signature absent from that set.
- [ ] Membership is **declared, not guessed** — a check participates by carrying an explicit
      marker (mirroring `# guard-layer: source`), and a check with an allowlist but no marker
      is reported as unclassified rather than silently skipped (T-2684 precedent: a forgotten
      marker is itself the shipped-but-dark condition).
- [ ] Checks whose allowlist is not signature-keyed against a scannable candidate set (e.g.
      `mcp-parity-census`, whose 236 entries are a deliberate frozen ledger, and the
      task-corpus checks) are either handled correctly or explicitly declared out of scope
      with a cited reason — never silently mishandled.
- [ ] Output carries an explicit scope disclaimer (T-2680): it detects entries that no longer
      correspond to a flagged site; it does **not** audit whether a live entry's cited reason
      is still true. That second property is what T-2669 slice 2 found false for 7 of 16
      entries and is NOT in scope here.
- [ ] Exit codes follow the layer contract: 0 = no stale entries, 1 = stale entry found,
      2 = tooling. **Fail-closed** — an unreadable allowlist, an uninvokable check, or a
      candidate set that comes back empty exits 2, never a vacuous 0.
- [ ] Carries the `# guard-layer: source` marker and appears in `scripts/run-guard-layer.sh --list`.
- [ ] Fixture suite under `tests/`, weighted toward the FIRING cases — a detector of this shape
      is trivially green when everything is current, and a green check that cannot go red is not
      a check (T-2812 precedent).
- [ ] **Load-bearing proof:** injecting a known-dead signature into a real allowlist fires the
      detector on exactly that entry; removing it returns the tree to clean.
- [ ] The current tree is measured and the result stated in the commit message — expected clean,
      per the probe above.


## Verification

# Detector runs clean against the real tree (expected: no stale entries today).
bash scripts/check-allowlist-staleness.sh --json > /tmp/.t2869-clean.json 2>&1 && grep -q '"ok": *true' /tmp/.t2869-clean.json
# Fixtures pass, and actually exercise the firing path.
bash tests/allowlist-staleness-fixtures.sh > /tmp/.t2869-fix.out 2>&1 && grep -qE '[0-9]+ passed' /tmp/.t2869-fix.out
! grep -qiE '(^|[^a-z])fail' /tmp/.t2869-fix.out
# Declared as a guard-layer member and discoverable by the runner.
grep -q '# guard-layer: source' scripts/check-allowlist-staleness.sh
bash scripts/run-guard-layer.sh --list > /tmp/.t2869-list.out 2>&1 && grep -q 'check-allowlist-staleness' /tmp/.t2869-list.out
# Scope disclaimer is present on the CLEAN path too, not only when firing (T-2680).
bash scripts/check-allowlist-staleness.sh > /tmp/.t2869-scope.out 2>&1 && grep -qi 'does not' /tmp/.t2869-scope.out
# Fail-closed: a nonexistent allowlist directory must exit 2, never 0.
bash -c 'bash scripts/check-allowlist-staleness.sh --checks-dir /nonexistent-t2869 >/dev/null 2>&1; test $? -eq 2'

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

### 2026-08-31T15:06:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2869-static-check-allowlists-have-no-stale-en.md
- **Context:** Initial task creation
