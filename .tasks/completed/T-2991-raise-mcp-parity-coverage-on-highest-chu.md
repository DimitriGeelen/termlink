---
id: T-2991
name: "Raise MCP parity coverage on highest-churn tools.rs regions (T-2748 ratchet)"
description: >
  S-17/C-06 gate: work the T-2748 parity-census allowlist down on the highest-churn
  tools.rs regions. Precondition for any tools.rs/channel.rs split (S-18). Evidence:
  docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md C-06.

status: work-completed
workflow_type: test
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
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
created: 2026-09-19T22:14:46Z
last_update: 2026-09-29T00:09:57Z
date_finished: 2026-09-29T00:09:57Z
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
  - ts: '2026-09-20T08:45:10Z'
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
  - ts: '2026-09-28T23:44:04Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 3
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=3 (body:portability-abstraction); 
      F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 1
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=1 
      (workflow:test); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-27T21:34:07Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 1
      effort: 8
    rationale: blast_radius=1 (1-file-ref-derived-T-3189); tier=1 
      (workflow:test); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2991: Raise MCP parity coverage on highest-churn tools.rs regions (T-2748 ratchet)

## Context

S-17 / C-06 (value-review consolidated): `tools.rs` is the #1-churn file (374 commits in 180d)
and only 24 of 260 MCP tools (9.2%) are asserted against their CLI verb by
`crates/termlink-mcp/tests/parity.rs`; the other 236 sit in
`.context/checks/mcp-parity-census-allowlist` as a ledger (T-2747). T-2748 is the ratchet
that works that ledger down; this task is its first slice, aimed at the regions that change
most, because that is where silent MCP/CLI drift is produced (C-26: one 45-day undetected
drift already).

**Churn is MEASURED, not asserted.** Per-tool ranking = number of distinct commits in the
last 180 days whose diff hunks fall inside that tool's handler region of `tools.rs` (region =
from its `#[tool(name = "…")]` marker to the next marker). Caveat stated up front: free
helper fns that sit between two markers are attributed to the preceding tool, so a tool
followed by a large helper block over-counts (e.g. `termlink_chat_arc_broadcast`). The ranking
is recorded in `docs/reports/T-2991-tools-rs-churn.md` with the script that produced it.

