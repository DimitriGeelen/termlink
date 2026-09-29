---
id: T-3215
name: "help catalog drift: termlink_agent_search description differs between MCP registry
  and CLI 'help --json'"
description: >
  Found by T-2991 parity case parity_help (kept #[ignore]). The two help catalogs
  are identical in every category, tool name, deprecated flag and parameter counts
  EXCEPT one description string: MCP termlink_help says 'Search chat-arc by content
  substring (chat-arc ONLY — not dm:* or inbox:*)' while CLI 'termlink help --json'
  says 'Search chat-arc by content substring'. One catalog was edited and the other
  not (the T-2069 duplicated-helper class). The CLI form is what the T-2483 charter-drift
  canary reads. Fix: make both read one registry, or sync the string; then un-ignore
  the parity case.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-009, parity, bug]
components: [crates/termlink-mcp/tests/parity.rs]
related_tasks: [T-2991, T-3199, T-1912, T-1928]
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
created: 2026-09-28T23:48:43Z
last_update: 2026-09-29T07:24:03Z
date_finished: 2026-09-29T07:24:03Z
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
  - ts: '2026-09-29T00:12:47Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=3 (body:portability-abstraction); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-29T00:12:58Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=232,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T06:49:37Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=8 (lines=303,acs=7)
    rubric_sha: e4a00f38e801
---

# T-3215: help catalog drift: termlink_agent_search description differs between MCP registry and CLI 'help --json'

## Context

Filed by T-2991 (arc-009) when `parity_help` reported the two help catalogs differing in one
`description` string. The premise "one catalog was edited and the other not" cannot hold: the
CLI's `termlink help --json` is a wrapper over `termlink_mcp::build_cli_help_json`
(`crates/termlink-cli/src/commands/help.rs:128`) — ONE registry, two renderers. What differed
was the BINARY, not the registry: `find_termlink_bin()` resolves `target/release/termlink`
without rebuilding, and that binary predated commit `7a114d1e6` (T-3199, 2026-09-28 17:42) which
added "(chat-arc ONLY — not dm:* or inbox:*)" to the string, while the MCP side was compiled
fresh INTO the test binary. R2 of T-3211 re-ran the pair against a rebuilt release binary at
02:07 and it agreed, but hit its session limit before recording that anywhere verb-gated; its
edit to `crates/termlink-mcp/tests/parity.rs` (switch the no-hub pairs to
`find_termlink_bin_fresh()`, drop the `#[ignore]`) sat uncommitted. This task disposes the finding
as NOT-A-DEFECT with the evidence recorded, and lands the harness fix so a stale prebuilt binary
can never again present as registry drift.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] AC1 — Root cause established as a harness artefact, not a registry drift: `help.rs` builds
      its JSON via `termlink_mcp::build_cli_help_json` (single registry), and the description
      string in question was changed by commit `7a114d1e6` (T-3199) AFTER the `target/release`
      binary the first run compared against was built. No edit to `tools.rs` or `help.rs` is
      made by this task (`git diff --stat HEAD -- crates/termlink-mcp/src/tools.rs
      crates/termlink-cli/src/commands/help.rs` is empty at close).
- [x] AC2 — Harness fix landed: every no-hub parity pair that runs a prebuilt CLI
      (`no_hub_pair`, `parity_help`, `parity_doctor`) resolves the binary through
      `find_termlink_bin_fresh()` (which runs `cargo build -p termlink --release` once per test
      process), so the CLI side is always the current tree; `parity_help` carries no `#[ignore]`.
- [x] AC3 — `cargo test -p termlink-mcp --test parity parity_help` passes as a live (not ignored)
      test: the catalogs are byte-identical after both sides are built from the same tree.
- [x] AC4 — `bash scripts/check-mcp-parity-census.sh` still scans clean (asserted 33, unexamined 0).
- [x] AC5 — RCA + Evolution sections filled: symptom, root cause (stale prebuilt binary vs
      fresh test crate), why the harness allowed it (`find_termlink_bin` documents it does NOT
      rebuild — T-1912 — and the pair used it anyway), prevention (AC2), and the lesson that a
      parity "drift" whose CLI side is a prebuilt binary must be re-checked against a fresh
      build BEFORE a task is filed.

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

