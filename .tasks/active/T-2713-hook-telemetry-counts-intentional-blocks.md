---
id: T-2713
name: "Hook telemetry counts intentional blocks (exit 2) as hook failures"
description: >
  fw_record_hook_fire treats any non-zero exit as failure, so a blocking enforcement hook trips the T-1626 hook-decay alarm by working correctly

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
created: 2026-08-14T18:54:20Z
last_update: 2026-08-14T19:13:08Z
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

# T-2713: Hook telemetry counts intentional blocks (exit 2) as hook failures

## Context

`fw audit` raised *"Hook threshold: 1 hook(s) failing over threshold (T-1626) —
`FAIL|check-human-ac-tick|20|2|0.1000`"*, whose documented meaning is "this hook
is failing in production, not just on the /tmp probe" and whose auto-registered
concern text suggests the hook "has decayed silently".

It has not. `check-human-ac-tick` returns non-zero on exactly one path
(`check-human-ac-tick.py:289`): the branch that **blocks an agent from ticking a
Human AC** — the entire purpose of the hook (T-1731, closing G2 from T-1729).
Every other path returns 0. The two recorded "failures" are two occasions on
which the guard worked.

The conflation is in `lib/hook-telemetry.sh:28`:

```sh
if [ "$exit_code" != "0" ]; then
    _fw_telemetry_increment "$working_dir/.hook-failure-counter" "$hookname"
fi
```

Any non-zero exit is a failure. But in the Claude Code hook protocol **exit 2 is
the documented "block the tool call" code**, semantically distinct from other
non-zero exits, which mean the hook itself errored. Treating them alike makes a
correctly-enforcing hook statistically indistinguishable from a broken one.

**Why this matters more than a cosmetic miscount.** The consequence is
structural: any blocking hook whose block rate exceeds `FAIL_RATIO` (0.10) is
*guaranteed* to trip this alarm, and the mitigation on offer — `hook-threshold.py
--register` — files a G-concern against a hook that is working perfectly. That
inverts the T-1626 signal it exists to carry (hooks decaying silently, e.g. bare
relative paths under cd-drift), and it teaches the operator that this warning is
noise. That is the same "a guard that fires regardless of system state trains you
to stop reading it" failure closed in T-2709 earlier this session, one layer up
in the telemetry.

`check-human-ac-tick` is the sharpest case because blocking is *all* it does, but
the flaw applies to every Tier-0/Tier-1 gate: `check-tier0`, `check-active-task`,
`budget-gate`, `check-project-boundary` all block by design and all currently sit
at 0 failures only because they have not blocked recently.

**Discovery note.** This did not appear because anything broke today. The two
failures date to `~2026-08-14T01:06` (mtime of `.hook-failure-counter`); routine
task-file edits during the audit remediation carried the fire count to the
`MIN_FIRES=20` floor, at which point 2/20 landed exactly on the 0.10 threshold.
The alarm was latent for hours and surfaced on an unrelated trigger.

**It then cleared itself, which is the strongest evidence of all.** A later audit
in the same session reported `[PASS] Hook threshold: no hooks failing over
threshold`. Nothing about the hook changed. The numerator is fixed at 2 (two
historical blocks); ordinary task-file edits kept firing the hook, so the
denominator grew past 20 and the ratio fell under 0.10. The alarm raised itself
and lowered itself purely as a function of unrelated activity. A signal that
appears and disappears without reference to the health of the thing it watches
carries no information about that thing — and the window in which it *does* fire
is exactly when a guard has been blocking often, i.e. when it is most valuable.

**Cross-repo.** `lib/hook-telemetry.sh` and `lib/hook-threshold.py` are vendored
framework files; a local edit is erased on the next re-vendor. Deliverable is an
upstream report, not an edit under `.agentic-framework/` (same disposition as
T-2711).

## Acceptance Criteria

### Agent
- [ ] The claim is evidenced, not asserted: cite `check-human-ac-tick.py:289` as the only non-zero return and `hook-telemetry.sh:28` as the conflation point
- [ ] The report states that exit 2 is the Claude Code "block" code and that other non-zero codes mean hook error, since the fix depends on that distinction
- [ ] The blast radius is named — every blocking gate (`check-tier0`, `check-active-task`, `budget-gate`, `check-project-boundary`) is subject to the same false alarm, not just this one hook
- [ ] A concrete remedy is proposed: count exit 2 as a `blocks` counter separate from `failures`, and threshold only on genuine failures
- [ ] The report warns against the tempting non-fix — re-baselining via `--register`, which would file a concern against a healthy hook and bury the real T-1626 signal
- [ ] Filed to `framework:pickup` and the post confirmed present
- [ ] No file under `.agentic-framework/` is edited by this task

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

## RCA

**Symptom:** `fw audit` reports `FAIL|check-human-ac-tick|20|2|0.1000` under a
check whose stated meaning is "the hook is failing in production" and whose
auto-registered concern says it "has decayed silently". The hook is healthy; the
two non-zero exits are two successful policy blocks.

**Root cause:** `lib/hook-telemetry.sh:28` classifies by `exit_code != 0`, which
merges two different meanings of non-zero. In the Claude Code hook protocol exit
2 means *the hook deliberately blocked the tool call*; other non-zero codes mean
*the hook itself failed*. The telemetry has one bucket for both, so "enforced a
policy" and "crashed" are the same event.

**Why structurally allowed:** the counter was added (T-1628) to detect hooks
failing silently, and at that time the hooks under suspicion were ones that
errored rather than blocked. Nothing encoded the protocol's distinction between
block and error, and no test asserts that a blocking hook stays `ok` — so the
gap only becomes visible once a blocking hook accumulates enough fires to reach
`MIN_FIRES`, which for a rarely-triggered guard can take weeks.

**Prevention:** split the counter — `blocks` (exit 2) recorded separately from
`failures` (other non-zero), with the threshold reading only `failures`. The
durable guard is a test that fires a hook returning 2 and asserts it is NOT
counted as a failure; without that, the two meanings will re-merge the next time
someone simplifies the condition.

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

### 2026-08-14T18:54:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2713-hook-telemetry-counts-intentional-blocks.md
- **Context:** Initial task creation

### 2026-08-14T19:13:00Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
