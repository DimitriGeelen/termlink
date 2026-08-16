---
id: T-2760
name: "parity_version and parity_info are non-hermetic — fail on git HEAD movement mid-build"
description: >
  parity_version/parity_info compare git-derived commit+version across MCP and CLI. The MCP test crate is compiled at cargo-test start; find_termlink_bin_fresh rebuilds the CLI at test runtime minutes later. Any commit landing in between guarantees divergence, failing the gate for a build-environment artifact rather than a product defect. Observed: MCP f28e9b857/0.11.1403 vs CLI 5859c89ad/0.11.1405, blocking T-2757 closure.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/tests/parity.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T11:52:06Z
last_update: 2026-08-16T12:10:42Z
date_finished: 2026-08-16T12:10:42Z
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

# T-2760: parity_version and parity_info are non-hermetic — fail on git HEAD movement mid-build

## Context

`parity_version` and `parity_info` compare the git-derived `version` (and, for
`parity_version`, `commit`) reported by the MCP surface against the CLI's. Both
crates derive those values from git at **compile** time via their `build.rs`.

The two sides are not compiled at the same moment:

- The **MCP** side is the test binary itself, compiled when `cargo test --workspace`
  starts. Its `version`/`commit` are frozen at that instant.
- The **CLI** side is `target/release/termlink`, rebuilt by
  `find_termlink_bin_fresh()` at **test runtime** — minutes later on a workspace
  build.

If any commit lands in that window, the two sides are built from different git
HEADs and the comparison fails. The failure names `version` — a field neither
side got wrong — so it reads as a product defect when it is a build-environment
artifact.

The existing `find_termlink_bin_fresh()` mitigation (T-1912) makes this *worse*
in one direction, not better: by guaranteeing the CLI is fresh while the MCP
side stays frozen, it converts "possibly stale binary" into "guaranteed
divergence whenever HEAD moves". Its own comment only anticipates a *stale*
binary, not a *moving* HEAD.

This is not hypothetical or rare. The framework's own automation commits during
long runs — handover generation, VERSION stamping, `last_update` touches — so a
`cargo test --workspace` gate racing a handover is the normal case, not an edge
case. It blocked T-2757's completion gate at 19/21 with exactly this signature:

```
MCP  built at commit f28e9b857 -> version 0.11.1403
CLI  built at commit 5859c89ad -> version 0.11.1405
```

Same class as **PL-220** (staleness checks comparing against the git-derived
VERSION file false-fire).

**The value that must be preserved.** Version comparison is not noise — T-1458
and T-2744 were both real "crate reports a plausible wrong version forever"
defects, and T-2746 exists to catch a *missing* derivation. So the fix must not
simply strip `version` unconditionally; that would trade a false positive for a
blind spot in the exact place two real defects have already landed.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A shared helper detects build skew by comparing the two sides' `commit` values
- [x] When commits DIFFER, `version`/`commit` are excluded from the diff and the test emits a diagnostic naming the skew and both commits — the run does not fail on that field
- [x] When commits MATCH, `version` is compared strictly, so a genuine T-1458/T-2744-class version-derivation defect still fails the test
- [x] Every other field in both envelopes is compared strictly in both cases — skew narrows the comparison to the two git-derived fields only, never the whole envelope
- [x] `parity_version` and `parity_info` both use the helper (the defect is present in both)
- [x] Unit coverage proves both branches: skew-detected (version excluded) and no-skew (version compared)
- [x] `cargo test -p termlink-mcp --test parity parity_version parity_info` passes on the current tree

**Evidence, and one honest limit.** The two integration tests pass on the current
tree (`2 passed; 0 failed`, 672s). That run went through the **strict** path — no
`BUILD SKEW` line was emitted, because a commit landed *before* the CLI rebuild
rather than between the two builds, so both sides saw the same HEAD. So the run
confirms the fix did not break the coherent case; it is **not** a live
demonstration of the skew branch.

