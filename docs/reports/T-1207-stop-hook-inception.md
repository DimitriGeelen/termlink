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

## Dialogue Log

### 2026-04-24 — inception kickoff
- **Agent:** Drafted exploration plan before any spikes, per inception discipline. Framework's own settings doc said defer on all three hooks (#Rec 3, #Rec 8); user is overriding that position. Pending user confirmation on the proposed spike order and the warn/block escalation shape before running S1-S3.
- **Human:** (awaiting input)
