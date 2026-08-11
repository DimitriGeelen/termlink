---
id: T-2610
name: "Born-dead retention: set_retention/create accept messages:0/days:0 (silent topic wipe on sweep)"
description: >
  Reliability #2: retention_from_json accepts value:0 -> Messages(0)/Days(0); on next sweep the topic is silently wiped (keep_last=0 / cutoff=now). set_retention returns ok:true; create silently defaults on invalid. Floor to >=1 + make create loud on present-but-invalid. Sibling of T-2604 born-dead ttl.

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
created: 2026-08-11T16:19:36Z
last_update: 2026-08-11T16:19:36Z
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

# T-2610: Born-dead retention: set_retention/create accept messages:0/days:0 (silent topic wipe on sweep)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `retention_from_json` (crates/termlink-hub/src/channel.rs) rejects a `value`
      of `0` on both the `days` and `messages` arms (returns `None`), so
      `channel.set_retention` surfaces its existing `-32602 "Missing or invalid
      'retention'"` instead of accepting a born-dead `Days(0)`/`Messages(0)` that
      wipes the topic on the next `channel.sweep` (`keep_last=0` / `cutoff=now`).
      Floors to `>=1`, matching the repo's count-param convention (`cap_per_topic.max(1)`,
      `ttl_ms.max(1)`, `since_days.clamp(1,365)`). `Retention::Latest` already covers
      the legitimate "keep newest 1" case.
- [x] `channel.create` rejects a **present, non-null** but invalid `retention`
      (value:0 OR unknown kind) with a loud `-32602` instead of silently falling
      back to its default — while an **absent or null** `retention` still applies
      the T-2426 debris/Forever default (backward-compatible). Closes the sibling
      silent-ignore on the create path.
- [x] Load-bearing unit tests: `retention_from_json` returns `None` for
      `{messages,0}` and `{days,0}` (and `Some` for value `1`); `set_retention`
      value:0 → `-32602`; `create` value:0 → `-32602`; `create` with absent
      retention still succeeds with the default. Proven load-bearing by
      temp-reverting the `messages` guard and confirming a test FAILS.
- [x] `cargo test -p termlink-hub` passes.

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

cargo test -p termlink-hub retention_from_json_rejects_zero_value
cargo test -p termlink-hub
# The guards are present on both retention arms:
grep -q "Messages(0) → sweep keep_last=0" crates/termlink-hub/src/channel.rs
grep -q "Days(0) makes sweep" crates/termlink-hub/src/channel.rs
# create rejects present-but-invalid retention loudly:
grep -q "invalid 'retention' in params" crates/termlink-hub/src/channel.rs

## RCA

**Symptom:** `channel.set_retention(name, {kind:"messages", value:0})` returned
`{ok:true}`; on the next `channel.sweep` the topic was **silently and entirely
wiped** — `Retention::Messages(0)` maps to `keep_last=0` (delete every record),
and `Retention::Days(0)` sets the sweep cutoff to `now` (prune everything). A
caller fat-fingering `value:0` (or assuming 0-indexed "keep newest") schedules a
total data loss with no error. `channel.create` with the same input silently
defaulted to Forever/Days(7), discarding the caller's explicit (if invalid) intent.

**Root cause:** `retention_from_json` applied no lower floor to the `days`/`messages`
value, unlike the repo's pervasive count-param convention (`cap_per_topic.max(1)`,
`ttl_ms.max(1)`, `since_days.clamp(1,365)`). `0` was accepted as a valid retention
even though a distinct `Retention::Latest` already covers "keep newest 1", so `0`
is never sane. Same born-dead-value class as T-2604 (claim `ttl_ms=0` → `ok:true`).

**Why structurally allowed:** the parse returned `Some(Retention::Messages(0))`
— compile-clean, and existing tests only exercised valid values or the
missing-retention path, never `value:0`. The destructive consequence was one
indirection away (in `Bus::sweep`), so nothing linked the accepted value to the
wipe. On `create` the `.unwrap_or_else(default)` conflated "absent" with
"present-but-invalid", hiding the bad value entirely.

**Prevention:** (a) `retention_from_json` floors both arms to `>=1` (born-dead
`0` → `None` → the handlers' existing `-32602`); (b) `create` now distinguishes
present-but-invalid (loud `-32602`) from absent/null (default), closing the
silent-default sibling; (c) load-bearing tests assert `None`/`-32602` for
`value:0` on `retention_from_json`, `set_retention`, and `create`, proven to fail
if a guard is reverted — so a regression to the silent-wipe is caught by the suite.

<!-- template (kept for the completion gate's anchor):
     REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
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

### 2026-08-11T16:19:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2610-born-dead-retention-setretentioncreate-a.md
- **Context:** Initial task creation