Scope fence: this is a TEST task. A divergence the new cases find is RECORDED (test comment +
Evolution + its own task), not fixed here — one bug = one task.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Churn ranking for `tools.rs` (180d, distinct commits per tool region) is recorded in `docs/reports/T-2991-tools-rs-churn.md` together with the measurement script and its stated attribution caveat — **DONE:** 374 commits analysed, 262 tool regions touched, top-45 listed, caveat in §Method.
- [x] At least 6 tools from the top-25 of that ranking gain a parity case in `crates/termlink-mcp/tests/parity.rs` (MCP call vs `termlink … --json`, hub-independent or on the shared session fixture), and each is removed from `.context/checks/mcp-parity-census-allowlist` — **DONE:** 9 cases (PAIR 25–33); 8 of the 9 tools are in the top-25 (`channel_unread` is #41); allowlist 236 → 227 entries.
- [x] `cargo test -p termlink-mcp --test parity` passes with the new cases (run without committing mid-run, T-2687); a new case that FAILS on genuine MCP/CLI drift is kept as `#[ignore]`-with-reason and the drift filed as its own task, never silently patched under this task — **DONE:** full suite `34 passed; 0 failed; 3 ignored` (497.9s on this contended host); the 3 ignored are PAIR 31/32/33 on drift owned by T-3213 / T-3215 / T-3214.
- [x] `bash scripts/check-mcp-parity-census.sh` is clean and reports `covered ≥ 30` (was 24), and the allowlist header's stated counts are updated to match — **DONE:** `clean — 260 MCP tool(s): 33 asserted, 227 acknowledged, 0 unexamined`, 12.6%; header updated to 33 / 227 / 12.6%.
- [x] Every divergence the new cases surface is listed in this task's Evolution with the task ID that owns it (or "none found" stated explicitly) — **DONE:** three found, three tasks (T-3213, T-3214, T-3215), see Evolution.

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

test -f docs/reports/T-2991-tools-rs-churn.md && grep -q "Attribution caveat" docs/reports/T-2991-tools-rs-churn.md
test "$(grep -cE '^termlink_(help|doctor|channel_ack_status|channel_state|channel_subscribe|channel_thread|channel_unread|inbox_list|agent_search)[[:space:]]' .context/checks/mcp-parity-census-allowlist)" = "0"
bash scripts/check-mcp-parity-census.sh --json > /tmp/.t2991-census 2>&1 && python3 -c "import json; d=json.load(open('/tmp/.t2991-census')); assert d['ok'] and d['covered']>=30 and d['unexamined']==0, d"
grep -q "33 of 260 MCP tools (12.6%" .context/checks/mcp-parity-census-allowlist
timeout 580 cargo test -p termlink-mcp --test parity -- parity_channel_ack_status_no_hub parity_channel_subscribe_no_hub parity_channel_thread_no_hub parity_channel_state_no_hub parity_inbox_list_no_hub parity_channel_unread_no_hub > /tmp/.t2991-cargo 2>&1 && grep -q "test result: ok. 6 passed; 0 failed" /tmp/.t2991-cargo
test "$(grep -c '#\[ignore = "T-32' crates/termlink-mcp/tests/parity.rs)" = "3"
test -f .tasks/active/T-3213-cli-agent-search---json-emits-no-json-on.md && test -f .tasks/active/T-3214-mcp-termlinkdoctor-lacks-3-checks-the-cl.md && test -f .tasks/active/T-3215-help-catalog-drift-termlinkagentsearch-d.md

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

### 2026-09-29 — R2 (T-3211): nine cases on the top-churn regions; three of them found drift on their first run
- **What changed:** The task shipped with placeholder ACs and a cost estimate read off template
  boilerplate; the first activity was measuring churn (374 commits / 180d, per-tool-region
  attribution with a stated caveat) and writing ACs from it. Of the nine tools that gained a case,
  SIX pass (`channel_ack_status`, `channel_subscribe`, `channel_thread`, `channel_state`,
  `inbox_list`, `channel_unread` — all hub-down envelope parity, the T-1914 contract) and THREE
  found genuine MCP/CLI drift on first execution, each kept `#[ignore]`-with-reason and filed as
  its own task per the scope fence:
  - **T-3213** — `termlink agent search --json` prints NOTHING on stdout on hub-down (anyhow chain
    on stderr, exit 1) while the MCP twin returns `{ok:false,error}`. The T-1914 class, four months
    after PAIR 6 caught it for `channel list`.
  - **T-3214** — `termlink_doctor` (MCP) emits 8 checks where the CLI emits 11: `ufw_listener`,
    `secret_cache`, `secret_cache_profiles` are CLI-only, and MCP adds a `strict` key the CLI
    does not echo. An agent reading MCP doctor cannot see secret-cache drift.
  - **T-3215** — the two help catalogs agree on every category / name / flag / parameter count
    except ONE description string (`termlink_agent_search`).
  Census: 24 → **33 asserted** (12.6%), 236 → 227 acknowledged, 0 unexamined.
- **Plan impact:** The "≥6 tools from the top-25" AC is met by 8 (plus `channel_unread`, #41). The
  full suite takes ~500s on this host because `ENV_LOCK` serialises every case and the host runs
  ~450 agent processes — the P-011 line therefore runs only the six new passing cases (~60s); the
  full-suite result (`34 passed; 0 failed; 3 ignored`) is recorded in the T-3211 R2 handback ledger.
  Four top-churn CLI verbs have NO `--json` flag at all (`batch tag`, `batch exec`, `deregister`,
  `agent chat-arc-recent`) and cannot be asserted by this harness — a CLI gap for T-2748's next slice.
- **Triggered:** T-3213, T-3214, T-3215 (arc-009, `parity,bug`). T-2748 (the ratchet parent) still
  has placeholder ACs; this slice's method + the no-json gap are its natural next scope.

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

### 2026-09-19T22:14:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2991-raise-mcp-parity-coverage-on-highest-chu.md
- **Context:** Initial task creation

### 2026-09-19T22:35:30Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-28T23:43:00Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-cea1f750
- **Timestamp:** 2026-09-29T00:10:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T00:09:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
