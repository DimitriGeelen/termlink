# T-2879 — Rail enrolment gap: how many local Claude sessions are actually reachable?

**Task:** T-2879 · **Type:** inception (exploration → go/no-go) · **Owner:** agent
**Measured:** 2026-09-25 on `dimitrimintdev` / hub `.107`.

## The question

TermLink's first charter verb is **discover**. arc-011's notify rail is code-complete for
ten of twelve slices, and its recorded blocker is not a code defect but a *fleet
condition*, quoted from arc-011's S10 note:

> THE INJECTOR HAS NO AUDIENCE YET — which is a fleet condition, not a code defect, and
> it also means S12 (role swap) rests on an assumption nothing currently satisfies.

T-2879 was captured from that: **1 of 14 local Claude sessions was on the presence rail.**
The inception asks whether that is still true, what mechanism causes it, and whether
closing it is a build or a decision.

## Findings

### F1 — The premise is still true, and the denominator was flattering (IW-1)

| measure | value |
|---|---|
| local Claude session records (`claude agents --json`) | **84** |
| of those, *reachable* (carry a `status` field — the T-2876 predicate) | **9** |
| states across all records | 76 `blocked`, 4 `busy`, 3 `idle`, 1 `working` |
| entries on the TermLink presence rail (`agent-listeners.sh --json`) | **1** (`penelope`) |
| of those, carrying `pty_session` (i.e. actually wakeable) | **0** |

So the ratio is not 1-of-14; against the population that could meaningfully enrol it is
**1 of 9**, and the single enrolled agent is **not wakeable**. The rail is not merely
sparse — by the T-2387 taxonomy it is in class (a) *LIVE-but-unwakeable* **and** class (c)
*rail-dark* simultaneously.

### F2 — This is NOT a detection gap. The framework is loud and correct.

This was the hypothesis worth killing, because it is the defect class this repo keeps
finding. It does not apply here. Run on demand:

```
waker-liveness canary: FIRING — 1 unwakeable LIVE agent(s), 0 dead waker(s), RAIL DARK
  [LIVE-no-waker] penelope  fp=d1993c2c3ec44c94  age=41s — presence-advertised but
      nothing can ring it (no pty_session on heartbeat).
  [rail-dark] ZERO LIVE listeners carry pty_session on this hub — the G-069 '0 wakers'
      state. Every DM sent here waits on the ~15s poll floor at best, forever at worst.
```

and the dashboard agrees rather than contradicting it — `/canaries` reports
`waker-liveness-canary FIRING`, log 36,053 bytes, in a summary of `total 32, firing 2,
erroring 0`. Detection and presentation are consistent. **The T-2387 canary is doing
exactly the job it was built for.**

### F3 — The real finding: 42 firings, correctly reported, unattended

`.context/working/.waker-liveness-canary.log` contains **42 `rail-dark` occurrences**
across 377 lines. On a daily cron that is roughly six weeks of the rail being dark,
announced every single day, with no action taken.

That inverts the usual shape. The recurring finding in this repo is *the framework was
blind*. Here the framework was loud and nobody listened — which is the **alarm-fatigue**
failure (T-2818, T-2833) seen from the receiving end. A canary that fires correctly for
six weeks without producing an action is, operationally, indistinguishable from one that
does not fire at all; the difference is only that the evidence exists afterwards.

### F4 — The mechanism is launch-time, so no agent-callable fix exists (IW-2)

PL-237 is explicit: **a running headless claude cannot be retrofitted onto the rail** —
it must be armed *at launch*. Enrolment is therefore a property of how a session is
spawned, not a verb an already-running agent can call on itself. The canary prescribes
the remediation itself:

```
bash scripts/tl-claude.sh start --reachable --agent-id <id> -- --resume
```

This closes the chain completely:

- arc-011 S10/S12 are blocked on "no audience";
- the audience gap **is** the rail-dark condition;
- the remediation is relaunching agents through the T-2388 launcher;
- **that is already ticketed as T-2389** (`started-work`, arc-007), and T-2389 is
  **`owner: human`**.

### F5 — Low enrolment is partly correct behaviour (IW-3)

CLAUDE.md states opting in is optional: *"Skip on throw-away sessions (<2 min) or hosts
that should not appear on the fleet."* 76 of the 84 records are `blocked` — finished
background sessions at rest. Those *should not* be on the rail, and counting them in the
denominator overstates the gap. The defensible statement is narrower and still damning:
**of the agents that are live enough to be worth reaching, none can be rung.**

## Recommendation — NO-GO on a build

There is nothing to build. The gap is not a missing capability, a missing check, or a
missing verb: every piece exists, is proven, and is already alarming. What is missing is
one operator action that is already ticketed (T-2389) and already signalled 42 times.

Filing a new build task here would add a fourth artefact describing a condition three
existing artefacts already describe, and would not move the rail one agent closer to
being wakeable. That is motion, not progress.

**What should happen instead:**

1. **T-2389 is the live item**, not T-2879. It is `owner: human` and cannot be executed
   from this session — relaunching agents through the T-2388 launcher is an operator act,
   and PL-237 means it cannot be done to sessions already running.
2. **Close T-2879 as dissolved-into-T-2389** rather than carrying it as independent work.
3. **SQ (surfaced, not resolved):** should `--reachable` become the *default* for
   `tl-claude.sh`, so enrolment stops depending on remembering a flag? That changes
   fleet-wide default behaviour and puts sessions on a shared rail by default, which has a
   footprint and privacy dimension. It is an operator policy call, not an implementation
   detail, and this run does not make it.

## Open Question dispositions

| # | question | disposition | rationale |
|---|---|---|---|
| IW-1 | Is the premise still true? | **answered** | Yes, and understated: 1 of 9 reachable, and that 1 is unwakeable (F1). |
| IW-2 | What is the mechanism? | **answered** | Launch-time only; PL-237 — a running session cannot be retrofitted (F4). |
| IW-3 | Is it a defect or the designed state? | **answered** | Both. 76/84 are at-rest and correctly absent; the defect is that 0 live agents are wakeable (F5). |
| IW-4 | Does closing it need a sovereign decision? | **deferred** | Yes — defaulting `--reachable` is an operator policy call, surfaced above, deliberately not made. |

## Dialogue Log

- **Autonomous run, 2026-09-25.** Selected as Q1 (BVP 70 / cost 3.6) and as the condition
  blocking arc-011's last two slices.
- **Hypothesis carried in and killed:** "the framework is blind to this". It is not —
  F2 disproves it. Recording the disproof matters more than the original hypothesis,
  because the reflex in this repo is to reach for a detection gap and there was none.
- **Consequence:** the recommendation inverted from "build a rail-enrolment check" to
  "there is nothing to build; one ticketed operator action is outstanding".
