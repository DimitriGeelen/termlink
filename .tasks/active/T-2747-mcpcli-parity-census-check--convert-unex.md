---
id: T-2747
name: "MCP/CLI parity census check — convert unexamined pairs into acknowledged (herdr rank 13)"
description: >
  Guard-layer static check enumerating MCP tools vs CLI commands; fires on any pair neither covered by parity.rs nor allowlisted with a cited reason

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
created: 2026-08-15T20:33:56Z
last_update: 2026-08-15T20:33:56Z
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

# T-2747: MCP/CLI parity census check — convert unexamined pairs into acknowledged (herdr rank 13)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The three circulating counts of `*_mcp` parallel helpers — 83 (T-1904 census),
      68 (`parity.rs` header as of T-2683/T-2689), 94 (herdr worker 3) — are reconciled:
      state the measured number, what each earlier number counted instead, and which
      (if any) was right. No number is carried forward on trust.
      — Measured **79** distinct `fn *_mcp` names. **None of the three was right, and the
      question was malformed:** the unit is ill-defined. `fn to_json_mcp` alone occurs 26
      times as a small helper redefined inside separate functions, so "helpers" can be
      counted as distinct names (79), as definitions (much higher), or as per-tool parallel
      implementations (much lower) — all defensible. 83 and 68 are also stale snapshots of a
      moving number. Resolution: do not count helpers at all. The check counts **tools**
      (`name = "termlink_…"` in `#[tool(…)]`) — one meaning, measured at **260**.
- [x] `scripts/check-mcp-parity-census.sh` exists, carries `# guard-layer: source`,
      enumerates MCP tools from source (no live binary, no cargo build) and reports
      three disjoint states per tool: covered by a `parity.rs` case / acknowledged in
      the allowlist / **unexamined** — the distinction the T-2680 lesson says must not
      collapse into a single green number
      — pure grep over `tools.rs` + `parity.rs`; runs in well under a second.
- [x] The check FIRES (exit 1) on any unexamined tool, and its output states the census
      explicitly (covered N, acknowledged M, unexamined K of TOTAL) so a reader cannot
      mistake partial coverage for full coverage
      — first run: `FIRING — 236 of 260 MCP tool(s) are UNEXAMINED (asserted: 24 = 9.2%)`.
      The CLEAN path states the same census, so a green cannot be misread either.
- [x] Load-bearing: removing a tool name from a covered `parity.rs` case moves that tool
      from covered to unexamined and re-fires; restoring returns it. Proven by
      temp-revert on the real tree, restored to a zero-diff tree
      — renamed the `"termlink_ping"` reference at `parity.rs:189`: covered 24→23,
      coverage 9.2%→8.8%, `↳ termlink_ping` fired. `git diff --stat` empty after restore.
- [x] Control (PL-219): the check does not fire on a tool that IS covered, and does not
      count a tool merely *mentioned* in a comment as covered
      — fixture 1 asserts the covered tool is absent from the firing list; fixture 7 proves
      both comment directions (a doc comment quoting `name = "termlink_help"` in the real
      `tools.rs` would otherwise inflate the total by one).
- [x] Allowlist at `.context/checks/mcp-parity-census-allowlist` (git-tracked per T-2681)
      is honoured; entries are counted and reported, never silently dropped; removing a
      line re-fires that tool
      — real-tree proof: deleting `termlink_agent_ack` re-fired it by name (acknowledged
      236→235). Fixtures 3/5/6 cover counted-not-dropped, removal, and the new-tool ratchet.
- [x] `bash tests/mcp-parity-census-fixtures.sh` passes and is hermetic (fixture source
      trees only — no cargo, no binary, no hub)
      — 26 assertions, 0 failed.
- [x] `bash scripts/run-guard-layer.sh` picks it up as a member and the roll-up is green
      — `PASS — 32/32 members clean` (30 before this task).
- [x] The first-run census figure is recorded in the task and in CLAUDE.md as a MEASURED
      number, with the scope disclaimer the charter-drift check now carries (T-2680): the
      check proves a pair is *asserted*, not that the two implementations agree
      — **260 tools, 24 asserted (9.2%), 236 acknowledged pending T-2748.** Disclaimer is
      emitted by the tool itself on both paths and carried in the `scope` JSON field.

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

bash scripts/check-mcp-parity-census.sh
bash tests/mcp-parity-census-fixtures.sh
out=$(bash scripts/run-guard-layer.sh 2>&1); echo "$out" | grep -q "check-mcp-parity-census"
bash scripts/run-guard-layer.sh

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

### 2026-08-15T20:33:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2747-mcpcli-parity-census-check--convert-unex.md
- **Context:** Initial task creation
