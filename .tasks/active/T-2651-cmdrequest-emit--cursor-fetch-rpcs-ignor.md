---
id: T-2651
name: "cmd_request emit + cursor-fetch RPCs ignore --timeout (naked rpc_call, no wall-clock
  bound)"
description: >
  execution.rs cmd_request uses naked client::rpc_call for the initial event.subscribe
  cursor-fetch (line 182) and event.emit (line 197). Both call Client::call which
  reads via unbounded next_line() with no timeout. The user's --timeout only governs
  the subscribe wait loop, so a wedged target session hangs cmd_request forever despite
  --timeout. Hardened sibling rpc_call_addr_with_timeout (T-2354) exists but was not
  used here.

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
created: 2026-08-12T19:14:20Z
last_update: '2026-08-18T18:58:39Z'
date_finished:
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2651: cmd_request emit + cursor-fetch RPCs ignore --timeout (naked rpc_call, no wall-clock bound)

## Context

Found in round-10 divergence hunt (verified in code). `cmd_request`
(crates/termlink-cli/src/commands/execution.rs) accepts a `--timeout` (seconds) that the
user expects to bound the whole request. But `--timeout` only governs the reply-wait loop
(lines ~228-240). The two RPCs BEFORE the wait loop use the naked, unbounded
`client::rpc_call`:

- line ~182: `event.subscribe` cursor-fetch (with `timeout_ms:1` in params, but that is a
  hub-side subscribe timeout — it does NOT bound the client's read of the response frame)
- line ~197: `event.emit`

`client::rpc_call` → `rpc_call_addr` → `Client::call`, which reads the response via
`next_line()` with NO wall-clock timeout (verified in crates/termlink-session/src/client.rs).
A hardened sibling `rpc_call_addr_with_timeout` (T-2354, uses `call_with_timeout`) already
exists — the emit path simply was not migrated to it.

**Failure scenario:** the target session is registered (socket exists, `find_session`
succeeds) but wedged — its accept loop is alive but the emit handler is blocked. The client
connects, sends `event.emit`, and blocks forever on the response read. The user passed
`--timeout 30` but the emit phase honors no timeout: the command hangs indefinitely.

**Why FILED, not auto-built:** async timeout wiring on a live RPC path. A load-bearing test
needs a hung-hub / slow-responder fixture (a mock socket that accepts then never replies) to
prove the timeout now fires — heavier than a pure-fn unit test. Related to T-2648
(`Client::call` unbounded `next_line()` — that caps response SIZE; this bounds response
TIME). Both stem from `Client::call` being naked; distinct fixes.

## Acceptance Criteria

### Agent
- [ ] `cmd_request`'s `event.emit` (line ~197) uses a timeout-bounded RPC (`rpc_call_addr_with_timeout` or an equivalent `rpc_call_with_timeout(socket_path, …)` convenience) derived from the user's `--timeout`
- [ ] The cursor-fetch `event.subscribe` (line ~182) is likewise bounded (or documented as intentionally best-effort with `Err(_) => None` — but the client read must still not hang forever)
- [ ] The total wall-clock a wedged target can hold `cmd_request` is bounded by `--timeout` (emit + wait, not just wait)
- [ ] Regression test with a mock socket that accepts then never responds proves `cmd_request` returns a timeout error within ~`--timeout`, not hangs
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

**Symptom:** `termlink request --timeout N` can hang far longer than N seconds (indefinitely)
when the target session accepts the connection but never responds to the emit RPC.

**Root cause:** `--timeout` is threaded only into the reply-wait loop. The preceding
`event.emit` and cursor-fetch `event.subscribe` use the naked `client::rpc_call`, whose
underlying `Client::call` reads the response frame via `next_line()` with no wall-clock
bound. So the timeout the user asked for does not cover the emit phase.

**Why structurally allowed:** a hardened `rpc_call_addr_with_timeout` (T-2354) exists, but
the pre-existing `cmd_request` emit path predates or diverged from it — a classic
un-migrated-sibling divergence. Nothing flags an RPC call-site that omits the timeout
variant.

**Prevention (beyond the fix):** consider a static check (sibling to T-2531 drain-sink /
T-2527 alloc-sink) that flags `client::rpc_call(` call-sites on user-timeout-bearing command
paths where the `_with_timeout` variant should be used — or make the non-timeout `rpc_call`
carry a sane default deadline so no call-site can hang unbounded. Cross-ref T-2648 (size cap
on the same read).

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

### 2026-08-12T19:14:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2651-cmdrequest-emit--cursor-fetch-rpcs-ignor.md
- **Context:** Initial task creation
