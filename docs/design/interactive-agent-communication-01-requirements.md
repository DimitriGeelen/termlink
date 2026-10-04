# Interactive agent communication: step 1, requirements

**Task:** T-3344 · **Arc:** arc-011 · **Chain:** arc-011-design, step 1 · **Role:** requirements collector
**Status:** DRAFT in batch mode. The operator interview on the open questions (section 9) is pending, so completion condition 6.1 of the card is not yet met.
**Review link:** none yet. This project's Watchtower has no inline design-review page (profile P2.1). The operator reads this file in the repository.

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-04 | First draft, batch mode. The operator's confirmed requirements R-1.1..R-14.4 are re-cast into the role-card skeleton. The 18 open decisions are written as interview questions. | T-3344 |

## 1 Inputs of record

1.1 Short names used in this document. Line numbers are those of the file as hashed in 1.2.
1.1.a `RQ` = the headline requirements document.
1.1.b `IAC` = the full design.
1.1.c `RV1` = the external design review comparison.
1.1.d `RV2` = the routing consultation comparison.
1.1.e `T3330R`, `ARC11`, `HT`, `HA` and the other short names are as defined in the header of `RQ`.

1.2 Inputs, with the hash read at the start of this step.

| Short | Path | sha256 |
|---|---|---|
| RQ | `docs/design/interactive-agent-communication-requirements.md` | `1382ffcc0c028cb60a84e45cc9530c1958e0c06811a1cab6859af06078a6f8a1` |
| IAC | `docs/design/interactive-agent-communication.md` | `266fea47b5916d7b0181cf0f32968d408fb76a203d4b8da19d55f6f3a7e92196` |
| RV1 | `docs/reports/T-3335-design-review/comparison.md` | `a725852f9097511a9d2c2c67253ada3a3939b7483a9dbea36f1d512edfb6c887` |
| RV2 | `docs/reports/T-3335-routing-consult/comparison.md` | `7d40ef362096073b11628df256ea843d51788fc2f67fc14ca4bb5dd6aa7c7500` |

1.3 Also read: `docs/design/roles/cards/_common.md`, the requirements-collector card, the AEF adapter, the TermLink profile, `docs/design/roles/README.md`, the chain file and the hand-back schema.

1.4 How this step was run.
1.4.a Mode: batch. No human was present (`_common.md` 5.2).
1.4.b The operator's answers are the confirmed requirements R-1.1..R-13.1 of `RQ` (the operator said "Yes, that's fine", `RQ` header and decision log 2026-10-04). Section 14 of `RQ` (R-14.1..R-14.4) is labelled unconfirmed and is treated as unconfirmed here.
1.4.c Nothing in section 9 is decided. Section 9 is the list of questions the operator has not answered.
1.4.d Reviewer findings that the operator has not ruled on are in section 7 (Conflicts) and section 9 (Open questions). They are not in section 6 (Requirements).

## 2 Stakeholders, protected assets and adversaries

2.1 Stakeholders (the humans and agents that approve, use or are affected).
2.1.a **S-1 The operator.** The only human approver (profile P1.3). Talks to agents through each agent's own terminal. Wants 1:1, many-to-many and operator-to-agent conversation (`RQ` 1.1, 1 item 3).
2.1.b **S-2 Agents of this project.** Sessions of 010-termlink. Several may run at once (profile P3.1). They send, receive, get woken, reply.
2.1.c **S-3 Peer agents of other projects.** The framework agent (AEF, 999-Agentic-Engineering-Framework), 055-agentic-fleet-cockpit, 832-Workflow-designer, ring20-manager (hub .122), ring20-dashboard (hub .121), Penelope (profile P3.1). AEF built a parallel receiver (`RQ` 3 item 2).
2.1.d **S-4 Sidecars.** One process per agent. They hold the store, the flag and the queue, and they inject.
2.1.e **S-5 Hubs.** Carry topics, receipts, presence, artifacts and the fallback path.
2.1.f **S-6 Harnesses.** The programs that run an agent (Claude Code today, opencode at 055). They own the hooks and the transcript (`RQ` 7, `RV1` 13).
2.1.g **S-7 Maintainers of the estate.** Whoever deploys the sidecars (`RQ` 13).

2.2 Protected assets (what an attack or a failure would damage).
2.2.a **A-1 Message content and its durability.** A message that was accepted must not be lost.
2.2.b **A-2 The sender's knowledge of where its message is.** (`RQ` R-1.2.)
2.2.c **A-3 An agent's attention and session.** A typed line into a busy prompt can be discarded (T-2396, `RQ` 8 item 6).
2.2.d **A-4 Identity.** Project, agent and instance identity. All agents on one host sign as one TermLink identity today (`RQ` 11 item 5).
2.2.e **A-5 Credentials.** Hub secrets and TLS pins, sidecar API tokens (profile P3.2).
2.2.f **A-6 The truth of the status record.** The register and the docs must say what actually operates (`RQ` 15).

2.3 Adversaries. The operator has not named an adversary list. This list is the agent's proposal, built from failures that the record documents. The operator is asked to confirm it (gap GP-0, section 8).

| Id | Adversary | Documented evidence |
|---|---|---|
| ADV-1 | A confused or overloaded agent: busy when typed into, silent when woken, unaware of mail | T-2396; the 2026-10-03 incident, mail stored and receipted and never surfaced (`RQ` 5 item 2) |
| ADV-2 | A compromised or misbehaving session that injects, impersonates or force-interrupts | `RV1` point 17: any project can impersonate another with the one host key |
| ADV-3 | A component that reports success it did not earn (a dark guard that reads green) | `RQ` 15 item 2: arc-003, arc-004, S10: recorded as built and not operating |
| ADV-4 | A hub, host or network outage, or a blip | `RQ` R-3.2 "the hub goes down"; offline queue (`RQ` 4 item 1) |
| ADV-5 | A hurried or mis-heard operator: voice transcription turned "15" into "50" and "C" into "Cozzyte" | `RQ` 10 item 5; `RQ` 11 item 11 |
| ADV-6 | A stale or wrong binding: a second local hub, a wrong runtime directory, a deaf agent that looks alive | `RV2` items 36 and 65: two hubs on .107, an agent on the wrong one is deaf |
| ADV-7 | A malicious process with privilege on a host (reads secrets, calls the local sidecar API, edits the store) | Not documented as an incident. Named because the card requires it. The threat modeler (step 2) assesses it. |

2.4 What each stakeholder sees: the sender (S-2) sees the stages of section 4; the operator (S-1) sees escalations and, later, the digest; peers (S-3) see presence and receipts.

## 3 Drawings

### 3.1 Drawing D-1: context view (structure)

**Drawing D-1 — context: the system, its stakeholders, the adversaries and the neighbours**

```mermaid
flowchart LR
  OP["S-1 Operator"]
  PEER["S-3 Peer projects: AEF, 055, 832, ring20"]
  subgraph SENDHOST["Sender host"]
    SA["S-2 Sending agent"]
    SCA["S-4 Sender sidecar"]
  end
  subgraph HUBS["S-5 Hub: topics, receipts, presence, artifacts"]
    TOP["Durable topics and inbox"]
    PRES["Presence directory"]
  end
  subgraph RECVHOST["Receiver host"]
    SCB["S-4 Receiver sidecar: store, flag, queue"]
    HAR["S-6 Harness: hooks and transcript"]
    RA["S-2 Receiving agent in a TermLink PTY"]
  end
  ADV["Adversaries ADV-1 to ADV-7"]
  SA -->|"1 message and optional blob"| SCA
  SCA -->|"2 push by API (contested, OD-1)"| SCB
  SCA -->|"2b fallback post"| TOP
  TOP -->|"2c pull"| SCB
  SCB -->|"3 RECEIVED, STORED, INJECTED, ANSWER READY"| SCA
  SCB -->|"4 one fixed line, typed"| RA
  RA --- HAR
  HAR -->|"5 ready flag and transcript evidence"| SCB
  OP <-->|"talks through the agent terminal"| RA
  PEER --- TOP
  PEER --- PRES
  ADV -.->|"busy or silent agent, outage, impersonation, false green"| RA
  ADV -.-> SCB
  ADV -.-> TOP
```

3.1.1 Text equivalent of D-1.
3.1.1.a The sending agent hands a message and an optional blob to its own sidecar (arrow 1).
3.1.1.b The sender sidecar delivers it to the receiver sidecar by API (arrow 2). Whether this may cross hosts is open (OD-1). The fallback is a post to a hub topic that the receiver sidecar pulls (arrows 2b and 2c).
3.1.1.c The receiver sidecar reports the stages RECEIVED, STORED, INJECTED and ANSWER READY back to the sender sidecar (arrow 3).
3.1.1.d The receiver sidecar types one fixed line into the receiving agent's session (arrow 4). The receiving agent runs under a harness. The harness reports readiness and transcript evidence to the sidecar (arrow 5).
3.1.1.e The operator talks to an agent through that agent's terminal. No component for this leg is built (`RQ` 1 item 3).
3.1.1.f Peer projects reach the same hub topics and the presence directory.
3.1.1.g The adversaries act on the receiving agent, the receiver sidecar and the hub topics.

### 3.2 Drawing D-2: main flow (behaviour)

**Drawing D-2 — main flow from send to answer, including the refusal paths**

```mermaid
sequenceDiagram
  participant SND as Sending agent
  participant SS as Sender sidecar
  participant RS as Receiver sidecar
  participant HUB as Hub
  participant HRN as Harness
  participant RCV as Receiving agent
  SND->>SS: give message (priority, optional blob)
  SS->>RS: push by API
  alt receiver sidecar unreachable
    SS->>HUB: fallback post to inbox topic
    HUB-->>RS: pull when the sidecar is back
    Note over SS,RS: retry and polling ladders, then UNDELIVERABLE or ESCALATED
  end
  RS-->>SS: RECEIVED with timestamp
  RS->>RS: store durably, raise flag
  RS-->>SS: STORED with timestamp
  loop every 30 seconds
    RS->>RS: flag up? read queue, highest priority first
    alt urgent
      RS->>RCV: type one fixed line now (route contested, OD-2)
    else prompt free (harness says ready)
      RS->>RCV: type one fixed line
    else not free or agent not running
      RS->>RS: wait for next tick, report NOT RUNNING if so
    end
  end
  HRN->>RCV: prompt hook surfaces the stored message
  HRN-->>RS: transcript shows the message
  RS-->>SS: INJECTED with evidence (AEF name HANDED_OVER)
  RS->>RS: flag down only when queue empty
  RCV->>RS: answer, or explicit no action
  RS-->>SS: ANSWER READY
  SS->>RS: sender pulls the answer, roles swap
```

3.2.1 Text equivalent of D-2.
3.2.1.a The sender gives a message to its sidecar. The sidecar pushes it to the receiver sidecar by API.
3.2.1.b Refusal path 1, no receiver sidecar: the sender sidecar posts to the hub inbox topic. The receiver sidecar pulls it when it is back. Retry and polling ladders run, ending in UNDELIVERABLE or ESCALATED (R-30, R-31).
3.2.1.c The receiver sidecar answers RECEIVED, stores the message durably, raises the flag, then answers STORED (R-14, R-15, R-16).
3.2.1.d Every 30 seconds the receiver sidecar checks the flag and reads the queue, highest priority first (R-17, R-18).
3.2.1.e Urgent: it types one fixed line at once, even into a busy prompt (R-19). The route that makes this safe is open (OD-2).
3.2.1.f Not urgent and the harness says ready: it types the line (R-20, R-21).
3.2.1.g Refusal path 2, not ready or the agent is not running: the message waits for the next tick. NOT RUNNING is reported, never treated as busy (`IAC` 20a).
3.2.1.h The prompt hook surfaces the stored message. The transcript shows it. Only then is INJECTED reported to the sender (R-24). The flag comes down only when the queue is empty (R-25).
3.2.1.i The agent answers or says "no action" (R-28). The receiver sidecar tells the sender sidecar ANSWER READY. The sender pulls it. Roles swap (R-26, R-27).

### 3.3 Drawing D-3: lifecycle of a message (behaviour)

**Drawing D-3 — states of a message and the transitions the requirements allow**

