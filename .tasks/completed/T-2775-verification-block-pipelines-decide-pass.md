---
id: T-2775
name: "Verification-block pipelines decide pass/fail by SIGPIPE, not by the check (L-613/PL-161 class)"
description: >
  update-task.sh runs each Verification line as 'if ( eval $cmd )' under 'set -euo pipefail', so 'cmd | grep -q PAT' exits 141 when grep matches early and SIGPIPEs the producer — the gate FAILS precisely when the check SUCCEEDS. Measured here: 61 active tasks carry 262 such lines (58% of all active verification commands). Reproduced under the exact gate construct. Cross-project confirmed by AEF L-613 and 050-email-archive PL-161.

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
created: 2026-08-16T19:50:37Z
last_update: 2026-08-16T20:07:23Z
date_finished: 2026-08-16T20:07:23Z
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

# T-2775: Verification-block pipelines decide pass/fail by SIGPIPE, not by the check (L-613/PL-161 class)

## Context

`update-task.sh` runs every `## Verification` line as `if ( eval "$cmd" ); then` under
`set -euo pipefail` (`:14`, `:1066`). pipefail survives into the condition, so a pipeline's
status is decided by its worst stage — including a producer that was SIGPIPEd because the
consumer exited early. `cargo test 2>&1 | grep -q "test result: ok"` therefore returns
**141**: the gate fails precisely when the check succeeds, and the earlier the match, the
more reliably it fails.

Arrived as two independent peer filings — AEF `L-613` (T-3039/T-3045) and
050-email-archive `PL-161` (T-1948) — on `channel:learnings`. Both were confirmed here
against the real gate construct before any work started.

The local finding is sharper than either report. **PL-080 recorded this class on
2026-04-25**, with the explicit instruction "Avoid bare `| grep -q` in verification
commands." Measured 2026-08-16: 1490 such lines across 802 tasks, 262 of them in 61 active
tasks — 58% of all active verification commands. The learning was accurate, named the exact
failure mode, and prevented nothing over four months. That is the evidence for a structural
check rather than more documentation.

## Acceptance Criteria

### Agent
- [x] `scripts/check-verification-pipefail.sh` exists, carries the
      `# guard-layer: source` marker, and flags `## Verification` lines whose
      exit status is decided by a pipeline under `pipefail` rather than by the
      check itself
- [x] The detector clears the genuinely-safe idioms (pipeline inside `$(...)`
      and backticks per PL-080; herestring) and fires on an arbitrary external
      producer — **and the measurement corrected the premise**: L-613's
      `printf '%s' "$out" | grep -q` is size-dependent (rc=141 above the pipe
      buffer), so it fires under its own `bounded-producer-pipeline` reason
      rather than being treated as the fix
- [x] A fixture suite `tests/verification-pipefail-fixtures.sh` covers both
      firing shapes, the safe idioms, the allowlist, and a control proving the
      check does not fire on the real tree — 26 assertions, 4 of which RUN the
      idioms under the gate's own construct so the rule rests on measurement
- [x] The already-exposed lines are enumerated in a git-tracked allowlist
      (`.context/checks/verification-pipefail-allowlist`, T-2681 convention) so
      they are counted and reported but do not fire, while a NEW occurrence
      fires immediately — 158 lines across 55 active tasks, keyed by a hash of
      the normalized command so a reword re-fires
- [x] Every output path (including the clean one) states the measured census and
      the scope limit, so a green can never be read as "all verification blocks
      are sound" (T-2680)
- [x] Exit codes follow the guard-layer contract: 0 clean, 1 firing, 2 tooling;
      `--json` emits `{ok, firing[], acknowledged[], census, scope}`
- [x] `bash scripts/run-guard-layer.sh` passes with the new member included —
      43/43 (was 41/41; check + fixture suite both auto-discovered)
- [x] The check is load-bearing: removing the `T-1296` allowlist line re-fires
      exactly that task's line and nothing else; restoring it returns rc=0

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

## RCA

