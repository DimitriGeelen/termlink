---
id: T-2407
name: "arc mcp-slimming S2 — trim the 600-1000 char MCP description band (~70 tools) to policy"
description: >
  Second slice of arc-005 mcp-slimming. Trim the ~70 tool descriptions in the 600-1000 char band per the S1 policy (keep purpose + non-obvious param gotchas + safety notes; cut T-XXXX archaeology, PL cross-refs, schema-restatement). cargo build -p termlink-mcp passes, tool count unchanged, anti-regrowth guard ceiling tightened toward target. Report bytes reclaimed.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:mcp-slimming]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-11T14:16:18Z
last_update: 2026-07-11T16:51:02Z
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

# T-2407: arc mcp-slimming S2 — trim the 600-1000 char MCP description band (~70 tools) to policy

## Context

Slice 2 of arc `mcp-slimming`. S1 (T-2406) trimmed the 11,751-char meta-tool + all 24
descriptions over 1000 chars (156,525 → 133,220 bytes, ~5,800 tokens/agent reclaimed). This
slice trims the next band — the ~70 tool descriptions between 600 and 1000 chars — applying
the same policy (`docs/operations/mcp-description-policy.md`): keep purpose + non-obvious
param semantics + safety/return-shape notes; cut T-XXXX archaeology, PL cross-refs, and
schema-restatement. No tools removed (trim text only).

## Acceptance Criteria

### Agent
- [x] **600–1000 band trimmed per policy.** DONE: all 86 descriptions in the 600–999 band
  trimmed per `docs/operations/mcp-description-policy.md` (cut T-XXXX/PL archaeology, CLI-parity
  prose, sibling cross-refs, schema-restatement). Agent-critical guidance preserved — verified
  by spot-check that `release`'s ack=true/false cursor-advance pivot and `renew`'s "absolute
  NOT relative add" gotcha survived. `cargo build -p termlink-mcp` passes; **tool count 273
  (unchanged)**; `cargo test -p termlink-mcp` green (879 passed, T-1962 drift-guard green).
  **Bytes reclaimed: 133,220 → 112,319 = 20,901 (~5.2k more tokens/agent/session).**
- [x] **Guard ceiling tightened in lockstep.** DONE: `MAX_DESC_CEILING` 1600→1560,
  `TOTAL_DESC_CEILING` 135000→113000 in `scripts/test-mcp-desc-budget.sh` (+ S2 ceiling-history
  comment line). `bash scripts/test-mcp-desc-budget.sh` → RESULT: PASS.

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
bash scripts/test-mcp-desc-budget.sh
cargo build -p termlink-mcp 2>&1 | tail -1 | grep -q Finished

## Evolution

### 2026-07-11 — delegated batch trim; diminishing-returns confirmed
- **What changed:** S2 reclaimed 20,901 bytes across 86 descriptions — roughly the same
  magnitude as S1's 23,305, but spread over 86 tools instead of concentrated in ~25. Confirms
  the front-loaded shape: per-tool yield is much lower here (~240 bytes/tool vs S1's head trim).
- **Plan impact:** The 86-tool batch was mechanical enough to delegate to a subagent (policy +
  guard already in place from S1 made it safe — the guard is the objective success signal, so
  the orchestrator could verify without reading all 86 edits). S3's long tail (<600 chars) will
  yield even less per tool; S3 is more about relocating any genuine archaeology to docs +
  final guard tightening than about big byte wins.
- **Triggered:** No new sub-tasks. A few tools (e.g. `poll_start`) retain minor CLI-parity
  prose after the batch — acceptable within policy tolerance ("one short see-also at most"),
  S3 can sweep stragglers.

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

### 2026-07-11T14:16:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2407-arc-mcp-slimming-s2--trim-the-600-1000-c.md
- **Context:** Initial task creation

### 2026-07-11T15:31:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-11T16:51:02Z — status-update [task-update-agent]
- **Change:** tags: +arc:mcp-slimming
