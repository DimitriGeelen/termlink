---
id: T-1869
name: "Framework upgrade dispatch — fw upgrade + full test suite + classify findings"
description: >
  Framework upgrade dispatch — fw upgrade + full test suite + classify findings

status: issues
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-29T12:28:12Z
last_update: 2026-05-29T21:09:09Z
date_finished: null
---

# T-1869: Framework upgrade dispatch — fw upgrade + full test suite + classify findings

## Context

Dispatched framework-upgrade task. Project shape detected as `consumer-initialized`
(`.framework.yaml` + `.agentic-framework/bin/fw` present, no root `FRAMEWORK.md`).
Upstream per dispatch: `https://github.com/DimitriGeelen/agentic-engineering-framework.git`
(OneDev mirror migrating, do NOT retry against it). Existing knowledge predicts
findings: PL-123 (vendor includes-list gaps) and PL-125 (web-suite 5/7 fail in
consumer context). Standing rule: NEVER edit `.agentic-framework/` source tree
from a consumer install; classify framework bugs as report-to-upstream.

NOTE: /opt/termlink is a TermLink dev box that ALSO consumes AEF. The dispatch
treats it as a consumer install per the consumer-initialized branch — that's
what we're testing.

## Acceptance Criteria

### Agent
- [ ] STEP 1 project-shape detection recorded (consumer-initialized confirmed)
- [ ] STEP 2 pre-upgrade snapshot captured (fw version, upstream config)
- [ ] STEP 3 `fw upgrade` completes (or failure captured in structured envelope)
- [ ] STEP 4 `fw doctor` runs to completion; warnings/failures recorded
- [ ] STEP 5 `fw test all` runs; per-suite pass/fail counts captured
- [ ] STEP 6 findings classified (framework-bug / termlink-bug / environmental)
- [ ] STEP 7 report-back summary posted (host, before/after, suites, findings)
- [ ] Any framework-bug findings filed as separate tasks (one-bug-per-task rule)
- [ ] PL-123 / PL-125 confirmed-or-disproved against this run (knowledge freshness)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
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

### 2026-05-29T12:28:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1869-framework-upgrade-dispatch--fw-upgrade--.md
- **Context:** Initial task creation

### 2026-05-29T12:50Z — dispatch executed, results

**STEP 1 shape:** `consumer-initialized` confirmed.
**STEP 2 snapshot:** fw v1.6.160, upstream_repo=`DimitriGeelen/agentic-engineering-framework`, version=1.6.160, upgraded_from=1.6.260, last_upgrade=2026-05-15.
**STEP 3 fw upgrade:** **FAILED — second SEV-1 repro of T-1699 fork-bomb.** Evidence appended to T-1699; no new bug filed (one-bug-per-task rule). New datum: recursion target shifted from `/opt/termlink` (last run) to `/opt/003-Vailliant-diagnosis` (this run) — peer-project leak through cloned-upstream path resolution.
**STEP 4 fw doctor:** 4 warnings, 0 failures — all pre-existing (path ambiguity, 72 stale tasks, host-level 347MB global install, mirror divergence).
**STEP 5 fw test all:** mixed.
- Bats unit: ERROR — `tests/unit` not vendored (PL-123 confirmed; addressed by my own T-1867).
- Web pytest: **136 PASS / 1 FAIL / 8 SKIP** — failure is `TestPatterns::test_patterns_cards_link_to_tasks`, environmental (asserts `/tasks/T-` substring in /patterns HTML, /patterns rendered without task links — test-data dependency on having patterns with task refs).
- Playwright: **0 pass / 3 FAIL / 444 ERROR.** All 3 FAIL + 444 ERROR share the same root cause — `ModuleNotFoundError: No module named 'web'` because conftest spawns `python3 -m web.app --port 3099` with `cwd=project_root`, but in consumer install `web/` lives inside `.agentic-framework/`, not at project root. **PL-125 confirmed live with quantification (last value was 5/7 — actual is 447/447 web tests blocked).**

