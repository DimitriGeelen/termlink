---
id: T-2749
name: "Pin mirror-grid wide-char/VS-16 width behaviour with a characterization test, then decide (herdr rank 22)"
description: >
  mirror_grid.rs drops combining marks and unicode-width 0.1 measures emoji-presentation as 1 while terminals draw 2; pin current behaviour and record an explicit decision rather than upgrading by momentum

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/mirror_grid.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T20:55:37Z
last_update: 2026-08-15T21:00:17Z
date_finished: 2026-08-15T21:00:17Z
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

# T-2749: Pin mirror-grid wide-char/VS-16 width behaviour with a characterization test, then decide (herdr rank 22)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] CHARACTERIZATION tests pin the CURRENT behaviour of `MirrorGrid::put_char` for the
      four cases that matter, each asserting what the code does today, not what is right:
      (a) a combining mark is DROPPED entirely (`mirror_grid.rs:186-189`), so the base
      character is not composed with it; (b) VS-16 (U+FE0F) is likewise dropped;
      (c) `➡️` (U+27A1 + VS-16) therefore occupies ONE cell, while terminals draw two;
      (d) a genuinely wide char (CJK) occupies two cells and marks its continuation cell
      — four tests added; all four pass against unmodified `put_char`, confirming the
      characterization is accurate rather than assumed.
- [x] Each test carries a comment stating it is a characterization test — it pins present
      behaviour so that a future change is DELIBERATE and visible in the diff, and a test
      failing here means behaviour changed, not necessarily that something broke
      — stated in a block header over all four, naming which two pin arguably-WRONG
      behaviour so a later reader does not mistake the pin for an endorsement.
- [x] The tests fail if the behaviour changes: proven by temporarily altering `put_char`
      (e.g. rendering zero-width chars instead of dropping them) and observing the
      specific characterization test fail, then restoring to a zero-diff tree
      — replaced the zero-width drop with `w = 1`: exactly the three zero-width tests
      FAILED (combining / vs16 / emoji) and the CJK control still passed — precise
      discrimination, not a blanket failure. `put_char` restored; diff is test lines only.
- [x] `cargo test -p termlink --bins` passes
      — 30 passed / 0 failed in the `mirror_grid` filter after restore.
- [x] A DECISION is recorded in `## Decisions` on whether to take the `unicode-width`
      major bump, with rationale — worker 1's explicit warning is "do not let this become
      a dependency upgrade by momentum", so the decision must be made and written down
      rather than left implicit by shipping a test and moving on
      — DECLINED, with the cost/benefit stated (workspace-wide API-breaking bump vs a
      one-column drift in a convenience view whose byte stream is untouched), three
      rejected alternatives, and the condition that would reverse it.
- [x] The herdr backlog banner records rank 22's outcome, including that this task
      deliberately does NOT change rendering behaviour

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

cargo test -p termlink --bins mirror_grid

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

### 2026-08-15 — Do not take the `unicode-width` major bump; pin and stop

- **Chose:** Keep `unicode-width = "0.1"` (locked at 0.1.14) and keep dropping zero-width
  characters. Ship the characterization tests, change no rendering behaviour, and record
  the known defect rather than fix it now.
- **Why:** The defect is real but strictly cosmetic and strictly local. `➡️` reserves one
  cell where a terminal draws two, so the mirror and the real screen disagree by one column
  from that point on that line. **The mirrored byte stream is untouched** — this is a
  rendering artifact in a convenience view, not corruption of anything TermLink transports
  or persists. Weigh that against a major-version bump of a workspace-wide dependency whose
  0.2 line changed its API: the blast radius reaches every crate that measures text, to
  correct a one-column drift in a mirror pane. That trade does not pay for itself right now,
  and it is exactly the "dependency upgrade by momentum" worker 1 warned against — the
  upgrade is attractive because it is *available*, not because the cost of the bug demands it.
- **Rejected:**
  - *Bump `unicode-width` to 0.2 now.* Not wrong in principle, and probably right eventually;
    wrong to do incidentally inside a task whose stated purpose was to characterize behaviour.
    An API-breaking bump deserves its own task, its own blast-radius check, and its own test
    pass — not a drive-by inside a display-fidelity ticket.
  - *Compose combining marks instead of dropping them.* A larger change than it looks: the
    grid is one `char` per cell, so composing means either a per-cell string or a normalization
    pass, both of which change the `Cell` representation that diff-render depends on.
  - *Do nothing at all.* Rejected because the behaviour was undocumented and unpinned: any
    future edit could have changed it silently in either direction, and nobody would have
    known which was intended.
- **What would change this:** a user-visible complaint about mirror alignment, or an
  independent reason to bump `unicode-width` (another consumer needing 0.2). At that point
  the tests here become the specification of what changes — they name the current answers,
  so a diff updating them shows exactly what moved.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-15T20:55:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2749-pin-mirror-grid-wide-charvs-16-width-beh.md
- **Context:** Initial task creation

### 2026-08-15T21:00:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
