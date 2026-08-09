---
id: T-2584
name: "fix: termlink_remote_inject reports ok:true on no-PTY resolved (T-2580 twin, cross-host path)"
description: >
  termlink_remote_inject (tools.rs:15134-15144 the Ok(Success(r)) arm) hardcodes top-level ok:true and bytes:p.text.len() regardless of what command.inject reported. The RPC has two success shapes (handler.rs:551-558 status:injected; handler.rs:561-570 status:resolved = NO PTY, keys resolved but never injected). Described in-code as the primary cross-host prompt-injection tool, it still asserts unconditional success on the no-PTY path. Failure: orchestrator hands a prompt to a headless remote agent with no PTY, hub returns status:resolved, tool returns ok:true bytes:142, prompt recorded delivered but never received. Fix: reuse the T-2580 mcp_inject_outcome pattern - branch on r.result[status]: injected=ok:true, else ok:false with note. From T-2468 MCP-flattening hunt, twin of T-2580 on the cross-host path.

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
created: 2026-08-09T21:58:12Z
last_update: 2026-08-09T22:14:28Z
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

# T-2584: fix: termlink_remote_inject reports ok:true on no-PTY resolved (T-2580 twin, cross-host path)

## Context

From the T-2468 MCP-flattening hunt — the **T-2580 twin on the cross-host path**.
`termlink_remote_inject` (`crates/termlink-mcp/src/tools.rs:15134-15144`, the
`Ok(Success(r))` arm) hardcodes top-level `"ok": true` and `"bytes": p.text.len()`
regardless of what `command.inject` reported. The RPC has two success shapes
(`crates/termlink-session/src/handler.rs`): L551-558 `status:"injected"` (real PTY
write) and L561-570 `status:"resolved"` (NO PTY — keys resolved but never injected,
with `"note":"No PTY session..."`). This is exactly the shape T-2580 fixed for the
LOCAL `termlink_inject` (now branches via `mcp_inject_outcome`). The remote path —
described in-code as "the primary cross-host prompt-injection tool" — still asserts
unconditional success. It DOES nest the full `"result": r.result` (so `status` is
recoverable by a diligent caller) but the headline `ok:true` lies, and orchestrators
branch on top-level `ok`. Failure: an orchestrator hands a prompt to a headless
remote agent registered without `--shell`; the hub returns `status:"resolved"`; the
tool returns `{ok:true, bytes:142}`; the prompt is recorded delivered but never
received — the "why is there still no response?" coordination failure, silently.

**Fix (SMALL, mechanical):** reuse the T-2580 `mcp_inject_outcome` pattern — branch
on `r.result["status"]`: `"injected"` → `ok:true`; anything else (`"resolved"`) →
`ok:false` with the note surfaced. Add a load-bearing test.

## Acceptance Criteria

### Agent
- [x] `termlink_remote_inject` sets top-level `ok` from the RPC `status` — only
      `status:"injected"` yields `ok:true`; `"resolved"` (no-PTY) yields `ok:false`
      with the RPC note surfaced (not a hardcoded `ok:true`). Done via pure helper
      `mcp_remote_inject_result_json` (tools.rs ~2959).
- [x] A load-bearing unit test asserts the no-PTY/`resolved` shape maps to
      `ok:false` and `injected` to `ok:true` — proven to FAIL on temp-revert.
      Test: `remote_inject_no_pty_is_not_reported_as_success` (FAILS when the
      helper hardcodes `ok:true`; confirmed).
- [x] `cargo test -p termlink-mcp --lib` passes (no regressions). 893 passed, 0 failed.

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
cargo test -p termlink-mcp --lib remote_inject_no_pty_is_not_reported_as_success

## RCA

**Symptom:** an orchestrator hands a prompt to a headless remote agent registered
without `--shell` (no PTY). The hub resolves the keys but writes nothing, returning
`status:"resolved"` as an RPC *success*. `termlink_remote_inject` reported
`{ok:true, bytes:142}` — the prompt recorded delivered but never received: the
"why is there still no response?" cross-host coordination failure, silently.

**Root cause:** the `Ok(Success(r))` arm (tools.rs ~15154) hardcoded top-level
`"ok": true` and `"bytes": p.text.len()` regardless of `r.result["status"]`. The
`command.inject` RPC has two success shapes — `status:"injected"` (real PTY write)
and `status:"resolved"` (no PTY, keys resolved but never injected, with a `note`).
The handler collapsed both into `ok:true`. It DID nest the full `result` (so
`status` was recoverable), but orchestrators branch on the top-level `ok`, which
lied. The MCP-flattening class again: a handler asserts success without inspecting
the structured RPC result.

**Why structurally allowed:** the LOCAL `termlink_inject` had the identical bug and
was fixed in T-2580 (`mcp_inject_outcome`), but the cross-host `termlink_remote_inject`
— described in-code as "the primary cross-host prompt-injection tool" — was a
separate code path with no shared helper and no test pinning its status-awareness,
so the T-2580 fix did not cover it.

**Prevention:** extracted the response build into a pure helper
`mcp_remote_inject_result_json` (single testable choke-point, twin of T-2580's
`mcp_inject_outcome`) + the load-bearing test
`remote_inject_no_pty_is_not_reported_as_success`, which FAILS if the status branch
is removed. This closes both inject surfaces (local T-2580 + remote T-2584). The
recurring MCP-flattening class — found 4× across T-2578/2580/2583/2584 — is captured
as a learning so a future agent recognises the pattern (handler collapses a
structured RPC result and drops/ignores a load-bearing field).

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

### 2026-08-09T21:58:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2584-fix-termlinkremoteinject-reports-oktrue-.md
- **Context:** Initial task creation

### 2026-08-09T21:58:50Z — status-update [task-update-agent]
- **Change:** status: started-work → captured

### 2026-08-09T22:14:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