The skew branch is proven by unit tests, not by a live skew event:
`build_skew_detected_when_commits_differ` feeds the exact observed pair
(`f28e9b857`/`0.11.1403` vs `5859c89ad`/`0.11.1405`) and asserts both the verdict
and the two excluded fields. `coherent_commits_still_fail_on_a_genuine_version_divergence`
drives the real `diff_json` and asserts it still errors, which is what keeps the
guard load-bearing.

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

# --- The four pure branches of the skew detector ---
out=$(cargo test -p termlink-mcp --test parity -- build_skew_detected_when_commits_differ 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-mcp --test parity -- no_skew_when_commits_match_so_version_is_compared_strictly 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-mcp --test parity -- absent_commit_is_not_treated_as_skew 2>&1); echo "$out" | grep -q "test result: ok"
#
# --- LOAD-BEARING: the guard must still catch a real wrong-version defect ---
# A version divergence under a SHARED commit is the T-1458/T-2744 shape. If this
# test ever passes trivially, the skew path has collapsed into an unconditional
# strip and this file no longer guards anything.
out=$(cargo test -p termlink-mcp --test parity -- coherent_commits_still_fail_on_a_genuine_version_divergence 2>&1); echo "$out" | grep -q "test result: ok"
#
# --- The two real integration tests this task exists to unblock ---
out=$(cargo test -p termlink-mcp --test parity -- parity_version parity_info 2>&1); echo "$out" | grep -q "2 passed"
#
# --- Structural: skew narrows the diff to exactly two fields, never more ---
grep -q 'if skew.skewed { &\["version", "commit"\] } else { &\[\] }' crates/termlink-mcp/tests/parity.rs
#
# --- Structural: both defective tests were migrated, not just one ---
test "$(grep -c 'detect_build_skew(&mcp_json, &cli_json)' crates/termlink-mcp/tests/parity.rs)" = "2"
#
# --- Guard layer must stay clean ---
bash scripts/run-guard-layer.sh

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

**Symptom:** `cargo test --workspace` fails `parity_version` + `parity_info` with
`version` mismatch (`0.11.1403` vs `0.11.1405`) and differing `commit` values,
blocking the P-011 completion gate on tasks that touch neither `info` nor
`version`. Observed blocking T-2757 at 19/21 legs green.

**Root cause:** The two compared surfaces are compiled at different times from a
mutable input. `version`/`commit` are baked in by each crate's `build.rs` at
compile time; the MCP test binary is compiled at `cargo test` start while
`find_termlink_bin_fresh()` rebuilds the CLI at test runtime. Git HEAD is
mutable state read by both builds, so any commit landing between the two builds
makes divergence certain. The test asserts equality across a boundary it does
not control.

**Why structurally allowed:** The build-coherence hazard *was* recognised — both
tests carry a comment about it and T-1912 added `find_termlink_bin_fresh()` — but
the mitigation modelled only a **stale** binary (fixed by rebuilding), not a
**moving** HEAD (made strictly worse by rebuilding, since it guarantees the two
builds straddle any intervening commit). Nothing tested the mitigation against
the case where HEAD moves mid-run, so the gap survived in a test whose comment
claims the problem is handled. Compounding it, the project's own automation
(handover commits, VERSION stamping) routinely commits during exactly the
multi-minute window a workspace test occupies — so the framework generates the
input that breaks its own gate.

**Prevention:** Skew is now *detected* rather than assumed absent: the helper
compares both sides' `commit` and, when they differ, reports the condition
explicitly and narrows the comparison to exclude only the two git-derived
fields. A flaky FAIL becomes a named, self-explaining condition. Crucially the
check stays load-bearing in the coherent case — when commits agree, `version` is
compared strictly, so the T-1458/T-2744 defect class (a crate reporting a
plausible wrong version indefinitely) still fails the test. Unit coverage pins
both branches so a future edit cannot silently collapse the skew path into an
unconditional strip.

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

### 2026-08-16T11:52:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2760-parityversion-and-parityinfo-are-non-her.md
- **Context:** Initial task creation

### 2026-08-16T11:52:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-16T12:10:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
