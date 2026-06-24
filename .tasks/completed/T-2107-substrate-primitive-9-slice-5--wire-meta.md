---
id: T-2107
name: "substrate primitive 9 slice 5 — wire metadata.cv_key into listener-heartbeat.sh (T-2027 user payoff)"
description: >
  substrate primitive 9 slice 5 — wire metadata.cv_key into listener-heartbeat.sh (T-2027 user payoff)

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
created: 2026-06-09T21:23:58Z
last_update: 2026-06-09T21:23:58Z
date_finished: 2026-06-09T21:32:47Z
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

# T-2107: substrate primitive 9 slice 5 — wire metadata.cv_key into listener-heartbeat.sh (T-2027 user payoff)

## Context

Substrate primitive #9 (broadcast-with-replay) — heartbeat producer wiring.
**This is the USER PAYOFF slice** for the whole arc.

Slices 1-4 (T-2103/T-2104/T-2105/T-2106) shipped the hub-side cv_index +
the operator surfaces (subscribe `--include-current-value`, `channel cv-keys`).
But the hub doesn't populate cv_index automatically — producers have to
tag their posts with `metadata.cv_key`. Without producers wiring it, the
substrate is dead infrastructure.

This slice flips the switch on the most common producer: presence
heartbeats. Every agent that runs `listener-heartbeat.sh` (i.e. every
agent that called `/be-reachable`) now emits `metadata.cv_key=<agent_id>`.
The hub records `(agent-presence, <agent_id>) -> latest_offset` per agent.

User payoff:
- `/peers --all` and `agent-listeners.sh` currently walk the WHOLE
  `agent-presence` topic to find the latest heartbeat per agent. On a
  fleet of N agents emitting one heartbeat every 30s, after T seconds
  the topic has N*T/30 envelopes; LATEST-per-agent is O(N*T/30) read.
- After this slice: late-joiner calls `channel.cv_keys agent-presence`
  → O(N) — one entry per agent. Or `subscribe --include-current-value`
  → O(N) envelopes — one per agent's latest heartbeat.
- For a 5-agent fleet that's been running 24h: 14400 envelopes vs 5.
  Substrate primitive #9's value materializes.

Out-of-scope (later, T-2018 §6 #6/#8):
- Wiring cv_key into other broadcast producers (chat-arc, dialog-presence).
  presence is the highest-value first target.
- Persistence of cv_index across hub restarts (currently in-memory only).

## Acceptance Criteria

### Agent
- [x] `scripts/listener-heartbeat.sh` emits `metadata.cv_key=$agent_id` on every heartbeat post by default
- [x] New flag `--no-cv-key` opt-OUT (in case an operator wants the legacy non-cv-tagged behavior); default behavior is opt-IN
- [x] Help text describes the new flag and links to substrate primitive #9 (T-2027/T-2089/T-2103/T-2107)
- [x] Sidecar smoke (port 19997): 2 alpha heartbeats + 1 beta + 1 gamma --no-cv-key → `cv-keys agent-presence` returns `alpha -> @1` (last-write-wins over alpha @0), `beta -> @2`, count=2; gamma correctly absent (opt-out path)
- [x] Sidecar smoke: gamma's envelope (offset 3) has NO `cv_key` metadata (grep on `channel subscribe --json` confirms `0` matches); alpha's two envelopes both carry `cv_key=smoke-claude-alpha` (grep count = 2)
- [x] `cargo build` no-op (script change only)
- [x] Default behavior is purely additive — alpha/beta envelopes carry the original metadata fields (agent_id, role, listen_topics, etc.) PLUS the new cv_key field; existing heartbeat consumers see no behavioral change

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

grep -q "cv_key" scripts/listener-heartbeat.sh
grep -q "no-cv-key\|no_cv_key" scripts/listener-heartbeat.sh
bash -n scripts/listener-heartbeat.sh
out=$(bash scripts/listener-heartbeat.sh --help 2>&1); echo "$out" | grep -qi "cv.key"

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

### 2026-06-09T21:23:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2107-substrate-primitive-9-slice-5--wire-meta.md
- **Context:** Initial task creation
