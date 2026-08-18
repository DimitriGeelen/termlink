---
id: T-2791
name: "Permission-denied runtime dir renders as a confident empty inventory (OBS-302 mechanism)"
description: >
  Permission-denied runtime dir renders as a confident empty inventory (OBS-302 mechanism)

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
created: 2026-08-18T12:57:50Z
last_update: 2026-08-18T13:01:33Z
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

# T-2791: Permission-denied runtime dir renders as a confident empty inventory (OBS-302 mechanism)

## Context

`discovery.rs:81-87` filters candidate session dirs with `.filter(|d| d.is_dir())`,
documented as "avoids noisy read_dir errors". `Path::is_dir()` returns `false` on
`EACCES`, so a runtime dir the caller cannot read is **indistinguishable from one that
does not exist**. For a non-root uid pointed at a `0700` root-owned
`TERMLINK_RUNTIME_DIR`, the chain is:

1. `all_runtime_dirs()` returns exactly that dir (`discovery.rs:44-47` exclusive-override)
2. `all_sessions_dirs()` drops it — `is_dir()` false under EACCES
3. zero session dirs → zero sessions → zero probes → `total_probed == 0`, `skipped == 0`
4. `events.rs:1220-1230`: empty + `skipped == 0` → prints bare `No event topics found.`,
   returns `Ok(())`, exit 0

The `--json` surface is *more* confidently wrong than the text one: T-2624's
partial-inventory fields count PROBE failures, not DISCOVERY failures, so the caller
receives `{"ok":true,"total_topics":0,"sessions_probed":0,"sessions_skipped":0}` — a
positive assertion that the inventory is complete and empty.

Directive #2: a plausible wrong answer is worse than an error. Reported independently by
999-AEF as OBS-302 (agent-chat-arc offset 102); this task establishes the mechanism and
the fix. Reply with source citations: `docs/reports/T-2791-reply-to-999-aef-defect-b.md`.

## Acceptance Criteria

### Agent
- [x] A regression test pins the defect: an unreadable candidate runtime dir must NOT be
      reported as an absent one. Test fails against current `all_sessions_dirs()`.
      → 6 new tests in `discovery.rs`. The two load-bearing ones are
      `permission_denied_at_stat_is_unreadable_not_skipped` and
      `permission_denied_at_read_dir_is_unreadable`; both assert `Unreadable`, which the
      pre-fix `.filter(|d| d.is_dir())` could not express at all (it had one outcome:
      dropped). Written against a PURE classifier `classify_candidate` rather than a
      chmod fixture, because the test process runs as root here and root bypasses the
      exact permission check under test — a chmod-based test would have passed while
      never exercising the branch. Same pattern as `decide_unix_peer` (T-2448,
      `server.rs:1929`). 13/13 `discovery` tests pass.
- [x] `all_sessions_dirs()` distinguishes "absent" from "present but unreadable" —
      `ENOENT` stays silently filtered (the legitimate intent of the original filter),
      `EACCES` is surfaced to the caller rather than dropped.
      → New `scan_sessions_dirs() -> SessionsDirScan { usable, unreadable }`.
      `all_sessions_dirs()` keeps its signature as `scan().usable`, so the three existing
      callers (`supervisor.rs:38`, `manager.rs:199`, and the CLI) are unaffected —
      pinned by the pre-existing `all_sessions_dirs_filters_nonexistent`, still green.
      `read_dir` is consulted separately from `metadata` because traversing (`--x`) and
      listing (`r--`) are distinct permissions.
- [x] `termlink topics` (text) no longer prints a bare `No event topics found.` when a
      candidate runtime dir was skipped for permissions; it names the unreadable path.
      → Live-proven as uid 1000 against a root-owned `0700` dir: names the path, states
      "this is not a complete answer", and gives the uid + remedy. Control (readable
      empty dir, same uid) still prints exactly `No active sessions.` — unchanged.
- [x] `termlink topics --json` no longer asserts a complete inventory in that case — the
      unreadable-dir count is a distinct field from `sessions_skipped` (which counts probe
      failures) so a consumer can tell the two apart.
      → New binary, unreadable dir: `{"dirs_unreadable":1,"inventory_complete":false,
      "unreadable_dirs":["/tmp/t2791probe/sessions"],...}`. Same invocation on the OLD
      binary: `{"ok":true,"sessions":[],"total_topics":0}` — byte-identical to the
      readable-empty control, which is the defect demonstrated side by side.
- [x] Reply to 999-AEF posted on agent-chat-arc confirming Defect B from source
      (`server.rs:743/766/813`) and correcting the two points where their advisory is
      version-dated or would misdirect an implementer.
      → agent-chat-arc offset 105 (`{"delivered":{"offset":105}}`), full text at
      `docs/reports/T-2791-reply-to-999-aef-defect-b.md`. Frontier acked to 105 at
      offset 106 so the topic is meaningful to sweep next time.

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

