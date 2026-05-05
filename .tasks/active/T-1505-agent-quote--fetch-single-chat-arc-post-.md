---
id: T-1505
name: "agent quote — fetch single chat-arc post by offset"
description: >
  agent quote — fetch single chat-arc post by offset

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T06:35:40Z
last_update: 2026-05-05T06:42:40Z
date_finished: 2026-05-05T06:42:40Z
---

# T-1505: agent quote — fetch single chat-arc post by offset

## Context

`agent timeline`, `agent recent`, `agent on-thread` all render lists with `[<offset>]` prefixes — operators frequently pick a specific offset and want to see that single post in isolation (with its parent if it's a reply). `cmd_channel_quote` (T-1346) already does this for any topic. `agent quote <offset>` is a thin wrapper hard-pinning topic to `agent-chat-arc`, so the operator types `agent quote 273` instead of `channel quote agent-chat-arc 273`. Joins the `agent.*` verb namespace; pure dispatch wiring, no new logic.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Quote { offset, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_quote("agent-chat-arc", offset, hub, json)`
- [x] `cargo build --release -p termlink` clean
- [x] `target/release/termlink agent quote --help` shows `<OFFSET>` positional and `--hub` / `--json` flags
- [x] Live smoke: `agent quote <real-offset>` renders the post (e.g. T-1504 announcement at offset 275)
- [x] Live smoke: `agent quote <offset> --json` returns `{topic, child, parent}` envelope
- [x] All existing channel.rs unit tests still pass

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->
- [ ] [REVIEW] Verify `agent quote` is operator-fluent
  **Steps:**
  1. `target/release/termlink agent timeline --window-secs 86400 -n 5` — note an offset
  2. `target/release/termlink agent quote <that-offset>`
  **Expected:** post rendered with parent line if reply, or `(no parent — not a reply)` otherwise.
  **If not:** report unexpected output.

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent quote --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent quote --help 2>&1 | grep -qi "OFFSET"
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
**Rationale:** Thin wrapper (10 LOC across cli.rs + main.rs) over existing `cmd_channel_quote` (T-1346). Closes the read/write namespace consistency gap — operators picking offsets from `agent recent`/`timeline`/`on-thread` now have a verb-namespace-local way to fetch full content. Zero new logic; risk is purely dispatch wiring. Live-smoked against real offset 273 (T-1503 announcement) — text + JSON modes both render correctly. Note: full operator-fluency requires `agent timeline`/`recent` to expose offsets in their render — separate follow-up.
**Evidence:**
- Build clean: `cargo build --release -p termlink` finished
- Verification gate 3/3 passed (build + help shape)
- Live smoke text mode: `agent quote 273` → `[273] d1993c2c3ec44c94 note: T-1503 shipped...`
- Live smoke JSON mode: `agent quote 273 --json` → `{topic, child, parent}` envelope with full payload
- No new tests needed (delegates to T-1346 tested helper)

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

### 2026-05-05T06:35:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1505-agent-quote--fetch-single-chat-arc-post-.md
- **Context:** Initial task creation

### 2026-05-05T06:42:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
