---
id: T-2482
name: "comms round-trip self-test — staged discover/send/consume PASS-FAIL prover"
description: >
  comms round-trip self-test — staged discover/send/consume PASS-FAIL prover

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-01T22:47:36Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-08-01T22:54:59Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:47Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:11Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2482: comms round-trip self-test — staged discover/send/consume PASS-FAIL prover

## Context

Fresh critical-review gap (T-2468 mandate, 4th re-issue). The codebase has 11
failure-detecting canaries + four observability arcs — all detect the substrate
breaking **after the fact**. The charter's core promise ("discover each other,
exchange durable messages") has **no single affirmative on-demand proof that the
full round-trip works right now**, attributed to the link that broke. The
operator's recurring real-world complaint is literally "why is there still no
response?" — a failure that collapses into silence.

`agent-send.sh --to <id>` already composes discover(pty gate) + send +
consume-confirm + diagnose, but (a) collapses to ONE exit code (can't say
DISCOVER=PASS / SEND=PASS / CONSUME=FAIL), (b) is negative-framed (fires a real
turn, fails loud), (c) has no affirmative "all three green" report. The only
thing named "selftest" (`agent-conversation-selftest.sh`, T-1829) is
**loopback-only** — same identity, never touches a live peer's PTY/doorbell.

**This build:** `scripts/comms-selftest.sh --peer <id>` — a thin staged prover
that composes existing primitives (`diagnose-unconsumed.sh` for DISCOVER +
attribution, `agent-send.sh` for SEND+CONSUME) and emits three PASS/FAIL stage
lines so a failure is pinned to discover vs send vs consume. Deepens the core
(Reliability: no silent failures); removes no surface; adds no subsystem.

## Acceptance Criteria

### Agent
- [x] `scripts/comms-selftest.sh` exists, is executable, and `--help` documents the
      purpose (affirmative round-trip proof), the three stages (DISCOVER / SEND /
      CONSUME), flags (`--peer`, `--message`, `--discover-only`, `--json`), and the
      exit codes (0 all-pass / 1 a-stage-failed / 2 tooling-error).
- [x] Emits a **per-stage PASS/FAIL** breakdown (DISCOVER, SEND, CONSUME) and, on
      failure, names the broken stage — the attribution `agent-send.sh`'s single
      exit code cannot give. (Live: `broken_stage=CONSUME` against a busy peer.)
- [x] **DISCOVER attribution:** a dead/absent peer → DISCOVER=FAIL(dead) and STOP
      (no pointless send); a LIVE-but-unarmed peer → DISCOVER=FAIL(unwakeable) and
      STOP (proven via the reused `diagnose-unconsumed.sh` PL-213 presence hook).
- [x] **SEND+CONSUME attribution:** maps `agent-send.sh`'s collapsed exit code —
      0 → SEND=PASS+CONSUME=PASS (round-trip proven); 3/124 → SEND=PASS+CONSUME=FAIL
      (durably written, peer never acked); precondition/other → SEND=FAIL (proven
      via a PL-213 send-rc test hook, no live hub needed). Own bounded send-timeout
      so it never hangs an operator.
- [x] `--discover-only` runs the side-effect-free DISCOVER stage alone (asserts the
      peer is reachable + armed without firing a turn), exit 0 when armed. (Live:
      green against sonnenstall/workshop-designer/workflow-designer/aef.)
- [x] `scripts/test-comms-selftest.sh` covers all-green, each stage-fail, discover
      -only, and tooling-error via fixtures; prints a final `PASS`/`FAIL` line; all
      cases pass (13/13).
- [x] Documented: `docs/operations/comms-selftest.md` (what it proves, how it
      composes the primitives) and referenced from CLAUDE.md.

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

test -x scripts/comms-selftest.sh
out=$(bash scripts/comms-selftest.sh --help 2>&1); echo "$out" | grep -q "DISCOVER"
out=$(bash scripts/test-comms-selftest.sh 2>&1); echo "$out" | grep -q "^test-comms-selftest: PASS"
test -f docs/operations/comms-selftest.md
grep -q "comms-selftest.sh" CLAUDE.md

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

**Symptom:** The core comms round-trip to a peer silently fails — the operator's
recurring "why is there still no response?" — with no single command to prove the
round-trip works or to say *which* link (discover/send/consume) broke.

**Root cause:** The substrate's liveness observability was built **entirely as
after-the-fact failure detection** (11 canaries + four observability arcs). The
affirmative on-demand round-trip proof was never composed. `agent-send.sh` does
compose discover+send+consume, but collapses them into a single exit code, so a
failure could not be attributed to a stage — the round-trip failed into silence.

**Why structurally allowed:** Observability grew reactively — each field failure
spawned a detector (a canary) for *that* failure, but nothing ever asked "is there
one affirmative proof that the whole promise holds right now?" The charter promise
("discover, exchange durable messages") had detectors for its failure modes but no
prover for its success. No gate required one.

**Prevention:** `scripts/comms-selftest.sh` IS the prevention — an on-demand,
staged, per-stage PASS/FAIL proof that names the broken link. Regression-locked by
`scripts/test-comms-selftest.sh` (13 cases, PL-213 fixtures, no live hub) and made
discoverable via `docs/operations/comms-selftest.md` + CLAUDE.md. It is the
affirmative complement to the canaries: "prove it works" beside "detect when it broke".

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

### 2026-08-01T22:47:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2482-comms-round-trip-self-test--staged-disco.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-211524fb
- **Timestamp:** 2026-08-01T22:55:01Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-01T22:54:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
