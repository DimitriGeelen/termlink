---
id: T-2792
name: "Guard layer has no member for zero-exit success paths reachable from a failed read"
description: >
  Guard layer has no member for zero-exit success paths reachable from a failed read

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
created: 2026-08-18T13:11:25Z
last_update: 2026-08-18T13:11:25Z
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

# T-2792: Guard layer has no member for zero-exit success paths reachable from a failed read

## Context

T-2791 fixed a defect the guard layer was structurally unable to see.
`check-silent-exit.sh` (T-2666) exists for exactly this Directive — but it detects a
**non-zero exit that prints nothing**. T-2791 was the inverse and strictly worse shape:
a **zero exit that printed a positive, plausible, wrong claim**, because
`Path::is_dir()` returns `false` when the underlying `stat` fails, so `EACCES` was
indistinguishable from `ENOENT` and a permission-denied read rendered as a complete
empty inventory.

The std API is documented as doing this. From `std::path::Path::exists`:

> *"If you cannot access the metadata of the file, e.g. because of a permission error
> or broken symbolic links, this will return `false`."*

So every `.exists()` / `.is_dir()` / `.is_file()` in a boolean position is a site where
an error has been silently converted into a definite negative answer. Most are harmless
(a cache probe, a "create if missing"). Some are load-bearing, and there is currently no
way to tell which without reading each one — the same condition every sibling static
check was built to end.

This would be the **twelfth** source-level static check, following the established
pattern (alloc-sink T-2527, drain-sink T-2531, silent-exit T-2666, busy-spin T-2672,
platform-lock T-2693, error-code-emission T-2699, version-derivation T-2746,
mcp-parity-census T-2747, release-artifact-drift T-2751, verification-pipefail T-2775,
task-template-idioms T-2777).

## Acceptance Criteria

### Agent
- [x] Census first, design second: the count of `.exists()` / `.is_dir()` / `.is_file()`
      sites in the product crates is measured and recorded in this task BEFORE the check
      is written. If the surface is large enough that the allowlist would be a ledger
      rather than a review list, that is stated plainly and the scope is cut to the
      crates where a wrong answer is user-visible — not silently absorbed.
      → **MEASURED: 384** bare predicate sites (cli 78, mcp 194, hub 60, session 39).
      That is a ledger, not a review list, so the scope was cut on the axis that
      matters — not by crate, but by **consequent**. Two candidate anchors were measured
      and rejected: `.filter()` closures (**1** real site — too narrow to justify a
      check) and bare `if !x.exists() {` (**220** — mostly benign `create-if-missing`,
      which fails loudly on its own). The shape that actually turns a failed read into a
      wrong ANSWER is a negated predicate gating an **empty-success return**: **10** sites.
      The check therefore says nothing about the other 374, and says so on every output
      path.
- [x] `scripts/check-error-swallowing-predicate.sh` exists, carries the
      `# guard-layer: source` marker so `run-guard-layer.sh` picks it up, and implements
      the sibling contract: exit 0 clean / 1 unacknowledged / 2 tooling (fail-closed),
      `--json`, `--root` (repeatable) and `--allowlist` for fixtures, `--quiet` /
      `--no-heartbeat` for cron.
      → `run-guard-layer.sh --list` shows both new members; full layer run is
      **48/48 clean** (46 before this task).
- [x] Allowlist lives at `.context/checks/error-swallowing-allowlist` (git-tracked per
      T-2681, NOT under gitignored `.context/working/`), and each entry's reason states
      **what the code does when the predicate is false for a reason other than absence**
      — "it's fine" is not a reason, matching the stricter platform-lock rule (T-2693).
      → 8 acknowledged, each with the EACCES behaviour spelled out. Two entries
      (`inbox.rs` × 2) are acknowledged with an explicit *condition*: the empty answer is
      NOT provably correct there, it is accepted only because T-1166 is retiring that
      primitive, and the entry says so and names what must change if retirement is
      abandoned. That is the honest form of an acknowledgement rather than a silent one.
- [x] Load-bearing proof: reverting T-2791's `classify_candidate` back to a bare
      `.filter(|d| d.is_dir())` makes the check FIRE on that site; restoring returns the
      tree to clean. Demonstrated, not asserted.
      → **The AC as written was wrong and is corrected here rather than fudged.** That
      revert does NOT fire this check, and should not: a filter closure is a different
      position from an empty-success gate, and is out of the declared scope. The
      load-bearing proof is fixture case **I1/I2**, which reproduces
      `manager.rs::list_sessions_in` exactly as it stood before T-2792 and requires a
      fire. Claiming the original AC passed would have been the precise failure mode
      this whole task exists to prevent.
