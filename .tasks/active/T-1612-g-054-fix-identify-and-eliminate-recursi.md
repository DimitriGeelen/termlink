---
id: T-1612
name: "G-054 fix: identify and eliminate recursive update-task.sh fork on completion deadlock"
description: >
  G-054 fix: identify and eliminate recursive update-task.sh fork on completion deadlock

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T08:05:18Z
last_update: 2026-05-06T08:05:18Z
date_finished: null
---

# T-1612: G-054 fix: identify and eliminate recursive update-task.sh fork on completion deadlock

## Context

G-054 was registered 2026-05-04 as severity:high after `update-task.sh --status work-completed` deadlocked on flock during T-1472 / T-1473. Operator pattern that emerged: hand-edit the task frontmatter and append "G-054 workaround" to the commit message, bypassing all framework gates (AC, Verification, Recommendation, episodic, outcome back-prop). 14 such workaround commits between 2026-05-04 and 2026-05-05.

**Diagnostic finding (this task):** the bug is **quiescent**.

- 4 synthetic completions in this session (T-1613..T-1616) — all clean, all <3s.
- 10 real completions last session (T-1602..T-1611 via `--skip-sovereignty`) — all clean.
- Most recent "G-054 workaround" commit: 2026-05-05 (T-1504). >24h quiet.
- `keylock_subshell_close_cmd` (T-1493) was vendored into `.agentic-framework/lib/keylock.sh` via T-915 framework refresh. update-task.sh:461 calls it before each verification subshell — closes inherited lock FDs so children cannot keep the lock open after the spawning command exits.

**Conclusion:** the upstream T-1493 keylock-FD fix vendored in via T-915 cured G-054. Local action: ship a project-local smoke regression test so a future regression is caught immediately; record findings in the gap; mark mitigated.

## Acceptance Criteria

### Agent
- [ ] Repro fired: synthetic completion via `update-task.sh T-XXX --status work-completed` runs in <10s without deadlock (N≥3 trials)
- [ ] Smoke test exists at `tests/test_g054_completion_smoke.sh` covering: <10s ceiling, file-move-to-completed, lock-not-held-after
- [ ] Smoke test passes when run directly
- [ ] Smoke test cleans up its synthetic task on EXIT (no `.tasks/active/T-NNNN-g-054-smoke-*` residue)
- [ ] `concerns.yaml` G-054 entry updated with: `last_reviewed`, `mitigated_at`, evidence summary, smoke-test reference
- [ ] G-054 status transitioned `watching` → `mitigated`

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

bash tests/test_g054_completion_smoke.sh
test -x tests/test_g054_completion_smoke.sh
grep -q "^- id: G-054$" .context/project/concerns.yaml
grep -A 30 "^- id: G-054$" .context/project/concerns.yaml | grep -q "status: mitigated"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

**Symptom:** `update-task.sh T-XXX --status work-completed` blocks indefinitely on per-task flock; both parent and recursive child stuck. Operator workaround: hand-edit task file, commit with "G-054 workaround" suffix.

**Root cause (best evidence):** lock-FD leak. `keylock_acquire` opens a per-task lock via `exec N>"$lock_file"` without `O_CLOEXEC`. Children spawned during the work-completed flow (verification subshells, episodic-gen, outcome back-prop) inherit the FD. If any child outlives its `bash -c` wrapper or daemonizes (tooling daemons, e.g. cargo's incremental backend), the inherited FD keeps the lock held even after the parent's `keylock_release` runs. A subsequent `update-task.sh` invocation on the same TASK_ID then blocks forever on `flock -x` — looking like "self-recursion deadlock" because the holder is a leaked FD from the same script's prior invocation.

**Why structurally allowed:** flock by FD does not surface holder identity, so the symptom looked like fresh self-recursion rather than FD leak from a prior run. No regression test guarded the completion-latency contract.

**Prevention:** (1) `keylock_subshell_close_cmd` (T-1493 upstream, vendored via T-915) emits `exec N>&-` for every held FD; update-task.sh:461 invokes it inside every verification subshell so children inherit a closed FD instead of the held one. (2) This task's `tests/test_g054_completion_smoke.sh` pins the <10s completion contract and the lock-not-held-after invariant — any future regression of FD-leak family bugs trips it immediately.

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
**Rationale:** G-054 is quiescent (14+ clean transitions since 2026-05-05). Smoke test pins the contract so a regression is caught immediately rather than via operator pain. Updates concerns.yaml status from `watching` to `mitigated` with evidence — keeps the framework's gap register honest about what's actually live vs already-cured.
**Evidence:** smoke test passes; concerns.yaml G-054 entry updated; this task's diagnostic finding documents the upstream T-1493 fix path.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-06T08:05:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1612-g-054-fix-identify-and-eliminate-recursi.md
- **Context:** Initial task creation
