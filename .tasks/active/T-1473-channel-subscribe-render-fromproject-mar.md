---
id: T-1473
name: "channel subscribe: render from_project marker (T-1448 follow-up #2)"
description: >
  channel subscribe: render from_project marker (T-1448 follow-up #2)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T08:34:24Z
last_update: 2026-05-04T08:34:24Z
date_finished: null
---

# T-1473: channel subscribe: render from_project marker (T-1448 follow-up #2)

## Context

T-1472 made `from_project` flow ON the wire (auto-injected on post). This
task makes it useful at READ time: `channel subscribe` non-JSON output
shows `(<project>)` after the sender_id when the envelope carries
`metadata.from_project`. JSON mode is unchanged. This closes the loop
T-1448 Design A opened — the second identity axis is now visible to a
human watching the chat-arc.

## Acceptance Criteria

### Agent
- [x] Pure helper `extract_from_project(&Value) -> Option<String>` mirrors `extract_mentions` shape, reads `metadata.from_project`. Returns None when absent or non-string.
- [x] Pure helper `render_from_project_marker(&str) -> String` returns `" (010-termlink)"` for non-empty input, `""` for empty.
- [x] `cmd_channel_subscribe` non-JSON render shows the marker between `sender_id` and `msg_type` on chat lines AND reaction lines. JSON output is byte-identical to before.
- [x] Unit tests cover: extract returns Some when metadata.from_project is set, None when absent, None when not a string. Marker renders correctly for typical, hyphenated, and empty values.
- [x] Live smoke: `target/release/termlink channel subscribe agent-chat-arc --cursor 200 --limit 4` shows `(010-termlink)` next to recently-injected envelopes.

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

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -qE "Compiling|Finished"
cargo test --release -p termlink --bins from_project 2>&1 | tail -5 | grep -q "test result: ok"

# (rest of file ignored below)
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

### 2026-05-04T08:34:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1473-channel-subscribe-render-fromproject-mar.md
- **Context:** Initial task creation
