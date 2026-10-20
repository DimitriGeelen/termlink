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
- [x] AC0 — The operator's ruling on SQ-8 (option 1 / 2 / 3 above, and `strict` mirrored vs
      dropped) is recorded in `## Decisions

### 2026-10-01 — SQ-8 RULED by the operator: option 1b
- **Chose:** the doctor checks move into a small NEUTRAL crate (e.g. `termlink-doctor`) that both the CLI and `termlink-mcp` depend on, so parity is a property of the code. **Mirror `strict`** into the CLI envelope and make the CLI's `ok` honour it. **Annotate** `termlink_doctor` with `readOnlyHint: true` and a `title` (MCP review criteria: every tool carries read/destructive hints; ours carry none).
- **Rejected:** (1) the checks living inside `termlink-mcp` (the crate would own config/audit logic); (2) a subprocess (version skew: this host ran three termlink binaries at different versions until 2026-10-01; it also breaks the in-process tests); (3) duplicated helpers (a second `hubs.toml` parser is the drift class itself).
- **Scoring** (joint value drivers): 1 = +46, 2 = +9, 3 = −20; steelman/strawman given in the session dialogue 2026-10-01.
` BEFORE any source edit. Producer-not-judge: the agent
      that wrote the options does not pick one.
- [x] AC1 — After the ruling: `termlink_doctor` (MCP) and `termlink doctor --json` (CLI) emit the
      SAME check list — same `check` names in the same order, same `status` per check — against an
      empty runtime dir, and the same `ok` verdict under `strict` with warnings present.
- [x] AC2 — `parity_doctor` (T-2991 PAIR 33) carries no `#[ignore]` and passes; the two MCP
      integration tests (`test_doctor_empty_env`, `test_doctor_with_sessions`) still pass
      in-process.
- [x] AC3 — Whichever option lands, no NEW copy of a hubs.toml parser or dispatch-manifest parser
      exists in `tools.rs` (grep for a second `fn load_hubs_config` / `DispatchManifest` impl is
      empty), and `bash scripts/check-drain-sink-caps.sh` + `bash scripts/check-platform-lock.sh`
      scan clean (option 2 adds an `.output()` site → allowlist entry with reason; the `ufw`/`ss`
      spawns must not be re-introduced in the MCP crate un-acknowledged).