- [x] Fixture suite `tests/error-swallowing-check-fixtures.sh` covers at minimum: a bare
      predicate fires; an acknowledged one does not; a predicate whose false-branch is
      provably an error path does not; a comment mentioning `.exists()` does not; and a
      PL-219 control that the real tree scans clean at the end.
      → **26 assertions, 26 pass**, across 10 groups (A–J) including all five named
      cases, `Err`/`bail!` consequents, a comment *between* gate and return (the T-2688
      lesson applied preemptively), and fail-closed tooling errors.
- [x] Both output paths — including the clean one — carry an explicit scope disclaimer
      (T-2680): the check detects a known error-swallowing SHAPE, it does not prove every
      predicate in the tree is used correctly.
      → Asserted by fixtures H1–H4 on text-clean, text-firing, and JSON-clean.

**Unplanned finding — the check found a gap in T-2791, its own predecessor.** On first
run it fired on `manager.rs::list_sessions`, whose `dirs.is_empty()` fallback re-probed
the *same* unreadable directory with `!exists()` and returned `Ok(vec![])`. T-2791 had
fixed discovery and the `topics` command, but every OTHER caller of `list_sessions` was
still being told "no sessions" for a directory it merely could not read. Fixed here
(both `list_sessions` and `list_sessions_in`), 516 session tests green.

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

out=$(bash tests/error-swallowing-check-fixtures.sh 2>&1 || true); grep -q "26 passed, 0 failed" <<< "$out"
out=$(bash scripts/check-error-swallowing-predicate.sh --no-heartbeat 2>&1 || true); grep -q "0 unacknowledged" <<< "$out"
out=$(bash scripts/check-error-swallowing-predicate.sh --no-heartbeat --json 2>&1 || true); grep -q '"ok":true' <<< "$out"
out=$(bash scripts/check-error-swallowing-predicate.sh --no-heartbeat --json 2>&1 || true); grep -q '"scope"' <<< "$out"
out=$(bash scripts/run-guard-layer.sh --list 2>&1 || true); grep -q "check-error-swallowing-predicate.sh" <<< "$out"
out=$(bash scripts/run-guard-layer.sh --list 2>&1 || true); grep -q "error-swallowing-check-fixtures.sh" <<< "$out"
out=$(head -3 scripts/check-error-swallowing-predicate.sh 2>&1 || true); grep -q "guard-layer: source" <<< "$out"
test -f .context/checks/error-swallowing-allowlist
out=$(git check-ignore .context/checks/error-swallowing-allowlist 2>&1 || true); test -z "$out"
out=$(cargo test -p termlink-session --lib 2>&1 || true); grep -q "0 failed" <<< "$out"

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

**Symptom:** T-2791 shipped a Directive #2 defect that had been live for the life of the
code and was found by a *peer project*, not by this repo. Nothing in the guard layer
could have found it.

**Root cause:** the layer had eleven source-level checks and none of them asked "can this
SUCCESS path be reached by a failed read?" `check-silent-exit.sh` (T-2666) is the nearest
neighbour and covers the same Directive, but its anchor is a **non-zero exit that prints
nothing**. T-2791 was the inverse: a **zero exit printing a positive, plausible, wrong
claim**. The inverse is strictly worse — a silent failure at least leaves the operator
suspicious, whereas a confident wrong answer terminates the investigation — and it was
the one shape with no coverage.

**Why structurally allowed:** the underlying hazard is a std-library ergonomic.
`Path::exists()` reduces a three-state question (present / absent / unknowable) to a
bool, and its docs say plainly that a permission error yields `false`. Every use is
therefore a potential error-swallow, which is exactly why nobody had built a check: 384
sites is a ledger, and a ledger nobody works down is the T-2483 shape (a guard whose
green means nothing). The insight that unblocked it was that the hazard is not the
predicate but the **consequent** — only an empty-success return converts the swallow into
a wrong answer. That cut 384 → 10 and made the check a review list.

**Prevention:** `scripts/check-error-swallowing-predicate.sh`, in the guard layer (48/48),
with a 26-assertion fixture suite whose case I1 reproduces the pre-fix
`list_sessions_in` and requires a fire — so the detector cannot silently stop recognising
its own defect. The allowlist is git-tracked (T-2681) and each entry must state the
EACCES behaviour, so an acknowledgement is an analysis rather than a silencer.

**Evidence it was worth building:** it fired on its first run on a gap in **T-2791
itself** — `list_sessions`'s fallback re-probed the same unreadable directory and
returned an empty list, so the fix I had shipped an hour earlier was incomplete for every
caller except the one command I had looked at. A guard earning its keep on the task that
created it is the strongest available argument that the blindness was real. (workflow_type=build with bug-tag, OR title matches
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

### 2026-08-18T13:11:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2792-guard-layer-has-no-member-for-zero-exit-.md
- **Context:** Initial task creation
