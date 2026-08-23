---
id: T-2727
name: "PTY sessions spawn with 0x0 winsize — openpty called with NULL winp"
description: >
  PTY sessions spawn with 0x0 winsize — openpty called with NULL winp

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
created: 2026-08-15T08:02:07Z
last_update: 2026-08-15T08:05:37Z
date_finished: 2026-08-15T08:05:37Z
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

# T-2727: PTY sessions spawn with 0x0 winsize — openpty called with NULL winp

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `PtySession::spawn` passes a non-NULL `winsize` to `openpty`, so a freshly
      spawned session reports a sane size before any caller-driven resize
- [x] The default is a named constant, not a bare literal, so the value is
      greppable and its rationale is stated at the definition
- [x] An existing caller-driven `resize()` still overrides the default — the
      default is a floor, not a pin
- [x] A regression test asserts `TIOCGWINSZ` on a freshly spawned PTY returns
      `rows > 0 && cols > 0`, and FAILS against the pre-fix code
- [x] `cargo test -p termlink-session` passes

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

# Capture-first (L-387): piping cargo straight into grep -q SIGPIPEs it (exit 101).
# Assert absence of failures rather than a literal pass-count, which would go
# stale the moment anyone adds a test.
out=$(cargo test -p termlink-session 2>&1); ! echo "$out" | grep -q "FAILED"
# The fix itself: winp is no longer NULL.
grep -q '&initial_ws,' crates/termlink-session/src/pty.rs
# Named constants, not bare literals.
grep -q 'pub const DEFAULT_PTY_ROWS' crates/termlink-session/src/pty.rs
grep -q 'pub const DEFAULT_PTY_COLS' crates/termlink-session/src/pty.rs
# Both regression tests exist and pass.
out=$(cargo test -p termlink-session winsize 2>&1); echo "$out" | grep -q "2 passed"

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

**Symptom:** every PTY-backed session spawned by TermLink reported a terminal
size of 0 rows × 0 columns to its child, until and unless some caller happened
to drive `resize()`. Nothing in the spawn path does. A full-screen child — vim,
less, top, or the agent TUIs this tool exists to host — queries `TIOCGWINSZ` at
startup and is told the terminal has no size.

**Root cause:** `crates/termlink-session/src/pty.rs` called
`libc::openpty(&master, &slave, NULL, NULL, NULL)`. The fifth argument is
`winp`, the initial window size. Passing NULL does not get a sensible kernel
default — it leaves the pty at 0×0. Verified independently before fixing:
`python3 -c "os.openpty(); ioctl(TIOCGWINSZ)"` → `rows=0 cols=0`.

**Why structurally allowed:** the codebase has exactly three `TIOCSWINSZ` /
`winsize` sites, and all three are caller-driven (`resize()` at `pty.rs:283/290`
and the client's own tty at `util.rs:100`). Sizing was treated throughout as
something a *client* supplies on attach, so the headless case — spawn a session,
never attach an interactive client, run a full-screen program in it — had no
owner. That case is the normal one for an agent-driven session, which is the
product's whole purpose. No test asserted a spawned PTY's size, so the defect
was invisible to the suite.

**Prevention:** `spawn_has_nonzero_winsize` asserts `TIOCGWINSZ > 0` on a fresh
session, and `resize_overrides_default_winsize` pins that the seeded value is a
floor rather than a pin — so a future change that hardcodes the size would fail
too. The pair is load-bearing: reverting `&initial_ws` to `std::ptr::null_mut()`
was tested and fails with "fresh PTY reported a degenerate 0x0 window size".

**Provenance:** found by reading herdr's issue corpus as a source of terminal
edge cases (T-2725 worker 1, class A, citing herdr #2828/#1709/#2625). That was
the explicit hypothesis of the herdr adoption research — that a younger PTY
implementation's bug backlog is worth more to us than its code — and this is the
first confirmed instance of it paying out.

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

### 2026-08-15T08:02:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2727-pty-sessions-spawn-with-0x0-winsize--ope.md
- **Context:** Initial task creation

### 2026-08-15T08:05:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
