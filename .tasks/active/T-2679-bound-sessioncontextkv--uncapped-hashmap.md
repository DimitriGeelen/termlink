---
id: T-2679
name: "Bound SessionContext.kv — uncapped HashMap in the PTY-owning session daemon at Interact scope"
description: >
  Bound SessionContext.kv — uncapped HashMap in the PTY-owning session daemon at Interact scope

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
created: 2026-08-13T23:11:35Z
last_update: 2026-08-13T23:18:59Z
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

# T-2679: Bound SessionContext.kv — uncapped HashMap in the PTY-owning session daemon at Interact scope

## Context

`SessionContext.kv` (`crates/termlink-session/src/handler.rs:29`) is a plain
`HashMap<String, serde_json::Value>` with no cap on key count, no cap on value size, and
no eviction. `handle_kv_set` (`handler.rs:966`) does a bare `ctx.kv.insert(...)`. `kv.set`
requires only `Interact` scope (`crates/termlink-session/src/auth.rs:191`) — in a
cooperating fleet, every authenticated peer. The map lives in the **session daemon**, the
process that owns the real PTYs backing the charter's fourth verb ("control terminal
sessions"), so unbounded growth OOM-kills the operator's live terminal sessions.

Third instance of a class closed twice in the immediately preceding session — T-2675
(bounded `PresenceTracker`) and T-2676 (bounded circuit-breaker map). Found by the
T-2678 charter guard-coverage review (finding F3, IW-3).

**Loud refuse, not evict.** `kv` is a caller-visible store: silently evicting a key the
caller believes it set is a Directive #2 (no silent failures) violation. Follows the
established LOUD-refuse convention of `HUB_AT_CAPACITY` (T-2048 IW-3) and `QueueFull`
(T-2051 R3). Overwriting an existing key is always allowed — it cannot grow the key set —
so a well-behaved caller updating a bounded set of keys never trips the cap.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `error_code::KV_STORE_FULL` (-32023) added to `termlink-protocol` with doc comment matching the established style, and a constant-value assertion test
- [x] `SessionContext` carries env-tunable bounds (`TERMLINK_KV_MAX_KEYS` default 1000, `TERMLINK_KV_MAX_VALUE_BYTES` default 65536) with a `with_bounds`-style deterministic constructor for tests (parallel tests must not race on env, per T-2675 idiom)
- [x] `handle_kv_set` refuses a NEW key when the store is at `max_keys`, returning `KV_STORE_FULL` with `data: {reason:"max_keys", limit, current}` — and does NOT insert
- [x] `handle_kv_set` refuses an oversized value, returning `KV_STORE_FULL` with `data: {reason:"max_value_bytes", limit, current}` — and does NOT insert
- [x] Overwriting an EXISTING key at the key cap still succeeds (cap gates growth, not updates)
- [x] Refusal emits no `kv.change` event (a refused write must not look like a write)
- [x] Unit tests cover: key-cap refusal, value-cap refusal, overwrite-at-cap success, and that a refused set leaves the map unchanged
- [x] `cargo test -p termlink-session -p termlink-protocol` passes with no pre-existing test regressions

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

cargo test -p termlink-protocol --lib 2>&1 | tail -5
cargo test -p termlink-session --lib 2>&1 | tail -5
grep -q 'KV_STORE_FULL: i64 = -32023' crates/termlink-protocol/src/control.rs
grep -q 'TERMLINK_KV_MAX_KEYS' crates/termlink-session/src/handler.rs
grep -q 'TERMLINK_KV_MAX_VALUE_BYTES' crates/termlink-session/src/handler.rs

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

### 2026-08-13T23:11:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2679-bound-sessioncontextkv--uncapped-hashmap.md
- **Context:** Initial task creation
