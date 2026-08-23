---
id: T-2720
name: "Fleet version floors not bumped when T-2533 shipped — canary green on known-bad binaries"
description: >
  check-fleet-binary-freshness.sh reports healthy while every reachable hub serves a binary 151-192 commits below the T-2533 silent-data-loss fix, because fleet-version-floors.conf was last bumped 2026-07-27 and T-2533 shipped a hub-side rail on 2026-08-08 without bumping it

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
created: 2026-08-15T05:13:21Z
last_update: 2026-08-15T06:14:48Z
date_finished: 2026-08-15T06:14:48Z
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

# T-2720: Fleet version floors not bumped when T-2533 shipped — canary green on known-bad binaries

## Context

Split out of T-2719 (G-019: fix the symptom, then close the framework blindness).

T-2533 (`ac859d321`, 2026-08-08, **v0.11.871**) fixed a **silent data-loss** bug on
the charter-core durable-messages verb: `unread`/`ack` used `count-1` as the latest
offset, so on a swept topic a reader could be told 0 unread while messages sat
unseen. The fix is hub-side — the hub must serve `latest_offset`, and pre-fix hubs
make the client take a documented `count-1` fallback.

**It was never deployed, and the guard that exists to catch exactly this said
healthy every day for a week.** Measured 2026-08-15:

```
$ bash scripts/check-fleet-binary-freshness.sh
fleet-binary-freshness: healthy — all floored reachable hubs at/above floor
  ✓ local-test:              served=0.11.720 >= floor=0.11.679
  ✓ workstation-107-public:  served=0.11.720 >= floor=0.11.679
  ✓ ring20-management:       served=0.11.679 >= floor=0.11.679
```

Every reachable hub is **below 0.11.871** — by 151, 151, and 192 commits.

**The canary is not broken.** It is doing precisely what it was configured to do.
`.context/cron/fleet-version-floors.conf` was last bumped 2026-07-27 (T-2465/T-2466)
to 0.11.679, and its own header carries the convention that was not followed:

> **BUMP THE FLOOR WHEN HUB-SIDE RAILS SHIP:** that is the operator's declaration
> that "shipped" must mean "capability-live" — the canary then names lagging hubs
> daily until they are restarted onto the new binary.

So the floor is a **manually-maintained assertion** about what "current" means, and
T-2533 shipped a hub-side rail without updating it. The canary's green is therefore
not evidence of freshness; it is evidence that nobody has re-declared the floor
since July. That is the same defect class as this session's other findings — a guard
whose verdict rests on an assumption that has quietly stopped holding (T-2680,
T-2709, T-2714, T-2715, T-2718), and the same class the canary itself was built for
(G-069, where .122 served a pre-arc-004 feature set for 13 days while recorded
closed=shipped).

**Cost:** a peer session (999-AEF) spent a debugging cycle on the resulting symptom
and filed it as a new bug (T-2719). It had been fixed 7 days earlier.

**Note on scope.** Bumping the floor is a repo edit and is in scope here. Actually
*upgrading* the hosts is an operator action (binary deploy + `systemctl restart`
through the unit per G-070) and is not. This task deliberately makes the canary
start firing — turning a false green into a true red is the deliverable.

## Acceptance Criteria

### Agent
- [x] `fleet-version-floors.conf` floors for our-lineage hubs are raised to >= 0.11.871 (the T-2533 fix), with a comment citing the task, the commit, and why that specific version
- [x] `ring20-dashboard` stays exempt — no upgrade foothold (T-2467); this task does not change exemptions
- [x] `bash scripts/check-fleet-binary-freshness.sh` now FIRES (exit 1) and names each lagging hub with its served version — verified by running it
- [x] The task records the structural finding: a version floor is a hand-maintained assertion, so a canary gated on it reports freshness only as recently as the last manual bump *(§Prevention)*
- [x] A durable prevention is proposed for the gap: something that couples "a hub-side rail shipped" to "the floor moved", since the convention alone has now failed once with a data-loss fix *(§Prevention, with the rejected alternative and why)*
- [x] Filed as an upstream record under `.context/upstream/` — it belongs in vendored task-lifecycle tooling, not this repo *(U-007, severity high)*

## Prevention

### The structural finding

`check-fleet-binary-freshness.sh` compares `served >= declared_floor`. It never
checks `declared_floor >= reality`. **A guard that measures the field against a
hand-written constant is only as fresh as the constant**, and nothing was watching
the constant. Its green means "no hub has regressed below what someone asserted in
July" — which is a much weaker claim than the one an operator reads off it.

### Rejected: a "floor vs HEAD lag" arm

The tempting fix is a second detector firing when a declared floor falls more than
N commits behind the repo's `VERSION`. **Do not build this.** On any actively
developed repo HEAD advances daily while deployed floors move only on a deploy, so
the arm goes red permanently within days of every upgrade and stays red. That
converts a one-bit canary into standing noise — the precise failure this session has
now catalogued three times (T-2709's monotonic latch, T-2680's over-broad green,
and T-2719's permanently-alarming unread count). A guard that is always red teaches
its operator to stop reading it, which is strictly worse than the gap it closes.

The lag is also not the thing we care about. 400 commits of CLI-only work above the
floor is harmless; the 192 commits here mattered only because **one** of them was a
hub-serving data-loss fix.

### Proposed: couple the floor bump to the shipping task

The precise trigger is "a task that changed hub-serving code reached
`work-completed`". That fires **once per shipping task** — never idles red, and
lands on the person who has the context to answer it. Shape:

> At `--status work-completed`, if the task's commits touch hub-serving crates
> (`crates/termlink-hub/`, `crates/termlink-bus/`, `crates/termlink-protocol/`),
> require either a bump to `fleet-version-floors.conf` in the same task, or an
> explicit one-line declaration that the change carries no hub-side rail.

This is the same instrument as the existing P-011 verification gate and the T-2480
shipped==live probe — mechanical, at closure time, not a background poller. It is
**task-lifecycle tooling under `.agentic-framework/`**, which is vendored, so the
deliverable is an upstream record rather than a local edit.

Note the honest limit: this catches the omission for *future* rails. It cannot
answer "which already-shipped commits above the current floor were hub-side" — that
backfill is a one-off review, not something to automate.

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

### 2026-08-15T05:13:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2720-fleet-version-floors-not-bumped-when-t-2.md
- **Context:** Initial task creation

### 2026-08-15T05:14:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-15T06:14:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
