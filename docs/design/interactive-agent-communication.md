# Interactive agent-to-agent communication: the design, front to end

**Owner:** the operator (design); TermLink and AEF (build) · **Task:** T-3335 · **Written:** 2026-10-03
**Supersedes nothing.** It consolidates `docs/design/arc-011-message-delivery-rail.md` (2026-09-22),
`docs/design/arc-011-sidecar-api-architecture.md` (T-3075), the operator rulings since then, and the
design as the operator restated it on 2026-10-03.

## 0. Why this document exists

The operator has stated this design many times. The agent kept losing parts of it and reported parts of it as working when they were not: on 2026-10-03 it described T-3325 as "the sidecar wakes only the right agent", while nothing reaches the agent at all. This document is the single place where the design is written down in full, so it is read rather than remembered. Section 11 says plainly what is operating today and what is only recorded as built.

## 1. The goal

1. **Two-way, interactive communication between running agents.** An agent sends a message to another agent. That agent receives it while it is working or idle, works on it, and answers. The answer comes back the same way.
2. **The sender always knows where its message is.** Every step is confirmed back to the sender with a timestamp.
3. **No agent depends on another being attached or polling by hand.** No LLM has to be "online" for delivery to proceed (arc-011: the cron drives it).
4. **This is the core of the design, not an add-on.** Telemetry, addressing, retry ladders and alarms exist to serve it.

## 2. Who talks: identity and addressing

