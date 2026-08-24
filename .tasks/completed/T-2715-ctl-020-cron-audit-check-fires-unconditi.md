---
id: T-2715
name: "CTL-020 cron audit check fires unconditionally in linked worktrees"
description: >
  audit.sh CTL-020 tests a gitignored host-local cron dir that a worktree can never
  have, and its mitigation would install cron pointing at a transient worktree

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T20:19:47Z
last_update: 2026-08-24T17:24:58Z
date_finished: 2026-08-24T17:24:58Z
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
  - ts: '2026-08-23T19:13:29Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=2
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

# T-2715: CTL-020 cron audit check fires unconditionally in linked worktrees

## Context

`fw audit` warns *"CTL-020: Cron audit directory missing
(<worktree>/.context/audits/cron)"* in every linked worktree, and the directory it
looks for is one the project has deliberately arranged never to exist there.

`audit.sh:3213`:

```sh
CRON_DIR="$AUDITS_DIR/cron"
if [ -d "$CRON_DIR" ]; then
    ...
else
    grace_warn "CTL-020: Cron audit directory missing ($CRON_DIR)" \
         "Directory not created" \
         "Run: fw audit schedule install"
fi
```

`AUDITS_DIR` is `$PROJECT_ROOT/.context/audits`, and `.gitignore:54` carries
`.context/audits/cron/`. The directory is host-local by design and is never
checked in, so a `git worktree add` cannot produce it. Cron itself is host-level
and runs from the main checkout — the audit already says so twice, skipping the
cron-drift and cron-misload checks with *"linked worktree (cron is host-level,
managed from the main checkout)"* (`audit.sh:1644`, `audit.sh:1723`). CTL-020 is
the same class and was simply not given the same guard.

**The mitigation is actively harmful here**, which is what makes this worth
filing rather than tolerating. *"Run: fw audit schedule install"* installs a
host-level cron entry pointing at `$PROJECT_ROOT` — in a worktree, that is a
directory which will be deleted when the worktree is removed. The operator would
be creating a permanent cron job aimed at a transient path. `audit.sh:1714` already
names this exact hazard for the sibling check: *"A linked worktree derives a
worktree-named target that is never installed."* CTL-020's mitigation walks
straight into it.

**The naive fix is also wrong.** `mkdir -p .context/audits/cron` clears this
warning by converting it into a different one — the populated branch then reports
*"CTL-020: No cron audit files in last hour"*, because an empty directory has no
recent files. It trades a truthful "this check does not apply here" for a false
"continuous auditing is broken", and fabricates host state to do it. That is the
PL-341 trap: making a metric look better by changing what it measures.

Sibling of T-2711, T-2713 and T-2714, all found in the same audit pass and all the
same shape — a check whose verdict rests on a layout assumption that no longer
holds. This one is the cheapest to fix: the helper (`fw_is_linked_worktree`) and
the precedent (two adjacent call sites) are already in the file.

**Cross-repo.** `agents/audit/audit.sh` is vendored; a local edit is erased on
re-vendor. Deliverable is an upstream report, not an edit under
`.agentic-framework/`.

## Acceptance Criteria

### Agent
- [x] The report cites `audit.sh:3213` (CTL-020 site) and `.gitignore:54` together, showing the directory is gitignored by design and therefore unreachable in a worktree
- [x] It cites the two existing skips (`audit.sh:1644`, `audit.sh:1723`) and the `fw_is_linked_worktree` helper as the established in-file precedent
- [x] It states why the current mitigation is harmful, not merely useless: `fw audit schedule install` in a worktree aims a host cron entry at a path that will be deleted
- [x] It states why `mkdir -p` is not the fix — it converts the warning into "no cron audit files in last hour" and fabricates host state
- [x] A concrete remedy is proposed: guard the CTL-020 block with `fw_is_linked_worktree "$PROJECT_ROOT"` and emit the same `info` skip line the two siblings use
- [x] Filed to `framework:pickup` and the post confirmed present
- [x] No file under `.agentic-framework/` is edited by this task

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

# The claim rests on three facts; each is checked rather than asserted.
grep -q "^\.context/audits/cron/$" .gitignore
grep -q 'CRON_DIR="$AUDITS_DIR/cron"' .agentic-framework/agents/audit/audit.sh
grep -q "fw_is_linked_worktree" .agentic-framework/agents/audit/audit.sh

