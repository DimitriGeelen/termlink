---
id: T-2109
name: "agent.find_idle cv_index fast path — substrate primitives 2 + 9 cross-reference optimization"
description: >
  agent.find_idle cv_index fast path — substrate primitives 2 + 9 cross-reference optimization

status: work-completed
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
created: 2026-06-09T22:24:25Z
last_update: 2026-06-09T22:24:25Z
date_finished: 2026-06-09T22:45:59Z
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

# T-2109: agent.find_idle cv_index fast path — substrate primitives 2 + 9 cross-reference optimization

## Context

Cross-reference optimization between substrate primitive #2 (DISPATCH —
`agent.find_idle`, T-2020/T-2045) and substrate primitive #9
(BROADCAST-WITH-REPLAY — cv_index, T-2089/T-2103..T-2108).

**Current cost.** `Bus::find_idle_agents` (crates/termlink-bus/src/lib.rs:414)
calls `self.subscribe("agent-presence", 0)` — walks EVERY heartbeat envelope
on the topic since the hub started. For a fleet of N agents heartbeating
every 30s for H hours, that's N × H × 120 envelopes per RPC call. A 5-agent
fleet running 24h: 14,400 envelopes walked per `find_idle` invocation.

**Available data.** Since T-2107, every `listener-heartbeat.sh` heartbeat
tags `metadata.cv_key=$agent_id`. The hub's cv_index
(crates/termlink-hub/src/cv_index.rs) records `(agent-presence, agent_id) →
latest_offset` last-write-wins, capped per-topic. Reading
`cv_index().current_values("agent-presence")` returns `Vec<(agent_id, offset)>`
in O(N_agents).

**Fast path.** For each (agent_id, offset) entry: read the single envelope
at that offset, extract role+capabilities+ts, apply LIVE+role+capability
filters + claimer anti-join. O(N_agents) reads vs O(N_heartbeats) walk.

**Fallback.** cv_index is in-memory + process-local. After hub restart it
takes one heartbeat cycle (~30s) to repopulate. If cv_index is empty for
agent-presence (cold start, or no producers wired post-T-2107), fall back
to the existing full walk — backward-compat with legacy producers.

**Trade-off.** Producers that opt out (`--no-cv-key`) are invisible to the
fast path. Documented behavior since T-2107: opt-out is a test/migration
escape hatch, not the default. The fast path matches what producers chose
to advertise.

**Cross-primitive value.** This is the natural closure of the
"Related primitives" callout in
`docs/operations/substrate-broadcast-with-replay.md` — proving the cv_index
substrate can be CONSUMED by other primitives, not just inspected.

**Out of scope.** cv_index persistence across hub restarts (deliberate
deferral, captured in substrate-broadcast-with-replay.md). Optimizing
other find_idle consumers (find-idle-history, claims-summary). Adding
cv_index to non-presence topics' find_idle paths.

## Acceptance Criteria

### Agent
- [x] New primitive `Bus::envelope_at(topic, offset) -> Result<Option<Envelope>>` on the bus — returns the single envelope at `offset` or `None` if not present. Unit-tested for present/absent/unknown-topic cases.
- [x] Hub-side `handle_agent_find_idle_with` (crates/termlink-hub/src/channel.rs) tries the cv_index fast path when `cv_index().current_values("agent-presence")` returns non-empty; otherwise falls back to `bus.find_idle_agents(...)` (the existing walk).
- [x] Fast path applies the same role + capability + LIVE-window + claimer-anti-join filters as the walk path. Sort order (freshest first) + limit semantics preserved.
- [x] cv_index path skips entries whose envelope cannot be read (envelope deleted by retention sweep, etc) — falls through to walk for that entry, never panics.
- [x] Unit tests in bus crate covering: cv_index populated (fast path hit), cv_index empty (walk fallback equivalent — `find_idle_agents` tests already cover), cv_index-with-stale-offset (envelope swept — `find_idle_from_hint_skips_swept_offsets`), role/capability filters apply correctly on fast path, limit semantics identical. Hub-level integration: cv_index is a process-global singleton so per-test isolation is infeasible — coverage via live sidecar smoke (next AC) instead.
- [x] Hub regression test suite: full `cargo test -p termlink-hub` passes (351 tests, no regression on prior count).
- [x] Live sidecar smoke: spin a sidecar hub on a non-production port, populate cv_index via two simulated heartbeats with `cv_key=alpha` / `cv_key=beta`, call `agent.find_idle` via the CLI, verify both agents appear in the output. Then post a third heartbeat with `--no-cv-key`, call `agent.find_idle` — verify the third agent does NOT appear via the fast path (documented trade-off). PASS via /tmp/T-2109-smoke.sh.
- [x] `docs/operations/substrate-broadcast-with-replay.md` "Related primitives" section updated — find_idle now consumes cv_index directly (close the "future optimization candidate" callout).
- [x] CLAUDE.md Quick Reference row for `agent find-idle` updated to mention the cv_index fast path (one line).

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

cargo build -p termlink-bus -p termlink-hub -p termlink 2>&1 | tail -5 | grep -q "warning\|Compiling\|Finished"
cargo test -p termlink-bus envelope_at 2>&1 | tail -10 | grep -q "test result.*ok"
cargo test -p termlink-hub find_idle 2>&1 | tail -20 | grep -q "test result.*ok"
test -f docs/operations/substrate-broadcast-with-replay.md
grep -q "cv_index\|cv-keys\|broadcast-with-replay" CLAUDE.md

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

### 2026-06-09T22:24:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2109-agentfindidle-cvindex-fast-path--substra.md
- **Context:** Initial task creation
