---
id: T-1165
name: "T-1155/8 Build shell pickup → channel bridge adapter"
description: >
  Keep framework-side pickup portability (shell-based, 442-line lib/pickup.sh). Add one adapter that reads pickup YAML envelopes and posts them to a 'pickup:' channel. New projects can post direct; legacy pickup still works. Per T-1155 S-5 phase 3.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, pickup, framework-bridge]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:17Z
last_update: 2026-04-24T12:16:49Z
date_finished: 2026-04-24T12:16:49Z
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

**Approach update (2026-04-24, per T-1214 fleet-diagnosis):** Build against the
JSON-RPC layer (`channel.post` method), NOT the `termlink channel` CLI. Rationale:
T-1214 S1 probe confirmed even the local install lags source by 192 commits and
.122 is stranger-lineage (0.9.844, no `channel` subcmd). The bridge must be
federation-tolerant: capability-probe the hub, use `channel.post` RPC when
available, fall back to `event.broadcast` + `inbox.post` (universally supported)
otherwise. See `docs/reports/T-1214-fleet-diagnosis.md`.

## Acceptance Criteria

### Agent
- [x] New framework bash script `lib/pickup-channel-bridge.sh` — invoked by the pickup processor after moving an envelope to `processed/`. Upstream commit `1231dba2`.
- [x] Bridge reads the processed envelope → capability-probes `termlink channel post` (Tier-A, T-1160) and uses it when available; falls back to `termlink event broadcast` topic=`framework:pickup` when `channel` subcmd is absent (T-1214 federation-tolerance). Payload carries `msg_type`, `sha`, `basename`, and full envelope when `jq` is present; ref-only otherwise.
- [x] Graceful degradation: bridge exits 0 on missing termlink, missing `sha256sum`/`shasum`, failed channel.post, or failed event.broadcast. Each path logs to `.context/working/.pickup-bridge.log` for audit. Smoke-tested locally against termlink 0.9.206 — event.broadcast fallback path posted to local hub successfully.
- [x] Idempotence: SHA-256 of envelope contents is the dedup key; stored at `.context/pickup/.bridge-posted/<sha>`. Smoke-verified — second invocation on same envelope emits `dedup` log and skips post.
- [x] Opt-out: `FW_PICKUP_CHANNEL_BRIDGE=0` short-circuits at the top of the script. Smoke-verified — log file receives no new entries when opt-out is set.
- [x] Hook in `lib/pickup.sh::pickup_process_one` at line ~302 (post-mv, pre-return) with a 2-line comment citing T-1165 and the portability contract. Bridge script's own header documents the env-var knob, topic, and fallback chain.
- [x] Integration-test surrogate: invoked bridge directly against a minimal envelope against the local hub — observed `posted via=event.broadcast topic=framework:pickup msg_type=pickup-learning sha=<…>`. Full inbox→processor→subscribe loop deferred until a consumer project has termlink ≥ 0.9.380 (per Context 'Recommended split' step 3).
- [x] Bridge is one-way: the script only writes OUT (post/broadcast); no inbound code path reads channels or creates inbox envelopes. Reaffirmed by design.
- [x] No changes to pickup envelope schema, processor logic, or closed vocabulary. Only additions: bridge script + hook line + header comment.
- [x] Updated `docs/reports/T-1155-agent-communication-bus.md` Phase-3 entry: records the 2026-04-24 ship, capability-probe + fallback design, and the one-way decision referencing T-956.

### Human
- [x] [REVIEW] Confirm one-way design — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — shell pickup → channel one-way design confirmed.
  **Steps:**
  1. Verify the bridge is post-only (channel → pickup is not implemented)
  2. Consider: do you want bidirectional (bus subscribers can *inject* pickups)? This would blur framework-pickup as a messaging channel with bus-as-channel. T-956 pickup lesson says keep them distinct.
  3. Decide whether to accept one-way or open a follow-up for bidirectional
  **Expected:** Approval of one-way + optional follow-up task
  **If not:** State the missing flow direction

## Verification

# Bridge landed upstream: syntactically valid, executable
bash -n /opt/999-Agentic-Engineering-Framework/lib/pickup-channel-bridge.sh
test -x /opt/999-Agentic-Engineering-Framework/lib/pickup-channel-bridge.sh
# Opt-out env var + federation-tolerance markers
grep -q "FW_PICKUP_CHANNEL_BRIDGE" /opt/999-Agentic-Engineering-Framework/lib/pickup-channel-bridge.sh
grep -q "event broadcast" /opt/999-Agentic-Engineering-Framework/lib/pickup-channel-bridge.sh
# Hook landed in pickup.sh
grep -q "T-1165: mirror envelope" /opt/999-Agentic-Engineering-Framework/lib/pickup.sh
# T-1155 design doc records Phase 3 ship
grep -q "T-1165.*Shell pickup" /opt/termlink/docs/reports/T-1155-agent-communication-bus.md

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

### 2026-04-24T12:09:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T12:16:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
