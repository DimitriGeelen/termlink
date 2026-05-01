---
id: T-1440
name: "whoami: surface identity_fingerprint (chat-arc sender_id visibility)"
description: >
  whoami: surface identity_fingerprint (chat-arc sender_id visibility)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/metadata.rs]
related_tasks: []
created: 2026-05-01T16:56:13Z
last_update: 2026-05-01T17:12:40Z
date_finished: 2026-05-01T17:12:40Z
---

# T-1440: whoami: surface identity_fingerprint (chat-arc sender_id visibility)

## Context

`termlink whoami` (metadata.rs:628) prints session info but does NOT surface
`identity_fingerprint` — the field T-1436 plumbed into `SessionMetadata` and
the canonical `sender_id` for chat-arc envelopes. Operators rolling out the
agent-chat-arc protocol can't see their own signing identity from whoami;
they have to dig into `~/.termlink/identity.key` or grep registration JSON.
Surfacing it is a small plumbing win that makes "what fingerprint am I
posting under?" trivially answerable + copy-pasteable into `--target-fp`.

## Acceptance Criteria

### Agent
- [x] `print_whoami_card` displays `Identity FP:` line in text mode when `reg.metadata.identity_fingerprint` is `Some` — placed directly under `PID:` line. metadata.rs:687-689
- [x] When `Some`, JSON mode also includes `identity_fingerprint` under the session object. metadata.rs:651-653
- [x] When `None` (legacy registration), output stays unchanged — `if let Some(fp)` guards both branches. Verified live: `tl-malcnj2m` (pre-T-1436 registration) shows no `Identity FP` line; `tl-3c6tvea2` (post-T-1436) shows `Identity FP:  d1993c2c3ec44c94`
- [x] Refactored to expose `whoami_card_json()` helper so tests assert on `Value` shape without capturing stdout — clean test seam
- [x] Unit tests in metadata.rs: `whoami_card_json_with_identity_fp_emits_field` + `whoami_card_json_without_identity_fp_omits_key`. Both pass: `test result: ok. 2 passed`
- [x] cargo build --release succeeds (binary 0.9.1681); cargo test green

## Verification

cargo build --release --bin termlink 2>&1 | tail -3
cargo test --release -p termlink whoami_card 2>&1 | tail -10

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

### 2026-05-01T16:56:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1440-whoami-surface-identityfingerprint-chat-.md
- **Context:** Initial task creation

### 2026-05-01T17:12:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
