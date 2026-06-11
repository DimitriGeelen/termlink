---
id: T-2146
name: "Canonical substrate-worker-loop.sh recipe script (T-2124 hello-world)"
description: >
  Canonical substrate-worker-loop.sh recipe script (T-2124 hello-world)

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
created: 2026-06-10T21:50:13Z
last_update: 2026-06-10T21:57:30Z
date_finished: 2026-06-10T21:57:30Z
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

# T-2146: Canonical substrate-worker-loop.sh recipe script (T-2124 hello-world)

## Context

T-2124 ships the master orchestrator recipe (docs/operations/substrate-orchestrator-
recipe.md) describing the canonical worker pattern in prose. Operators
adapting the substrate to their workload still hand-write the loop from
scratch each time. Drop a real adaptable script — `scripts/substrate-
worker-loop.sh` — that demonstrates the canonical claim → renew-loop →
release(--ack) lifecycle with proper sigterm cleanup and the same R3
loud-fail behavior the rest of the substrate verbs use. The "hello
world" for substrate users.

Pattern composition: T-2097 `/claim` (acquire), T-2101 `/renew`
(extend-while-busy), T-2098 `/release` (ack=true or --retry for ack=false).
The script picks one offset off a topic (operator supplies the topic +
offset), runs the worker command, renews automatically every ttl/2
during execution, releases at the end. Exit non-zero from the worker
command → release(--retry) so the slot reopens. SIGTERM/SIGINT during
work → graceful release(--retry) before exit. Read-side observability
via existing `/claims`, `/queue-status`, `/governor` — no new verbs.

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-worker-loop.sh` exists, executable (chmod +x), shebang `#!/usr/bin/env bash`
- [x] `bash -n scripts/substrate-worker-loop.sh` passes — syntax ok
- [x] `--help` exits 0, documents `--topic`, `--offset`, `--cmd`, `--ttl-ms`, `--claimer`, `--renew-every-ms`, `--hub` + exit-code table
- [x] Missing required flag (`--topic`, `--offset`, `--cmd`) → exits 2 with `Usage: --X required (see --help)` stderr line
- [x] Successful path verified live on smoke:t2146 offset 0 — claim → worker → release(--ack), exit 0
- [x] Failure path verified live on offset 1 — worker exit 7 → release (no --ack), exit 1; re-claim on offset 1 succeeded confirming slot reopened
- [x] Signal handling wired via trap INT TERM — releases-without-ack, exits 130 (smoke: trap is set; can't unit-test SIGTERM cleanly in CI without flakes)
- [x] Auto-renew background subshell renews every `renew_every_ms` (default ttl_ms/2) with `--additional-ttl-ms $TTL_MS`; renew failures land on stderr and don't kill the loop
- [x] Master recipe doc gains a "Ready-to-adapt script (T-2146)" block at the top of the "Canonical worker pattern" section
- [x] shellcheck clean — `shellcheck scripts/substrate-worker-loop.sh` produces no output

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

test -x scripts/substrate-worker-loop.sh
bash -n scripts/substrate-worker-loop.sh
scripts/substrate-worker-loop.sh --help > /tmp/.t2146.out 2>&1 && grep -q -- "--topic" /tmp/.t2146.out
out=$(scripts/substrate-worker-loop.sh 2>&1 || true); echo "$out" | grep -q "Usage\|required"
grep -q "scripts/substrate-worker-loop.sh" docs/operations/substrate-orchestrator-recipe.md

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

### 2026-06-10T21:50:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2146-canonical-substrate-worker-loopsh-recipe.md
- **Context:** Initial task creation

### 2026-06-10T21:57:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