```mermaid
stateDiagram-v2
  [*] --> SENT
  SENT --> RECEIVED : receiver sidecar calls back
  SENT --> REJECTED : receiver refuses
  SENT --> UNDELIVERABLE : retry budget used up
  RECEIVED --> STORED : durable store done
  STORED --> QUEUED : flag raised
  QUEUED --> QUEUED : not free or not running, wait for next tick
  QUEUED --> TYPED : urgent, or prompt free
  TYPED --> INJECTED : transcript evidence seen
  TYPED --> QUEUED : no evidence in the window, eligible again
  QUEUED --> ESCALATED : deadline passed
  INJECTED --> ANSWER_READY : agent answered
  INJECTED --> NO_ACTION : agent said no action, proposed state OD-8
  ANSWER_READY --> REPLIED : sender pulled the answer
  REPLIED --> [*]
  NO_ACTION --> [*]
  UNDELIVERABLE --> [*]
  REJECTED --> [*]
  ESCALATED --> [*]
```

3.3.1 Text equivalent of D-3.
3.3.1.a A message starts as SENT. It becomes RECEIVED, then STORED, then QUEUED when the flag is raised.
3.3.1.b A SENT message ends as REJECTED if the receiver refuses, or UNDELIVERABLE when the retry budget is used up.
3.3.1.c A QUEUED message stays QUEUED each tick while the prompt is not free or the agent is not running. It becomes TYPED when it is urgent or the prompt is free.
3.3.1.d TYPED is internal. It is not a stage the sender is told. The sender is told INJECTED only on transcript evidence (R-24). With no evidence in the window the message goes back to QUEUED.
3.3.1.e A QUEUED message whose deadline passes becomes ESCALATED.
3.3.1.f An INJECTED message ends as REPLIED (the sender pulled the answer) or as NO_ACTION. NO_ACTION is a proposed state that no implementation has. It needs an operator ruling (OD-8).
3.3.1.g The names INJECTED and HANDED_OVER, and the extra states, are not decided (OD-8). The drawing shows the states the requirements allow, with the proposed one marked.

## 4 Answers

4.1 The question set is drafted here, one question per topic of `RQ`, because the profile supplies none (card 2.2). Each answer is the operator's confirmed requirement, with the operator's own words where `RQ` records them. A question for which `RQ` holds no confirmed answer is listed in 4.3 as an open gap.

4.2 Answers from the confirmed requirements.

**QS-1 What does interactive communication mean, and who talks?**
4.2.1 Answer: agents hold interactive, two-way conversations while they work, 1:1, many-to-many and with the operator (R-1.1 of `RQ`). The sender always knows where its message is (R-1.2). No agent has to be attached or polling by hand for delivery to proceed (R-1.3). Source: operator read-back, `RQ` 1.
4.2.2 Operator's words: "I am absolutely very clear that I want to have this … after 20 times me telling you this is a really important part of this design, you don't have this quite essential part of bi-directional interactive communication." (`RQ` 1 item 11, CONSULT:13).
4.2.3 Follow-up QS-1.1, how does the operator talk to an agent? Answer on record: through the agent's own terminal. No component is built (`RQ` 1 item 3). Open as a design question, see GP-11.

**QS-2 What original capability must stay?**
4.2.4 Answer: keystroke injection and output read-back into a terminal (R-2.1), and conversation over the hub instead of fire-and-forget dispatch (R-2.2). Operator's words: "i wanted a means to simulate keyboard input and capture console output" (`RQ` 2 item 9, T243R:96).

**QS-3 What is a sidecar?**
4.2.5 Answer: one per agent; a separate, very simple process with an API (`RQ` R-3.1). Independent of the hub "because the hub goes down" (`RQ` R-3.2). It always respawns, portably, not only under systemd (`RQ` R-3.3). It carries the startup chain: start agent, session, project, hub (`RQ` R-3.4). It re-resolves when an FQDN or IP stops resolving (`RQ` R-3.5). Operator wording, "verbatim in intent": "a separate process exposing an API, deliberately independent of the hub because the hub goes down. Very simple. Always respawns. It also carries the startup chain — start agent, start session, start project, start hub — and re-resolves when an FQDN or IP stops resolving." (`RQ` 3 item 10, RAIL:111-117).
4.2.6 Operator's principle for respawn: "reliability and fragility, so I'm not to save a few tokens just to get a quick solution. I want a solid solution." (`RQ` 3 item 12, ARC11:360-361).
4.2.7 Follow-up QS-3.1, is the sidecar API local only or does it send across hosts? The record holds two answers that conflict, see conflict C-2 and OD-1.

**QS-4 How is a message sent?**
4.2.8 Answer: the sender gives the message, optionally with a blob, to its own sidecar's API (`RQ` R-4.1). The sender's sidecar delivers it to the receiver's sidecar API, push first (`RQ` R-4.2). The hub is the fallback, plus discovery and blob storage (`RQ` R-4.3). Operator's words: "Send: sender agent -> its own sidecar (API) -> the receiver's sidecar (API)." (T3330R:252). And "We have multiple hosts, so that's not a question." (`RQ` 4 item 6, T-3397:270-273).

**QS-5 How is a message received?**
4.2.9 Answer: RECEIVED, the receiver's sidecar calls the sender's sidecar immediately (`RQ` R-5.1). STORED, the message is stored durably and a second call goes back (`RQ` R-5.2). A new-message flag is raised (`RQ` R-5.3). Operator's words: "We store it and immediately go back to the sidecar receiver … an API call that's received." (`RQ` 5 item 10, T3330R:229). The read-back has four calls: RECEIVED, STORED, INJECTED, ANSWER READY (T3330R:252-259).

**QS-6 How often, and what does the tick do?**
4.2.10 Answer: a cron-style job checks the flag every 30 seconds (`RQ` R-6.1). With the flag up it reads the queue, highest priority first (`RQ` R-6.2). Urgent is injected immediately even when the agent is busy, by a route that cannot silently lose the message (`RQ` R-6.3). Not urgent: injected if the prompt is free, otherwise it waits for the next tick (`RQ` R-6.4). Operator's words: "Every 30 seconds … when the flag is up, the message queue gets read … urgent gets injected immediately. Non-urgent, we check again if the prompt is free. If the prompt is not free, we wait again until the next 30 seconds." (CONSULT:17).
4.2.11 Follow-up QS-6.1, what is urgent and how is it marked? Answer on record: the queue has a `priority` clamped to [-9,9], and urgent is band 5 or more by default in the built injector (`RQ` 6 item 3, ARC11 S8 and S9). No confirmed requirement fixes the marking. See candidate CAND-4 in OD-17.
4.2.12 Follow-up QS-6.2, urgent into a busy prompt. Answer: confirmed as a bypass (R-6.3). The operator's earlier ruling SQ-4 said the opposite and has not been recorded as superseded. Open: OD-2.

**QS-7 Who says the prompt is free?**
4.2.13 Answer: the agent's own harness. The Stop hook marks ready at the end of a turn, and the prompt-submit hook clears it before the next turn (`RQ` R-7.1). It is not inferred from the screen: a long tool call looks idle but is not safe to type into (`RQ` R-7.2).

**QS-8 What is typed, and when is it reported?**
4.2.14 Answer: one short line is typed into the session with `termlink pty inject` (`RQ` R-8.1). INJECTED (AEF: HANDED_OVER) is reported only with evidence from the agent's transcript that it saw the message (`RQ` R-8.2). The flag comes down only when the queue is empty (`RQ` R-8.3). Operator's words: "INJECTED: called back to the sender's sidecar once the message is in the prompt. … The flag is cleared only when the queue is empty." (T3330R:257-258).

**QS-9 How does the answer come back?**
4.2.15 Answer: ANSWER READY, the receiver's sidecar tells the sender's sidecar and the sender pulls the answer (`RQ` R-9.1). Roles swap for the reply (`RQ` R-9.2). A woken agent always replies or explicitly says "no action", never silence (`RQ` R-9.3). If an agent's ears are dead it halts instead of assuming there is no mail (`RQ` R-9.4).

**QS-10 What happens when a push cannot land?**
4.2.16 Answer: the other side polls: 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week, 1 month, 1 quarter, 1 year, each rung twice (`RQ` R-10.1). This ladder is the framework default for all polling, changeable per situation (`RQ` R-10.2). Operator's words: "that should be the standard fallback mechanism for the framework for any polling activities. It can be changed situationally, but that should be the standard." (IN-AEF@180).
4.2.17 Follow-up QS-10.1, was the first rung 15 s or 50 s? Dictation said "50". It was read as 15 and "pending operator correction" (T3330R:261). The operator confirmed the read-back, but no explicit correction is recorded. Open: OD-3.

**QS-11 How is an agent addressed?**
4.2.18 Answer: five levels, host, hub, project, session, agent (`RQ` R-11.1). Each level has a canonical name and an instance identity: FQDN for the host, a hub name, the `pid` for the project, a role for the agent (`RQ` R-11.2). Messages carry `to_circuit` and never fall back across projects (`RQ` R-11.3). Operator's decision on the key: T-3325 Q1 = C, recorded from a voice reading, overturnable (`RQ` 11 item 11).

**QS-12 What is recorded, reported and alarmed?**
4.2.19 Answer: every step is a timestamped event, copied to the hub, pullable from any agent (`RQ` R-12.1). Each agent posts a daily digest and reflects on it (`RQ` R-12.2). Alarms only for urgent messages. Everything else escalates when it piles up, like audit warnings (`RQ` R-12.3). Later: an observability database and learning from it, possibly a hub-steward agent (`RQ` R-12.4). Operator's words: "Liveness alarm only for urgent messages. Everything else accumulates and escalates when it piles up, like audit warnings (design to be decided)." (T3330R:265).

**QS-13 How does it ship?**
4.2.20 Answer: every sidecar ships with every TermLink deployment (`RQ` R-13.1). Operator's words: "are all the sidecars we develop … part of our [TermLink] package? When the deployment is done we should deploy all the sidecars with it." (T3330R:230).

**QS-14 Which earlier items were discussed once and lost? (UNCONFIRMED)**
4.2.21 `RQ` section 14 lists four candidates: R-14.1 native consumer, R-14.2 typed assignment and result messages, R-14.3 the hub refuses to call a message delivered without a receipt, R-14.4 the startup chain. The operator has not confirmed or dropped them. They are carried as R-40..R-43 with status "unconfirmed". Open: OD-13.

4.3 Open gaps: questions for which no answer is on record. Each is also a finding of the gap review (section 8).

| Id | Question | Status |
|---|---|---|
| QS-15 | Who may call a sidecar API, and with what credential? | Open gap GP-1 |
| QS-16 | What if a session in the conversation is compromised or hostile? | Open gap GP-2 |
| QS-17 | How many messages may one peer push, and how many may be outstanding? | Open gap GP-3 |
| QS-18 | What must keep working when the hub, the sender host or the receiver host is offline? | Open gap GP-4 |
| QS-19 | What is the recovery after a sidecar restarts in the middle of a delivery? | Open gap GP-5 |
| QS-20 | How is an agent, a peer, a credential or a role revoked? | Open gap GP-6 |
| QS-21 | Which identifiers are stable, who mints them, and what is the unit of "read"? | Open gap GP-7 |
| QS-22 | What does the sender see when the target cannot be reached by design (no hooks, no PTY, other OS)? | Open gap GP-8 |
| QS-23 | Who are the adversaries? | Open gap GP-0, list proposed in 2.3 |

## 5 Glossary

5.1 Each term is defined once. Later roles MUST use these meanings. A term marked (contested) has a definition the operator has not settled.

