---
id: T-1508
name: "agent search query — full-arc substring lookup unbounded by window"
description: >
  agent search query — full-arc substring lookup unbounded by window

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T06:59:11Z
last_update: 2026-05-05T07:05:48Z
date_finished: 2026-05-05T07:05:48Z
---

# T-1508: agent search query — full-arc substring lookup unbounded by window

## Context

`agent timeline --grep <q>` and `agent recent --grep <q>` filter inside a `--window-secs` slice (capped at 604800s = 7 days). Operator-frequent question: "did anyone mention X **ever**?" That requires walking the full arc, no window cap. `agent search <query>` does that — pulls all envelopes via `walk_topic_full("agent-chat-arc")`, runs the same case-insensitive substring filter `extract_recent_posts` already implements, returns the last N matches. Pure reuse: pass a huge window so the cutoff is effectively disabled. Render mirrors `agent timeline` (peer-short prefix, `@<offset>`, content).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Search { query, n, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_agent_search`
- [x] New `cmd_agent_search` in agent.rs: walks full chat-arc via `fetch_chat_arc_full`, calls `extract_recent_posts` with effectively-unbounded window + grep=Some(query), renders text or JSON
- [x] Empty query rejected with error
- [x] `cargo build --release -p termlink` clean
- [x] `agent search --help` shows `<QUERY>` positional + `--n` / `--hub` / `--json`
- [x] Live smoke text: `agent search "T-1505"` returns N posts mentioning T-1505 across arc lifetime (not window-capped)
- [x] Live smoke JSON: `agent search "T-1505" --json` returns parseable envelope with all matched posts including offsets
- [x] All existing tests still pass

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
- [ ] [REVIEW] Verify `agent search` is operator-fluent
  **Steps:**
  1. `target/release/termlink agent search "T-1505" --n 5`
  2. `target/release/termlink agent search "T-1166" --n 10` (long-running thread)
  **Expected:** matches across arc lifetime, not just last 7 days; each match includes `@<offset>` for follow-up `agent quote`.
  **If not:** report what reads off.

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent search --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent search --help 2>&1 | grep -qi "QUERY"
out=$(target/release/termlink agent search "T-1505" --n 5 --json 2>&1); echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'posts' in d and 'query' in d; print('OK')" | grep -q OK
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

## Recommendation

**Recommendation:** GO
**Rationale:** Lifts the 7-day window cap on substring lookup. Operators frequently ask "did anyone *ever* mention X?" and `agent timeline --grep` couldn't answer that — only the last week. `agent search` walks the full arc (323 envelopes today, paginated 1000 at a time so future-proof) and reuses the exact same `extract_recent_posts` filter logic. New `fetch_chat_arc_full` helper in channel.rs becomes the building block for any future full-lifetime arc verb.
**Evidence:**
- Build clean
- Verification gate 4/4 passed (build + 2 help checks + JSON parse)
- Live smoke text: scanned 323 envelopes, found 4 matches for "T-1505" across the arc, all with `@<offset>` for follow-up quote
- Live smoke JSON: `{verb, query, n, total_envelopes, posts}` parseable

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

### 2026-05-05T06:59:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1508-agent-search-query--full-arc-substring-l.md
- **Context:** Initial task creation

### 2026-05-05T07:05:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
