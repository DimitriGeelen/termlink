You are an independent reviewer of one proposed design principle. Work from the text below only; do not read other files and do not modify anything. Answer in English, under ~900 words, numbered headings, hierarchical labels (1, 1a), never plain bullets.

## Context

TermLink is a message rail for AI coding agents on a home lab: hubs (one per host or group of hosts) route messages; each agent has a sidecar that receives, stores and hands messages to the agent; conversations may run directly sidecar-to-sidecar on short-lived "circuits" set up through the hubs. Addressing has five identity levels: host, hub, project, session, agent (canonical name plus a runtime instance id at each level). A delivery that cannot reach the exact instance falls back up the levels (agent, then session, then project) within the same project; it never crosses into another project.

Current state of confidentiality (from the step-2 threat model): every holder of a hub token that can read a topic reads every direct message on it (all agents on a host share the hub secret; peer hosts hold it too), and the hub operator or anyone holding the hub database reads all stored content. All agents on a host currently run as root; the operator has ruled a target of one isolated OS account per agent, created by the orchestrator through a narrow privileged helper, each creation a human-approved ("Tier 0") act. Agents have per-agent signing keys whose public keys are published on signed "cards" served by their home hub; fleet admission and first-contact trust are still open. There is no operator key yet (an open question, GP-11). Already ruled: every message is signed by the sending agent's key (proposed), a stolen circuit credential is useless without the agent key, urgent mail from outside the interrupt allow-list is downgraded not dropped, the receiver's store survives a host crash and detects loss.

Three open operator questions this principle would answer together:
1. OQ-7: who may read content on the hub, token holders (today: everyone with a token).
2. OQ-10: does the hub keep a copy of message content (needed to recover a receiver store lost after STORED) or only stage metadata.
3. OQ-11: who may read content on the hub, the hub operator (today: yes).

## The proposed principle (the operator's idea, formalised by the orchestrator)

P1. Need-to-know follows the identity levels. An agent reads mail addressed to it. A project can read the mail of all its own agents (mail addressed to the project, and mail to any agent of that project). No project reads another project's mail.
P2. The hub is a mailman: it reads the envelope (sender, recipient, conversation id, sequence, priority, stage, sizes, timestamps) to route, chase, detect stuck messages and run canaries; it does not read the content.
P3. Mechanism (orchestrator's proposal): content is encrypted end-to-end to the recipient's level, to the agent's key for agent-addressed mail and to a project key held by that project's agents for project-addressed mail. The hub may keep an encrypted copy, so a lost receiver store can be recovered by the recipient without the hub being able to read it. The fallback up the identity levels never crosses a project boundary; mail that cannot be delivered within its project waits or dead-letters, it is not re-addressed to another project.
P4. Exceptions are break-glass: a recovery key held off-host by the operator, usable only through an explicit, approved, logged act. (Caveat already noted: today's "Tier 0" gate is a hook inside the agent harness and does not bind a root process; a real break-glass needs a key the hosts do not hold, which depends on GP-11.)

Costs already identified: envelopes still reveal who talks to whom and when; key management (project keys, rotation, joining, a new participant's access to history); recipient public keys come from hub-served cards, so a hostile hub at first contact can substitute a key unless the fingerprint is confirmed out of band; on today's root-shared hosts root reads local keys.

## Questions
1. Is the principle sound as a design direction for this system? Name any flaw in P1-P4, including in the analogy (a mailman reads only the envelope).
2. P1: should a project be able to read all its agents' mail, or does that undermine agent-level confidentiality in a way the operator should know? Is there a better grouping?
3. P3: is end-to-end encryption to an agent key or a project key the right mechanism here, and what is the smallest workable version (key distribution, rotation, membership change, recovery of a lost store) for a home lab? What would you do first?
4. P4: is an off-host operator recovery key the right break-glass, and what must it guarantee (approval, logging, scope, single use)?
5. What does this principle cost or break: hub functions that need content (search, digests, telemetry, canaries), debugging, and the ruled requirements described above?
6. Recommendation: adopt, adopt with changes (which), or reject, with the main reason.
