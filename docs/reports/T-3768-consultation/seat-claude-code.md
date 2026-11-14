**1. Pick: A, with three conditions**

- **Operator amends PD-234 in writing.** A does not follow from the ruling; it reverses the clause "any new unarmed agent still fire[s]".
- **The lapse is a calendar date** (or a machine-testable W2 marker), after which class (a) fires again. "Until W2" with no date is permanent.
- **One check first:** confirm what changed around 11-13.

Reasons: when every agent on the host trips the class, it cannot tell one agent from another. A is the only option that stops the always-on alarm (rule 3) without pretending the check still means something.

**2. Options**

- **A.**
  - Strongest case: it matches reality, costs one line, and keeps the count visible.
  - Weakness: with no armed wakers, class (b) is vacuous and (c) is already off, so the whole canary becomes inert while looking alive. It also needs approval the bundle implies it already has.
- **B.**
  - Strongest case: it sits within the ruling's letter, is git-tracked, and needs no amendment.
  - Weakness: an allowlist containing everyone is A with more toil. Thirteen identical boilerplate reasons add nothing, and every new agent is a guaranteed false alarm, which trains reflexive allowlisting.
- **C.**
  - Strongest case: the alarm is literally true, since no agent on this host has a ring path until W2. It is also the only option needing no approval.
  - Weakness: it violates rule 3 outright, and the 24-hour auto-refile produces task spam.
- **D.**
  - Strongest case: it is the only option that measures whether delivery works (rule 2).
  - Weakness: it is a build that doesn't fix today's red, it overlaps two existing checks, and it strains the one-in-one-out cap.

**3. Risks and what would change my mind**

- **Unexplained transition.** The ruling expected two unarmed agents; now there are 13, and firing began about 11-13. If the sidecar heartbeat simply omits `pty_session` while wakers still run, this is a signal regression, so fix the signal rather than mute it. If wakers were removed deliberately, ask who approved it under rule 1.
- **Hidden wake gap.** A makes the lack of any doorbell quiet. If idle agents missing mail until their next prompt is not an accepted cost, I'd keep a visible non-paging status rather than silence.
- **If any agent is still meant to be armed,** I'd switch to B scoped to the truly unarmed ones.
- **If W2 is months out or undated,** the hollow canary matters more. I'd then do a narrow D: extend notify-sidecar to live-but-undeclared agents, since it only checks *declared* ones. That is a modification, not a new guard.
- **If the operator declines to amend,** B is the fallback.

**4. Framing problems**

- A is described as "citing the ruling", but the ruling says the opposite for new unarmed agents. That understates the approval A needs.
- The ruling is itself labelled "B" and describes an allowlist, which nudges toward option B.
- "None uses push-wake" is asserted, not evidenced. Zero dead wakers is consistent with clean disarm or with wakers still alive.
- The bundle doesn't say why 11 agents changed state, whether that was approved, or when W2 is due.
- It treats this as a noise problem. The underlying question is whether the host's current no-doorbell state is accepted.
- Nobody asks whether the canary should be retired at W2 instead of patched. That question is the real "one out" under rule 4.
