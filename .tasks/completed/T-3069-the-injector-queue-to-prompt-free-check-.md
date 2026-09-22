---
id: T-3069
name: "The injector: queue to prompt-free check to inject to verify-working to L3"
description: >
  THE missing middle of the rail. Read the journal queue, check the prompt is free
  (be-reachable-pushwaker READY/BUSY/UNKNOWN, built and unused), inject the message,
  VERIFY the agent is actually working on it, and only then post L3 with evidence=idle-gated-inject.
  Without this the rail delivers and confirms but nothing ever reaches an agent. Blocks
  the honest use of L3.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-011]
components: [scripts/be-reachable-pushwaker.sh]
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
created: 2026-09-22T12:51:49Z
last_update: 2026-09-22T21:21:21Z
date_finished: 2026-09-22T21:21:21Z
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
  - ts: '2026-09-22T14:42:57Z'
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
  - ts: '2026-09-22T14:57:32Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=253,acs=12)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-22T14:59:09Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (3-components); tier=2 (workflow:build); effort=8 
      (lines=253,acs=12)
    rubric_sha: e4a00f38e801
---

# T-3069: The injector: queue to prompt-free check to inject to verify-working to L3

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/notify-injector.sh` runs the full chain in order: arrival record →
      agent reachable? → prompt free? → read queue → inject → **verify** → L3
- [x] It SOURCES the PTY classifier from `scripts/lib/pty-state.sh` rather than
      carrying its own copy. Two copies of a subtle heuristic drift, and the one
      that drifts is the one that quietly stops catching things
- [x] **AGENT-NOT-RUNNING is its own outcome**, distinct from BUSY, with its own
      exit code. Operator requirement: "I want to inject, but I check if my agent
      is running, I see it's not running." Treating absent as busy would retry
      forever against a session that no longer exists
- [x] **UNKNOWN defers, never injects.** The classifier is fail-safe biased by
      design (READY only on a positive idle marker) and the injector must preserve
      that bias — a wrong READY is the blind inject this whole rung exists to kill
- [x] **Injection is VERIFIED before L3 is posted.** `termlink inject` returns
      before submission (T-2396, proven live): on a busy or manual-accept session
      the text lands unsubmitted and is discarded. So after injecting, the injector
      re-reads the PTY and requires evidence the text was consumed; if it cannot
      confirm, it posts NOTHING and says so
- [x] L3 is posted with `--evidence idle-gated-inject`, the kind that means the
      injector observed a READY prompt before injecting — and it is the FIRST
      truthful caller of that rung
- [x] Exit contract: 0 injected+verified · 1 nothing to do · 2 tooling ·
      3 agent not running · 4 prompt busy/unknown (deferred) · 5 injected but
      NOT verified (the loud case — a message may be sitting unsubmitted)
- [x] `--dry-run` prints the decision and the message it WOULD inject, and injects
      nothing, so the chain can be inspected on a live host without side effects
- [x] `tests/notify-injector-fixtures.sh` is hermetic (stub `termlink`, fixture
      journal, fixture flag) and pins: not-running ≠ busy, UNKNOWN defers,
      unverified injection posts no L3, a verified injection posts exactly one L3,
      and an empty queue is "nothing to do" rather than an error
- [x] **MET 2026-09-22 (third attempt), after T-3079 removed the blocker.** The full
      chain ran end to end against a live Claude Code REPL and every link was verified
      from the RECEIVING side, never from the sender's own claim:
      * fresh topic `proof:t3079-1790111420`, one message posted at offset 0 (a fresh
        topic deliberately — the arc topic's L3 watermark stood at 145, so a receipt
        there would have been satisfied by history rather than by this run)
      * `journal-mirror.sh` mirrored it, populating the T-3071 `priority` column
      * injector: agent running -> prompt **READY** (the T-3079 classifier; this is
        the step that returned UNKNOWN forever before) -> queue read -> injected ->
        verified by a BUSY transition -> L3 posted -> **L3 read back from the hub**
      * **exit 0**, and independently confirmed by me, not by the script:
        `termlink channel subscribe` shows offset 1 = `receipt stage=read
        evidence=idle-gated-inject up_to=0`. The FIRST truthful machine-earned L3 on
        this rail — the two receipts that existed before carried `operator` (a human
        vouching) and the withdrawn `wake-consumer`.
      * **and the message really reached the prompt**, asserted on the RECEIVER's own
        transcript rather than the PTY: `7b8bcc50-….jsonl` carries it as a `user` turn —
        `[peer-message d1993c2c @proof:t3079-1790111420 #0] T-3079 live proof…` — which
        is the T-2876 DELIVERED verdict, the strongest evidence available.
      **This run also found a real defect and it is fixed here:** the injector logged
      `L3 posted` for a receipt that was never written, because `notify-ack-read`'s
      contract conflates "posted" with "already acked" under exit 0. It now reads the
      receipt back from the hub and refuses to claim a delivery it cannot see;
      disavowed `wake-consumer` evidence does not count. Pinned by fixture T24.

  **Superseded — kept for the record (not a separate criterion; this is the
  history of the AC above, which is now met).** Proven against a REAL session
      end to end, with the L3 receipt read back from the hub.
      **What WAS proven live (2026-09-22):**
      * against the running AEF session `tl-vayovuqm`, the injector reached the
        prompt check and returned **rc=4 (deferred)** — correct, because that is a
        shell session carrying no Claude Code idle markers.
      * the classifier's **fail-safe ORDERING was validated on a real screen**. A
        freshly launched Claude REPL sat on a resume picker whose text matched BOTH
        `resumesession` (modal → UNKNOWN) and `?forshortcuts` (→ READY). The
        UNKNOWN-modal case is tested FIRST, so it deferred. A classifier checking
        READY first would have injected into the picker's search box. That ordering
        was a comment; it is now an observation.
      **What was NOT proven:** the inject → verify → L3 path against a live REPL.
      A session was spawned for it (`inj-proof`), never reached READY in 120s, and
      its own picker text read `rate limited — wait and re…`. Session was stopped
      and deregistered; 0 remain. No claim is made about the full path.
      **ATTEMPT 2 (same day), also failed, and it surfaced something bigger.**
      Rather than spawn another session, all 14 registered TermLink sessions were
      classified. **Every one returned UNKNOWN**, and spot-checking showed why:
      they are SHELL endpoints (AEF, cashweb, pen) or FINISHED dispatch workers
      sitting at `root@host:/path#`. The classifier is correct — there is a
      pushwaker test asserting a raw shell prompt defers — but the implication is
      the finding: **there is currently no live interactive Claude REPL registered
      as a TermLink session on this host, so the injector has no audience.**
      Two failures on this AC, so per the procAsFit binding ("if a task fails its
      acceptance criteria twice, record the failure mode and move on") it is not
      attempted a third time. The blocker is environmental, and it is now
      understood rather than merely observed.

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

bash -n scripts/notify-injector.sh

# 24 assertions. The five outcomes stay distinct, and T24 pins the false claim this
# task's own live run produced: an ack returning 0 without writing a receipt must
# NOT be reported as "L3 posted".
bash tests/notify-injector-fixtures.sh

# The injector sources the classifier rather than copying it, and the classifier's
# own verdicts are asserted from real captured PTY bytes.
bash tests/pty-state-fixtures.sh

# Structural pins for the two properties the live proof depended on.
grep -q 'lib/pty-state.sh' scripts/notify-injector.sh
grep -q 'l3_covered' scripts/notify-injector.sh

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

### 2026-09-22 — the blocker is NOT "no REPL exists". It is the classifier.

Operator approved spawning agents, so the SQ-2 premise was tested directly rather
than reasoned about. It turned out to be **wrong in the half that mattered**, and the
real blocker is one layer down.

**What was believed (SQ-2, recorded twice):** *no interactive Claude REPL is registered
as a TermLink session, so the injector has no audience.* True as an observation — all
sessions probed UNKNOWN — but the inference drawn from it was wrong.

**What is now measured:**

1. **A REPL can be spawned and IS injectable.** `bash scripts/tl-claude.sh start --name
   wake-proof2 --backend tmux -- --continue` produced a live Claude Code v2.1.267 REPL
   inside a TermLink session. `termlink inject wake-proof2 "Reply with exactly:
   WAKE-PROOF-OK" --enter` was **answered** — the response appears in the PTY. So
   inject → run → respond works against a real REPL. The audience exists the moment
   someone spawns one.

2. **The classifier still refuses to authorise the inject, permanently.** Sampled every
   10s for 100s after a completed turn: **no positive marker at any sample**, so
   `pushwaker_probe_pty` returns UNKNOWN and the injector defers (rc=4) forever. That is
   why the live AC failed twice. It was never an audience problem.

3. **Why it refuses — the marker set does not match this UI.** The four positive markers
   are `?forshortcuts | newtask? | checkingforupdate | /cleartosave`. On this host,
   `?forshortcuts` is **absent at every window size** on a chat REPL with auto-mode on;
   the footer reads `⏵⏵ auto mode on (shift+tab to cycle)` instead. Measured presence at
   the 2500-byte default: `shift+tabtocycle` and `automodeon` PRESENT, `?forshortcuts`
   ABSENT.

4. **Widening the window is NOT the fix, and the file already says why.** Measured after
   a completed turn: 2500B → no markers; 4000B → a **stale** `esctointerrupt` reappears;
   6000B → stale busy marker AND idle markers together. Enlarging the tail reintroduces
   exactly the scrollback contamination the docstring warns about, and would pin the
   classifier to BUSY forever. The narrow window is correct; the marker set is wrong.

5. **A measurement trap worth recording.** `scripts/lib/pty-state.sh` references
   `"$TERMLINK"` with **no default**. Sourced from a bare shell that has not set it, the
   probe runs an empty command, reads nothing, and classifies UNKNOWN — indistinguishable
   from a genuine UNKNOWN. My first sweep of five sessions reported all-UNKNOWN through
   this bug, which would have "confirmed" SQ-2 on a broken instrument. A one-line
   `TERMLINK="${TERMLINK:-termlink}"` in the lib would close it.

**CANDIDATE FIX, evidenced but NOT shipped.** The stripped tail ends with the composer
prompt `❯` (U+276F) followed by U+00A0 when the REPL is idle with an EMPTY composer —
which is precisely the condition an injector wants, and it also refuses when text is
already pending (the T-2396 shape). Measured: idle tail ends `…9:25 PM ❯ `;
mid-turn (`esctointerrupt` present) the tail ended with response text (`effort`), not
the prompt. Note `tr -d '[:space:]'` does NOT strip U+00A0 in the C locale, so any
pattern must account for it.

**Deliberately not shipped tonight.** Two live data points is not enough for a
safety-critical heuristic whose failure mode is a blind inject into a busy prompt —
the exact data loss this arc exists to prevent. The honest next step is a quiescence
conjunction (no busy marker AND tail ends with an empty composer AND tail unchanged
across two reads), validated over many turns, with fixtures built from captured PTY
bytes rather than from reasoning. That needs API budget this host does not have right
now: the REPL footer reports **94% of the weekly limit used, resetting Sep 28**.

**Consequence for SQ-2:** it should be re-stated. "Should we arm agents before building
injection machinery?" is answered — arming works and takes one command. The open
question is narrower and different: *make the idle classifier reliable across Claude
Code UI versions*, which is a TermLink concern, not an AEF one.

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

### 2026-09-22T12:51:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3069-the-injector-queue-to-prompt-free-check-.md
- **Context:** Initial task creation

### 2026-09-22T13:10:18Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-011

### 2026-09-22T14:42:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-15e1982c
- **Timestamp:** 2026-09-22T21:21:23Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — It SOURCES the PTY classifier from `scripts/lib/pty-state.sh` rather than
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/lib/pty-state.sh in: It SOURCES the PTY classifier from `scripts/lib/pty-state.sh` rather than`

### 2026-09-22T21:21:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Live AC MET at third attempt after T-3079 removed the classifier blocker: full chain proven end to end on a fresh topic, L3 read back from the hub (evidence=idle-gated-inject), and the message confirmed as a user turn in the receiver's own transcript. Also fixed the injector claiming L3 posted for a receipt never written.
