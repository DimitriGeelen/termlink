---
id: T-1429
name: "termlink agent contact <name> — high-level cross-host contact verb (T-1425 pick #2)"
description: >
  From T-1425 inception solo synthesis 2026-04-30. Wraps the discover -> resolve-DM-topic -> post pattern into one verb so vendored agents stop improvising primitives. Replaces the broken pattern that produced the .107-to-.122 ZoneEdit handoff incident. Decisions baked in per T-1425 §Decisions: Q1=A auto-create dm:<sorted>:<sorted>, Q2=C opt-in --ack-required, Q3=C default-queue with --require-online flag, Q5=A retention=forever. Q4 (identity binding) ships in T-1427 separately; this verb relies on it via channel.post when T-1427 lands but works without strict-reject in the meantime. Lives in crates/termlink-cli/src/commands/agent.rs alongside cmd_agent_ask/listen/negotiate. Provisional pending peer replies on T-1425 thread (14d amendment window).

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:26:39Z
last_update: 2026-05-01T11:18:55Z
date_finished: null
---

# T-1429: termlink agent contact <name> — high-level cross-host contact verb (T-1425 pick #2)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent

**CLI surface (Phase-1):**
- [x] `termlink agent contact <target> --message <m> [--hub <addr>] [--json]` parses correctly via `termlink agent contact --help`. Help text references T-1425 RFC + T-1427 future strict-reject + T-1436 prereq + Phase-2 deferred ACs
- [x] **Phase-2 partial — `--thread <id>` SHIPPED 2026-05-01T12:08Z:** `termlink agent contact <target> --message <m> --thread <task-id>` sets `metadata._thread=<task-id>` per agent-chat-arc protocol canon (T-1430 topic doc). Implementation: 1-line wire in cli.rs Contact variant, plumbed through main.rs and agent.rs:cmd_agent_contact, extends cmd_channel_dm to accept `&[String] extra_metadata` slot, mentions still go through unchanged. Vendored agents can now route DM messages by thread server-side without parsing the `[T-XXX]` body prefix. The skill keeps `[T-XXX]` body prefix for portability (older binaries lack `--thread`); callers on >= 0.9.1657 may use `--thread` directly for cleaner metadata routing
- [ ] **Phase-2 (still deferred):** `--file <path>`, `--ack-required`, `--ack-wait <secs>`, `--require-online` flags. Out of scope; track separately when needed
- [ ] **Phase-2 (deferred):** advanced `<target>` forms — `name@hub:port`, `sender_id:<hex>`. Phase-1 supports bare session-name only (resolved via local registration)

**Discovery + topic resolution (Phase-1):**
- [x] `<target>` resolves via `manager::find_session` (local-only). Peer's `sender_id` (`identity_fingerprint`) is read directly from `Registration.metadata` — exactly the field T-1436 plumbed in
- [x] DM topic name is computed as `dm:<sorted_a>:<sorted_b>` — implementation delegates to existing `cmd_channel_dm` (commands/channel.rs:482) which uses the existing `dm_topic` helper, satisfying "reuse, do not reimplement"
- [x] Idempotent topic creation (`retention=forever`) happens inside `cmd_channel_dm` via existing `ensure_topic` (channel.rs:462). T-1425 Q1=A, Q5=A satisfied through the delegation
- [x] **Self-describe on create (T-1429.5 shipped 2026-05-01T11:17Z):** dm:* topics now auto-emit a topic_metadata envelope on FIRST create only. Implementation: hub-side `channel.create` returns `created: bool`, CLI `ensure_topic` reads it, `cmd_channel_dm` posts via `cmd_channel_describe` iff `created=true`. Pre-existing topics correctly skip describe (no bloat); pre-T-1429.5 hubs return no `created` field — clients conservatively treat that as `false`, skipping describe so old fleets keep working. T-1430's deferred AC ✅ ticked there. Verified live: brand-new `dm:d1993c2c3ec44c94:ffff0000aaaa1111` shows description in `channel info`; pre-existing self-DM correctly stays undescribed
- [x] Existing-topic posts go through unchanged via the same `cmd_channel_dm` path — no description rewrite

