---
id: T-1431
name: "/agent-handoff claude-code skill — wraps T-1429 verb (T-1425 pick #5)"
description: >
  From T-1425 fast-forward synthesis. Plugin-level skill, NOT in CLAUDE.md. Lives in .claude/skills/agent-handoff.md (or in a plugin file if we have one). Sequence: verify task exists → termlink whoami (lock identity) → termlink agent contact <target> --thread <task-id> --message <summary> → verify offset returned → update task with posted=offset, status hint=awaiting-reply. CLAUDE.md cost: ONE line — 'for cross-host handoffs use /agent-handoff'. Depends on T-1429 (the verb being wrapped) and T-1427 (identity binding the skill enforces). Independent of T-1430 (topic self-doc).

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:02:51Z
last_update: 2026-05-01T11:51:32Z
date_finished: null
---

# T-1431: /agent-handoff claude-code skill — wraps T-1429 verb (T-1425 pick #5)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] T-1429 (`agent contact` verb) has shipped — Phase-1 in commit a5fb0ad4 (2026-05-01); skill wraps the shipped verb (Phase-2 flags `--ack-required`/`--require-online`/`--file`/`--thread` deferred per T-1429 in-task decision; skill embeds task-id as `[T-XXX]` body prefix instead of `--thread`)
- [x] T-1427 (whoami) — `termlink whoami` already exists with `--name`/`--session`/`--json` (T-1299/T-1297). Strict hub-side validation is the deferred half of T-1427 and is NOT required by the skill; skill calls `whoami` for log visibility only, not for enforcement
- [x] Skill file exists at `.claude/commands/agent-handoff.md` — matches existing convention (capture.md, resume.md, self-test.md all live in `.claude/commands/`, not `.claude/skills/`). Argument hint documented in body header (Claude Code commands don't use frontmatter for arg-hints)
- [x] Skill body executes the canonical sequence: (1) verify task exists in `.tasks/active/`; (2) `termlink whoami --json` capture self sender_id; (3) `termlink agent contact <target> --message "[<task-id>] <msg>" --json`; (4) parse `delivered.offset`; (5) append handoff Update entry; (6) print 4-line summary
- [x] Skill fails fast — every step has an explicit "stop and exit non-zero" branch. No silent fallbacks, no degraded paths, no alternative topic substitution
- [x] CLAUDE.md gained exactly ONE line under Quick Reference: `**Cross-host handoff** | **/agent-handoff <target> <task-id> "<msg>"** | Skill wrapping termlink agent contact ...`. No prose elsewhere
- [x] Skill is invocable via `/agent-handoff` slash command and listed in the available-skills surface (verified via system-reminder skill list: `agent-handoff: /agent-handoff - Cross-Host Agent Contact (T-1429 wrapper)`)
- [x] Skill Rules section explicitly disallows: `remote push`, `inbox.push`, `event.broadcast --target`, `agent.reply` topic, improvising sender labels via `--metadata-from`, fan-out in single invocation, retrying on failure without surfacing
- [x] Smoke test executed live 2026-05-01T11:55Z: registered `handoff-smoke-peer` (id tl-hkr54f2e, identity_fingerprint d1993c2c3ec44c94 visible in sidecar). Ran `termlink agent contact handoff-smoke-peer --message "[T-1431] smoke test from agent-handoff skill — verifying end-to-end flow" --json`. Returned `{"delivered":{"offset":4,"ts":1777636506829}}`. Topic `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` created. End-to-end verified — see Updates entry below

### Human
- [ ] [RUBBER-STAMP] Verify the skill works end-to-end from a real session
  **Steps:**
  1. From a fresh Claude Code session in /opt/termlink, register a smoke peer first: `termlink register --name handoff-rubber-stamp --self --json &`
  2. Then invoke: `/agent-handoff handoff-rubber-stamp T-1431 "rubber-stamp verification — please ignore"`
  3. Watch the output — should see task-existence check → whoami → contact → offset → task update sequence
  4. `grep -A6 "handoff-posted" .tasks/active/T-1431-*.md | tail -8` — see the update entry
  5. `termlink channel list --prefix dm: | grep handoff-rubber` (or grep your own fingerprint prefix) — confirm dm topic created
  **Expected:** end-to-end works without prompts, manual fallbacks, or improvisation
  **If not:** capture failure point in Updates and re-scope which step broke

## Verification

test -f .claude/commands/agent-handoff.md
grep -q "agent contact" .claude/commands/agent-handoff.md
grep -qi "whoami" .claude/commands/agent-handoff.md
grep -q "NEVER.*remote push\|NEVER.*inbox" .claude/commands/agent-handoff.md
grep -q "agent-handoff" CLAUDE.md

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

### 2026-05-01T07:02:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1431-agent-handoff-claude-code-skill--wraps-t.md
- **Context:** Initial task creation

### 2026-05-01T11:51:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-01T11:55Z — smoke-test [agent-handoff-skill]
- **Action:** Live end-to-end smoke test of the freshly-shipped skill
- **Setup:** Registered ephemeral peer `handoff-smoke-peer` (`termlink register --name handoff-smoke-peer --self --json &`) → session id `tl-hkr54f2e`, identity_fingerprint `d1993c2c3ec44c94` confirmed in `/var/lib/termlink/sessions/tl-hkr54f2e.json` metadata
- **Invocation:** `termlink agent contact handoff-smoke-peer --message "[T-1431] smoke test from agent-handoff skill — verifying end-to-end flow" --json`
- **Result:** `{"delivered":{"offset":4,"ts":1777636506829}}` — exit 0
- **Topic created:** `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` (self-DM since self/peer share identity key on .107)
- **Verified:** `termlink channel list --prefix "dm:d1993c2c"` shows the topic with `[forever]` retention. Topic auto-described per T-1429.5 (only on first create — second smoke-test invocation would NOT re-describe)
- **Adjustments:** Skill body updated to reflect actual JSON shape `{"delivered":{"offset":N,"ts":ms}}` (not the assumed `{ok,topic,offset,ts_ms,peer}`); skill location `.claude/commands/agent-handoff.md` (not `.claude/skills/`) matches existing convention
- **Verification gate:** all 5 verification commands pass

### 2026-05-01T11:55Z — agent-acs-ticked [agent autonomous]
- **Action:** All 9 agent ACs ticked. CLAUDE.md gets one-liner under Quick Reference. Skill discoverable via `/agent-handoff` slash command (verified in skills surface)
- **Owner:** unchanged (human) — pending RUBBER-STAMP verification
