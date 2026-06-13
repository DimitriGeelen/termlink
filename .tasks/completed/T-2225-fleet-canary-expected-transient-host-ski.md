---
id: T-2225
name: "Fleet canary: expected-transient host skip (.141 alert-fatigue fix)"
description: >
  Teach check-fleet-doorbell-mail-health.sh to treat operator-declared transient hosts (e.g. laptop-141) as expected-down so a sleeping laptop does not DRIFT the whole-fleet canary (G-019 alert-fatigue prevention).

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
created: 2026-06-13T22:54:10Z
last_update: 2026-06-13T23:01:39Z
date_finished: 2026-06-13T23:01:39Z
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

# T-2225: Fleet canary: expected-transient host skip (.141 alert-fatigue fix)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Canary loads an operator-declared expected-transient host set from a file (default `.context/cron/fleet-dm-canary-transient`, one profile NAME per line, `#` comments) merged with the `FLEET_DM_CANARY_TRANSIENT` env var (comma-separated); `--transient-file PATH` overrides the file.
- [x] A declared-transient host that is unreachable or `setup-fail` is classified `transient_skipped` and does NOT flip `overall_ok` (exit stays 0, no DRIFT) — a sleeping laptop no longer DRIFTs the whole-fleet canary.
- [x] Skip suppresses down-ness only, not brokenness: a transient host that is reachable still counts `pass`; reachable-but-`fail` still triggers DRIFT.
- [x] Both human and `--json` output surface transient-skipped hosts with a clear marker plus a `transient_skipped=N` summary field.
- [x] `laptop-141` is declared transient in `.context/cron/fleet-dm-canary-transient`; a live run shows the fleet canary returns exit 0 (was DRIFT) with `.141` marked `(transient — skipped)`.
- [x] A regression test in `test-check-fleet-doorbell-mail-health.sh` covers the transient-skip path (unreachable host + declared transient → ok=true, transient_skipped>=1, exit 0).

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification
bash -n scripts/check-fleet-doorbell-mail-health.sh
bash scripts/test-check-fleet-doorbell-mail-health.sh
# Deterministic proof: an unreachable host declared transient via env exits 0 (no DRIFT).
printf '[hubs.t2225-trans]\naddress = "127.0.0.1:6"\n' > /tmp/.t2225-verif.toml; FLEET_DM_CANARY_TRANSIENT=t2225-trans bash scripts/check-fleet-doorbell-mail-health.sh --hubs-file /tmp/.t2225-verif.toml --no-heartbeat
# laptop-141 is declared in the canonical transient file:
grep -q '^laptop-141' .context/cron/fleet-dm-canary-transient` instead of a Human AC here. Only keep [REVIEW] if
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

**Symptom:** The `fleet-doorbell-mail-canary` fired DRIFT (exit 1) on every daily
run because `laptop-141` — a WSL-on-Windows laptop that is frequently powered
off — was unreachable (No route to host). `/canaries` showed permanent FIRING,
which trains operators to ignore the canary and thereby masks a *real*
production-hub failure when one eventually occurs.

**Root cause:** Two compounding gaps. (1) The canary treated every `hubs.toml`
profile as a permanent production hub — an unreachable transient laptop counted
toward `unreachable_count` identically to a down production hub, flipping the
fleet verdict to DRIFT. (2) A latent stderr leak: the T-1892 dedup helper's
informational `skipping duplicate` line was captured by the cron's
`--quiet >> log 2>&1`, so even a fully-healthy run wrote bytes to the log —
already partially breaking the "empty log = healthy" contract.

**Why structurally allowed:** `hubs.toml` has no notion of host criticality or
expected-transience, and the canary's roster was simply "every profile" with no
expected-down classification. The `--quiet` contract ("silent unless drift") was
never enforced against helper-emitted stderr, only against the script's own
stdout.

**Prevention:** (1) Declarative expected-transient host list (git-tracked file
`.context/cron/fleet-dm-canary-transient` UNION `FLEET_DM_CANARY_TRANSIENT` env),
with a `transient_skipped` classification that is visible in output but does NOT
flip `overall_ok`. (2) Regression tests T6 (skip works → exit 0) and T7
(skip-leak guard — an *undeclared* unreachable host still DRIFTs), so a future
edit cannot silently widen the skip. (3) Quiet-mode stderr suppression restores
the "empty log = healthy" contract, verified by reproducing the exact cron
redirection (0 bytes on a healthy run).

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

### 2026-06-13T22:54:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2225-fleet-canary-expected-transient-host-ski.md
- **Context:** Initial task creation

### 2026-06-13T23:01:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
