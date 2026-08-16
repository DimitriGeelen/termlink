---
id: T-2777
name: "Task template teaches the SIGPIPE-unsafe verification idiom the guard exists to catch"
description: >
  The default task template's L-387/T-2090 hints prescribe 'out=$(cmd); echo "$out" | grep -q PAT', which T-2775 measured as size-dependent-unsafe (rc=141 above the pipe buffer). Every new task inherits it, and check-verification-pipefail.sh does not scan .tasks/templates/, so the guard cannot see its own upstream source.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T20:32:35Z
last_update: 2026-08-16T20:48:02Z
date_finished: 2026-08-16T20:48:02Z
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

# T-2777: Task template teaches the SIGPIPE-unsafe verification idiom the guard exists to catch

## Context

T-2775 shipped `scripts/check-verification-pipefail.sh` after measuring that a
verification command whose exit status is decided by a pipeline into an
early-exiting consumer returns 141 — the gate fails precisely when the check
succeeds. It also corrected two peer projects (999-AEF L-613, 050-email-archive
PL-161) whose published remediation, `printf '%s' "$out" | grep -q PAT`, is safe
only while `$out` fits the pipe buffer.

`.tasks/templates/default.md` prescribes that same size-dependent idiom, twice:

  - L-387 hint:  `out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"`
  - T-2090 hint: `echo "$out" | grep -q PAT` (explicitly framed as the safe form)

Every task created from the template inherits this advice, which is the most
plausible mechanism behind the 158 ledgered lines T-2775 found. And
`check-verification-pipefail.sh` resolves its scan roots as `.tasks/active`
(+ `completed` under `--include-completed`) — `.tasks/templates/` is not among
them, so the guard is structurally unable to see the file that generates the
violations it catches downstream.

This is the T-2680 shape one layer up: a guard reporting clean over a surface it
never looked at. The tree scanning clean was true *and* the template was seeding
new instances the whole time.

## Acceptance Criteria

### Agent
- [x] `.tasks/templates/default.md` no longer prescribes a pipeline-decided idiom; both the L-387 and T-2090 hint blocks teach the herestring (`grep -q PAT <<< "$out"`) and state why capture alone is insufficient
- [x] The other templates (`inception.md`, `path-c-deep-dive.md`) are checked for the same idiom and fixed if present
- [x] A guard pins the template against regression. NOTE: pointing the existing check at `.tasks/templates/` reports `tasks_with_verification: 0` — the idiom lives in `#` comment lines, which the check correctly skips as non-executable. Scanning templates as if they were tasks would therefore be a guard that can never fire; the control must inspect the template's *prescriptive text* instead
- [x] The check still reports the real tree clean after the template fix (no new unacknowledged firing)
- [x] A fixture asserts the guard is load-bearing: a template that prescribes the unsafe idiom fails it, the fixed template passes
- [x] `bash tests/verification-pipefail-fixtures.sh` passes with the new assertions
- [x] `bash scripts/run-guard-layer.sh` passes

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

# NOTE: every command below uses the herestring form this task exists to establish.
bash scripts/check-task-template-idioms.sh --no-heartbeat
bash tests/task-template-idioms-fixtures.sh
bash scripts/check-verification-pipefail.sh
out=$(bash tests/task-template-idioms-fixtures.sh 2>&1 || true); grep -q "21 passed, 0 failed" <<< "$out"
out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -q "guard layer: PASS" <<< "$out"
# The [REVIEWER] conversion line no longer prescribes a bare pipeline. Absence
# assertion: grep the file directly, negated. NOT `grep -qv`, which returns 0
# whenever ANY line fails to match — i.e. unconditionally, a gate that cannot fail.
! grep -Fq 'bin/fw reviewer T-XXX 2>&1 | grep -q' .tasks/templates/default.md
# NOTE: there is deliberately no absence assertion for the old L-387 string. It is
# still in the template ON PURPOSE, as the labelled counter-example warning readers
# off it. `check-task-template-idioms.sh` above is the real control: it distinguishes
# prescribing the idiom from citing it, which a raw string grep cannot.

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

