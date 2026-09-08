---
id: T-2696
name: "comms-selftest and substrate-smoke are executed by nothing"
description: >
  Two of the four charter-verb affirmative provers are referenced only in comments
  and docs. Wiring them to cron needs a prerequisites-absent (exit 2) contract first,
  or they fire on a quiet host rather than on breakage (T-2694 F2/G3).

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [scripts/canary-status.sh, scripts/check-cron-install-drift.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T08:01:43Z
last_update: 2026-09-08T07:47:50Z
date_finished: 2026-09-08T07:47:50Z
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
  - ts: '2026-09-08T07:43:21Z'
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
    rubric_sha: e4a00f38e801
---

# T-2696: comms-selftest and substrate-smoke are executed by nothing

## Context

Scoped 2026-09-07 (ground truth re-measured — the filing still holds):

- **Nothing executes either prover.** `grep -rln` over `.context/cron/`,
  `.github/workflows/`, `/etc/cron.d/termlink-*`: every hit is a comment or the
  provers' own files. The stuck-claims crontab MENTIONS substrate-smoke in prose only.
- **comms-selftest already has a hermetic harness** — `scripts/test-comms-selftest.sh`
  (T-2482) drives all stages via canned fixtures (`TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON`
  + `COMMS_SELFTEST_TEST_SEND_RC`), no live hub. But it sits in `scripts/`, and the
  guard-layer runner (T-2684) picks up fixture suites by the `tests/*fixtures*.sh`
  naming convention — so it is hermetic AND executed by nothing. The cheapest honest
  close for the comms half is a thin `tests/comms-selftest-fixtures.sh` wrapper exec'ing
  it: guard layer runs it on every push/PR via T-2686 CI, no new canary, no peer
  intrusion (T-2486 explicitly warned against manufacturing a 14th canary; a daily
  cron that proof-pings a live peer is intrusive and half-overlaps T-2387/T-2295).
- **substrate-smoke has NO hermetic harness and its live run is the value** — so its
  half wants the T-2557 session-control canary pattern: a wrapper canary that runs the
  prover and TRANSLATES the verdict. Critical trap measured today: smoke's `create`
  stage on an unreachable hub → `stage_fail` → **exit 1**, i.e. the prover reports
  BROKEN on a quiet/hub-down host. That is precisely the T-2694 F2/G3 objection this
  task's description records. The wrapper must therefore precheck hub reachability
  itself (cheap RPC) and exit 2 (non-firing, /preflight territory) when unreachable,
  BEFORE invoking smoke — smoke's own exit-2 arm covers only usage/missing-dep.
- Smoke is cron-safe otherwise: self-reaps its `smoke:*` topic on every exit path
  (T-2754 trap), ~seconds of runtime, bounded retention (messages:100).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `tests/comms-selftest-fixtures.sh` exists and exec's `scripts/test-comms-selftest.sh`,
      so the guard-layer runner (tests/*fixtures*.sh convention) and the T-2686 CI job both
      execute the comms prover's hermetic harness on every run; `bash scripts/run-guard-layer.sh --list`
      names it as a member.
- [x] `scripts/check-substrate-smoke-freshness.sh` exists, mirroring the T-2557 verdict
      split: hub unreachable (own precheck) → exit 2 non-firing; smoke exit 0 → 0;
      smoke exit 1 → 1 FIRING naming the broken stage; smoke exit 2 → 2. `--json`,
      `--quiet`, heartbeat touch, test seams (canned smoke rc + output per PL-213).
- [x] Fixture suite for the canary wrapper (tests/…-fixtures.sh) pins all four verdict
      translations INCLUDING the hub-down→2 remap (the F2/G3 case — the load-bearing leg).
- [x] `.context/cron/substrate-smoke-canary.crontab` written with the `# Installed to:`
      header and the T-2685 split-stream redirect idiom; `check-cron-install-drift.sh`
      sees it (fires MISSING until the human installs — that is the intended signal).
- [x] CLAUDE.md canary section gains the new canary paragraph (18th), same
      empty-log-healthy convention.

### Human
- [ ] [RUBBER-STAMP] Install the smoke-canary crontab:
  **Steps:**
  1. `cd /opt/termlink && sudo cp .context/cron/substrate-smoke-canary.crontab /etc/cron.d/termlink-substrate-smoke-canary && bash scripts/check-cron-install-drift.sh`
  **Expected:** drift check reports OK for the new crontab.
  **If not:** the declared `# Installed to:` path and the cp destination differ — fix the header, not the check.

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

# AC1: the comms wrapper exists, is a guard-layer member, and passes.
test -x tests/comms-selftest-fixtures.sh
bash scripts/run-guard-layer.sh --list > /tmp/.t2696-list 2>&1 && grep -q "comms-selftest-fixtures.sh" /tmp/.t2696-list
bash tests/comms-selftest-fixtures.sh > /tmp/.t2696-comms 2>&1 && grep -q "test-comms-selftest: PASS" /tmp/.t2696-comms
# AC2+AC3: canary exists; the 16-assertion suite pins all four translations incl. hub-down→2.
test -x scripts/check-substrate-smoke-freshness.sh
bash tests/substrate-smoke-canary-fixtures.sh > /tmp/.t2696-canary 2>&1 && grep -q "16 passed, 0 failed" /tmp/.t2696-canary
# The F2/G3 remap directly, not only via the suite: canned unreachable hub => rc 2.
TERMLINK_SMOKE_CANARY_TEST_HUB_RC=1 bash scripts/check-substrate-smoke-freshness.sh --no-heartbeat --quiet > /dev/null 2>&1; test $? -eq 2
# AC4: crontab written with self-declared install path + split-stream idiom; drift check sees it.
grep -q "^# Installed to:.*termlink-substrate-smoke-canary" .context/cron/substrate-smoke-canary.crontab
grep -q "2>> .context/working/.substrate-smoke-canary.log.stderr" .context/cron/substrate-smoke-canary.crontab
bash scripts/check-canary-log-hygiene.sh > /tmp/.t2696-hyg 2>&1
# Either the crontab is already installed, or the drift check names it MISSING —
# both states prove the check SEES it (state-tolerant so the human's finalize re-run passes post-install).
test -f /etc/cron.d/termlink-substrate-smoke-canary || { bash scripts/check-cron-install-drift.sh > /tmp/.t2696-drift 2>&1; grep -q "MISSING: substrate-smoke-canary.crontab" /tmp/.t2696-drift; }
# AC5: CLAUDE.md paragraph present, chain count updated.
grep -q "### Substrate-smoke canary (T-2696" CLAUDE.md
grep -q "all eighteen follow" CLAUDE.md
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

## Recommendation

**Recommendation:** GO
**Rationale:** Both halves shipped exactly as scoped. The comms half joins the guard layer (runs on every push/PR — no new canary, no live-peer intrusion); the smoke half is the 18th cron canary with the F2/G3 hub-down→exit-2 remap fixture-pinned as the first assertion. The remaining step is the standard one-line crontab install every canary in this repo has gone through.
**Evidence:**
- `bash tests/comms-selftest-fixtures.sh` → test-comms-selftest: PASS; `run-guard-layer.sh --list` names both new suites as members
- `bash tests/substrate-smoke-canary-fixtures.sh` → 16 passed, 0 failed (hub-down→2 pinned even with a canned "broken" smoke verdict standing by)
- `check-cron-install-drift.sh` reports the new crontab MISSING — the intended shipped-but-not-yet-installed signal; `check-canary-log-hygiene.sh` clean on the split-stream redirect

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

### 2026-08-14T08:01:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2696-comms-selftest-and-substrate-smoke-are-e.md
- **Context:** Initial task creation

### 2026-08-14T08:01:56Z — status-update [task-update-agent]
- **Change:** horizon: now → next

### 2026-09-08T07:43:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-dc5bc99c
- **Timestamp:** 2026-09-08T07:47:54Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `tests/comms-selftest-fixtures.sh` exists and exec's `scripts/test-comms-selftest.sh`,
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/test-comms-selftest.sh in: `tests/comms-selftest-fixtures.sh` exists and exec's `scripts/test-comms-selftest.sh`,`

### 2026-09-08T07:47:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