grep -q 'termlink_mcp::build_cli_help_json' crates/termlink-cli/src/commands/help.rs
test "$(grep -c 'ignore = "T-3215' crates/termlink-mcp/tests/parity.rs)" = 0
test "$(grep -c 'find_termlink_bin_fresh()' crates/termlink-mcp/tests/parity.rs)" -ge 3
test -z "$(git diff --stat HEAD -- crates/termlink-mcp/src/tools.rs crates/termlink-cli/src/commands/help.rs)"
# Freshness precondition for the next line: the prebuilt CLI must postdate BOTH registry sources
# (this is the exact property whose absence produced the false drift). Rebuild with
# `cargo build -p termlink --release` if it fails — >10 min on this host (T-3211 R3 measured),
# which is why the test is run with the helper's documented TERMLINK_BIN override instead of
# letting find_termlink_bin_fresh() nest that build inside the test process.
test target/release/termlink -nt crates/termlink-mcp/src/tools.rs && test target/release/termlink -nt crates/termlink-cli/src/commands/help.rs
TERMLINK_BIN=$PWD/target/release/termlink cargo test -p termlink-mcp --test parity parity_help > /tmp/.t3215-help 2>&1 && grep -q 'test result: ok. 1 passed' /tmp/.t3215-help
bash scripts/check-mcp-parity-census.sh > /tmp/.t3215-census 2>&1

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# ── Asserting an ABSENCE: prove the search could have succeeded (T-3144) ──
#
# `! grep -q "PATTERN" file` exits 0 when the pattern is absent. It ALSO exits 0
# when the file was renamed, deleted, or is empty — so the leg cannot distinguish
# "the bad thing is not there" from "I could not look", and the gate reports green
# over a check that never ran. Pair every absence assertion with something that
# fails if the search could not happen:
#
#     test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first
#     grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion
#     cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer
#
# Count-equals-zero is the same defect wearing a different hat, and it is the one
# that bites hardest over a COMMAND's output rather than a file:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]   # WRONG
#
# If cargo is missing, or dies before emitting diagnostics, there are no `^error`
# lines, the count is 0, and the leg passes — a build gate that goes green
# precisely when the build could not run. Measured in this corpus, not invented.
# Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
#
# T-3144 censused 2853 task files: 71 absence assertions, 41 already correct, 30
# not. The convention mostly works — this note is here so the next one is written
# right, because a vacuous leg is invisible until the day the path moves.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## RCA

**Symptom:** `parity_help` (PAIR 32, T-2991) failed on its first run: MCP `termlink_help` and CLI
`termlink help --json` agreed on every category, name, flag and parameter count but differed in
the `description` of `termlink_agent_search` — MCP carried "(chat-arc ONLY — not dm:* or inbox:*)",
the CLI did not. It was filed as registry drift (T-2069 duplicated-helper class).

**Root cause:** the two surfaces share ONE registry — `help.rs` calls
`termlink_mcp::build_cli_help_json` — so registry drift is impossible by construction. The CLI
side of the comparison was `target/release/termlink` resolved by `find_termlink_bin()`, which
(per its own T-1912 doc comment) does NOT rebuild; that binary predated commit `7a114d1e6`
(T-3199, 2026-09-28 17:42) which added the parenthetical. The MCP side was compiled fresh into
the test crate. Same tree, two build times, one string.

**Why structurally allowed:** the harness offered both `find_termlink_bin()` and
`find_termlink_bin_fresh()`, documented the difference, and left the choice to each pair; the
no-hub pairs picked the non-rebuilding one because they do not compare git-derived metadata,
which is the only case the fresh helper's doc names. A content comparison against a prebuilt
binary is exactly as time-sensitive, and nothing said so.

**Prevention:** every pair that runs a prebuilt CLI now resolves it through
`find_termlink_bin_fresh()` (`no_hub_pair`, `parity_help`, `parity_doctor`); a stale release
build can no longer masquerade as drift. Process rule, recorded in Evolution: a parity "drift"
whose CLI side is a prebuilt binary is re-checked against a fresh build BEFORE a task is filed.

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

### 2026-09-29 — the defect was in the measurement, not the registry
- **What changed:** at filing (T-2991, R2 of T-3211) the finding read as one catalog edited
  without the other. Reading `help.rs` shows a single registry with two renderers; the only
  way the strings can differ is two build times. `git log -S` dated the string change (T-3199,
  17:42) between the release build the first run used and the test crate's compile.
- **Plan impact:** the description's proposed fix ("make both read one registry, or sync the
  string") is moot — they already do. The work is a harness fix plus an honest NOT-A-DEFECT
  disposition, not a source change to `tools.rs` / `help.rs`.
- **Triggered:** the sibling `parity_doctor` (T-3214) was re-confirmed by R2 against the SAME
  rebuilt binary and still diverges (10 CLI checks vs 8 MCP) — that one is real and stays
  filed. T-3213 (`agent search --json` silent on hub-down) is a CLI code path, not a build
  artefact, and stays filed. No new task.

### 2026-09-29 — the fresh helper's nested build is the suite's long pole on this host
- **What changed:** `find_termlink_bin_fresh()` nests `cargo build -p termlink --release`
  inside the test process. Measured in T-3211 R3: a top-level release build took 12m29s, and
  `parity_help` with the nested build did not finish within 590s. `build.rs` re-runs on every
  `.git/logs/HEAD` change (T-1057 — correct for version freshness), and this repo receives
  commits from several concurrent sessions, so "up to date" rarely survives long enough for the
  nested build to be a no-op. R2's 498s full-suite time was this cost, not `ENV_LOCK`.
- **Plan impact:** the Verification block runs the pair with the helper's documented
  `TERMLINK_BIN` override and asserts freshness explicitly (`test <bin> -nt <registry sources>`),
  which is the exact property the false drift lacked. AC2 stands: the harness default remains the
  fresh build, so an unattended run is still safe — just slow.
- **Triggered:** finding carried to the T-3211 R3 handback (F7). Not a new task from here —
  whether the nested build should be replaced by a freshness assertion is a harness-wide
  question for T-2748's parity backlog, not this disposition.

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

### 2026-09-28T23:48:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3215-help-catalog-drift-termlinkagentsearch-d.md
- **Context:** Initial task creation

### 2026-09-29T06:49:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-01a1cd28
- **Timestamp:** 2026-09-29T07:24:07Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T07:24:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
