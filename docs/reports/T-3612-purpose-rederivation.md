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
4. Round 1 done: four vendor reads (claude, codex, antigravity, opencode via GitHub; the local seat could not run).
   Round 2 synthesis: `docs/reports/T-3612-synthesis-round2.md`, seven real choices Q1-Q7.
5. 2026-10-27, Q1 (headline: trusted delivery vs live terminals + agent wake), operator: "it's really basically A
   and B … if I have trusted delivery, but [the] agent is not responding to it, then it's still quite useless.
   That's really one of the pain points we have been working on. Both are really important."
   Agent reading: delivery is end-to-end, done only when a live agent that can act has the message; candidate
   sentence "TermLink gets every message to an agent that can act on it, in a real terminal, and shows where it is
   until it does." Consequences: O-2 extends from "stored" to "handed over to a live agent, answered or loudly
   stuck", absorbing O-1 and O-4 into one chain; the unbuilt "main" (OD-18, T-3359) becomes core to the purpose.
   Status: view recorded, not a ruling; the sentence is drafted in round 4.
6. 2026-10-27, Q2 (does TermLink dispatch or only provide primitives). Agent first line: TermLink owns the record
   and handoff mechanics, never "what runs where or when". Operator correction: with the five-level circuit
   addressing and the ladder back (session down -> back to the project / hub), TermLink IS instrumental in where
   a message or job can be delivered, so in that sense it decides where; but it does not do orchestration and does
   not do routing in the policy sense; it is the base layer.
   Agent restatement: TermLink routes for deliverability (resolve hub/host/project/agent/session to a live
   recipient; main and takeover), never for strategy (what work, priority, best placement: AEF/projects).
   Record check: OD-10 (rulings summary §3.3) sends new requests to project + role resolved by the home hub, but
   conversation mail to a dead bound instance gets a dead letter, "never a silent redirect"; OD-15 allows resume
   under a grant. Open question put to the operator: keep the visible fall-back for conversation mail, or make it
   drop back to the project automatically (would amend OD-10).
7. 2026-10-27, operator: "search back if we didn't already have this discussion in detail". Found: yes, ruled
   2026-10-05 (T-3344 Decisions). OD-10 C: new requests to project + role (home hub resolves); conversation mail to
   the bound exact copy, dead letter if ended, never redirected; role-only rejected because conversation context is
   lost on restart. OD-15 A refined with the operator's additions: a copy that died mid-conversation is RESUMED
   (same transcript) when the home hub says DEAD, the conversation is open, the transcript is resumable and a grant
   covers it; otherwise dead letter, sender sees DEAD, re-addressing to the role is the sender's choice; a fresh copy
   never answers conversation mail; "may start what is not running, may not start an exact instance that is gone,
   except resuming a conversation that died mid-way". The open question is answered by these rulings; no change.