| Term | Meaning |
|---|---|
| Agent | A running AI session that does work. It has a role and an instance. |
| Circuit | The five-level address of an agent: host, hub, project, session, agent. Carried in `to_circuit`. |
| Conversation | A series of messages with one `conversation_id`. |
| Doorbell | The one fixed line typed into a session to make it look at its stored mail. It carries a count and ids, no peer content. |
| Flag | The new-message marker the sidecar raises when a message is stored and lowers only when the queue is empty. |
| Hub | The server that carries durable topics, receipts, presence and artifacts. Per host or per fleet. State is per hub (G-060). |
| Harness | The program that runs an agent and owns its hooks and its transcript. |
| Home hub | The hub that holds a project's identity and decides liveness for it (contested, OD-18, OD-11). |
| Injection | Typing the doorbell into the agent's session. Typing is not the same as the agent seeing the message. |
| INJECTED | The stage reported to the sender when transcript evidence shows the agent saw the message. AEF calls it HANDED_OVER (contested, OD-8). |
| Instance | One running copy at a level, for example one session. It has a runtime id. |
| Message | One post carrying content, a priority, an id and a conversation id, with an optional blob. |
| Polling ladder | The standard waiting cadence 15 s to 1 year, each rung twice (R-30). |
| Priority | A number in [-9,9] on a queued message. Urgent is a high band (contested, CAND-4 in OD-17). |
| Queue | The sidecar's store of messages not yet handed over, ordered by priority then arrival. |
| RECEIVED | The receiver's sidecar tells the sender's sidecar immediately that it has the message (contested, OD-4). |
| Retry ladder | AEF's schedule for re-sending a post, 2×1 min to 2×1 month, then dead-letter (D-600). Not the same as the polling ladder (contested, OD-3). |
| Role | The function an agent serves in a project, such as manager. A role is not an instance. |
| Sidecar | The separate, simple process per agent that stores, flags, queues, injects and reports. |
| STORED | The message is durable on the receiver side and a second call goes back (contested, OD-4). |
| Stage | One named step of delivery reported to the sender: SENT, RECEIVED, STORED, INJECTED, ANSWER READY, REPLIED and the failure stages. |
| Startup chain | Start agent, start session, start project, start hub (R-9, never built). |
| Tick | The 30-second check of the flag. |
| Transcript evidence | A record in the receiver's own session transcript that shows the agent saw the message. |
| Urgent | A message that is injected at once even when the agent is busy (R-19). How it is marked is open. |
| Unreachable | An agent that cannot be woken now. Not the same as dead. |

## 6 Requirements

6.1 How to read this section.
6.1.a R-1..R-39 are the operator's confirmed requirements (`RQ` R-1.1..R-13.1), carried over one for one. R-40..R-43 are unconfirmed (`RQ` section 14).
6.1.b Each requirement has: **a** statement, **b** type, priority and source, **c** rationale, **d** verification method, **e** acceptance criteria, **f** what it holds against (only where it says something MUST be impossible), **g** status note.
6.1.c Types: functional, security invariant, interface, operational, quality. Priority: P1 needed for the goal, P2 needed for a solid system, P3 later or unconfirmed.
6.1.d The closing rule of the operator's standing instruction applies to every verification below: nothing is called working until a live test with two real running agents and a negative control passes (profile P1.2.e, `IAC` item 58).
6.1.e Where a requirement is contested by a reviewer, the statement stays as the operator confirmed it. The contest is named in **g** and decided in section 9.
6.1.f "Status" facts come from `RQ` section 15 (2026-10-03) and were not re-measured in this step.

### 6.2 Goal (`RQ` section 1)

R-1 **Interactive conversation.**
R-1.a Agents MUST hold interactive, two-way conversations with each other while they are working: 1:1, many-to-many, and agents with the operator.
R-1.b Functional · P1 · source `RQ` R-1.1; operator, 2026-10-03: "I am absolutely very clear that I want to have this …".
R-1.c Rationale: this is the goal the other requirements serve.
R-1.d Verification: live test with two real agents (1:1), then three (many-to-many); a demonstration for the operator leg.
R-1.e Acceptance. (1) Given agents A and B both running, when A sends a turn with a `conversation_id`, then the turn is in B's transcript and B's reply, with the same id, reaches A. (2) Given agents A, B and C on one broadcast topic, when A posts, then B and C each receive it. (3) The operator leg has no acceptance criterion yet, because no component is designed for it (GP-11).
R-1.g Status: 1:1 and broadcast carry on the hub. The live leg into a running agent does not operate on this host. The operator leg is designed only.

R-2 **The sender always knows where its message is.**
R-2.a The sender MUST be able to see, for every message it sent, the stage the message has reached with a timestamp. A message MUST NOT be in transit with its stage invisible to the sender.
R-2.b Functional, security invariant · P1 · source `RQ` R-1.2.
R-2.c Rationale: the operator's failure was "you've been telling me it works" while mail sat stored and unseen (`RQ` 1 item 11).
R-2.d Verification: negative-control test. Stop the receiver's injector, then read the sender's record.
R-2.e Acceptance. Given a message was sent and the receiver's sidecar is stopped, when the sender reads its record, then the stage is SENT or waiting or overdue and is never INJECTED or delivered. Given the receiver is healthy, when the message passes each stage, then the sender's record carries one timestamped row per stage.
R-2.f Holds against: ADV-3 (false success), ADV-4 (outage), ADV-1 (silent agent).
R-2.g Status: built in pieces (sender ledger, ack tracker). The per-message timeline is designed, not built (`RQ` 1 item 4d).

R-3 **No one has to be attached.**
R-3.a No agent MUST have to be attached or polling by hand for delivery to proceed.
R-3.b Operational · P1 · source `RQ` R-1.3.
R-3.c Rationale: an agent in the middle of work cannot also watch a mailbox.
R-3.d Verification: live test with a receiver left idle at its prompt that runs no polling.
R-3.e Acceptance. Given the receiving agent is idle at its prompt and runs no polling, when a message is sent, then the message appears in its transcript within a stated bound (proposed: two ticks plus margin, not confirmed).
R-3.g Status: NOT met for agents on this host (`RQ` 1 item 5). A running session cannot be made injectable afterwards (PL-237). Conflict C-3, question OD-6.

### 6.3 Origins (`RQ` section 2)

R-4 **Keep keystroke injection and output read-back.**
R-4.a The system MUST keep the ability to inject keystrokes into a terminal session and read its output back.
R-4.b Interface · P1 · source `RQ` R-2.1; operator: "i wanted a means to simulate keyboard input and capture console output".
R-4.c Rationale: the founding verb of TermLink and the base of the injector.
R-4.d Verification: `scripts/session-selftest.sh` (spawn, exec a sentinel, cleanup).
R-4.e Acceptance. Given a spawned session, when `termlink exec <s> 'echo <sentinel>' --json` runs, then the result is ok, exit code 0, and stdout carries the sentinel.
R-4.g Status: built and operating.

R-5 **Conversation over the hub, not fire-and-forget.**
R-5.a Agents MUST be able to exchange conversation turns as durable hub messages that carry a `conversation_id`, in place of fire-and-forget dispatch.
R-5.b Functional · P1 · source `RQ` R-2.2.
R-5.c Rationale: T-256 replaced one-shot dispatch with a conversation.
R-5.d Verification: fixture and live test over `channel post` and `channel subscribe`.
R-5.e Acceptance. Given a topic and two agents, when A posts a turn with a `conversation_id` and B subscribes, then B reads the turn with the same id.
R-5.g Status: built on the hub.

### 6.4 The sidecar (`RQ` section 3)

R-6 **One sidecar per agent, simple, with an API.**
R-6.a Each agent MUST have one sidecar: a separate, very simple process with an API.
R-6.b Functional, quality · P1 · source `RQ` R-3.1; operator wording "a separate process exposing an API … Very simple."
R-6.c Rationale: carrying delivery must not depend on any LLM turn.
R-6.d Verification: process inspection and an API call; a review of size against a measure that the operator has not given.
R-6.e Acceptance. Given agent X is registered, when the agent's own process is killed, then X's sidecar is still running and its status call answers. Given two agents, then there are two sidecar processes.
R-6.g Status: built as shell scripts for three agents; the API that exists is local control only (`RQ` 3 item 1). "Very simple" has no measure (GP-12).

R-7 **Independent of the hub.**
R-7.a A sidecar MUST NOT need the hub to be up to accept, store, flag and inject a message from a sender on the same host.
R-7.b Operational · P1 · source `RQ` R-3.2; operator: "deliberately independent of the hub because the hub goes down".
R-7.c Rationale: a hub outage must not stop local delivery.
R-7.d Verification: stop the hub, then run the live test on one host.
R-7.e Acceptance. Given the hub is stopped, when agent A on host H sends to agent B on host H, then B's sidecar stores and flags the message and the flow of R-17..R-25 completes.
R-7.g Status: partial. TermLink's sidecar reads hub topics to learn of mail. AEF's receiver takes loopback mail with no hub (`RQ` 3 item 3). `RV1` section 2 says GLM and 055 call this requirement untestable or without a stated benefit. Question: OD-1.

R-8 **Always respawns, portably.**
R-8.a A sidecar that dies MUST be restarted by a supervisor on hosts with systemd and on hosts without it.
R-8.b Operational · P1 · source `RQ` R-3.3; operator: "I want a solid solution." (SQ-8).
R-8.c Rationale: a sidecar that stays dead is a silent deaf agent.
R-8.d Verification: kill the sidecar with SIGKILL and watch for the new process, on a systemd host and on a cron or launchd host.
R-8.e Acceptance. Given a running sidecar, when it is killed with SIGKILL, then a new sidecar for the same agent runs within the supervision interval (today 5 min) and its heartbeat is fresh.
R-8.g Status: built and operating on this host. Not verified on macOS.

R-9 **Carries the startup chain.**
R-9.a The sidecar MUST carry the startup chain: start the agent, the session, the project and the hub.
R-9.b Functional · P3 · source `RQ` R-3.4; operator spec wording (RAIL:111-117).
R-9.c Rationale: the operator wants one component that brings the whole chain up.
R-9.d Verification: start from a fully stopped state, in a test estate.
R-9.e Acceptance. Given the project, session, agent and hub are all stopped, when the startup chain runs, then each level is started in order and each is verified live before the next.
R-9.g Status: not built, not designed beyond one sentence. The scope is contested: the 7-vendor review says no agent is respawned by inbound mail without an explicit grant (`RQ` O15.2). Question: OD-15.

R-10 **Re-resolves a moved peer.**
R-10.a A sidecar MUST re-resolve a peer's address when the FQDN or IP stops resolving.
R-10.b Functional · P2 · source `RQ` R-3.5.
R-10.c Rationale: hosts change address.
R-10.d Verification: fixture that changes name resolution between two sends.
R-10.e Acceptance. Given a peer's FQDN now resolves to a new IP, when the sender's sidecar sends, then it uses the new address, or reports the failure to resolve. It never keeps sending to the old address silently.
R-10.g Status: partial. It notices a change and does not reconnect (`RQ` 3 item 6).

### 6.5 Sending (`RQ` section 4)

R-11 **Hand the message to the own sidecar.**
R-11.a The sender MUST give the message, optionally with a blob, to its own sidecar's API.
R-11.b Interface · P1 · source `RQ` R-4.1.
R-11.c Rationale: one local send point where durability and identity are applied.
R-11.d Verification: API fixture and live test.
R-11.e Acceptance. Given a sender and its sidecar, when the sender calls send with a message and a blob, then the call returns a message id, and the receiver verifies the blob's sha256 before the message is flagged.
R-11.g Status: there is no send API on the TermLink sidecar. Senders use `agent-send.sh` and `channel post` (`RQ` 4 item 1). AEF has `fw sidecar send` on one host.

R-12 **Sidecar to sidecar, push first.**
R-12.a The sender's sidecar MUST deliver the message to the receiver's sidecar API, trying push first.
R-12.b Functional · P1 · source `RQ` R-4.2; operator: "Send: sender agent -> its own sidecar (API) -> the receiver's sidecar (API)."
R-12.c Rationale: delivery without the hub as the first path.
R-12.d Verification: live test with a hub-side counter that stays unchanged on the push path.
R-12.e Acceptance. Given both sidecars are up, when a message is sent, then it reaches the receiver sidecar by a direct API call and no message post is made to a hub topic on that path.
R-12.g Status: same host in AEF only. Cross-host is designed only. All three weighted reviewers disagree for cross-host, and the charter forbids a second bus (`RV1` points 2 and 11). Conflict C-2, question OD-1.

R-13 **The hub is the fallback.**
R-13.a The hub MUST serve as the fallback path, and as discovery and blob storage.
R-13.b Functional · P1 · source `RQ` R-4.3.
R-13.c Rationale: a message must still arrive when no receiver sidecar answers.
R-13.d Verification: stop the receiver sidecar, send, restart it.
R-13.e Acceptance. Given the receiver's sidecar is down, when a message is sent, then it is on the receiver's hub inbox topic, and when the sidecar returns it is pulled, stored and flagged, and the sender's record shows each stage reached.
R-13.g Status: built and operating.

### 6.6 Receiving (`RQ` section 5)

