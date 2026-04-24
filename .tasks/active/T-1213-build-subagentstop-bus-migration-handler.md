---
id: T-1213
name: "Build SubagentStop bus-migration handler (T-1209 follow-up)"
description: >
  Implement per T-1209 GO: S1' spike first (test if non-zero SubagentStop exit mutates orchestrator-visible response); then bus-migration handler — over-threshold (T=8KB) returns auto-migrate to fw bus, orchestrator sees R-NNN pointer. Retires advisory check-dispatch.sh when live. Goal is no information loss. See docs/reports/T-1209-subagentstop-hook-inception.md.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [hook, dispatch, bus, framework-bridge]
components: []
related_tasks: [T-1209, T-175]
created: 2026-04-24T10:05:14Z
last_update: 2026-04-24T10:31:47Z
date_finished: null
---

# T-1213: Build SubagentStop bus-migration handler (T-1209 follow-up)

## Context

Implement the bus-migration handler per T-1209 GO. Inception goal: "no information loss"
for sub-agent results.

**Revised per S1' finding (2026-04-24):** SubagentStop **cannot** mutate the
orchestrator-visible response — the hook fires post-delivery and exit-code 2 only
forces re-execution. See `## Decisions` below. Net effect: the handler captures
and preserves the full transcript to `fw bus`, but cannot rewrite what the
orchestrator already ingested. The migration benefit is structural memory (bus
has an R-NNN entry any future turn can read) + operator visibility (stderr
nudge) + per-dispatch size telemetry for threshold tuning. This is still a
significant improvement over the current PostToolUse advisory (`check-dispatch.sh`):
information is durably captured before compaction, future tooling can read the
bus instead of re-scanning raw returns, and the size histogram informs whether
the 8KB threshold is right or should be tuned.

Design: `agents/context/subagent-stop.sh` reads `transcript_path`, extracts the
last assistant message from the JSONL, measures bytes, and if over threshold T
(initial T=8KB per human direction), posts to `fw bus` with subject
`subagent-return-<agent_type>`. Always logs `{ts, agent_type, bytes}` to
`.context/working/subagent-returns.jsonl` for the S2 size-survey data stream.
Retires `check-dispatch.sh` once live.

Parent research: `docs/reports/T-1209-subagentstop-hook-inception.md`.

## Acceptance Criteria

### Agent
- [x] S1' spike: determine SubagentStop mutation semantics — **COMPLETE**: hook
      cannot mutate orchestrator-visible response; only force re-run via exit 2
      or do logging side effects. Evidence: Claude Code hooks docs
      (https://code.claude.com/docs/en/hooks.md) per claude-code-guide sub-agent
      query 2026-04-24. See Decisions section.
- [ ] Telemetry: `agents/context/subagent-stop.sh` appends one JSON line per
      dispatch to `.context/working/subagent-returns.jsonl` with fields
      `{ts, agent_type, agent_id, bytes, migrated}`. File survives handover.
- [ ] Bus handler: over-threshold T=8KB returns trigger `fw bus post --task
      <focus> --agent subagent --summary <first-line> --blob <temp-file>`
      (auto-spill per `fw bus` size rules). Handler always exits 0 (non-blocking).
- [ ] `fw hook subagent-stop` dispatcher added in bin/fw (routes to
      agents/context/subagent-stop.sh).
- [ ] Hook installed in `.claude/settings.json` SubagentStop block calling
      `.agentic-framework/bin/fw hook subagent-stop`.
- [ ] Manual test: stub payload with 20KB `last assistant message` → handler
      writes telemetry line + fw bus R-NNN entry + returns 0. `migrated=true`.
- [ ] Manual test: stub payload with 500B return → handler writes telemetry line,
      no fw bus entry (under threshold). `migrated=false`.
- [ ] `check-dispatch.sh` marked retired (header comment "superseded by
      SubagentStop handler, T-1213") and removed from the PostToolUse matcher
      in `.claude/settings.json`.

### Human
- [ ] [REVIEW] After 2-3 real dispatches in next session, verify telemetry and
      migration work end-to-end.
      **Steps:**
      1. `tail -5 .context/working/subagent-returns.jsonl`
      2. Run: `.agentic-framework/bin/fw bus manifest` for the current focus task
      **Expected:** telemetry lines present for each dispatch; over-threshold
      returns have matching R-NNN entries in the bus manifest.
      **If not:** check `.context/working/subagent-stop.log` for handler stderr;
      verify `.claude/settings.json` SubagentStop entry is active and points at
      `.agentic-framework/bin/fw hook subagent-stop`.

## Verification

# Handler script exists and is executable
test -x .agentic-framework/agents/context/subagent-stop.sh
# Dispatcher route is live
.agentic-framework/bin/fw hook --help 2>&1 | grep -q subagent-stop
# settings.json has SubagentStop entry
grep -q '"SubagentStop"' .claude/settings.json
# Stub test exists and passes
test -x agents/context/tests/subagent-stop-stub-test.sh
agents/context/tests/subagent-stop-stub-test.sh

## Decisions

### 2026-04-24 — SubagentStop mutation semantics (S1' spike result)
- **Chose:** Handler is capture-and-log only, not response-mutating. Preserve the
  full transcript via `fw bus`, emit stderr nudge, do NOT attempt to intercept or
  rewrite the sub-agent message the orchestrator ingests in this turn.
- **Why:** Claude Code docs (hooks.md) state SubagentStop "cannot modify subagent
  behavior directly." The hook fires after the subagent's message has reached the
  orchestrator; exit 2 forces the subagent to continue (not what we want), and no
  documented API replaces the delivered text. Information-loss avoidance is still
  achieved because (a) transcript is on disk via `transcript_path`, (b) full
  content lands in `fw bus`, (c) any subsequent turn can read the bus manifest.
- **Rejected:** "Mode B mutation" as originally scoped in T-1209 S3 — rewriting
  orchestrator-visible text with an R-NNN pointer. Not supported by the hook API.
  The T-1209 inception was approved under the broader "no information loss" goal
  which this capture-only design still satisfies.

## Updates

### 2026-04-24T10:05:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1213-build-subagentstop-bus-migration-handler.md
- **Context:** Initial task creation

### 2026-04-24T10:31:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T10:40Z — S1' spike complete [agent]
- **Question:** Can SubagentStop mutate the orchestrator-visible response?
- **Answer:** **No.** Hook fires after delivery; only exit 2 (force subagent to
  continue) and log-style side effects are supported. No documented API to replace
  the text. Source: Claude Code hooks documentation.
- **Payload fields (documented):** `session_id`, `transcript_path`, `cwd`,
  `hook_event_name`, `agent_id`, `agent_type`. **No `last_assistant_message` field** —
  handler must read the last assistant entry from the transcript JSONL itself.
- **Design shift:** T-1209 S3 "Mode B mutation" is not buildable. Redesign to
  capture-and-log (still satisfies "no information loss"). ACs rewritten accordingly.
- **Next build steps:** S2+S3 merged into one handler: telemetry always, bus
  migration when over threshold. Stub test for both paths. Install into hook
  dispatcher + settings.json.
- **Artifact:** no separate spike artifact — finding captured here + in Decisions.
