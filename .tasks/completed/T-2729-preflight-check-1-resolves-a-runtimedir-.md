---
id: T-2729
name: "preflight Check 1 resolves a runtime_dir the binary never uses, giving a false
  PASS"
description: >
  preflight Check 1 resolves a runtime_dir the binary never uses, giving a false PASS

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [scripts/substrate-preflight.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T08:51:29Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-15T09:03:04Z
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
  - ts: '2026-08-18T18:56:57Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2729: preflight Check 1 resolves a runtime_dir the binary never uses, giving a false PASS

## Context

Backlog rank 3 from `.context/upstream/herdr-adoption-backlog.md` (T-2725
research). Check 1 exists for one reason — PL-021 prevention: catch a
`runtime_dir` that does not survive reboot, before the hub silently regenerates
its secret and TLS cert on every boot and every client sees auth-mismatch.

`scripts/substrate-preflight.sh:240` resolves that directory as:

```sh
local rd="${TERMLINK_RUNTIME_DIR:-/tmp/termlink-0}"
```

The binary resolves it differently — `crates/termlink-session/src/discovery.rs:10-26`:

1. `$TERMLINK_RUNTIME_DIR`
2. `$XDG_RUNTIME_DIR/termlink`
3. `$TMPDIR/termlink-$UID`
4. `/tmp/termlink-$UID`

The script implements step 1 and a hardcoded misspelling of step 4. Steps 2 and
3 are absent, and the UID is pinned to `0`.

**Consequence.** Whenever `TERMLINK_RUNTIME_DIR` is unset and `XDG_RUNTIME_DIR`
is set — the default on any systemd host with a user session, i.e. the common
case — the hub uses `/run/user/<uid>/termlink` while the check inspects
`/tmp/termlink-0`. The check is not merely inaccurate; it is reading an
unrelated path. And `/run/user/*` fails the `case` on line 242, so it takes the
`*)` branch and reports:

> PASS — "not on /tmp — persists across reboot"

`/run/user/<uid>` is a tmpfs that systemd destroys at the user's last logout —
sooner than reboot, not later. So the check makes its most confident claim
precisely where it is most wrong, on the failure mode it was written to catch.

Same defect class as T-2726 (Check 5 called a 581-commit-stale hub "fresh"),
T-2680 and T-2709: **a guard whose verdict rests on an assumption about its
input that no longer holds.** Related: PL-002 — "pre-flight liveness checks must
verify the service responds, not just that the file exists".

Two parts, one root cause (the script's model of `runtime_dir` diverges from the
binary's), kept in one task because part B is unreachable until part A lands:

- **A.** Replicate `discovery.rs`'s resolution order, including `$UID`.
- **B.** Teach the volatility classifier that `/run/user/*` and any tmpfs mount
  are volatile — a PASS must mean "verified persistent", not "not /tmp".

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Preflight resolves `runtime_dir` by the same four-step order as
      `discovery.rs:10-26`, honouring `$XDG_RUNTIME_DIR`, `$TMPDIR` and the real
      `$UID` — no hardcoded `/tmp/termlink-0`
- [x] A `runtime_dir` under `/run/user/*` is classified VOLATILE (fail), not
      PASS — with a message naming logout, not only reboot, as the trigger
- [x] Volatility is decided by the actual mount type of the resolved directory
      (tmpfs ⇒ volatile) rather than by path prefix alone, so an unanticipated
      tmpfs location cannot inherit a false PASS the way `/run/user` did
- [x] The PASS message states the evidence for persistence (which path, which
      filesystem), so a reader can tell a verified PASS from an unexamined one
- [x] Fixture tests drive the resolver and the classifier over: explicit
      override, XDG set, TMPDIR set, bare fallback with non-zero UID, and a
      tmpfs path — and FAIL against the pre-fix script
- [x] `bash scripts/substrate-preflight.sh` still runs end-to-end and its other
      five checks are unchanged in verdict

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

# 17 hermetic assertions over the resolver and the classifier. No root, no
# mounting, no host state — driven through the TERMLINK_PREFLIGHT_TEST_* seams.
bash tests/substrate-preflight-runtime-dir-fixtures.sh

# The whole guard layer, which the fixture suite joins by naming convention.
bash scripts/run-guard-layer.sh

# The hardcoded resolution must not come back. Comments are stripped first:
# this file now *documents* the old expression in two places, and prose about a
# defect is not the defect — the same distinction the silent-exit and
# charter-drift checks make. Capture-then-grep per L-387.
src=$(grep -v '^[[:space:]]*#' scripts/substrate-preflight.sh); ! echo "$src" | grep -q 'TERMLINK_RUNTIME_DIR:-/tmp/termlink-0'

# The script must still run end-to-end on a real host (0 = pass, 1 = warn are
# both fine here; 2 would mean a check hard-failed). `|| rc=$?` is required:
# P-011 runs under `set -e`, so a bare `cmd; rc=$?` never reaches the
# assignment when cmd is non-zero — it kills the shell first. Sibling trap to
# L-387's SIGPIPE.
rc=0; bash scripts/substrate-preflight.sh >/dev/null 2>&1 || rc=$?; [ "$rc" -le 1 ]

## RCA

**Symptom.** `/preflight` Check 1 reported `PASS — "not on /tmp — persists
across reboot"` for `runtime_dir`s that are in fact destroyed at logout, and on
the common systemd configuration reported on `/tmp/termlink-0`, a path the hub
never opens. Verified live: this host's mount table carries
`tmpfs on /run/user/1000 type tmpfs`, and running the pre-fix script under a
fixture where `XDG_RUNTIME_DIR=/run/user/1000` yields a verdict about
`/tmp/termlink-0` — a directory unrelated to the one the binary would use.

**Root cause.** The check re-implemented `runtime_dir` resolution instead of
mirroring `crates/termlink-session/src/discovery.rs:10-26`, and captured only
two of its four steps — with the fallback's `$UID` frozen at `0`. Separately,
volatility was inferred from the path *spelling* (`/tmp*`, `/var/tmp*`) rather
than from the filesystem, so `/run/user/<uid>` — a tmpfs by design — fell
through to the branch that asserts persistence.

**Why structurally allowed.** Three compounding reasons.

1. *The duplicated constant was never linked to its source.* Nothing connects
   the shell literal to `discovery.rs`, so the binary's resolution order could
   change — or, as here, always have been richer — with no signal. Same shape as
   T-2728's two copies of `strip_ansi_codes`, one line of abstraction up.

2. *The check had no tests at all.* `scripts/` carried fixture suites for the
   static checks, but `substrate-preflight.sh` — six checks, gating deploys —
   had none, so no assertion existed that could disagree with it.

3. *It was correct exactly where it did not matter.* On a host that sets
   `TERMLINK_RUNTIME_DIR` explicitly, step 1 wins and script and binary agree.
   That is a host someone has already fixed. The check's entire audience is
   hosts that have *not* been configured — precisely the population where it
   silently read the wrong path. So routine use on a healthy host would never
   surface it, and this one was found by reading the code against the binary's,
   not by anything running.

**Prevention.** Distinct from the fix:

- `tests/substrate-preflight-runtime-dir-fixtures.sh` — 17 assertions covering
  each resolution step, precedence between them, and mount-type classification.
  Proven load-bearing: **11 of the 17 fail against the pre-fix script**, and it
  joins the guard layer automatically by `*fixtures*` naming, so CI runs it on
  every push (T-2686).
- A `## Verification` grep that fires if `${TERMLINK_RUNTIME_DIR:-/tmp/termlink-0}`
  is ever reintroduced.
- Fixture 6 generalises the class rather than patching the instance: it asserts
  a tmpfs at an arbitrary path (`/opt/scratch`) is caught. Fixing only
  `/run/user` would have left the next unanticipated tmpfs location free to
  inherit the same false PASS.
- The PASS message now carries its evidence (`on xfs (disk-backed…)`), so a
  reader can distinguish a verified PASS from an unexamined one — the property
  whose absence made this invisible in the first place.

**Note on scope.** Part B found a second latent bug during fixture work: the
mountpoint matcher never selected `/`, because `"$mp"/*` expands to `//*` for
root, so every path on the root filesystem classified as `unknown`. Caught by
fixture 1 asserting the PASS names a filesystem — an assertion written for the
"evidence" AC, which then earned its keep immediately by failing for an
unrelated reason.

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

### 2026-08-15T08:51:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2729-preflight-check-1-resolves-a-runtimedir-.md
- **Context:** Initial task creation

### 2026-08-15T09:03:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
