---
id: T-2773
name: "Session server fails OPEN on peer-credential extraction failure — T-2448 hub hardening never migrated to the sibling"
description: >
  server.rs:239-242 allows the connection when PeerCredentials extraction errors ('graceful degradation'), while the hub's decide_unix_peer rejects the same case fail-closed (T-2448, from T-2447 F1). Same uid gate, opposite security posture. The session server also drops uid-mismatched peers silently (bare continue, line 231) — the defect T-2772 just fixed in the hub.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/server.rs, crates/termlink-session/src/auth.rs, crates/termlink-session/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T17:41:00Z
last_update: 2026-08-16T19:52:21Z
date_finished: 2026-08-16T19:52:21Z
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

# T-2773: Session server fails OPEN on peer-credential extraction failure — T-2448 hub hardening never migrated to the sibling

## Context

The same-uid gate on a local Unix socket is implemented **twice**, and the two
copies disagree about what to do when `SO_PEERCRED` extraction fails.

`crates/termlink-hub/src/server.rs` fails **closed** — T-2448 (from T-2447 F1)
made an unreadable peer credential a rejection, on the reasoning that "I cannot
tell who you are" is not a licence to grant `Execute`. T-2772 then made that
refusal legible to the refused party.

`crates/termlink-session/src/server.rs:239-242` fails **open**: the `Err` arm
logs at `debug!` and *allows the connection*, commented "graceful degradation on
unsupported platforms". The session server then assigns `Execute` scope when the
registration carries no `token_secret` — so on any platform where peer-credential
extraction fails, an unidentified peer receives full command-execution scope over
the session's control plane. That is the opposite posture on the identical gate.

It also carries the defect T-2772 just closed in the hub: a uid mismatch is a
bare `continue` (line 231), so the refused client reads only
`Connection reset by peer (os error 104)` and cannot distinguish a deliberate
policy refusal from a crashed server — Directive #2, a refusal nobody can read.

**The mechanism, not just the instance.** Nothing made the hub's hardening
propagate to its sibling, and nothing detects that the two have diverged. A
third copy of the policy would reproduce exactly that. `PeerCredentials` already
lives in `termlink-session/src/auth.rs` and the hub depends on that crate, so the
decision belongs there as ONE function both accept loops call — divergence
becomes impossible rather than merely discouraged.

## Acceptance Criteria

### Agent
- [x] The uid/peer-credential decision is expressed by a single shared pure
      function in `termlink-session/src/auth.rs` (beside `PeerCredentials`), and
      BOTH the hub accept loop and the session accept loop route their decision
      through it — no second policy implementation remains
- [x] The session accept loop rejects a connection when peer-credential
      extraction fails (fail-closed, matching T-2448); no code path remains that
      allows a connection whose peer identity is unknown
- [x] A peer refused by the session server receives a readable `AUTH_DENIED`
      JSON-RPC envelope before the stream closes, distinguishing the two causes
      (uid mismatch vs credentials-unavailable) — no bare `continue`
- [x] The refusal write cannot stall the session accept loop (a client that
      never reads must not block other connections) — the write is `tokio::spawn`ed,
      mirroring the hub's capacity refusal
- [x] Unit tests assert on what the CLIENT receives for both refusal causes on
      the session side, not only on server-side log emission
- [x] A test pins the shared-policy property: hub and session server produce the
      same decision for the same (creds, owner_uid) input
- [x] `cargo test --workspace` passes — 2026-08-16, exit 0, every suite `0 failed`
      (termlink-session 463 → 468 with the five new cases)
- [x] `bash scripts/run-guard-layer.sh` passes — 41/41 members clean

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

## Decisions

**Shared function, not a third copy.** The obvious minimal fix — change `Err(e) =>
allow` to `Err(e) => reject` in the session server — closes the instance and leaves
the mechanism untouched: two implementations of one security policy, still free to
drift the next time either is hardened. The repo's own convention (T-2069) permits
duplicating *tiny pure helpers* across crates, and that convention is the wrong
reach here; a security posture is exactly the thing that must not have two
opinions. `PeerCredentials` already lives in `termlink-session/src/auth.rs` and the
hub already depends on that crate, so the policy moved there and the hub's copy was
deleted rather than a second one added.

**The refusal builder moved too.** Its `data` shape (`reason` / `peer_uid` /
`owner_uid`) is a wire contract clients parse, so the same drift argument applies.
Only the human-readable remediation differs by endpoint, which is a `UnixEndpoint`
parameter — the hub offers authenticated TCP, a session control plane is local-only
and points at a capability token through the hub instead.

**Known limit of the test coverage — stated, not papered over.** The unit tests
prove the policy is fail-closed and prove what the refused client receives. They do
NOT prove the accept loop *calls* the policy; that is the same "coverage of a
builder says nothing about whether the builder is called" gap T-2699 documented.
Two things carry that weight instead: the Accept arm is exercised end-to-end by the
crate's existing socket tests (8+ call sites drive `run_accept_loop` and connect
same-uid), and the wiring is pinned by greps in `## Verification`. The Reject arm
cannot be integration-tested in-process — nothing in the test binary can be a
different uid, and `SO_PEERCRED` cannot be made to fail on Linux on demand. The
structural mitigation is that the `match` is now exhaustive over `UnixPeerDecision`
with no allow-arm to fall into: reintroducing fail-open requires writing new code,
not deleting a check.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The fail-open path must be gone: no code may allow a connection whose peer is unknown.
! grep -rn "allowing connection" crates/termlink-session/src/server.rs
# The policy must exist in exactly one place (auth.rs), and both servers must call it.
grep -q "pub fn decide_unix_peer" crates/termlink-session/src/auth.rs
grep -q "decide_unix_peer" crates/termlink-hub/src/server.rs
grep -q "decide_unix_peer" crates/termlink-session/src/server.rs
cargo test -p termlink-session --quiet
cargo test -p termlink-hub --quiet
bash scripts/run-guard-layer.sh
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

### 2026-08-16T17:41:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2773-session-server-fails-open-on-peer-creden.md
- **Context:** Initial task creation

### 2026-08-16T19:29:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-16T19:52:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
