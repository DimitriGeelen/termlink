# T-1207: Stop hook design — inception

**Parent task:** T-173 (captured with placeholder ACs).
**Status:** Exploration plan drafted; awaiting user review before spikes.

## Problem Statement

Claude Code's `Stop` hook fires after **every assistant response**, with `last_assistant_message` and transcript path in the JSON payload. The framework wants to close **G-005** — "pure-conversation sessions bypass enforcement". Today the task gate only blocks Write/Edit/Bash; a user can converse for hours (reading code, asking questions, planning) without ever creating a task, committing, or ending the session cleanly. Those sessions produce no artifacts, no handover, no episodic.

## Framework-existing position to override

`docs/claude-code-settings.md §Rec #3` currently says SessionEnd doesn't exist and existing mitigations (budget gate, commit cadence, PreCompact) are sufficient. User has now directed inception-and-build, overriding that deferral. Rec #8 (SubagentStop) is marked "Explore in future" — also overridden.

## Constraints

- **Stop fires after every assistant response.** It is NOT a session-end event. Wrong mental model = wrong design.
- **Can block** (non-zero exit). Too aggressive = agent becomes unusable.
- **Runs in every session.** Latency matters (budget gate is already there).
- **Consumer-project portable.** Script must live at framework-level, consumed via the `fw hook` wrapper.
- **Must not break legitimate Q&A.** The user asking "explain this code" is not a violation.

## Assumptions (to register)

- A1: Most "pure conversation" sessions that miss enforcement are shorter than 20 exchanges — so check-after-N is the right shape.
- A2: Users can be warned without being blocked — a soft nag at N=10 and a harder warn at N=20 is tolerable.
- A3: The `.context/working/` state already carries enough signal to judge "productive vs idle" (tool counter, last commit hash, budget level).

## Exploration Plan

**Spike S1 — payload survey (1h):** Log Stop hook payloads for 3 real sessions. Capture message length, transcript length, what signals are present. Confirm or refute A3.

**Spike S2 — heuristic prototype (2h):** Implement a Stop-guard that reads payload, increments a counter at `.context/working/.stop-counter`, checks against `.tool-counter` and `.last-commit-hash`. Emits stderr warning at N=10 with 0 tools used and no new commits. Non-blocking. Does it fire on real sessions?

**Spike S3 — escalation design (1h):** Draft the full ladder (nag @ N=10 → warn @ N=20 → block @ N=30 with bypass flag). Mock three scenarios: (a) exploring a new codebase, (b) debugging without edits, (c) pure planning conversation. Does the ladder catch each appropriately?

## Decision Framework (GO / NO-GO criteria)

**GO if:** S2 fires on at least 2 of 3 real sessions with zero false positives AND the escalation ladder from S3 makes sense for all 3 scenarios.
**NO-GO if:** Stop hook latency adds >100ms per response, OR the heuristic cannot distinguish productive Q&A from idle-chat without significant per-user config.

## Implementation destination

Framework-side. New stop-guard script under `agents/context/`, wired through the `fw hook` dispatcher. Consumer project settings.json wires the Stop matcher to the framework wrapper.

## Revised design (per human direction 2026-04-24)

**No block. No silent nag-then-warn-then-block ladder. Pattern = agent-asks-human, with conversation capture as the ultimate goal.**

Mechanism (works within Claude Code semantics):
1. Stop hook exits 0 (never blocks) but emits stderr text when threshold conditions are met. Stderr from a Stop hook becomes additional context the agent sees on the next turn.
2. Stderr message says: "Detected N exchanges with 0 tools / 0 commits / no active task. Recommend prompting user to capture this conversation."
3. Agent on next turn proactively asks user: **"We've been talking for a while without capturing anything. Should I create a task summarizing this conversation so far? (y/n)"**
4. On `y`: agent runs `fw work-on "summary" --type spec` (or similar), inserts a brief context dump, continues. Conversation is now captured.
5. On `n`: agent writes a "dismissed" marker (`.context/working/.stop-dismissed-at-N`), counter resumes from there. Same nudge fires again at +N exchanges.

**Ultimate goal:** no conversation produces zero artifacts. Either it's worth capturing (task created) or the user explicitly says "this one isn't" (dismissed marker logged for episodic).

**Threshold (revised, single-step):** N=10 exchanges with 0 tool uses AND 0 commits AND no `.context/working/focus.yaml` set. One nudge, then dismissable. Re-fires at N+10.

**Spike S1-S3 implications:**
- S1 (payload survey) still needed — confirms `.tool-counter` and `.last-commit-hash` give clean signal.
- S2 redesigned: implement passive logger (no nudge yet) that records when threshold WOULD have fired across real sessions for 1 week. Refine threshold from data before issuing real nudges.
- S3 dropped (no escalation ladder). Replaced with **S3' — agent prompt template:** the exact phrasing the agent uses when nudged, so the question is consistent across sessions.

## Dialogue Log

### 2026-04-24 — inception kickoff + direction received
- **Agent:** Drafted exploration plan with nag/warn/block ladder.
- **Human:** No block. Use a y/n user-question pattern. Agent should pick up the nudge autonomously and ensure conversation is captured. That's the ultimate goal.
- **Agent:** Redesigned per direction — Stop hook becomes a passive observer that emits an agent-visible nudge; agent owns the user-prompt; capture-or-dismiss is the only outcome. Updated spike plan: S1 unchanged, S2 = passive 1-week threshold-tuning, S3 → S3' = agent prompt template. Pending: confirm 1-week passive observation period is right before issuing real nudges, or do you want nudges live from day 1 with looser threshold?
