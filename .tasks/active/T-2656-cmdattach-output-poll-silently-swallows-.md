---
id: T-2656
name: "cmd_attach output-poll silently swallows hub RPC-error (frozen-but-connected terminal, no signal) — sibling of T-2644"
description: >
  pty.rs cmd_attach: if let Ok(result) = unwrap_result(resp) drops an application-level RPC error on query.output; loop spins silently while the transport-Err sibling loudly breaks. Keystrokes also vanish (fire-and-forget). Fix must distinguish transient vs fatal RPC errors.

status: captured
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
created: 2026-08-12T19:56:51Z
last_update: 2026-08-12T19:57:36Z
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

# T-2656: cmd_attach output-poll silently swallows hub RPC-error (frozen-but-connected terminal, no signal) — sibling of T-2644

## Context

Round-11 silent-failure hunt (verified in code). In `cmd_attach`'s poll branch
(`crates/termlink-cli/src/commands/pty.rs:501-526`), the `query.output` response
is handled as:

```rust
Ok(resp) => {
    if let Ok(result) = client::unwrap_result(resp) { ... }
    // else: application-level RPC error dropped, loop continues silently
}
Err(_) => { eprintln!("\r\nConnection lost."); break; }
```

The transport-`Err` arm (pty.rs:522) loudly surfaces + breaks, but the
`unwrap_result` `Ok`-only match one branch up silently drops a JSON-RPC
*application* error (transport succeeded, hub returned an error object — e.g.
OUTPUT_UNAVAILABLE -32007, or a session-gone error). The loop just keeps
polling: the attached operator sees a frozen-but-"connected" terminal with no
error, and keystrokes also vanish (fire-and-forget `command.inject` at
pty.rs:490). Directive #2 (no silent failures) + #3 (actionable) — the
divergence with the sibling `Err` arm is the tell.

Distinct from T-2644 (same `cmd_attach`, but the INPUT-drop path). One-bug-one-
task: this is the OUTPUT-poll error-swallow.

**Why FILED, not auto-fixed:** the correct fix is a judgment call — not every
RPC error on `query.output` is fatal (a transient may be retryable; a
session-gone is fatal). The fix must classify the error code (surface+break on
fatal like session-not-found; surface-once+continue or backoff on transient),
which is a design decision, and testing it needs an interactive-loop fixture
(hub returns an RPC error while the socket stays open). Delicate/interactive —
same bucket as T-2644.

## Acceptance Criteria

### Agent
- [ ] `cmd_attach` no longer silently discards an application-level RPC error on `query.output` — the error is surfaced to the operator (stderr), mirroring the sibling transport-`Err` arm
- [ ] The error is classified: a fatal error (session-not-found / closed) surfaces + breaks; a transient (e.g. OUTPUT_UNAVAILABLE) surfaces at most once and either backs off or breaks per an explicit decision recorded in `## Decisions`
- [ ] The keystroke fire-and-forget at pty.rs:490 is reviewed in the same pass — decide whether a persistent inject failure should also surface (cross-ref T-2644)
- [ ] A fixture test (hub returns RPC-error on `query.output` with socket open) proves the loop surfaces the error instead of spinning silently
- [ ] `cargo build -p termlink` + `cargo test -p termlink --bins` pass

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

## RCA

**Symptom:** while attached to a session (`termlink attach <sess>`), if the hub
starts returning an application-level RPC error on `query.output` (session
died, output buffer gone, capability revoked), the terminal appears connected
but frozen — no new output, no error message — and typed keystrokes silently
vanish. Only a transport-level disconnect prints "Connection lost."

**Root cause:** the poll branch matches `Ok(resp)` then `if let Ok(result) =
unwrap_result(resp)` with no `else`. A JSON-RPC error object arrives as
transport-`Ok` but `unwrap_result`-`Err`, so it falls through the `if let` and
the loop continues as if there were simply no new output. The sibling
transport-`Err` arm handles its failure loudly; this one does not.

**Why structurally allowed:** `if let Ok(_) = ...` on a fallible result is an
easy silent-drop shape (no compiler warning for the missing `else`), and
Directive #2 (no silent failures) is a convention, not enforced. No test drives
the attach loop against an RPC-error-returning hub.

**Prevention:** surface + classify the RPC error (fatal vs transient), and add a
fixture test for the RPC-error-while-connected case. Broader: an
`if let Ok(_) =` silent-drop lint on interactive I/O loops is a static-check
candidate (sibling to the T-2527/T-2531 source-level checks).

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

### 2026-08-12T19:56:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2656-cmdattach-output-poll-silently-swallows-.md
- **Context:** Initial task creation
