---
id: T-3213
name: "CLI 'agent search --json' emits no JSON on hub-down (T-1914 class) — MCP twin
  returns {ok:false,error}"
description: >
  Found by T-2991 parity case parity_agent_search_no_hub (kept #[ignore]). With no
  hub socket, 'termlink agent search needle --json' exits 1 with EMPTY stdout and
  an anyhow chain on stderr, while termlink_agent_search (MCP) returns {ok:false,
  error:'Hub is not running …'}. Same class PAIR 6 caught for 'channel list' in 2026-06
  (T-1914): an early error path that does not honour --json. Fix: route the hub-down
  branch of cmd_agent_search through json_error_exit when --json is set; then un-ignore
  the parity case.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-009, parity, bug]
components: [crates/termlink-mcp/tests/parity.rs]
related_tasks: [T-2991, T-1914, T-1915]
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
created: 2026-09-28T23:48:21Z
last_update: 2026-09-29T07:29:15Z
date_finished: 2026-09-29T07:29:15Z
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
  - ts: '2026-09-29T00:10:45Z'
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
  - ts: '2026-09-29T07:26:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=8 
      (lines=264,acs=7)
    rubric_sha: e4a00f38e801
---

# T-3213: CLI 'agent search --json' emits no JSON on hub-down (T-1914 class) — MCP twin returns {ok:false,error}

## Context

`cmd_agent_search` (`crates/termlink-cli/src/commands/agent.rs:2914`) validates the empty-query
case through `json_error_exit` when `--json` is set, then calls
`super::channel::fetch_chat_arc_full(hub).await.context(…)?` — a bare `?`. With no hub socket
that returns the `hub_socket()` error ("Hub is not running (no socket at …)"), and the anyhow
chain goes to stderr with EMPTY stdout, exit 1. A `jq` consumer sees a silent empty pipe — the
T-1914 class that T-1915 DRYed across `channel.rs` via `hub_socket_or_json_exit` but which never
reached this `agent.rs` site. MCP `termlink_agent_search` returns `hub_down_err()` →
`{ok:false, error:"Hub is not running …"}`. Measured 2026-09-29 with `target/debug/termlink`
against an empty runtime dir: stdout empty, stderr "Error: Fetching chat-arc full slice for
search / Caused by: Hub is not running (no socket at …)", rc 1. This is the ONLY
`fetch_chat_arc_full(hub)` call in `agent.rs` (`grep -c` = 1), so the fix closes the class at
its single site. Found by T-2991's `parity_agent_search_no_hub` (kept `#[ignore]` pending this).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] AC1 — With `--json` set and no hub socket, `termlink agent search <q> --json` writes
      `{"ok": false, "error": "<msg containing 'Hub is not running'>"}` to STDOUT and exits 1
      (via `super::json_error_exit`, the T-1914/T-1915 convention). Without `--json` the
      human-format path is unchanged (anyhow chain on stderr, exit 1).
- [x] AC2 — The guard covers every failure of the chat-arc fetch on the JSON path (hub-down AND
      a hub-side RPC error), mirroring MCP `termlink_agent_search`, which returns `json_err` for
      both; `agent.rs` still has exactly one `fetch_chat_arc_full(hub)` call site and it is the
      guarded one.
- [x] AC3 — `parity_agent_search_no_hub` (PAIR 31, T-2991) carries no `#[ignore]` and passes
      against a CLI binary built from the fixed tree.
- [x] AC4 — `bash scripts/check-silent-exit.sh` stays clean (the new exit is LOUD by
      construction) and `cargo build -p termlink` compiles warning-free for the touched file.
- [x] AC5 — RCA + Evolution filled: symptom, root cause (bare `?` on the JSON path at the one
      `agent.rs` site the T-1915 DRY pass never reached), why allowed (the T-1915 helper lives
      in `channel.rs` and was applied to `cmd_channel_*` only; nothing scanned `agent.rs` for the
      same shape), prevention (the parity pair is now live, so a regression fails CI via T-2686).

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

cargo build -p termlink --quiet
test "$(grep -c 'fetch_chat_arc_full(hub)' crates/termlink-cli/src/commands/agent.rs)" = 1
d=$(mktemp -d) && mkdir -p "$d/sessions" && ! (TERMLINK_RUNTIME_DIR=$d HOME=$d target/debug/termlink agent search needle --json > /tmp/.t3213-out 2>/dev/null) && python3 -c "import json; d=json.load(open('/tmp/.t3213-out')); assert d['ok'] is False and 'Hub is not running' in d['error'], d"
test "$(grep -c 'ignore = "T-3213' crates/termlink-mcp/tests/parity.rs)" = 0
TERMLINK_BIN=$PWD/target/debug/termlink cargo test -p termlink-mcp --test parity parity_agent_search_no_hub > /tmp/.t3213-par 2>&1 && grep -q 'test result: ok. 1 passed' /tmp/.t3213-par
bash scripts/check-silent-exit.sh > /tmp/.t3213-se 2>&1

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

**Symptom:** `termlink agent search <q> --json` with no hub socket exits 1 with EMPTY stdout and
an anyhow chain on stderr; a `jq` consumer sees a silent empty pipe. MCP `termlink_agent_search`
returns `{ok:false, error:"Hub is not running …"}` for the same state. Caught by T-2991's
`parity_agent_search_no_hub`.

**Root cause:** `cmd_agent_search` guarded the empty-query case with `json_error_exit` but
fetched the chat-arc slice with a bare `?` (`fetch_chat_arc_full(hub).await.context(…)?`), so
any fetch failure — hub-down or hub-side RPC error — bypassed the JSON path entirely.

**Why structurally allowed:** T-1915 DRYed the T-1914 fix into `channel.rs::hub_socket_or_json_exit`
and applied it to the 45 `cmd_channel_*` sites; `agent.rs` verbs reach the hub through
`fetch_chat_arc_full`, a different seam, and nothing scanned that crate for the same shape. The
silent-exit static check (T-2666) keys on `std::process::exit(<literal>)` after a closed block and
cannot see an anyhow `?` that leaves stdout empty; only a behavioural pair can.

**Prevention:** the fetch failure now routes through `json_error_exit` on the JSON path (this fix),
and `parity_agent_search_no_hub` is live — un-ignored — so a regression fails the parity suite that
T-2686 wires into every push/PR. This was the only `fetch_chat_arc_full(hub)` call in `agent.rs`.

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

### 2026-09-29 — one site, both failure modes
- **What changed:** the filing said "route the hub-down branch through json_error_exit". Reading
  the site showed hub-down is not a distinct branch — it is one of two failure modes of the same
  fetch (socket absent vs RPC error), and MCP answers `{ok:false, error}` for both. Guarding the
  `Err` arm of the fetch, not a hub-down probe, is what actually mirrors the MCP twin.
- **Plan impact:** none beyond that; the fix stayed a single `match`. The parity pair asserts
  with `TERMLINK_BIN=target/debug/termlink` in Verification, because the harness default
  (`find_termlink_bin_fresh()` → nested release build) costs >10 min per commit on this host
  (measured in T-3215's Evolution) and a debug build of the CLI is 46s.
- **Triggered:** nothing new. R2's pre-measurement (T-2991 check 18) found 5 of 9 sampled CLI
  verbs already loud on hub-down; the four without any `--json` flag are T-2748 scope (F5 there).

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

### 2026-09-28T23:48:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3213-cli-agent-search---json-emits-no-json-on.md
- **Context:** Initial task creation

### 2026-09-29T07:26:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c03a19c4
- **Timestamp:** 2026-09-29T07:29:26Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T07:29:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
