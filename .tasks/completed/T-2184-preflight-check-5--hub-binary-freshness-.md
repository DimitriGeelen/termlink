---
id: T-2184
name: "/preflight Check 5 — hub binary freshness probe (T-2183 symmetric to T-2181 Check 4)"
description: >
  Add /preflight Check 5: probe local hub.governor_status response shape — absence of rate_buckets_evicted_total field signals pre-T-2139 hub (replaced binary, not restarted). Symmetric companion to T-2181 Check 4 (CLI binary freshness). Detection mechanism: positive presence-check of a field shipped by a known release tag; remediation hint: 'restart hub to pick up new binary'. Cross-ref: PL-209 (rate-limit pressure observation, the originating misdiagnosis), T-2181 (client-side prior art), T-2154 (substrate-preflight.sh script), T-2158 (/preflight skill). Horizon=next.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/substrate-preflight.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-11T21:13:22Z
last_update: 2026-06-11T21:17:32Z
date_finished: 2026-06-11T21:17:32Z
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

# T-2184: /preflight Check 5 — hub binary freshness probe (T-2183 symmetric to T-2181 Check 4)

## Context

Symmetric companion to T-2181 `/preflight` Check 4 (CLI binary
freshness). Captured 2026-06-11 from T-2183, where PL-209's
"evicted_total=n/a" turned out to be a stale **hub** binary
(/proc/<pid>/exe → `... (deleted)`, predates T-2139's
`rate_buckets_evicted_total` field) — not a missing telemetry feature.
The misdiagnosis took 30+ minutes of investigation; a deploy-time
probe would catch the same class instantly.

Detection strategy: probe local hub via `termlink hub status
--governor --json` and test for `rate_buckets_evicted_total` field
presence in `.governor` object. Field absence ⇒ pre-T-2139 hub ⇒
WARN with `restart hub` remediation hint. Field present (any value
including 0) ⇒ PASS.

Graceful degradation: hub not running ⇒ SKIP (different failure
mode, runtime_dir Check 1 is upstream); JSON parse error ⇒ SKIP;
no hubs.toml ⇒ Check 2 already caught it.

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-preflight.sh` gains a new `check_hub_binary_freshness()` function modelled after `check_binary_freshness()` (Check 4 template, lines 329-365).
- [x] Function probes local hub via `termlink hub status --governor --json`, tests for `rate_buckets_evicted_total` field presence (using `grep` or `jq` — pick the lighter dependency match for the existing file), classifies WARN on absence with remediation hint, PASS on presence, SKIP on hub-not-running.
- [x] Function wired into the "Run all checks" sequence after `check_binary_freshness`.
- [x] Script header docstring updated: "Check 5: hub binary freshness via field-presence probe" with cross-ref to T-2139 + T-2183 root cause.
- [x] `usage()` updated: Check 5 line added.
- [x] CLAUDE.md `/preflight` catalog row updated: "Four checks" → "Five checks", Check 5 entry added (T-2184 cross-ref).
- [x] `.claude/commands/preflight.md` skill description updated to mention Check 5 (if it enumerates checks).
- [x] Live validation on this host: `./scripts/substrate-preflight.sh` emits the Check 5 line. Since the local hub IS stale (PID 2058543, deleted-on-disk binary), expect WARN. After hub restart (out-of-scope here, recipe documented), expect PASS.

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

### 2026-06-11T21:13:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2184-preflight-check-5--hub-binary-freshness-.md
- **Context:** Initial task creation

### 2026-06-11T21:14:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-11T21:17:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
