---
id: T-3135
name: "Sidecar API: local control surface + portable respawn supervisor"
description: >
  Build task for arc-011 slices S1 (sender sends via a sidecar API) and S12 (roles
  swap - agent replies as sender), created after T-3075's inception went GO. Scope
  per T-3075 Scope Fence: a LOCAL API answering status/queue/inject <id>/agent-state/ack
  <offset> about this host's own mailbox and prompt; an always-respawning supervisor
  for it; re-resolving this host's own FQDN/IP on change. BINDING per operator ruling
  SQ-8 (arc-011.yaml, 2026-09-24, verbatim principle: reliability and fragility, not
  saving a few tokens for a quick solution -- I want a solid solution): systemd-only
  respawn is REJECTED as the cheap answer; the respawn supervisor MUST carry a portable
  fallback (README asserts macOS support in five places and release.yml cross-builds
  two Darwin targets -- a systemd-only respawn would silently narrow what the product
  claims to be). S12's role-swap reply is code-complete once S1's sidecar API exists
  (the agent replies via the same channel.post path already proven end-to-end by T-3069/T-3079);
  its remaining blocker is external to this repo -- framework-agent-systemd's systemd
  unit does not have termlink in --allowed-commands, which is a different project's
  config (T-559 boundary) and cannot be fixed from here.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-011]
components:
  - scripts/notify-sidecar-api.sh
  - scripts/notify-sidecar.sh
  - scripts/notify-injector.sh
  - tests/notify-sidecar-api-fixtures.sh
  - docs/design/arc-011-sidecar-api-architecture.md
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
created: 2026-09-25T07:15:32Z
last_update: 2026-09-28T23:05:04Z
date_finished: 2026-09-28T23:05:04Z
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
      blast_radius: 5
      tier: 2
      effort: 8
    rationale: blast_radius=5 (5-components-medium-blast); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3135: Sidecar API: local control surface + portable respawn supervisor

## Context

Build task for arc-011 slices S1 + S12, scoped by T-3075's Scope Fence (IN: a LOCAL API
answering `status` / `queue` / `inject <id>` / `agent-state` / `ack <offset>` about THIS
host's own mailbox and prompt; an always-respawning supervisor; re-resolving this host's
own FQDN/IP on change. OUT: anything that moves a message BETWEEN hosts) and bound by two
operator rulings: SQ-1 (the injector is a TermLink PRIMITIVE, local-control only — the §6
bright line in `docs/design/arc-011-sidecar-api-architecture.md`) and SQ-8 (systemd-only
respawn REJECTED; a portable fallback is REQUIRED). Nothing here is a new transport: every
verb wraps a seam that already exists and is proven — `notify-sidecar.sh` flag/heartbeat
files, the `journal.sqlite` mirror (T-2298/T-3071), `notify-injector.sh` (T-3069, SQ-4
READY-gated), `notify-ack-read.sh` (T-3067), `scripts/lib/pty-state.sh` (T-2402/T-3069),
and `notify-sidecar-supervisor.sh` (T-3050). ACs written 2026-09-29 by the T-3211 R1
procAsFit worker (the task was created with placeholder ACs and G-020 refused source edits
until real ones existed).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] AC1 — `scripts/notify-sidecar-api.sh` exists and dispatches exactly the five Scope-Fence
      verbs for `--agent-id NAME`: `status`, `queue`, `inject <id>`, `agent-state`,
      `ack <offset> --evidence <kind>`. Every verb reads or acts on THIS host's own state only:
      `status` from `$TERMLINK_NOTIFY_DIR/<agent>.{heartbeat,flag,pid}` + the supervisor pidfile
      + local hub reachability; `queue` from the local journal using the SAME content-row
      predicate and ordering as `notify-injector.sh` (msg_type not in the meta set, priority DESC,
      ts ASC, offset ASC — so the API and the injector can never disagree about what is next);
      `agent-state` echoes READY/BUSY/UNKNOWN/NOT-RUNNING via `scripts/lib/pty-state.sh` (ambiguity
      is UNKNOWN, never READY); `inject` delegates to `notify-injector.sh` (SQ-4: READY-gated, never
      blind) and refuses an `<id>` that is not in the queue; `ack` delegates to `notify-ack-read.sh`
      and refuses without `--evidence`. `--json` on every verb. Exit 0 = answered/acted, 1 = a
      not-OK verdict (DEAF, BUSY, NOT-RUNNING, deferred), 2 = usage/tooling (fail-closed).
- [x] AC2 — Bright-line tripwire (§6, mirrors `no_federation_tripwire.rs`): the API refuses any
      `--hub`, `--peer`, `--to`, or remote-address argument with exit 2 and a message naming the
      bright line; and `tests/notify-sidecar-api-fixtures.sh` statically asserts the script never
      invokes a cross-host verb (`channel post`, `agent contact`, `agent send`, `remote `, `artifact
      put`, `broadcast`). A future edit adding one fails the fixture.
