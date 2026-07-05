---
id: T-2367
name: "Relay AEF pickup-cron 'fw not on PATH' task-create bug to AEF"
description: >
  AEF lib/pickup.sh pickup_create_inception calls bare 'fw' which is unresolved under the pickup cron's minimal PATH — logs 'WARN: fw not on PATH — cannot create task' and creates nothing while the envelope still moves to processed/ + mirrors to topic (silent failure; likely deeper root cause of G-063). Discovered during T-2366. Relay to AEF by hand-creating the AEF task (fw on PATH) to dodge the same bug.

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
created: 2026-07-05T20:22:12Z
last_update: 2026-07-05T20:22:28Z
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

# T-2367: Relay AEF pickup-cron 'fw not on PATH' task-create bug to AEF

## Context

During the T-2366 relay, AEF's per-minute pickup cron was observed to PROCESS
delivered envelopes (move to `processed/` + mirror to the topic) yet log
`WARN: fw not on PATH — cannot create task` for each — creating NO actionable
AEF task. This task relays that finding to AEF as a bug-report, hand-creating
the AEF task with `fw` on PATH so the report itself doesn't fall into the same
hole. See T-2366 for the discovery context.

## Acceptance Criteria

### Agent
- [x] AEF bug-report task created (via `termlink_run`, fw on PATH + `env -u CLAUDECODE`) describing the `pickup_create_inception` bare-`fw` cron failure, with RCA + fix options + evidence — **AEF T-100249**
- [x] Created AEF task verified present + well-formed in AEF `.tasks/active/T-100249-*` (status captured, type build, owner agent, horizon next, tags [pickup, bug-report]; description carries the RCA, `pickup_create_inception` ×3), attributed to termlink source (T-2367)
- [x] termlink T-2367 RCA block filled (bug-class gate G-019)

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

**Symptom:** AEF's per-minute pickup cron logs `WARN: fw not on PATH — cannot
create task` for every processed envelope; the envelope reaches `processed/` +
mirrors to the topic but NO `Pickup: …` task is created. Looks delivered, isn't
actionable (observed 2026-07-05T19:55 for card-redirect P-050/P-051, T-2366).

**Root cause:** `lib/pickup.sh::pickup_create_inception` gates task creation on
`command -v fw` and invokes bare `fw task create`. The AEF pickup cron
(`/etc/cron.d/agentic-audit-999-…`, `PROJECT_ROOT=AEF … fw pickup process`) runs
with a minimal PATH (`/usr/local/bin:/usr/bin:/bin`) that excludes
`/opt/999-Agentic-Engineering-Framework/bin`, so bare `fw` is unresolved →
the `else` branch fires (`WARN … cannot create task; return 1`).

**Why structurally allowed:** the envelope is still moved to `processed/` +
recorded in `dedup.log` + mirrored to the topic regardless of the task-create
failure — a Directive-2 silent-failure. Nothing fails loud, nothing re-queues.
Interactive/manual `fw pickup process` (fw on PATH via shell profile) succeeds,
so the bug is invisible except under cron — masking it. This is a strong
candidate for the deeper root cause of **G-063** (framework:pickup "write-only
sink / zero receipts"): cron-delivered pickups never become tasks.

**Prevention (proposed to AEF):** (1) invoke the resolved framework binary
(`"$FRAMEWORK_ROOT/bin/fw"` / the already-known `$FW`) instead of bare `fw` in
`pickup_create_inception` — robust regardless of cron PATH; (2) additionally
fail LOUD when task-create fails (route the envelope to `rejected/` or emit a
non-zero pickup summary) rather than moving it to `processed/` as if successful.
This task itself is only the *relay*; the fix lands in AEF under AEF sovereignty.

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

### 2026-07-05T20:22:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2367-relay-aef-pickup-cron-fw-not-on-path-tas.md
- **Context:** Initial task creation

### 2026-07-05T20:22:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
