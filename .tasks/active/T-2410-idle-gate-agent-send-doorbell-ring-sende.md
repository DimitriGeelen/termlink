---
id: T-2410
name: "idle-gate agent-send doorbell ring (sender-side Stage-3, symmetric with recipient waker)"
description: >
  idle-gate agent-send doorbell ring (sender-side Stage-3, symmetric with recipient waker)

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
created: 2026-07-12T16:55:03Z
last_update: 2026-07-12T16:55:03Z
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

# T-2410: idle-gate agent-send doorbell ring (sender-side Stage-3, symmetric with recipient waker)

## Context

`agent-send.sh`'s doorbell ring does a **blind** `termlink inject <session> "/check-arc
respond …" --enter` on the LOCAL path (lines ~518-522), with no PTY state probe. The
*recipient's* own waker (`be-reachable-pushwaker.sh`, T-2402 Stage-3) is idle-gated —
it probes READY/BUSY/UNKNOWN and defers rather than injecting blind — but the
*sender-initiated* ring is not. Result reproduced live 2026-07-12 (T-2409 follow-on):
an `agent-send` to a busy peer (workflow-designer, heartbeat fresh but mid-turn)
blind-injects `/check-arc respond` into its active input → discarded → woken-but-silent
(escalated correctly). Two harms: (1) a blind inject can **corrupt a busy peer's
in-progress input**; (2) a peer that is only BRIEFLY busy (between turns) is missed
even though waiting one probe cycle would land the doorbell cleanly. Fix: make the
sender's ring **symmetric** with the recipient waker — probe PTY state before each
local inject, inject only at READY, defer on BUSY/UNKNOWN — reusing the exact T-2402
primitive (`pushwaker_probe_pty`) via `BE_REACHABLE_PUSHWAKER_LIB=1` sourcing. NOT a
fix for the persistently-busy case (that is the operator-held design fork, PL-253 /
T-2396 — an interactive agent is not an instant responder); this hardens the
briefly-busy + input-safety cases within the current design. See
[[project_comms_loud_contract]].

## Acceptance Criteria

### Agent
- [x] On the LOCAL doorbell path (`ring_remote=0`), `agent-send.sh` probes the peer PTY
      state via the reused T-2402 `pushwaker_probe_pty` (sourced with
      `BE_REACHABLE_PUSHWAKER_LIB=1`) before each ring's inject: injects only when
      `READY`; on `BUSY`/`UNKNOWN` it **does not** blind-inject that ring (avoids
      corrupting a busy peer's input) and logs a deferral, then still runs the existing
      wake-confirm receipt poll.
- [x] Fail-safe / no-regression: the REMOTE path (`remote inject`) is unchanged; on the
      FINAL ring, if the peer never reached READY, agent-send falls back to today's
      blind local inject (never worse than pre-change delivery for unclassifiable
      sessions); an env opt-out `AGENT_SEND_IDLE_GATE=0` restores pure-blind behaviour.
- [x] The woken-but-silent escalation still fires LOUD (append to
      `.woken-but-silent-canary.log`, exit non-zero) when all rings defer/blind-fail and
      no receipt arrives — the idle-gate never converts a loud failure into a silent one.
- [x] Hermetic test `tests/agent-send-idle-gate.sh` (no live hub/PTY) proves the
      extracted pure decision `agent_send_idle_gate_decide <state> <ring> <max_rings>
      <gate_enabled>`: READY→inject; BUSY/UNKNOWN on a non-final ring→defer; any
      non-READY state on the FINAL ring→inject (blind fallback); gate_enabled=0→inject
      regardless of state. AND that the reused T-2402 `pushwaker_pty_state` classifier
      maps a BUSY tail ("esc to interrupt")→BUSY, an idle tail ("? for shortcuts")→READY,
      an ambiguous tail→UNKNOWN. Plus `bash -n scripts/agent-send.sh` clean.

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
bash tests/agent-send-idle-gate.sh
bash -n scripts/agent-send.sh

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

### 2026-07-12T16:55:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2410-idle-gate-agent-send-doorbell-ring-sende.md
- **Context:** Initial task creation

### 2026-07-12 — implemented + tested
- **Motivation:** Reproduced live (T-2409 follow-on) — `agent-send` to a busy peer
  (workflow-designer, heartbeat fresh but mid-turn) blind-injected `/check-arc respond`
  into its active input → discarded → woken-but-silent (correctly escalated). The
  recipient waker (T-2402) is idle-gated; the sender ring was not. Asymmetry closed.
- **Built:**
  - `scripts/lib-idle-gate.sh` — pure `agent_send_idle_gate_decide <state> <ring>
    <max_rings> <gate_enabled>` (inject|defer). READY→inject; non-READY non-final→defer;
    non-READY final→inject (blind fallback, never starve); gate-off→inject.
  - `scripts/agent-send.sh` — sources the decision helper + the T-2402 probe primitive
    (`pushwaker_probe_pty`, via `BE_REACHABLE_PUSHWAKER_LIB=1`; best-effort, disables
    silently if a lib is missing). LOCAL ring now probes the PTY and defers on
    BUSY/UNKNOWN instead of blind-injecting; REMOTE ring unchanged; `AGENT_SEND_IDLE_GATE=0`
    opt-out. wake-confirm receipt-poll + woken-but-silent escalation paths untouched.
  - `tests/agent-send-idle-gate.sh` — 16 hermetic cases (decision matrix + T-2402
    classifier + `bash -n` + wiring grep). ALL PASS.
- **Regression:** relay-b1/b2/b3 + relay-wake-confirm + `agent-send --help` all green.
- **Scope honesty:** hardens the briefly-busy + input-safety cases within the current
  design. Does NOT force a persistently-busy interactive peer to consume — that is the
  operator-held design fork (PL-253 / T-2396). See [[project_comms_loud_contract]].
