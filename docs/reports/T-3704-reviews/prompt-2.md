You are an independent code reviewer. You have no tools and no other context; judge only from the bundle.

The bundle holds the built slice A of a security unit: a key card per agent at a message hub, a post-time check that an arc-011 post is signed by the key on the card of the agent it claims to be, the receiver-side name binding, the design note it implements, the earlier design-review dispositions, a mutant table, the code diff and two real-hub test files, plus an operator ruling on one stated limit.

Report:
1. A verdict line: PASS (the code implements the design and closes the forgery it targets; no defect that would let a forged post through or block legitimate traffic) or FAIL.
2. Then each finding, most severe first, as: <severity: BLOCKER | MAJOR | MINOR> — <file:area> — <what is wrong> — <a concrete input or sequence that shows it> — <the fix>.
   - BLOCKER: a forged arc-011 post passes in enforce mode, a legitimate post of the agent is refused, a non-arc-011 post is affected, or the code contradicts the operator ruling.
   - MAJOR: a design-v2 commitment (dispositions table) not implemented or not tested, a fail-open path, a crash or lock defect.
   - MINOR: everything else worth fixing.
3. Do not report the operator's stated limit (root agents can write cards) as a defect; do report anything that makes it worse than stated.
Be strict and concrete. No credit for intent. At most 12 findings.
