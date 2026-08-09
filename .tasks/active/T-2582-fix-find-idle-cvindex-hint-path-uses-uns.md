---
id: T-2582
name: "fix: find-idle cv_index hint path uses unsigned cv_key as identity, surfacing ghost idle agents the walk path drops"
description: >
  find-idle cv_index hint fast-path takes agent_id from the unsigned cv_key and skips the walk path agent_id-presence guard; a malformed agent-presence post surfaces as a ghost idle dispatch target on the fast path only.

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
created: 2026-08-09T21:54:34Z
last_update: 2026-08-09T21:54:34Z
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

# T-2582: fix: find-idle cv_index hint path uses unsigned cv_key as identity, surfacing ghost idle agents the walk path drops

## Context

From the T-2468 charter verb-1 ("discover peers") adversarial hunt. `find_idle`
has two code paths that must return identical results: the authoritative walk
(`Bus::find_idle_agents`, `crates/termlink-bus/src/lib.rs:552`) and the cv_index
fast path (`Bus::find_idle_agents_from_hint`, lib.rs:661, taken when cv_index is
populated and below cap, T-2109). The walk derives each agent's identity from the
SIGNED envelope field `env.metadata.get("agent_id")` and DROPS any envelope lacking
it (lib.rs:583). The hint path takes the identity from the cv_index KEY (the
`cv_key`, an unsigned, caller-controllable value) and never inspects
`env.metadata.agent_id` (lib.rs:685-713). Net: `agent-presence` is a public topic —
a client can post `--metadata cv_key=ghost` with NO `agent_id`. cv_index records
`(agent-presence, ghost) → offset`; while the hint path is active, `find_idle`
resolves that offset and emits `ghost` as an idle dispatch target that the
authoritative walk path drops. Same fleet, same query, two answers depending on
hidden hub state (cv_index populated/below-cap) — a path-dependent discovery fork
and a "trust any poster" Reliability gap on the charter-core discover verb.

## Acceptance Criteria

### Agent
- [x] `find_idle_agents_from_hint` derives each agent's identity from the SIGNED
      `env.metadata.get("agent_id")` (exactly like the walk path), NOT the cv_index
      key — and `continue`s (drops the entry) when the envelope lacks `agent_id`.
- [x] A load-bearing unit test posts a normal heartbeat AND a ghost presence
      envelope (cv_key present, no `agent_id`), builds a hint covering both, and
      asserts the hint path returns ONLY the real agent — matching what
      `find_idle_agents` (walk) returns for the same state — proven to FAIL if the
      fix is reverted (temp-revert).
- [x] `cargo test -p termlink-bus --lib find_idle` passes (no regressions).

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
out=$(cargo test -p termlink-bus --lib find_idle_from_hint_ignores_ghost_without_signed_agent_id 2>&1); echo "$out" | grep -q "1 passed"
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

**Symptom:** `find_idle` (the "discover peers" verb) could return a "ghost" idle
agent — an identity that never legitimately heartbeated — as a dispatch target,
but only while the cv_index fast path was active; the authoritative walk path
dropped it. Same fleet, same query, two answers depending on hidden hub state.

**Root cause:** `find_idle_agents_from_hint` took each agent's identity from the
cv_index KEY (an unsigned `cv_key` that any client can set on a public
`agent-presence` post) instead of the SIGNED `env.metadata.agent_id`, and — unlike
the walk path — never dropped envelopes lacking `agent_id`.

**Why structurally allowed:** the T-2109 fast path was written to reproduce the
walk's FILTER chain (LIVE window, role, capability, claimer anti-join — all
verified identical) but its IDENTITY-source step diverged silently. No test posted
a malformed/ghost presence envelope through the hint path, so the two paths' agreement
was only ever tested on well-formed input.

**Prevention:** (1) the hint path now derives identity from `env.metadata.agent_id`
and drops entries lacking it, mirroring the walk exactly; (2) load-bearing test
`find_idle_from_hint_ignores_ghost_without_signed_agent_id` posts a ghost envelope
and asserts hint-path identities == walk-path identities — proven to FAIL on
temp-revert. The test compares the two paths' full identity SETS, so any future
identity-source divergence re-fails it.

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

### 2026-08-09T21:54:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2582-fix-find-idle-cvindex-hint-path-uses-uns.md
- **Context:** Initial task creation
