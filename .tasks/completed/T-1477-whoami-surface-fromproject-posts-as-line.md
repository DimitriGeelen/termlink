---
id: T-1477
name: "whoami: surface from_project posts-as line (T-1448 finishing touch)"
description: >
  whoami: surface from_project posts-as line (T-1448 finishing touch)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/metadata.rs]
related_tasks: []
created: 2026-05-04T11:11:43Z
last_update: 2026-05-04T11:14:25Z
date_finished: 2026-05-04T11:14:25Z
---

# T-1477: whoami: surface from_project posts-as line (T-1448 finishing touch)

## Context

T-1448 campaign closer. T-1472 (sender) + T-1473 (render) + T-1474
(addressing) shipped the from_project / to_project disambiguation pattern.
But operators running `termlink whoami` still see the FP and cwd without
the resolved `from_project` — they have to infer it from the cwd path.

This adds one line to whoami output: `Posts as:    from_project=<project>`
when the queried session's cwd resolves to a `.framework.yaml` with a
`project_name`. Same resolver as `channel post` auto-injection — single
source of truth. JSON adds a `posts_as: { from_project: "<project>" }`
block.

The line answers: "If I post to chat-arc from this session, what
from_project will be stamped?" — letting the operator catch
co-resident-disambiguation surprises before they happen.

## Acceptance Criteria

### Agent
- [x] `termlink whoami` plain output gains a `Posts as:    from_project=<project>` line under Cwd when the queried session's cwd resolves a `.framework.yaml` with `project_name`
- [x] When no `.framework.yaml` or no `project_name`: the line is omitted (verified — t1700 sessions on /opt/999-AEF show no Posts as line)
- [x] `termlink whoami --json` adds a `posts_as: { from_project: "<project>" }` block when resolvable; absent otherwise (verified)
- [x] Resolver reused — `channel::resolve_project_name_from` exposed `pub(crate)`; metadata.rs calls it via `super::channel::resolve_project_name_from`
- [x] `cargo build -p termlink` succeeds (8.39s)
- [x] Smoke: `target/debug/termlink whoami --session tl-7zlfowtz` (termlink-agent, cwd /opt/termlink) shows `Posts as: from_project=010-termlink` ✓
- [x] Smoke (negative): `target/debug/termlink whoami --session tl-i7wmcek4` (cwd /opt/999-Agentic-Engineering-Framework, no .framework.yaml in chain) shows NO `Posts as:` line ✓
- [x] Smoke (incidental): `target/debug/termlink whoami --session tl-kr4ulsog` (framework-agent, cwd /root) shows `Posts as: from_project=root` because /root/.framework.yaml exists and declares project_name=root — confirms walk-up logic works

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

cargo build -p termlink
target/debug/termlink whoami --session tl-7zlfowtz > /tmp/t1477-w.txt 2>&1; grep -q 'from_project=010-termlink' /tmp/t1477-w.txt
target/debug/termlink whoami --session tl-i7wmcek4 > /tmp/t1477-w2.txt 2>&1; ! grep -q 'Posts as' /tmp/t1477-w2.txt
target/debug/termlink whoami --session tl-7zlfowtz --json > /tmp/t1477-j.json 2>&1; python3 -c "import json; d=json.load(open('/tmp/t1477-j.json')); assert d.get('posts_as',{}).get('from_project')=='010-termlink', d"

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

### 2026-05-04T11:11:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1477-whoami-surface-fromproject-posts-as-line.md
- **Context:** Initial task creation

### 2026-05-04T11:14:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
