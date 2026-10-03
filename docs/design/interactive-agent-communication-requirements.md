# Interactive agent-to-agent communication: requirements and design

**Owner:** the operator · **Task:** T-3335 · **Started:** 2026-10-04
**Status:** skeleton confirmed by the operator (read-back 1–15, 2026-10-04); sections are being expanded with the design conversations and decisions.
**Sources:** `docs/reports/T-3335-design-history-termlink.md`, `docs/reports/T-3335-design-history-aef.md`, and the rounds they cite (March–October 2026).

How each section is laid out:
1. **Requirements:** numbered R-x.y, from the operator's confirmed read-back. A requirement changes only by an operator decision, recorded in the Decision log.
2. **Design:** how the requirement is met, with the component or file that does it.
3. **Conversations and decisions:** the rounds in which this was discussed (date, task, source file:line), the operator's own words, the rulings, and contradictions between rounds.

---

## 1. Goal

### Requirements
- R-1.1 Agents hold interactive, two-way conversations with each other while they are working: 1:1, many-to-many, and agents with the operator.
- R-1.2 The sender always knows where its message is.
- R-1.3 No agent has to be attached or polling by hand for delivery to proceed.

### Design
(to expand)

### Conversations and decisions
(to expand: "interactive multi-turn conversation between two or more agents" (April); "push messages + automatic pickup for truly dynamic, interactive conversation" (May); "bi-directional interactive communication … a really important part of this design" (2026-10-03).)

## 2. Origins

### Requirements
- R-2.1 Keep the original capability: inject keystrokes into a terminal and read its output back (T-007).
- R-2.2 Replace fire-and-forget dispatch with conversation over the hub (T-256).

### Design
(to expand: doorbell + mail, T-1800.)

### Conversations and decisions
(to expand: T-007 2026-03-08, T-256 2026-03-23, T-243 2026-04-26, T-1800 2026-05-25.)

## 3. The sidecar

### Requirements
- R-3.1 One sidecar per agent: a separate, very simple process with an API.
- R-3.2 Independent of the hub, "because the hub goes down".
- R-3.3 Always respawns, portably, not only under systemd (SQ-8).
- R-3.4 Carries the startup chain: start agent, session, project, hub.
- R-3.5 Re-resolves when an FQDN or IP stops resolving.

### Design
(to expand)

### Conversations and decisions
(to expand: arc-011 spec 2026-09-22; SQ-1 local-control reading; SQ-8; AEF T-3396/T-3397.)

## 4. Sending

### Requirements
- R-4.1 The sender gives the message, optionally with a blob, to its own sidecar's API.
- R-4.2 The sender's sidecar delivers it to the receiver's sidecar API. Push first.
- R-4.3 The hub is the fallback, plus discovery and blob storage.

### Design
(to expand)