### 2026-08-16 — a separate guard rather than extending the sibling check
- **Chose:** a new `scripts/check-task-template-idioms.sh` that reads template prose.
- **Why:** pointing `check-verification-pipefail.sh` at `.tasks/templates/` reports
  `tasks_with_verification: 0`. The template's guidance is entirely `#` comments,
  which that check skips *correctly* — a comment is not an executable verification
  command. Extending its scan roots would have produced a guard that can never fire,
  which is worse than no guard because it reads as coverage.
- **Rejected:** teaching the sibling check to read comments — that breaks its
  contract; it would then flag every task that merely *discusses* a risky idiom.

### 2026-08-16 — labelled counter-examples clear, unlabelled ones fire
- **Chose:** a line carrying `UNSAFE` / `DO NOT` / `WRONG` / `NEVER` / `BAD` reads as
  a counter-example; unmarked, it reads as a recommendation and fires.
- **Why:** the fixed template must still be able to *show* the bad form to warn
  against it, so something has to separate demonstration from prescription. A
  same-line label keeps the convention self-documenting.
- **Evidence it works:** it fired on this task's own rewritten hint, which cited
  `cmd | grep -q PATTERN` without a marker. Labelling the line cleared it.

### 2026-08-16 — backtick handling diverges from the sibling check, deliberately
- **Chose:** unwrap backticks and read the content, rather than treating them as
  status-discarding substitutions.
- **Why:** in a shell command a backtick span IS a substitution; in a markdown
  template it is code formatting, and the prescription lives inside it. The real
  pre-T-2777 `[REVIEWER]` line was exactly that. Treating them as substitutions made
  the check silently clear the precise defect it exists to catch — fixture B2 pins it.
- **Accepted cost:** a prescribed *real* backtick substitution would false-fire. The
  idiom this template teaches is `$( )`, and backticks are deprecated shell style.

### 2026-08-16 — `sh -c` wrapping: two wrong assumptions, settled by measurement
- **Chose:** treat `sh -c "<script>"` as ISOLATED from the outer `pipefail` unless the
  inner script itself sets `pipefail`.
- **Why:** measured under the gate's own construct — `bash -c "seq … | grep -q …"`
  returns **0**, not 141. `pipefail` is a shell option, not an environment variable,
  so the inner shell starts without it. An inner `set -o pipefail` (141) and an
  exported `SHELLOPTS` (141) both re-arm it; the first is detected, the second is a
  property of the environment rather than of the line and is named in the scope string.
- **The path here is the point.** Quote-stripping was added to fix a false positive (a
  `|` inside a grep PATTERN). That looked like it would hide `sh -c` pipelines, so it
  was "fixed" to unwrap and flag them — confidently, with a docstring arguing why the
  alternative was an unacceptable blind spot. Both readings were plausible; the second
  was wrong. Only running the five variants settled it.
- **Consequence:** three lines in this tree (T-1673 ×2, T-1885) are that shape and had
  been ledgered as risky by T-2775. They never were. Their acknowledgements were
  **removed** (158 → 155) rather than carried as debt that does not exist — an
  acknowledgement for a non-existent risk overstates the backlog and misleads whoever
  eventually works it.
- **Rejected:** keeping the three entries "to be safe". A ledger is only useful if
  every line in it is real.

### 2026-08-16 — folding the sibling-check fix into this task
- **Chose:** fix the quoting false positive in `check-verification-pipefail.sh` here
  rather than filing a separate task.
- **Why:** it surfaced *from* this task's own Verification block and blocked it — the
  check fired on an absence assertion whose grep pattern contained a pipe. Splitting it
  out would have left this task unable to pass its own gate.
- **Noted:** this does stretch one-task-one-deliverable. Recording the stretch here
  rather than leaving it as folklore.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-16T20:32:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2777-task-template-teaches-the-sigpipe-unsafe.md
- **Context:** Initial task creation

### 2026-08-16T20:48:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
