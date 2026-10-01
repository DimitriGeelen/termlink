---
id: T-3214
name: "MCP termlink_doctor lacks 3 checks the CLI doctor runs (ufw_listener, secret_cache,
  secret_cache_profiles) and carries an extra 'strict' key"
description: >
  Found by T-2991 parity case parity_doctor (kept #[ignore]). Against an empty runtime
  dir + empty HOME: CLI 'termlink doctor --json' emits 11 checks (runtime_dir, sessions_dir,
  sessions, hub, ufw_listener, sockets, dispatch, secret_cache, secret_cache_profiles,
  identity, version; summary pass=11); MCP termlink_doctor emits 8 (no ufw_listener
  / secret_cache / secret_cache_profiles; summary pass=8) and an extra top-level 'strict'
  key the CLI does not echo. An agent reading MCP doctor cannot see secret-cache drift
  the operator's CLI would show. Decide which side is canonical (the T-2069 convention
  says the CLI verb), port the missing checks, drop or mirror 'strict'; then un-ignore
  the parity case.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-009, parity, bug]
components: [crates/termlink-mcp/src/tools.rs, 
      crates/termlink-cli/src/commands/infrastructure.rs, 
      crates/termlink-mcp/tests/parity.rs]
related_tasks: [T-2991, T-1712, T-1689, T-2069, T-3215]
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
created: 2026-09-28T23:48:32Z
last_update: 2026-10-01T15:49:18Z
date_finished:
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
  - ts: '2026-09-29T00:12:57Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=232,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T07:33:24Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (3-components); tier=2 (workflow:build); effort=8 
      (lines=292,acs=7)
    rubric_sha: e4a00f38e801
---

# T-3214: MCP termlink_doctor lacks 3 checks the CLI doctor runs (ufw_listener, secret_cache, secret_cache_profiles) and carries an extra 'strict' key

## Context

**Measured divergence (T-3211 R3, 2026-09-29, `target/debug/termlink` vs `tools.rs:13146`,
empty runtime dir + HOME with a `sessions/` subdir, on a host that has `ufw`):**

| check | CLI `doctor --json` | MCP `termlink_doctor` |
|---|---|---|
| runtime_dir, sessions_dir, sessions, hub, sockets, dispatch, identity, version | yes | yes |
| `ufw_listener` (T-934; only when `ufw status` succeeds AND names a termlink rule) | yes | **no** |
| `inbox` (T-1001/T-1415; only when the hub socket exists) | yes | **no** |
| `secret_cache` (T-1171/G-011) | yes | **no** |
| `secret_cache_profiles` (T-1284/G-011) | yes | **no** |
| top-level `strict` echo (T-1712) | **no** | yes |
| `ok` under `--strict`/`strict:true` with warnings | `true` (exit 1) | `false` |

So an agent reading MCP doctor cannot see secret-cache drift, a firewall rule with no listener, or
inbox state; and the two `ok` verdicts disagree under strict. `parity_doctor` (T-2991 PAIR 33)
compares the whole envelope minus `message`/`ts_ms`/`pid` and is `#[ignore]`d pending this task.

**Why this is not a straight port (the decision this task needs — SQ-8 in the T-3211 R3
handback).** `termlink-mcp` is a dependency OF `termlink-cli`, not the reverse. The four missing
checks lean on CLI-crate code that is not small: `audit_secret_cache`,
`audit_hubs_for_self_hub_cache`, `crate::config::load_hubs_config` (the hubs.toml parser),
`crate::manifest::DispatchManifest`, `resolve_hub_paths`, `secret_cache_dir`, `sum_inbox_counts`.
Three ways to reach parity, none settled by an existing convention:

1. **Move the doctor down** — extract the check collection into `termlink-mcp` (or
   `termlink-session`) as one function; CLI keeps text rendering, `--fix` side effects and the
   exit code; MCP calls the same function. The `help.rs` / `build_cli_help_json` pattern
   (T-3215 proved it is what makes drift impossible). Cost: the helpers above move crates
   (hubs.toml loader and dispatch manifest included) — a crate-boundary refactor, blast radius
   well beyond the 3 components scored here.
