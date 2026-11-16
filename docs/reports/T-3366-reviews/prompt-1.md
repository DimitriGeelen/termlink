You are an independent close reviewer. You have no tools and no other context. Judge only from the bundle.

The task T-3366 claims it is done: a hub's canonical id is minted once and persisted, the hub has a signing key that
sidecars pin, existing hubs were migrated without renaming any inbox, and the unit's evidence and mutants pass. One
criterion (forwarding `sidecar:` mail) was moved to another task by an operator ruling quoted in the bundle.

Give, first, one verdict line exactly in this form:
T-3366: PASS | FAIL | UNCLEAR — one sentence why

- PASS: the quoted evidence shows every remaining acceptance criterion is met now, and you found no defect that
  breaks a fail-closed rule of AL-17 / AL-18 as the bundle states them.
- FAIL: the evidence contradicts a criterion, something is still undone, or the code breaks a stated rule.
- UNCLEAR: the evidence does not settle it (say what is missing).

Then list, numbered, at most five concrete defects or gaps you found in the code or the evidence (file and the
line or function, what goes wrong, under which input). Judge in particular: the spec readings logged under
Evolution (are they defensible?), the runme migration order (could any hub restart onto the new binary without
`hub.id`?), and whether any claim in the evidence map is not backed by the quoted tests.

Be strict. Do not give credit for intent, or for evidence the bundle does not contain.
