---
id: T-2711
name: "revisit-due-scan.sh exits 0 when it cannot find the tasks dir"
description: >
  Direct invocation of revisit-due-scan.sh mis-resolves PROJECT_ROOT in vendored mode and reports success

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T18:42:09Z
last_update: 2026-08-14T18:42:09Z
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

# T-2711: revisit-due-scan.sh exits 0 when it cannot find the tasks dir

## Context

`revisit-due-scan.sh` resolves `PROJECT_ROOT` from an env var when set (the cron
line supplies it) and otherwise falls back to walking up from its own directory
looking for a `.framework.yaml` **or** `FRAMEWORK.md` marker. In vendored /
shared-tooling mode that fallback always lands on the wrong root:

- the consumer project's marker is `.framework.yaml` at the project root
- the **vendored framework** at `.agentic-framework/` carries `FRAMEWORK.md`
- the walk starts inside `.agentic-framework/agents/context/`, so it matches
  `FRAMEWORK.md` at `.agentic-framework/` *first* and never reaches the project

`TASKS_DIR` becomes `.agentic-framework/.tasks/active`, which does not exist, and
the script then does this (`revisit-due-scan.sh:96-100`):

```sh
if [ ! -d "$TASKS_DIR" ]; then
    echo "revisit-due-scan: tasks dir not found at $TASKS_DIR" >&2
    exit 0
fi
```

Observed here:

```
$ bash .agentic-framework/agents/context/revisit-due-scan.sh
revisit-due-scan: tasks dir not found at .../.agentic-framework/.tasks/active
exit=0
```

**Scope — read this before "fixing" it.** Routing through `fw` works correctly:
`fw task revisit-due` sets `PROJECT_ROOT` and returns the right answer (verified
2026-08-14: it correctly reports T-1898 and T-2250 as ripe). So this is *not* the
reason the G-053 revisit mechanism is dark on this host — that is T-1452, whose
cron + handover-banner integration is still `started-work`. Filed separately so
the two are not conflated.

What is wrong here is narrower and still worth fixing: **the script reports
success when it could not look.** Exit 0 on "tasks dir not found" is
indistinguishable from "scanned everything, nothing is ripe", which is the same
class this repo has repeatedly closed elsewhere — guard-layer `ERROR` kept
distinct from `PASS` (T-2684), the `DEGRADED:` clause on the stuck-claims canary
(T-2709), the scope disclaimer on the charter-drift canary (T-2680). A scan that
cannot find its input should exit non-zero (2, the established tooling-error
code), not 0.

**Cross-repo.** The script lives in the vendored `.agentic-framework/` and a
local edit is erased on the next re-vendor (as T-2705 just demonstrated). The
deliverable is therefore an upstream report on `framework:pickup` — the channel
CLAUDE.md designates for exactly this — not an edit under `.agentic-framework/`.

## Acceptance Criteria

### Agent
- [ ] The mis-resolution is reproduced and recorded with the exact command, the resolved `TASKS_DIR`, and the exit code
- [ ] The report distinguishes this defect (silent exit 0) from T-1452 (mechanism never wired), so the upstream reader does not conflate them
- [ ] The report states why `fw task revisit-due` is unaffected — otherwise upstream will fail to reproduce and close it as invalid
- [ ] A concrete remedy is proposed: exit 2 on missing tasks dir, and prefer `.framework.yaml` over `FRAMEWORK.md` when both are found on the walk (nearest-consumer-root wins)
- [ ] The filing is posted to the `framework:pickup` topic and the post is confirmed present (not just sent)
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

# The repro still holds (direct invocation cannot find the tasks dir). This is a
# read of the vendored script's behaviour, not of a local fix.
out=$(bash .agentic-framework/agents/context/revisit-due-scan.sh 2>&1); echo "$out" | grep -q "tasks dir not found"
# ...and the fw-routed path is unaffected, which is the claim the filing rests on.
out=$(.agentic-framework/bin/fw task revisit-due 2>&1); echo "$out" | grep -q "T-2250"
# This task must not have edited the vendored framework (a local fix is erased on re-vendor).
test -z "$(git status --porcelain .agentic-framework)"

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

**Symptom:** `bash .agentic-framework/agents/context/revisit-due-scan.sh` prints
`tasks dir not found` and exits **0**, so any caller that gates on the exit code
reads "scan succeeded, nothing ripe" when in fact nothing was scanned.

**Root cause:** the `PROJECT_ROOT` fallback treats `.framework.yaml` and
`FRAMEWORK.md` as equivalent markers. In vendored mode both exist, and the
framework's own `FRAMEWORK.md` is strictly nearer to the script than the
consumer's `.framework.yaml`, so the walk stops one level too early — inside the
framework rather than inside the project that vendored it. The marker set does
not distinguish "I am the framework repo" from "I am a project using it".

**Why structurally allowed:** the missing-directory branch was written as a
tolerant no-op, presumably so the script is safe to run in a repo with no
`.tasks/`. That is a reasonable intent, but it makes an environment fault
indistinguishable from a clean result, and nothing tests the fallback path —
every real invocation goes through `fw`, which sets `PROJECT_ROOT` and masks it.
The audit's `PROJECT_ROOT` check (T-2648, OBS-097) is scoped to `web/` and `lib/`
Python and so cannot see a shell script under `agents/`.

**Prevention:** exit 2 rather than 0 when the tasks dir is absent, so the
condition is loud; and prefer the consumer marker when both are present. The
durable guard is widening the existing T-2648 `PROJECT_ROOT` audit check to cover
`agents/**/*.sh`, which is upstream's call to make — noted in the filing.

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

### 2026-08-14T18:42:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2711-revisit-due-scansh-exits-0-when-it-canno.md
- **Context:** Initial task creation
