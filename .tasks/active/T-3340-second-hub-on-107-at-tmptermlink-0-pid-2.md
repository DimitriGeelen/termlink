---
id: T-3340
name: "Second hub on .107 at /tmp/termlink-0 (pid 2919639) splits sessions from inboxes,
  silently (AEF T-3779)"
description: >
  Measured 2026-10-04: hub pid 906293 (/var/lib/termlink, TCP 9100, all sidecar inboxes)
  and hub pid 2919639 (~/.local/bin/termlink, no TERMLINK_RUNTIME_DIR, default /tmp/termlink-0,
  unix only, since 2026-10-03 21:03). Agents started without the env var bind to the
  second hub and are deaf with no signal (AEF T-3779, 055 M1). Find who starts it,
  stop it starting, migrate its sessions, and add detection (preflight or canary:
  more than one hub per host for one uid).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
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
created: 2026-10-04T10:50:14Z
last_update: 2026-10-04T13:18:17Z
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
  - ts: '2026-10-04T13:18:17Z'
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
---

# T-3340: Second hub on .107 at /tmp/termlink-0 (pid 2919639) splits sessions from inboxes, silently (AEF T-3779)

## Context

Read-only investigation 2026-10-04 (sub-agent; nothing changed):
1. pid 2919639 (`/root/.local/bin/termlink hub start`, PPid 1, started 2026-10-03 21:03:26), cgroup
   `system.slice/agentic-fleet-cockpit.service`, cwd `/opt/0506-Voxtype-extention`, env `OPENCODE=1`,
   no TERMLINK_RUNTIME_DIR/XDG_RUNTIME_DIR/TMPDIR. `/tmp/termlink-0/invocation-audit.jsonl.1`: `hub status`,
   `list`, `info`, then `hub restart` at 21:03:27 and `hub start` 109 ms later. Most likely an OpenCode agent
   launched by the fleet cockpit ran `termlink hub restart`, which re-spawns `hub start` detached
   (`crates/termlink-cli/src/commands/infrastructure.rs:732-790`). A hub already ran there before 21:03 (restart
   refuses otherwise, `:736-753`); the original starter predates the audit log (2026-10-02 ~12:20).
2. Code paths: `cmd_hub_start` (`infrastructure.rs:118`) checks "already running" only in its own runtime dir
   (`crates/termlink-hub/src/pidfile.rs:60-91`); `resolve_hub_paths` (`infrastructure.rs:9-35`) prefers the
   default dir whenever its pidfile is Running OR Stale, falling back to `/var/lib/termlink` only when no pidfile
   exists, so once a `/tmp/termlink-0` hub exists every client without the env var latches onto it; MCP
   `hub_restart` has the same logic (`crates/termlink-mcp/src/tools.rs` ~18750-18775). `agentic-fleet-cockpit.service`
   and its tmux server carry no TERMLINK_* env, so every fleet pane resolves `/tmp/termlink-0`.
3. Side risk: `pidfile::check` (`pidfile.rs:29-40`) is a bare process-exists test; a reused PID in a stale
   pidfile would be SIGTERMed by restart.
4. On the second hub: 32 sessions, all PIDs alive (26 on 0.12.103, others 0.12.13-0.12.90); ~23
   `register --shell` task workers, 9 `claude-master-*`. `/var/lib/termlink/sessions` has 43.
5. Detection blind: preflight Check 6 (`scripts/substrate-preflight.sh:749-751`) stops at the first pidfile found;
   no canary counts hubs per uid.
6. Not runme (its only evening log is 21:16, after the start); no commits 20:30-21:30.
Recommended (not applied): `hub start` refuses when another live hub for the uid exists in any candidate dir
(unless `--allow-second-hub`); `resolve_hub_paths` prefers a live `/var/lib/termlink` hub over a stale default
pidfile; pidfile liveness checks `/proc/<pid>/cmdline`; set TERMLINK_RUNTIME_DIR in the cockpit's service and
OpenCode config (055's project: file with them) or `/etc/environment`; a preflight check or canary counting live
hubs per uid; migrate the 32 sessions before the second hub is stopped (operator approval).

## Acceptance Criteria

### Agent
- [x] Pidfile liveness is "a termlink process", not "a process": a pidfile whose PID is alive but not termlink (reused PID) reads Stale, so `hub restart`/`stop` never signal an unrelated process (unit test)
- [x] `resolve_hub_paths`: a Stale default-dir pidfile no longer wins over a live `/var/lib/termlink` hub (unit test with injectable dirs)
- [x] `termlink hub start` refuses, naming the other hub's pid and runtime dir, when another live termlink hub for this uid exists in any other candidate dir (`/var/lib/termlink`, `$XDG_RUNTIME_DIR/termlink`, `$TMPDIR/termlink-$UID`, `/tmp/termlink-$UID`); `--allow-second-hub` overrides (unit test)
- [x] Detection: `scripts/substrate-preflight.sh` gains a check that WARNs when more than one live termlink hub runs for this uid, naming each runtime dir and its session count; hermetic fixture via a seam
- [x] `cargo test -p termlink-hub -p termlink-cli` passes for the touched modules; `cargo build --release` succeeds
- [x] 055 asked to set `TERMLINK_RUNTIME_DIR=/var/lib/termlink` for `agentic-fleet-cockpit.service` and its OpenCode config (their project)

### Out of scope (operator)
Migrating the 32 sessions on `/tmp/termlink-0` and stopping hub pid 2919639: a runme action, after this ships.

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
cargo test -p termlink-hub --lib pidfile > /tmp/.t3340a 2>&1 && grep -q "test result: ok" /tmp/.t3340a
cargo test -p termlink --bin termlink t3340 > /tmp/.t3340b 2>&1 && grep -q "4 passed" /tmp/.t3340b
bash tests/substrate-preflight-single-hub-fixtures.sh > /tmp/.t3340c 2>&1 && grep -q "failed: 0" /tmp/.t3340c
bash scripts/check-platform-lock.sh > /tmp/.t3340d 2>&1 && grep -q "clean" /tmp/.t3340d
bash -n scripts/substrate-preflight.sh

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

### 2026-10-04 — scope of the guard
- **Chose:** `hub start` refuses a second hub only when TERMLINK_RUNTIME_DIR is unset; `--allow-second-hub` overrides.
- **Why:** an explicit runtime dir is a deliberate choice (the systemd unit and the test suites set it); the stray hub had none. Refusing on an explicit dir would let a stray hub block the canonical systemd hub.
- **Rejected:** changing `discovery::runtime_dir()` to prefer /var/lib/termlink for every client (moves all sessions at once, including the 32 live ones; an operator action, not a code side effect).

### 2026-10-04 — test result note
- Full `cargo test -p termlink-hub --lib`: 517/518; the one failure is `channel_subscribe_no_hang_under_concurrent_walks_t2258`, a known load-sensitive 10 s stress test (T-2335, T-3293), in channel.rs which this change does not touch; 1 of 3 isolated reruns passed at load average ~10.

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

### 2026-10-04T10:50:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3340-second-hub-on-107-at-tmptermlink-0-pid-2.md
- **Context:** Initial task creation

### 2026-10-04T13:18:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
