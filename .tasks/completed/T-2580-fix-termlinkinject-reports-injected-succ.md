---
id: T-2580
name: "fix: termlink_inject reports 'Injected successfully' on no-PTY session where nothing was injected"
description: >
  x

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T21:39:00Z
last_update: 2026-08-09T21:42:58Z
date_finished: 2026-08-09T21:42:58Z
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

# T-2580: fix: termlink_inject reports 'Injected successfully' on no-PTY session where nothing was injected

## Context

From the T-2468 charter verb-4 ("control terminal sessions") adversarial hunt. The
`command.inject` RPC (`crates/termlink-session/src/handler.rs:561-570`) returns a
`Response::success` with `status:"resolved"` (NOT an RPC error) when the target
session has no PTY (`ctx.pty == None`, i.e. registered without `--shell`) — the keys
are resolved but written NOWHERE. The MCP `termlink_inject` tool
(`crates/termlink-mcp/src/tools.rs:11395`) maps ANY `Ok(_)` to the literal string
`"Injected successfully"`, ignoring the `status` field. Net: injecting into a
non-PTY session reports "Injected successfully" while the intended command never
reached any terminal — a silent no-op reported as success (Reliability directive:
"no silent failures"; and the specific "inject reports success but the command did
not run" class from the founding session-control verb).

## Acceptance Criteria

### Agent
- [x] `termlink_inject` inspects the RPC result `status` — only `status:"injected"`
      yields "Injected successfully"; any other status (notably `"resolved"`, the
      no-PTY path) returns an honest failure-shaped result (`json_err` /
      `{"ok":false,"error":...}`) carrying the RPC `note`, NOT a success string.
- [x] The mapping is extracted into a pure, unit-testable helper
      (`mcp_inject_outcome`) so the status-awareness has a single choke-point.
- [x] A load-bearing unit test asserts `status:"injected"` → success and
      `status:"resolved"` → an error-shaped (`ok:false`) result that does NOT read
      "Injected successfully" — proven to FAIL if the status check is removed
      (temp-revert).
- [x] `cargo test -p termlink-mcp --lib` passes (no regressions).

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
out=$(cargo test -p termlink-mcp --lib inject_no_pty_is_not_reported_as_success 2>&1); echo "$out" | grep -q "1 passed"
out=$(grep -n 'fn mcp_inject_outcome' crates/termlink-mcp/src/tools.rs); echo "$out" | grep -q mcp_inject_outcome
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

**Symptom:** `termlink_inject` against a session registered without `--shell` (no
PTY) returns "Injected successfully" even though the keystrokes were resolved and
discarded — the intended command never reached any terminal.

**Root cause:** The `command.inject` RPC returns a *success* envelope with
`status:"resolved"` (not an error) for the no-PTY case, and the MCP tool mapped any
`Ok(_)` to the fixed string "Injected successfully" without reading `status`.

**Why structurally allowed:** The RPC's success-with-status shape (injected vs
resolved) is a legitimate internal distinction, but the MCP boundary collapsed both
to one message. There was no per-tool test asserting the no-PTY path reads as a
failure, and the mapping was an inline `Ok(_) =>` with no choke-point to test.

**Prevention:** (1) extracted the pure `mcp_inject_outcome` helper that branches on
`status` — only `"injected"` is success, everything else returns `{ok:false,error}`;
(2) load-bearing test `inject_no_pty_is_not_reported_as_success` — proven to FAIL
when the status check is removed (temp-revert). Same class as T-2578: an MCP tool
flattening a structured RPC result into a fixed string drops load-bearing signal.

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

### 2026-08-09T21:39:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2580-fix-termlinkinject-reports-injected-succ.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-918e27c3
- **Timestamp:** 2026-08-09T21:43:18Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-09T21:42:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