# --- The report exists and cites each thing the ACs require ---
out=$(cat docs/reports/T-2715-ctl020-cron-audit-worktree-blindness.md); echo "$out" | grep -q 'audit.sh:3213'
out=$(cat docs/reports/T-2715-ctl020-cron-audit-worktree-blindness.md); echo "$out" | grep -q '.context/audits/cron/'
out=$(cat docs/reports/T-2715-ctl020-cron-audit-worktree-blindness.md); echo "$out" | grep -q 'audit.sh:1638'
out=$(cat docs/reports/T-2715-ctl020-cron-audit-worktree-blindness.md); echo "$out" | grep -q 'fw_is_linked_worktree'
# the two arguments that distinguish this from a cosmetic warning
out=$(cat docs/reports/T-2715-ctl020-cron-audit-worktree-blindness.md); echo "$out" | grep -q 'will disappear'
out=$(cat docs/reports/T-2715-ctl020-cron-audit-worktree-blindness.md); echo "$out" | grep -q 'fabricates host state'
out=$(cat docs/reports/T-2715-ctl020-cron-audit-worktree-blindness.md); echo "$out" | grep -q 'CTL-020 skipped'
# --- The claims are TRUE of this tree. A report that only greps itself would
# pass while every fact in it was wrong, so these re-measure independently. ---
git check-ignore -q .context/audits/cron/
test ! -d .context/audits/cron
out=$(grep -c 'fw_is_linked_worktree "$PROJECT_ROOT"' .agentic-framework/agents/audit/audit.sh); test "$out" -ge 2
grep -q 'fw_is_linked_worktree()' .agentic-framework/lib/paths.sh
# --- Filed upstream and the post is present (>=37 == our offset 36 landed) ---
out=$(timeout 40 termlink channel info framework:pickup --json); echo "$out" | python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin)["count"]>=37 else 1)'
# --- No file under .agentic-framework/ was edited by this task (G-062) ---
test -z "$(git status --porcelain .agentic-framework/)"
# The deliverable is an upstream report, so the vendored tree must stay untouched.
test -z "$(git status --porcelain .agentic-framework)"

## RCA

**Symptom:** CTL-020 warns *"Cron audit directory missing"* on every `fw audit`
run in a linked worktree, and neither of the two obvious responses is correct —
the printed mitigation would install a host cron entry aimed at a path that gets
deleted, and creating the directory by hand converts the warning into a different
false one.

**Root cause:** the check tests `[ -d "$PROJECT_ROOT/.context/audits/cron" ]`.
That path is gitignored (`.gitignore:54`) precisely because it is host-local
continuous-audit output, so it exists only where cron actually runs — the main
checkout. A linked worktree is guaranteed to fail the test regardless of whether
continuous auditing is healthy. The check conflates "this directory is here" with
"continuous auditing is configured", and those diverge the moment the audit runs
from anywhere but the main checkout.

**Why structurally allowed:** the audit already knows how to handle this. It
carries `fw_is_linked_worktree` and uses it at two adjacent sites
(`audit.sh:1638`, `audit.sh:1708`) to skip the cron-drift and cron-misload checks
with an explicit *"cron is host-level, managed from the main checkout"* info line.
Those two were fixed reactively, each when it started failing (T-2437 and its
predecessor), rather than by sweeping every cron-dependent check at once — so
CTL-020 was left behind. The class was identified and the third instance was not
searched for. Nothing exercises the audit against a worktree layout, so a check
that cannot pass there is indistinguishable from one reporting a real fault.

**Prevention:** guarding CTL-020 fixes this instance; it does not stop the fourth.
The durable guard is a fixture that runs `fw audit` inside a `git worktree add`
and asserts no check reports a failure whose cause is the worktree layout itself.
That is the same missing test T-2714 needs for the hooks-path family — one
fixture would cover both, which is an argument for fixing them together upstream.

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

### 2026-08-14T20:19:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2715-ctl-020-cron-audit-check-fires-unconditi.md
- **Context:** Initial task creation

### 2026-08-14T20:21:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-95b7d4e2
- **Timestamp:** 2026-08-24T17:25:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-24T17:24:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
