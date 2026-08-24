---
id: T-2721
name: "Apply worktree-safe hook-path resolution locally in vendored audit.sh (transient
  until re-vendor)"
description: >
  Four audit checks (commit-msg, C-002, CTL-011, CTL-020) report false findings in
  a linked worktree because they concatenate PROJECT_ROOT/.git/hooks or require a
  gitignored cron dir; apply the git rev-parse --git-path fix locally so the audit
  is truthful now, while U-003/U-004 remain the durable upstream path

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
created: 2026-08-15T05:38:10Z
last_update: 2026-08-23T22:06:07Z
date_finished: 2026-08-23T22:06:07Z
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

# T-2721: Apply worktree-safe hook-path resolution locally in vendored audit.sh (transient until re-vendor)

## Context

Sibling of T-2714 (U-003) and T-2715 (U-004), which report these defects upstream.
Those tasks deliberately carry the AC *"No file under `.agentic-framework/` is
edited"* — the vendored tree is overwritten on re-vendor, so a local edit is not a
durable fix. **This task exists so that constraint stays intact**: the local edit is
a separate, explicitly-transient deliverable, not a quiet amendment to T-2714.

**Why do it locally at all.** Four audit checks report findings that are **false in
a linked worktree**, and one of them is a false negative about a *live safety gate*:

| check | site | why it is false here |
|---|---|---|
| commit-msg hook | `audit.sh:2393` | concatenates `$PROJECT_ROOT/.git/hooks/…`; in a worktree `.git` is a **file**, so that path can never exist |
| C-002 research gate | `audit.sh:3022` | same concatenation — claims the C-001 gate is missing while it is installed and enforcing (marker at hook line 166) |
| CTL-011 pre-push | `audit.sh:3375` | same concatenation |
| CTL-020 cron audits | `audit.sh:3236` | `.context/audits/cron/` is gitignored, so it cannot exist in a worktree |

An audit that reports four findings which are not true is worse than one that
reports nothing: it trains its reader to discount the output, and it buries the
warnings that *are* real. Leaving them standing to preserve a vendoring preference
inverts the priority.

**The fix is not a skip for three of the four.** `git rev-parse --git-path
hooks/<name>` resolves a normal checkout **and** a worktree **and** honours
`core.hooksPath` in one call, so those checks now genuinely *verify* rather than
being suppressed. Only CTL-020 is skipped, matching the existing
`fw_is_linked_worktree` idiom already used by the cron-drift and cron-misload checks
in the same file (`audit.sh:1638`, `:1708`) — cron is host-level and its audit
directory is gitignored, so there is nothing in a worktree to verify.

**Transience is the known cost, and it is not hypothetical.** T-2705 re-vendored the
framework on 2026-08-14, ~1000 commits of churn in one commit. The next re-vendor
erases this edit and the four false findings return. That is precisely why U-003 and
U-004 remain the durable path and are NOT closed by this task.

## Acceptance Criteria

### Agent
- [x] All three hook checks resolve via `git rev-parse --git-path hooks/<name>` with a fallback to the literal path, so they work in a normal checkout and a worktree
- [x] The resolver handles git returning a **relative** path (normal checkout) as well as an absolute one (worktree)
- [x] CTL-020 skips on a linked worktree via the existing `fw_is_linked_worktree` helper, using `info` (not `warn`), matching the two cron checks in the same file
- [x] Warning text now prints the **resolved** path rather than a hardcoded `.git/hooks/...` string, so a future reader sees what was actually checked
- [x] `bash -n` passes on the edited script
- [x] A full `fw audit` run confirms the four findings are gone and no new finding appeared in their place *(2026-08-24: a single full run exceeds the 300s tool ceiling and was killed twice at exit 143, so this was verified as two full sections plus two direct predicate tests — see `## Evolution`. Stated rather than glossed: the AC as written was not executed verbatim.)*
- [x] T-2714 and T-2715 remain open with their upstream ACs unticked — this task does not close them, because a local edit is not the prevention *(2026-08-24: T-2714 has since closed — on its own upstream filing at `framework:pickup` offset 35, not on this local edit. The guard this AC encodes held: what closed it was the prevention, not the patch. T-2715 remains open.)*
- [x] The transience is recorded where the next re-vendor will be felt, so the regression is expected rather than rediscovered *(T-2705 §Updates — the re-vendor task itself; a note inside `audit.sh` cannot survive the event it warns about)*

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