- [x] AC4 — RCA + Evolution filled (root cause: two hand-maintained check lists; prevention: one
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

cargo build -p termlink-doctor -p termlink-mcp -p termlink > /tmp/.t3214-build 2>&1
cargo test -p termlink-doctor > /tmp/.t3214-d 2>&1 && grep -q "test result: ok" /tmp/.t3214-d
cargo test -p termlink-mcp --test mcp_integration test_doctor > /tmp/.t3214-i 2>&1 && grep -q "3 passed" /tmp/.t3214-i
cargo test -p termlink-mcp --test parity parity_doctor > /tmp/.t3214-p 2>&1 && grep -q "2 passed" /tmp/.t3214-p
cargo test -p termlink --bin termlink > /tmp/.t3214-c 2>&1 && grep -q "test result: ok" /tmp/.t3214-c
test -f crates/termlink-mcp/tests/parity.rs && ! grep -q 'ignore = "T-3214' crates/termlink-mcp/tests/parity.rs
test -f crates/termlink-mcp/src/tools.rs && ! grep -qE "fn load_hubs_config|impl DispatchManifest|struct DispatchManifest" crates/termlink-mcp/src/tools.rs
grep -q "termlink_doctor::run_doctor" crates/termlink-mcp/src/tools.rs && grep -q "termlink_doctor::run_doctor" crates/termlink-cli/src/commands/infrastructure.rs
bash scripts/check-drain-sink-caps.sh --no-heartbeat
bash scripts/check-platform-lock.sh

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

**Symptom:** MCP `termlink_doctor` lacked `ufw_listener`, `inbox`, `secret_cache` and `secret_cache_profiles`. It alone echoed `strict`, and the two `ok` verdicts disagreed under strict. An agent reading MCP doctor could not see secret-cache drift or a firewall rule with no listener.

**Root cause:** two hand-maintained check lists. `cmd_doctor` (CLI) and `termlink_doctor` (MCP) were written as parallel copies (the T-1706/T-1712 "G-057 parallel-implementation" pattern). Every check added to the CLI afterwards (T-934, T-1001/T-1415, T-1171, T-1284) landed in one copy only. The helpers those checks need (`audit_secret_cache`, the hubs.toml loader, `DispatchManifest`) lived in the CLI crate, which depends on `termlink-mcp` and not the reverse. So porting a check meant duplicating a parser, and nobody did.

**Why structurally allowed:** nothing compared the two lists. `parity_doctor` (T-2991) found the drift on its first run but was `#[ignore]`d pending this task, so CI stayed green.

**Prevention:** one list. `crates/termlink-doctor::run_doctor` is the only place a check exists, and both surfaces call it. A check added there appears on both, and one added anywhere else is a visible second implementation. `parity_doctor` (un-ignored) and the new `parity_doctor_strict_with_warning` fail CI if the presentation layer diverges (envelope keys, `ok`, `strict`, CLI exit code vs `ok`). The platform-lock and drain-sink scanners now include the new crate, so the moved `ufw`/`ss` spawns are scanned and acknowledged rather than silently out of scope.

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

### 2026-10-20 — what "move the checks down" actually had to move
- **What changed:** the checks were never the hard part. Their data sources were: the hubs.toml model and loader (`config.rs`), and the dispatch-manifest data model (`manifest.rs`). Both moved into `termlink-doctor` as `hubs_config` and `manifest`. The CLI re-exports them, so its ~112 `crate::config::` / `crate::manifest::` call sites and their tests are unchanged. Strict loader, save, profile resolution and git-worktree operations stay in the CLI. `resolve_hub_paths` (T-3340) moved too, because the MCP doctor needs the same `/var/lib/termlink` fallback the CLI uses.
- **Plan impact:** the two scanners' default roots did not include the new crate. Moving the `ufw`/`ss` spawns out of the CLI made `check-platform-lock` report clean with 10 sites scanned instead of 12: green because it had stopped looking. Both scanners now list `crates/termlink-doctor/src`, the spawns are acknowledged at their new signature, and the stale CLI platform-lock entries are gone.
- **Plan impact:** text-mode follow-up lines (stale names, `--fix` outcomes, hints) became `Check::notes`, which only the CLI renders, so `--fix` behaviour and wording are preserved. The CLI now prints its lines after collection rather than while each check runs; the content is the same.
- **Triggered:** fabric cards for the three new files were not created, because `fw fabric register` refuses: no `.fabric/subsystems.yaml` rule matches `crates/termlink-doctor/`. That needs a subsystem entry, which I left for a follow-up because other workers share `.fabric/`.

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

### 2026-10-01 — SQ-8 RULED by the operator: option 1b
- (Copied here from AC0, where the ruling text was recorded inline.) **Chose:** a neutral `termlink-doctor` crate both the CLI and `termlink-mcp` depend on. Mirror `strict` into the CLI envelope and make the CLI's `ok` honour it. Annotate `termlink_doctor` with `readOnlyHint: true` and a `title`. **Rejected:** checks inside `termlink-mcp`; a subprocess; duplicated helpers.

### 2026-10-20 — MCP never applies `--fix`
- **Chose:** `run_doctor` takes `fix`, and MCP always passes `false`. **Why:** the read-only annotation must be true, and remediation stays an explicit operator CLI action. **Rejected:** a `fix` MCP param, which would make the tool destructive and contradict the ruled annotation.

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

### 2026-10-20 — option 1b landed [agent w-mcp]
- **New crate `crates/termlink-doctor`:** `run_doctor(&DoctorOptions) -> DoctorReport` is the one check list: runtime_dir, sessions_dir, sessions, hub, ufw_listener, sockets, dispatch, inbox, secret_cache, secret_cache_profiles, identity, version. `DoctorReport::to_json()` gives `{ok, strict, checks, summary}`, and `ok()` = no fails and, under strict, no warns. Moved in with it: `hubs_config` (HubsConfig/HubEntry, config-dir resolver, read-only loader), `manifest` (DispatchManifest data model + rfc3339 helpers), `resolve_hub_paths(_in)`, and the secret-cache / identity / inbox helpers. The CLI re-exports all of them, so existing call sites and tests are unchanged.
- **CLI:** `cmd_doctor` only renders: text lines plus `Check::notes`, or JSON. Exit code is `!report.ok()`, so the envelope now carries `strict` and its `ok` agrees with the exit code.
- **MCP:** `termlink_doctor` calls `run_doctor` with `fix: false` and is annotated `title = "TermLink doctor"`, `read_only_hint = true`. The local `doctor_ok_rollup` and its 3 tests moved to the doctor crate as `DoctorReport::ok` tests.
- **Tests:** `parity_doctor` un-ignored and passing. New `parity_doctor_strict_with_warning` (orphaned `.sock` → warn, both `ok:false`, CLI echoes `strict`, CLI exit 1). New `test_doctor_annotated_read_only`. `test_doctor_empty_env` now also asserts the secret-cache checks and `strict`. Both doctor integration tests run under a temp HOME (`TempHome` guard), so the shared checks never read the real `~/.termlink`. Results: doctor 4/4; MCP lib 941, integration 101, parity 39 (0 ignored); CLI bin 1220.
- **Static checks:** `check-platform-lock.sh` and `check-drain-sink-caps.sh` default roots now include `crates/termlink-doctor/src`. The `ufw`/`ss` spawns are acknowledged at their new signature `lib.rs::ufw_listener_check` (stale CLI platform-lock entries removed). Both scan clean: 12 and 8 sites. Their fixture suites pass (26/26, 6/6). alloc-sink, silent-exit, busy-spin, version-derivation, parity-census and error-code-emission all clean.
- **Not done:** no fabric cards for the new crate's files. `fw fabric register` refuses until `.fabric/subsystems.yaml` has a rule for `crates/termlink-doctor/` (see Evolution).
