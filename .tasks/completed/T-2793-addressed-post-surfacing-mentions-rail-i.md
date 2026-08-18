---
id: T-2793
name: "Addressed-post surfacing: mentions rail is built but dark, no sender sets it
  and no daily verb reads it"
description: >
  Addressed-post surfacing: mentions rail is built but dark, no sender sets it and
  no daily verb reads it

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-18T18:33:07Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-18T18:43:30Z
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
  - ts: '2026-08-18T18:57:01Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 5
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=5 (body:silent-class-removed); 
      D3=2 (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2793: Addressed-post surfacing: mentions rail is built but dark, no sender sets it and no daily verb reads it

## Context

Two independent operators — this agent and 999-AEF — each wrote a post to
`agent-chat-arc` naming the other as recipient in the body, and each found the other's
post only by manually sweeping the topic. Both posts were durable and both arrived; the
failure was not delivery. It was **addressing**: to every surfacing verb either side
owns, a post that names its recipient is an undifferentiated broadcast. The unread count
is honest and useless — it says "something happened" on a topic where something always
happens (a `-vendored` heartbeat lands hourly on two hubs).

999-AEF stated it at arc @114: *"My unread count showed it; my inbox did not surface it
as addressed to me. Symmetric failure, both directions, which is a stronger signal than
either of us hitting it alone."* Confirmed at arc @115 and homed here, since this is
TermLink's transport (their T-1333 gap-homing rule).

**The capability already exists and is dark.** T-1513 shipped `metadata.mentions` as a
structured envelope field plus `termlink agent mentions <USER>` / `channel mentions-of`
to read it. Measured this session: **zero callers.** No script or skill sets
`metadata.mentions` when posting (`agent-send.sh`, `chat-arc-broadcast.sh`,
`agent-respond.sh` all omit it), and no daily verb reads it — `grep -rl mentions
scripts/ .claude/commands/` returns exactly two files, `lint-doc-cli-references.sh` and
`check-error-code-emission.sh`, both of which merely grep source text and neither of
which uses the feature. `/check-arc` surfaces unread `dm:*` topics and chat-arc
broadcasts, but has no notion of "addressed to me".

This is the T-2683 class — a capability nothing executes — in the comms rail rather than
the guard layer, and it has now cost two real operators real time in a single session.

**Scope of this task (one deliverable):** the DETECTOR. A check that answers "is there a
post on a broadcast topic that addresses me and that I have not acked?" It must catch
the two posts that actually occurred, which means it cannot rely on `metadata.mentions`
alone — neither post set it. Wiring senders to tag outgoing posts is the natural sequel
and is deliberately NOT in scope here (one task = one deliverable); it is filed
separately if this lands.

## Acceptance Criteria

### Agent
- [x] `scripts/check-addressed-posts.sh` exists, is executable, and is correctly tiered.

      **AC CORRECTED DURING BUILD — as written it was wrong.** It required the
      `# guard-layer: source` marker. That marker means "safe to run anywhere: no live
      hub, no network, no host state", and this check reads a live topic over the
      network: in CI it would exit 2 on every run and park a permanent ERROR in the
      guard-layer roll-up, which is precisely how a roll-up stops being read. Marking it
      as declared would have satisfied the AC and damaged the thing the AC exists to
      protect. It is a **runtime cron canary** (`.context/cron/addressed-posts-canary.crontab`,
      4-hourly, `/canaries` discovers its log); its hermetic FIXTURE SUITE is the
      guard-layer member. Verified both ways: `run-guard-layer.sh --list` shows the
      suite as a member and the check as unclassified, and fixtures L1/L2 now assert the
      absence of the marker rather than its presence.
- [x] Detects an addressing post by EITHER signal: structured `metadata.mentions`
      matching self, OR a body-text match against self-identity aliases — because the two
      real posts (arc @105, @114) set no `metadata.mentions` and a detector that only
      reads the structured field would have missed both, which is the exact failure
      being fixed. Fixture C (alias-only) and D (mentions-only) each fire alone.
- [x] Distinguishes ADDRESSED-UNACKED (fires) from ADDRESSED-ACKED and
      NOT-ADDRESSED (quiet). Proven on the live topic, not asserted: 115 envelopes → 15
      addressed → 2 past the ack frontier (@114, @115). After `channel ack --up-to 115`
      the same command returned rc=0 with no output. Fixture E pins the acked case.
- [x] Exit contract matches the sibling checks: 0 / 1 / 2, fail-closed. An unreadable or
      empty topic response exits 2 with `"refusing to report clean"` rather than
      reporting nothing addressed to me (fixture J2, plus a live check against
      `/dev/null` state).
- [x] Fixture suite `tests/addressed-posts-check-fixtures.sh` passes 29/29 and group C
      reproduces the real arc @114 payload (recipient named in body, `metadata.mentions`
      absent) and REQUIRES a fire. C5 additionally pins the jq `.`-rebinding regression
      that made every alias match every payload.
- [x] Every output path carries the scope disclaimer, including the clean one and the
      `die()` JSON error path. Two further degradations are declared rather than
      swallowed: `aliases_configured:false` when the ledger is empty, and
      `identity_resolved:false` / `ack_frontier_available:false` when identity cannot be
      resolved (fixtures H1/H2/K1/K2).

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
       Conversion: this AC should be moved to ### Agent and this line added to
       ## Verification (herestring, not a pipeline — see the L-387 hint below):
         out=$(bin/fw reviewer T-XXX 2>&1 || true); grep -q "Overall:.*PASS" <<< "$out"
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
# Pipefail/SIGPIPE hint (L-387, corrected by T-2775): P-011 runs each command
# under `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141
# (SIGPIPE) when grep matches and closes stdin while the upstream is still
# writing — verification then "fails" BECAUSE the check succeeded, and the
# earlier the match, the more reliably it fails.
#
# USE ONE OF THESE — both measured rc=0 at 3M lines:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# The herestring is preferred: a herestring spawns no producer process, so there
# is nothing to SIGPIPE and it cannot regress as output grows. In the second form
# the pipeline sits inside a command substitution, whose status is discarded — the
# OUTER `test` decides.
#
# DO NOT capture-then-pipe. This template previously prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# and it is size-dependent, not safe: `echo`/`printf` is a producer like any
# other, so once $out exceeds the pipe buffer it is still writing when `grep -q`
# exits and pipefail propagates 141. The capture bounds the DATA but does not
# remove the PRODUCER. Anything wrapping `cargo test`, `fleet doctor --json`, or a
# full log is already in that size range. (T-2775 measured this; 999-AEF L-613 and
# 050-email-archive PL-161 published the capture-then-pipe form before the
# correction — both have since adopted the herestring.)
#
# Corollary (T-2090): intermediate stages are just as fatal — `... | tail -3 |
# grep -q PAT` re-introduces the same risk. With a herestring the question does
# not arise; grep scans the whole captured string anyway.
#
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before the hint;
# T-2775 then measured 1490 exposed lines across 802 tasks despite the hint, which
# is why `scripts/check-verification-pipefail.sh` now enforces it structurally.
#
out=$(bash tests/addressed-posts-check-fixtures.sh 2>&1 || true); grep -q "29 passed, 0 failed" <<< "$out"
test -x scripts/check-addressed-posts.sh
out=$(head -12 scripts/check-addressed-posts.sh 2>&1 || true); grep -q "runtime cron canary" <<< "$out"
out=$(bash scripts/run-guard-layer.sh --list 2>&1 || true); grep -q "addressed-posts-check-fixtures.sh" <<< "$out"
out=$(bash scripts/check-canary-log-hygiene.sh 2>&1 || true); grep -q "clean" <<< "$out"
test -f .context/cron/addressed-posts-canary.crontab
out=$(cat .context/cron/addressed-posts-canary.crontab 2>&1 || true); grep -q "log.stderr" <<< "$out"
test -f .context/checks/addressed-aliases
# Both detectors are declared on the JSON contract, and the scope disclaimer rides every path.
out=$(ADDRESSED_TEST_STATE_JSON=/dev/null bash scripts/check-addressed-posts.sh --no-heartbeat --json 2>&1 || true); grep -q '"scope"' <<< "$out"

# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## RCA

**Symptom.** Two operators, opposite ends of the same conversation, same session: each
wrote a post to `agent-chat-arc` naming the other as recipient, and each found the
other's only by manually sweeping the topic. 999-AEF's post @102 sat ~2h before a hand
sweep found it; their receipt @114 the same. Neither side's tooling said a word.

**Root cause.** TermLink has no notion of an ADDRESSED broadcast post. A post is either
a DM (`dm:<a>:<b>`, surfaced by `/check-arc`) or an undifferentiated broadcast. Naming
the recipient in the body — the natural thing to do, and what both sides actually did —
puts a post in the second category. The only signal left is the topic's unread count,
which on `agent-chat-arc` is honest and useless: a `-vendored` heartbeat lands hourly on
two hubs, so the count always says "something happened".

**Why structurally allowed.** The capability existed and was dark. T-1513 shipped
`metadata.mentions` plus `agent mentions` / `channel mentions-of` to read it. Measured at
filing: `termlink agent mentions '*'` returned `[]` across all 115 envelopes — the
structured rail had never once carried a value — and `grep -rl mentions scripts/
.claude/commands/` returned two files, both of which merely grep source text. Nothing
sets the field; nothing reads it in any daily path. That is the T-2683 class (a
capability nothing executes) in the comms rail rather than the guard layer. Both halves
had to be dark for the miss to happen, and both were.

**Prevention.** `scripts/check-addressed-posts.sh` + a 4-hourly cron canary, firing when
a post addresses me and sits past my ack frontier. Distinct from the fix in one
important way: it does not require senders to change anything. It reads
`metadata.mentions` when present AND matches an alias ledger against the body, because a
detector reading only the structured field would have been blind to both of the posts
that motivated it — a guard built from an incident that cannot see the incident.

**Three silent-green failures found while building it, each caught only by running it
against the live topic.** Recording them because the pattern is the point: every one
produced a confident, plausible, wrong verdict, and none produced an error.

1. *jq `.`-rebinding.* `select($body | contains(.))` rebinds `.` to `$body` inside the
   pipe, so every alias matched every payload. The check fired on 114 of 115 envelopes —
   read as maximum signal, worth exactly zero. Pinned by fixture C5.
2. *Non-discriminating `sender_id`.* Excluding my own posts by `sender_id` looked
   obviously correct and silently zeroed the whole check: all 115 envelopes on this hub
   share one `sender_id` (a relay fingerprint, not a per-author identity), so the
   exclusion dropped everything and reported a serene `addressed_total: 0` over a topic
   holding two posts written straight at me. Now applied only where the field actually
   discriminates, and said out loud where it does not. Pinned by fixtures F1/F2/G1.
3. *Unresolved identity degrading quietly.* `whoami` is ambiguous on this host (the
   T-2690 PID-ancestor case) and no `be-reachable.state` carries an `agent_id`, so
   identity resolution failed — taking the ack frontier and self-exclusion with it. The
   check still printed a verdict. It now declares `identity_resolved:false` and
   `ack_frontier_available:false` on every output path. Pinned by fixtures H1/H2.

The irony is load-bearing rather than decorative: a check written specifically to stop
confident-wrong-answers produced three of them before it worked, and every one was
invisible to its own fixtures until it met real data. That is the argument for the live
run, not for more unit tests.

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

### 2026-08-18T18:33:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2793-addressed-post-surfacing-mentions-rail-i.md
- **Context:** Initial task creation

### 2026-08-18T18:43:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
