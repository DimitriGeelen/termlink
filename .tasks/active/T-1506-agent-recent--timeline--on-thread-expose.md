---
id: T-1506
name: "agent recent / timeline / on-thread expose offset in render and JSON"
description: >
  agent recent / timeline / on-thread expose offset in render and JSON

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T06:43:04Z
last_update: 2026-06-07T10:47:40Z
date_finished: 2026-05-05T06:52:16Z
---

# T-1506: agent recent / timeline / on-thread expose offset in render and JSON

## Context

T-1505 shipped `agent quote <offset>` but the operator-fluent path requires `agent recent` / `timeline` / `on-thread` to expose offsets in their renders. Currently they don't — the JSON envelope has no `offset` field and text renders only show `[<age>]`. Fix: add `offset: u64` to `RecentPost` (default 0 when wire envelope lacks the field), populate from `m.get("offset")`, surface in JSON + text headers (e.g. `[1h ago] @316 msg_type=note thread=T-1503`). Closes the read→quote loop.

## Acceptance Criteria

### Agent
- [x] `RecentPost` struct gains `offset: u64` field
- [x] `RecentPost::to_json` includes `offset`
- [x] `extract_recent_posts` populates offset from `m.get("offset")` (default 0 if missing)
- [x] `render_recent_body` shows `@<offset>` token in header line
- [x] `render_timeline_body` shows `@<offset>` token in header line
- [x] `render_on_thread_text` shows `@<offset>` token in header line
- [x] New unit test: extract_recent_posts populates offset from envelope's `offset` field
- [x] New unit test: missing `offset` field defaults to 0 (no panic, no skip)
- [x] All existing channel.rs unit tests still pass
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `agent timeline --window-secs 3600 --n 3` text mode shows `@<n>` tokens
- [x] Live smoke: `agent timeline --window-secs 3600 --n 3 --json` includes `offset` per post

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
- [ ] [REVIEW] Verify offset rendering reads naturally
  **Steps:**
  1. `target/release/termlink agent timeline --window-secs 3600 --n 3`
  2. `target/release/termlink agent recent <peer-fp> --window-secs 3600 --n 3` (any peer from the timeline output)
  **Expected:** each post header line includes `@<offset>` (e.g. `[5m ago] @316 msg_type=note ...`).
  **If not:** suggest a different format / position for the offset token.

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release --bin termlink commands::channel::tests::recent_posts 2>&1 | tail -3 | grep -qE "test result: ok"
out=$(target/release/termlink agent timeline --window-secs 3600 --n 3 --json 2>&1); echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); assert all('offset' in p for p in d.get('posts',[])); print('OK')" | grep -q OK
target/release/termlink agent timeline --window-secs 3600 --n 3 2>&1 | grep -q '@'
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
**Rationale:** Closes the read→quote loop opened by T-1505. Operator can now see `@<offset>` in any of the four reading verbs (recent, timeline, on-thread, plus the JSON envelope) and feed that directly into `agent quote <offset>`. Backwards compatible: synthetic test envelopes without `offset` default to 0 (covered by new test). Format `[<age>] @<offset> msg_type=...` keeps the existing visual rhythm — bracketed age stays the leftmost token, `@n` is a compact addition.
**Evidence:**
- 26/26 unit tests pass (2 new: offset_populated_from_envelope + offset_defaults_zero_when_missing)
- Build clean
- Live smoke text: timeline shows `@315`, `@316`, `@317` (real offsets from the arc)
- Live smoke JSON: every post carries `offset` field
- Verification gate 4/4 passed (build + tests + JSON + text)

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

### 2026-06-07T07:35Z — Human AC fresh re-smoke for [REVIEW] click [agent autonomous]

Per `[Fresh re-smoke before rubber-stamp]` memory: task is ~33 days old. Ran AC steps verbatim against live arc:

```
$ termlink agent timeline --window-secs 3600 --n 3
[32m ago] [9219671e] @2787 msg_type=note project=proxmox-ring20-management
    @skills-manager-agent — directive D-2026-0607-alert-dispatch-dedup...
[29m ago] [d1993c2c] @2788 msg_type=chat thread=T-1438 project=010-termlink

$ termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 3600 --n 3
[29m ago] @2788 msg_type=chat thread=T-1438 project=010-termlink
```

**Both verbs render `@<offset>` in the header line, exact format match to AC's expected `@316` example.** The `@2787` / `@2788` offsets are also addressable via `agent quote <offset>` — the read→quote loop is closed as designed.

Box ready to tick.

### 2026-05-05T06:43:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1506-agent-recent--timeline--on-thread-expose.md
- **Context:** Initial task creation

### 2026-05-05T06:52:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
