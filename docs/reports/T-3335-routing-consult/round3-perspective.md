## Round 3 — which of a project's agents answers? (operator, 2026-10-04)

The operator (voice, lightly edited): "Often we've got a number of agents running for the same project. If I say, hey, go look for Ring20 manager, which is a project — which session ID, which agent ID, or even which project ID do you start looking for? So there I would say we also need the process. I'm not sure if we would say everything has a route to the central agent, because that's also adding a bottleneck. I'm not ruling it out. But the other option would be a federation function that at least one central agent has. And if that central agent falls away, or is not available anymore, another agent then takes over that role."

Operator's playback, as confirmed in the conversation:
1. Several agents (sessions, instances) often run for one project at the same time.
2. A human-style name ("ring20-manager") does not say which project id, session or agent instance to deliver to. A resolution process is needed.
3. Option one: everything for a project routes through one central agent. Risk: a bottleneck. Not ruled out.
4. Option two: a coordinator ("federation") function held by at least one central agent per project; if it becomes unavailable, another agent takes over the role.

Agent reflection (for you to challenge):
5. This is a third question beside rounds 1-2. Round 1 asked where an identity lives and whether it is alive; round 2 asked how two resolved agents converse. This asks **who answers for a project** when the sender names a project or a role, not an instance.
6. Four shapes:
   6a. **No coordinator.** The directory returns every live instance with its declared role; the sender picks by rule. Simple, no single point of failure, but every sender must implement the choice, and two senders may pick differently.
   6b. **A coordinator that routes everything.** A front desk per project receives all role-addressed mail and passes it on. A real bottleneck if it also carries the conversation.
   6c. **A coordinator that only introduces.** It answers "who handles this", then steps aside; the conversation runs on a circuit directly with the chosen instance. This is round 2's signalling-vs-conversation split applied inside a project, and it removes most of the bottleneck.
   6d. **No election: a deterministic rule** (for example the oldest live instance, or an explicit "primary" flag) that every party evaluates the same way from the directory. Failover is re-evaluating the rule.
7. **The coordinator should probably be a function, not an AI agent's attention.** An AI agent may be in a minutes-long turn; if routing waits for it, every message to the project waits. The role could be held by an agent's sidecar (deterministic code), or by the home hub, with the AI agent consulted only when the choice needs judgement.
8. **Failover can reuse an existing primitive.** The hub already offers leased claims with expiry, renewal, cooperative transfer and a claim id (channel claim / renew / claim-transfer, T-2019/T-2046). A "coordinator for project P" lease at P's home hub gives one holder at a time, automatic takeover when the holder stops renewing, and a fencing token against two coordinators after a partition. No Raft or gossip election is needed at this scale.
9. **Agents need to declare their function.** Presence already carries capabilities metadata; a declared role ("manager", "dashboard", "reviewer") makes "ring20-manager" resolvable as project + role, matching AEF's decision 2a ("function names route, instance ids ride inside").
10. **Exact-instance addressing must stay possible and fail loudly** when that instance is gone (rounds 1-2), so a conversation already bound to an instance is never silently moved to another.
11. Observed today: ring20-manager's inbox holds 8 unanswered requests from a peer. A project-level "who answers" function would also be the place that notices an unanswered inbox.