### Conversations and decisions
(to expand: SQ-1 "sending stays on the hub" vs the operator's sidecar-to-sidecar send; charter non-goal (no second bus); AEF T-3688 cross-host.)

## 5. Receiving

### Requirements
- R-5.1 RECEIVED: the receiver's sidecar calls the sender's sidecar immediately.
- R-5.2 STORED: the message is stored durably and a second call goes back.
- R-5.3 A new-message flag is raised.

### Design
(to expand)

### Conversations and decisions
(to expand: L1/L2/L3 ladder (June); RECEIVED/INJECTED two events (September); STORED new (October); RECEIVED vs STORED one call or two (open).)

## 6. The 30-second tick

### Requirements
- R-6.1 A cron-style job checks the new-message flag every 30 seconds.
- R-6.2 With the flag up, it reads the queue, highest priority first.
- R-6.3 Urgent: injected immediately, even when the agent is busy (hard bypass), by a route that cannot silently lose the message.
- R-6.4 Not urgent: injected if the prompt is free; otherwise it waits for the next tick.

### Design
(to expand: system cron's minimum is one minute, so built as a supervised loop.)

### Conversations and decisions
(to expand: operator 2026-09-20 "sidekick" recollection; AEF watcher 2026-10-02 in the operator's words; SQ-4 (urgent never into a busy prompt) superseded by the operator's "urgent bypass".)

## 7. Readiness: is the prompt free?

### Requirements
- R-7.1 Readiness is reported by the agent's own harness: the Stop hook marks ready at the end of a turn; the prompt-submit hook clears it before the next turn.
- R-7.2 It is not inferred from the screen: a long tool call looks idle but is not safe to type into.

### Design
(to expand)

### Conversations and decisions
(to expand: T-1207 Stop hook (April); idle classifier T-2402/T-3079 (July/September); T-3397 hook readiness and TermLink's "do build (b)" (2026-09-21).)

## 8. Injection

### Requirements
- R-8.1 One short line is typed into the agent's session (`termlink pty inject`).
- R-8.2 INJECTED (AEF: HANDED_OVER) is reported only with evidence the agent saw the message, from its transcript.
- R-8.3 The flag comes down only when the queue is empty.

### Design
(to expand)

### Conversations and decisions
(to expand: T-2396 inject lands unsubmitted; evidence rules; D-696.)

## 9. Answering

### Requirements
- R-9.1 ANSWER READY: the receiver's sidecar tells the sender's sidecar; the sender pulls the answer.
- R-9.2 Roles swap for the reply.
- R-9.3 A woken agent always replies or explicitly says "no action"; never silence.
- R-9.4 If an agent's ears are dead, it halts instead of assuming there is no mail.

### Design
(to expand)

### Conversations and decisions
(to expand: T-2402 Stage 6 (July); DEAF→halt, deterministic-notify-sidecar (June); S12 roles swap.)

## 10. Fallback polling

### Requirements
- R-10.1 Where a push cannot land, the other side polls: 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week, 1 month, 1 quarter, 1 year, each rung twice.
- R-10.2 This ladder is the framework default for all polling, changeable per situation.

### Design
(to expand)

### Conversations and decisions
(to expand: AEF T-3434 retry ladder; operator ladder 2026-10-03; AEF T-3770.)

## 11. Addressing and identity

### Requirements
- R-11.1 The five-level circuit: host / hub / project / session / agent.
- R-11.2 Each level has a canonical name and an instance identity: FQDN for the host, a hub name, the `pid` for the project, a role for the agent.
- R-11.3 Messages are addressed with `to_circuit` and never fall back across projects.

### Design
(to expand)

### Conversations and decisions
(to expand: D-599/T-3433; T-3325 Q1 = C; AEF 7-vendor naming review @250; T-3751.)

## 12. Telemetry

### Requirements
- R-12.1 Every step is a timestamped event, copied to the hub, pullable from any agent.
- R-12.2 Each agent posts a daily digest and reflects on it.
- R-12.3 Alarms only for urgent messages; everything else escalates when it piles up, like audit warnings.
- R-12.4 Later: an observability database and the learning from it; possibly a hub-steward agent.

### Design
(to expand)

### Conversations and decisions
(to expand: T-3330 IW-1 = C amended; external review 4/4; T-3319; T-3333.)

## 13. Deployment

### Requirements
- R-13.1 Every sidecar ships with every TermLink deployment.

### Design
(to expand)

### Conversations and decisions
(to expand: operator 2026-10-03; releases ship the binary only today.)

## 14. Discussed once and then lost

### Requirements (to be confirmed or dropped by the operator)
- R-14.1 Native consumer: the agent confirms from inside its own turn (August, T-2838).
- R-14.2 Typed assignment and result messages (August).
- R-14.3 The hub refuses to call a message delivered without a receipt (August).
- R-14.4 The startup chain (September; never built).

### Conversations and decisions
(to expand)

## 15. Status

### What operates today
- 15.1 AEF has built most of sections 4–9 in its bleeding-edge sidecar, reaching only agents started with `claude-fw --termlink`.
- 15.2 On this host nothing injects into a running agent: our framework copy does not contain AEF's sidecar, and no agent here was started injectable.

### Detail
(to expand: recorded-as-built vs operating, with evidence.)

---

## Open decisions
(to fill: O1, O2, … each with both positions quoted.)

## Decision log
(to fill: every ruling, dated, with the operator's words.)
