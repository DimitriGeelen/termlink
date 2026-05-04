---
id: T-1472
name: "channel post: default from_project metadata injection (T-1448 follow-up #1)"
description: >
  channel post: default from_project metadata injection (T-1448 follow-up #1)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/channel.rs]
related_tasks: []
created: 2026-05-04T08:15:05Z
last_update: 2026-05-04T08:32:16Z
date_finished: 2026-05-04T08:32:16Z
---

# T-1472: channel post: default from_project metadata injection (T-1448 follow-up #1)

## Context

T-1448 inception (GO Design A) identified `from_project` as the second axis the
chat-arc needs to disambiguate co-resident agents that share a host+user
identity key. Field measurement showed only 7% organic adoption — the
inception explicitly chose to **mandate** the convention via cheap CLI
default rather than wait for organic emergence. This task ships pick (a):
the CLI-side default injection.

Sequenced first per the inception order (a → c → b): (a) unblocks (c) by
making the metadata field appear unconditionally; (b) is the operator-visible
payoff, but needs the field to already be in the wire to resolve `<name>:project`.

Source: `docs/reports/T-1448-co-resident-agent-identity-inception.md`,
`crates/termlink-cli/src/commands/channel.rs:265-280` (metadata assembly point).

## Acceptance Criteria

### Agent
- [x] Pure helper `default_from_project()` in CLI walks up CWD looking for `.framework.yaml`; on hit, parses and returns `project_name` value. Returns `None` when no marker found or no `project_name` field. Independent of any global state — tested with tempdirs.
- [x] `cmd_channel_post` injects `from_project=<resolved>` into metadata IFF (a) `from_project` is not already in user's `--metadata` flags, AND (b) helper returns `Some(...)`. User-supplied value wins; absent project root means no injection (silent).
- [x] When posting to a chat-arc topic (`agent-chat-arc`, `dm:*`) and no `from_project` could be resolved AND user did not supply one, emit a one-line stderr warning naming the topic and the gap. Other topics: silent (preserves backward compat for ad-hoc topics).
- [x] Unit tests cover: helper finds `.framework.yaml` at root, finds it via walk-up, returns None when missing, returns None when YAML lacks `project_name`. Injection: user override wins, absent project = no inject, present project = injects. Warning logic: chat-arc + unresolvable = warn, chat-arc + user-supplied = no warn, other topic + unresolvable = no warn.
- [x] `docs/conventions/multi-turn-dialog.md` gets a `## Well-known metadata keys` section listing `from_project` (this task), `_thread`, `_from`, `in_reply_to` (existing) — sets the catalog T-1448 referenced.
- [x] `cargo test --release -p termlink --bins from_project` passes; `cargo build --release -p termlink` builds clean.
- [x] Live smoke: post to local hub `agent-chat-arc` with no `--metadata from_project=...`; verify `channel info` / events stream shows the injected metadata field. Override smoke: post with explicit `--metadata from_project=otherval`; verify the user value wins.

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

cargo build --release -p termlink 2>&1 | tail -5 | grep -qE "Compiling|Finished|warning"
cargo test --release -p termlink --bins from_project 2>&1 | tail -5 | grep -q "test result: ok"
grep -q "from_project" docs/conventions/multi-turn-dialog.md

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

### 2026-05-04T08:15:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1472-channel-post-default-fromproject-metadat.md
- **Context:** Initial task creation

### 2026-05-04T08:32:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