- [x] AC3 — Portable respawn (SQ-8): `scripts/notify-sidecar-supervisor.sh` gains `--loop` (an
      in-process forever loop with its own pidfile + per-cycle heartbeat under `$TERMLINK_NOTIFY_DIR`,
      needing only bash — no systemd, launchd, or cron) and `--emit-unit systemd|launchd|cron|auto`
      which prints the host-native declaration that respawns the loop (systemd unit with
      `Restart=always`, launchd plist with `KeepAlive`, or `@reboot` + periodic cron lines); `auto`
      detects what the host has and states which it chose. The fixture proves: loop mode restarts a
      killed fake sidecar within one interval; each emitted unit is well-formed (plist parses with
      python `plistlib`; unit carries `Restart=`; cron carries `@reboot`); `auto` never exits 0 while
      printing nothing.
- [x] AC4 — Own-identity re-resolution: `status` reports `host_fqdn` and `host_ip`, compares them to
      the last values recorded in `$TERMLINK_NOTIFY_DIR/.host-identity`, records the new values, and
      reports `host_identity_changed=true` with old→new when they differ — loud, never silent. Test
      seams `SIDECAR_API_TEST_FQDN` / `SIDECAR_API_TEST_IP` (PL-213).
- [x] AC5 — S12 recorded honestly: `docs/operations/notify-sidecar-api.md` documents the five verbs,
      exit codes, the bright line, and states that the role-swap reply (S12) travels the EXISTING
      `channel.post` path (`scripts/agent-respond.sh` / `/reply`), deliberately NOT through this API;
      and names the external blocker (framework-agent-systemd's unit lacks termlink in
      `--allowed-commands`, T-559 boundary) as an external dependency, not as done.
- [x] AC6 — `bash tests/notify-sidecar-api-fixtures.sh` passes hermetically (PL-213 seams, no hub,
      no live sidecar) and the pre-existing `tests/notify-sidecar-supervisor-fixtures.sh` still passes
      after the supervisor change.

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

bash -n scripts/notify-sidecar-api.sh && bash -n scripts/notify-sidecar-supervisor.sh
bash tests/notify-sidecar-api-fixtures.sh > /tmp/.t3135-api 2>&1 && grep -q " 0 failed" /tmp/.t3135-api
bash tests/notify-sidecar-supervisor-fixtures.sh > /tmp/.t3135-sup 2>&1 && grep -q " 0 failed" /tmp/.t3135-sup
bash scripts/notify-sidecar-api.sh --help > /tmp/.t3135-help 2>&1 && grep -q "agent-state" /tmp/.t3135-help
bash -c 'bash scripts/notify-sidecar-api.sh status --agent-id x --hub 10.0.0.1:9100 > /tmp/.t3135-bl 2>&1; [ $? -eq 2 ]' && grep -qi "bright line" /tmp/.t3135-bl
bash scripts/notify-sidecar-supervisor.sh --emit-unit launchd > /tmp/.t3135-plist 2>/dev/null && python3 -c "import plistlib; d=plistlib.load(open('/tmp/.t3135-plist','rb')); assert d['KeepAlive'] is True"
bash scripts/notify-sidecar-supervisor.sh --emit-unit systemd > /tmp/.t3135-unit 2>&1 && grep -q "Restart=always" /tmp/.t3135-unit
test -f docs/operations/notify-sidecar-api.md && grep -q "allowed-commands" docs/operations/notify-sidecar-api.md

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

### 2026-09-29 — S1's spec wording is not what SQ-1 permits, and the build follows the ruling
- **What changed:** The arc slice reads "sender SENDS via a sidecar API". SQ-1 (operator,
  2026-09-23) reframed the sidecar as a LOCAL PRIMITIVE and design §6 draws the bright line at
  moving a message between hosts. So what was built answers `status/queue/inject/agent-state/ack`
  about THIS host and refuses every remote-address argument; sending to a peer stays
  `channel.post` via the hub. The slice is recorded "built" in the ruled sense, with the drift
  named in the arc note rather than smoothed over.
- **Plan impact:** S12 (roles swap) cannot be "code-complete once S1 exists" as the description
  claimed — the reply half travels the existing channel.post path, not this API. Recorded PARTIAL:
  the receiving half (truthful L3 `ack`) is built; the reply half is blocked externally
  (framework-agent-systemd `--allowed-commands`, T-559 boundary).
- **Triggered:** no new task. `inject <id>` was narrowed during build to accept only the
  policy-next id (T-3071 ordering must not have a second home); `queue` was made
  hub-independent and labels `watermark_source=local` — the first live run showed 91 pending
  above watermark -1 on a topic whose hub L3 is far higher, so the doc carries that caveat
  explicitly. Portable respawn (SQ-8) landed as `--loop` + `--emit-unit systemd|launchd|cron|auto`;
  `auto` on this host chose systemd and said so.

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

### 2026-09-25T07:15:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3135-sidecar-api-local-control-surface--porta.md
- **Context:** Initial task creation

### 2026-09-28T22:52:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-29ccf1bc
- **Timestamp:** 2026-09-28T23:05:33Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Reviewer:** inline
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-09-28T23:05:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
