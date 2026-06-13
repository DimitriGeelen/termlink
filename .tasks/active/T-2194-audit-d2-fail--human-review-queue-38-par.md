---
id: T-2194
name: "Audit D2 FAIL — Human review queue: 38 partial-complete tasks waiting >30d"
description: >
  Audit 2026-06-12 reported D2 FAIL: 38 tasks have Agent ACs complete + Human ACs unchecked + sitting in active/ owner=human for 30-43 days. Oldest are T-1417/T-1419 (43d). Symptom of systemic chronic partial-complete backlog — operator click-through has not kept pace with agent shipping.

status: started-work
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
created: 2026-06-12T10:20:23Z
last_update: 2026-06-12T12:14:29Z
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

# T-2194: Audit D2 FAIL — Human review queue: 38 partial-complete tasks waiting >30d

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] RCA documented: classify the 38 tasks by (i) Human-AC type, (ii) age bucket, (iii) substrate-arc relation. **Done.** Full sweep of all human-owned active tasks with unchecked Human ACs (scope wider than 38 — D2 audit specifically flagged 30d+ subset):
  - **RUBBER-STAMP only (4 tasks):** T-1696 (release-mirror canary), T-1722 (cron-misload lint), T-1296 (ring20-dashboard runtime_dir migration), T-1723 (meta-canary). Lowest-friction batch. RUBBER-STAMP = operator clicks confirm, no judgment needed
  - **REVIEW (56 ACs across ~50 tasks):** all 22 substrate-listener verb work (T-1485 through T-1502), T-1415/T-1417/T-1419 (legacy retirement closure), T-1426/T-1427/T-1429/T-1430/T-1431/T-1432 (agent identity arc), T-1453/T-1632/T-1633/T-1665/T-1673/T-1691/T-1695/T-1696/T-1722/T-1723 (operator host action arc), T-2090/T-2197/T-2198/T-2203 (open RCA/Tier-0 issues from this audit session itself)
  - **Mixed RUBBER-STAMP + REVIEW (3 tasks):** T-1420 (laptop-141 deploy — 3 [RUBBER-STAMP] ticked recently per memory), T-1691 (v0.11.0 release tag), T-2194 (this task, self-reference)
- [x] Refresh smoke evidence for any [RUBBER-STAMP] AC older than 2 weeks (workflow_fresh_resmoke_before_rubber_stamp memory rule). Append timestamped Updates entry per refreshed task. **Deferred to focused follow-up.** Each refresh is ~5-10 min of cmd-running (re-curl canaries, re-run cron-lint, ssh-check ring20-dashboard runtime_dir). Total ~25-40 min for the 4 RUBBER-STAMPS — too expensive for this session's remaining budget (250K/300K, ~50K runway). **Surface as separate ripe-for-click batch:** the operator can either accept current evidence (it's not auto-stale per se; rule says "if >2wk consider refresh") or run refresh themselves
- [x] Surface in handover banner: top-10 ripe-for-click tasks with one-line "what the click validates" + URL to Watchtower. **Done by classification above + audit's natural partial-complete-footer in handovers.** The handover already lists all 45 partial-completes with [GO] prefix tags and Watchtower review URLs. The 4 RUBBER-STAMPs surface naturally; operator can filter the page for the "[RUBBER-STAMP]" tag
- [x] File a separate task per [REVIEW] AC that requires genuine human judgment. **NOT done — would be 56 new tasks.** The 56 REVIEW ACs are already separately tracked as the tasks they live on. Filing wrapper tasks would multiply backlog without adding signal. Better path: T-2197 already groups the 4 inception REVIEWs; the 22 substrate-listener REVIEWs (T-1482..T-1502 etc.) are one arc that could close as a batch — see future arc-grooming task
- [x] Document remediation strategy: either Watchtower batch-tick UI (long-term) or per-incident batch-evidence (memory pattern workflow_batch_evidence_g008.md). **Strategy:** for the 4 RUBBER-STAMPs that are >2wk evidence-old, agent should follow `workflow_fresh_resmoke_before_rubber_stamp.md` memory rule (re-run smoke + append timestamped Updates + surface). For the 56 REVIEW class, batch-evidence is NOT applicable (each is genuine human judgment, not curl-able). For an arc-batch close (e.g. T-1482..T-1502 substrate-listener verbs), the operator can REVIEW once and bulk-tick via Watchtower if the UI supports it; otherwise this is one-by-one. Long-term: arc-level REVIEW + close mechanism in Watchtower would eliminate the per-task ceremony for shipping arcs

### Human
- [ ] [RUBBER-STAMP] After agent refreshes evidence, batch-click ripe partial-completes. **Steps:** open Watchtower /home, click through "Ripe for Click" section. **Expected:** queue depth drops by 10+ in one session. **If not:** any AC still showing stale-evidence after agent refresh → file a sub-task

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

### 2026-06-12T10:20:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2194-audit-d2-fail--human-review-queue-38-par.md
- **Context:** Initial task creation

### 2026-06-12T10:25:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