# --- The resolver is present, top-level, and used by all three hook checks ---
awk '/^_resolve_hook_path\(\)/{print NR}' .agentic-framework/agents/audit/audit.sh | head -1 | { read n; test -n "$n" -a "$n" -lt 600; }
out=$(grep -c '_resolve_hook_path ' .agentic-framework/agents/audit/audit.sh); test "$out" -ge 3
out=$(sed -n '/_resolve_hook_path()/,/^}/p' .agentic-framework/agents/audit/audit.sh); echo "$out" | grep -q 'PROJECT_ROOT/\$_p'
# --- CTL-020 skips a linked worktree with info, matching its cron siblings ---
out=$(sed -n '3245,3255p' .agentic-framework/agents/audit/audit.sh); echo "$out" | grep -q 'info "CTL-020 skipped'
# --- The script still parses ---
bash -n .agentic-framework/agents/audit/audit.sh
# --- Load-bearing: the FIXED predicate resolves an executable pre-push hook.
# The old predicate ($PROJECT_ROOT/.git/hooks/pre-push) fails here — that failure
# IS CTL-011's false negative, so this passing is the fix doing the work.
h=$(git rev-parse --git-path hooks/pre-push); test -x "$h"
# --- The audit itself now reports C-002 satisfied. This was the worst of the four:
# it claimed a live inception research-artifact gate was missing. ~250s; the full
# multi-section run exceeds the tool ceiling, which is why this is scoped.
out=$(timeout 280 .agentic-framework/bin/fw audit --section oe-research 2>&1); echo "$out" | grep -q 'PASS.*C-002: commit-msg hook has research artifact check'

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

### 2026-08-24 — the last AC could not be run as written, and saying so is the point

- **What changed:** the remaining AC asked for "a full `fw audit` run". A full run
  exceeds the 300s tool ceiling on this corpus and was killed twice at exit 143. It
  was verified instead as two *complete* sections plus two direct predicate tests:
  `enforcement` reports `[PASS] Commit-msg hook installed`; `oe-research` reports
  `[PASS] C-002: commit-msg hook has research artifact check`; CTL-011's fixed
  predicate resolves an executable hook where the old `$PROJECT_ROOT/.git/hooks/`
  form fails; CTL-020's info-skip is present at `audit.sh:3250` and was observed
  live in this session's pre-push audit. All four findings are gone, and the two
  C-001 warnings that remain are genuine and unrelated (inception tasks lacking
  research artifacts).
- **Plan impact:** none to the fix. The AC is ticked with the deviation stated
  inline rather than glossed — this session filed T-2714 about a check that
  reported green over commands it never ran, and quietly ticking "full audit" on
  a scoped audit would be the same defect committed by the task that filed it.
- **Triggered:** nothing new. Two incidental findings were handled in place:
  the audit correctly **fails closed** under lock contention (exit 75, *"no verdict
  produced"*) rather than reporting a false clean — worth knowing, because a first
  dry-run showed both audit checks FAILing and the cause was the concurrent
  background run, not the checks. And `/review/<task>` was returning 500 fleet-wide
  on a stale `dispatch_pause` import; the module was present and tracked, so the fix
  was `fw watchtower restart`, captured as PL-360.

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

### 2026-08-15T05:38:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2721-apply-worktree-safe-hook-path-resolution.md
- **Context:** Initial task creation

### 2026-08-15T05:41:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-108cc660
- **Timestamp:** 2026-08-23T22:06:12Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-23T22:06:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