out=$(cargo test -p termlink-session --lib discovery 2>&1 || true); grep -q "13 passed; 0 failed" <<< "$out"
out=$(cargo test -p termlink-session --lib discovery 2>&1 || true); grep -q "permission_denied_at_stat_is_unreadable_not_skipped ... ok" <<< "$out"
out=$(cargo test -p termlink-session --lib discovery 2>&1 || true); grep -q "permission_denied_at_read_dir_is_unreadable ... ok" <<< "$out"
out=$(cargo test -p termlink-session --lib discovery 2>&1 || true); grep -q "absent_dir_is_still_skipped_silently ... ok" <<< "$out"
out=$(cargo test -p termlink-session --lib discovery 2>&1 || true); grep -q "all_sessions_dirs_filters_nonexistent ... ok" <<< "$out"
out=$(cat crates/termlink-session/src/discovery.rs 2>&1 || true); grep -q "pub fn scan_sessions_dirs" <<< "$out"
out=$(cat crates/termlink-session/src/discovery.rs 2>&1 || true); grep -q "PermissionDenied) => CandidateOutcome::Unreadable" <<< "$out"
out=$(cat crates/termlink-cli/src/commands/events.rs 2>&1 || true); grep -q "inventory_complete" <<< "$out"
out=$(cat crates/termlink-cli/src/commands/events.rs 2>&1 || true); grep -q "dirs_unreadable" <<< "$out"
test -f docs/reports/T-2791-reply-to-999-aef-defect-b.md
out=$(cat docs/reports/T-2791-reply-to-999-aef-defect-b.md 2>&1 || true); grep -q "server.rs:813" <<< "$out"
out=$(cat docs/reports/T-2791-reply-to-999-aef-defect-b.md 2>&1 || true); grep -q "discovery.rs:81-87" <<< "$out"

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
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## RCA

**Symptom:** A non-root process pointed at a `0700` root-owned `TERMLINK_RUNTIME_DIR`
runs `termlink topics` and is told `No event topics found.` (exit 0), or in JSON
`{"ok":true,"sessions":[],"total_topics":0}`. Both are indistinguishable from the answer
given for a genuinely empty, fully-readable runtime dir. Reported independently by
999-AEF as OBS-302, where it caused a peer project (0503-codex) to build an entire
diagnosis on filesystem reads that were quietly returning nothing.

**Root cause:** `discovery.rs:85`, `.filter(|d| d.is_dir())`. `Path::is_dir()` is
documented to return `false` when the underlying `stat` fails **for any reason**, which
includes `EACCES`. So "this directory does not exist" and "I am not allowed to look at
this directory" produced the same value, and the second was discarded as the first. Two
further layers repeated the collapse — `manager.rs:230` gates on `!sessions_dir.exists()`
(also false under EACCES) and `manager.rs:217` swallows `read_dir` errors at `debug`
level — so even had discovery reported honestly, the result would have been re-silenced.

**Why structurally allowed:** three reasons, and the third is the one that matters.

1. The filter's stated intent was noise suppression — the doc comment says "avoids noisy
   read_dir errors" — and that intent is *legitimate* for `ENOENT` (an uncreated
   `/tmp/termlink-$UID` is the common case). The bug is that one predicate was used for
   two conditions, and no test distinguished them.
2. T-2624 had already added partial-inventory fields to this exact command, so the
   "our answer might be incomplete" concept existed — but it counts **probe** failures,
   sessions found and then unreachable. A session never discovered is invisible to it.
   The JSON surface therefore asserted `sessions_skipped: 0` — actively claiming
   completeness — making the machine-readable output *more* confidently wrong than the
   human one.
3. **The guard layer could not see this class.** `check-silent-exit.sh` (T-2666) exists
   for precisely this directive, but it detects a **non-zero exit that prints nothing**.
   This is the inverse and strictly worse shape: a **zero exit that prints a positive,
   plausible, wrong claim**. Nothing in the layer asks "can this success path be reached
   by a failed read?" Recorded as the follow-up below rather than claimed as fixed here.

**Prevention:** the fix itself is not prevention — `classify_candidate` is. It is a pure
function with the `EACCES` branch pinned by name, so re-collapsing the two conditions now
fails a test rather than passing silently. It is deliberately pure because the test
process runs as **root**, and root bypasses the permission check under test: a
chmod-based fixture would have gone green while never once exercising the branch, which
is the T-2683 "a guard nothing executes" shape reproduced inside the regression test for
it. The live proof was therefore run out-of-band by dropping to uid 1000, old binary
against new, same directory.

Structural follow-up (not closed by this task): the guard layer has no member for
"zero-exit success path reachable from a failed read". Filed separately rather than
folded in — one bug, one task. (workflow_type=build with bug-tag, OR title matches
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

### 2026-08-18T12:57:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2791-permission-denied-runtime-dir-renders-as.md
- **Context:** Initial task creation
