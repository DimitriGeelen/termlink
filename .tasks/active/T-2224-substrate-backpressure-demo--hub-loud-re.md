---
id: T-2224
name: "substrate BACKPRESSURE demo — hub loud-refuses at rate-limit/capacity (arc-001 #10 proof)"
description: >
  substrate BACKPRESSURE demo — hub loud-refuses at rate-limit/capacity (arc-001 #10 proof)

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T21:48:38Z
last_update: 2026-07-04T22:29:59Z
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

# T-2224: substrate BACKPRESSURE demo — hub loud-refuses at rate-limit/capacity (arc-001 #10 proof)

## Context

Feasibility investigation (originally filed as a build task for a 5th substrate
proof demo) into whether the governor backpressure path (#10) can be proven by
a self-contained smoke-gate demo, the way T-2211/2212/2214 prove the claim
primitive. Outcome: NO-GO — see Recommendation. Investigation value captured as
learning PL-218.


## Acceptance Criteria

### Agent
- [x] Rate-limit trigger path assessed from source: governor buckets key on `params.from` > `peer_addr` > `peer_pid` (server.rs:756) and `channel.post` never sets `params.from` (bus_client.rs::post_to_params), so sequential CLI posts each mint a fresh pid-keyed bucket — a shell demo structurally cannot accumulate to the limit (Recommendation §1)
- [x] HUB_AT_CAPACITY path assessed for smoke-gate fit: forceable via `TERMLINK_MAX_CONNECTIONS=N` + N+1 held subscribes, but capturing the Nth refusal in shell is timing-dependent — flaky-risk disqualifies it as a deterministic regression gate (Recommendation §2)
- [x] Existing-coverage audit: governor behavior already asserted by `governor.rs` unit tests (`rate_hits_total`, eviction) — a shell demo adds no coverage at that layer (Recommendation §3)
- [x] Recommendation section filled with NO-GO + three-point rationale + copy-pasteable Tier-0 decision route; no code shipped under this task
- [x] Investigation payoff captured as learning PL-218 (peer_pid keying is the PL-209 rate-bucket-bloat mechanism; identity-fingerprint keying named as the natural follow-up inception) — present in .context/project/learnings.yaml

### Human
- [ ] [RUBBER-STAMP] Record the inception decision (Tier-0, sovereignty-gated)
  **Steps:**
  1. `cd /opt/termlink && .agentic-framework/bin/fw inception decide T-2224 no-go --rationale "demo not LC; governor covered by unit tests; root-cause captured as PL-218"`
  **Expected:** Decision section populated with no-go + rationale; task ready for work-completed
  **If not:** Paste the command output — the gate message names the blocking section

## Recommendation

**Recommendation:** NO-GO on a backpressure/governor smoke-gate demo.

**Rationale:**
1. **Rate-limit is infeasible to trigger from sequential CLI posts.** The
   hub's rate governor keys buckets by `sender_key`, precedence
   `params.from` > `peer_addr` > `peer_pid` (`server.rs:756`). `channel.post`
   sets `sender_id` but NOT `params.from` (`bus_client.rs::post_to_params`),
   so on UDS the key falls through to `peer_pid`. Every CLI invocation is a
   new process → a fresh pid → a distinct rate bucket that never accumulates.
   A demo firing N sequential posts presents as N distinct senders and never
   hits the limit.
2. **The HUB_AT_CAPACITY path is feasible but flaky-risk.** It can be forced
   with `TERMLINK_MAX_CONNECTIONS=N` + N+1 held `channel subscribe`
   connections, but holding an exact connection count open and capturing the
   Nth refusal in shell is timing-dependent — a poor fit for a deterministic
   regression gate (a flaky smoke stage erodes trust in the whole suite).
3. **The governor is already covered** by `governor.rs` unit tests
   (`rate_hits_total` / eviction assertions) — the layer a shell demo would
   add little to.

**Investigation payoff (PL-218):** the `peer_pid` keying is the mechanism
behind PL-209's rate-bucket bloat — short-lived posters each mint a bucket
(`governor.rs:312` notes `rate_buckets_active=258_236` against a ~5-agent
fleet). T-2137 idle-TTL eviction mitigates but does not address the root. A
deeper fix — key UDS buckets on the T-1427 verified identity fingerprint
instead of the ephemeral pid — is a security tradeoff (client-asserted
`sender_id` is spoofable; `peer_pid` is kernel-trusted but ephemeral) and is
the natural follow-up inception if bucket bloat is judged worth fixing beyond
eviction.

**Decision route (Tier-0, human):**
`.agentic-framework/bin/fw inception decide T-2224 no-go --rationale "demo not LC; governor covered by unit tests; root-cause captured as PL-218"`

No new code was shipped under this task.


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

grep -q "PL-218" /opt/termlink/.context/project/learnings.yaml
grep -q "NO-GO on a backpressure/governor smoke-gate demo" /opt/termlink/.tasks/active/T-2224-substrate-backpressure-demo--hub-loud-re.md || grep -q "NO-GO on a backpressure/governor smoke-gate demo" /opt/termlink/.tasks/completed/T-2224-substrate-backpressure-demo--hub-loud-re.md

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

### 2026-06-13T21:48:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2224-substrate-backpressure-demo--hub-loud-re.md
- **Context:** Initial task creation

### 2026-06-13T21:52:00Z — status-update [task-update-agent]
- **Change:** workflow_type: build → inception

### 2026-07-04T22:15:00Z — placeholder-ACs-filled [agent]
- **Action:** Replaced the two template placeholders with 5 evidence-anchored Agent ACs (all satisfied by the 2026-06-13 investigation) + a [RUBBER-STAMP] Human AC carrying the Tier-0 decide command — unblocks the Watchtower review/decide flow that errored on placeholders
- **Blocked follow-up:** committing this file requires the C-001 research artifact (docs/reports/T-2224-*.md), whose write the budget gate blocked at session end. FULL DRAFT STAGED at `.context/working/T-2224-artifact-draft.md` — next session: cp to `docs/reports/T-2224-backpressure-demo-inception.md` (strip the leading comment block), rm the draft, commit together with this file

### 2026-07-05T00:00:00Z — c001-artifact-landed [agent]
- **Action:** Promoted the staged draft to `docs/reports/T-2224-backpressure-demo-inception.md` (comment block stripped, draft removed) — C-001 commit gate satisfied; committed together with the AC fill above
- **Context:** Post-compaction budget reset re-allowed docs/ writes; task now fully decide-ready (rec NO-GO, command in the Human AC)