R-14 **RECEIVED, immediately.**
R-14.a On accepting a message the receiver's sidecar MUST call the sender's sidecar with RECEIVED and a timestamp, immediately.
R-14.b Interface · P1 · source `RQ` R-5.1; operator: "an API call that's received".
R-14.c Rationale: the sender learns at once that the message arrived.
R-14.d Verification: live test with timestamps on both sides.
R-14.e Acceptance. Given the receiver sidecar is up, when it accepts a message, then the sender's record gains a RECEIVED row, with the receiver's timestamp, within a bound (proposed: 2 s, not confirmed).
R-14.g Status: TermLink posts a hub receipt on a 15 s poll, per topic, with no per-message id (`RQ` 5 item 1). AEF answers synchronously, same host. Contested: OD-4 (one call or two), OD-5 (callback or record and pull).

R-15 **STORED, durably, then a second call.**
R-15.a The receiver MUST store the message durably, and then call the sender again with STORED. STORED MUST NOT be reported before the store is durable.
R-15.b Functional, security invariant · P1 · source `RQ` R-5.2; operator read-back "STORED: called again once stored."
R-15.c Rationale: a message that survives a sidecar restart is the base of "no silent loss".
R-15.d Verification: kill the receiver sidecar after STORED and restart it.
R-15.e Acceptance. Given STORED was reported, when the receiver sidecar is killed and restarted, then the message is still in the store with the same id. Given a store that fails, then STORED is not reported.
R-15.f Holds against: ADV-3, ADV-4.
R-15.g Status: no distinct STORED call exists in either build. AEF's RECEIVED already means stored. Contested: OD-4.

R-16 **A new-message flag is raised.**
R-16.a After the message is durably stored the sidecar MUST raise a new-message flag. The flag MUST NOT be raised before the store.
R-16.b Functional · P1 · source `RQ` R-5.3.
R-16.c Rationale: the tick reads the flag, so the flag is the trigger for injection.
R-16.d Verification: fixture and live inspection of the flag file.
R-16.e Acceptance. Given a message is stored, then the flag exists with a timestamp and a pending count, and the message is already readable from the store at that moment.
R-16.g Status: built and operating for three agents. Its pending count is shared with the sidecar's own auto-ack, so 0 can mean "receipted", not "seen" (`RQ` 5 item 2).

### 6.7 The 30-second tick (`RQ` section 6)

R-17 **Check the flag every 30 seconds.**
R-17.a A job MUST check the new-message flag every 30 seconds.
R-17.b Functional · P1 · source `RQ` R-6.1; operator: "Every 30 seconds …".
R-17.c Rationale: bounds pickup latency when the agent is idle. System cron cannot do 30 s, so it is a supervised loop (`IAC` 17a).
R-17.d Verification: tick log over 100 consecutive ticks.
R-17.e Acceptance. Given a running sidecar, then consecutive ticks are at most 30 s plus a stated tolerance apart, and a tick that finds the flag up proceeds to R-18.
R-17.g Status: TermLink: not built (installed driver is `*/5`, checks no prompt). AEF: built (`SIDECAR_TICK`, default 30) for armed agents only.

R-18 **Read the queue, highest priority first.**
R-18.a With the flag up, the job MUST read the queue in order of priority, highest first, and by arrival time within one priority.
R-18.b Functional · P1 · source `RQ` R-6.2.
R-18.c Rationale: urgent mail must not wait behind routine mail.
R-18.d Verification: fixture with three queued messages.
R-18.e Acceptance. Given queued messages P=5 (arrived second), P=0 (first) and P=-1 (third), when the tick reads the queue, then the order is P=5, P=0, P=-1. Two messages with equal priority are read oldest first.
R-18.g Status: built inside the injector, which nothing schedules (`RQ` 6 item 1).

R-19 **Urgent: injected immediately, by a route that cannot silently lose it.**
R-19.a An urgent message MUST be injected immediately, even when the agent is busy, by a route that cannot silently lose the message.
R-19.b Security invariant · P1 · source `RQ` R-6.3; operator, 2026-10-03: "inject when it's free and with urgent bypass" and "urgent gets injected immediately".
R-19.c Rationale: the operator wants an interruption that is not waited out. The qualifier is the operator's own: "Reliability is important at all times but can also be out of band." (SCAPI:207-208).
R-19.d Verification: live test with the receiver inside a long tool call, plus a negative control in which the typed line is discarded.
R-19.e Acceptance. Given an urgent message and a receiver in the middle of a long tool call, when the message is handled, then (1) it is already durable in the store, (2) a line is typed at once, (3) either transcript evidence appears and INJECTED is reported, or the message becomes eligible again and an escalation follows, and (4) the message count in the store is the same before and after. A discarded typed line never causes the message to vanish.
R-19.f Holds against: ADV-1 (a busy agent discards the typed line, T-2396), ADV-4.
R-19.g Status: contested. Two of three weighted reviewers say never type into a busy prompt, and the operator's own earlier ruling SQ-4 says the same and is not recorded as superseded (`RQ` 6 item 16a, `RV1` point 10). No safe route is built (`RQ` 6 item 6). Conflict C-1, question OD-2.

R-20 **Not urgent: only when the prompt is free.**
R-20.a A non-urgent message MUST be injected only if the prompt is free. Otherwise it MUST wait for the next tick.
R-20.b Functional · P1 · source `RQ` R-6.4; operator: "If the prompt is not free, we wait again until the next 30 seconds."
R-20.c Rationale: typing into a busy prompt loses the text.
R-20.d Verification: live test with a busy receiver and a negative control.
R-20.e Acceptance. Given a non-urgent message and a receiver that is not ready, when a tick runs, then nothing is typed and the message stays queued with the flag up. Given the receiver becomes ready, when the next tick runs, then the line is typed. If the agent is not running, then NOT RUNNING is reported and not treated as busy.
R-20.g Status: TermLink built it behind the screen classifier, not scheduled. AEF built it behind the ready flag, armed agents only.

### 6.8 Readiness (`RQ` section 7)

R-21 **Readiness is reported by the harness.**
R-21.a The harness's own hooks MUST report readiness: the Stop hook marks the session ready at the end of a turn and the prompt-submit hook clears it before the next turn. A missing or unreadable flag MUST read as not ready.
R-21.b Interface · P1 · source `RQ` R-7.1.
R-21.c Rationale: only the harness knows a turn is open.
R-21.d Verification: hook fixture and live test.
R-21.e Acceptance. Given the Stop hook fired for session S, then S's record says ready. Given a prompt is submitted, then S's record says not ready before the new turn starts. Given S's record is missing, then S reads not ready.
R-21.g Status: AEF built it for `claude-fw --termlink` sessions. TermLink has none. Reviewers want a harness-neutral adapter (`RV1` point 13). Question: OD-7.

R-22 **Readiness is not inferred from the screen.**
R-22.a The system MUST NOT decide that a prompt is free from what the screen shows.
R-22.b Security invariant · P1 · source `RQ` R-7.2.
R-22.c Rationale: a long tool call looks idle and is unsafe to type into.
R-22.d Verification: live test with a long silent tool call.
R-22.e Acceptance. Given a receiver inside a 60-second silent tool call with a quiet screen, when a tick runs, then no non-urgent line is typed.
R-22.f Holds against: ADV-1.
R-22.g Status: the built TermLink classifier infers from the screen. `IAC` still says so. The operator once said "use PTY inject when the cursor is silent". Conflict C-5, question OD-7.

### 6.9 Injection (`RQ` section 8)

R-23 **One short line is typed.**
R-23.a The sidecar MUST type one short line into the agent's session with `termlink pty inject`. The line MUST NOT carry peer content.
R-23.b Interface · P1 · source `RQ` R-8.1.
R-23.c Rationale: the content arrives by the prompt hook, so a lost keystroke cannot lose content.
R-23.d Verification: fixture that records the typed text.
R-23.e Acceptance. Given a queued message, when the sidecar injects, then the typed text is one line, holds a count and message ids and no message body, and the target is the session named in the claim written before typing.
R-23.g Status: built in AEF and in the TermLink injector. Operating for armed sessions only.

R-24 **INJECTED only with evidence.**
R-24.a INJECTED (AEF: HANDED_OVER) MUST be reported only when the agent's transcript shows it saw the message. A typed line alone MUST NOT be reported as INJECTED.
R-24.b Security invariant · P1 · source `RQ` R-8.2; operator: "INJECTED: called back to the sender's sidecar once the message is in the prompt."
R-24.c Rationale: "a rung is not a read" (PL-253).
R-24.d Verification: the receiver-side prover `scripts/session-message-selftest.sh`, plus a discard negative control.
R-24.e Acceptance. Given a line typed into a prompt that discards it, when the evidence window passes (AEF: 90 s), then INJECTED is not reported and the message is eligible again. Given the transcript carries the message, then INJECTED is reported with its timestamp.
R-24.f Holds against: ADV-3 (a sidecar that claims success), ADV-1.
R-24.g Status: AEF built it. The TermLink injector posts a weaker stage on a BUSY transition. Mapping is undecided. Contested: OD-8.

R-25 **The flag comes down only when the queue is empty.**
R-25.a The sidecar MUST lower the flag only when the queue is empty.
R-25.b Functional · P2 · source `RQ` R-8.3.
R-25.c Rationale: a lowered flag with queued mail hides that mail from the tick.
R-25.d Verification: fixture with two queued messages.
R-25.e Acceptance. Given two queued messages, when one is handed over, then the flag stays up. When the second is handed over, then the flag is down.
R-25.g Status: designed. Not verified as built. The injector has an open gap that re-serves the oldest offset (`RQ` 8 item 5).

### 6.10 Answering (`RQ` section 9)

R-26 **ANSWER READY.**
R-26.a When an answer is ready, the receiver's sidecar MUST tell the sender's sidecar, and the sender MUST be able to pull the answer.
R-26.b Interface · P1 · source `RQ` R-9.1; operator read-back "ANSWER READY: … the sender pulls it."
R-26.c Rationale: the sender learns of the answer without polling.
R-26.d Verification: live test.
R-26.e Acceptance. Given the receiver stored a reply, when it calls ANSWER READY, then the sender's record shows it with a timestamp, and the pulled answer equals the reply.
R-26.g Status: not built in TermLink. AEF delivers a reply as a new send. Contested: OD-5.

R-27 **Roles swap for the reply.**
R-27.a The reply MUST use the same path and the same stages with sender and receiver swapped.
R-27.b Functional · P2 · source `RQ` R-9.2.
R-27.c Rationale: one path, one set of guarantees.
R-27.d Verification: live test of both directions.
R-27.e Acceptance. Given a reply from B to A, then A's sidecar shows RECEIVED, STORED and INJECTED for it, with timestamps, as B's did for the first message.
R-27.g Status: partial. TermLink's reply goes through `channel.post` (`RQ` 9 item 3).

R-28 **A woken agent replies or says "no action".**
R-28.a A woken agent MUST reply, or explicitly say "no action". It MUST NOT stay silent.
R-28.b Security invariant · P1 · source `RQ` R-9.3.
R-28.c Rationale: "silence always means a bug" (T-2402).
R-28.d Verification: live test with a negative control in which the agent posts nothing.
R-28.e Acceptance. Given an injected message, when the agent's turn ends with neither a reply nor a declared no-action, then an escalation is raised within the deadline. Given the agent posts an acknowledged-no-action, then the sender sees that terminal state.
R-28.f Holds against: ADV-1.
R-28.g Status: skill text only, no hook enforces it. Neither sender state set has a no-action state (`RQ` 9 item 4). Question: OD-8.

R-29 **Deaf means halt.**
R-29.a If an agent's ears are dead it MUST halt message-dependent work and say so. It MUST NOT assume there is no mail.
R-29.b Security invariant · P1 · source `RQ` R-9.4 (June, confirmed).
R-29.c Rationale: "no flag" only means "no mail" if the listener is alive.
R-29.d Verification: stale-heartbeat fixture and live test.
R-29.e Acceptance. Given the sidecar heartbeat is stale, when the agent checks for mail, then the verdict is DEAF and the agent reports the halt. Given a fresh heartbeat and no flag, then the verdict is CLEAR.
R-29.f Holds against: ADV-3, ADV-4, ADV-6 (a deaf agent that looks alive).
R-29.g Status: `notify-check.sh` exists. Nothing runs it at a yield point, so no owner (`RQ` 9 item 5).

### 6.11 Fallback polling (`RQ` section 10)

