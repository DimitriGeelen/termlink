---
id: T-2564
name: "listener-heartbeat: warn loudly when presence post is QUEUED not delivered
  (verb-1 silent-dark fix)"
description: >
  listener-heartbeat: warn loudly when presence post is QUEUED not delivered (verb-1
  silent-dark fix)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/listener-heartbeat.sh, scripts/test-listener-heartbeat.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T11:13:09Z
last_update: 2026-08-23T20:39:32Z
date_finished: 2026-08-23T20:39:32Z
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
  - ts: '2026-08-20T15:20:37Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2564: listener-heartbeat: warn loudly when presence post is QUEUED not delivered (verb-1 silent-dark fix)

## Context

T-2468 purpose-review, silent-failure lens (verb 1 "discover peers"). When the hub
is unreachable, `channel post agent-presence` does NOT fail — the offline queue
absorbs it and returns exit 0 (`PostOutcome::Queued`, channel.rs:1063-1081).
`listener-heartbeat.sh`'s `emit_once` treats ANY rc 0 as delivered (line 197-206)
and never inspects the JSON envelope for the `queued` marker. So during an outage the
producer's own heartbeat loop reports success every cycle while NO heartbeat reaches
the hub's agent-presence topic — peers doing verb-1 discovery (`/peers`, find-idle,
comms-selftest DISCOVER) see the agent as OFFLINE, and the producer is never told
(G-063 "write-only sink nobody noticed", from the sender's side; the literal "why is
there still no response?" from the sender's end). waker-liveness (T-2387) /
frozen-husk (T-2239) both key off a heartbeat that LANDED, so a queued heartbeat is
invisible to them too.

Scope: the MECHANICAL warn (parse the `queued` envelope, emit a loud stderr warning
so the agent knows it is dark on presence). The DESIGN questions (should presence
even queue vs drop; should the loop exit non-zero after N consecutive queued cycles)
are deferred to a separate FILE task — those need judgment on ephemeral-presence
semantics.

## Acceptance Criteria

### Agent
- [x] `emit_once` parses the `channel post --json` envelope and, when the post was
      QUEUED (not delivered), emits a loud one-line stderr warning naming the topic
      and that peers will see this agent as OFFLINE until the hub returns.
- [x] The warn fires on the queued case ONLY — a normally-delivered post stays silent
      (no new stderr noise on the happy path); rc still 0 (back-compat: queuing is not
      a hard failure, matching current loop behaviour).
- [x] Consecutive-queued cycles are tracked and the warning notes the run length
      (so a persistent outage is distinguishable from a one-off blip in the log).
- [x] Verified hub-independently via a test seam that feeds a canned queued envelope
      (no live hub needed); a delivered envelope produces no warn.
- [x] The exact envelope marker used is confirmed against the CLI's actual
      `channel post --json` queued output (not assumed).

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

bash -n scripts/listener-heartbeat.sh
bash -n scripts/test-listener-heartbeat.sh
# queued envelope → loud warn on stderr, rc 0 (delivered → silent)
qerr=$(TERMLINK_HEARTBEAT_TEST_POST_JSON='{"queued":{"queue_id":"q","queue_path":"/x"}}' bash scripts/listener-heartbeat.sh --agent-id vtest --once 2>&1 >/dev/null); echo "$qerr" | grep -q "QUEUED, not delivered"
derr=$(TERMLINK_HEARTBEAT_TEST_POST_JSON='{"delivered":{"offset":1,"ts":1}}' bash scripts/listener-heartbeat.sh --agent-id vtest --once 2>&1 >/dev/null); echo "$derr" | grep -qv "QUEUED" && ! echo "$derr" | grep -q "QUEUED"
grep -q 'queued_run' scripts/listener-heartbeat.sh
# the new T8/T9 unit cases pass (suite as a whole has pre-existing hub-readback fails T4/T5/T7 — out of scope, see T-2565)
out=$(bash scripts/test-listener-heartbeat.sh 2>&1); echo "$out" | grep -q "PASS: T8" && echo "$out" | grep -q "PASS: T9"

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

### 2026-08-09T11:13:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2564-listener-heartbeat-warn-loudly-when-pres.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-6fdd58a3
- **Timestamp:** 2026-08-23T20:39:34Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-23T20:39:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
