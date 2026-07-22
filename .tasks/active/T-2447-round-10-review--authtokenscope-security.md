---
id: T-2447
name: "Round-10 review — auth/token/scope security core findings"
description: >
  Capture of round-10 adversarial auth/token/scope-enforcement review; build deferred (session at urgent budget). Decompose one-bug-one-task on pickup.

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
created: 2026-07-21T22:23:13Z
last_update: 2026-07-22T05:54:38Z
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

# T-2447: Round-10 review — auth/token/scope security core findings

## Context

Round-10 adversarial review of TermLink's auth/token/scope-enforcement core
(`termlink-session/src/auth.rs`, hub-side `server.rs` scope enforcement,
`token.rs`/`cli.rs` token lifetime). **Verdict: the security-critical paths are
sound** — see "Verified clean" below. Two genuine MED gaps found. **This is a
capture, not a build**: the session was at *urgent* budget when the review
completed, and forcing a `termlink-hub` recompile+test cycle at urgent risks
crossing the critical BLOCK mid-build. `horizon: later`; decompose one-bug-one-
task on pickup (both fixes are small — Finding 1 first).

### FINDING 1 — [MED] Fail-open on peer-credential extraction failure

`crates/termlink-hub/src/server.rs` ~623-628 (Unix accept branch). If
`PeerCredentials::from_tokio_stream` returns `Err`, the code logs
`debug!("...allowing connection")` and falls through; the connection is then
spawned (~649-658) with `Some(PermissionScope::Execute)` **unconditionally** —
the same-UID check (~612) is skipped entirely.
- **Failure mode:** a fail-open default — cred-extraction failure grants FULL
  Execute with no UID gate. Contradicts the Reliability directive ("no silent
  auth failures"). Real exploitability is near-zero (on Linux `SO_PEERCRED`
  effectively never fails for a connected `AF_UNIX` socket), which is why it is
  MED not HIGH, but a security default must fail CLOSED.
- **Fix:** on the `Err` arm, `continue` (reject the connection) — or, if a
  softer landing is preferred, spawn with `None` scope (unauthenticated →
  requires `hub.auth`). Reject is the correct fail-closed choice for a
  same-user-trust socket. Add a pure helper (e.g.
  `scope_for_unix_peer(cred_result) -> Option<PermissionScope>`) so the
  fail-closed decision is unit-testable without needing SO_PEERCRED to actually
  fail. Verification: `cargo test -p termlink-hub --lib` + a unit test asserting
  `Err`/mismatch → reject, same-UID → Execute.

### FINDING 2 — [MED] No TTL ceiling on `token create` (captured-token reuse window)

`crates/termlink-cli/src/commands/token.rs` ~62 + `cli.rs` ~4374-4376, and
`auth.rs` `create_token` (~314-366). Token expiry EXISTS and is enforced
(`auth.rs` ~350-352; default 1h), and a `nonce` field exists (`auth.rs` ~233,
286) — **good**. But `--ttl` is uncapped: `termlink token create --ttl
999999999` mints an effectively-permanent bearer token. The `nonce` is never
persisted or checked server-side, so there is no replay protection within the
token's validity window — a captured TCP/TLS-caller token is replayable until
`expires_at`.
- **Failure mode:** an unbounded TTL defeats the expiry safeguard for
  network callers; mostly moot for Unix/SO_PEERCRED callers. In-scope-leaning
  for the trusted-mesh model, but the missing ceiling is a real footgun.
- **Fix (small):** clamp `ttl_secs` at creation to a sane max (e.g.
  `TERMLINK_MAX_TOKEN_TTL_SECS`, default 24h or 7d) with a loud warn when
  clamped. Replay protection (server-side nonce cache) is a larger, separate
  effort — file only if the threat model tightens; note it, don't build blindly.

### Verified CLEAN (a clean bill is a valid result)

- **HMAC comparison is constant-time** — `auth.rs` ~335 uses
  `mac.verify_slice(...)` (hmac crate → `subtle` constant-time eq); no `==` on
  raw tags anywhere. No timing side channel.
- **No state-mutating RPC skips the scope check** — enforced centrally in
  `process_request_message` for BOTH transports; `hub_method_scope` maps every
  mutating method and falls back to `auth::method_scope` catch-all `_ => Execute`
  (deny-by-default). Regression test already asserts no leak to the catch-all.
- **No cross-transport scope bypass** — line and WS paths both call the single
  `process_request_message`; WS-only `hub.ws_subscribe` is itself auth-gated +
  rate-limited; WS push checks `granted_scope.is_some()`.
- **Notifications are not a bypass** — `router::route` returns `None` for
  notifications; a mutating method sent as a notification is dropped, not run.
- **Auth errors fail closed** — missing secret → AUTH_DENIED/internal; bad token
  → AUTH_DENIED; unauthenticated non-auth call → AUTH_REQUIRED. No permissive
  default in the token path (the only fail-open is Finding 1).
- **PeerCredentials UID check is sound** — exact `uid == owner_uid`; different-
  UID rejected; Execute only for same-UID (modulo Finding 1's Err path).
- **`token inspect` does not verify the signature** (only base64-decodes for
  display) — not a vuln (local read-only introspection); flagged only so it is
  never used as an authorization decision (it isn't).

## Acceptance Criteria

### Agent
- [x] (Tracking/capture task — no direct build ACs.) On pickup: build Finding 1 (fail-closed peer-cred, unit-tested) as its own task, then Finding 2 (TTL clamp) as its own task; close this tracker when both are filed/built. → **DONE:** Finding 1 = T-2448 (closed, 433 hub tests green), Finding 2 = T-2449 (closed, 390 session tests green).

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

### 2026-07-21T22:23:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2447-round-10-review--authtokenscope-security.md
- **Context:** Initial task creation

### 2026-07-21T22:23:27Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-07-22T05:54:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
