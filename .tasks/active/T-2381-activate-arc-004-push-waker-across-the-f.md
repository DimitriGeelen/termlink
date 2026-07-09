---
id: T-2381
name: "Activate arc-004 push-waker across the fleet — make shipped push-wake capability live"
description: >
  Activate arc-004 push-waker across the fleet — make shipped push-wake capability live

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
created: 2026-07-07T18:12:31Z
last_update: 2026-07-07T18:26:25Z
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

# T-2381: Activate arc-004 push-waker across the fleet — make shipped push-wake capability live

## Context

arc-004 (push-transport) is closed=shipped but DARK in the field: T-2380 E4
found zero push-waker processes running on .107 — all live agents are silently
un-reachable. This task OPERATES the already-shipped capability (not new build):
arm the waker where we can, verify push-wake E2E on the real fleet, coordinate
arming on hosts we don't own, and record fleet activation state. Preconditions to
respect: PL-237 (push-wake activation conditions), PL-236 (identity-resolver env).

## Findings (2026-07-07, grounded — LIVE on real .107 hub)

**arc-004 WORKS on the real .107 production hub — proven, both rails.** Armed a
real PTY-backed probe (`arc004-probe`, pushwaker pid 3629693); sent messages on
the local hub and observed `/check-arc respond` injected into its PTY:
- **Inbox rail** via `bash scripts/agent-send.sh --to arc004-probe` — fired.
  fp-INDEPENDENT (addressee = agent_id) → the ROBUST path.
- **DM rail** via `agent contact --target-fp dcd44820fc12daed` — fired. Needs the
  correct **per-agent** fp.

**NEW BUG (send/receive fp mismatch on shared hosts) — the real fleet blocker:**
the waker resolves self_fp per-agent via `agent identity --resolve`
(`dcd44820fc12daed`), but the SENDER's `agent contact <NAME>` resolves the target
via `session.discover` → **shared host fp `d1993c2c`** → wrong dm topic → **dm
rail never wakes for name-based contact**. Hermetic demo hid this by hardcoding
fps. Workaround: use inbox-rail doorbell (`agent-send.sh --to <name>`,
fp-independent) OR `agent contact --target-fp <per-agent-fp>`. Fix (sender-side
analog of the be-reachable `--resolve` fix) → T-2380 candidate.

**Second bug (NOT minor — reconfirmed 2026-07-09, blocked the real send):**
`agent contact --target-fp 9219671e28054458` tried to create the dm topic with
retention `Messages(1000)` but it exists as `Forever` → `channel.create` -32603
blocked the coordination send entirely. Same topic that landed T-2379 offset 52
via agent contact earlier — so the topic's retention was `Forever` by the time I
re-sent, and agent contact's non-idempotent create now refuses. Topic-create
retention MUST be idempotent (or agent contact must skip create when the topic
exists). Workaround used: direct `termlink channel post <dm-topic> --hub .122
--metadata conversation_id=T-2381 --mention ring20-management-agent` — appends to
the existing Forever topic (no create attempt), landed **offset 56**. This is a
real field blocker on any peer whose dm topic is `Forever`, not cosmetic → upgrade
to a T-2380 candidate / its own bug task.

**Fleet constraint (PL-237):** push-wake needs an injectable PTY; plain
`claude --resume` (no tmux) gets no waker → fix fork in runbook + T-2380 C7.

**CLEANUP PENDING (budget gate blocked the kill):** probe `register --shell`
pid 3628688 (`arc004-probe`) still registered — next session:
`kill 3628688`, confirm `termlink list | grep arc004` empty. The pushwaker was
reaped cleanly by `be-reachable.sh stop`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Read + honor PL-237 / PL-236 push-wake preconditions before arming (verifier report + PL-237/236/166/200)
- [x] Push-waker armed on **the real .107 production hub** (not hermetic) — probe session `arc004-probe`, pushwaker pid 3629693 confirmed alive via `ps`; self_fp resolved per-agent (`dcd44820fc12daed`, NOT shared host fp — PL-236 sidestep works)
- [x] Push-wake verified END-TO-END on the real .107 hub (EXCEEDS AC — real, not hermetic): **both rails injected `/check-arc respond` into the probe PTY** — inbox rail via `agent-send.sh --to` (fp-independent) AND dm rail via `agent contact --target-fp dcd44820fc12daed`
- [x] Fleet activation state recorded — runbook `docs/operations/arc-004-fleet-activation.md` (per-host table + 3 preconditions + recipe)
- [x] Coordination message sent to ring20-manager to arm their wakers per the runbook — landed **offset 56** on dm:9219671e28054458:d1993c2c3ec44c94 (.122) via direct `channel post` (agent contact was blocked by the retention-idempotency bug; routed around it). Message = full activation recipe + 3 preconditions + the send-side fp-mismatch + retention blockers + .122 read-wedge remediation. Staged verbatim in `.context/working/T-2381-ring20-manager-arc004-activation.md`. Awaiting ring20-manager ack on thread T-2381.

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

### 2026-07-07T18:12:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2381-activate-arc-004-push-waker-across-the-f.md
- **Context:** Initial task creation
