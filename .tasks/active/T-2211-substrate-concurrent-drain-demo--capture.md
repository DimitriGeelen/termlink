---
id: T-2211
name: "Substrate concurrent-drain demo — capture arc-001 demo_evidence"
description: >
  Self-contained shell demo: N synthetic workers drain an M-unit work-queue on the live hub via the claim primitive, asserting each unit is won by exactly one worker (no double-claim) — the operator-facing proof of arc-001's headline mechanic. Composes existing verbs (channel create/post/claim/claims-summary); no new primitive, no human-gated inception. Captures demo_evidence for the arc.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T13:59:19Z
last_update: 2026-06-13T14:03:51Z
date_finished: 2026-06-13T14:03:39Z
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

# T-2211: Substrate concurrent-drain demo — capture arc-001 demo_evidence

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-drain-demo.sh` exists, composes only existing verbs (channel create/post/claim/claims-summary/release), `--workers N --units M --hub --json --keep --help`
- [x] Demo seeds an M-unit topic, races N concurrent workers via the claim primitive, and asserts each unit won by EXACTLY one worker (disjoint union = no double-claim) — exits 0 on clean drain, non-zero on any double-claim/gap
- [x] Live run on this host PASSes (captured output committed under docs/reports/)
- [x] arc-001 `demo_evidence` field populated with the proof path (factual; no status/decision change)

### Human
- [ ] [REVIEW] The demo is a convincing operator-facing proof of the headline mechanic
  **Steps:**
  1. Read docs/reports/T-2211-substrate-drain-demo.md
  2. Optionally run `scripts/substrate-drain-demo.sh --workers 4 --units 12`
  **Expected:** clean drain, zero double-claims, each unit attributed to one worker
  **If not:** note what is unconvincing or missing

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

bash -n scripts/substrate-drain-demo.sh
test -f docs/reports/T-2211-substrate-drain-demo.md

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

## Recommendation

**Recommendation:** GO (partial-complete — Agent ACs met; one Human [REVIEW] AC awaits you)

**Rationale:** All four Agent ACs pass. `scripts/substrate-drain-demo.sh` composes only
shipped substrate verbs (no new primitive, no hub change, no human-gated inception) and
proves arc-001's headline mechanic — N concurrent workers drain an M-unit queue via the
claim primitive with exclusive delivery (each unit won exactly once, zero double-claims)
under real contention (18-60 CLAIM_CONFLICTs/run). Verified live at 3/9, 5/15, 4/12 — all
PASS, exit 0. This is the operator-facing companion to the existing Rust race test and
substrate-smoke.sh, and it populates the arc's previously-null demo_evidence. Arc closure
remains gated on the 4 human GO/NO-GO inception decisions (T-2022/24/25/26) per T-2201 —
unchanged by this work.

**Evidence:**
- `scripts/substrate-drain-demo.sh` (190 LOC, `bash -n` clean, exit 0 on clean drain / 1 on violation)
- `docs/reports/T-2211-substrate-drain-demo.md` — captured runs + reproduce instructions
- arc-001 `demo_evidence` field populated (commit this session)
- Reproduce: `TERMLINK_BIN=target/release/termlink scripts/substrate-drain-demo.sh --workers 4 --units 12`

## Updates

### 2026-06-13T13:59:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2211-substrate-concurrent-drain-demo--capture.md
- **Context:** Initial task creation

### 2026-06-13T14:03:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
