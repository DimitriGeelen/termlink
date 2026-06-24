---
id: T-2267
name: "Cross-hub comms stability: fix RPC scope mapping + actionable auth errors"
description: >
  Cross-hub comms stability: fix RPC scope mapping + actionable auth errors

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-24T07:35:58Z
last_update: 2026-06-24T07:51:30Z
date_finished: 2026-06-24T07:51:30Z
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

# T-2267: Cross-hub comms stability: fix RPC scope mapping + actionable auth errors

## Context

Recurring cross-hub comms instability (ring20 .122 and peers). Live root cause
confirmed 2026-06-24: every `channel.*` JSON-RPC method falls through to the
deny-by-default `_ => PermissionScope::Execute` arm
(`crates/termlink-session/src/auth.rs:202`, via `hub_method_scope`
`crates/termlink-hub/src/server.rs:371-389`) because the channel surface was
never added to either scope table. Result: read-only methods like
`channel.list`/`state`/`info`/`subscribe` wrongly demand `execute` scope, so
sane-scoped cross-hub callers (even the MCP `control` default) are denied on a
plain read — then misdiagnose it as an identity/signing problem. Paired with a
non-actionable `-32010` error (`server.rs:835-854`, names required scope but
gives no remediation), this produces the repeated "I can't reach the hub"
failures. Deep review of the full comms surface (scope model, transport/auth,
messaging UX) feeds this fix; report at `docs/reports/T-2267-comms-review.md`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Read-only `channel.*` methods (list, subscribe, receipts, claims, claims_summary, cv_keys) plus `agent.find_idle` are explicitly classified as `Observe` in `hub_method_scope` — verified by unit test `channel_surface_has_explicit_scopes`.
- [x] Append / own-lease mutations (post, claim, renew, release) + addressed delivery (event.emit_to) are `Interact`; topic-lifecycle / policy / operator-override / destructive ops (create, set_retention, transfer_claim, force_release, trim, sweep) are `Control` — verified by unit test. (See Decisions: own-lease=Interact, override/destructive=Control.)
- [x] A scope-matrix regression test asserts the classification for the full known `channel.*` surface, so a future method falling through to the `Execute` catch-all fails the test (closes the silent-drift gap).
- [x] The `-32010` permission-denied error message includes an actionable remediation hint (names the required scope, the `termlink token create --scope` command, and that it is a scope mismatch not a bad secret) — matching the actionability of the existing AUTH_REQUIRED hint.
- [x] `cargo test -p termlink-session -p termlink-hub` passes (755 tests, 0 failures); `cargo build` is clean.

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
cargo test -p termlink-session -p termlink-hub 2>&1 | tail -20
cargo build 2>&1 | tail -5

## RCA

**Symptom:** Cross-hub callers (agents and MCP tools) repeatedly fail to talk to remote hubs (esp. ring20 .122). Even read operations like `channel.list` are denied unless the caller holds an `execute`-scope token; callers misread the denial as a missing-identity/signing problem and give up.

**Root cause:** The `channel.*` RPC surface was never enumerated in either scope table (`hub_method_scope` server.rs:371 / `method_scope` auth.rs:171). All channel methods therefore hit the deny-by-default `_ => PermissionScope::Execute` arm (auth.rs:202), so read-only channel reads require the highest privilege tier.

**Why structurally allowed:** The scope tables were extended method-by-method as features shipped, but the catch-all `Execute` default silently absorbed the entire `channel.*` family — no test asserted that known channel methods get a sensible scope, so the misclassification never surfaced as a failure, only as field friction. Self-asserted scope (any secret-holder mints any scope) also meant the misclassification provided zero security benefit while harming honest low-scope callers.

**Prevention:** A scope-matrix regression test over the full known `channel.*` surface — any method that falls through to the `Execute` catch-all instead of an explicit classification fails the test. Plus an actionable `-32010` message so a future scope mismatch is self-diagnosing rather than misattributed.

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

### 2026-06-24 — claim-lifecycle scope split (Interact vs Control)
- **Chose:** `channel.claim`/`renew`/`release` → **Interact**; `channel.transfer_claim`/`force_release` → **Control**.
- **Why:** claim/renew/release mutate only the caller's OWN lease (self-scoped, low blast radius) — symmetric with `channel.post` appending one's own message. transfer_claim and force_release reassign or override ANOTHER worker's ownership, so they warrant the higher Control tier. Mirrors the audit recommendation.
- **Rejected:** Putting all claim-lifecycle at Control (my pre-audit AC draft) — would force ordinary workers to hold Control just to manage their own work units, defeating the least-privilege intent.

### 2026-06-24 — fix scope at hub_method_scope, keep deny-by-default
- **Chose:** Add explicit arms to `hub_method_scope` (server.rs); leave the `_ => auth::method_scope` → `_ => Execute` deny-by-default intact.
- **Why:** Genuinely unknown/future methods should still fail closed. The bug was never the default — it was that the known `channel.*` surface was never enumerated. Explicit classification preserves the security property while fixing the honest-caller breakage.
- **Rejected:** Flipping the catch-all to Observe — would silently expose any new method at the lowest tier.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-24T07:35:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2267-cross-hub-comms-stability-fix-rpc-scope-.md
- **Context:** Initial task creation

### 2026-06-24T07:51:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
