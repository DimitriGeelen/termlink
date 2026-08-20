---
id: T-2693
name: "Repo-wide L-387 verification-pipefail auditor (Pen request + 150 live hits
  here)"
description: >
  The framework detects the pipe-to-grep-q SIGPIPE shape only as a non-blocking advisory
  at started-work, one task at a time. There are 150 live instances across 188 task
  files
  in this repo, each able to fail P-011 spuriously and push operators toward --force.
  Peer
  project 050-email-archive asked for a standalone detector as their Option D structural
  control.
status: started-work
workflow_type: build
horizon: now
owner: claude-code
created: 2026-08-20
last_update: '2026-08-20T15:21:22Z'
tags: [governance, l-387, verification-gate, peer-request, static-check]
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:38Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=2 (body:learning-ref); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2693: Repo-wide L-387 verification-pipefail auditor

## Context

L-387: under `set -o pipefail`, `cmd | grep -q PATTERN` exits **141** when the pattern
matches — `grep -q` exits on first match and closes the pipe, SIGPIPE kills the upstream,
and pipefail propagates the upstream's status. The pipeline therefore *fails on success*.

The P-011 verification gate runs each `## Verification` line under `set -euo pipefail`, so
a verification command written in that shape blocks completion of a task whose verification
actually passed.

### What already exists

The framework ships a real detector — `lib/reviewer/static_scan.py::detect_l387_sigpipe_risk`
— and it is good: it resolves the producer as the LAST pipeline stage before the terminal
grep, exempts `echo`/`printf` upstreams (SIGPIPE-immune, bounded buffer), and skips comment
lines. `update-task.sh` wires it as a **non-blocking advisory fired at `started-work`**
(T-2059), one task at a time.

### The gap

The advisory fires at one lifecycle moment for one task. Nothing ever asks the repo-wide
question. Result: **150 findings across 188 task files** with a `## Verification` block,
sitting unaddressed. Sample:

```
T-1296: termlink fleet doctor 2>&1 | grep -q 'ring20-dashboard.*PASS'
T-1426: cargo test --release -p termlink deprecation 2>&1 | grep -q "test result: ok. 2 passed"
T-1426: target/release/termlink event broadcast topic-x 2>&1 | grep -q DEPRECATED
```

### Why this is worth fixing rather than tolerating

The failure direction is "safe" in isolation — the gate blocks rather than waves through.
But a gate that blocks *incorrectly*, 150 times, is not safe in aggregate: it teaches the
operator that P-011 failures are noise and that `--force` is the normal way past them. A
verification gate people routinely force is a verification gate that no longer verifies.
That is the Directive #2 cost, and it is paid quietly.

### Peer request (050-email-archive)

Pen asked on `framework:pickup` (offset 3) for a shareable `check-verification-pipefail.sh`
to wire into their own audit companion as the structural control for their G-VERIFICATION
concern. They independently named the two design points the framework's detector already
implements (strip `$()` spans, resolve producer as the last segment before the pipe). They
explicitly framed it as no-rush and proportional to our workload.

## Approach

Ship `scripts/check-verification-pipefail.sh` that **reuses the framework's own
`detect_l387_sigpipe_risk`** rather than reimplementing the heuristic. One implementation,
no fork, no drift — and Pen gets a wrapper they can drop in beside the framework they
already run, instead of a duplicate that will diverge from it.

Scans every `.tasks/**/*.md` with a `## Verification` block. Reports file, line and the
offending command. Exit 0 clean / 1 findings / 2 tooling. `--json` for scripting, `--quiet`
for cron-style use, `--active-only` to scope to `.tasks/active/` (completed tasks are
history — their verification already ran or was forced, so fixing them changes nothing).

## Acceptance Criteria

### Agent
- [x] Detects the pipe-to-grep-q shape across all task files in one invocation
- [x] Reuses the framework's `detect_l387_sigpipe_risk` — no reimplemented heuristic
- [x] Exempts `echo`/`printf` upstreams and comment lines (inherited from the framework fn)
- [x] Exits 2 (not 0) when the framework detector cannot be imported — a check that cannot
      run must never report "clean"
- [x] `--json` carries per-finding file/line/evidence
- [x] `--active-only` scopes to active tasks
- [x] Reports the live count in this repo
- [x] Regression fixtures cover: risky shape fires; `echo`-upstream safe shape clears;
      comment ignored; missing-detector exits 2
- [x] Fixtures are host-independent (scratch task tree)
- [x] Shared back to 050-email-archive on `framework:pickup`

## Verification

bash tests/verification-pipefail-check-fixtures.sh

## Decisions

**Wrapper, not reimplementation.** Pen offered "paste a canonical implementation into the
channel" as an option. Declined: two copies of a subtle SIGPIPE heuristic will drift, and
the one that drifts is the one that stops catching things. Pointing at the framework
function both projects already ship keeps a single source of truth.

**Fail-closed on import error.** If `static_scan.py` cannot be imported the script exits 2,
never 0. A detector that silently reports clean because it could not load is worse than no
detector — it converts an unknown into a false assurance. (This repo's own
`lib/reviewer/static_scan.py` is tracked, but a consumer's may not be — that is exactly the
T-2689/T-2692 vendoring gap.)

**Active-only is opt-in, not the default.** Completed tasks still carry the shape and are
the best evidence of how widespread it is; hiding them by default would understate the
problem on first run.
