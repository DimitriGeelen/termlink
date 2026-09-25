---
id: T-3134
name: "artifact CLI verbs: termlink artifact put/get"
description: >
  Build task for arc-011 slice S2 (payload may carry a binary blob), created after
  T-3076's inception went GO. Add termlink artifact put <path> --to <peer> and termlink
  artifact get <sha256> --expected-sha256 <sha256> -o <path> as thin CLI wrappers
  over the existing send_artifact_via_client/download_artifact_via_client functions
  (crates/termlink-session/src/artifact.rs), per T-3076's Scope Fence. --expected-sha256
  is mandatory on get per IW-3.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-011]
components:
  - crates/termlink-cli/src/commands/artifact.rs
  - crates/termlink-cli/src/cli.rs
  - tests/artifact-cli-fixtures.sh
related_tasks: [T-3140]
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
created: 2026-09-25T07:15:06Z
last_update: 2026-09-25T10:23:23Z
date_finished: 2026-09-25T10:23:23Z
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
  - ts: '2026-09-25T07:15:51Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-25T10:08:40Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 3
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=3 (body:component-discoverability); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-25T07:16:00Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-25T07:16:38Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (3-components); tier=2 (workflow:build); effort=8 
      (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3134: artifact CLI verbs: termlink artifact put/get

## Context

Blob-on-a-topic exists end to end at the protocol layer (`artifact.put` + `channel.post
--artifact-ref`, `artifact.get`) but no `termlink artifact` CLI subcommand exposes it, so a
shell script (which is what the notify-rail is made of) can post an artifact reference but
cannot move the bytes. Two thin CLI wrappers over the existing
`send_artifact_via_client`/`download_artifact_via_client` functions in
`crates/termlink-session/src/artifact.rs` close the gap — no protocol/hub/MCP changes (T-3076
Scope Fence).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink artifact put <path> --to <peer> [--json] [--timeout N]` uploads the file via
      `send_artifact_via_client` (inline or chunked per the existing 64KB threshold) and prints
      `{ok, sha256, channel_offset, size, via, target}`. `LegacyOnly` (peer hub predates
      artifact.put/channel.post) is a loud, non-zero-exit failure naming the fallback
      (`channel post --file`), never a silent success.
- [x] `termlink artifact get <sha256> --expected-sha256 <sha256> -o <path> [--json] [--timeout N]`
      downloads via `download_artifact_via_client` and writes the bytes to `<path>`.
      `--expected-sha256` is a clap-required (non-`Option`) argument (IW-3: mandatory, not
      optional like `file receive`'s). Before ever contacting the hub, the command verifies
      the positional `<sha256>` and `--expected-sha256` agree (case-insensitive) — a mismatch
      is refused fail-fast with no network call. `download_artifact_via_client` itself already
      verifies the downloaded bytes hash to the requested sha256, so the caller-observable
      guarantee is two-layer: the CLI checks its own two arguments agree, and the library
      checks the bytes match what was asked for.
- [x] Both verbs connect to the LOCAL hub only (`super::infrastructure::resolve_hub_paths()`,
      matching `file send`'s own local-hub pattern) — no new remote/fleet routing, matching
      Scope Fence's "no changes to artifact.put/get protocol methods, hub-side router, or
      capability-fallback logic".
- [x] `cargo build -p termlink` succeeds with the new `Command::Artifact` variant wired
      into `main.rs` (the CLI crate's package name is `termlink`, dir `termlink-cli`).
- [x] Unit tests: clap parse tests (`put` parses `--to`; `get` rejects a missing
      `--expected-sha256`; `get` parses `-o`/`--output`) in `cli.rs::cli_tests`, plus a pure
      `sha256_args_agree` mismatch-detection unit test in `commands/artifact.rs`.
- [x] `tests/artifact-cli-fixtures.sh` exercises the fail-fast mismatch path and both verbs'
      `--help` output against the built binary — hermetic, no live hub required (mirrors the
      T-2153-class convention of testing CLI-observable behavior without live infra).

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

cargo build -p termlink -p termlink-session > /tmp/.t3134-build.out 2>&1 && grep -q "Finished" /tmp/.t3134-build.out
cargo test -p termlink artifact > /tmp/.t3134-clitest.out 2>&1 && grep -q "test result: ok" /tmp/.t3134-clitest.out
cargo test -p termlink cli_tests:: > /tmp/.t3134-clitest2.out 2>&1 && grep -q "test result: ok" /tmp/.t3134-clitest2.out
bash tests/artifact-cli-fixtures.sh > /tmp/.t3134-fixtures.out 2>&1 && grep -q "^ALL PASS" /tmp/.t3134-fixtures.out

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

### 2026-09-25 — manifest.from must be an identity fingerprint, not a pid label
- **What changed:** The Scope Fence said "no changes to the existing functions" and I
  expected that to mean the thin wrappers would work unmodified once wired up. Proving
  `put` end-to-end against a real (isolated, disposable) local hub instead of stopping at
  unit tests surfaced a live `channel.post error -32014: sender_id ... does not match
  identity fingerprint` — `send_artifact_via_client` copies `ArtifactManifest.from`
  verbatim into the signed envelope's `sender_id`, and the hub enforces it must equal the
  signing identity's fingerprint (T-1427). I had copied the `format!("cli-{pid}")` pattern
  from `file.rs::try_send_via_artifact` (the closest precedent) — which turns out to carry
  the same latent bug, unexercised because `file send` is deprecated. Fixed by using
  `identity.fingerprint().to_string()` in my own new call site only, per Hypothesis-Driven
  Debugging (one hypothesis, one test, confirmed by the fixed retry succeeding).
- **Plan impact:** None to T-3134's own scope — the fix is entirely inside the new
  `commands/artifact.rs`, no change to `send_artifact_via_client` itself. But it revealed
  the SAME bug is live today in all three pre-existing callers (file.rs, remote.rs,
  tools.rs), none of which I'm authorized or scoped to touch here.
- **Triggered:** Filed T-3140 (captured, scored, not worked — out of this task's Scope
  Fence and this dispatch's selected unit of work) for the three pre-existing sites.

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

### 2026-09-25T07:15:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3134-artifact-cli-verbs-termlink-artifact-put.md
- **Context:** Initial task creation

### 2026-09-25T10:08:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e890ea40
- **Timestamp:** 2026-09-25T10:23:26Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-25T10:23:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
