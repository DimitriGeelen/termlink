# External review: a measurable meaning for "very simple" (T-3344, gap GP-12)

You are an external reviewer. Be critical: find where these proxies measure the wrong thing, can be gamed, or miss
what actually makes a receive path unreliable. Answer in under 800 words.

## Context

arc-011 delivers messages between AI coding agents through TermLink hubs. Requirement R-6 (P1), operator wording:
"Each agent MUST have one sidecar: a separate, very simple process with an API." ("a separate process exposing an
API … Very simple.") Rationale: carrying delivery must not depend on any LLM turn. R-6's acceptance currently tests
only one-sidecar-per-agent; "very simple" has no measure.

Today's receive chain (measured 2026-10-06): 7 shell scripts, 2,472 lines (sidecar 620, injector 469, wake
consumer 323, journal mirror 248, sidecar API 319, two supervisors 327 + 166); about 3,900 lines with launchers;
runs only from the source checkout. Under an operator ruling (OD-9), where the AEF framework runs, AEF's own Python
receiver is the sidecar; a TermLink implementation follows the same contract. So the measure must apply to any
implementation.

Real failures this arc traces to complexity rather than size:
1. A sidecar's signing key was stored under another agent's name; it refused 32,205 confirmations with a fresh
   heartbeat.
2. Two receivers on one inbox (now forbidden).
3. One agent deliberately excluded from the waker because of a receipt-identity defect.
4. Receiving mail takes several cooperating processes per agent (sidecar, injector, waker, mirror, supervisors).
5. A second hub silently split sessions from inboxes (wrong runtime directory).

Related operator rulings: the receiver reports per-agent reachability (receiver up, right hub, adapter present,
last real hand-over time); each project declares its mail hub by canonical id; verification uses tiered tests with
negative controls and the real failures as standing controls.

## The proposal under review

Five measurable proxies for "very simple":
1. One process per agent, plus one supervisor per host (counted).
2. One install command that works outside the source checkout, with no required settings beyond the declared mail hub.
3. One status call answers "can this agent receive, and if not, why" on one screen.
4. A small API: about 10 calls at most.
5. Size as a tripwire, not a target: the sidecar's own code capped (proposed 1,000 lines, about 40 % of today), a
   static check flags growth past the cap for review.

Rejected: a line-count budget alone; operator judgement only; deferring to the phasing step.

## Questions

Q1. Do these proxies measure what made the receive path fail? Which failure above would still pass all five?
Q2. Which proxy can be gamed (e.g. one process that hides five threads, a status call that reports green while
    deaf), and how would you close that?
Q3. Is "one process per agent" right, or is a per-host receiver serving several agents simpler and safer?
Q4. Are line counts and API-call counts meaningful across a shell and a Python implementation? Better measures?
Q5. What is missing: e.g. number of states, number of persistent files/stores, number of identities/keys per agent,
    recovery steps after a crash, time for an operator to diagnose "why didn't my message arrive".
Q6. Are the numbers (10 calls, 1,000 lines) defensible, or should they be derived? How?

Format: numbered findings, each tagged [AGREE], [CHANGE] or [MISSING], one-line reasoning each; then a one-line
overall verdict.