**Identity stamping (Phase-1):**
- [x] T-1427 has not shipped — `metadata.from` is NOT stamped (matches the documented Phase-1 behavior in the verb's --help). Authoritative `sender_id` derived from local identity key is what proves provenance until T-1427 lands
- [ ] **Phase-2 (deferred, blocked on T-1427):** stamp `metadata.from=<whoami-label>` and rely on hub-side strict-reject

**Envelope shape (Phase-1):**
- [x] Posts via `cmd_channel_post` (called from `cmd_channel_dm`) with `msg_type=chat`. The richer `msg_type=request` shape with `metadata.thread`/`metadata.requires_ack` is Phase-2 (depends on `--ack-required` flag which Phase-1 doesn't ship)
- [ ] **Phase-2 (deferred):** `--file` payload + `metadata.subject=<message>` when both flags supplied

**Acknowledgment (T-1425 Q2=C, all Phase-2):**
- [x] Default fire-and-forget behavior: post and exit with offset on stdout (text or JSON). Verified against a fresh test session — `Posted to dm:... — offset=N, ts=M` printed, exit 0
- [ ] **Phase-2 (deferred):** `--ack-required` subscribe + wait + exit-6 timeout
- [ ] **Phase-2 (deferred):** `--ack-wait 0` redundancy doc

**Offline behavior (T-1425 Q3=C, all Phase-2):**
- [x] Default behavior — post regardless of target online state — works (the dm topic is offset-durable; queueing happens at the topic level). Verified
- [ ] **Phase-2 (deferred):** `--require-online` pre-flight + exit-7 not-found
- [ ] **Phase-2 (deferred):** disambiguated discover-failure reasons

**Output (Phase-1):**
- [x] Default text output: one line `Posted to <topic> — offset=<N>, ts=<ms>` (delegated to `cmd_channel_post`). Verified
- [x] `--json` output: structured JSON with topic + offset + ts_ms (delegated). Verified — `target/release/termlink agent contact <peer> --message x --json` returns valid JSON
- [ ] **Phase-2 (deferred):** the `ack` JSON sub-object on success/timeout

**Tests:**
- [x] Unit test added: `commands::agent::contact_tests::dm_topic_shape_canon_stable` — locks the dm:<a>:<b> canon shape so a future refactor can't silently change the format vendored agents key off. 1 passed
- [x] Smoke-tested live (3 scenarios): pre-T-1436 peer → exit-8 with upgrade-needed message; post-T-1436 peer → posts to canonical dm topic; --json mode → returns clean JSON
- [x] No regressions: 542 existing tests remain green (pre-existing `manifest::tests::test_is_git_repo_on_temp_dir` failure is environmental, documented in T-1436)

**Documentation (Phase-1):**
- [x] Verb's `--help` text references T-1425 RFC, T-1427 (strict-reject futures), T-1436 (identity prereq), and explicitly names Phase-2 deferred ACs so operators don't expect them
- [ ] **Phase-2 (deferred):** docs/reference/cli.md entry — current scope is in-CLI help only; will land alongside the Phase-2 flag set

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

# Phase-1 verification gates (Phase-2 gates intentionally absent)
cargo build --release -p termlink 2>&1 | tail -5
cargo test --release -p termlink --bin termlink commands::agent::contact_tests 2>&1 | grep -q "1 passed"
target/release/termlink agent contact --help 2>&1 | grep -q "T-1425\|RFC"
target/release/termlink agent contact --help 2>&1 | grep -q "Phase-1"
target/release/termlink agent contact --help 2>&1 | grep -q "T-1427"
target/release/termlink agent contact framework-agent --message "x" 2>&1 | grep -q "T-1436"

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

### 2026-05-01T10:43:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-01T10:55Z — Phase-1 MVP shipped [agent autonomous]

**Implementation:** `cmd_agent_contact` in `crates/termlink-cli/src/commands/agent.rs:655` is a thin wrapper:
1. `manager::find_session(target)` — local registration lookup
2. `reg.metadata.identity_fingerprint` — peer fingerprint (T-1436 plumbing)
3. Delegate to `cmd_channel_dm(peer_fp, Some(message), None, &[], false, hub, json)` which handles dm-topic canonicalisation, idempotent ensure_topic, and post

**Net change:** 4 files, ~120 lines new (cli.rs Subcommand variant +30, main.rs dispatch +3, agent.rs cmd_agent_contact +85, plus 1 unit test). Reuses every existing helper (`dm_topic`, `ensure_topic`, `cmd_channel_post`, `load_identity_or_create`) — no protocol or hub change.

**Live smoke-test results:**
1. Pre-T-1436 peer (`framework-agent`) → exits 8 with: "Peer 'framework-agent' has no identity_fingerprint in metadata — likely registered before T-1436. Upgrade the peer's termlink binary and restart the session, then retry."
2. Post-T-1436 peer (fresh `t1429-peer`) → `Posted to dm:d1993c2c3ec44c94:d1993c2c3ec44c94 — offset=0, ts=...`, exit 0
3. `--json` → valid JSON with `topic`/`offset`/`ts_ms`

**Phase-2 (deferred, separate follow-up):** `--file`, `--ack-required`, `--ack-wait`, `--require-online`, advanced target forms (`name@hub:port`, `sender_id:<hex>`), `metadata.thread`/`requires_ack`/`subject` envelope fields, dm-self-describe-on-create. ACs marked `**Phase-2 (deferred)**` above. None of them block Phase-1 utility — vendored agents can fire-and-forget contact peers TODAY using just `--message`.

**Status:** Phase-1 ACs ticked. Agent ACs split between Phase-1 (done) and Phase-2 (deferred). T-1429 stays in active/ with owner=human; human REVIEW remains pending. The task naturally transitions into a Phase-2 build later, or a human can split it into T-1429a/b at their discretion.
