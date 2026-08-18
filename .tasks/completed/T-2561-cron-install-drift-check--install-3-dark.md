---
id: T-2561
name: "cron-install-drift check + install 3 dark canaries (shipped≠live, G-069)"
description: >
  cron-install-drift check + install 3 dark canaries (shipped≠live, G-069)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T08:20:25Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-09T08:24:41Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:51Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=2 (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2561: cron-install-drift check + install 3 dark canaries (shipped≠live, G-069)

## Context

Self-critical review of the T-2556/57/58 round found a **shipped ≠ live** gap
(G-069) in that very work: the 3 new canaries were committed as git-tracked
`.context/cron/*.crontab` files but never installed to `/etc/cron.d`, so they would
NEVER fire — false comfort. Confirmed: 23 git-tracked crontabs, only 20 installed;
the 3 dark ones are dead-letter / session-control / stuck-claims. The deeper
blindness (G-019): **nothing detects a git-tracked canary crontab that isn't
installed** — the meta-canary-aliveness (T-1723) checks heartbeats of canaries that
ARE running; it can't see one that was never scheduled. This task builds the missing
detection AND remediates the 3 dark canaries. The check is a standalone
static/preflight check (NOT a new cron — a canary-to-detect-uninstalled-canaries
would itself need installing, recursive).

## Acceptance Criteria

### Agent
- [x] `scripts/check-cron-install-drift.sh` exists, is executable, and for each `.context/cron/*.crontab` parses its own `# Installed to: <path>` header (robust to naming exceptions like `agentic-audit` → `/etc/cron.d/agentic-audit-termlink`) and classifies: MISSING (declared path absent) → FIRES (exit 1, the G-069 class); DRIFT (present but content differs) → non-firing WARNING by default, fires only under `--strict`; OK (present + byte-identical); exit 2 on tooling error
- [x] On a non-`/etc/cron.d` host (dir absent — macOS/dev) the check is informational, not firing (exit 0 with a skip note); a crontab with no `Installed to:` header is skipped with a note
- [x] `--json`, `--quiet`, `--strict` flags
- [x] Test hooks `CRON_DRIFT_SRC_DIR` + `CRON_DRIFT_INSTALLED_DIR` feed fixture dirs so all branches (missing / drifted / clean) are verifiable without touching the real host
- [x] Load-bearing PROVEN: a fixture with a missing crontab fires (exit 1); an all-installed fixture exits 0 (temp-revert the firing gate → the missing fixture stops firing)
- [x] Remediation: the 3 dark canaries (dead-letter, session-control, stuck-claims) are installed to `/etc/cron.d/termlink-*`; after install `check-cron-install-drift.sh` exits 0 on the real host
- [x] CLAUDE.md documents the check in a brief section (deploy-time / preflight tier, distinct from the cron canaries it guards)

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
test -x scripts/check-cron-install-drift.sh
# missing fixture fires (exit 1)
d=$(mktemp -d); mkdir -p "$d/src" "$d/inst"; printf '# x\n# Installed to: /etc/cron.d/termlink-z\n1 1 * * * root echo z\n' > "$d/src/z-canary.crontab"; CRON_DRIFT_SRC_DIR="$d/src" CRON_DRIFT_INSTALLED_DIR="$d/inst" bash scripts/check-cron-install-drift.sh >/dev/null 2>&1; rc=$?; rm -rf "$d"; test "$rc" -eq 1
# installed+matching fixture healthy (exit 0)
d=$(mktemp -d); mkdir -p "$d/src" "$d/inst"; printf '# x\n# Installed to: /etc/cron.d/termlink-z\n1 1 * * * root echo z\n' > "$d/src/z-canary.crontab"; cp "$d/src/z-canary.crontab" "$d/inst/termlink-z"; CRON_DRIFT_SRC_DIR="$d/src" CRON_DRIFT_INSTALLED_DIR="$d/inst" bash scripts/check-cron-install-drift.sh --quiet >/dev/null 2>&1; rc=$?; rm -rf "$d"; test "$rc" -eq 0
# real host: the 3 T-2556/57/58 canaries are installed (0 missing)
test -f /etc/cron.d/termlink-stuck-claims-canary && test -f /etc/cron.d/termlink-session-control-canary && test -f /etc/cron.d/termlink-dead-letter-canary
# real host check has 0 missing (exit 0; pre-existing drifts are non-firing warnings)
bash scripts/check-cron-install-drift.sh --quiet
grep -q "Cron-install-drift check (T-2561" CLAUDE.md

## RCA

**Symptom:** The 3 canaries shipped in the immediately-prior round (T-2556/57/58)
were committed as git-tracked `.context/cron/*.crontab` but never installed to
`/etc/cron.d` — so they would never fire. "Shipped" (code merged) was mistaken for
"live" (actually scheduled): the exact G-069 class the fleet-freshness work exists
to catch, recurring one layer up.

**Root cause:** installing a canary crontab is a manual operator step with no
detection. Committing the file feels like completing the canary, but the file is
inert until copied to `/etc/cron.d`. The meta-canary-aliveness check (T-1723)
verifies heartbeats of RUNNING canaries — it structurally cannot see one that was
never scheduled (no heartbeat ever written).

**Why structurally allowed:** the source-of-truth (git) and the live artifact
(/etc/cron.d) are two copies with no reconciliation check; nothing audited
"is every git-tracked crontab actually installed?" The verification gate (P-011)
ran the canary's fixture tests but never asserted the crontab was scheduled.

**Prevention:** `check-cron-install-drift.sh` is that reconciliation — MISSING fires.
It reads each crontab's self-declared `# Installed to:` header (no naming rule to
drift). Run ad-hoc after committing a canary, or fold into `/preflight`. The 3 dark
canaries are now installed and the check is green (0 missing). It also surfaced 2
pre-existing drifts (fleet-doorbell-mail, substrate-preflight) as non-firing
warnings — a separate finding for a future task.

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

### 2026-08-09T08:20:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2561-cron-install-drift-check--install-3-dark.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-5aa80033
- **Timestamp:** 2026-08-09T08:24:42Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -rf`

### 2026-08-09T08:24:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