5. **The five-level circuit address:** host / hub / project / session / agent (agreed with AEF, D-599 / T-3433; operator ruling 2026-09-25).
6. **Two identities per level** (operator 2026-10-03; AEF's 7-vendor review, @250: 7/7 adopt with changes):
   6a. **Host:** canonical = FQDN. The IP is the instance underneath and never appears in an address (7/7).
   6b. **Hub:** a canonical hub NAME plus an instance id (5/7). Today's hub "id" is its TLS fingerprint (`cacc73ea32b121dd`), which changes on certificate rotation (PL-021). It is an instance id and must not name durable topics. There may be several hubs per host.
   6c. **Project:** canonical id = the minted `pid` (`pid-<16 hex>`, AEF T-3534; back-filled by `fw upgrade`, T-3750); this is the only routing id (7/7). The readable name (`010-termlink`) is for display. A running checkout is a workspace id in metadata, not routable (6/7). A project may run as several instances, on one or several hosts and networks, and those instances may talk to each other.
   6d. **Session:** a canonical id plus a runtime id (operator). The review says routing does not stop at a session label (6/7). Still open.
   6e. **Agent:** canonical = a ROLE (e.g. "reviewer") plus the running instance (7/7). A message for a role lands on whatever instance holds it. If the instance died, the message still lands in the right place, but no agent is respawned by inbound mail without an explicit operator grant, budget, restart limits and an authenticated sender (7/7).
7. **The address on a message:** `metadata.to_circuit`, the counterpart of `from_circuit` (T-3325, ruling Q1 = C). It is read in path form (`//host/hub/project[/session][/@agent]`) and in AEF's V9 form (`aef::host=…::hub=…::project=…::session=…::@agent::`). It is written in path form now and switches when AEF cuts over.
8. **Matching is level by level from level 1.** A named level that differs means the message is not for this agent. If the named agent or session is not live, the message falls back to the deepest level that resolves. **It never falls back across projects** (AEF agrees, @141). A message with no address is for everyone, which is the old behaviour.
9. **The name → id directory** (AEF T-3751 IW-3) is open. The agent's brief recommended the hub keeps project identity cards; the operator's ruling is pending.

## 3. The parts

10. **The agent.** A Claude Code session (or another harness) doing work in one project.
11. **Its sidecar** (operator, arc-011 Q1):
    11a. a separate, very simple process with an API;
    11b. independent of the hub, because the hub goes down;
    11c. always respawns;
    11d. carries the startup chain (start agent, start session, start project, start hub) and re-resolves when an FQDN or IP stops resolving.
12. **The receiver's local store** (S3, `journal.sqlite`), and a **new-message flag** (S4).
13. **The queue**, ordered by priority (S8).
14. **The prompt-free check** (S7). It is independent of any LLM: it decides from the terminal's behaviour whether the agent's prompt is free.
15. **The injector** (S10). It puts the message into the agent's session and verifies the agent started working on it.
16. **The cron driver** (S11). It runs steps 23–27 on a schedule, so nothing needs an LLM online.
17. **The hub.** The durable substrate: topics, discovery, artifact (blob) storage, the telemetry record (§7), and the fallback path when a direct call cannot land.
18. **The sender's ledger** (S6). One row per message sent, with the step it has reached.

## 4. The message, front to end

19. **Send.** The sending agent hands the message, with an optional blob, to its own sidecar through the sidecar's API.
20. **Deliver.** The sender's sidecar delivers it to the receiver's sidecar through the receiver's API. (Open point O1: earlier ruling SQ-1 kept sending on the hub.)
21. **RECEIVED.** The receiver's sidecar immediately calls the sender's sidecar: "received", with a timestamp.
22. **STORED.** The receiver stores the message durably (it survives a restart), then calls the sender's sidecar again: "stored".
23. **Flag up.** A new-message flag is set for the receiving agent.
24. **Urgent?** The receiver reads the message's urgent flag.
    24a. **Urgent:** injected immediately (operator 2026-10-03). (Open point O2: ruling SQ-4 says urgent shortens the WAIT but never skips the prompt-free CHECK, re-probing for up to 120 s, because injecting into a busy prompt can silently lose the text, T-2396.)
    24b. **Not urgent:** queued. When the prompt is free, the injector takes the highest-priority message and injects it. While the prompt is busy, the cron-style check retries.
25. **INJECTED.** Once the message is in the agent's prompt, and the agent is verified to be working on it, the receiver's sidecar calls the sender's sidecar: "injected". Injection alone is not evidence: a bare inject can land unsubmitted and be discarded (T-2396), so INJECTED needs an observation, for example a BUSY transition or the message appearing in the receiver's transcript.
26. **Flag down.** The new-message flag comes down only when the queue is empty.
27. **The agent works** on the message.
28. **ANSWER READY.** When the answer is ready, the receiver's sidecar tells the sender's sidecar that an answer is ready, and the sender pulls it.
29. **Roles swap.** The answer travels the same path in reverse (S12): the receiver is now the sender, and steps 19–28 apply.

## 5. When a push cannot land: the standard polling ladder

30. Push is primary. Where a push cannot land, the other side polls.
31. **The ladder** is the framework default for ANY polling, changeable per situation. Each rung is polled twice: 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week, 1 month, 1 quarter, 1 year. (Dictation said "50" for the first and fourth rungs; read as 15, pending operator correction.)
32. Sent to AEF on framework:pickup @309 and AEF inbox @180. Filed as AEF **T-3770**: reconcile it with AEF's T-3434 retry ladder and make it the declared default for every poller.

## 6. Blobs

33. A message may carry a binary blob (S2). It uses TermLink's artifact path (`artifact.put`, content-addressed by sha256), not the legacy `file send`, and `--expected-sha256` is mandatory on fetch.

## 7. Telemetry (ruling T-3330 IW-1 = C amended, 2026-10-03)

34. **Every step is a timestamped event:** RECEIVED, STORED, INJECTED, ANSWER READY, plus SENT and failures. Each event carries the message id, the step, the time, from and to.
35. **Each sidecar records its own events and copies them to the hub** through a local outbox, in the background, never in the message's path. If the hub is down, the events wait and are sent later.
36. **The hub keeps the events for a retention window** (for example 30 days). Any agent can pull one message's journey or one agent's delays.
37. **Each agent posts a daily digest:** counts per step, delays between steps, and the stuck and unanswered messages. The digest is the durable summary.
38. **Agents reflect on the telemetry:** what it says about how well they communicate.
39. **Alarms: immediate only for messages marked urgent.** Everything else accumulates and escalates when it piles up, the way audit warnings do. How alarms and escalations surface is still to be designed.
40. **Later:** an observability database (the AEF agent, a specific hub or a specific agent; possibly bundled with every hub), where the learning from the message traffic happens. That is its own design inception, linked to T-3319. The operator also proposes a **hub steward agent** that owns telemetry, alarms, escalation and learning (inception T-3333).
41. Volume is small: about 21 messages a day on this hub, so about 100 events and 20 KB a day.

## 8. Deployment

42. **Every sidecar ships with every TermLink deployment** (operator 2026-10-03). Today releases ship only the binary, and the ~13 sidecar scripts run only from the `/opt/termlink` checkout (T-3330 research §3).
43. The sidecar must respawn portably, not only under systemd (SQ-8, "solid over quick": `notify-sidecar-supervisor.sh --loop`, `--emit-unit systemd|launchd|cron`).

## 9. Who builds what (current understanding)

44. **TermLink** supplies primitives: the hub, durable topics, discovery, artifact storage, `pty inject`, presence, and the telemetry record. Per SQ-1 (2026-09-23) the injector stays in TermLink as a primitive, and the sidecar API is a LOCAL control surface (status, queue, inject, agent-state, ack). It refuses any cross-host argument, because TermLink's charter rules out a second cross-host bus.
45. **AEF** has built a per-agent receiver (T-3693): an HTTP API on 127.0.0.1 with bearer-token auth, answering RECEIVED synchronously and recording HANDED_OVER and REPLIED, plus a 30-second watcher (T-3684/T-3685) that injects via `termlink pty inject`. Cross-host comes with T-3688. The proposed split (sent to AEF @184, no answer yet): AEF owns the sidecar protocol; TermLink owns the telemetry record, pull, digest and discovery.

## 10. Rulings this design rests on

46. **SQ-1 (2026-09-23):** the injector stays in TermLink as a primitive; local-control reading of the sidecar API.
47. **SQ-4:** urgent shortens the wait, never skips the prompt-free check (but see O2).
48. **SQ-5:** install the cron driver (done 2026-09-22).
49. **SQ-8:** solid over quick; portable respawn.
50. **T-3325 Q1 = C:** `to_circuit`; read both grammars; write path form.
51. **T-3330 IW-1 = C amended:** the telemetry as in §7.

## 11. What is actually operating today (2026-10-03), stated plainly

52. **For the agents on this host, the core of §4 does not work.** A message reaches the receiving sidecar and is stored and receipted (steps 21–23, as hub receipts rather than API calls), but **nothing injects it into a running agent's session** (step 24 onwards). The agent learns of it only if it goes and looks.
53. **Why:**
    53a. The injector needs a session started with a terminal it can type into (`tl-claude.sh start --reachable`). No agent on this host was started that way, and a running session cannot be given one afterwards (PL-237).
    53b. So the injector is scheduled by nothing (its own header, T-3207).
    53c. The wake consumers only log, and `claude-termlink` has none at all.
54. **The slice register says otherwise.** `arc-011.yaml` records S7, S10 and S11 as **built**, and S10 as "PROVEN LIVE 2026-09-22". That proof was one injection into one test session on a fresh topic (T-3079). It was never turned into the running service for real agents. Recorded "built" therefore meant "worked once", which is the exact failure this document exists to stop.
55. **Also not operating:** API calls back to the sender (RECEIVED, STORED, INJECTED and ANSWER READY are, at best, receipts on hub topics); per-step telemetry; the daily digest; sidecars shipped with deployments.
56. **What does operate:** hub transport and storage; per-message addressing (T-3325, live since 2026-10-02 23:17Z); session-start listing of unseen mail (T-3327, manual); the polling-free acknowledgement rule.
57. **Nothing in §4 may be called working until an end-to-end test passes live:** two real running agents, a message injected while the receiver is busy and while it is idle, every step confirmed back to the sender with timestamps, the answer returned, plus a negative control. This is the arc-011 closing rule, restated.

## 12. Open points and contradictions (for the operator)

58. **O1, the send path.** Today's design (step 20) is sidecar → sidecar API. SQ-1 kept sending on the hub (`channel.post`), with the sidecar API local-only, because a cross-host sidecar API would make TermLink a second bus. AEF's receiver already does sidecar → sidecar on one host. Which holds across hosts, and who carries it?
59. **O2, urgent into a busy prompt.** Today: "urgent is injected immediately". SQ-4: urgent never skips the prompt-free check, because text injected into a busy prompt can be silently lost (T-2396). Which rule, or what makes "immediately" safe?
60. **O3, RECEIVED vs STORED.** Are they two separate calls (today's design), or one (AEF's RECEIVED already means stored durably)?
61. **O4, sessions not started injectable.** How does an already-running agent session become reachable, given PL-237?
62. **O5, the session level** of the address (6d).
63. **O6, the directory** (9).
