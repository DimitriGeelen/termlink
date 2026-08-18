---
id: T-2736
name: "interact timeout must name the cause when a child terminal query goes unanswered
  (herdr item 8)"
description: >
  interact timeout must name the cause when a child terminal query goes unanswered
  (herdr item 8)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/pty.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T12:49:27Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-15T12:57:09Z
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
  - ts: '2026-08-18T18:56:57Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=1 (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2736: interact timeout must name the cause when a child terminal query goes unanswered (herdr item 8)

## Context

Herdr adoption backlog item 8 (rank 8), scoped by the backlog to legibility
only. Reading the handler found the defect was larger than filed.

The backlog's framing: nothing in the tree reads child output looking for DSR
`CSI 6n` / `CSI 14t|16t` / OSC 10/11/4 and replies, so a child that queries and
waits blocks to the deadline with an empty diff. Worker 1's cluster analysis is
the right root: **TermLink models a PTY nobody is watching** — which is
charter-correct, and is exactly why nothing sizes it, tears down its modes
(T-2732), or answers it.

**What reading `cmd_interact` added.** The timeout branch does not merely fail to
explain itself — it reports `"output": ""` in JSON and a bare "Timeout after Ns
waiting for command to complete". The poll loop had *already collected* the
child's output and computed the diff; the deadline check sits above that code, so
the evidence was gathered, paid for, and then discarded at the one moment it was
needed. Retaining it is the larger half of this fix; naming the cause is the
smaller half.

**Scope boundary, stated.** No DSR/OSC responder is built. Answering these
queries means TermLink starts pretending to be a terminal emulator, which is a
separate design decision with real consequences for what the product claims to
be — the backlog says so, and this task holds that line. The remedy shipped here
is that the operator is told what is happening and why no retry will help.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `interact`'s timeout no longer reports `"output": ""` — `last_diff` retains
      what the poll loop collected and `tail_for_diagnosis` returns a bounded,
      UTF-8-safe tail of it in both output modes
- [x] The timeout names a **cause**: `InteractTimeout::{UnansweredQuery,
      NoOutput, NoMarker}`, rendered into the error line
- [x] Unanswered terminal queries detected for `CSI 6n`, `CSI 14t/16t/18t`,
      DA1 `CSI c`, DA2 `CSI >c`, kitty `CSI ?u`, OSC 10/11/4
      (`TERMINAL_QUERIES`)
- [x] Detection requires the query to be genuinely unanswered —
      `has_meaningful_output_after` ignores whitespace AND neighbouring escape
      sequences, so a child that continued is not flagged
      (`a_query_that_was_answered_and_moved_on_is_not_flagged`)
- [x] Each cause carries an actionable hint; the query hint states the silence is
      by design and no retry will help
      (`query_hint_says_the_silence_is_by_design`)
- [x] Text and `--json` both carry cause + hint + retained output; JSON gains
      `cause` and `hint` keys alongside `bytes_captured`
- [x] `classify_interact_timeout` is pure over the diff — 11 unit tests, no PTY,
      no session, no sleep
- [x] **Scope held: legibility only.** No responder built; the hint points at
      `termlink attach` (where the operator's own terminal replies) rather than
      making TermLink answer
- [x] New tests demonstrated load-bearing by temp-revert: replacing the
      classifier with unconditional `NoMarker` failed **6** tests; restored to a
      byte-identical tree, 46/46 green
- [x] `cargo test -p termlink --bins commands::pty` passes (46 passed)
- [x] `bash scripts/run-guard-layer.sh` stays clean (27/27)

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

# `termlink` is a bin-only package — `--lib` errors with "no library targets".
cargo test -p termlink --bins commands::pty
bash scripts/run-guard-layer.sh

## RCA

**Symptom.** `termlink interact` on a child that queries the terminal blocks to
the deadline and then reports "Timeout after Ns waiting for command to complete"
with `"output": ""`. The operator is told their command was slow, given no
evidence, and left to conclude the fault is in the command or the timeout value.
Neither is true, and no amount of retrying or raising `--timeout` helps.

**Root cause.** Two independent gaps, in the same eight lines. (1) TermLink
drives a PTY with no terminal emulator behind it, so DSR/OSC queries are never
answered — charter-correct, but the child blocks forever and nothing said so.
(2) The deadline check sits *above* the code that computes the output diff, so at
the moment of failure the collected evidence was out of scope and the branch
filled the field with an empty string rather than reaching for it.

**Why structurally allowed.** The timeout branch was written as a *guard* — an
early bail at the top of the loop — and guards are habitually written before the
work they protect. That placement is why it had nothing to report: it ran before
the data existed on every iteration, including the last. Nothing tested it,
because testing it required a live PTY, a real child, and a real wall-clock
deadline; the decision was entangled with the loop, so it was effectively
untestable and stayed untested. That is the same shape as several findings this
session — the logic was fine as far as it was written, and no one could see what
it left out.

**Prevention.** The decision is now a pure function over the diff
(`classify_interact_timeout`), so all three branches are unit-testable without a
PTY, and the loop retains its freshest diff specifically so the guard has
something to say. `has_meaningful_output_after` deliberately does not count
whitespace or neighbouring escape sequences as progress — the two ways the
detector would have quietly missed the case it exists for. Eleven tests, six of
which fail if the classifier is reverted to unconditional `NoMarker`.

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

### 2026-08-15T12:49:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2736-interact-timeout-must-name-the-cause-whe.md
- **Context:** Initial task creation

### 2026-08-15T12:57:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
