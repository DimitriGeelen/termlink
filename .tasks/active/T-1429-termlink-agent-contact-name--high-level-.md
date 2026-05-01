---
id: T-1429
name: "termlink agent contact <name> — high-level cross-host contact verb (T-1425 pick #2)"
description: >
  From T-1425 inception solo synthesis 2026-04-30. Wraps the discover -> resolve-DM-topic -> post pattern into one verb so vendored agents stop improvising primitives. Replaces the broken pattern that produced the .107-to-.122 ZoneEdit handoff incident. Decisions baked in per T-1425 §Decisions: Q1=A auto-create dm:<sorted>:<sorted>, Q2=C opt-in --ack-required, Q3=C default-queue with --require-online flag, Q5=A retention=forever. Q4 (identity binding) ships in T-1427 separately; this verb relies on it via channel.post when T-1427 lands but works without strict-reject in the meantime. Lives in crates/termlink-cli/src/commands/agent.rs alongside cmd_agent_ask/listen/negotiate. Provisional pending peer replies on T-1425 thread (14d amendment window).

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:26:39Z
last_update: 2026-04-30T21:27:46Z
date_finished: null
---

# T-1429: termlink agent contact <name> — high-level cross-host contact verb (T-1425 pick #2)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent

**CLI surface:**
- [ ] `termlink agent contact <target> [--message <m>] [--file <path>] [--thread <id>] [--ack-required] [--ack-wait <secs>] [--require-online] [--json]` parses correctly via `termlink agent contact --help`
- [ ] At least one of `--message` or `--file` is required; supplying both is allowed (file is the body, message is a one-line subject)
- [ ] `<target>` accepts: bare session name (resolved via discover), `name@hub:port` (explicit hub), or `sender_id:<hex>` (already-known peer identity)

**Discovery + topic resolution:**
- [ ] When `<target>` resolves via `discover`, the verb finds the session and extracts its `sender_id` (or equivalent stable identifier)
- [ ] DM topic name is computed as `dm:<sorted_id_a>:<sorted_id_b>` where `sorted_id_a < sorted_id_b` lexicographically — same canonicalization as T-1319; reuse the existing helper rather than reimplement
- [ ] If the topic doesn't exist on the local hub, the verb auto-creates it with `retention=forever` (T-1425 Q1=A, Q5=A)
- [ ] **Self-describe on create (deferred from T-1430):** when auto-creating a `dm:<a>:<b>` topic, the verb also calls `cmd_channel_describe` once with: "Direct messages between sender_id `<a>` and `<b>`. Same protocol as `agent-chat-arc`. Created by `termlink agent contact` on first use." — applied idempotently (skip if topic already had a description; safe to re-apply on existing topic without description)
- [ ] If the topic exists already, the verb posts to it without altering retention or description

**Identity stamping:**
- [ ] If T-1427 (whoami) has shipped: `metadata.from=<self_label>` is stamped from `whoami`, and the post relies on hub-side strict-reject for verification
- [ ] If T-1427 has NOT shipped at build time: omit `metadata.from`; the envelope's authoritative `sender_id` is sufficient. Document in the verb's --help that strict identity binding lands with T-1427.

**Envelope shape:**
- [ ] Posts with `msg_type=request`, `metadata.thread=<thread>` (default: short generated id like `c-<8hex>`), `metadata.requires_ack=<bool>`
- [ ] On `--file`, payload is the file contents (size cap applies — same as `channel post`); on `--message`, payload is the message string
- [ ] If both supplied, payload is the file contents and `metadata.subject=<message>` carries the one-line message

**Acknowledgment (T-1425 Q2=C):**
- [ ] Default behavior: post and exit with offset on stdout (or JSON if `--json`). No wait, no ack
- [ ] With `--ack-required`: after post, subscribe to the DM topic from offset+1 and wait up to `--ack-wait` seconds (default 30) for an `m.receipt` envelope from the target's `sender_id`. Exit 0 on receipt, exit 6 on timeout (new exit code, document in --help)
- [ ] `--ack-wait 0` with `--ack-required` is equivalent to default (no wait); document the redundancy

**Offline behavior (T-1425 Q3=C):**
- [ ] Default behavior: post regardless of target online state (the chat arc is offset-durable; queueing IS the natural behavior)
- [ ] With `--require-online`: pre-flight via `discover` and exit 7 if no live session matches `<target>`. Verb does NOT post in this branch — caller deals
- [ ] Discover failure on `--require-online` reports the specific reason (not found / hub unreachable / timeout) so callers can disambiguate

**Output:**
- [ ] Default text output: one line `Posted to <topic> — offset=<N>, ts=<ms>` matching existing `channel post` format
- [ ] `--json` output: `{"topic": "<dm:...>", "offset": N, "ts_ms": M, "ack": null|{"received": true, "offset": A, "ts_ms": B} }`
- [ ] On `--ack-required` timeout, JSON output sets `ack.received=false, ack.timeout_seconds=<N>`

