---
id: T-2772
name: "Hub silently drops uid-mismatched Unix connections — client sees bare ECONNRESET,
  not a refusal"
description: >
  Hub silently drops uid-mismatched Unix connections — client sees bare ECONNRESET,
  not a refusal

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T17:25:00Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-16T17:49:16Z
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
  - ts: '2026-08-18T18:56:59Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2772: Hub silently drops uid-mismatched Unix connections — client sees bare ECONNRESET, not a refusal

## Context

`run_accept_loop` (crates/termlink-hub/src/server.rs) rejects a uid-mismatched Unix
peer with `tracing::warn!` + `continue`. `continue` drops the accepted stream with
nothing written, so the client sees a bare `Connection reset by peer (os error 104)`
and is told nothing about WHY. The refusal is a deliberate T-2448 fail-closed policy
decision, but it is indistinguishable on the wire from a crashed hub, a half-open
socket, or a protocol fault.

The very next block in the same function proves the correct shape exists: the T-2048
connection-cap refusal writes one envelope (`write_capacity_refusal`) before closing,
commented "LOUD refuse per IW-3 — write one envelope, close socket". So the uid gate
is the odd one out, ~10 lines above a working example.

**Measured live on .107, 2026-08-16 — three consecutive wrong diagnoses.** The AEF
agent (Codex, `/opt/999-Agentic-Engineering-Framework`) started its own hub as root
(PID 3869961) and then queried it as `dimitri-mint-dev`. Its own hub refused it. The
agent concluded in turn: (1) "the agent-channel authorization is still broken",
(2) "current runtime permissions are now correct ... the remaining fault is inside the
running hub's channel RPC handling". The connection never reaches RPC handling — it is
dropped in the accept loop before a single byte is parsed. A silent refusal did not
merely fail to help; it actively produced three confident wrong answers, which is
Directive #2's stated harm ("no silent failures") in its exact form.

Two distinct gates produce two distinct errors, and only the first is self-explaining:

| gate | mechanism | client sees |
|---|---|---|
| socket file mode | kernel denies `connect()` (needs write) | `Permission denied (13)` |
| `decide_unix_peer` | hub accepts, checks `SO_PEERCRED`, refuses, `continue` | `Connection reset (104)` |

Scope: this task makes the SECOND gate self-explaining. It does NOT change the
same-uid policy itself (that is T-2771's question) and does NOT touch the socket mode.

## Acceptance Criteria

### Agent
- [x] A uid-mismatched Unix peer receives a structured refusal envelope naming the
      cause before the connection closes — not a bare reset
- [x] The refusal names BOTH uids (peer and hub owner), so the reader can see it is a
      mismatch rather than a generic denial
- [x] The refusal carries actionable remediation text: run the client as the hub's uid,
      or start a hub under the client's own uid
- [x] The credential-extraction-failure branch (`uid_mismatch: None`) also refuses
      loudly, with its own distinct cause text — fail-closed stays fail-closed
- [x] The same-uid accept path is unchanged (no new refusal on the happy path)
- [x] Unit tests cover: uid-mismatch envelope content, cred-failure envelope content,
      and that both name a remediation
- [x] `cargo test --workspace` passes — 3595 passed, 0 failed, exit 0 (2026-08-16)

### Live end-to-end proof (2026-08-16)

The three unit tests assert on `build_uid_refusal`'s output. **That is builder
coverage, and this task's own RCA quotes T-2699 on why builder coverage proves
nothing about whether the builder is CALLED** — the exact trap that left
`PROTOCOL_VERSION_TOO_OLD` unwired with a passing test. A unit test cannot reach the
accept-loop call site, because the uid gate reads the real `getuid()`. So the call
site was proven against a live hub instead, using the two uids present on .107:

```
# scratch hub as root (uid 0) on an isolated runtime_dir; socket mode opened to
# 0777 so the FILE gate cannot mask the UID gate under test
TERMLINK_RUNTIME_DIR=/tmp/t2772-proof ./target/debug/termlink hub start

# client as dimitri-mint-dev (uid 1000)
$ su - dimitri-mint-dev -c 'TERMLINK_RUNTIME_DIR=... termlink channel list'
Error: Hub returned error for channel.list: JSON-RPC error -32010:
  Connection refused: peer uid 1000 does not match hub owner uid 0. This hub
  accepts local Unix connections only from its own uid. Either run the client as
  uid 0, or start a separate hub under uid 1000.

# same-uid happy path, same hub, unchanged
$ TERMLINK_RUNTIME_DIR=... ./target/debug/termlink channel list
  broadcast:global  [messages:1000]          # RC=0
```

Before this change the first command returned `Connection reset by peer (os error
104)` with no further information. Scratch hub stopped and `/tmp/t2772-proof`
removed; the two pre-existing hubs on this host (PIDs 3093442, 3869961) were
confirmed untouched before and after.

**Residual, stated rather than hidden:** no automated test covers the call site. If
someone reverts the `tokio::spawn(write_uid_refusal(...))` back to a bare `continue`,
the three unit tests still pass. Closing that needs either a uid-parameterised accept
loop or a privileged integration test; noted in T-2771 IW-5 as part of the "refusal
the refused party cannot read" detector rather than bolted on here.

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

cargo test -p termlink-hub --lib 2>&1 | tail -5
out=$(cargo test -p termlink-hub --lib uid_refusal 2>&1); echo "$out" | grep -q "test result: ok"
grep -q "write_uid_refusal" crates/termlink-hub/src/server.rs

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

**Symptom:** A `termlink` client whose uid differs from the hub's gets
`Connection reset by peer (os error 104)` on every call, with no indication that a
policy refused it. Observed live: the AEF agent misdiagnosed it three times in a row,
landing on "the fault is inside the running hub's channel RPC handling" — a component
the connection never reaches.

**Root cause:** `run_accept_loop` handles the `UnixPeerDecision::Reject` arm with
`tracing::warn!` followed by bare `continue`. The warn goes to the HUB's log; the
client gets an unexplained TCP-level reset. Server-side observability was mistaken for
client-side observability — the refusal is loud in the one place the person diagnosing
it cannot see.

**Why structurally allowed:** T-2448 correctly changed the uid gate to fail CLOSED and
was measured on the security property (does it reject?), not on the reporting property
(does the rejected party learn why?). No test asserted anything about what the client
receives, because "connection dropped" has no envelope to assert on. The adjacent
capacity gate does it correctly, so this was an inconsistency inside one function
rather than a missing convention — the convention existed and was not applied.

**Prevention:** unit tests that assert on the refusal envelope's CONTENT (both uids
present, remediation present) for both reject branches. These fail if a future change
reverts to a bare `continue`, because the envelope simply would not be produced.
Broader class — "a refusal the refused party cannot read" — is a candidate detector
for the guard layer, noted in T-2771 IW-5 rather than built here.

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

### 2026-08-16T17:25:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2772-hub-silently-drops-uid-mismatched-unix-c.md
- **Context:** Initial task creation

### 2026-08-16T17:49:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