R-30 **The polling ladder.**
R-30.a Where a push cannot land, the other side MUST poll on this schedule, each rung twice: 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week, 1 month, 1 quarter, 1 year.
R-30.b Functional · P2 · source `RQ` R-10.1; operator: "that should be the standard fallback mechanism for the framework for any polling activities."
R-30.c Rationale: a standard cadence instead of ad-hoc retries.
R-30.d Verification: simulated-clock test of the rung times.
R-30.e Acceptance. Given a failed push and a simulated clock, then polls occur at 15 s ×2, 1 min ×2, 5 min ×2 and so on through 1 year ×2, in that order.
R-30.g Status: designed only. The first rung "15" was read from the dictation "50" and has no recorded correction. All three weighted reviewers call the year-long polling wrong (`RV1` point 4). Conflict C-6, question OD-3.

R-31 **The ladder is the framework default.**
R-31.a The ladder of R-30 MUST be the framework default for all polling, and MUST be changeable per situation.
R-31.b Functional · P2 · source `RQ` R-10.2.
R-31.c Rationale: one default, no per-script cadence.
R-31.d Verification: config fixture.
R-31.e Acceptance. Given a poll with no explicit schedule, then it uses the default ladder. Given a per-situation override, then it uses the override and the default is unchanged.
R-31.g Status: designed only. AEF filed T-3770 to reconcile it with its retry ladder.

### 6.12 Addressing and identity (`RQ` section 11)

R-32 **Five-level circuit.**
R-32.a An agent MUST be addressed by a five-level circuit: host, hub, project, session, agent.
R-32.b Interface · P1 · source `RQ` R-11.1.
R-32.c Rationale: a message must name its target exactly.
R-32.d Verification: parser fixture on both grammars.
R-32.e Acceptance. Given a circuit in path form and in the V9 form, then both parse to the same five levels, and writing emits path form.
R-32.g Status: built as `to_circuit` (T-3325). It decides which sidecar wakes. Nothing wakes an agent on this host.

R-33 **Two identities per level.**
R-33.a Each level MUST have a canonical name and an instance identity: FQDN for the host, a hub name, the `pid` for the project, a role for the agent.
R-33.b Interface · P2 · source `RQ` R-11.2.
R-33.c Rationale: names survive restarts, instance ids tell copies apart.
R-33.d Verification: identity fixture.
R-33.e Acceptance. Given an address, then each of the five levels yields both a name and an instance id. Given the hub's TLS fingerprint rotates, then the hub name is unchanged.
R-33.g Status: partial. The hub id used today is the rotating TLS fingerprint and a hub name is not built. The project slot still uses the folder name (`RQ` 11 item 3). Questions: OD-11, OD-12.

R-34 **Never fall back across projects.**
R-34.a A message addressed with `to_circuit` MUST NOT be delivered to an agent of a different project.
R-34.b Security invariant · P1 · source `RQ` R-11.3; agreed by both sides (IN-AEF@129, IN-010@141).
R-34.c Rationale: a wrong delivery is worse than a visible failure.
R-34.d Verification: fixture with only a wrong-project sidecar present.
R-34.e Acceptance. Given `to_circuit` names project P and only project Q's sidecar is present, then nothing is delivered or woken at Q and the sender sees the target as unreachable.
R-34.f Holds against: ADV-2 (impersonation, shared host key), ADV-6 (wrong binding).
R-34.g Status: built for the wake decision. A message with no address still wakes everyone (`IAC` 11a). Question on the session level: OD-10.

### 6.13 Telemetry (`RQ` section 12)

R-35 **Every step is a timestamped event, copied to the hub.**
R-35.a Every step MUST be recorded as a timestamped event, copied to the hub, and pullable from any agent.
R-35.b Operational · P2 · source `RQ` R-12.1; operator: "to collect the telemetry of our communications … the different steps, how much time it takes and how much delay".
R-35.c Rationale: the operator wants to learn from delivery delays.
R-35.d Verification: fixture that sends a message through all stages, then a pull from a second agent.
R-35.e Acceptance. Given a message that passed four stages, when a different agent pulls its journey, then it receives events carrying message id, step, time, from and to, in order, and the copy to the hub did not sit in the message's own path.
R-35.g Status: designed only. The retention window is open: OD-16.

R-36 **Daily digest and reflection.**
R-36.a Each agent MUST post a daily digest and reflect on it.
R-36.b Operational · P3 · source `RQ` R-12.2; operator: "one times per day".
R-36.c Rationale: traffic data is only useful if someone reads it.
R-36.d Verification: scheduled-job fixture.
R-36.e Acceptance. Given 24 hours of traffic, then each agent has posted one digest with counts per step, delays, stuck messages and unanswered messages. A missing digest is itself visible.
R-36.g Status: not built.

R-37 **Alarms only for urgent.**
R-37.a The system MUST raise an immediate alarm only for urgent messages. All other problems MUST escalate when they pile up, like audit warnings.
R-37.b Operational · P2 · source `RQ` R-12.3; operator: "Liveness alarm only for urgent messages. Everything else accumulates and escalates …".
R-37.c Rationale: alarm fatigue.
R-37.d Verification: fixture with an overdue urgent message and a pile of routine ones.
R-37.e Acceptance. Given an urgent message unhandled past its deadline, then an alarm is raised at once. Given many routine messages unhandled, then no immediate alarm is raised and an escalation entry appears when the pile crosses a threshold (the threshold is to be decided).
R-37.g Status: designed only. How it surfaces, and where the last rung lands, is open (OD-14). "Design to be decided" is the operator's own phrase.

R-38 **Later: observability database, learning, hub steward.**
R-38.a The telemetry MUST stay readable by a later observability database, and a hub-steward agent MAY be added.
R-38.b Quality · P3 · source `RQ` R-12.4.
R-38.c Rationale: the operator wants learning from traffic later.
R-38.d Verification: review.
R-38.e Acceptance. Given telemetry events on the hub, a later consumer can read all of them without a change to the message path.
R-38.g Status: later. The steward is T-3333, captured.

### 6.14 Deployment (`RQ` section 13)

R-39 **Every sidecar ships with every deployment.**
R-39.a Every sidecar MUST ship with every TermLink deployment, and MUST start from the deployment and not from a source checkout.
R-39.b Operational · P1 · source `RQ` R-13.1; operator: "When the deployment is done we should deploy all the sidecars with it."
R-39.c Rationale: today about 13 scripts run only from `/opt/termlink`.
R-39.d Verification: install on a clean host or container from the release artifact, then start a sidecar.
R-39.e Acceptance. Given a clean host with the release artifact installed and no `/opt/termlink` checkout, when an agent is registered, then its sidecar can be started and passes the live test.
R-39.g Status: NOT met. Releases publish the binary only (`RQ` 13 item 1). Question: OD-9.

### 6.15 Unconfirmed: discussed once and lost (`RQ` section 14)

6.15.1 R-40..R-43 are not confirmed by the operator. They are listed so that nothing is silently dropped. Keep or drop: OD-13.

R-40 **Native consumer (unconfirmed).**
R-40.a For agents the project launches, the agent SHOULD confirm receipt from inside its own turn.
R-40.b Functional · P3 · source `RQ` R-14.1 (T-2838, GO 2026-08-25).
R-40.c Rationale: "A receipt cannot exist unless a turn happened." (T2838R:175-208).
R-40.d Verification: spike S2 re-run.
R-40.e Acceptance. Given an agent launched by the project, then no receipt exists for a message unless a turn of that agent took place.
R-40.g Status: partly met by AEF's prompt hook plus transcript evidence (`RQ` 14 item 1).

R-41 **Typed assignment and result messages (unconfirmed).**
R-41.a Assignments and results SHOULD be typed messages (`assignment.v0`, `result_manifest.v0`).
R-41.b Interface · P3 · source `RQ` R-14.2.
R-41.c Rationale: an orchestrator needs machine-readable hand-offs.
R-41.d Verification: schema fixture.
R-41.e Acceptance. Given an `assignment.v0` message, then a consumer validates it against the schema and a malformed one is refused with a stated reason.
R-41.g Status: helpers only, no verb uses them.

R-42 **The hub does not call a message delivered without a receipt (unconfirmed).**
R-42.a The hub SHOULD refuse to call a message delivered without a receipt.
R-42.b Security invariant · P3 · source `RQ` R-14.3.
R-42.c Rationale: "delivered" equal to "hub accepted" was the original failure.
R-42.d Verification: hub fixture.
R-42.e Acceptance. Given a post with no receipt, then the hub reports it as delivered-unconfirmed and never as delivered.
R-42.f Holds against: ADV-3.
R-42.g Status: not built. The CLI already says delivered-unconfirmed versus consumed (`RQ` 14 item 1).

R-43 **The startup chain (unconfirmed, duplicate).**
R-43.a Same sentence as R-9.
R-43.b Functional · P3 · source `RQ` R-14.4.
R-43.c Rationale: see R-9.
R-43.d Verification: see R-9.
R-43.e Acceptance: see R-9.
R-43.g Status: `RQ` itself says it duplicates R-3.4. Proposed disposition: merge into R-9. Not decided: OD-13.

## 7 Conflicts

7.1 How to read this section. A conflict is a place where two sources say different things about the same requirement. "Left open" means the operator has not ruled. Reviewer findings that the operator has not ruled on are here and in section 9, not in section 6.

7.2 Conflicts left open.

| Id | Conflict | Sources | Affects | Disposition |
|---|---|---|---|---|
| C-1 | Urgent into a busy prompt. The operator's 2026-10-03 words say bypass. The operator's SQ-4 (2026-09-23) says "urgent NEVER injects into a BUSY prompt". Codex and GLM say never type into a busy prompt. 055 agrees with the bypass on conditions. | `RQ` 6 item 16a; `RV1` section 2 and point 10 | R-19, R-23, R-24 | Left open: OD-2. R-19 stands as confirmed. SQ-4 is still recorded RESOLVED with no supersession. |
| C-2 | Cross-host send. The operator's read-back sends sidecar to sidecar. SQ-1 keeps sending on the hub, and the charter forbids a second bus. Codex, GLM and 055 all say the hub carries cross-host mail. In `RV2` round 2 Codex, GLM and 055 accept a hub-set-up circuit for established conversations, and AEF wants a hub-path binding and a socket only after a measurement. | `RQ` O1; `RV1` points 2 and 11; `RV2` items 22-33, 61-62, 72 | R-7, R-12, R-13 | Left open: OD-1. R-12 stands as confirmed. |
| C-3 | "No agent has to be attached" against the fact that a running session cannot be given a PTY afterwards (PL-237). Today R-3 is false for running agents on this host. | `RQ` O16 item 1, O3, section 15 | R-3 | Left open: OD-6. |
| C-4 | Independence from the hub against the hub fallback and discovery. The design's own paths use the hub. `RV1` calls the independence "untestable" (GLM) and "has no stated failure it buys" (055). | `RQ` 3 item 3; `RV1` section 2 | R-7, R-13 | Left open: OD-1. The acceptance criterion of R-7 is limited to same-host delivery so that it can be tested. |
| C-5 | Screen against harness for readiness. The operator said once "use PTY inject when the cursor is silent". R-22 forbids inferring from the screen. `IAC` section 3a text still describes the screen classifier. | `RQ` 7 item 12a | R-21, R-22 | Left open: OD-7. R-22 stands as confirmed. |
| C-6 | The polling ladder against the retry ladder, and against the reviewers. R-30 runs to a year. D-600 stops at about 76 days and dead-letters. All three weighted reviewers call the year-long polling wrong. The first rung "15" was a reading of "50". | `RQ` 10 item 7; `RV1` point 4 | R-30, R-31 | Left open: OD-3. |
| C-7 | Callbacks against record-and-pull. The operator's read-back has synchronous calls back to the sender. Codex and GLM call the call back "wrong as written". All three reviewers say the durable record is the truth and the callback is an optimisation. RECEIVED and STORED are two calls in the read-back and one in AEF. | `RQ` 5 items 12 and 13, 9 item 12; `RV1` points 5 and section 2 | R-14, R-15, R-26 | Left open: OD-4, OD-5. |
| C-8 | Address rulings. D-599 says `inbox:<circuit-id>`. D-660 says `inbox:<agent-id>` and "does NOT keep sidecar:". AEF says D-660's wording is being amended, and the amended text was not found. R-33 wants a hub name, and the id in use is a rotating fingerprint. | `RQ` 11 item 13 | R-32, R-33 | Left open: OD-12. |
| C-9 | Alarms only for urgent against GLM's liveness invariant, canary mail with deadlines. All three reviewers say an end-to-end canary is what detects drift. | `RQ` 12 item 12b; `RV1` point 9 | R-37 | Left open: OD-14, CAND-5. R-37 stands as confirmed. |
| C-10 | INJECTED needs transcript evidence (R-24). The built TermLink injector reports a weaker stage on a BUSY transition. The mapping to AEF's HANDED_OVER is "not decided anywhere". | `RQ` 8 items 3, 4 and 13 | R-24 | Left open: OD-8. |
| C-11 | Two parallel receivers. TermLink has shell sidecars that run from a checkout. AEF has a receiver, watcher and hooks, for wrapper-started agents. All three reviewers say one owner. | `RQ` 3 and 13; `RV1` point 7 | R-6, R-39 | Left open: OD-9. |
| C-12 | "Always respawns" and the startup chain against the 7-vendor result that no agent is respawned by inbound mail without an explicit grant. | `RQ` O15 item 2 | R-8, R-9 | Left open: OD-15. |
| C-13 | Several agents under one project address. A project or role address has no rule for which agent answers. | operator, 2026-10-04; `RV2` sections 9-14 | R-32, R-34 | Left open: OD-18. |