**Tests:**
- [ ] Unit tests in `crates/termlink-cli/src/commands/agent.rs` for: target parsing (3 forms), topic name canonicalization (sorted id pair), default vs --ack-required vs --require-online flag combinations
- [ ] Integration test: post via verb, read via `channel subscribe`, confirm envelope shape (msg_type, metadata.thread, metadata.requires_ack)
- [ ] No regressions in existing `cmd_agent_ask` / `cmd_agent_listen` / `cmd_agent_negotiate` — they share the file but are independent

**Documentation:**
- [ ] Verb's `--help` text mentions: T-1425 RFC reference, "deprecates `remote push` for agent-to-agent contact", and the canonical envelope shape
- [ ] One line added to `docs/reference/cli.md` (or wherever current CLI ref lives) — discoverable but not duplicating the in-CLI help

### Human
- [ ] [REVIEW] Verify the verb's UX from a vendored-agent perspective
  **Steps:**
  1. Build: `cargo build --release -p termlink && cargo build --release --target x86_64-unknown-linux-musl -p termlink`
  2. Default fire-and-forget post:
     `termlink agent contact ring20-management-agent --message "smoke test from T-1429"`
     Expected: one-line `Posted to dm:... — offset=N, ts=M`, exit 0
  3. With ack required (likely times out — receiver not reading on this topic yet):
     `termlink agent contact ring20-management-agent --message "ack test" --ack-required --ack-wait 5`
     Expected: exit 6 after 5s timeout, error message identifies it as ack-timeout not transport failure
  4. With require-online against a known-down hub:
     `termlink agent contact <a-known-down-target> --message "x" --require-online`
     Expected: exit 7, error message names the discovery failure mode
  5. JSON mode parses cleanly:
     `termlink agent contact ring20-management-agent --message "x" --json | jq .`
  **Expected:** all five UX paths produce predictable, readable output. Exit codes are distinct (0/6/7) and documented.
  **If not:** capture the failing path in this task's Updates and re-scope.

## Verification

cargo build --release -p termlink 2>&1 | tail -5
cargo test --release -p termlink-cli --lib commands::agent 2>&1 | tail -10
target/release/termlink agent contact --help 2>&1 | grep -q "ack-required"
target/release/termlink agent contact --help 2>&1 | grep -q "require-online"
target/release/termlink agent contact --help 2>&1 | grep -q "T-1425\|RFC"

## Decisions

### 2026-05-01 — Identity-discovery prereq blocks Phase-1 build

- **Chose:** Defer T-1429 build pending identity-discovery wiring; ship Phase-1 only after the prereq lands
- **Why:** AC "When `<target>` resolves via `discover`, the verb finds the session and extracts its `sender_id`" requires `SessionMetadata` (crates/termlink-session/src/registration.rs:147) to expose the owner's identity fingerprint. It does not today (verified 2026-05-01: the struct has shell/term/cwd/termlink_version/data_socket — no identity_fingerprint). Without that field, name-resolution → dm-topic-canonicalisation cannot work; we'd be building on `display_name` collisions
- **Rejected (A):** Build with `display_name` as the dm-key — would create dm:display_name collisions (any two sessions with the same display name on different hosts collapse onto the same topic) and diverge from the `dm:<sorted_fp>:<sorted_fp>` convention that `cmd_channel_dm` (T-1319) already canonicalised
- **Rejected (B):** Build a Phase-1 MVP that takes `--peer-fingerprint <hex>` directly — duplicates `cmd_channel_dm` with an alias rename; no net new value over the existing verb; vendored agents would still need to know the peer fingerprint
- **Rejected (C):** Skip discovery entirely, post to a global topic — abandons the dm:* canon and forks the protocol away from T-1425 §3.2

**Phase split:**
- **Phase-0 (prereq, NOT YET TASKED):** Add `identity_fingerprint: Option<String>` to `SessionMetadata`; populate from `load_identity_or_create()` at registration time; surface in `session.discover` response. This is a small, structural change that benefits any future identity-aware verb (T-1427 strict-reject, T-1429 contact, T-1430 self-describe). Estimate: 50-80 lines + migration test for legacy registrations missing the field.
- **Phase-1 (this task post-prereq):** Basic `agent contact <name> --message "..."` with discover-based resolution, dm-topic auto-create with self-describe, default fire-and-forget post. ACs marked CLI surface + Discovery + Self-describe + minimal Output.
- **Phase-2 (separate follow-up):** `--file`, `--ack-required`, `--require-online`, advanced target forms (`name@hub:port`, `sender_id:<hex>`). Independently shippable after Phase-1.

**Action for next session:** capture the Phase-0 prereq as a separate task before reopening this one. T-1429's ACs are otherwise sound — they just need Phase-0 to land first.


## Updates

### 2026-04-30T21:26:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1429-termlink-agent-contact-name--high-level-.md
- **Context:** Initial task creation
