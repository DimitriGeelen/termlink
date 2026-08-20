---
id: T-2813
renumbered_from: T-2687  # T-2823 cross-branch collision
name: "pickup dedup hash fails open to constant sha256 and pickup_create_inception
  mints empty-name tasks"
description: >
  pickup dedup hash fails open to constant sha256 and pickup_create_inception mints
  empty-name tasks

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/events.rs, crates/termlink-mcp/src/tools.rs, scripts/check-framework-tracking-drift.sh, tests/framework-tracking-drift-fixtures.sh, tests/pickup-failopen-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-19T23:32:30Z
last_update: 2026-08-20T18:12:06Z
date_finished: 2026-08-20T18:12:06Z
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
  - ts: '2026-08-20T15:20:37Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 4
      F-RECALL: 0
      F-ORCH: 4
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=4 
      (body:cross-machine); F-RECALL=0 (no-signal); F-ORCH=4 
      (body:rubric-routable)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2813: pickup dedup hash fails open to constant sha256 and pickup_create_inception mints empty-name tasks

## Context

`lib/pickup.sh` has two fail-open paths on the cross-project pickup ingest rail.
`pickup_dedup_hash` extracts `type` / `payload.summary` / `source.project` with greps
whose failure is swallowed (`2>/dev/null || true`), then hashes the joined string
unconditionally — so an unreadable/absent envelope yields the **constant** digest
`sha256("||") = 565d240f5343e625ae579a4d45a770f1f02c6368b5ed4d06da4fbe6f47c28866`.
That constant appears in `.context/pickup/dedup.log` 5× (2026-06-23, 2026-08-14 ×3,
2026-08-15) under four different envelope filenames — the dedup ledger carries a poison
entry that collides across unrelated envelopes, so any future envelope that also fails
to read is silently classified a duplicate of them and dropped.

Symmetrically `pickup_create_inception` builds `"Pickup: ${summary} (from
${source_project})"` with no emptiness guard, so the same extraction failure mints a
garbage task instead of refusing — four exist in the operator's checkout today
(T-2683/T-2684/T-2685/T-2686, all named `Pickup:  (from )`), each scored BVP 70 /
hv-hc by the estimator, so they outrank real work in `fw bvp --quadrant hv-hc`.

Both are Directive #2 (no silent failures) violations on a governance-critical rail.

## Acceptance Criteria

### Agent
- [x] `pickup_dedup_hash` returns non-zero and prints a diagnostic to stderr when the
      envelope is unreadable OR when all three hash inputs extract empty, instead of
      emitting the constant `sha256("||")` digest
- [x] `pickup_record_dedup` refuses to append a ledger line when the hash cannot be
      computed (no new poison entries), and says so on stderr
- [x] `pickup_dedup_check` treats an uncomputable hash as "not a duplicate" (fail-safe:
      never silently drop an envelope) while printing a warning
- [x] `pickup_create_inception` refuses (non-zero, stderr diagnostic naming the file and
      the missing field) when `payload.summary` or `source.project` extract empty —
      no task is created
- [x] `pickup_process_one` propagates that refusal instead of continuing to
      `pickup_record_dedup` / `mv`-to-processed, so the envelope stays inspectable
- [x] A regression test script exercises the refusal paths against fixture envelopes,
      asserts the happy path still processes, and passes
- [x] `bash -n .agentic-framework/lib/pickup.sh` passes

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

bash -n .agentic-framework/lib/pickup.sh
bash tests/pickup-failopen-fixtures.sh

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

**Symptom:** Four tasks named `Pickup:  (from )` (T-2683–T-2686) appeared in
`.tasks/active/`, created by the pickup cron at 07:00 / 11:45 / 15:00 on 2026-08-14 and
15:15 on 2026-08-15. Each carries `description: Auto-created from pickup envelope.
Source: . Type: .` — every interpolated field empty. The BVP estimator scores them 70 /
hv-hc, placing four content-free tasks in the top of the HV/HC prioritisation table.

**Root cause:** Both `pickup_dedup_hash` and `pickup_create_inception` extract envelope
fields with `$({ grep "^  summary:" "$file" 2>/dev/null || true; } | ...)`. The
`2>/dev/null || true` guard converts *every* failure — file absent, file unreadable,
field absent — into the empty string, and neither function checks the result before
using it. `pickup_dedup_hash` then hashes `"${type}|${summary}|${project}"` = `"||"`,
whose digest `565d240f5343e625ae579a4d45a770f1f02c6368b5ed4d06da4fbe6f47c28866` is a
fixed constant; `pickup_create_inception` interpolates the empties straight into the
task name and calls `fw task create` regardless.

**Why structurally allowed:** `pickup_validate_envelope` checks only that the field
*keys* are greppable (`grep -q "^  summary:"`) on the file as it stood at validation
time. Nothing re-checks that the *values* actually extracted non-empty at the two later
points of use, and the two consumers sit after a `pickup_create_inception` call that can
change the file's state. The constant digest then poisons `.context/pickup/dedup.log`:
because dedup is a pure hash match, every subsequent envelope that also fails to extract
collides with those rows and is silently dropped as a "duplicate" — a fail-open
detection path degrading into a fail-closed *data-loss* path, with no operator signal in
either direction.

**Prevention:** `pickup_dedup_hash` now returns non-zero on unreadable input or
all-empty extraction rather than emitting a hashable constant; `pickup_record_dedup`
refuses to write a ledger row it cannot compute; `pickup_dedup_check` degrades to
"not a duplicate" (never silently dropping an envelope) and warns;
`pickup_create_inception` refuses to mint a task with an empty summary or project.
`tests/pickup-failopen-fixtures.sh` pins all four refusals plus the happy path, so a
future re-introduction of a bare `|| true` on these paths fails the suite.

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

### 2026-08-19T23:32:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2813-pickup-dedup-hash-fails-open-to-constant.md
- **Context:** Initial task creation

### 2026-08-20 — verified and closed; the fix is vendored, so durability was the missing half [agent]

Picked up as the top HV/LC task (BVP 99). Every Agent AC was already ticked and the work was
genuinely done — `bash tests/pickup-failopen-fixtures.sh` runs **12/12** against the real
`lib/pickup.sh`, exercising each refusal path and each happy path. Re-verified today, not
taken on trust.

It had been sitting `started-work` since 2026-08-19 for a mechanical reason, now understood:
`fw task update --status work-completed` was **failing on every build task in a worktree**
(unguarded source of an untracked file — diagnosed and fixed under T-2806). This task was one
of 24 in that state.

**What the verification pass found, which is why this did not just close.** `lib/pickup.sh` is
**vendored**, and this task records no upstream filing. So the fix was one `fw update` from
deletion — and a re-vendor is being proposed on another branch right now (T-2705). Closing it
as "complete" would have been true of the code and false of the outcome.

Handled under **T-2812**: the fix is registered in `.vendor-divergence.yaml` as
`status: filed-upstream` and reported to 999-AEF at `framework:pickup` **offset 27**, with the
fail-safe directions spelled out — the WRITE paths refuse, the READ path admits, because a
dedup check that failed closed would drop mail. `scripts/check-vendor-divergence.sh` now fires
if a local change to vendored code goes unregistered.

Nothing about the fix itself changed. What changed is that it will survive the next re-vendor,
or be noticed if it does not.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-17f04e2c
- **Timestamp:** 2026-08-20T18:12:07Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-20T18:12:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
