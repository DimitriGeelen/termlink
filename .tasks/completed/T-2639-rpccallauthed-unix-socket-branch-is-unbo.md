---
id: T-2639
name: "rpc_call_authed unix-socket branch is unbounded — bound the read like the TCP branch (T-2354 divergence)"
description: >
  channel.rs rpc_call_authed: the unix branch returns unbounded rpc_call_addr while the TCP branch adopts call_with_timeout + TERMLINK_RPC_READ_TIMEOUT_SECS (T-2354). Every LOCAL channel op flows through the unbounded branch; a starved local hub hangs the whole local channel surface. Divergence F2 (T-2637).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, crates/termlink-session/src/client.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T12:36:43Z
last_update: 2026-08-12T14:09:18Z
date_finished: 2026-08-12T14:09:18Z
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

# T-2639: rpc_call_authed unix-socket branch is unbounded — bound the read like the TCP branch (T-2354 divergence)

## Context

Round-6 (T-2637) divergence-class finding F2 — the sibling of T-2638 on the local
RPC path. Verified sites (2026-08-12):
- `crates/termlink-cli/src/commands/channel.rs:359` — the unix branch of
  `rpc_call_authed` does `return client::rpc_call_addr(addr, method, params).await;`
  (UNBOUNDED read).
- `crates/termlink-cli/src/commands/channel.rs:401-419` — the TCP branch adopts
  `c.call_with_timeout(method, …, read_timeout)` with `read_timeout` from
  `TERMLINK_RPC_READ_TIMEOUT_SECS` (default 30s, clamped 1..=600) and carries the
  T-2354 comment naming the exact hazard.
The T-2354 comment's own field symptom (`channel info/unread --hub` hanging on a
wedged record-walk) runs against the LOCAL hub over the unix socket too — i.e. the
branch that most needs the bound is the one that lacks it. Every local channel op
and every local watch loop (`claims-summary --watch`, `cv-keys --watch`, …) reaches
the hub via this unbounded branch.

Filed (not built inline) because: the fix requires a bounded connect-then-
`call_with_timeout` form on the unix path (the convenience `rpc_call_addr` used
today has no bounded twin yet — `rpc_call_addr_with_timeout` is proposed in T-2635),
it is the hot path for the ENTIRE local channel surface (high blast radius / wire
behavior), and proving it needs a black-hole unix-socket harness. Coordinate with
T-2635 (shared bounded-primitive need) and T-2638 (already-shipped sibling).

## Acceptance Criteria

### Agent
- [x] The unix-socket branch of `rpc_call_authed` (channel.rs ~359) bounds its RPC read with the SAME `TERMLINK_RPC_READ_TIMEOUT_SECS` convention (default 30s, clamped 1..=600) the TCP branch uses — no local channel RPC can await a response line indefinitely. **Done:** extracted `rpc_read_timeout()` (channel.rs) shared by BOTH branches; the unix branch now routes through `rpc_call_addr_with_timeout(addr, method, params, rpc_read_timeout())`.
- [x] A shared bounded convenience (`rpc_call_addr_with_timeout`, coordinate with T-2635) is used rather than re-implementing the connect+call_with_timeout dance inline; if T-2635 lands first, adopt its primitive. **Done:** built `rpc_call_addr_with_timeout` in `termlink-session/src/client.rs` (bounds connect via `connect_addr_with_timeout` + read via `call_with_timeout`). This is the shared primitive T-2635/T-2640 will also adopt.
- [x] Peer-cred trust on the unix socket is preserved (the bound is orthogonal to auth — the unix branch skips token auth by design; that behavior is unchanged). **Done:** the unix branch still skips `hub.auth`; only the read bound was added.
- [x] A black-hole-server test (accepts the unix connection but never writes a response line) proves a local channel RPC returns a timeout error within the bound instead of hanging. **Done:** `channel::tests::rpc_call_authed_unix_branch_is_bounded` (channel-level) + `client::tests::rpc_call_addr_with_timeout_bounds_a_silent_server` (primitive-level). Both proven load-bearing via temp-revert (unbounded → outer 5s guard trips at exactly 5.0s).
- [x] `cargo test -p termlink` green; `cargo build -p termlink` succeeds.

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

cargo test -p termlink-session --lib client::tests::rpc_call_addr_with_timeout
cargo test -p termlink --bins channel::tests::rpc_call_authed_unix_branch_is_bounded
cargo build -p termlink

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

**Symptom:** Any local channel op (`channel info/unread/subscribe/…`) and any local
watch loop hangs indefinitely when the local hub accepts the unix connection but
never writes a response line (blocking-pool starvation mid record-walk — T-2258).
The CLI presents as frozen with no error.

**Root cause:** `rpc_call_authed` bounds the read on only ONE of its two branches.
The TCP branch adopts `call_with_timeout` (T-2354); the unix branch returns the
unbounded `rpc_call_addr`. Since all local traffic is unix, the bound added for
T-2354 does not cover the path it was motivated by.

**Why structurally allowed:** the T-2637 divergence class — a bound was added to one
branch of one funnel and skipped on the sibling branch. The T-2354 change reasoned
about TCP hubs (`--hub <tcp>`) and did not audit the local path; no test exercised
the unix branch against a silent server.

**Prevention:** a single bounded convenience (`rpc_call_addr_with_timeout`) both
branches share, plus a black-hole-unix-socket regression test. The T-2637 sweep is
the systematic detector; T-2638 already closed the find-idle sibling and T-2635 the
BusClient flush sibling — this closes the local channel funnel.

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

### 2026-08-12T12:36:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2639-rpccallauthed-unix-socket-branch-is-unbo.md
- **Context:** Initial task creation

### 2026-08-12T12:39:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-32b15f42
- **Timestamp:** 2026-08-12T14:10:27Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — A shared bounded convenience (`rpc_call_addr_with_timeout`, coordinate with T-2635) is used rather than re-implementing the connect+call_with_timeout dance inline; if T-2635 lands first, adopt its pri
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink-session/src/client.rs in: A shared bounded convenience (`rpc_call_addr_with_timeout`, coordinate with T-2635) is used rather than re-implementing the connect+call_with_timeout `

### 2026-08-12T14:09:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