7.3 Conflicts resolved by the record.
7.3.a **Tick interval.** TermLink built `*/5`. AEF proposed 15 s, "never validated". The operator said 30 s on 2026-10-02 and 2026-10-03. Resolved: 30 s (`RQ` 6 item 16b). R-17 carries it.
7.3.b **Three calls or four.** The T-3330 brief had three calls. The read-back has four. Resolved: four, because it is later and the operator confirmed it (`RQ` 5 item 13a).
7.3.c **Receiver-to-sender direction of the answer.** R-26 says the sender pulls. AEF delivers the reply as a pushed new message. Not a conflict in the requirement, which the operator confirmed. It is a design difference for step 4. It is raised in OD-5.

## 8 Gap review

8.1 Method. Card 3.4 asks what the questions did not cover. Eight classes are checked, plus the adversary list and four more that the reviewers raised. For each: what the record says, and what is missing. A gap is not a requirement. Each gap goes to the operator, or to the threat modeler at step 2, in the column "Goes to".

| Id | Class | What the record shows | What is missing | Goes to |
|---|---|---|---|---|
| GP-0 | Adversary list | The operator named failures, not adversaries. | An operator-confirmed adversary list. 2.3 is the agent's proposal. | Operator, step 2 |
| GP-1 | Credential custody | One host key signs for every agent. AEF's receiver uses a bearer token on loopback. Hub secrets live in the runtime directory (`RQ` 11 item 5; profile P3.2). | Who holds which credential, who may call a sidecar API, how the sidecar proves who sent a message. GLM: sidecar-to-sidecar calls "have no trust model" (`RV1` point 17). | Step 2, OD-15 |
| GP-2 | Compromised sessions | AEF frames peer text as untrusted data (D-695, `RQ` O16 item 2). | No requirement says peer content is untrusted. No requirement limits what a compromised session can inject or claim. | CAND-1, step 2 |
| GP-3 | Approval fatigue and attention | The operator wants alarms only for urgent mail (R-37). Codex: "successful delivery can itself make the agents unusable" (`RV1` point 18). | Rate limits, a bound on outstanding requests, expiry and cancellation. No limit on how often an urgent message may interrupt. | CAND-12, OD-15 |
| GP-4 | Offline operation | R-7 and R-13 cover a hub outage. The offline queue covers hub blips (`RQ` 4 item 1). | What the sender sees when the receiver's host is offline for days. What a sidecar does when its own host has no network. Same-host delivery with the hub down is only partly built. | Step 4 |
| GP-5 | Recovery | R-15 covers restart after STORED. The injector has an open gap that re-serves the oldest offset (`RQ` 8 item 5). | The recovery rule after a sidecar dies mid-delivery. Exactly-once on retry: receiver-side dedupe on `client_msg_id` is mandatory because the ladder outlives the hub's 5-minute dedupe window (`RQ` O16 item 3). | CAND-2 |
| GP-6 | Revocation | The `sidecar:` alias has "no end date". | How a credential, a peer, a role holder or an address is revoked. How a revoked agent stops receiving. | Step 2, OD-12 |
| GP-7 | Resource identity | The hub id is a rotating TLS fingerprint. The project slot is the folder name. All agents on a host share one key. 055 measured a read marker per inbox instead of per instance (`RV2` item 70). | Which identifiers are stable, who mints them, and the unit of "read" (inbox, instance or message). | OD-10, OD-11, OD-12 |
| GP-8 | Unsupported targets | Injection needs a TermLink-owned PTY and a hook-capable harness. 055 runs opencode too. README promises macOS. Local models fail design review but that is not a target. | What the sender sees when the target has no hooks, no PTY, a headless session or another OS. Reachability must be a visible state, not an inference. | OD-6, OD-7, CAND-6 |
| GP-9 | Time and order | GLM: clock skew across hosts corrupts the timestamped timeline (`RV1` point 19). GLM: two paths plus retries break per-conversation order (`RV2` item 16). | A rule for which clock a timestamp uses. Sequence numbers per conversation. | CAND-13, CAND-18 |
| GP-10 | Version skew | 055: version skew makes a deaf agent look like an old one. AEF: refuse cross-version sends unless declared compatible (`RV2` item 66). | A version rule between sidecars. | CAND-13 |
| GP-11 | Operator leg | R-1 includes agents with the operator. No component is built or designed for it (`RQ` 1 item 3). | A design for how the operator is a party to a conversation. | Operator, step 4 |
| GP-12 | "Very simple" | R-6 says "very simple". The receive chain is about 2,460 shell lines (`RQ` 3 item 1d). | A measurable proxy for "simple". | Operator, step 3 |

8.2 What the gap review did not check: whether any of the status facts in section 15 of `RQ` changed after 2026-10-03; T-3770, T-3688 and T-3751 were not looked up.

## 9 Open questions

9.1 These are the questions the operator has NOT answered. They are `RQ` and `IAC` OD-1..OD-18. The orchestrator puts them to the operator one at a time, in this order, and waits for "next" before the following one (standing instruction, 2026-10-01).
9.2 For each question: **a** the question in one sentence; **b** why it matters; **c** options A to D, each with the strongest reviewer position, quoted from `RV1` or `RV2`; **d** the recommendation and its reason; **e** the requirements it changes; **f** sources. Nothing here is decided.
9.3 Reviewer positions are quoted from the two comparison files. "Codex", "GLM" and "055" are the three weighted reviewers of `RV1`. In `RV2` the reviewers are Codex, GLM, 055 and AEF.

### 9.4 OD-1 Cross-host send path

OD-1.a Question: may a sidecar push a message directly to a sidecar on another host, or does the hub carry every cross-host message?
OD-1.b Why: R-12 says push first. The charter forbids a second bus. The 2026-10-03 ruling hands the cross-host leg to AEF, and the charter objection then applies to AEF's plan.
OD-1.c Options.
OD-1.c.A The hub carries cross-host mail. Sidecar-to-sidecar push is for one host. Codex: "retain hub-mediated transport … If cross-host delivery while hubs are down is mandatory, explicitly amend the charter: assigning the second transport to AEF does not remove it from the architecture." 055: "Keep the hub as the cross-host carrier; sidecar-to-sidecar calls on one host only … A second bus adds a path, and E4 shows two paths diverge." GLM: "Cross-host sidecar-to-sidecar push *is* a second bus … The T-3330 ruling handing cross-host to AEF should be reopened." (`RV1` point 2).
OD-1.c.B Sidecar to sidecar across hosts, and amend the charter openly. The operator's read-back: "Send: sender agent -> its own sidecar (API) -> the receiver's sidecar (API)." and "We have multiple hosts, so that's not a question." (`RQ` O1). `RV1` section 2: "Your read-back says sender sidecar to receiver sidecar first, with the hub as fallback. All three say the hub should be the carrier, and that the alternative is a charter change that must be decided openly, not inherited through the T-3330 ruling."
OD-1.c.C The hubs set up a circuit and the sidecars then talk directly for an established conversation. Codex (`RV2` item 22): "direct circuits are a reasonable preferred transport for established conversations, provided their delivery contract survives reconnection and fallback." GLM: "A conversation is not merely N letters." Both attach conditions: set-up through the hubs, per-circuit short-lived credentials, one delivery contract on both paths (`RV2` items 25-27).
OD-1.c.D The conversation is a binding on the hub path, and a socket circuit comes only after a measurement. AEF (`RV2` item 62): "Build a socket circuit only when a measurement shows either: the hub path costs more than ~10 % of median turn time; hub outages break more than a few conversations a week."
OD-1.d Recommendation: A for the first build, with C or D decided later on measurements. Reason: A is what Codex, GLM and 055 said in the first review. It is consistent with AEF's measured position (D). It needs no charter change. It contradicts the operator's confirmed read-back, so only the operator can choose it.
OD-1.e Changes: R-7, R-12, R-13.
OD-1.f Sources: `RQ` O1; `RV1` points 2 and 11; `RV2` items 22-33, 61-62, 72.

### 9.5 OD-2 Urgent into a busy prompt, and the safe route

OD-2.a Question: when a message is urgent and the agent is busy, may the sidecar type into the prompt, and by what route is it kept from being lost?
OD-2.b Why: this is the T-2396 loss mode. R-19 confirms the bypass. SQ-4 says never, and is not recorded as superseded.
OD-2.c Options.
OD-2.c.A Confirm the bypass, type only the fixed doorbell line, content stored first. 055 (`RV1` section 2): "**For** the bypass: record SQ-4 superseded; type only the fixed one-line doorbell, never content, message durable first; measure re-injects".
OD-2.c.B Never type into a busy prompt. Deliver urgent content through the harness's own hook channel. GLM: "Typing into a busy PTY should remain forbidden, full stop." Deliver urgent via the Stop-hook context channel. `RV1` point 10: "GLM offers a route none of the documents considered: deliver urgent content through the harness's own hook-context channel, which cannot be lost as unsubmitted input."
OD-2.c.C Use an authenticated harness interrupt, and if none exists report the limit and escalate. Codex (`RV1` section 2): "Against typing into a busy terminal; prefer an authenticated harness interrupt, else report the limit and escalate. Keeping busy-PTY typing must be 'an explicit risk acceptance'".
OD-2.c.D Keep SQ-4. Urgent only shortens the wait for a free prompt. SQ-4 text: "Urgent shortens the WAIT; it does not bypass the prompt-free CHECK." (`RQ` 6 item 13).
OD-2.d Recommendation: A, with the content also delivered by the hook route of B, and SQ-4 recorded as superseded. Reason: it is the operator's confirmed rule. The reviewers' loss concern is about content, and R-23 already keeps content out of the typed line. A discarded doorbell then costs a delay, not a message. Re-injects are measured so the choice can be reversed on evidence.
OD-2.e Changes: R-19, R-23, R-24.
OD-2.f Sources: `RQ` O4; `RV1` point 10.

### 9.6 OD-3 Polling ladder against retry ladder

OD-3.a Question: is R-10.1 (15 s to one year, each rung twice) the right polling ladder, and how does it relate to AEF's retry ladder?
OD-3.b Why: topics are retention-bounded, so late rungs poll for mail that no longer exists. The first rung "15" was a reading of "50".
OD-3.c Options.
OD-3.c.A Keep R-10.1 as the operator confirmed, and confirm the first rung as 15 s. The operator, relayed to AEF: "that should be the standard fallback mechanism for the framework for any polling activities. It can be changed situationally, but that should be the standard." (`RQ` 10 item 5).
OD-3.c.B Two mechanisms. `RV1` point 4: "The year-long polling ladder (R-10.1) is wrong. Topics are retention-bounded, so late rungs poll for messages that no longer exist. All three want two separate mechanisms: bounded retry for re-sending, and a short wait that ends in a visible 'stuck' escalation." The 12-rung ladder stays the default for other polling.
OD-3.c.C Use AEF's retry ladder only: 2×1 min to 2×1 month, then dead-letter, about 76 days (D-600, `RQ` 10 item 4).
OD-3.c.D Keep R-10.1 and cap its rungs by the retention of the topic being polled.
OD-3.d Recommendation: B. Reason: all three reviewers say it, it does not drop the operator's ladder (it stays the framework default for other polling), and it gives the operator a visible "stuck". Ask the operator to confirm 15 s as the first rung in the same answer.
OD-3.e Changes: R-30, R-31.
OD-3.f Sources: `RQ` O11; `RV1` point 4.

