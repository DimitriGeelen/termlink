## Round 3 questions

Answer in English, under ~1200 words, numbered headings below, hierarchical labels (1, 1a, 1ab), never plain bullets. Do not modify any files. This adds to rounds 1-2; keep or revise your earlier positions explicitly.

1. **Is this a distinct problem?** Restate it in your words, and say how it relates to rounds 1-2.
2. **Which shape** (6a no coordinator, 6b routing coordinator, 6c introducing coordinator, 6d deterministic rule, or another), and why, at this scale (a handful of agents per project, 5-20 hubs, AI agents whose turns take seconds to minutes).
3. **Function or agent.** Should the coordinator role be held by an AI agent, its sidecar, or the home hub? Where does judgement belong, and where must it not be on the critical path?
4. **Failover.** Is a hub-held lease (claim with TTL, renew, transfer, claim id as fencing token) sound for "one coordinator per project"? Split brain, partitions, takeover time, and what happens to in-flight work.
5. **Addressing.** What a sender writes ("ring20-manager", a project, a role), how it resolves step by step to one instance, how agents declare their role, and what is returned when nobody holds the role.
6. **With circuits.** How the coordinator fits round 2: introduction only, then a direct circuit to the chosen instance? When must a conversation stay with one instance, and when may it move?
7. **First slice and negative tests:** including the coordinator dying mid-introduction, two agents both believing they hold the role, and a project with no live agent.
8. **What is missing.**
