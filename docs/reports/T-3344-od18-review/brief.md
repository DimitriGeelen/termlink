# External review: several agents per project — who answers, who coordinates (T-3344, OD-18)

You are an external reviewer. Be critical: find where this recommendation is wrong, unsafe, or missing something.
Answer in under 900 words.

## Context

arc-011 delivers messages between AI coding agents (e.g. Claude Code sessions) through TermLink hubs. A project
often runs several agents/sessions at once. The operator asked (2026-10-04): "If I say, hey, go look for Ring20
manager, which is a project — which session ID, which agent ID, or even which project ID do you start looking for?"
and offered two options: (1) everything routes through one central agent (risk: bottleneck), (2) a coordinator
function held by at least one central agent per project; if it becomes unavailable, another agent takes over.

Earlier reviewers (round 3) merged these: the central agent becomes a deterministic role resolver at the project's
home hub; "another takes over" becomes lease failover; "introduce, then step aside"; no AI agent on the critical
path; a claim id is not a fencing token, use a generation/epoch; a home-hub outage means "authority unknown, retry",
never takeover by another hub. The motivating failure: one project (ring20-manager) left 8 requests unanswered.

Already ruled by the operator (relevant):
- OD-10: new requests address project + role, resolved by the home hub; conversation mail goes to the bound exact
  instance; never redirected; a dead instance's conversation mail is dead-lettered.
- OD-11 / CAND-8: each project has a stable id and a declared home hub (canonical hub id, verified); identity cards
  at the home hub; the same id seen in two places is flagged, never guessed.
- OD-14: the last escalation rung lands in "the main agent's" session-start attention list.
- OD-15: an agent may be started or resumed by mail only under an operator grant; a fresh copy never answers
  conversation mail; an instance that died mid-conversation is resumed from its own transcript under conditions.
- CAND-6: per-agent reachability on the card (receiver up, right hub, adapter present, last real hand-over time).
- CAND-14: each agent sees its own list of owed answers.
- The hub already has claims with leases (claim, renew, release, atomic transfer to another owner); leases lapse if
  not renewed; there is no generation/epoch today.

## The recommendation under review (option A)

1. A project has named singleton roles, at least "main" (the coordinator OD-14 points to); more may be added.
2. Each role is held through a lease at the project's home hub, with a generation number. The holder's sidecar
   (code, not an AI turn) renews it. If renewals stop, the lease lapses and another ELIGIBLE agent of the project
   takes the role with a new generation. Eligibility excludes workers, reviewer seats and sub-agents.
3. Introduce, then step aside: a message to project + role is answered by the home hub with the exact holder and
   its generation; the conversation then binds to that instance (OD-10); the hub carries no traffic.
4. Fencing: the generation is stamped on every introduction and checked wherever accepted; a stale holder's
   actions are refused.
5. Home hub down = "authority unknown, retry", never takeover by another hub.
6. Obligations on failover: owed answers addressed to the ROLE move to the new holder; owed answers in a
   CONVERSATION stay with the instance and follow OD-15 (resume or dead-letter).
7. Pools of interchangeable workers come later, as first-to-claim among workers.

Rejected: B a central coordinator agent carrying all traffic; C deliver to all candidates, first claim wins
(observers pick different winners); D sender must name the exact instance.

Open: lease duration/renewal interval; how an eligible agent is chosen when several qualify; pools.

## Questions

Q1. Is a sidecar-renewed lease the right liveness signal for holding a role? (The sidecar can be alive while the
    agent behind it is stuck, busy for hours, or deaf — a recent real failure: a sidecar with a fresh heartbeat
    refused every delivery confirmation 32,205 times.) What should the lease actually prove?
Q2. Failover while the old holder is merely busy (long task) rather than dead: how do you avoid flapping and avoid
    two agents both believing they hold "main"? Is generation fencing enough, and where must it be enforced?
Q3. Moving role-addressed obligations on failover: safe? What about work the old holder had half-done?
Q4. Selection when several agents qualify: propose a rule (and whether the operator should be able to pin one).
Q5. Anything missing: e.g. the operator's own role as a party, a project with zero eligible agents (ties to OD-15
    grants), a project spanning hosts, role addressing for projects on other hubs, human-visible "who holds main".
Q6. Anything you would remove as unnecessary weight?

Format: numbered findings, each tagged [AGREE], [CHANGE] or [MISSING], with one-line reasoning; then a one-line
overall verdict.
