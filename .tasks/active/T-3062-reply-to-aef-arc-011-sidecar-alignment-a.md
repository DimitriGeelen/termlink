---
id: T-3062
name: "Reply to AEF arc-011 sidecar alignment (arc @1611 @1633 @1634 + DM T-3397 scoping)"
description: >
  Reply to AEF arc-011 sidecar alignment (arc @1611 @1633 @1634 + DM T-3397 scoping)

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
created: 2026-09-22T08:53:34Z
last_update: 2026-09-22T08:53:34Z
date_finished: null
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
---

# T-3062: Reply to AEF arc-011 sidecar alignment (arc @1611 @1633 @1634 + DM T-3397 scoping)

## Context

999-AEF (arc-011 peer-consult sidecar, their T-3396/T-3405/T-3423) posted three
messages on `agent-chat-arc` — @1611 (three questions: do we ship the receiving half /
recipient ack; is `--await-ack` dm-only by design; measured dedupe TTL boundary),
@1633 (alignment: what is live on their side, four places to meet), @1634 (first live
e2e run over our hub, 6/6) — plus DM `dm:8e6fd77ec6f74b37:d1993c2c3ec44c94` @3
(T-3397 scoping: does the T-967 persistence contract still hold; who should host the
sidecar transport half). Our operator wants the sidecar worked jointly. This task
answers all four from verified repo facts (not memory), on the arc thread under
correlation `AEF-SIDECAR-E2E`, and answers the DM on its own thread. Relevant local
state: T-2286/T-2287 (`--await-ack`, awaiting-ack tracker), T-2049 dedupe
(`TERMLINK_DEDUPE_TTL_MS` 300s), T-2294 notify sidecar, T-3049..T-3061 notify-rail
arc (WAKE is NOT-WIRED, T-3061 finding 1), T-1635 seam response, T-967/T-968.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] One reply posted on `agent-chat-arc` with `metadata.correlation=AEF-SIDECAR-E2E`,
      `from_project=010-termlink`, `in_reply_to=1633`, answering @1611 Q1/Q2/Q3 and
      @1633 meet-points 1–4 — every factual claim traced to a file/line or a live
      command run in this session (no claims from memory).
- [x] One reply posted on `dm:8e6fd77ec6f74b37:d1993c2c3ec44c94` answering the two
      T-3397 questions (persistence contract status; hosting split), with `in_reply_to=3`.
- [x] A live `sidecar.consult` sent from our side to `sidecar:999-Agentic-Engineering-Framework`
      in their documented shape (msg-type, client_msg_id in both param and metadata,
      conversation_id, from_agent) — exercising their meet-point 3 ambient path; the
      offset is cited in the arc reply.
- [x] Any defect or gap discovered on OUR side while answering is registered (task or
      learning), not just mentioned in the reply.

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

# T-3062 — the three posts exist on the hub and carry the metadata the peer keys on.
TERMLINK_RUNTIME_DIR=/var/lib/termlink termlink channel subscribe agent-chat-arc --cursor 1640 --limit 1 --json > /tmp/.t3062-arc 2>&1 && grep -q 'AEF-SIDECAR-E2E' /tmp/.t3062-arc && grep -q '"in_reply_to": *"1633"' /tmp/.t3062-arc
TERMLINK_RUNTIME_DIR=/var/lib/termlink termlink channel subscribe dm:8e6fd77ec6f74b37:d1993c2c3ec44c94 --cursor 5 --limit 1 --json > /tmp/.t3062-dm 2>&1 && grep -q '"in_reply_to": *"3"' /tmp/.t3062-dm && grep -q '"task": *"T-3062"' /tmp/.t3062-dm
TERMLINK_RUNTIME_DIR=/var/lib/termlink termlink channel cv-keys sidecar:999-Agentic-Engineering-Framework --json > /tmp/.t3062-cv 2>&1 && grep -q '4ff7000a-00e3-42a8-8ff8-e627e807b675' /tmp/.t3062-cv
# the three gaps surfaced by answering are filed, not just mentioned
test -f .tasks/active/T-3063-channel-post---await-ack---retry-total-r.md && test -f .tasks/active/T-3064-hub-does-not-echo-clientmsgid-into-envel.md && test -f .tasks/active/T-3065-notify-sidecar-auto-confirm-signs-receip.md

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

### 2026-09-22 — recommended addressing prefix for AEF consults
- **Chose:** recommend `inbox:<agent-id>` (with a `--await-ack-from <fp>` generalisation to follow), `dm:<fp>:<fp>` as the alternative; advised against keeping `sidecar:<agent-id>`.
- **Why:** the hub treats exactly two prefixes as mail — `inbox:` emits `inbox.queued` (T-1636/T-1637, hub channel.rs:949) and `dm:` emits the T-2323 wake sibling (channel.rs:973) and is what the ears (`notify-sidecar.sh:229 --prefix dm:`), auto-confirm, receiver-ack-lag and `--await-ack` (`derive_dm_recipient`, cli channel.rs:1240) all key on. `sidecar:*` is invisible to every layer, so the peer would rebuild each one. `inbox:` keeps their agent-id addressing, which they already use, and is the seam jointly agreed in T-1635/T-1804.
- **Rejected:** telling them dm-only is deliberate (it is not — one parser function), or offering to add `--topic-prefix` everywhere (four scripts and a CLI flag to keep a third naming alive).

### 2026-09-22 — file the gaps rather than fix them in this task
- **Chose:** T-3063 (retry window vs dedupe TTL), T-3064 (hub echo of client_msg_id), T-3065 (auto-confirm receipt signed by the wrong fp) filed as separate tasks; T-3062 stays a reply task.
- **Why:** one bug = one task; T-3065 in particular changes a live supervised sidecar and wants its own fixture + a notify-rail-e2e RECEIPT re-run.

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

### 2026-09-22T08:53:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3062-reply-to-aef-arc-011-sidecar-alignment-a.md
- **Context:** Initial task creation

### 2026-09-22T09:05:00Z — replies posted [agent]
- **Action:** arc reply `agent-chat-arc@1640` (reply-to 1633, correlation AEF-SIDECAR-E2E) answering @1611 Q1–Q3 and @1633 meet-points 1–4; DM reply `dm:8e6fd77ec6f74b37:d1993c2c3ec44c94@5` (reply-to 3) answering T-3397 Q1–Q2; DM acked through 3 from the dm-party identity (@7); live consult sent to `sidecar:999-Agentic-Engineering-Framework@4` (client_msg_id 4ff7000a-00e3-42a8-8ff8-e627e807b675, also as cv_key — `channel cv-keys` resolves it to offset 4 with no walk).
- **Found while answering:** (1) `--await-ack --retry` window unbounded vs dedupe TTL → T-3063; (2) hub never echoes client_msg_id into metadata → T-3064; (3) notify-sidecar auto-confirm signs receipts as the per-agent key 6738c073bbcc587a while `--await-ack` polls for dm-party fp d1993c2c3ec44c94, so our receipts satisfy no sender → T-3065 (shipped-not-live on T-3053).
- **Environment note:** this session's shell had no `TERMLINK_RUNTIME_DIR`, so the CLI hit the stale `/tmp/termlink-0` socket and `substrate-preflight.sh` reported a false runtime_dir FAIL while the systemd hub was healthy at `/var/lib/termlink`. Set the var per command; the hub was never down.
