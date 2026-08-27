---
id: T-2685
name: "Canary crontabs merge tooling errors into findings logs"
description: >
  All 19 canary cron job lines use '>> findings.log 2>&1', merging exit-2 tooling
  errors into the exit-1 findings log and destroying the empty-log-means-healthy contract
  (T-2683 F2/G2+G3). Live false positive on release-mirror. Split stderr to a sibling
  log and guard the idiom.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/canary-status.sh, scripts/check-canary-log-hygiene.sh, scripts/check-cron-install-drift.sh, tests/canary-log-hygiene-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T05:56:25Z
last_update: 2026-08-27T09:44:18Z
date_finished: 2026-08-27T09:44:18Z
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
bvp_scores_proposed:
  - ts: '2026-08-23T19:13:28Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-23T19:13:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2685: Canary crontabs merge tooling errors into findings logs

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Every canary cron job line routes stderr to a sibling `<log>.stderr` instead of merging it into the findings log with `2>&1` — the tooling stream is preserved, never discarded, and never mistaken for a finding
- [x] All 24 git-tracked crontabs are migrated; zero remaining `>> <findings>.log 2>&1` job lines
- [x] The sibling `.stderr` path does not match `/canaries`' `.*-canary.log` discovery glob, so it cannot be read as a findings log
- [x] The polluted `.release-mirror-canary.log` is truncated, but only after independently confirming the mirror is actually synced (never blind-truncate a findings log)
- [x] `scripts/check-canary-log-hygiene.sh` fires on any crontab job line that merges stderr into a findings log — G-019 half: the next canary written with the old idiom is caught, not silently reintroduced
- [x] The hygiene check is a member of the T-2684 guard layer (`# guard-layer: source` marker) so it runs with everything else
- [x] Hygiene check exit codes: 0 = clean, 1 = a merging job line, 2 = tooling; `--json` for scripting; test seam `CANARY_HYGIENE_SRC_DIR` for hermetic fixtures
- [x] The check only flags *findings* logs — an operator appending stderr to a scratch/debug log is not the defect and must not fire
- [x] Fixture suite `tests/canary-log-hygiene-fixtures.sh` covers: merging line fires, split line passes, a `2>/dev/null` discard fires (loses diagnostics), non-canary redirect ignored, absent dir is a tooling error
- [x] Load-bearing: reintroducing `2>&1` into any crontab makes the check fire; restoring returns it to clean
- [x] `docs/operations/` documents the split-stream idiom so the next canary author copies the right shape

> **⚠️ Shipped ≠ live (G-069).** This fix is committed but **not yet active on this
> host**: `/etc/cron.d` needs root, so the installed crontabs still run the old
> `2>&1` form. `bash scripts/check-cron-install-drift.sh` now reports 21
> UNINSTALLED_JOBS entries — that is this fix awaiting install, not a regression.
> Remediation (per crontab, as root):
> `sudo cp .context/cron/<name>.crontab /etc/cron.d/termlink-<name>`
> then re-run the check until it returns `healthy`. Until then the canary logs on
> this host remain one-bit channels a tooling error can still deafen.

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
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

bash tests/canary-log-hygiene-fixtures.sh
bash scripts/check-canary-log-hygiene.sh --quiet
bash scripts/run-guard-layer.sh --quiet
bash scripts/run-guard-layer.sh --list > /tmp/.t2685-list.out 2>&1 && grep -q "check-canary-log-hygiene.sh" /tmp/.t2685-list.out

## RCA

**Symptom:** `.context/working/.release-mirror-canary.log` contained
`error: origin HEAD empty` while `check-mirror-freshness.sh` itself exited 0 with
"GitHub mirror: synced". Per CLAUDE.md a non-empty release-mirror log directs the
operator to inspect the OneDev job log and rotate `github-push-token` — real work in
response to a fault that did not exist.

**Root cause:** every canary cron job line was written as
`… >> <findings>.log 2>&1`. The `2>&1` merges the stderr stream into the findings
log. Since each canary implements `exit 0 healthy / exit 1 firing / exit 2 tooling
error`, and the findings log's entire documented meaning is "exit 1 happened", a
check that *could not run* wrote into the channel reserved for "the watched thing is
broken". 30 job lines across all 24 crontabs had it.

**Why structurally allowed:** the exit-code split was implemented and documented in
every canary *script*, and both prior charter reviews (T-2468, T-2678) verified those
scripts. Nobody read the *crontab line* that consumes them, which is where the
contract is actually honoured or discarded. Guarding the producer and never the
consumer left the contract unenforced at the only point that mattered. Compounding:
"empty log = healthy" is a one-bit channel, so the first tooling error permanently
deafened the canary — a genuine finding afterwards would append to an already-dirty
log and change nothing an operator could see.

**Prevention:** `scripts/check-canary-log-hygiene.sh` scans every git-tracked crontab
job line that appends to a findings log and fires on `2>&1` (merge) or `2>/dev/null`
(silent discard). It carries the `# guard-layer: source` marker, so T-2684's
`run-guard-layer.sh` executes it alongside every other source-level guard — the next
canary written with the old idiom is caught rather than silently reintroduced.
Proven load-bearing: reintroducing `2>&1` into `dead-letter-canary.crontab` makes it
fire; restoring returns it to clean.

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

### 2026-08-14T05:56:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2685-canary-crontabs-merge-tooling-errors-int.md
- **Context:** Initial task creation

### 2026-08-14T06:07:27Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e64d29b8
- **Timestamp:** 2026-08-27T09:46:58Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-27T09:44:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
