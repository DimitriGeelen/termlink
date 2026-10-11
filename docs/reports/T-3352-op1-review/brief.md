You are an independent reviewer of ONE operator decision. Read only; modify nothing. Answer in English, under ~900 words, numbered headings, hierarchical labels (1, 1a), never plain bullets. Give your own recommendation; you are not asked to agree with anyone.

## Context
TermLink is a message rail for AI coding agents on a home lab (hubs route messages, each agent has a sidecar). Today every agent on a host runs as `root` (about 86 process lines on the main host). The operator ruled (step 2, OQ-2): the shared root user is accepted for now; the committed target is one isolated operating-system account per agent, created by the orchestrator through a narrow privileged helper, each creation a human-approved, logged Tier 0 act (every creation, one event each). Host root remains a permanent residual.

The design (docs/design/interactive-agent-communication-03-floor-and-phasing.md, step 3 v0.2.1) phases the work: PH-0 preparation/measurement, PH-1 basic version (disposable estate), PH-2 first operational release (one host, one project, hub path, real agents), PH-3 identity, directory and admission (multi-hub fleet; Tier 0 event machinery live), PH-4 authority, roles and many-to-many (grants, allow-lists, eligibility, membership: configurable authority held in policy records "an agent cannot write"), PH-5 direct sidecar-to-sidecar circuits, PH-6 completion. Read section 10.1 (what accounts buy, price table, stages A0-A4, options) and section 3.7 (base strength) of that document; the requirements are in docs/design/interactive-agent-communication-01-requirements.md (R-82).

Stages: A0 pilot (inventory, one non-critical agent under its own account; measures the real price; a price gate follows where the operator confirms with the measured price), A1 template/helper/installed code/launcher, A2 migrate in waves with one-command return, A3 all agents migrated, A4 orchestrator, runme.sh, cron and hub restarts without general root.

## The question (OP-1): when does each agent get its own account, and when is the price decided?
A. Accounts first: A3 before the first operational release (PH-2).
B. Pilot early (A0 in PH-3, once the Tier 0 route for account creation exists) with a price gate; A3 before PH-4 exits; A4 in PH-6. PH-2 and PH-3 ship at base strength with a stated risk acceptance (RA-1).
C. Accounts after PH-5, as the last security step.
D. The pilot only inside this arc; the migration becomes its own arc, decided after the pilot.

## Please answer
1. Which option would you choose, and why? Is there a better option not listed (e.g. a different placement, or splitting differently)?
2. What does the recommended option risk in practice, concretely, for a home lab of AI agents running as root (prompt injection, a misbehaving agent, a compromised peer)? Is shipping PH-2/PH-3 at base strength defensible?
3. Are the price estimates and the stage cut plausible? What is most likely to be underestimated (e.g. the harness needing a login per account, the operator's attach workflow, shared git checkout permissions)?
4. Anything in the plan that is unsafe or would make the migration fail (helper becoming general root, half-migrated state, rollback)?
5. One-line verdict: the option you recommend and the single strongest reason.
