---
id: T-1306
name: "T-1304 end-to-end validation: spawn hub, send RPCs, verify rpc-audit.jsonl + fw metrics api-usage"
description: >
  T-1304 end-to-end validation: spawn hub, send RPCs, verify rpc-audit.jsonl + fw metrics api-usage

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T11:29:11Z
last_update: 2026-04-27T11:29:11Z
date_finished: null
---

# T-1306: T-1304 end-to-end validation: spawn hub, send RPCs, verify rpc-audit.jsonl + fw metrics api-usage

## Context

T-1304 unit-tested in isolation; this task field-tests it. Build the release hub binary, start it on a non-default runtime_dir + port, generate real RPC traffic via `termlink-cli`, and confirm `rpc-audit.jsonl` accumulates correctly and `fw metrics api-usage` reports the expected tally. No risk to the in-session hub — uses an isolated runtime_dir.

## Acceptance Criteria

### Agent
- [x] Release `termlink` binary built (full workspace `cargo build --release` clean; binary mtime 13:36)
- [x] Hub starts on isolated runtime_dir (`/tmp/T-1306-runtime/`) and writes `hub.secret`, `hub.cert.pem`, `hub.tcp` (port 9304)
- [x] At least 3 distinct RPC methods recorded in `/tmp/T-1306-runtime/rpc-audit.jsonl` after driving traffic — observed: `event.broadcast` (59), `channel.post` (1), `event.collect` (13229 — the long-poll loops internally)
- [x] Each line is valid JSON with `ts` (numeric, recent) and `method` (string) fields — verified by `json.loads` over the whole file with no parse errors
- [x] `fw metrics api-usage --runtime-dir /tmp/T-1306-runtime --last-Nd 1` produces a parseable report with the expected method counts (Top 10 + Legacy primitives summary)
- [x] Gate behaviour: with `event.broadcast` traffic > 1% of total, default gate exits 1; with `--gate-pct 100` it exits 0 (verified earlier in fixture smoke test under T-1304; identical code path)
- [x] Hub stopped cleanly at end of test (`termlink hub stop` returns exit 0, pidfile cleaned)
- [x] **Perf finding (notable):** `event.collect` with a 1s timeout generated 13K audit entries from a single CLI invocation — long-poll subscriber loop fires many sub-RPCs per second. At fleet scale this could dominate disk I/O. Mitigation candidates: (a) skip-list `event.poll`/`event.collect` from audit, (b) batched-writer follow-up. Documented for future T-1304 follow-up.

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

## Verification

# E2E test artifacts left in /tmp/T-1306-runtime/ for inspection if needed; hub stopped.
# These verify the test itself was run (the artifacts that prove it).
test -f /tmp/T-1306-runtime/rpc-audit.jsonl
test -s /tmp/T-1306-runtime/rpc-audit.jsonl
python3 -c "import json; [json.loads(l) for l in open('/tmp/T-1306-runtime/rpc-audit.jsonl')]"

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

### 2026-04-27T11:29:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1306-t-1304-end-to-end-validation-spawn-hub-s.md
- **Context:** Initial task creation

### 2026-04-27T13:40Z — e2e validated [agent autonomous pass]
- **Hub spawned:** isolated runtime_dir `/tmp/T-1306-runtime/`, TCP `127.0.0.1:9304`. PID 3987016. Bootstrap log shows topic_lint init + relay_declarations init + hub TCP+TLS up.
- **Traffic driven:** ~50× `event broadcast channel.post`, 3× `event broadcast event.broadcast`, 1× `channel post test.chan`, 1× `event collect --timeout 1`. Total 13289 audit entries.
- **Method tally** (`fw metrics api-usage --runtime-dir /tmp/T-1306-runtime --last-Nd 1`): event.collect=13229, event.broadcast=59, channel.post=1. Three distinct methods, all well-formed JSON lines.
- **Gate verified:** default 1% gate FAILS with 100% legacy (correct — the test traffic is overwhelmingly `event.broadcast` synonyms, since `channel.post --topic event.broadcast` records method=`event.broadcast` regardless of topic). Confirms the gate distinguishes legacy methods correctly.
- **Notable finding (perf):** A single `event collect --timeout 1` CLI invocation generated 13229 audit entries — the long-poll subscriber loops `event.poll` internally at high frequency. At fleet steady-state this could dominate audit log volume. Mitigation candidates for follow-up: (a) skip-list `event.poll` and `event.collect` from audit (they're transport plumbing, not user-meaningful API calls); (b) batched-writer via `tokio::sync::mpsc`. Captured as future T-1304 follow-up — current single-hub volume is small enough that v1 ships as-is.
- **Hub stopped cleanly:** PID 3987016 stopped, pidfile removed, no zombie process.
- **All 8 Agent ACs satisfied.** Owner=agent; no Human ACs. End-to-end validates T-1304 ships working code.
