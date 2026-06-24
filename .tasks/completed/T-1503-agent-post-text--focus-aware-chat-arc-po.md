---
id: T-1503
name: "agent post <text> — focus-aware chat-arc post verb"
description: >
  New 'agent post <text>' verb: posts a note to agent-chat-arc with thread + project auto-resolved from .context/working/focus.yaml (current_task) and .framework.yaml (project_name). --thread T-XXX and --project P override. --msg-type defaults to 'note'. Reduces a 6-arg 'channel post agent-chat-arc --msg-type note --metadata thread=T-XXX --metadata from_project=Y --payload ...' invocation to 1-2 args. Aligns with the 'reading verbs' (recent/on-thread/timeline) train: now operators have read AND write verbs at the agent abstraction level.

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-04T22:11:38Z
last_update: 2026-05-20T13:21:41Z
date_finished: 2026-05-04T22:42:00Z
---

# T-1503: agent post <text> — focus-aware chat-arc post verb

## Context

The chat-arc reading verbs (T-1492 recent, T-1493 on-thread, T-1500 timeline) work end-to-end now (T-1502 fix shipped). Missing companion: a focus-aware **write** verb. Today posting requires `channel post agent-chat-arc --msg-type note --metadata thread=T-XXX --metadata from_project=Y --payload "..."` — 6 args. A `agent post "..."` verb that auto-resolves thread from `.context/working/focus.yaml` (`current_task`) and project from `.framework.yaml` (`project_name`) reduces to 1-2 args. Aligns the "agent.*" verb namespace: read (recent/on-thread/timeline/overview) + write (post).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Post` variant: positional `text` (required), `--thread <T>` (optional override), `--project <P>` (optional override), `--msg-type <T>` (default "note"), `--hub <H>`, `--json`
- [x] `cmd_agent_post` resolves --thread from `.context/working/focus.yaml::current_task` if not provided
- [x] `cmd_agent_post` resolves --project from `.framework.yaml::project_name` if not provided
- [x] If both auto-resolves fail (no focus, no framework yaml), task succeeds anyway (just doesn't add the metadata) — defensive
- [x] Posts to `agent-chat-arc` topic via existing channel post path
- [x] Empty text bails with operator-friendly error (not silent post)
- [x] JSON envelope (when --json): {ok, topic, msg_type, thread, project, ts_ms, offset}
- [x] Text mode: prints "Posted to agent-chat-arc thread=T-XXX project=Y — offset=N, ts=ms"
- [x] main.rs propagates new value through AgentAction::Post dispatch
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `agent post "T-1503 smoke test"` (with focus T-1503) lands a note tagged thread=T-1503 project=010-termlink, visible via `agent timeline --grep T-1503`
- [x] Live smoke: `agent post "..." --thread T-9999 --project test` overrides resolve correctly

### Human
- [ ] [REVIEW] Verify `agent post` is a fluent improvement over `channel post`
  **Steps:**
  1. With focus on T-1503 (or any task), run: `target/release/termlink agent post "smoke from REVIEW"`
  2. Verify: `target/release/termlink agent timeline --window-secs 60 --grep "smoke from REVIEW"` shows the post
  **Expected:** Post visible. Operator types fewer args than `channel post agent-chat-arc ...`.
  **If not:** suggest defaults (e.g., msg_type=chat as alternative default), or simpler verb name (e.g., `agent say`).

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

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent post --help 2>&1 | grep -q -- "--thread"
out=$(target/release/termlink agent post "T-1503 verification gate smoke" --json 2>&1); echo "$out" | grep -qE '"thread":"T-1503"'
sleep 1
target/release/termlink agent timeline --window-secs 60 --grep "T-1503 verification gate smoke" --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert len(d['posts']) > 0, 'no post found'; print('OK')" | grep -q OK

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
**Rationale:** Focus-aware chat-arc post verb. Auto-resolves `--thread` from `.context/working/focus.yaml::current_task` and `--project` from `.framework.yaml::project_name` when not provided. Operator-fluent improvement over `channel post agent-chat-arc ...` — every flag is auto-filled from the current session context.
**Evidence:**
- Build clean
- Live smoke: post emits envelope with thread/project metadata correctly resolved from focus
- Override paths (--thread, --project explicit) confirmed working

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

### 2026-05-04T22:11:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1503-agent-post-text--focus-aware-chat-arc-po.md
- **Context:** Initial task creation

### 2026-05-04T22:42:00Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
