---
id: T-2365
name: "Fix G-067 audit per-file python fork — batch frontmatter parse-check"
description: >
  Pre-push structure audit forks one python3 per task file (~2127) importing web.shared.parse_frontmatter; ~85s wall, intermittently kills pushes (exit 143). Batch into one python3 invocation. Land in AEF upstream, re-vendor into termlink.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-05T19:00:55Z
last_update: 2026-07-05T19:14:46Z
date_finished: 2026-07-05T19:14:46Z
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

# T-2365: Fix G-067 audit per-file python fork — batch frontmatter parse-check

## Context

G-067: the pre-push structure audit (`agents/audit/audit.sh`, T-2067 frontmatter-parse
check, lines ~617-659) loops over every task file in `.tasks/active` + `.tasks/completed`
(termlink: 2127 files) and fork-execs a separate `python3 -c` per file importing
`web.shared.parse_frontmatter`. That is O(N) interpreter startups → ~85s wall. In a
consumer project (`web/` absent) each fork immediately hits `ImportError` → exits 0 →
zero useful output, ~85s pure waste. Root cause of the recurring push-failure friction
(background `git push` killed mid-audit, exit 143 / "Terminated"). Fix belongs in AEF
upstream (`/opt/999-Agentic-Engineering-Framework`, remote origin/master), then re-vendor
into termlink. See memory project_g067_audit_slowness.

## Acceptance Criteria

### Agent
- [x] AEF `agents/audit/audit.sh` T-2067 frontmatter-parse block forks python3 at most ONCE per audit run (not once per task file) — verified: the batched block has one `| python3 -c` fork; the second `python3` match is the T-2297 explanatory comment
- [x] Output-equivalence preserved: block is byte-identical to AEF origin/master (which AEF's own audit runs with web/ present); in termlink (consumer, `web/` absent) it emits nothing and returns in 0.07s — identical net result to the old per-file loop (both record zero fails in a web/-absent project)
- [x] fm-parse component wall-time on termlink's 2127-file corpus drops from ~142s (measured: 200 forks=13.3s extrapolated) to <1s (measured: 0.07s — consumer web/-absent fast path). Full structure section drops ~206s→~64s; the residual ~64s is TWO SEPARATE bottlenecks (arc-commit-recency ~20s, large-file gate ~26s) logged as a follow-up finding, NOT part of the G-067 fm-parse fork
- [x] Fix already landed in AEF origin/master as T-2297 (commit 06041f9b); re-vendored into termlink's `.agentic-framework/agents/audit/audit.sh` by surgical block replacement (byte-identical to master) and committed (db4d52e4). No AEF mutation needed (upstream fix pre-existed)

<!-- No Human ACs — this is a pure tooling fix with deterministic verification. -->

<!-- REMOVED-Human-section-guidance:
     Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
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

## RCA

**Symptom:** Pre-push structure audit ran ~206s and intermittently killed
`git push` (exit 143 / "Terminated") throughout the session.

**Root cause:** termlink's vendored `.agentic-framework/agents/audit/audit.sh`
predated AEF commit T-2297 (06041f9b). The stale vendored copy still fork-exec'd
one `python3` per task file (2127 files, ~142s) to import
`web.shared.parse_frontmatter` — and in a consumer project (`web/` absent) every
one of those forks did nothing but hit `ImportError` and exit 0. The upstream
fix (single batched fork) already existed on AEF master; termlink just hadn't
re-vendored it. A stale-vendor bug, not a novel logic bug.

**Why structurally allowed:** Nothing surfaces vendored-framework drift.
`fw vendor` is manual and there is no check that compares termlink's vendored
audit.sh (or any vendored script) against the upstream master ref, so a
performance fix landing in AEF is invisible to consumers until someone
re-vendors. Compounding it, the audit has no self-timing guard — a section that
balloons to 206s emits no warning; it just silently risks the push timeout.

**Prevention:** (1) This re-vendor removes the dominant 142s. (2) Durable
prevention (logged as follow-up, not done here): either a vendored-drift canary
(diff key vendored scripts against upstream master, warn on divergence) OR a
self-timing guard in audit.sh that WARNs when a `--section` exceeds a wall-time
budget — the latter would have surfaced G-067 the day it first crossed the line.
(3) Residual non-fm-parse bottlenecks (arc-recency ~20s, large-file gate ~26s)
are documented in `docs/reports/T-2365-*.md` so the next optimizer starts from
the measured profile.

## Verification

# T-2365 batch marker present in the re-vendored termlink copy
grep -q "T-2365" /opt/termlink/.agentic-framework/agents/audit/audit.sh
# The batched T-2297 block is present (single-fork, stdin-streamed — not per-file fork)
grep -q "T-2297: single batched python3" /opt/termlink/.agentic-framework/agents/audit/audit.sh
# Exactly one real python3 fork in the fm-parse block (the '| python3 -c' pipe)
blk=$(sed -n '/T-2067: task-frontmatter parse check/,/Anchor-task existence check/p' /opt/termlink/.agentic-framework/agents/audit/audit.sh); n=$(echo "$blk" | grep -c '| python3 -c'); [ "$n" -eq 1 ]
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

### 2026-07-05T19:00:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2365-fix-g-067-audit-per-file-python-fork--ba.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-0235cb29
- **Timestamp:** 2026-07-05T19:14:47Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-05T19:14:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
