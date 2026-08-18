---
id: T-2746
name: "Guard: a crate reporting a user-visible version must carry the git derivation"
description: >
  G-019 prevention for T-2744: nothing detects a crate whose user-visible version
  comes from Cargo.toml because it lacks the build.rs derivation its siblings carry

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [scripts/check-version-derivation.sh, 
      tests/version-derivation-check-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T19:41:53Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-15T20:28:28Z
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
  - ts: '2026-08-18T18:56:58Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2746: Guard: a crate reporting a user-visible version must carry the git derivation

## Context

G-019 prevention for T-2744, filed because that task's fix is mitigation for one
instance and not prevention for the class.

T-2744 found `termlink-session` stamping `env!("CARGO_PKG_VERSION")` into every
session's metadata, which resolved to its Cargo.toml constant `0.9.0` — frozen
since it was written — because that crate lacked the `build.rs` git derivation
that `termlink-cli` and `termlink-mcp` both carry. Sessions recorded `0.9.0`
while the binary reported `0.11.720`, for the entire life of the field.

T-2744 added the missing build.rs and a regression test for *that crate*. What
remains uncovered is the mechanism: the convention lives in hand-copied build.rs
files, and a crate simply not having one is not a difference anything checks
for. A fourth crate added tomorrow with the same omission would be caught by
nothing — and, as T-2744 showed, would report a plausible wrong version rather
than failing, which is the Directive #2 shape (a wrong answer, not an error).

Shape to consider, matching the six existing source-level static checks
(`scripts/check-*.sh`, `# guard-layer: source` marker, git-tracked allowlist
under `.context/checks/` per T-2681): flag any crate whose sources reference
`env!("CARGO_PKG_VERSION")` — or otherwise surface a version to a user or to
persisted state — while having no `build.rs` that emits
`cargo:rustc-env=CARGO_PKG_VERSION`. Crates where the Cargo.toml version is
genuinely the right answer get an allowlist entry stating why.

Worth checking during scoping whether the better fix is upstream of the guard:
a small shared build-dependency crate holding the derivation once would remove
the copies the guard would otherwise police. Three hand-copies is itself the
smell. Decide that before writing the check — a guard that enforces a
duplication is worse than deleting the duplication.

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/check-version-derivation.sh` exists, carries the `# guard-layer: source`
      marker, and exits 0 on the current tree
      — `clean — every version-reading crate derives its version (4 scanned, 0 acknowledged)`.
      The 4 are exactly cli / mcp / hub / session, the only crates that read the version.
- [x] The check FIRES (exit 1) on a crate that reads `env!("CARGO_PKG_VERSION")` but has
      no `build.rs` emitting `cargo:rustc-env=CARGO_PKG_VERSION` — proven by temporarily
      neutering `crates/termlink-session/build.rs` and restoring to a zero-diff tree
      — both firing shapes proven separately, each naming a distinct reason:
      emit removed → `has a build.rs but it never emits cargo:rustc-env=CARGO_PKG_VERSION`;
      file removed → `reads the version but has no build.rs` (the actual T-1458/T-2744
      shape). Fired on exactly 1 crate both times; `git diff --stat` empty after restore.
- [x] Control (PL-219): a crate that never reads the version and has no `build.rs` does
      NOT fire, so the check is not the tautology "every crate must have a build.rs"
      — real tree: `termlink-bus`, `termlink-protocol`, `termlink-test-utils` all have no
      build.rs and are not examined. Fixture 2 asserts it directly.
- [x] Allowlist at `.context/checks/version-derivation-allowlist` (git-tracked per
      T-2681) is honoured: listed crates are counted and reported but do not fire;
      removing a line re-fires that crate
      — fixtures 6/7/8 cover all three directions, including `acknowledged_count` in JSON.
- [x] `bash tests/version-derivation-check-fixtures.sh` passes and is hermetic — no
      cargo build, no live binary, fixture crate trees only (covers fire / clean /
      allowlisted / control)
      — 16 assertions, 0 failed. Load-bearing beyond the happy path: disabling the
      comment-stripping filter fails fixture 5 (`prose mentioning the macro does not
      count as a read`) and two knock-on assertions — so the control assertions can fail.
- [x] `bash scripts/run-guard-layer.sh` lists the new check as a member and reports it
      PASS, with the roll-up still green
      — `PASS static-check check-version-derivation.sh`,
      `PASS fixture-suite version-derivation-check-fixtures.sh`,
      roll-up `PASS — 30/30 members clean` (was 28/28).
- [x] The shared-build-dependency-crate alternative is decided in `## Decisions` with
      the evidence, not left as an open question in Context
      — decided against, because both observed instances were a missing file rather than
      a diverged copy, so deduplication would have prevented neither.

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

bash scripts/check-version-derivation.sh
bash tests/version-derivation-check-fixtures.sh
out=$(bash scripts/run-guard-layer.sh 2>&1); echo "$out" | grep -q "check-version-derivation"
bash scripts/run-guard-layer.sh

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

### 2026-08-15 — Static check, not a shared build-dependency crate

- **Chose:** Build the static check. Leave the three hand-copied `build.rs` files as they are.
- **Why:** The Context of this task framed these as alternatives and leaned toward
  deduplication ("a guard that enforces a duplication is worse than deleting the
  duplication"). Scoping showed that framing is wrong on the evidence. Both observed
  instances were a **missing file**, not a **diverged copy**:
  T-1458 (`termlink-hub`) and T-2744 (`termlink-session`) each had no `build.rs` at all.
  A shared build-dependency crate deduplicates the derivation but does nothing about a
  crate that never calls it — it would have prevented **neither** instance. The
  duplication is real and is a smell, but it is not the defect vector, so removing it
  would not have closed the gap this task exists to close.
- **Rejected:** A `termlink-build-version` build-dependency crate. Not wrong, just
  orthogonal — it lowers the cost of adding the derivation without making its absence
  detectable. Worth doing on its own merits later; doing it *instead* of the check would
  have left the class open while looking like it had been addressed, which is the
  failure mode PL-345 describes.
- **Note:** the two are complements, not rivals. If the shared crate is built later, this
  check keeps working unchanged — it tests for the emitted `cargo:rustc-env` line, not
  for how the build script is written.

### 2026-08-15 — A test-only read counts as a read

- **Chose:** Flag a crate whose only `env!("CARGO_PKG_VERSION")` use is inside a test.
- **Why:** PL-148 names precisely this as the tautology trap — `assert_eq!(reported,
  env!("CARGO_PKG_VERSION"))` passes whether the constant is right or wrong, because both
  sides are the same compile-time value. A crate whose only use is that assertion is a
  crate whose version cannot be verified from inside itself, which is the condition worth
  surfacing, not a false positive to suppress.
- **Rejected:** Excluding `#[cfg(test)]` regions. It would need real Rust parsing to do
  correctly, and it would suppress exactly the case most worth reporting.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-15T19:41:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2746-guard-a-crate-reporting-a-user-visible-v.md
- **Context:** Initial task creation

### 2026-08-15T20:20:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-08-15T20:28:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
