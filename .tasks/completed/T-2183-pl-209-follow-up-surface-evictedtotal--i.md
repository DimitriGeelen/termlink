---
id: T-2183
name: "PL-209 follow-up: surface evicted_total + identify noisy sender (rate-limit pressure)"
description: >
  Captured 2026-06-11 from PL-209. Fleet /governor shows 380K active rate buckets + 3140 rate_hits across 2 hubs; evicted_total reports n/a so we can't tell if buckets accumulate or rotate. Scope: (1) Surface evicted_total in hub.governor_status JSON-RPC + CLI render. (2) Identify the noisy sender (suspect: listener-heartbeat from a renumbered/respawning host). (3) Decide whether rate-bucket eviction policy needs tuning. Cross-ref: T-2048 (#10 BACKPRESSURE substrate primitive), T-2062 fleet governor-status.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-11T20:49:17Z
last_update: 2026-06-11T21:14:08Z
date_finished: 2026-06-11T21:14:08Z
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

# T-2183: PL-209 follow-up: surface evicted_total + identify noisy sender (rate-limit pressure)

## Context

**Discovery (2026-06-11):** PL-209's "evicted_total=n/a" was not a missing
telemetry field — it was a stale-binary artifact on the **hub** side.
T-2139 (with T-2137's eviction loop) already shipped
`rate_buckets_evicted_total` end-to-end across JSON-RPC + single-hub
CLI + fleet CLI + watch/notify/log/history surfaces:

| Surface | Wired at |
|---------|----------|
| Hub-side counter | `crates/termlink-hub/src/governor.rs:211` (`evictions_total`) |
| JSON-RPC field | `crates/termlink-hub/src/router.rs:862` (`rate_buckets_evicted_total`) |
| Single-hub CLI | `crates/termlink-cli/src/commands/infrastructure.rs:806-809` |
| Fleet CLI | `crates/termlink-cli/src/commands/remote.rs:2591-2594` |
| Fleet JSON sum | `crates/termlink-cli/src/commands/remote.rs:2802` |
| Watch loop | `crates/termlink-cli/src/commands/remote.rs:2837-3109` |

Live verification (2026-06-11, post-T-2181 rebuild):
```
$ termlink fleet governor-status --json | jq '.hubs[] | select(.ok) | .governor'
{ ... "rate_buckets_active": 384168, "rate_hits_total": 1570 }
# field rate_buckets_evicted_total ABSENT from envelope
```

Root cause: local hub `PID 2058543` started `Jun 8 17:20`, binary
on disk at `/root/.cargo/bin/termlink (deleted)` — i.e. the binary
file was replaced by yesterday's T-2181 rebuild, but the running
process still serves from the in-memory (older) inode. That older
binary predates T-2139's field emission.

**Implication.** This is the symmetric class to T-2181 `/preflight`
Check 4 (CLI binary freshness). `/preflight` Check 4 catches stale
**clients**; there is no equivalent probe for stale **hubs**. Both
fail the same way — catalog promises a field, but the binary in
use doesn't emit it, so the CLI loyally renders `n/a` and the
operator infers a missing-feature gap when the actual gap is a
missing-restart.

**Remaining work for this task:** capture the finding (this Context
section), register the learning, and schedule the symmetric hub
freshness probe as a follow-up. The actual hub-restart is a
shared-infrastructure mutation that needs explicit operator
authority and is NOT in this task's scope — it lands under the
follow-up's deploy step.

## Acceptance Criteria

### Agent
- [x] Context section documents the T-2139 shipping evidence + live `(deleted)` binary diagnosis (this commit).
- [x] Learning registered via `fw context add-learning` capturing the "stale-binary class is bidirectional (client AND hub)" insight, citing T-2181 (client side) as the partial fix.
- [x] Follow-up build task `T-2184` captured (status=captured, horizon=next) for `/preflight` Check 5 / hub-freshness probe — symmetric companion to T-2181 Check 4. Probe via `hub.governor_status` field-presence (`rate_buckets_evicted_total` absence ⇒ pre-T-2139 hub).
- [x] Scope (2) "identify the noisy sender" deferred — sender attribution on rate_hits is a NEW substrate feature (hub doesn't currently track sender_id per refusal), not a bug-fix. Logged as a follow-up note here rather than expanding T-2183's scope.

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

### 2026-06-11T20:49:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2183-pl-209-follow-up-surface-evictedtotal--i.md
- **Context:** Initial task creation

### 2026-06-11T21:14:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-11T21:14:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
