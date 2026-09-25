# T-3144 — Census: verification legs that assert an absence with nothing proving the search could succeed

**Task:** T-3144 (inception) · **Status:** in progress · **Opened:** 2026-09-25

## The question

One question, one number: **how many verification legs in this corpus assert an ABSENCE
with nothing establishing that the search could have succeeded?**

The shape:

```sh
! grep -q "PATTERN" path/to/file      # passes if PATTERN is absent
                                      # ALSO passes if the file is gone
                                      # ALSO passes if the file is empty
                                      # ALSO passes if the path was renamed
```

A leg like this is indistinguishable, at exit-code level, between *"the bad thing is not
there"* and *"I could not look."* Same class as T-2831 (a `## Verification` block gating on
nothing because its commands sat under the wrong heading) and T-2818 (a gate that blocks
wrongly often enough that people learn to force past it), from a third direction.

## Why this is being asked now

Not from our own observation — from a peer's. `832-Workflow-designer` posted it to
`framework:pickup` at offsets 152 and 153 (their task T-843), explicitly marked
*"NOT A BUG REPORT AGAINST YOUR CODE"* and *"Nothing here is a request"*. They measured
**122** instances in their own tree, red since 2026-09-01, the count having risen
**78 → 122** in that window.

Two independent reasons to expect a non-zero rate here:

- **T-2831** found the sibling defect in this exact corpus — commands filed under the wrong
  heading, so P-011 passed vacuously on a task claiming 8/8 ACs checked.
- **T-3142 was parked one commit before this task was filed**, because its own load-bearing
  test passed vacuously: it reported `clean — 0 references` against a fixture that named an
  absent verb. We wrote the defect we are now looking for, inside a check built to find it.

## Method

Scope: `## Verification` blocks only, across `.tasks/active/` and `.tasks/completed/`.
Completed tasks are included deliberately — they are the best evidence of how widespread the
shape is, and they are where a vacuous pass has already done its damage (the T-2818 precedent
for corpus-wide scoping).

A leg is a **candidate** when it is a negated search — `! grep`, `! rg`, or a `grep -c` /
`wc -l` count-equals-zero assertion — over a named path.

A candidate is **FIRING** when nothing in the same verification block establishes that the
path was readable: no `test -f` / `test -s` / `[ -f ]` on it, no companion positive `grep -q`
against the same file, and no producing command joined with `&&` whose exit code would carry
a read failure.

It is **CLEARED** when such a companion exists. That is the correct shape and must not be
counted, or the census measures style rather than risk.

## What this inception will NOT do

It will not build a guard-layer member. The recommendation recorded at filing was **GO = run
the census and report a number**, explicitly not *build*. The number is the whole question:
122-scale and the shape is endemic and structural; single digits and our `## Verification`
convention is already closing it, and a new member would be breadth for its own sake — the
accretion T-2483 exists to prevent. NO-GO is an acceptable outcome.

## Findings

### The number (IW-1)

```
task files scanned   2853
candidates             71     negated searches / count-equals-zero assertions
  CLEARED              41     a companion establishes the search could succeed
  FIRING               30     nothing does
```

**30**, against 832's 122. Not zero — the shape is real here — but a quarter of their rate,
and the composition matters more than the ratio.

### Distribution (IW-2)

Thinly spread, not concentrated: **23 distinct tasks, never more than 2 legs in any one.**
There is no single copy-pasted idiom carrying the count, which rules out "fix one template
line and 20 instances vanish".

Two recognisable shapes:

- **Bare file search** — `! grep -q "PATTERN" path/to/file`. The textbook case.
- **Count over a COMMAND's output** — `[ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]`.
  This one is sharper than it looks: if `cargo` is absent, or fails before emitting
  diagnostics, there are no `^error` lines, the count is 0, and the leg passes. A build gate
  that goes green precisely when the build could not run.

**The convention is already working, and that is the load-bearing finding.** 41 of 71
candidates — 58% — carry the correct companion. Spot-checked two of them by hand rather than
trusting the classifier: T-1417 pairs `! grep -q '…' docs/migrations/T-1166-….md` with
`test -f` on the same path one line below; T-2873 pairs its negated grep with a positive
`grep -q "fn build_inject_keys"` against the same `tools.rs` one line above. Both are exactly
right. Authors here are writing the guard unprompted more often than not.

### Liveness (IW-3)

**Zero of the firing legs are vacuous today.** Of 14 resolvable literal paths in the firing
set, 14 exist. Every instance is a latent trap awaiting a rename, not a gate currently
reporting green over nothing.

And the firing set is almost entirely historical: **28 of 30 sit in `.tasks/completed/`.**
Only **2** are in active tasks — both in T-1415, both searching `crates/`, the repository's
core source directory, which is not plausibly going missing.

## Recommendation

**NO-GO on a guard-layer member.** The census was worth running and the answer is that this
does not warrant a check.

The decisive argument is not the count, it is what an operator could do about it. 28 of 30
findings live in closed tasks. Those verification blocks will never run again; editing them
rewrites history for no behavioural gain. A new guard-layer member would therefore report 30
findings on day one, of which 28 are structurally un-actionable, and would need a 28-entry
allowlist before it could ever be green. That is precisely the T-2833 disease — the draft
that fired on 58 legitimate-but-unfixable instances and had to be re-scoped, because *a check
that is permanently red is a check nobody reads*. Adding it to a layer already running ~16
minutes (T-3090) would be breadth accretion of the kind T-2483 exists to prevent.

The remaining 2 live instances are one task over one directory. That is a comment, not a
subsystem.

**The cheaper remedy that reaches the actual leverage point** is the task template. It already
carries a Pipefail/SIGPIPE section teaching verification-line hygiene — that is where
`cmd > /tmp/.out 2>&1 && grep -q PAT /tmp/.out` is prescribed, and it is read at the moment a
verification block is being written. An absence-assertion rule belongs beside it:

```sh
# Asserting an ABSENCE: prove the search could have succeeded.
test -f path/to/file && ! grep -q "PATTERN" path/to/file     # existence first
grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f            # or a positive companion
cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out        # or an &&-joined producer
```

That reaches every NEW block, which is where 832's 78 → 122 growth came from — and it costs
nothing at runtime. **This is offered as the recommendation, not performed:** editing the
task template is a convention change affecting every future task, which is the operator's
call, not an agent's (Authority Model — initiative, not decision).

### Worth sending back to 832

They marked 152/153 "nothing here is a request", so this is a courtesy rather than a reply
they asked for — but a second independent measurement of the same shape, from a corpus 23×
larger with a quarter the rate, is the kind of thing their census can be calibrated against.
Specifically: our 58% already-correct rate suggests the remedy is upstream of detection.

## Dialogue Log

## Dialogue Log

### 2026-09-25 — opened

Filed out of the T-3143 triage of 8 inbound `framework:pickup` filings. 832's 152/153 were
classified "not a request — but our class": the only two filings in that batch addressed to
nobody, and still the most relevant thing in it.

Four Open Questions declared before any measurement (T-2194 gate). IW-4 is deliberately
askable in the negative — the guard layer is already ~16 minutes and "the shape exists" is
not sufficient grounds to extend it.
