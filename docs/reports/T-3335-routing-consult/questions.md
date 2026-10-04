## Questions

Answer in English, under ~1300 words, using the numbered headings below. Use hierarchical labels (1, 1a, 1ab), never plain bullets. Give your own view and disagree where you think the proposal is wrong. Do not modify any files.

1. **The split.** Hubs exchange which identities are present and whether they are alive (control plane); agents talk directly once resolved (data plane); hub relay only as a fallback; hubs never synchronize messages. Is this the right architecture for a fleet of AI agents across hosts? Your verdict and its biggest weakness.
2. **Protocol models.** Which existing protocol families should this borrow from, and what exactly from each: IP routing (RIP, OSPF, BGP), DNS, SIP registrar/proxy with ICE/STUN/TURN, gossip membership (SWIM, Serf, Consul), XMPP server-to-server, e-mail MX, or others. Scale: about 5 to 20 hubs, a few hundred agent instances, hubs mostly one hop from each other. Do you agree that IP routing is the wrong closest fit?
3. **Liveness.** States (alive / suspect / dead / unknown?), who is authoritative, how fast "dead" may be declared, and exactly how a sender uses the answer (retry, stop, dead letter).
4. **Directory contents.** What each hub advertises across the five address levels (host, hub, project, session, agent; each with a canonical id and instance ids), how it is scoped, cached and expired, and how a role address resolves to an instance.
5. **The direct path.** What "direct" should mean here (sidecar-to-sidecar network connection, or direct into the destination hub), what it requires (listening surface, authentication, NAT/firewall, durability, receipts), and when direct should NOT be preferred.
6. **The fallback ladder.** Order of attempts, what the sender is told at each step, how one message is settled when it may travel by more than one path, and deduplication.
7. **First slice and test.** The smallest build that proves it with two real agents on two hosts, including a negative control (a dead agent, and a hub outage that must not be reported as dead).
8. **What is missing:** a risk, an option or a requirement nobody stated.
