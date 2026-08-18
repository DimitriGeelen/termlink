---
id: T-2744
name: "Session metadata records a frozen termlink_version, not the build version"
description: >
  Session metadata records a frozen termlink_version, not the build version

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/registration.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T19:30:28Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-15T19:43:02Z
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
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2744: Session metadata records a frozen termlink_version, not the build version

## Context

Found while scoping herdr adoption backlog rank 12 (worker 4, R1 subset), which
proposes `termlink sessions --stale-binary` on the stated grounds that
`metadata.termlink_version` "is already written (`registration.rs:220`, `:286`),
so it is a read over data we already have".

**That premise is false, and the field is inert.** `registration.rs:286` records
`env!("CARGO_PKG_VERSION")`, which resolves against `termlink-session`'s own
`Cargo.toml` — pinned at `0.9.0` and never moved. The git-derived version this
project actually versions by comes from a `build.rs` that emits
`cargo:rustc-env=CARGO_PKG_VERSION`, and that override applies only to the crate
being built. `termlink-cli` has such a build.rs. `termlink-mcp` has one.
`termlink-session` — the crate that writes the field into session metadata —
does not.

Measured on this host: every live session reports
`"termlink_version":"0.9.0"` while `termlink --version` reports `0.11.720`.
The field has recorded the same constant for every build ever made.

This is PL-344 in its plainest form: a correct answer reached on two surfaces and
never propagated to the third, and the third is the one that persists the value
other tools are invited to trust. It is also why rank 12 must not be built first.
A detector over this field would compare `0.9.0` against the running version and
return either "every session is stale" or "none are", forever, with no way to
tell from the output that it was reading a constant — a guard whose verdict rests
on an assumption about its input that does not hold (PL-343), shipped green.

Scope here is the defect only: make the recorded version the build version. The
detector is refiled as a follow-up so it can rest on data that carries
information.

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
> **MEASURED (2026-08-15).** A probe session registered against a scratch
> `TERMLINK_RUNTIME_DIR` records
> `"termlink_version":"0.11.1359"`, and `termlink --version` reports
> `termlink 0.11.1359` — exact match. Before the fix the same read returned
> `0.9.0` against a binary reporting `0.11.720`. Probe and scratch dir removed
> afterwards; no real session state was touched.
>
> **LOAD-BEARING PROOF.** Replacing the git-derived value in `build.rs` with the
> Cargo.toml constant fails
> `recorded_version_is_the_build_version_not_the_cargo_toml_constant` with
> `left: "0.9.0", right: "0.9.0"` — the exact frozen value the defect produced.
> Restoring returns the crate to 463/463 and the tree to zero diff.

- [x] `termlink-session` derives its version from git the way `termlink-cli` and `termlink-mcp` already do, so `metadata.termlink_version` records the build that registered the session
- [x] A live `termlink list --json` shows a newly-registered session carrying the same version `termlink --version` reports — measured, not inferred from the code
- [x] The build.rs re-run triggers match the CLI's, so the version does not freeze after the first build (the T-1057 bug this project already paid for once)
- [x] A test pins that the recorded version is not the stale `0.9.0` crate constant, so the defect cannot silently return
- [x] Load-bearing proof recorded: removing the git derivation makes that test fail
- [x] The follow-up detector task is filed with the corrected premise, so rank 12 is not picked up again on the false one (T-2745)

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

cargo test -p termlink-session --lib
cargo test -p termlink --bins
bash scripts/run-guard-layer.sh

## RCA

**Symptom:** every session registration recorded
`metadata.termlink_version: "0.9.0"` regardless of which binary registered it.
On this host, live sessions reported `0.9.0` against a binary reporting
`0.11.720`.

**Root cause:** `registration.rs:286` stamps `env!("CARGO_PKG_VERSION")`. This
project versions by git tags, and the derivation is done by a `build.rs`
emitting `cargo:rustc-env=CARGO_PKG_VERSION` — an override that applies **only
to the crate being built**. `termlink-cli` had that build.rs. `termlink-mcp` had
one. `termlink-session`, the crate that actually persists the value, did not, so
its `env!` resolved to its own Cargo.toml constant.

**Why structurally allowed:** two things, and the second is the general one.
(i) Nothing ever read the field back, so a wrong value cost nothing and stayed
invisible — it only surfaced when herdr rank 12 proposed *depending* on it.
(ii) More generally, nothing detects a crate that reports a user-visible version
without carrying the derivation. The convention lives in two hand-copied
build.rs files; a third crate simply not having one is not a difference any
check looks for. That is PL-344 — a correct answer reached on some surfaces and
never propagated — with no structural backstop, exactly the shape the repo's
static checks exist to convert from *discipline* into *enforcement*.

**Prevention:** for this instance,
`recorded_version_is_the_build_version_not_the_cargo_toml_constant` fails if the
derivation is removed, and it compares against a build-script-exported copy of
the Cargo.toml value rather than a hardcoded literal — so the test cannot go
stale the way the field it guards did. For the general class, prevention does
**not** yet exist: a fourth crate added tomorrow with the same omission would
not be caught. Filed as T-2746 rather than left implicit, since a gap that is
mitigated but not prevented is still open (G-019).

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

### 2026-08-15T19:30:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2744-stale-binary-session-detector--termlink-.md
- **Context:** Initial task creation

### 2026-08-15T19:43:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
