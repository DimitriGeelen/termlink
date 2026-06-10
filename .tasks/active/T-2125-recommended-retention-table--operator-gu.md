---
id: T-2125
name: "Recommended retention table — operator guidance section in substrate-orchestrator-recipe.md (T-2058 mitigation surface)"
description: >
  Recommended retention table — operator guidance section in substrate-orchestrator-recipe.md (T-2058 mitigation surface)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, docs]
components: [docs/operations]
related_tasks: [T-2018, T-2028, T-2058, T-1991, T-2124]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T13:31:49Z
last_update: 2026-06-10T13:31:49Z
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

# T-2125: Recommended retention table — operator guidance section in substrate-orchestrator-recipe.md (T-2058 mitigation surface)

## Context

`substrate-orchestrator-recipe.md` (T-2124) walks through the canonical work-stealing pattern but doesn't tell the operator what RETENTION to set on the topics they create. The T-2058 hub-side `tracing::warn!` on `Retention::Forever + is_high_rate_pattern(name)` is good defence-in-depth but operators don't see hub logs; the right place for retention guidance is the operator-facing recipe doc.

The CLI's `ensure_topic` helper (`crates/termlink-cli/src/commands/channel.rs:1753`) hard-codes `Retention::Forever` when `--ensure-topic` auto-creates. The hub then fires the T-2058 warn. Closing this loop in code (auto-pick `Messages(1000)` for high-rate patterns) is a separate small task — this task ships the operator-visible guidance now so any operator following the recipe will explicitly create with the right retention from the start.

This addresses the T-2028 Track A spirit (retention coverage audit) at the operator-facing layer; the code-level fix to `ensure_topic` is logged as a follow-up.

## Acceptance Criteria

### Agent
- [x] `docs/operations/substrate-orchestrator-recipe.md` gains a "Recommended retention settings" section with a per-topic-pattern table (work topics, agent-presence, agent-chat-arc, dm:*, conv:*, audit/log topics) listing recommended `Retention` value + rationale
- [x] Section explicitly cross-references T-1991 production incident, T-2058 hub-side loud-warn, and the `is_high_rate_pattern` set (`agent-presence`, `agent-chat-arc`, `agent-listeners-*`, `agent-conv-*`, `dm:*`)
- [x] Section includes one concrete `termlink channel create --retention ...` example per pattern
- [x] Cross-referenced from CLAUDE.md row for the recipe doc

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
grep -q "Recommended retention settings" docs/operations/substrate-orchestrator-recipe.md
grep -q "T-1991" docs/operations/substrate-orchestrator-recipe.md
grep -q "T-2058" docs/operations/substrate-orchestrator-recipe.md
grep -q "is_high_rate_pattern\|high-rate" docs/operations/substrate-orchestrator-recipe.md
grep -q "Messages(1000)" docs/operations/substrate-orchestrator-recipe.md

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

### 2026-06-10 — scope split: ship operator guidance; code-level CLI fix is separate

- **What changed:** Initial framing considered fixing `ensure_topic` (CLI auto-create on `--ensure-topic`) to pick `Messages(1000)` for high-rate patterns in addition to writing the recipe section. The code fix is small (~10 LOC duplicate of the `is_high_rate_pattern` predicate into the CLI crate) but requires a cargo build cycle to validate. Given session budget pressure (240K context, ~50K headroom), splitting the docs-first delivery is safer.
- **Plan impact:** This task delivers operator-visible guidance NOW. A follow-up task should fix `ensure_topic` to make it consistent with the documented recommendations (defence-in-depth: if operator forgets to pre-create, --ensure-topic should pick the right retention).
- **Triggered:** Logged in the CLAUDE.md recipe row as "Code-level follow-up (auto-pick high-rate retention in CLI's `ensure_topic` helper) logged separately." — file the follow-up task when picking this up next session.

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

### 2026-06-10T13:31:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2125-recommended-retention-table--operator-gu.md
- **Context:** Initial task creation