**STEP 6 classification:**
1. **Framework bug** — fw upgrade fork-bomb. Already T-1699 (SEV-1, started-work). Evidence appended.
2. **Framework bug** — playwright conftest assumes `web/` at project root. Already PL-125. Fix would set `cwd=FRAMEWORK_ROOT` or PYTHONPATH-augment. Not filing a new task — PL-125 already in learnings.yaml; this is consumer-context test config drift covered by my own T-1867 line of work (extend consumer test infra).
3. **Framework bug** — bats unit tests not vendored. Already PL-123. Covered by T-1867 + T-1866 (vendor scripts/skills, conceptually extends to tests).
4. **Environmental** — TestPatterns assertion. Skip — test-data dependent, doesn't repro on framework's own data.
5. **Environmental** — 96% disk usage on /. Pre-existing.

**STEP 7 report:** posted in conversation transcript.

**No source edits** to `.agentic-framework/` per dispatch rule. All findings flow to upstream via T-1699 evidence pile + T-1867 vendor-includes path.

### 2026-05-29T12:41:52Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** fw upgrade SEV-1 fork-bomb (T-1699 repro #2); test suite blocked by PL-125 (web pkg path) and PL-123 (bats unit not vendored)

### 2026-05-29T~12:58Z — dispatch re-executed at user request (option 2)

Re-ran the full dispatch verbatim. All findings reproduce identically:
- STEP 3: fork-bomb confirmed (T-1699 repro #3 in two days).
- STEP 4: doctor 4W/0F (identical).
- STEP 5: bats ERR / web 136P/1F/8S / playwright 3F/444E (byte-identical totals).

**One new datum on the wrong-target propagation:** this run the children
all targeted `/opt/termlink` (correct). The 12:30Z run targeted
`/opt/003-Vailliant-diagnosis` (wrong). **The shift is intermittent**
— same .framework.yaml, same .agentic-framework/, same operator, same
hour, different outcome. Suggests a TOCTOU on a global registry or a
cloned-upstream-side `find_project_root` walk that depends on
working-directory state at clone time. Worth a deeper upstream
investigation (separate from the fix recipe, which is independent of
this question).

Posted follow-up envelope to framework.upgrade.report on all 5 hubs
referencing 41f108b1 + this rerun datum.

### 2026-05-30T01:00Z — STEP 3 unblocked structurally — T-1699 fix landed upstream

framework-agent landed the T-1699 fork-bomb fix as upstream T-2099
(commit `be72baa5`) + follow-on T-2100 (`f11e3c4a`). The two-line
patch (env-scoped handoff in `lib/upgrade.sh` + caller-FRAMEWORK_ROOT
honour in `bin/fw`) matches the T-1699 recipe 1:1.

**Implication for T-1869.** The dispatch's STEP 3 (`fw upgrade`) was
blocked by T-1699 fork-bomb on both 2026-05-29 runs. With the fix now
upstream, a fresh dispatch re-run would clear STEP 3 — but **executing
fw upgrade is operator-gated** (consequential action, fork-bomb risk
if the fix has a regression we haven't anticipated). Suggested
operator verification:

```
cd /opt/termlink
.agentic-framework/bin/fw upgrade
# Should complete normally. If fork-bomb resumes:
#   pkill -TERM -f 'fw-upstream'
```

After verification:
- STEP 3 AC ticks
- T-1699 closes (its lone open AC was "fw upgrade completed")
- T-1867 propagation gets end-to-end-tested in the same flow — the
  doorbell+mail toolkit's 9 skills + 11 scripts arrive at the consumer
  via `lib/upgrade.sh:7b/10` (the T-1867 block).

**One run, three closures.** That's the cleanest sequencing — re-run
the dispatch when convenient.

**STEPS 4/5/6/7 remain re-runnable.** Even if STEP 3 is deferred, the
other steps (doctor, test all, classify, report) can run independently.
The findings from 2026-05-29 (PL-123 bats not vendored, PL-125 playwright
web/ path) are already filed and don't need re-classification unless
fresh data emerges.

**T-1869 disposition.** Stays in `issues` until STEP 3 verifies. Adding
this update so the next-session pickup sees the upstream landing
without re-investigating.
