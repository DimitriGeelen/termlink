---
id: T-1436
name: "T-1429-prereq: identity_fingerprint in SessionMetadata + session.discover"
description: >
  Phase-0 prereq for T-1429 (agent contact verb). Add identity_fingerprint: Option<String> to SessionMetadata; populate from load_identity_or_create() at registration time; ensure session.discover response includes it. Unblocks T-1429 Phase-1 (name-resolution to canonical dm:<a>:<b> topic). Decision documented in T-1429.md.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1429, identity, registration]
components: []
related_tasks: []
created: 2026-05-01T10:12:02Z
last_update: 2026-05-01T10:33:55Z
date_finished: 2026-05-01T10:33:55Z
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
- [x] `SessionMetadata` (registration.rs:170) gains `identity_fingerprint: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]` so legacy registrations on disk continue to deserialize unchanged. Field landed at registration.rs:178-186
- [x] At session registration time, the field is populated by best-effort `load_identity_fingerprint_best_effort()` helper (registration.rs:14-32) which reads `$HOME/.termlink/identity.key` if present and resolves to the fingerprint via `Identity::load_or_create`. Failures (no HOME, missing key, IO error) silently resolve to `None` so test envs and unprivileged contexts don't crash
- [x] `session.discover` shape includes `identity_fingerprint` automatically because the handler serialises the existing `Registration.metadata` struct via serde — no separate handler change needed. Verified by spawning a fresh session post-build: `termlink list --json` returned `metadata.identity_fingerprint = "d1993c2c3ec44c94"` for `t1436-smoke`, matching the agent-chat-arc sender ID for this host (proves the same fingerprint is used as `sender_id` on channel.post and now as the SessionMetadata field — exactly what T-1429 needs)
- [x] Migration test: `session_metadata_legacy_json_without_identity_fingerprint` (new) constructs a JSON without the field and asserts it deserialises with `identity_fingerprint: None` — passes
- [x] Backward compat: pre-T-1436 sessions on disk (e.g. `framework-agent`, `termlink-agent` registered 19h ago) lack the field in their JSON; verified via `termlink list --json` they show no `identity_fingerprint` key — no errors. New sessions (post-T-1436 build) populate it. Live confirmation
- [x] All 19 unit tests in `crates/termlink-session/src/registration.rs::tests` pass — `cargo test --release -p termlink-session --lib registration` returns `19 passed; 0 failed`
- [x] Round-trip serialization test landed: `session_metadata_identity_fingerprint_round_trip` — sets a 64-char hex fingerprint, serialises, deserialises, asserts preserved exactly. Passes

**Pre-existing test failure (NOT introduced by T-1436):** `manifest::tests::test_is_git_repo_on_temp_dir` fails because `/tmp/.git` exists on this host (environmental drift). The test asserts a tempdir is NOT inside a git repo, which fails when `/tmp/.git` exists because `git rev-parse --git-dir` walks parent dirs. Unrelated to this task — bug is in the test's environmental assumption, not in my code. Should be tracked as a separate concern.

### Human

This is owner=agent. No human REVIEW required; the changes are structural/serialization-only. If the human wants to spot-check after merge, the canonical observation is:

```bash
termlink list --json | jq '.[].metadata.identity_fingerprint'
```

Should print one fingerprint per session (or `null` for sessions registered before this task landed).

## Verification

cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink-session --lib registration 2>&1 | grep -q "19 passed"
cargo test --release -p termlink-session --lib registration::tests::session_metadata_identity_fingerprint_round_trip 2>&1 | grep -q "test result: ok"
cargo test --release -p termlink-session --lib registration::tests::session_metadata_legacy_json_without_identity_fingerprint 2>&1 | grep -q "test result: ok"

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

### 2026-05-01T10:13:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-05-01T10:33:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
