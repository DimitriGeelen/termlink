---
id: T-2414
name: "doorbell confirmation window too tight for real agents — 90s default misses measured 98s replies"
description: >
  doorbell confirmation window too tight for real agents — 90s default misses measured 98s replies

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
created: 2026-07-17T11:08:39Z
last_update: 2026-07-17T11:08:39Z
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

# T-2414: doorbell confirmation window too tight for real agents — 90s default misses measured 98s replies

## Context

`agent-send.sh`'s delivery-confirmation window is ~90s: `max_rings=3` × `timeout=10`
(30s of ring polling) + `AGENT_SEND_GRACE_SECS=60` (T-2412 grace poll). That default
was picked by intuition, never against measured peer behaviour.

Measured live 2026-07-17 (two real claude-code peers on the .107 shared host, driven
via `agent-send.sh --to <agent>`):

| peer          | reply latency | in window (~90s)? |
|---------------|---------------|-------------------|
| `aef`         | 44s           | yes               |
| `sonnenstall` | 98s           | NO — missed by ~8s |

A real claude agent must finish its current churn before it reads the doorbell
(`aef` was observed mid-`Churned for 41s` when rung). 44-98s is normal, not pathological
— but the window sits right in the middle of that range, so a genuinely-answered
doorbell is coin-flip-reported as `FAILED — receiver never acked` and escalated to
`.woken-but-silent-canary.log`.

The consequence is the whole point of the arc: a false "silent peer" is exactly what
trains agents and operators to stop trusting the rail and fall back to passive waiting.
T-2413 fixed the matcher so a late reply is recognised; this fixes the window so it is
still being watched for when it lands. Both are prerequisites for "the doorbell works
across the whole fleet as intended".

Distinct from T-2413 (matcher blind to `msg_type=turn`, a correctness bug). This is a
default tuned against no data — the confirm path is correct but stops watching too soon.

## Acceptance Criteria

### Agent
- [x] `AGENT_SEND_GRACE_SECS` default raised to cover the measured p100 (98s) with margin
- [x] Total confirmation window (rings + grace) is >= 120s at defaults
- [x] The default is justified in-code by the measured latency data, not left as a bare number
- [x] `AGENT_SEND_GRACE_SECS=0` still disables the grace poll (pre-T-2412 give-up timing preserved)
- [x] Operator-overridable via env (`AGENT_SEND_GRACE_SECS=N`) — no hard-coding
- [x] Hermetic test asserts the default value and the 0-disables contract; suite passes

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

bash tests/agent-send-grace-window.sh
bash tests/wake-confirm-reply-match.sh
bash -n scripts/agent-send.sh

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

**Symptom:** A doorbell the peer genuinely answers is reported
`FAILED — receiver never acked` and escalated as woken-but-silent, because the peer
replied at 98s and the sender stopped watching at ~90s (measured live 2026-07-17,
peer `sonnenstall`).

**Root cause:** The confirmation window (`max_rings=3` × `timeout=10` + grace 60 = ~90s)
was chosen without measuring how long a real claude-code peer takes to answer. Measured
range is 44-98s, so the default lands mid-distribution and fails a coin-flip share of
genuinely-answered doorbells.

**Why structurally allowed:** Every prior test of this path was hermetic — fixtures
resolve instantly, so no test could ever observe that the window is too short. The one
number that mattered (real peer reply latency) was never measured, and nothing in the
suite forced a comparison between the default and observed field behaviour. The failure
mode only appears against a live agent that has to finish its current churn first, and
it presents as the peer's fault ("receiver never acked") rather than the sender's —
so it reads as someone else's bug and is never traced back to a tuning default.

**Prevention:** (1) The default now carries the measured latency table in-code, so the
next person changing it sees the evidence it must satisfy rather than a bare integer.
(2) `tests/agent-send-grace-window.sh` pins the default and asserts the total window
clears the measured p100 — lowering it below field-observed latency now fails the suite.
(3) The 0-disables contract is pinned so the escape hatch can't be refactored away.
Residual gap logged: hermetic tests still cannot detect window-vs-reality drift if peer
latency grows; the real detector is the woken-but-silent canary, which must be read as
"possible tuning drift", not only "peer is broken".

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

### 2026-07-17T11:08:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Note:** superseded by the 2026-07-17 entry below
- **Output:** /opt/termlink/.tasks/active/T-2414-doorbell-confirmation-window-too-tight-f.md
- **Context:** Initial task creation

### 2026-07-17 — window widened against measured field data

- **Measured, not guessed:** drove real doorbells to two live claude-code peers on the
  .107 shared host and timed the replies off the rail envelopes: `aef` 44s,
  `sonnenstall` 98s. Old window ~90s (3 rings x 10s + 60s grace) caught the first and
  missed the second by ~8s — a false `receiver never acked` + canary escalation on a
  doorbell the peer had genuinely answered.
- **Fix:** `AGENT_SEND_GRACE_SECS` default 60 -> 120 => ~150s total window, clearing the
  measured p100 by 52s. The measured table now lives in-code beside the number so the
  next person to touch it sees the evidence it must satisfy.
- **Guard proven, not assumed:** temporarily reverted the default to 60 in a scratch copy
  and re-ran `tests/agent-send-grace-window.sh` — it failed with exactly the intended
  message (`total window 90s does NOT clear measured p100 98s — re-measure before
  lowering`). The test is a real regression guard, not a rubber stamp.
- **Asymmetry that drove the choice:** erring long costs seconds of waiting; erring short
  costs a rail nobody believes. Bias long.
- **Suites:** `tests/agent-send-grace-window.sh` 10/10 ALL PASS;
  `tests/wake-confirm-reply-match.sh` 19/19 ALL PASS (T-2413 unaffected).
