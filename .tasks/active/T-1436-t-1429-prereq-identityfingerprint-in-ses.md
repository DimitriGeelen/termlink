---
id: T-1436
name: "T-1429-prereq: identity_fingerprint in SessionMetadata + session.discover"
description: >
  Phase-0 prereq for T-1429 (agent contact verb). Add identity_fingerprint: Option<String> to SessionMetadata; populate from load_identity_or_create() at registration time; ensure session.discover response includes it. Unblocks T-1429 Phase-1 (name-resolution to canonical dm:<a>:<b> topic). Decision documented in T-1429.md.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [T-1429, identity, registration]
components: []
related_tasks: []
created: 2026-05-01T10:12:02Z
last_update: 2026-05-01T10:12:34Z
date_finished: null
---

# T-1436: T-1429-prereq: identity_fingerprint in SessionMetadata + session.discover

## Context

T-1429 (`termlink agent contact <name>`) needs to map a session display-name
→ owner-identity-fingerprint, so it can compute the canonical
`dm:<sorted_a>:<sorted_b>` topic. Today `SessionMetadata`
(crates/termlink-session/src/registration.rs:147) only carries
shell/term/cwd/termlink_version/data_socket — no identity field. Adding
this is the prerequisite that unblocks T-1429's discovery AC and the
self-describe-on-create AC deferred from T-1430. It is structural and
benefits other identity-aware verbs too (T-1427 strict-reject hooks here).

## Acceptance Criteria

### Agent
- [ ] `SessionMetadata` (registration.rs:147) gains `identity_fingerprint: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]` so legacy registrations on disk continue to deserialize unchanged
- [ ] At session registration time (search for current `SessionMetadata { … }` constructions), the field is populated from `load_identity_or_create()?.fingerprint().to_string()`
- [ ] `session.discover` response shape (whichever handler builds the per-session JSON) includes `identity_fingerprint` when present, omits when absent — verified by snapshot test or by inspecting at least one live registration JSON
- [ ] Migration test: an old registration JSON without the field still deserializes into `SessionMetadata { identity_fingerprint: None, … }` — no panic, no error
- [ ] Backward compat: `register`/`spawn` flows that previously didn't set the field still work; tests covering registration construction continue to pass without modification (the field has a default)
- [ ] All existing unit tests in `crates/termlink-session/src/registration.rs::tests` pass
- [ ] At least one new test asserts the round-trip serialization shape — `SessionMetadata { identity_fingerprint: Some("abc123…"), … }` → JSON → back, fingerprint preserved

### Human

This is owner=agent. No human REVIEW required; the changes are structural/serialization-only. If the human wants to spot-check after merge, the canonical observation is:

```bash
termlink list --json | jq '.[].metadata.identity_fingerprint'
```

Should print one fingerprint per session (or `null` for sessions registered before this task landed).

## Verification

cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink-session --lib registration 2>&1 | tail -10
cargo test --release -p termlink-cli --lib 2>&1 | tail -5

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

### 2026-05-01T10:12:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1436-t-1429-prereq-identityfingerprint-in-ses.md
- **Context:** Initial task creation

### 2026-05-01T10:12:34Z — status-update [task-update-agent]
- **Change:** horizon: now → next
