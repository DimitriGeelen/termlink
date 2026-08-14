---
id: T-2687
name: "MCP termlink_topics returns a silently partial inventory — CLI parity fields never migrated"
description: >
  T-2624 added sessions_probed/skipped/unreachable/bad_result to the CLI's topics --json as an explicit partial-inventory signal, but never to the MCP termlink_topics tool. An MCP consumer cannot tell the topic set excludes unreachable sessions. Caught by parity_topics, which has been red since 2026-08-12 because nothing runs cargo test (T-2683 F1).

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
created: 2026-08-14T06:11:35Z
last_update: 2026-08-14T06:31:28Z
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

# T-2687: MCP termlink_topics returns a silently partial inventory — CLI parity fields never migrated

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink_topics` (MCP) emits `sessions_probed`, `sessions_unreachable`, `sessions_bad_result`, `sessions_skipped` — the T-2624 partial-inventory signal the CLI already carries
- [x] Probe outcomes are classified the same way as the CLI (`Unreachable` for timeout/transport, `BadResult` for an unwrappable or malformed result), not silently dropped
- [x] The zero-session early-return path emits the same field set (all zeros) on BOTH sides — T-2624 missed that path too, so the two surfaces diverged there as well
- [x] `cargo test -p termlink-mcp --test parity` passes, including `parity_topics`
- [x] A unit test covers the classification helper directly (unreachable vs bad-result vs topics, and empty topic lists excluded from `sessions`)
- [x] `cargo test --workspace` is green — no other suite regressed

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

cargo test -p termlink-mcp --test parity
cargo test -p termlink-mcp --lib topics_probe
cargo build -p termlink-mcp -p termlink

## RCA

**Symptom:** `cargo test --workspace` failed on `parity_topics`. `termlink topics --json`
(CLI) returned four fields — `sessions_probed`, `sessions_unreachable`,
`sessions_bad_result`, `sessions_skipped` — that `termlink_topics` (MCP) did not.

**Root cause — two distinct defects, one masking the other.**

1. **Product.** T-2624 added those four fields to the CLI as an explicit
   partial-inventory signal ("a consumer can now tell the topic set excludes sessions
   that timed out or errored") and never added them to the MCP tool. The MCP handler
   used an `if let Ok(Ok(resp)) = … && let Ok(result) = … && let Some(topics) = …`
   chain, so every failed probe fell out of the chain into silence: an agent calling
   `termlink_topics` received a truncated inventory presented as complete. Directive #2
   ("no silent failures") was fixed for the human-facing surface and left broken on the
   agent-facing one — the surface with *more* consumers, since MCP is how agents read
   the substrate.

2. **Test harness.** Once the field sets matched, the test still failed on VALUES: MCP
   reported `sessions_unreachable: 0`, the CLI reported `1`. `parity_topics` ran on the
   default current-thread tokio runtime while `call_cli` blocks the test thread in
   `.output()`. That starved the session's `accept_loop`, so the CLI subprocess could
   never be accepted and timed out. The file's own comment listed `parity_topics` among
   tests that "do not need the socket" because it is *hub-less* — conflating "needs no
   hub" with "needs no socket", when it starts a SESSION the CLI must RPC. Diagnostic
   confirmation: with the multi_thread flavor the test completes in 0.06s instead of
   burning two 5s probe timeouts.

**Why structurally allowed:** the parity test existed, was correct, and would have
caught defect 1 on the day it landed — but **nothing runs `cargo test`**.
`release.yml` runs `cargo build --release`; `doc-lint.yml` runs 2 of 28 check scripts;
the pre-push audit runs the `structure` section only. The suite was red from
2026-08-12 and surfaced only because T-2683 happened to run it by hand. Defect 2 was
invisible underneath defect 1: the field-set mismatch failed the assert before the
value mismatch could be reached.

**Prevention:** T-2686 adds a `test` job to `release.yml` running `cargo test
--workspace`, with both build jobs declaring `needs: [test]` so a red suite blocks the
build outright, plus a per-commit `guard-layer` job in `doc-lint.yml`. The correcting
comment in `parity.rs` now states the real rule — *any* test that starts a session and
shells out to the CLI needs `multi_thread` — rather than the "hub-less" proxy that
misclassified this one.

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

### 2026-08-14T06:11:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2687-mcp-termlinktopics-returns-a-silently-pa.md
- **Context:** Initial task creation

### 2026-08-14T06:12:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
