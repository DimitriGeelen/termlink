---
id: T-2314
name: "arc-004 build: active reconnect-to-WS with backoff (T-2311 GO, Option B RB1+RB2)"
description: >
  arc-004 build: active reconnect-to-WS with backoff (T-2311 GO, Option B RB1+RB2)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:push-transport]
arc_id: push-transport
components: [crates/termlink-cli/src/commands/channel.rs, scripts/demo-ws-push.sh]
related_tasks: [T-2311, T-2309, T-2313, T-2310]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-02T20:10:35Z
last_update: 2026-07-02T20:45:56Z
date_finished: 2026-07-02T20:45:56Z
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

# T-2314: arc-004 build: active reconnect-to-WS with backoff (T-2311 GO, Option B RB1+RB2)

## Context

arc-004 `push-transport` follow-on. T-2309 (S3b) made `channel subscribe <topic> --push`
try the WebSocket **once** and then permanently degrade to the 1 s poll floor on any
drop — a long-lived agent that sees one transient blip runs ~10× slower forever with no
recovery short of a process restart. T-2311 inception recorded **GO** for Option B
(alternating reconnect loop with poll catch-up). This task builds it.

Design: `docs/reports/T-2311-arc-004-active-reconnect-inception.md`. Two slices:
- **RB1** — outer reconnect loop with exponential backoff + jitter + cap around
  `run_ws_push`; a healthy session resets the backoff; symmetric "back on push" notice.
- **RB2** — one poll catch-up pass from the durable cursor on each drop (drains the gap,
  advances the cursor → no missed events, no double-render); scripted-drop wire evidence.

Invariant preserved: the durable poll path stays authoritative (WS is a faster transport,
never a source of truth). Unix hubs are unaffected (still connect over the raw socket).

## Acceptance Criteria

### Agent
- [x] `ws_reconnect_backoff` is a pure fn (exponential base×2^(n-1), capped, jittered) with a unit test covering growth, cap, and jitter bounds
- [x] The `--push` branch wraps `run_ws_push` in a reconnect loop: on drop it runs one poll catch-up pass from the durable cursor, backs off, and retries the WS; it settles on the steady poll loop only after the failure cap
- [x] A healthy WS session (ran ≥ threshold) resets the backoff attempt counter so the cap fires only on consecutive fast failures, not on long stable sessions
- [x] The catch-up pass drains the gap from the durable cursor and advances it, so no event is missed after a blip and gap events are not re-drained on every subsequent reconnect cycle (RB2); brief WS↔catch-up overlap on the first drop is the documented acceptable degrade ("correctness over dedup")
- [x] `scripts/demo-ws-push.sh` gains a scripted-drop + restart segment proving post→push RESUMES after a hub blip (reconnected notice + a post-restart DM delivered) and the catch-up gap-drain runs without runaway re-emission
- [x] `cargo test -p termlink-cli` (backoff + existing channel tests) and `cargo test -p termlink-session ws_` pass; `cargo build --release -p termlink` exits 0
- [x] Unix-hub `--push` behaviour is unchanged (`scripts/demo-ws-push-unix.sh` still passes)

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

cargo test -p termlink ws_reconnect_backoff
cargo test -p termlink-session ws_
cargo build --release -p termlink

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

### 2026-07-02 — reset-on-healthy-session instead of connect-signalling
- **What changed:** The reconnect cap must not fire on a long-lived agent that had
  many successful sessions punctuated by occasional drops — only on consecutive fast
  failures against a genuinely down hub. Rather than plumb a "did it connect?" signal
  out of `run_ws_push`/`stream_ws_events` (bigger change to the session crate), the
  loop measures session wall-time via `Instant`: a session that lasted ≥ a threshold
  is deemed healthy and resets the attempt counter to 0.
- **Plan impact:** RB1's "backoff/cap" is refined — cap counts *consecutive* fast
  failures, not lifetime reconnects. No session-crate change needed.
- **Triggered:** none — kept within T-2314.

## Decisions

### 2026-07-02 — Option B (alternating reconnect + catch-up) over Option A (concurrent floor)
- **Chose:** Reconnect loop that alternates WS ↔ one poll catch-up pass, per T-2311 GO.
- **Why:** Bounded consumer-loop change; reuses `run_ws_push` + a small catch-up helper;
  preserves the no-miss guarantee by draining the gap from the durable cursor and
  advancing it. No new transport, no protocol/hub change.
- **Rejected:** Option A (continuous concurrent poll floor + WS accelerator with
  cross-stream dedup) — best UX but two renderers of the same events and a double-print
  surface; heavier than a v1 warrants. Option C (stay-on-poll + periodic probe) — an
  inverted B with no advantage.

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

### 2026-07-02T20:10:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2314-arc-004-build-active-reconnect-to-ws-wit.md
- **Context:** Initial task creation

### 2026-07-02T20:45:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
