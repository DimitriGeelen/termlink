---
id: T-2712
name: "Fabric watch patterns exclude the guard layer — coverage measured over a subset"
description: >
  Widen watch-patterns.yaml to cover scripts/tests that already carry cards, and register the newly-visible files

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T18:50:18Z
last_update: 2026-08-14T19:07:26Z
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

# T-2712: Fabric watch patterns exclude the guard layer — coverage measured over a subset

## Context

`fw audit` reported *"Fabric: 48 card(s) point at files no watch pattern
covers — the registry already treats these as components; drift checks cannot
see them, so coverage is measured over a subset"*. The uncovered cards were
almost entirely the **guard layer**: 34 under `scripts/` (the `check-*.sh`
canaries and static checks), plus the shell and Rust test suites. Those are the
files this project documents most heavily, and someone had deliberately
registered cards for them — but `.fabric/watch-patterns.yaml` listed only
`crates/*/src/**/*.rs`, `lib/`, `web/`, `agents/`, `bin/`, so drift detection
could not see a single one.

**The interesting part is what that did to the numbers.** Card-edge coverage
read `31/150 (18%) have no edges`, which sounds like a healthy graph. It was
measured over the subset the patterns happened to match. Widening the patterns
and registering what they expose moves it to `193/344 (56%)` — and *nothing got
worse*. Those 188 files always had no edges; they were simply not being counted.
18% was the flattering number, 56% is the real one.

That is the same failure this repo has closed repeatedly one layer down: a guard
reporting green over a narrower scope than it appears to cover (T-2680's canary
that called 214 tools clean while scanning for six name patterns; T-2681's
allowlists whose green depended on untracked local state). It is worth stating
plainly because the honest fix makes a headline metric look three times worse,
and the next person to read `193/344` will be tempted to "improve" it by
narrowing the patterns again.

**Deliberately left uncovered (4 cards).** `install.sh`,
`systemd-templates/*.service`, `docs/guides/upstream-reporting.md`, and
`.claude/commands/capture.md` have cards because they are real components, but
they are not source files and inventing globs to chase four singletons would
make the pattern list meaningless. The residual 4 is intentional, not a
leftover.

## Acceptance Criteria

### Agent
- [x] `.fabric/watch-patterns.yaml` covers the file classes that already carry cards: `scripts/**/*.sh`, `tests/**/*.sh`, `crates/*/tests/**/*.rs`, `scripts/**/*.py`, `crates/*/build.rs`
- [x] The widening is justified in-file, so the next reader knows why the list grew and does not narrow it back
- [x] `fw fabric drift` reports every watched file registered (was: 4 unregistered against the narrow patterns; now 352/352 against the wide ones)
- [x] The `card(s) point at files no watch pattern covers` warning drops from 48 to 4, and the residual 4 are named and justified rather than silently tolerated
- [x] The edge-coverage metric shift (18% → 56%) is recorded as a measurement-scope change, NOT reported as a regression or quietly omitted
- [x] Registration is done via `fw fabric scan` + `fw fabric enrich` so derived edges are captured (56 new edges across session/cli/mcp/protocol/agent-mesh), not by hand-writing skeletons

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

# Drift must be clean against the WIDE patterns (the narrow-pattern clean bill
# was the defect, so this only means anything with the new globs in place).
out=$(.agentic-framework/bin/fw audit --sections structure 2>&1); echo "$out" | grep -q "Fabric drift: all"
# The uncovered-card warning is down to the 4 deliberate non-source singletons.
out=$(.agentic-framework/bin/fw audit --sections structure 2>&1); echo "$out" | grep -q "4 card(s) point at files no watch pattern covers"
# The guard layer is actually watched now — spot-check a canary and a fixture suite.
test -f .fabric/components/scripts-check-stuck-claims-freshness.yaml
test -f .fabric/components/tests-stuck-claims-check-fixtures.yaml
# The patterns file still explains itself (guards against a silent re-narrowing).
grep -q "T-2712" .fabric/watch-patterns.yaml

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

### 2026-08-14T18:50:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2712-fabric-watch-patterns-exclude-the-guard-.md
- **Context:** Initial task creation

### 2026-08-14T18:51:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
