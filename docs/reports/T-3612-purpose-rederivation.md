# T-3612 — Re-deriving TermLink's purpose, intent and direction

Status: round 1 dispatched (2026-10-27). Inception, recommendation DEFER (shaping in progress). voi_score 0.9 (operator).

## Why

TermLink's purpose is written down in three places that no longer agree:

1. `docs/CHARTER.md`: "a hub-mediated, durable append-log message bus with terminal endpoints". Its canonical
   sentence was never blessed by the operator (T-2470, Human AC open).
2. `.context/project/objectives.yaml`, revised by the operator on 2026-10-21: "the trusted coordination substrate
   for a fleet of AI agents and their operator … dispatch and hand back work", with objectives O-1..O-7.
3. `policy/value-drivers.yaml`: D1-D4 plus F-RECALL and F-ORCH, framework-generic; none names a TermLink outcome.

The 2026-10-27 value-review run (T-3609) could not confirm its yardstick because of this.

## Method (operator, 2026-10-27)

1. **Round 1, independent reads.** Five external seats, one per vendor, each read the bleeding-edge repo (GitHub
   `DimitriGeelen/termlink` main = OneDev origin = local HEAD, commit named in the brief): code, docs, design
   documents, arcs and the last two months of task discussion. Each derives, without seeing the others or our
   own conclusions: what TermLink is, what it is for, what it is becoming, its users, objectives, value drivers,
   non-goals and tensions. Brief: `.context/working/t3612/brief.md`.
   Seats: claude-code (Anthropic), codex (OpenAI), opencode (Z.ai GLM), antigravity (Google), local-gpu (ollama).
2. **Round 2, synthesis.** Commonality and divergence across the five; which divergences are real choices.
3. **Round 3, dialogue.** The synthesis goes to the operator for discussion and exploration (Dialogue Log below).
4. **Round 4, sharpening.** One purpose statement with objectives and value drivers, reviewed again externally.
   Adoption is the operator's decision (sovereign: charter sentence, objectives, driver weights).

## Round 1 results

(pending)

## Round 2 synthesis

(pending)

## Dialogue Log

1. 2026-10-27, operator: "I thought you're hinting a little bit that maybe our value drivers are not correct. We
   even could challenge if our purpose and objectives are correct. If we look at the discussions we had for arc and
   future designs last two months." Agent finding: the round-1 yardstick of T-3609 had used the charter and value
   drivers only and missed objectives.yaml (2026-10-21) entirely; the charter sentence is unblessed (T-2470).
2. 2026-10-27, operator: ask five external agents to read the bleeding-edge GitHub repo and the last two months
   of arc and task discussions and design documents, derive purpose and intent and what TermLink is evolving
   into; bring them together for commonality and divergence; then a dialogue round with the operator; then a
   review round to sharpen it into one purpose and objectives with value drivers. Before that: make sure the
   bleeding edge is pushed. Done: 323 files of uncommitted ledger and memory found and committed (T-3611).
3. 2026-10-27, operator: voi_score 0.9; go ahead with the five reviewers.
