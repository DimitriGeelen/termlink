You are an independent design reviewer. You have no tools and no other context; judge only from the bundle.

The bundle describes how one agent pair (the "W1 pair") could report where each prompt came from (human or
automated) to a main-session lease, so that a tightening already built can be switched on for that project. Today
nothing in that pair's sessions can report it. The agent framed four options (A-D), scored them and recommends C
with A as fallback. Do not anchor on the agent's recommendation or scores; test them.

Your answer MUST start with this front matter, exactly these keys, in this order, filled in (no other keys, no
prose before it). Use the given value sets only.

---
reviewer: <your model or product name>
task: T-3694
question: W1 pair prompt-origin route
recommended_option: <A | B | C | D | NEW-1 | NEW-2 | ...>
agrees_with_agent: <yes | partly | no>
confidence: <high | medium | low>
scores: {A: <int>, B: <int>, C: <int>, D: <int>}
top_risk: "<one sentence: the single most dangerous way your recommended option goes wrong>"
blocking_unknowns: ["<fact that must be checked before building, one per item>"]
---

Then these sections, in this order, with these headings:

## 1. Scoring
Your own score per option on the same drivers and weights the bundle uses (D1 9, D2 7, D3 5, D4 3, F-AUTONOMY 4),
-2..+2 each, as a table with totals. The totals in the front matter must equal this table. One line where you differ
from the agent's scores and why.

## 2. Steelman and strawman
For EACH option A-D: the strongest honest case for it (one its advocate would sign), and the weakest version of
the case for it, shown up. Two to four sentences each.

## 3. Why (your recommended option)
The deciding reasons, concretely.

## 4. Why not (each other option)
For each rejected option: its strongest real weakness as a concrete sequence that goes wrong, severity-tagged
high/medium/low.

## 5. Alternatives the agent missed
Options not in A-D (number them NEW-1, NEW-2, ...). For each: what it is, what it costs, who must build it, and
whether it lets the criterion "main status shows origin and last_human_at on the W1 pair" be met.

## 6. Enhancements and suggestions
Changes that would make your recommended option safer or cheaper: tests that pin it (including a negative
control), rollout order, what to verify in the field, and anything in the bundle you believe is factually wrong or
inconsistent (for example whether C is really distinct from B, the effect of an unbound PTY-counter source, two
claimants per prompt, or a session that is headless).

Be concrete and brief. No credit for intent.
