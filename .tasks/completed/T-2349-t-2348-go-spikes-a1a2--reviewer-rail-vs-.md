---
id: T-2349
name: "T-2348 GO spikes: A1/A2 — reviewer rail vs inception task files"
description: >
  T-2348 GO spikes: A1/A2 — reviewer rail vs inception task files

status: work-completed
workflow_type: test
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-04T10:23:05Z
last_update: 2026-07-04T10:27:46Z
date_finished: 2026-07-04T10:27:46Z
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

# T-2349: T-2348 GO spikes: A1/A2 — reviewer rail vs inception task files

## Context

T-2348 (reviewer-agent-assisted inception decides) was GO'd by the human 2026-07-04
(commit c62de76b). The build is proposed AEF-side (pickup 073), but two assumptions
were filed UNTESTED and de-risking them is TermLink-side spike work:

- **A1:** the fw independent-review v0.1 rail (T-1885, `fw reviewer`) can consume an
  inception TASK FILE as its review subject (current validators target other shapes).
- **A2:** inception evidence claims are machine-extractable with enough structure to
  verify (file:line refs, shell commands, decision ids).

Deliverable: one spike-findings report `docs/reports/T-2349-reviewer-rail-inception-spikes.md`
answering A1 + A2 with evidence, plus a findings addendum note for the AEF pickup thread
if the answers change the pickup-073 proposal shape. NO build under this task — build is
AEF-side per the GO.

Research artifact: `docs/reports/T-2348-reviewer-assisted-inception-decides.md` (parent).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A1 answered with evidence: ran `fw reviewer --no-write --json` against THREE
  real inceptions (T-2338, T-2276 active; T-2348 completed) — all consumed natively;
  the rail already ships an inception-aware pattern (`disposition-incomplete`,
  T-2191) gating on `workflow_type: inception` + parsing IW entries. Verbatim
  verdict envelope in the spike report. **A1 = YES, no input adapter needed.**
- [x] A2 answered with evidence: claim shapes across the 3 inceptions (T-NNNN refs,
  file:line, docs/reports paths, G-/PL- ids, commit hashes, runnable commands) map
  1:1 onto the rail's existing `_CITATION_PATTERNS` (static_scan.py:1518).
  Extraction ratio: 3 of 4 real `answered` IW rationales carry matchable citations
  but only 1 is detected — two verified extraction defects (template-comment
  phantom entries; first-line-only rationale capture) reproduced in isolation.
  **A2 = YES in principle; extraction precision blocked by 2 upstream defects.**
- [x] Spike report `docs/reports/T-2349-reviewer-rail-inception-spikes.md` exists,
  answers A1+A2 ("A1 verdict"/"A2 verdict" sections), states the pickup-073
  implication (no input adapter; fix extraction defects first, then the
  CONFIRMED/UNVERIFIED/CONTRADICTED verification layer). Defects filed upstream
  to AEF as pickup 074 (see Updates).

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
test -f docs/reports/T-2349-reviewer-rail-inception-spikes.md
out=$(cat docs/reports/T-2349-reviewer-rail-inception-spikes.md); echo "$out" | grep -q "A1 verdict"
out=$(cat docs/reports/T-2349-reviewer-rail-inception-spikes.md); echo "$out" | grep -q "A2 verdict"

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

### 2026-07-04T10:28Z — spikes complete + defects filed upstream [agent]
- **A1:** YES — rail consumes inception task files natively (T-2191 pattern already
  inception-aware). Scanned T-2338/T-2276/T-2348 with `--no-write` (no mutation).
- **A2:** YES in principle — claim shapes match the rail's `_CITATION_PATTERNS`;
  two extraction defects verified by direct module repro: (1) HTML template-comment
  IW entries parsed as real (phantom/duplicate findings on T-2338+T-2276),
  (2) `^\s*rationale:\s*(.+?)$` first-line-only capture misses continuation-line
  citations (T-2338 IW-2 `channel.rs:8597-8601`).
- **Upstream relay:** defects filed as
  `/opt/999-Agentic-Engineering-Framework/.pickup/074-reviewer-disposition-detector-two-defects.md`
  (directory drop per PL-228, verified on disk 2,787 bytes) — cross-referenced to
  pickup 073 (the T-2348 proposal these defects sit under).
- **Deliverable:** `docs/reports/T-2349-reviewer-rail-inception-spikes.md`.

### 2026-07-04T10:23:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2349-t-2348-go-spikes-a1a2--reviewer-rail-vs-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-170c35b8
- **Timestamp:** 2026-07-04T10:27:47Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-04T10:27:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
