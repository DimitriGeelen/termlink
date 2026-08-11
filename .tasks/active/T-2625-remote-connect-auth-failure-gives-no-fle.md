---
id: T-2625
name: "remote connect auth failure gives no fleet-reauth next step (usability dead-end)"
description: >
  remote connect auth failure gives no fleet-reauth next step (usability dead-end)

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
created: 2026-08-11T23:02:28Z
last_update: 2026-08-11T23:02:28Z
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

# T-2625: remote connect auth failure gives no fleet-reauth next step (usability dead-end)

## Context

reliability/usability-hunt (Directive #3 — "actionable errors") Finding #1,
HIGH severity / high confidence, verified in code.

`crates/termlink-cli/src/commands/remote.rs:786` + `:789` — `connect_remote_hub`
is the single connection helper behind ~15 direct `remote *` subcommands
(push/exec/inject/inbox/…). Its auth-failure arms emit only the raw hub
code+message with NO remediation:

```rust
Ok(RpcResponse::Error(e)) => anyhow::bail!("Authentication failed: {} {}", e.error.code, e.error.message),
Err(e)                    => anyhow::bail!("Authentication error: {}", e),
```

Yet the hub-restart / secret-rotation case (PL-021) is the single most-
documented failure mode in this codebase, and `fleet doctor` DOES hint the
remedy (`remote.rs:2195` — `"HMAC secret mismatch — run: termlink fleet reauth …"`).
A user running `termlink remote exec <hub> …` against a hub that rotated its
secret gets a dead-end `Authentication failed: -32xxx invalid signature` while
`fleet doctor` on the same hub would tell them to reauth. The inconsistency IS
the Directive-#3 defect. The `hub` (`host:port`) is in scope at both bail sites,
so the hint can name it and point at `fleet doctor` (to map addr→profile) then
`fleet reauth`. Mirrors the T-2554 `claim_err_actionable` convention (pure hint
helper + load-bearing test).

## Acceptance Criteria

### Agent
- [x] A pure helper builds an actionable auth-failure hint naming the hub, `fleet doctor`, and `fleet reauth` (mirrors T-2554 `claim_err_actionable`)
- [x] Both auth-failure bail sites (`RpcResponse::Error` and transport `Err`) append the hint
- [x] Load-bearing unit test asserts the hint names `fleet reauth` + `fleet doctor` + the hub address; proven load-bearing via temp-revert (helper → empty string → test fails)
- [x] `cargo test -p termlink --bins` passes for the new test

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

cargo test -p termlink --bins commands::remote::tests::auth_failure_hint_names_reauth_recovery

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

**Symptom:** `termlink remote <cmd> <hub>` against a hub that rotated its secret prints a dead-end `Authentication failed: -32xxx invalid signature` with no next step, even though the fix (`fleet reauth`) is well-known and `fleet doctor` hints it for the same condition.

**Root cause:** `connect_remote_hub`'s auth-failure arms (`remote.rs:786`/`:789`) format only the raw hub code+message; they carry no remediation hint, unlike the fleet-doctor path (`remote.rs:2195`) which names `fleet reauth`. Every direct `remote *` subcommand funnels through this one helper, so the gap is fleet-wide across the direct-connect surface.

**Why structurally allowed:** the actionable-error convention (T-2554 `claim_err_actionable`) was applied to the claim family but never retrofitted to the auth path; there was no test asserting that auth failures name a recovery command, so the inconsistency was invisible. Directive #3 ("actionable errors") is enforced by discipline, not by a gate — so a primary path could regress to a dead-end while a secondary path stayed actionable.

**Prevention:** a pure `auth_failure_hint` helper (mirroring T-2554) with a load-bearing unit test asserting the hint names `fleet doctor` + `fleet reauth` + the hub — reverting the helper to empty fails the test. Failure scenario: hub restarts into volatile /tmp (PL-021), rotates secret; `termlink remote exec 192.168.10.122:9100 …` → before: bare "invalid signature"; after: "…— the hub may have rotated its secret (see PL-021). Run `termlink fleet doctor` … then `termlink fleet reauth <profile>`".

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

### 2026-08-11T23:02:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2625-remote-connect-auth-failure-gives-no-fle.md
- **Context:** Initial task creation
