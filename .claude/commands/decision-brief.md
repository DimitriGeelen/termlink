# /decision-brief — take the operator through ONE decision (T-3305)

When the operator asks for a decision to be reviewed, or types `/decision-brief
<topic>`, or a task needs a human ruling (inception go/no-go, a sovereign question,
an open `IW-N`, any `needs input:` that carries a choice), present it in the format
below. The format was shaped by the operator over SQ-8, SQ-10, SQ-11 and T-3304
IW-1..IW-3 (2026-10-01); every rule here exists because its absence cost a round trip.

**Invocation:** `/decision-brief <topic-or-task-id>`. With no argument, take the next
open decision in the current queue.

## The two standing rules (non-negotiable)

1. **One decision per message.** Present ONE question, then stop. Leave room for
   the answer and for questions about it. **Do not move to the next decision until
   the operator says "next"** (or equivalent), even if they already answered.
   Never batch ("SQ-8 1, SQ-10 a+b, ..."). Questions about the open decision are
   answered in place; the decision stays open.
2. **Always recommend.** Never present options without saying which one you would
   pick and why. A survey without a recommendation is not finished work.

## Step 1: Get the facts before writing anything

- **Measure and verify, don't recall.** Read the code, run the read-only command,
  count the records. Cite `file:line` or the command for every factual claim.
- **Look for what already exists.** Before proposing to build X, grep for X. (T-3304:
  a recommended "hub timer" already existed and was switched on; an "existing" gap
  signal turned out to be unused by the hub. Both were found only by checking.)
- **If an earlier claim of yours turns out wrong, say so first**, in a short
  "What I got wrong" list, before the new analysis. Correct the record on disk too.
- **Check the decision is decidable before inviting it (ring20 lesson 5.17, adopted
  2026-10-06).** Every open question it depends on is disposed, and the recommendation
  is current after any scope change. If not, say what is missing and keep shaping; while
  an inception is still being shaped its Recommendation stays DEFER ("shaping in
  progress"). Never point the operator at a GO button that a gate will refuse.
- **External review rounds are capped at 2–3 per step (ring20 lesson 5.16).** After the
  cap every remaining finding is accepted (with a change request), rejected (with a
  reason) or deferred (with a risk acceptance), never sent round again; final-round
  points become acceptance criteria in the build task. Subscription-seat reviewers
  (claude-code, codex, opencode) may be dispatched without asking; a paid route
  (OpenRouter) needs the operator's go each time; never put secrets in a brief (lesson 5.18).

## Step 2: Write the brief

Use these sections, in this order, in plain language:

### Background
What the thing is, how it works today, and why this decision is on the table now.
**Explain terms; don't assume them.** Define words like cursor, offset, retention,
sweep the first time, and prefer one concrete example or analogy over abstraction
(T-3304: "the hub searches" was misread as "downloading"; a librarian analogy and
"position 100 onwards" fixed it). Keep it to what the decision needs.

### The problem
One or two sentences: what goes wrong, for whom, how often, how we know.

### Options
Two to four options, lettered **A, B, C, D**, so the operator can answer with a
letter. Include "do nothing / defer" when it is a real choice. One line each on what
the option does, then any shared details (defaults, scope) once, not per option.

### Scoring
Score every option against the **joint value drivers** on **−2..+2**
(−2 clearly harms, 0 neutral, +2 clearly advances), multiply by the driver weight,
and total. Read the drivers and weights from BOTH files at brief time — never from
memory, they change:

- the project's `policy/value-drivers.yaml` (protected D1–D4 + project free drivers)
- the framework's `.agentic-framework/policy/value-drivers.yaml` (D1–D4 + framework
  free drivers)

D1–D4 appear in both with the same meaning; score them once. Label every free
driver with its source (e.g. `F-ORCH (project)`, `F-AUTONOMY (framework)`). A driver
the decision genuinely does not touch may be dropped, with one line saying which and
why. Table shape:

```
| Driver (weight) | A | B | C |
|---|---|---|---|
| D1 Antifragility (9) | −1 | +1 | +2 |
| ...                  |    |    |    |
| **Total**            | **−20** | **+24** | **+40** |
```

Say once that totals are indicative: the ordering is what matters, the magnitudes
are judgement.

### Steelman and strawman
For **every** option: the strongest honest case for it (steelman) and the weakest
version of the argument for it, shown up (strawman). The steelman of the option you
do not recommend must be one its advocate would sign.

### Recommendation
The option, the deciding reason in one or two sentences, and **what a ruling
authorises and what it leaves open** (follow-on decisions, build tasks, anything
that comes back to the operator for wording or approval).

Close with: the decision stays open for a ruling and questions; name what is next in
the queue and that it waits for "next".

## Step 3: Bias check before sending

- **Evidence vs momentum.** Is the recommendation driven by facts, or by where the
  conversation was already heading? (T-3304 IW-2: the operator caught exactly this.)
  If an earlier conversation shaped the framing, say so.
- **Outside opinions are quoted from disk, never from memory.** When reviewers or
  other agents were consulted, re-read their saved answers and represent where they
  **disagree**, not only where they agree.
- **Do the scores follow from the steelman/strawman?** If the strawman of the
  recommended option is serious, the scores should show it.

## Step 4: When the operator rules

1. **Restate what you understood before acting.** Operator messages are often voice
   transcriptions ("RecordBee" = "record B", "see you then" = "C then"). Say how you
   read it; if a detail is missing (e.g. a default value), state the assumption you
   are recording and that it can be overturned.
2. **Record it** in the owning task's `## Decisions`:
   `### <date> — <question>: <title> (operator ruling)` with **Chose / Why /
   Rejected**, plus what was left open. Mark the matching `IW-N` disposition
   `answered` with a one-line rationale. Add the exchange (questions, corrections,
   ruling) to the research artifact's `## Dialogue Log`.
3. **Commit**, or if a gate refuses (e.g. the inception two-commit limit), say so and
   say where the ruling waits.
4. **Then stop.** Wait for "next" before presenting the following decision.

## Rules

- NEVER present more than one decision per message.
- NEVER omit the recommendation or the steelman of the options you reject.
- NEVER put a fact in the brief you did not verify this session; mark anything
  unverified as such.
- NEVER treat silence, an automated notification, or your own earlier text as the
  operator's ruling.
- NEVER write an unlabelled bullet in the brief the operator reads (T-3329). Label every
  list item `1`, `2`, `3` → `1a`, `1b` → `1aa`, `1ab`, numbered uniquely across the whole
  brief (continue, do not restart per section). Options keep `A`–`D`. The operator rules
  by reference ("C, and drop 2b"), often by voice.
- NEVER cite an id the operator must know or decide by (IW-2, GP-11, §5E.2, T-3359, a gap,
  a finding number) without a one-sentence plain-language restatement next to it
  (ring20 lesson 5.15, adopted 2026-10-06), e.g. "GP-11 (the operator as a party to agent
  conversations)". Applies to chat, Watchtower criteria, runme and approval hand-offs.
