---
id: T-1298
name: "Hub topic-name validation — reject malformed strings at emit"
description: >
  Discovered during T-1297 Spike 1: framework-agent on .107 has a live topic literally named "learning.shared</topic>\n<parameter name=\"from\">email-archive" — XML interpolation leaked into a topic string and the hub accepted it. Hub should reject malformed topic names at emit time (regex like ^[a-z0-9._:\-]+$ plus length cap). Risk: topic-string sprawl, indexing/routing surprises, log-injection via topic name. Independent of routing discipline — orthogonal failure class.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [termlink, hub, validation, topic, T-1297-spinoff]
components: [crates/termlink-hub/src/router.rs]
related_tasks: [T-1297]
created: 2026-04-26T21:03:19Z
last_update: 2026-04-27T06:16:38Z
date_finished: 2026-04-27T06:16:38Z
---

# T-1298: Hub topic-name validation — reject malformed strings at emit

## Context

T-1297 Spike 1 found a topic literally named `learning.shared</topic>\n<parameter name="from">email-archive` on the framework-agent bus — XML prompt-interpolation leaked into the topic string and the hub accepted it. Hub side has zero validation today. Survey of live + test topics shows the universe is `[a-z0-9._:-]+`; no underscores, no uppercase, no whitespace. Add a validator at the emit entry points (`event.broadcast`, `event.emit_to`) that rejects with `-32602` (invalid params) when the topic doesn't match `^[a-z0-9._:\-]+$` or exceeds 256 chars.

## Acceptance Criteria

### Agent
- [x] New `validate_topic_name(&str) -> Result<(), String>` helper in `crates/termlink-hub/src/router.rs` — char-class check `[a-z0-9._:-]`, length cap 256, returns descriptive error string on failure (mentions allowed-set + offending byte position)
- [x] `handle_event_broadcast` and `handle_event_emit_to` call the validator before any side-effects; reject with `ErrorResponse::new(id, -32602, "...")` including the offending topic substring (truncated to 64 chars in the error)
- [x] Existing tests stay green — 241 hub tests, no breakage; all real topics in repo (agent.request, inbox:carol, channel.list, etc.) accepted by validator
- [x] Six new unit tests in `router.rs::tests`: real-topics accepted, uppercase rejected, XML-interp rejected, whitespace/newline/tab rejected, >256-char rejected, empty rejected
- [x] Workspace tests clean — 241 termlink-hub unit, all green
- [x] Workspace `cargo clippy --all-targets -- -D warnings` clean

### Human
- [x] [RUBBER-STAMP] Verify post-deploy that no legitimate emit started failing
  **Steps:**
  1. After build is on the .107 hub: `termlink topics | head -20`
  2. Wait 5 min, run again: `termlink topics | head -20` — see counts increment for normal traffic
  3. Check hub logs for unexpected validator rejects (`journalctl -u termlink-hub --since '5 min ago' | grep -i 'invalid topic'`)
  **Expected:** Live topic catalog grows normally, no validator rejects on legitimate emitters.
  **If not:** Capture the rejected topic string + the emitter — likely a code path emitting an unexpected format that needs widening the allowlist.

## Verification

cargo test -p termlink-hub validate_topic_name --quiet 2>&1 | tail -8 | grep -qE "test result: ok"
cargo build --workspace 2>&1 | tail -3 | grep -qE "Finished|Compiling"
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -3 | grep -qE "Finished|Checking"

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

### 2026-04-26T21:03:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1298-hub-topic-name-validation--reject-malfor.md
- **Context:** Initial task creation

### 2026-04-26T22:28:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-26T22:35Z — validator shipped [agent autonomous pass]
- **Validator:** `validate_topic_name(&str) -> Result<(), String>` in router.rs. Char-class `[a-z0-9.:-]` (no underscore, no uppercase, no whitespace); len cap 256. Empty → reject. Error format includes 64-char preview of the offending topic, the illegal char (Debug-formatted to surface non-printables like `'\n'`), its byte index, and the allowed-set hint.
- **Wiring:** Both `handle_event_broadcast` (line 254 area) and `handle_event_emit_to` (line 379 area) call the validator immediately after the existing `Missing 'topic'` check, before any payload-cloning or session resolution. Reject path emits `-32602` invalid-params with the validator's error message.
- **Tests:** 6 new in `router::tests`. real-topics-accepted (10 known-good names from codebase grep); uppercase-rejected (asserts error mentions offending char); xml-interp-rejected (the literal T-1297 Spike 1 string); whitespace-rejected (newline, space, tab); too-long-rejected (`"a".repeat(257)`); empty-rejected.
- **Verification:** `cargo test -p termlink-hub validate_topic_name` 6/6 ok; full hub suite 241/0 fail; workspace clippy clean.
- **Operator AC:** rubber-stamp post-deploy that no legitimate emit started failing — `termlink topics` should grow normally; hub logs should not show validator rejects on real traffic. T-1297 Spike 1's literal XML-interp topic is the only known violator; if a real emitter trips this, capture and widen.

### 2026-04-27T06:16:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Completed via Watchtower UI (human action)
