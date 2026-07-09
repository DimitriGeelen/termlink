---
id: T-2387
name: "Waker-liveness canary — surface 'host reachable but no push-waker process' (comms loud-contract observability; arc-004 shipped-neq-live guard, T-2380 E4/F3, G-069)"
description: >
  Daily canary (empty-log=healthy convention) that fires when a host advertises presence/reachability but has zero running push-waker process — the arc-004 dark-in-field class. Prevents shipped-neq-live recurrence for the comms rail.

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
created: 2026-07-09T09:29:24Z
last_update: 2026-07-09T23:14:11Z
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

# T-2387: Waker-liveness canary — surface 'host reachable but no push-waker process' (comms loud-contract observability; arc-004 shipped-neq-live guard, T-2380 E4/F3, G-069)

## Context

G-069 class for the comms rail: arc-004 push-wake shipped and live-proven (T-2388),
but "0 wakers fleet-wide" sat dark for weeks with nothing firing (T-2380 E4/F3 —
the operator only found out by asking "why is there still no response?"). The
load-bearing signal already exists: **`metadata.pty_session` on the agent-presence
heartbeat = the waker-running signal** (T-1834/T-2385 — be-reachable's PTY-bound
heartbeat sets it; a LIVE presence WITHOUT it is exactly "reachable but unwakeable",
breakpoint #2). `scripts/agent-listeners.sh --json` already surfaces `pty_session`
per listener. Second local signal: a `~/.termlink/be-reachable*.state` file whose
pushwaker pid is dead = the waker died silently after arming. Ship the standing
guard as the ninth empty-log=healthy daily canary (pattern: T-2359 fleet-binary /
T-2239 frozen-husk).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/check-waker-liveness-freshness.sh` exists and classifies two firing classes: (a) **LIVE-but-unwakeable** — any LIVE agent-presence listener on the local hub lacking `metadata.pty_session` (read via `scripts/agent-listeners.sh --json`); (b) **dead-waker** — any local `~/.termlink/be-reachable*.state` whose recorded pushwaker pid is not alive (confirmed a waker process via /proc cmdline, guarding pid-recycle per the T-2239 pattern). Exit 0 = healthy / 1 = firing / 2 = tooling error (hub unreachable ≠ firing — informational, PL-219).
- [x] Third firing class (c) **rail-dark**, opt-in via `--expect-armed` (the cron on .107 passes it): fire when ZERO LIVE listeners carry `pty_session` at all — the literal G-069 "0 wakers fleet-wide" observed state, which class (a) alone cannot see (no LIVE listeners → nothing to flag). Without the flag, an empty rail is informational only (hosts that legitimately run no agents stay quiet).
- [x] `--quiet` prints only on firing (cron-friendly); `--json` emits a jq-friendly envelope carrying per-class entries + counts (`unwakeable[]`, `dead_wakers[]`); test hook `TERMLINK_WAKER_TEST_JSON=<file>` feeds canned listeners JSON for hub-independent verification (PL-213 convention), and a `TERMLINK_WAKER_STATE_DIR` override points the state-file scan at a fixture dir.
- [x] Daily cron shipped: `.context/cron/waker-liveness-canary.crontab` runs the script `--quiet` appending to `.context/working/.waker-liveness-canary.log` (empty-log = healthy, `/canaries` auto-discovers) + touches the `.heartbeat` companion; crontab **installed to /etc/cron.d** on .107 (pre-push audit requirement, PL-173 class).
- [x] Both firing classes proven via the test hooks: a canned LIVE-no-pty_session listener fires class (a); a fixture state file with a dead pid fires class (b); an all-green fixture (LIVE with pty_session, alive waker pid) exits 0.
- [x] CLAUDE.md canary section gains the waker-liveness entry (ninth empty-log=healthy canary, paired with the existing eight); `bash -n` passes on the script.

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

bash -n scripts/check-waker-liveness-freshness.sh
# class (a) LIVE-no-pty fires (exit 1) — self-contained fixture, PL-213 test hooks
t=$(mktemp -d) && printf '{"ok":true,"listeners":[{"agent_id":"x","status":"LIVE","age_secs":5,"pty_session":null,"identity_fingerprint":"ff","host":"h"}]}' > "$t/l.json" && ! TERMLINK_WAKER_TEST_JSON="$t/l.json" TERMLINK_WAKER_STATE_DIR="$t" bash scripts/check-waker-liveness-freshness.sh --no-heartbeat >/dev/null 2>&1
# class (b) dead-waker fires (exit 1)
t=$(mktemp -d) && printf '{"ok":true,"listeners":[]}' > "$t/l.json" && printf '{"agent_id":"d","pid":999999,"pushwaker_pid":999999,"pty_session":"d"}' > "$t/be-reachable-d.state" && ! TERMLINK_WAKER_TEST_JSON="$t/l.json" TERMLINK_WAKER_STATE_DIR="$t" bash scripts/check-waker-liveness-freshness.sh --no-heartbeat >/dev/null 2>&1
# class (c) rail-dark fires only WITH --expect-armed
t=$(mktemp -d) && printf '{"ok":true,"listeners":[]}' > "$t/l.json" && ! TERMLINK_WAKER_TEST_JSON="$t/l.json" TERMLINK_WAKER_STATE_DIR="$t" bash scripts/check-waker-liveness-freshness.sh --no-heartbeat --expect-armed >/dev/null 2>&1 && TERMLINK_WAKER_TEST_JSON="$t/l.json" TERMLINK_WAKER_STATE_DIR="$t" bash scripts/check-waker-liveness-freshness.sh --no-heartbeat >/dev/null 2>&1
# all-green healthy (exit 0) + json envelope carries ok:true
t=$(mktemp -d) && printf '{"ok":true,"listeners":[{"agent_id":"a","status":"LIVE","age_secs":5,"pty_session":"a","identity_fingerprint":"aa","host":"h"}]}' > "$t/l.json" && out=$(TERMLINK_WAKER_TEST_JSON="$t/l.json" TERMLINK_WAKER_STATE_DIR="$t" bash scripts/check-waker-liveness-freshness.sh --no-heartbeat --json 2>&1); echo "$out" | grep -q '"ok": true'
# cron: git-tracked source byte-identical to installed /etc/cron.d copy (PL-173 class)
cmp .context/cron/waker-liveness-canary.crontab /etc/cron.d/termlink-waker-liveness-canary
# CLAUDE.md documents the ninth canary
grep -q "Waker-liveness canary (T-2387" CLAUDE.md

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

### 2026-07-09T09:29:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2387-waker-liveness-canary--surface-host-reac.md
- **Context:** Initial task creation

### 2026-07-09T23:14:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