**Symptom:** A `## Verification` line of the form `cmd | grep -q PAT` reports exit 141,
failing the P-011 completion gate, even though the pattern was found. The failure is
*inversely* correlated with the check being satisfied: the earlier `grep -q` matches, the
more certainly the producer is still writing when the pipe closes.

**Root cause:** `update-task.sh:14` sets `set -euo pipefail` and `:1066` evaluates each
line as `if ( eval "$cmd" ); then`. pipefail survives into the condition, so the pipeline
adopts the status of its worst stage. `grep -q` exits 0 on first match and closes the read
end; the producer receives SIGPIPE and exits 141; pipefail surfaces the 141 rather than
grep's 0. The check's own verdict is discarded in favour of a signal about process
teardown.

**Why structurally allowed:** two layers.

*Detection.* Nothing read verification blocks as code. The guard layer had nine members,
every one aimed at product source (`crates/**`, `scripts/**`, docs, release config); the
gate that decides whether product source may be called done was itself unexamined. So the
one place the framework asserts "this work is verified" was the one place with no guard.

*Knowledge.* This was not an unknown. PL-080 recorded it on 2026-04-25 with a correct
diagnosis and an explicit instruction. It was written into `learnings.yaml`, surfaced by
`fw work-on` as related knowledge, and accumulated 1490 violations anyway. A learning is
read at most once, by whoever happens to run the command that surfaces it; a check runs
every time. Treating documentation as a control is what allowed a known defect to spread
for four months.

**Prevention:** `scripts/check-verification-pipefail.sh`, guard-layer member #10, fires on
any verification line whose exit status is pipeline-decided, with the 158 pre-existing
lines ledgered by command-hash so a reword re-fires and a new occurrence fires
immediately. 26 fixtures, four of which execute the idioms under the gate's own construct
so the detection rule rests on measurement rather than on belief about bash — which is how
the measurement caught that the peer projects' prescribed remediation is itself
size-dependent, a defect that would otherwise have been adopted here verbatim.

**Not prevented, deliberately:** the 158 existing lines still misreport. Retrofit is a
human decision (§ Decisions). Until then the residual risk is a false FAIL that pushes an
operator toward `--force` — which skips the entire block, so the second-order harm exceeds
the first-order one.

## Decisions

**A check, not another learning.** PL-080 (2026-04-25) already named this class and said
"avoid bare `| grep -q`". Four months later there were 1490 instances. The evidence that
documentation is not a control here is the four months, not an argument — so the
deliverable is a guard-layer member rather than a louder learning.

**The measurement changed the fix.** The remediation both peer projects published
(`out=$(cmd); printf '%s' "$out" | grep -q PAT`) was going to be adopted verbatim until it
was actually run: it returns 141 once the captured value exceeds the pipe buffer, because
capturing bounds the DATA but does not remove the PRODUCER. The two idioms that hold at
any size are the pipeline inside `$(...)` and a herestring. The correction was posted back
to `framework:pickup` (offset 1) since both projects are mid-retrofit on the weaker form.

**Ledger, not retrofit.** The 158 existing lines are acknowledged rather than rewritten.
Both peer projects classed the retrofit as human-authority, and this is 55 task files
this task does not own, several `owner: human`. The failure direction is a false FAIL, not
a false PASS, so leaving them ledgered is conservative — the risk they carry is that a
spurious failure pushes someone toward `--force`, which is precisely why the retrofit
should be a deliberate decision rather than a side effect of adding the check.

**Verification lines here use the safe idiom on purpose** — the check would otherwise fire
on its own task, and a guard that cannot pass its own rule is not a rule.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# NOTE: these deliberately avoid `cmd | grep -q` — see ## Decisions.
bash scripts/check-verification-pipefail.sh
bash tests/verification-pipefail-fixtures.sh
test -f .context/checks/verification-pipefail-allowlist
grep -q "guard-layer: source" scripts/check-verification-pipefail.sh
out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -q "guard layer: PASS" <<< "$out"
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

### 2026-08-16T19:50:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2775-verification-block-pipelines-decide-pass.md
- **Context:** Initial task creation

### 2026-08-16T19:56:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-16T20:07:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