2. **Subprocess the CLI verb** — MCP runs `current_exe() doctor --json [--strict]` under
   `tokio::time::timeout` + `kill_on_drop` (the T-1689 / T-2116 pattern; 5 sites allowlisted in
   `.context/checks/drain-sink-allowlist`). Parity by construction, ~60 lines. BUT the MCP test
   harness is IN-PROCESS (`TermLinkTools::new()` inside the test binary — `parity.rs::mcp_client`,
   `mcp_integration.rs`), so `current_exe()` there is the TEST binary: `test_doctor_empty_env` and
   `test_doctor_with_sessions` would fail, and NONE of the five existing subprocess tools has an
   integration or parity test for exactly this reason. Making it testable means a `TERMLINK_BIN`
   override in the resolver plus tests that depend on a prebuilt CLI — the T-3215 freshness hazard
   moved into the integration suite.
3. **Duplicate the helpers into `tools.rs`** — the T-2069 convention, but that convention is for
   "tiny pure helpers", and a second hubs.toml parser is the duplicated-registry class T-2069 warns
   about (it is how `parity_doctor` came to diverge in the first place).

Also to rule: mirror `strict` into the CLI envelope (and make the CLI's `ok` honour it, so `ok`
agrees with its own exit code) or drop it from MCP. Recommendation, not a decision: option 1
with `strict` mirrored — it is the only one that cannot re-drift, and the CLI already depends on
`termlink_mcp` for exactly this shape in `help.rs`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] AC0 — The operator's ruling on SQ-8 (option 1 / 2 / 3 above, and `strict` mirrored vs
      dropped) is recorded in `## Decisions

### 2026-10-01 — SQ-8 RULED by the operator: option 1b
- **Chose:** the doctor checks move into a small NEUTRAL crate (e.g. `termlink-doctor`) that both the CLI and `termlink-mcp` depend on, so parity is a property of the code. **Mirror `strict`** into the CLI envelope and make the CLI's `ok` honour it. **Annotate** `termlink_doctor` with `readOnlyHint: true` and a `title` (MCP review criteria: every tool carries read/destructive hints; ours carry none).
- **Rejected:** (1) the checks living inside `termlink-mcp` (the crate would own config/audit logic); (2) a subprocess (version skew: this host ran three termlink binaries at different versions until 2026-10-01; it also breaks the in-process tests); (3) duplicated helpers (a second `hubs.toml` parser is the drift class itself).
- **Scoring** (joint value drivers): 1 = +46, 2 = +9, 3 = −20; steelman/strawman given in the session dialogue 2026-10-01.
` BEFORE any source edit. Producer-not-judge: the agent
      that wrote the options does not pick one.
- [ ] AC1 — After the ruling: `termlink_doctor` (MCP) and `termlink doctor --json` (CLI) emit the
      SAME check list — same `check` names in the same order, same `status` per check — against an
      empty runtime dir, and the same `ok` verdict under `strict` with warnings present.
- [ ] AC2 — `parity_doctor` (T-2991 PAIR 33) carries no `#[ignore]` and passes; the two MCP
      integration tests (`test_doctor_empty_env`, `test_doctor_with_sessions`) still pass
      in-process.
- [ ] AC3 — Whichever option lands, no NEW copy of a hubs.toml parser or dispatch-manifest parser
      exists in `tools.rs` (grep for a second `fn load_hubs_config` / `DispatchManifest` impl is
      empty), and `bash scripts/check-drain-sink-caps.sh` + `bash scripts/check-platform-lock.sh`
      scan clean (option 2 adds an `.output()` site → allowlist entry with reason; the `ufw`/`ss`
      spawns must not be re-introduced in the MCP crate un-acknowledged).
- [ ] AC4 — RCA + Evolution filled (root cause: two hand-maintained check lists; prevention: one
      list, or a live parity pair that fails CI on the next divergence).

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

### 2026-09-28T23:48:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3214-mcp-termlinkdoctor-lacks-3-checks-the-cl.md
- **Context:** Initial task creation

### 2026-10-01T15:49:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
