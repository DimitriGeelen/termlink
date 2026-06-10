---
id: T-2126
name: "ensure_topic auto-pick high-rate retention — T-2125 code follow-up"
description: >
  ensure_topic auto-pick high-rate retention — T-2125 code follow-up

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, substrate-primitive-9, code-followup]
components: [crates/termlink-cli/src/commands/channel.rs]
related_tasks: [T-2125, T-2058, T-2018, T-1991]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T15:17:13Z
last_update: 2026-06-10T15:17:13Z
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

# T-2126: ensure_topic auto-pick high-rate retention — T-2125 code follow-up

## Context

T-2125 (recommended-retention table in `substrate-orchestrator-recipe.md`) shipped
the operator-facing guidance for high-rate substrate topics but left a defence-in-depth
gap: the CLI's `ensure_topic` helper at `crates/termlink-cli/src/commands/channel.rs:1749`
hard-codes `{"kind": "forever"}` regardless of topic-name pattern. The hub's T-2058
loud-warn fires on create (`crates/termlink-hub/src/channel.rs:349`) but operators
who run with reduced log verbosity miss it.

The same `is_high_rate_pattern` predicate the hub already uses
(`crates/termlink-hub/src/channel.rs:326`) is `pub(crate)` and not accessible from
the CLI crate. Per T-2069 convention (tiny pure helpers are duplicated, not
cross-crate-shared), duplicate it into the CLI and switch `ensure_topic` to pick
`Retention::Messages(1000)` when the pattern matches, `Retention::Forever`
otherwise.

The dominant call site is `cmd_channel_dm` (line 1811) which always creates
`dm:<a>:<b>` topics — every DM auto-create today lands `Retention::Forever`. The
secondary call site is `channel post --ensure-topic` (line 450) which can hit any
pattern.

## Acceptance Criteria

### Agent
- [x] `is_high_rate_pattern` predicate duplicated into `crates/termlink-cli/src/commands/channel.rs` matching the hub's pattern set verbatim (`agent-presence`, `agent-chat-arc`, `agent-listeners-*`, `agent-conv-*`, `dm:*`)
- [x] `ensure_topic` picks `{"kind":"messages","value":1000}` when `is_high_rate_pattern(name)` is true, `{"kind":"forever"}` otherwise
- [x] Unit test `tests::is_high_rate_pattern_matches_known_patterns` (or co-located) verifies all five patterns + at least two negatives
- [x] `cargo check -p termlink` clean (warnings tolerated only if pre-existing on `main`)
- [x] `cargo test -p termlink --bin termlink is_high_rate_pattern` passes both is_high_rate_pattern test cases
- [x] Cross-reference comment in `ensure_topic` body pointing at `T-2058 / T-2125` and the duplicated predicate, so the next reader knows why both crates carry it

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

cd /opt/termlink && grep -q "fn is_high_rate_pattern" crates/termlink-cli/src/commands/channel.rs
cd /opt/termlink && out=$(grep -A 3 'async fn ensure_topic' crates/termlink-cli/src/commands/channel.rs); echo "$out" | grep -q "is_high_rate_pattern"
cd /opt/termlink && grep -q '"kind": "messages"' crates/termlink-cli/src/commands/channel.rs
cd /opt/termlink && cargo check -p termlink 2>&1 | tail -1 | grep -q "Finished\|Compiling"
cd /opt/termlink && cargo test -p termlink --bin termlink is_high_rate_pattern 2>&1 | tail -10 | grep -q "test result: ok"
cd /opt/termlink && grep -q "T-2058\|T-2125" crates/termlink-cli/src/commands/channel.rs

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

### 2026-06-10 — filing
- **What changed:** T-2125 shipped the docs-first half (retention guidance to operators); this task is the code-level complement (CLI auto-create defence-in-depth).
- **Plan impact:** Predicate must be duplicated (T-2069 convention) not cross-crate-imported.
- **Triggered:** No new sub-tasks expected; single-deliverable build.

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

### 2026-06-10T15:17:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2126-ensuretopic-auto-pick-high-rate-retentio.md
- **Context:** Initial task creation