### 9.7 OD-4 RECEIVED and STORED: one call or two

OD-4.a Question: does the receiver call the sender once (after the durable write) or twice (RECEIVED, then STORED)?
OD-4.b Why: the operator's read-back has two calls. AEF answers once, after the store.
OD-4.c Options.
OD-4.c.A Two calls, defined precisely. Codex (`RV1` section 2): "Keep both, defined precisely: RECEIVED = volatile, STORED = durable, only STORED releases the sender".
OD-4.c.B One call after the durable write. GLM: "answer once, after fsync-and-rename". 055: "RECEIVED after the durable write".
OD-4.c.C Two events on the wire only if a sender can act on the gap, otherwise one (`IAC` item 61).
OD-4.c.D Two calls, and RECEIVED may be merged into STORED when the store is faster than a set bound.
OD-4.d Recommendation: A. Reason: it keeps the confirmed requirement and gives each call a different meaning, so the sender can tell "arrived" from "safe".
OD-4.e Changes: R-14, R-15.
OD-4.f Sources: `RQ` O2; `RV1` section 2.

### 9.8 OD-5 Callback to the sender, or record and pull

OD-5.a Question: is delivery settled by callbacks to the sender's sidecar, or by a durable record that the sender pulls?
OD-5.b Why: a callback fails when the sender's sidecar is down. The operator's read-back is push by API with pull as the fallback.
OD-5.c Options.
OD-5.c.A Callbacks are the live path and are authoritative. The operator's design (`RQ` 5 item 12).
OD-5.c.B The durable record is the truth and callbacks are an optimisation. `RV1` point 5: GLM, "nothing may depend on [the callback] — it fails exactly when the sender is down." 055, "Settlement must come from the durable record", keyed by message id. Codex, "Receiver progress must not depend on sender availability. Pull repairs missed notifications."
OD-5.c.C Record and pull only, no callbacks. Codex and GLM: the call back is "wrong as written" and record-and-pull meets the need (T3330R:242).
OD-5.c.D Callbacks for the live path, and the hub record as the durable copy (`IAC` item 70).
OD-5.d Recommendation: B, which includes the callbacks of D. Reason: it keeps the operator's push and makes it safe when the sender is down.
OD-5.e Changes: R-14, R-15, R-26.
OD-5.f Sources: `RQ` O10; `RV1` point 5.

### 9.9 OD-6 How an already-running session becomes reachable

OD-6.a Question: how does a session that is already running, without a TermLink PTY, become something a sidecar can reach?
OD-6.b Why: PL-237: a running headless session cannot be retrofitted. R-3 is false for such sessions today.
OD-6.c Options.
OD-6.c.A Relaunch every agent through the reachable launcher. Codex: "Inventory capabilities; relaunch through the reachable launcher; UNREACHABLE until an end-to-end challenge succeeds". GLM: "Hooks-first; relaunch through one wrapper; an unwrapped session is 'pull-only — stated, not papered over'".
OD-6.c.B No relaunch. 055: "**No relaunch** ('not realistic for a mixed fleet'); a harness-side pull channel at the harness's yield points, marked pull-only".
OD-6.c.C Accept that running sessions are reached only by pull and the session-start listing, and show the state (`RQ` O3).
OD-6.c.D Relaunch through the launcher, and add a harness-side pull channel for the mixed fleet later.
OD-6.d Recommendation: A, with every unlaunched agent shown as not reachable. Reason: two of three reviewers say it, and it is the only option that never reports mail as delivered to a session that cannot receive it.
OD-6.e Changes: R-3 (its acceptance and status).
OD-6.f Sources: `RQ` O3; `RV1` section 2.

### 9.10 OD-7 Readiness: hooks or screen, and who owns it

