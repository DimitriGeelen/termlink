---
id: T-1165
name: "T-1155/8 Build shell pickup → channel bridge adapter"
description: >
  Keep framework-side pickup portability (shell-based, 442-line lib/pickup.sh). Add one adapter that reads pickup YAML envelopes and posts them to a 'pickup:' channel. New projects can post direct; legacy pickup still works. Per T-1155 S-5 phase 3.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [T-1155, bus, pickup, framework-bridge]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:17Z
last_update: 2026-04-24T09:03:15Z
date_finished: null
---

# T-1165: T-1155/8 Build shell pickup → channel bridge adapter

## Context

Fourth migration in the T-1155 bus rollout: the shell-based pickup system (inbox/processed/rejected directories under `<framework>/.context/pickup/`) gets a one-way bridge to the bus. **Shell pickup stays portable** — it still works offline, without termlink, in any framework consumer. The bridge lets online bus subscribers also see pickups.

Depends on: T-1160 (channel API shipped). Referenced in PL-040 (pickup type closed vocabulary) — this task does not change that vocabulary, only mirrors envelopes to the bus.

**Prerequisite audit (2026-04-24):**

*Hook point identified:* `lib/pickup.sh::pickup_process_one @302` — immediately after `mv "$file" "$PICKUP_PROCESSED/"`, before the `return 0`. This is the one clean site where a processed envelope exists on disk at a known path.

*Channel CLI status:* `termlink channel {post,subscribe,list,create,queue-status}` verbs exist in source (`crates/termlink-cli/src/main.rs @374/386/392` + `commands::channel::cmd_channel_*`). Verified via `cargo run -- channel --help` in the termlink workspace. **BUT** the currently-installed `termlink` on this host is v0.9.206 (source is 0.9.385) — installed CLI does NOT recognize the `channel` subcommand.

*Implication:* the bridge script itself can be written now, but it cannot run end-to-end until a consumer project runs `cargo install --path crates/termlink-cli`. The "graceful degradation" AC (exit 0 if termlink missing or old) covers this — the bridge will silently no-op on pre-0.9.380ish installs.

*Recommended split:*
1. Write the bridge script (`lib/pickup-channel-bridge.sh`), install the hook line in pickup.sh, add the env-var opt-out, add header doc. Test with `bash -n` + a staged envelope + mocked `termlink` stub.
2. Cross-project installation step — ensure termlink ≥ 0.9.380 is deployed to framework consumers that want the bridge live.
3. Integration test (requires step 2 complete on at least one peer).

**Note on boundary:** Bridge script lives in `/opt/999-Agentic-Engineering-Framework/lib/`, not in termlink repo. T-559 boundary hook will block direct Edit/Write — must use `/tmp` staging + `cp` + cross-repo commit via `termlink dispatch --workdir` (same pattern as T-1206 fleet.py mirror).

## Acceptance Criteria

### Agent
- [ ] New framework bash script `lib/pickup-channel-bridge.sh` — invoked by the pickup processor after moving an envelope to `processed/`
- [ ] Bridge reads the processed envelope YAML → constructs `channel.post(topic="framework:pickup", msg_type="pickup-<type>", payload=<envelope-yaml>)` via `termlink channel post`
- [ ] Graceful degradation: if `termlink` binary missing OR hub unreachable, the bridge logs and exits 0 (non-fatal — shell pickup must stay portable per T-1155 §"Out of scope" / §"Migration strategy Phase 3")
- [ ] Idempotence: the bridge carries a dedup key derived from the envelope sha256 — re-running does not duplicate channel posts
- [ ] Opt-out: env var `FW_PICKUP_CHANNEL_BRIDGE=0` disables the bridge entirely for projects that don't want their pickups on the bus
- [ ] Documentation in `lib/pickup.sh` header explaining the bridge hook + env-var knob
- [ ] Integration test: drop an envelope in `inbox/` → wait for processor → verify (a) envelope is in `processed/` and (b) `termlink channel subscribe framework:pickup` observes it
- [ ] Bridge is one-way: channel posts do NOT create new inbox envelopes. Cross-project sharing remains via the existing `termlink file send` + pickup path (which itself becomes `channel.post msg_type=artifact` under T-1164 — naturally composed)
- [ ] No changes to existing pickup envelope schema, processor logic, or the closed vocabulary (bug-report | learning | feature-proposal | pattern) — this task is observation only
- [ ] Update `docs/reports/T-1155-agent-communication-bus.md` with the shell-bridge design decision

### Human
- [x] [REVIEW] Confirm one-way design — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — shell pickup → channel one-way design confirmed.
  **Steps:**
  1. Verify the bridge is post-only (channel → pickup is not implemented)
  2. Consider: do you want bidirectional (bus subscribers can *inject* pickups)? This would blur framework-pickup as a messaging channel with bus-as-channel. T-956 pickup lesson says keep them distinct.
  3. Decide whether to accept one-way or open a follow-up for bidirectional
  **Expected:** Approval of one-way + optional follow-up task
  **If not:** State the missing flow direction

## Verification

bash -n lib/pickup-channel-bridge.sh
test -x lib/pickup-channel-bridge.sh
grep -q "FW_PICKUP_CHANNEL_BRIDGE" lib/pickup-channel-bridge.sh
grep -q "pickup-channel-bridge" lib/pickup.sh

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

### 2026-04-20T14:12:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1165-t-11558-build-shell-pickup--channel-brid.md
- **Context:** Initial task creation

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next
