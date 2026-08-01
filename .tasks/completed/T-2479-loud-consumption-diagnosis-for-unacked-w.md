---
id: T-2479
name: "Loud consumption-diagnosis for unacked wakes (T-2476 P2 build, G-083)"
description: >
  Loud consumption-diagnosis for unacked wakes (T-2476 P2 build, G-083)

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
created: 2026-08-01T21:11:41Z
last_update: 2026-08-01T21:11:41Z
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

# T-2479: Loud consumption-diagnosis for unacked wakes (T-2476 P2 build, G-083)

## Context

Build for T-2476 (P2, G-083 — GO recorded). `agent-send.sh` already rings + waits for
a `msg_type=receipt` and returns **exit 3** when no receipt arrives within `max_rings` —
so a wake that IS consumed gets confirmed. G-083's gap is that the **not-acked path is
silent/undiagnosed**: a peer that is LIVE + armed (`pty_session` set) but busy or in
manual-accept mode never posts a receipt, and the sender gets only a bare "not acked" —
no classification of *why* (LIVE≠listening) and no remediation. This is the operator's
"comms doesn't flow without manual nudging" pain.

**Deliverable:** a loud consumption-diagnosis that turns silent "rung-but-unconsumed"
into a classified, actionable failure. Given a peer + dm-topic (+ optional conversation_id),
it reads the load-bearing signals — peer presence freshness (LIVE?), `metadata.pty_session`
(armed?), and receipt presence on the dm-topic — and classifies:
- **consumed** — a receipt exists (exit 0, healthy).
- **busy-or-manual** — peer LIVE + armed but no receipt (the alarming G-083 case; message
  durably written at offset X, session not consuming). Remediation: peer must consume
  manually / check auto-accept (IS_SANDBOX) / relaunch through the T-2388 armed launcher.
- **unwakeable** — peer LIVE but NOT armed (no `pty_session`; T-2380 breakpoint #2).
  Remediation: relaunch armed via `tl-claude.sh start --reachable`.
- **dead** — peer presence stale/absent. Remediation: relaunch the peer.

Standalone verb `scripts/diagnose-unconsumed.sh` (operator-runnable ad-hoc AND invoked by
`agent-send.sh` on its exit-3 path so the diagnosis is automatic). Test-JSON hooks feed
canned presence + receipt fixtures for host-independent unit tests (PL-213 pattern).

## Acceptance Criteria

### Agent
- [x] `scripts/diagnose-unconsumed.sh` exists: given `--peer`/`--topic` (+ optional `--cid`), reads presence + receipt state and classifies into consumed / busy-or-manual / unwakeable / dead with a LOUD per-class remediation line
- [x] Exit codes are contract-stable: 0 = consumed, 1 = busy-or-manual, 2 = unwakeable, 3 = dead, 4 = tooling error (documented in `--help`)
- [x] Test-JSON hooks (`TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON` + `TERMLINK_DIAGNOSE_TEST_RECEIPT_JSON`) let the classifier run without a live hub (PL-213)
- [x] `scripts/test-diagnose-unconsumed.sh` covers all four classes via the test hooks and passes (9/9)
- [x] `agent-send.sh` invokes the diagnosis on its not-acked (exit 3) path — a silent unconsumed wake now emits the classified diagnosis (grep=3, syntax `bash -n` clean)
- [x] Signal validated against a live peer: `agent-listeners.sh --json` carries `pty_session`/`status`/`age_secs`; live probe of peer `aef` → LIVE+armed→busy-or-manual (the "prove the signal first" step, G-083)

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
test -x scripts/diagnose-unconsumed.sh
bash scripts/diagnose-unconsumed.sh --help 2>&1 | grep -q "busy-or-manual"
bash scripts/test-diagnose-unconsumed.sh 2>&1 | tail -1 | grep -q "PASS"
grep -q "diagnose-unconsumed.sh" scripts/agent-send.sh

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

### 2026-08-01T21:11:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2479-loud-consumption-diagnosis-for-unacked-w.md
- **Context:** Initial task creation
