---
id: T-1642
name: "Vendor reviewer policy + PYTHONPATH fix — unblock T-1636 closure"
description: >
  fw reviewer fails in /opt/termlink: (1) ModuleNotFoundError on lib.reviewer.static_scan (PYTHONPATH gap), (2) policy/anti-patterns.yaml + escalation-patterns.yaml absent from vendored .agentic-framework. Copy both yamls from upstream /opt/999-Agentic-Engineering-Framework/policy/ into .agentic-framework/policy/, then patch fw script reviewer dispatch lines to set PYTHONPATH=$FRAMEWORK_ROOT. Goal: unblock T-1636 AC #8 (Reviewer verdict PASS) so the inbox.queued v2 peer-consult feature can close.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-15T22:49:50Z
last_update: 2026-05-15T22:49:50Z
date_finished: null
---

# T-1642: Vendor reviewer policy + PYTHONPATH fix — unblock T-1636 closure

## Context

`fw reviewer T-XXX` in /opt/termlink fails with two distinct symptoms:
1. `ModuleNotFoundError: No module named 'lib'` — `python3 -m lib.reviewer.static_scan` invoked without PYTHONPATH; bare CWD-dependent import. In framework-repo-self CWD=PROJECT_ROOT works (lib/ lives at repo root); in vendored consumer projects lib/ lives at `.agentic-framework/lib/` and is invisible to the python -m loader.
2. `ERROR: catalogue not found at /opt/termlink/policy/anti-patterns.yaml` — the policy catalogues `anti-patterns.yaml` + `escalation-patterns.yaml` exist upstream at `/opt/999-Agentic-Engineering-Framework/policy/` but were never copied into vendored installs.

Local fix: copy both yamls into `.agentic-framework/policy/`, patch the 5 reviewer dispatch lines in `.agentic-framework/bin/fw` to set `PYTHONPATH=$FRAMEWORK_ROOT`. Unblocks T-1636 AC #8 (Reviewer verdict PASS) so the inbox.queued v2 peer-consult feature can close. Channel-1 mirror to upstream once green.

## Acceptance Criteria

### Agent
- [ ] `.agentic-framework/policy/anti-patterns.yaml` present, byte-identical to upstream `/opt/999-Agentic-Engineering-Framework/policy/anti-patterns.yaml`
- [ ] `.agentic-framework/policy/escalation-patterns.yaml` present, byte-identical to upstream
- [ ] All 5 reviewer dispatch lines in `.agentic-framework/bin/fw` (static_scan, audit, override_cli, drift_cli, reverify_cli) prepend `PYTHONPATH=$FRAMEWORK_ROOT` to the env block
- [ ] `.agentic-framework/bin/fw reviewer T-1636` runs without ModuleNotFoundError or catalogue-not-found
- [ ] Reviewer verdict captured (PASS / CONCERN / FAIL) and recorded in this task's Updates
- [ ] If verdict is PASS, T-1636 AC #8 ticked + status work-completed in a follow-on commit
- [ ] Channel-1 mirror: same patches applied to upstream `/opt/999-Agentic-Engineering-Framework` via framework-agent, committed + pushed to onedev

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

test -f .agentic-framework/policy/anti-patterns.yaml
test -f .agentic-framework/policy/escalation-patterns.yaml
grep -c 'PYTHONPATH=' .agentic-framework/bin/fw | grep -q '^[5-9]$\|^[1-9][0-9]'
.agentic-framework/bin/fw reviewer T-1636 --no-write 2>&1 | grep -v 'ModuleNotFoundError\|catalogue not found' | grep -q 'verdict\|Catalogue\|PASS\|CONCERN\|FAIL'

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

### 2026-05-15T22:49:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1642-vendor-reviewer-policy--pythonpath-fix--.md
- **Context:** Initial task creation
