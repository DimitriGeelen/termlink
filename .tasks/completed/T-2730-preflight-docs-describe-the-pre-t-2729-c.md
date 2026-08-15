---
id: T-2730
name: "preflight docs describe the pre-T-2729 Check 1 and hand out a /tmp-only diagnostic"
description: >
  preflight docs describe the pre-T-2729 Check 1 and hand out a /tmp-only diagnostic

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
created: 2026-08-15T09:06:47Z
last_update: 2026-08-15T09:10:46Z
date_finished: 2026-08-15T09:10:46Z
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

# T-2730: preflight docs describe the pre-T-2729 Check 1 and hand out a /tmp-only diagnostic

## Context

Follow-up to T-2729, which taught Check 1 to resolve `runtime_dir` by the
binary's own four-step order and to classify volatility by mount type rather
than path spelling. The operator-facing docs still describe the pre-fix
behaviour. Two of them are actively misleading rather than merely stale:

- `.claude/commands/preflight.md:37` — the Check 1 table row still reads
  "`TERMLINK_RUNTIME_DIR` NOT on /tmp".
- `.claude/commands/preflight.md:118` — the diagnostic handed to an operator
  whose Check 1 failed is `mount | grep ' /tmp '` + `cat
  /usr/lib/tmpfiles.d/tmp.conf`. After T-2729 the check fires on tmpfs
  *anywhere*, so the most likely failure is now `/run/user/<uid>` — against
  which this diagnostic prints nothing and reads as "no problem found". A
  diagnostic that comes up empty on the failure it is dispatched for is worse
  than none: it argues the check was wrong.
- `docs/operations/substrate-getting-started.md:69` — sample PASS output no
  longer matches what the script prints (the message now names the filesystem).

Same class as the defect T-2729 fixed, one layer out: guidance whose correctness
rests on an assumption about the system that no longer holds.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `.claude/commands/preflight.md` Check 1 row describes resolution by the
      binary's four-step order and volatility by filesystem, not by `/tmp`
- [x] The Check-1-failed diagnostic covers the `/run/user` (tmpfs) case it is
      now most likely to be dispatched for — not only the `/tmp` mechanisms
- [x] `docs/operations/substrate-getting-started.md` sample output matches what
      the script actually prints today
- [x] No doc under `docs/` or `.claude/` still asserts Check 1 tests "not on
      /tmp" as its criterion (verified by grep, comments excluded)

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

# No doc may still state the pre-T-2729 criterion. Capture-then-grep (L-387);
# `|| true` because grep -r exits 1 on no-match, which is the passing case.
hits=$(grep -rn "not on /tmp" docs/ .claude/ 2>/dev/null || true); [ -z "$hits" ]

# T-2726 removed this string from the script; the getting-started sample still
# showed it. Same drift, caught in passing — keep it pinned.
hits=$(grep -rn "field — fresh binary" docs/ .claude/ scripts/ 2>/dev/null || true); [ -z "$hits" ]

# The docs the guard layer owns must still be consistent.
bash scripts/run-guard-layer.sh

## RCA

**Symptom.** After T-2729 widened Check 1, three operator-facing docs still
described the narrower pre-fix behaviour, and one of them handed an operator a
diagnostic (`mount | grep ' /tmp '`) that prints nothing for the case the check
now most often fires on (`/run/user/<uid>`).

**Root cause.** The check's description is duplicated across the script header,
`CLAUDE.md`, `.claude/commands/preflight.md`, `docs/operations/substrate-tunables.md`
and a worked example in `docs/operations/substrate-getting-started.md`. Nothing
links them, so a behaviour change updates whichever copies the author remembers.

**Why structurally allowed.** `check-preflight-doc-set-drift.sh` exists and
passed throughout — it verifies the doc *set* is present, not that any doc's
claims match the script's behaviour. A guard reporting green over a property it
does not test is the same shape as the defect T-2729 fixed and as T-2680's
canary over-reporting its scope. This one was found only by grepping after the
fact.

**Prevention.** The two `## Verification` greps pin the specific stale claims so
their return fails completion. That is narrower than the general problem — a
check that doc prose matches script behaviour is not something grep can do — so
the honest statement is that this is a *tripwire for the known strings*, not
coverage of the class. Filing the general "assert doc claims against observed
script output" check would be a separate, larger piece of work; noting it here
rather than implying it is done.

**Evidence of the wider class.** While fixing this, the same sweep found
`docs/operations/substrate-getting-started.md:73` still advertising T-2139's
`rate_buckets_evicted_total` probe — a string T-2726 deleted from the script
earlier in this same session. Two doc-drift instances from two consecutive
preflight fixes is the pattern, not the exception.

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

### 2026-08-15T09:06:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2730-preflight-docs-describe-the-pre-t-2729-c.md
- **Context:** Initial task creation

### 2026-08-15T09:10:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
