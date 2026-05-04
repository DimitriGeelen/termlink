---
id: T-1464
name: "Close G-048 (T-1388 fix landed) + rate-vs-window-rolloff caveat in T-1166 doc"
description: >
  Close G-048 (T-1388 fix landed) + rate-vs-window-rolloff caveat in T-1166 doc

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T05:48:02Z
last_update: 2026-05-04T05:48:02Z
date_finished: null
---

# T-1464: Close G-048 (T-1388 fix landed) + rate-vs-window-rolloff caveat in T-1166 doc

## Context

Two small hygiene items bundled:

1. G-048 (high) was opened 2026-04-28 after a base64-encoded secret leaked
   through `termlink remote push` error output. T-1388 shipped redaction
   with 5 unit tests in commit history; status remained "watching". Per
   CLAUDE.md G-019, gap must not close until prevention exists — it does.
   Mark resolved.
2. T-1462's diff feature reports rate as `total_fleet_delta / elapsed_min`.
   When the elapsed interval crosses the audit-log retention window
   (currently rolling 60-90d depending on hub), some of the apparent
   "decay" is just window roll-off (calls older than the window dropped),
   not migration progress. For short intervals (< 1d) this is negligible.
   For multi-day intervals operators need to know to read the rate
   conservatively. Add a one-paragraph caveat to the Decay-rate sampling
   subsection.

## Acceptance Criteria

### Agent
- [x] G-048 status updated to `resolved` in `.context/project/concerns.yaml` with `resolved: 2026-05-04` and a `resolution:` referencing T-1388 + grep evidence.
- [x] `fw gaps` shows G-048 resolved: 7 watching, 11 resolved (was 8/10).
- [x] T-1166 migration doc gains a "Rate interpretation caveat" paragraph plus an interval-choice table under "Decay-rate sampling".
- [x] No source-code changes (admin metadata + doc only).

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

grep -A 3 "id: G-048" .context/project/concerns.yaml | grep -q "status: resolved"
grep -q "Rate interpretation caveat" docs/migrations/T-1166-retire-legacy-primitives.md
.agentic-framework/bin/fw gaps 2>&1 | grep -v "G-048" | grep -q "watching"
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

### 2026-05-04T05:48:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1464-close-g-048-t-1388-fix-landed--rate-vs-w.md
- **Context:** Initial task creation