OD-7.a Question: is readiness taken only from harness hooks, and who builds the hook for agents that are not AEF's?
OD-7.b Why: R-22 forbids the screen. The built TermLink classifier uses the screen. TermLink has no hook readiness, and its own T-3250 is captured with no ruling.
OD-7.c Options.
OD-7.c.A Hooks are primary and the screen classifier stays only as a labelled degraded fallback (`IAC` item 63).
OD-7.c.B Hooks only. Retire the screen classifier. `RV1` point 3: "Readiness comes from harness hooks, never from screen inspection, and a ready flag is only an observation."
OD-7.c.C Hooks through a harness adapter contract. `RV1` point 13: 055 "Wants an adapter contract with READY/BUSY/NOT RUNNING plus evidence, two adapters from day one, and a per-release parity test."
OD-7.c.D Keep the screen classifier as the main signal (the operator's 2026-09-20 words "use PTY inject when the cursor is silent").
OD-7.d Recommendation: C. Reason: all three reviewers want hooks, and 055 runs opencode, so a Claude-only wording is not enough. Ownership: TermLink owns the adapter contract and AEF supplies the Claude adapter, which matches Codex's "AEF should supply harness readiness/context adapters."
OD-7.e Changes: R-21 (harness-neutral wording), R-22, R-24.
OD-7.f Sources: `RQ` O8; `RV1` points 3 and 13.

### 9.11 OD-8 Stage names, and "acknowledged, no action"

OD-8.a Question: which one name means "the agent saw it", and is "acknowledged, no action" a terminal state?
OD-8.b Why: TermLink's INJECTED needs a BUSY transition. AEF's HANDED_OVER needs transcript evidence. "This is not decided anywhere." (T3330R:223).
OD-8.c Options.
OD-8.c.A One name, INJECTED (the operator's word), defined as transcript evidence. A typed line with no evidence is ATTEMPTED. Add a terminal ACKNOWLEDGED_NO_ACTION. `RV1` point 6: "Nothing may be called INJECTED/HANDED_OVER without transcript evidence (GLM: otherwise 'ATTEMPTED'). All three add a terminal 'no action' state."
OD-8.c.B One name, HANDED_OVER (AEF's word, `IAC` item 74), with the same additions.
OD-8.c.C Two names: INJECTED for the typed line with a BUSY transition, HANDED_OVER for transcript evidence (today's split).
OD-8.c.D Keep the names and add only the no-action state.
OD-8.d Recommendation: A. Reason: it is the operator's own word, it meets R-24, and "ATTEMPTED" keeps the weaker TermLink evidence visible without calling it more than it is.
OD-8.e Changes: R-24, R-28.
OD-8.f Sources: `RQ` O13; `RV1` point 6.

### 9.12 OD-9 Who owns the receive side, and how sidecars ship

OD-9.a Question: whose receive side is the one that ships: AEF's receiver, TermLink's scripts, or a new `termlink sidecar` in the binary?
OD-9.b Why: R-39 says every sidecar ships with every deployment. Releases ship the binary only. Two receivers exist.
OD-9.c Options.
OD-9.c.A Adopt AEF's receiver (T-3330 option A). It reaches only AEF-governed projects.
OD-9.c.B Extend TermLink's scripts (option B). No package ships them.
OD-9.c.C Move the receive side into the binary as `termlink sidecar` (option C), with AEF supplying the harness adapters. Codex (`RV1` point 7): "TermLink should own the transport-neutral message lifecycle … AEF should supply harness readiness/context adapters." Section 2: "One versioned runtime, ideally `termlink sidecar`".
OD-9.c.D Defer (option D).
OD-9.d Recommendation: C. Reason: it is the only option that ships through the existing channels, and it gives one owner, which all three reviewers asked for. GLM: "one owner, either TermLink's supervisor or AEF's watcher, not both." The port is about 2.5-3.9k shell lines.
OD-9.e Changes: R-6, R-39.
OD-9.f Sources: `RQ` O9; `RV1` points 7 and section 2 packaging row.

### 9.13 OD-10 Session level of the address

OD-10.a Question: does a message address a role and a project, or an exact session?
OD-10.b Why: the operator's wording is "a canonical id plus a runtime id". Six of seven vendors said routing does not stop at a session label. Codex raised the incarnation problem.
OD-10.c Options.
OD-10.c.A Route to project and agent role. The session id is metadata (`IAC` item 64).
OD-10.c.B Route to the exact session, fenced by incarnation. `RV1` point 15 (Codex): "readiness generation, exclusive injection ownership, invalidation on restart; never silently redirect an exact-session message to another instance."
OD-10.c.C Both. `RV2` item 6: "Exact-instance messages fail loudly; role messages re-resolve. Never silently redirect."
OD-10.c.D Canonical id plus runtime id exactly as the operator worded it, no further rule.
OD-10.d Recommendation: C. Reason: it keeps role mail working when an instance restarts, and it never delivers an exact-session message to the wrong copy, which R-34 requires.
OD-10.e Changes: R-32, R-33, R-34.
OD-10.f Sources: `RQ` O6; `RV1` section 2 and point 15; `RV2` items 6, 47, 49.

### 9.14 OD-11 Name-to-id directory

OD-11.a Question: who keeps the directory that maps a project name to its id?
OD-11.b Why: R-33 needs stable ids. The decision in AEF T-3751 is open.
OD-11.c Options.
OD-11.c.A The hub keeps project identity cards (`IAC` item 65).
OD-11.c.B No directory. Ids ride inside names (7/7 vendor option B).
OD-11.c.C AEF keeps it.
OD-11.c.D Each project keeps its own.
OD-11.d Recommendation: A, with this rule from the routing consultation: "the directory may carry only identities the home hub minted and observed" (055, `RV2` item 37), and `RV2` item 71 ("hubs exchange a directory of who is present and able to receive, never messages; only the home hub may say dead and unknown is never dead"). Reason: the reviewers did not address ownership, but they agree on this rule and A is the option that applies it.
OD-11.e Changes: R-33.
OD-11.f Sources: `RQ` O5; `RV2` items 37, 71.

### 9.15 OD-12 Address rulings D-599 and D-660, and stable ids

OD-12.a Question: which address ruling stands, and does the `sidecar:` alias get an end date?
OD-12.b Why: D-599 says `inbox:<circuit-id>`. D-660 says `inbox:<agent-id>` and "does NOT keep sidecar:". AEF says D-660's wording is being amended. The hub id used is a rotating fingerprint.
OD-12.c Options.
OD-12.c.A D-599 stands. Ask AEF for the amended D-660 text. Give the alias an end date.
OD-12.c.B D-660 stands.
OD-12.c.C Both stay, indefinitely. D-660 says "Option 3 (support both) was explicitly refused".
OD-12.c.D D-599, plus a hub name as canonical and the fingerprint as instance id (R-33).
OD-12.d Recommendation: A. Reason: it needs no new decision of ours beyond asking AEF, and it ends the open-ended alias. The hub name of D is a separate build question.
OD-12.e Changes: R-32, R-33.
OD-12.f Sources: `RQ` O12.

### 9.16 OD-13 Section 14 items: keep or drop

OD-13.a Question: of R-40..R-43, which does the operator want kept?
OD-13.b Why: they were discussed once and lost. Nothing confirms them.
OD-13.c Options.
OD-13.c.A Keep R-41 and R-42 as later slices. Drop R-43 as a duplicate of R-9. Keep R-40 only if TermLink wants a consumer that is not AEF's.
OD-13.c.B Keep all four.
OD-13.c.C Drop all four and record them as dropped by decision.
OD-13.c.D Keep only R-40, since the by-construction receipt is its own safeguard.
OD-13.d Recommendation: A. Reason: it is the disposition that `RQ` O7 already suggests, and R-42 helps R-2's honest "delivered".
OD-13.e Changes: R-40..R-43.
OD-13.f Sources: `RQ` O7.

### 9.17 OD-14 Alarms, escalation and the hub steward

OD-14.a Question: besides urgent alarms and pile-up escalation, is there an end-to-end canary, and where does the last escalation land?
OD-14.b Why: the operator's rule is alarms only for urgent. All three reviewers say drift is caught only by a canary that sends a real message.
OD-14.c Options.
OD-14.c.A Urgent alarm, pile-up escalation like audit warnings, daily digest (`IAC` item 73). Nothing else.
OD-14.c.B A plus an end-to-end canary. `RV1` point 9: "What breaks first is deployment drift: healthy-looking sidecars with no scheduler, wrong hub, missing hooks. Detected only by an end-to-end canary that sends a real message, not by process heartbeats." GLM's R6: "canary mail with deadlines, INJECTED within T1, REPLIED within T2" (`RQ` O16).
OD-14.c.C Add a hub-steward agent (T-3333).
OD-14.c.D Defer.
OD-14.d Recommendation: B. Reason: the canary tests the rail, it is not a per-message alarm, so it does not break the operator's rule. The last rung must land in one place the operator reads, and that is proved by a negative control (T-3461 found an unread queue).
OD-14.e Changes: R-37.
OD-14.f Sources: `RQ` O14; `RV1` point 9; `IAC` item 73.

### 9.18 OD-15 Interrupt consent, respawn and the startup chain

OD-15.a Question: may a peer interrupt a working session, may inbound mail start an agent, and what is the startup chain?
OD-15.b Why: with the urgent bypass, a peer can force-interrupt a session, and one host key lets any project impersonate another.
OD-15.c Options.
OD-15.c.A Interrupts only from an authenticated sender and only for agents that allow it. Respawn only with an explicit operator grant, a budget and restart limits. `RV1` point 17: "sidecar-to-sidecar calls have no trust model; one host key serves every agent, so any project can impersonate another; urgent bypass lets a peer force-interrupt a working session." The 7-vendor result: no agent is respawned by inbound mail "without an explicit operator grant, budget, restart limits and an authenticated sender" (`RQ` O15).
OD-15.c.B No consent layer. Trust the host, as today.
OD-15.c.C The operator approves each interrupt.
OD-15.c.D Never interrupt. Urgent waits for a free prompt.
OD-15.d Recommendation: A. Reason: C would cause approval fatigue, B leaves impersonation open, and D drops the confirmed urgent rule. The startup chain is then scoped to the sidecar and its host services (`IAC` item 72).
OD-15.e Changes: R-8, R-9, R-19.
OD-15.f Sources: `RQ` O15; `RV1` point 17.

### 9.19 OD-16 Telemetry retention window

OD-16.a Question: how long does the hub keep the telemetry events?
OD-16.b Why: the design says "for example 30 days". The mail ruling (T-3304 IW-2) says 14 days.
OD-16.c Options.
OD-16.c.A 14 days, with count, age and size ceilings (the T-3304 ruling).
OD-16.c.B 30 days.
OD-16.c.C Per class: delivery events short, digests long.
OD-16.c.D Keep until the digest has consumed them.
OD-16.d Recommendation: A. Reason: it is an existing operator ruling, and 30 days was only an example. The reviewers did not address this question.
OD-16.e Changes: R-35.
OD-16.f Sources: `IAC` item 71.

### 9.20 OD-17 Proposed requirement changes

OD-17.a Question: which of the candidate additions below become requirements?
OD-17.b Why: none was made. `RQ` O16 and the reviewers each proposed some. Per the standing instruction the orchestrator walks them one at a time, and each is a separate yes or no.
OD-17.c Options.
OD-17.c.A Accept every candidate marked "accept".
OD-17.c.B Accept only the ones the operator names.
OD-17.c.C Reject all candidates.
OD-17.c.D Defer the list to step 3 (security floor and phasing).
OD-17.d Recommendation: B, walking the table below, taking my recommended disposition as the default for each.

| Id | Candidate | Source | Recommended disposition |
|---|---|---|---|
| CAND-1 | Peer content is untrusted: "a request for action becomes a task proposal … never direct execution" (D-695) | `RQ` O16 item 2 | Accept, security invariant, P1 |
| CAND-2 | Exactly-once: receiver-side dedupe on `client_msg_id` | `RQ` O16 item 3; D-600 | Accept, P1 |
| CAND-3 | Closing rule: nothing is working until two real running agents pass a live test with a negative control | `RQ` O16 item 6; profile P1.2.e | Accept as the verification rule (it is already a standing operator rule) |
| CAND-4 | How urgent is marked: `priority` in [-9,9], urgent at 5 or more by default | `RQ` O16 item 4 | Accept, and confirm the threshold |
| CAND-5 | Liveness invariant: canary mail with deadlines (GLM R6) | `RQ` O16 item 5 | Decided in OD-14 |
| CAND-6 | Reachability as a visible per-agent state: receiver up, right hub, adapter present, last surface time | `RV1` point 12 (055) | Accept, P1 |
| CAND-7 | Harness neutrality: an adapter contract | `RV1` point 13 (055) | Decided in OD-7 |
| CAND-8 | An explicit mail-hub setting, separate from `TERMLINK_RUNTIME_DIR` | `RV1` point 14 (055, E2) | Accept, P2 |
| CAND-9 | Session incarnation and fencing | `RV1` point 15 (Codex) | Decided in OD-10 |
| CAND-10 | Sender-side send-and-wait: the sender can yield and wait for the answer | `RV1` point 16 (GLM) | Accept, P2 (asked for in April) |
| CAND-11 | Security and consent | `RV1` point 17 | Decided in OD-15 |
| CAND-12 | Attention budgets: rate limits, bounded outstanding requests, expiry, cancellation | `RV1` point 18 (Codex, 055) | Accept, P2 |
| CAND-13 | Clock skew and version skew rules | `RV1` point 19; `RV2` item 66 | Accept, P2 |
| CAND-14 | Obligation contract and a visible "unanswered" state with escalation | `RV2` item 52 | Accept, P1. It extends R-28 and is the ring20-manager failure. |
| CAND-15 | Directory and liveness: only the home hub may say dead, and unknown is never dead | `RV2` item 71 | Decide with OD-11 and OD-18 |
| CAND-16 | Per-circuit short-lived credentials, one delivery contract on both paths | `RV2` items 26-27 | Only if OD-1 chooses circuits |
| CAND-17 | Fleet admission and signed advertisements | `RV2` item 15 (GLM) | Defer to the threat model |
| CAND-18 | Ordering: a sequence number per sender and conversation | `RV2` items 16, 27 | Accept, P2 |
| CAND-19 | One settlement record per `client_msg_id` at the destination | `RV2` items 39, 56 (055) | Accept, P1. It supports OD-5 option B. |

OD-17.d.1 Also pending, as consequences of other questions: the wording of R-14, R-15 and R-26 follows OD-4 and OD-5. R-33 follows OD-12. R-43 follows OD-13. The two lists of open decisions were merged in `RQ` and `IAC` (Codex's drift point, `RV1` point 20), so that item needs no decision.
OD-17.e Changes: new requirements, numbered after R-43.
OD-17.f Sources: `RQ` O16; `RV1` section 3; `RV2` sections 3, 6, 9, 12, 13.

### 9.21 OD-18 Several agents per project: who answers

OD-18.a Question: when a project has several agents, who answers a message addressed to the project or to a role, and who coordinates?
OD-18.b Why: the operator raised it on 2026-10-04. `RV2` names ring20-manager's 8 unanswered requests as the real failure.
OD-18.c Options.
OD-18.c.A A deterministic rule at the project's home hub, over a fenced lease held by code. Introduce, then step aside. No AI agent on the path. `RV2` item 44: "Introduce, then step aside (6c semantics), resolved at the project's home hub." Item 45: "Neither ordinary resolution nor lease renewal should await an AI turn." (Codex).
OD-18.c.B A central coordinator agent that carries the traffic. `RV2` item 43: "Not a coordinator that carries all traffic (6b). Both reject it as the default; Codex allows it only where work must be triaged or aggregated."
OD-18.c.C No exclusivity by rule. Codex (`RV2` section 10): "Rejected for exclusivity: observers with different views pick different winners". GLM adds fork-with-claim for pools: deliver to all holders, first to claim wins.
OD-18.c.D The sender names the exact instance, and no role resolution exists.
OD-18.d Recommendation: A for singleton roles, with fork-with-claim for pools later, eligibility rules (workers, reviewer seats and sub-agents may not hold a role, 055), and an obligation record with a visible "unanswered" state. Reason: all four consulted parties agree on this shape (`RV2` item 71), and "Resolution chooses whom to ask. Explicit acceptance establishes who owes an answer." (Codex, item 50).
OD-18.e Changes: R-32, R-34, new requirements after OD-17.
OD-18.f Sources: `RV2` sections 9-14.

## 10 Earlier requirements

10.1 The earlier list is `RQ` R-1.1..R-14.4. Every item is kept. None is changed or dropped, because the operator confirmed them and only the operator may change them. The R-n numbering is new.

| Earlier | Now | Disposition | Reason |
|---|---|---|---|
| R-1.1 | R-1 | kept | confirmed |
| R-1.2 | R-2 | kept | confirmed |
| R-1.3 | R-3 | kept | confirmed. Not met for running agents, OD-6 |
| R-2.1 | R-4 | kept | confirmed |
| R-2.2 | R-5 | kept | confirmed |
| R-3.1 | R-6 | kept | confirmed. Ownership OD-9 |
| R-3.2 | R-7 | kept | confirmed. Narrowed acceptance only, OD-1 |
| R-3.3 | R-8 | kept | confirmed |
| R-3.4 | R-9 | kept | confirmed. Scope OD-15 |
| R-3.5 | R-10 | kept | confirmed |
| R-4.1 | R-11 | kept | confirmed |
| R-4.2 | R-12 | kept | confirmed. Contested, OD-1 |
| R-4.3 | R-13 | kept | confirmed |
| R-5.1 | R-14 | kept | confirmed. Contested, OD-4, OD-5 |
| R-5.2 | R-15 | kept | confirmed. Contested, OD-4 |
| R-5.3 | R-16 | kept | confirmed |
| R-6.1 | R-17 | kept | confirmed |
| R-6.2 | R-18 | kept | confirmed |
| R-6.3 | R-19 | kept | confirmed. Contested, OD-2 |
| R-6.4 | R-20 | kept | confirmed |
| R-7.1 | R-21 | kept | confirmed. Wording may be made harness-neutral, OD-7 |
| R-7.2 | R-22 | kept | confirmed |
| R-8.1 | R-23 | kept | confirmed |
| R-8.2 | R-24 | kept | confirmed. Name OD-8 |
| R-8.3 | R-25 | kept | confirmed |
| R-9.1 | R-26 | kept | confirmed. Contested, OD-5 |
| R-9.2 | R-27 | kept | confirmed |
| R-9.3 | R-28 | kept | confirmed |
| R-9.4 | R-29 | kept | confirmed. No built owner |
| R-10.1 | R-30 | kept | confirmed. Contested, OD-3 |
| R-10.2 | R-31 | kept | confirmed |
| R-11.1 | R-32 | kept | confirmed |
| R-11.2 | R-33 | kept | confirmed. "Hub name" not built, OD-12 |
| R-11.3 | R-34 | kept | confirmed |
| R-12.1 | R-35 | kept | confirmed. Retention OD-16 |
| R-12.2 | R-36 | kept | confirmed |
| R-12.3 | R-37 | kept | confirmed. Surfacing OD-14 |
| R-12.4 | R-38 | kept | confirmed |
| R-13.1 | R-39 | kept | confirmed. Not met, OD-9 |
| R-14.1 | R-40 | kept (unconfirmed) | the operator has not confirmed or dropped it, OD-13 |
| R-14.2 | R-41 | kept (unconfirmed) | same |
| R-14.3 | R-42 | kept (unconfirmed) | same |
| R-14.4 | R-43 | kept (unconfirmed) | duplicate of R-9. Proposed merge, OD-13 |

## 11 Residual risks and notes for the operator

11.1 This draft is not final. Completion condition 6.1 of the card (every question answered or recorded as an open gap) is met only after the interview on section 9.
11.2 Section 6 states what the operator confirmed. At least five of those statements (R-7, R-12, R-14, R-19, R-30) are contested by reviewers. This document does not choose between them.
11.3 Several requirements describe a state that does not operate today: R-3, R-12, R-17, R-19, R-24, R-39. The status lines say so. The closing rule (CAND-3) is the test before any of them may be called working.
11.4 The adversary list in 2.3 is the agent's proposal and has not been confirmed (GP-0).
11.5 Facts about what runs on the host come from `RQ` section 15, dated 2026-10-03. They were not re-measured.
11.6 The Mermaid drawings were checked for rendering as recorded in the hand-back. The project's review surface has no design page yet (profile P2.1).

## 12 Change requests to earlier steps

12.1 None. This is step 1.
