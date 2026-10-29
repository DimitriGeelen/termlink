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
8. 2026-10-28, Q3 (how wide is the record). Searched first: ruled 2026-10-01 in T-3304 (IW-1 B: authoritative for
   coordination within a declared retention window, long-term records in purpose-built stores, forever only as an
   owned exception; IW-2 C 14-day default + ceiling on post + sweeper everywhere; IW-3 B gap signal, page end
   reasons, time-range reads, never a query language or joins; IW-4 C-prime topic records, owner, idle/unread
   flags, never auto-delete). Worker-contract dispatch records fall inside "coordination". Unfinished: T-3315
   (charter non-goal 2 rewording, operator wording, captured/later) and T-3314 (review of the 75 forever topics,
   captured). Operator: "yes", recorded as CONFIRMED by the 2026-10-01 ruling; T-3315's rewording folds into
   round 4.
9. 2026-10-28, Q4 (strict star or hub-brokered). Searched first: ruled 2026-10-05, OD-1 C (rulings summary §3.1):
   hubs set up a circuit, established conversations run sidecar-to-sidecar, the hub path is the fallback; build
   order identity + directory, then bound conversations on the hub path, then direct circuits with hub-minted
   short-lived credentials. OD-11: hubs exchange cards, never messages (federation still out). The charter's
   "strict star" sentence is stale. Built: hub path and binding; direct circuit not yet. Operator: "yes", recorded
   as CONFIRMED by OD-1; charter rewording ("hubs never relay each other's messages; a hub may broker a direct
   circuit and remains the fallback") folds into round 4.
10. 2026-10-28, Q5 (how far identity and security go). Searched first: objectives O-7 (2026-10-21: every act
   attributable and authorised); T-3351 OQ-2 (2026-10-07: shared root accepted for now, committed target one OS
   account per agent via a narrow Tier-0 helper; "we accept it for now, but we definitely want to move to a more
   isolated per-agent account … always via Tier 0"); arc-011 PH-2 OQ-b (2026-10-12: root-only accepted for PH-2,
   real separation Track A in PH-3); charter non-goal 5 (no hostile-tenant promise). Built: signed posts, T-3493
   signed inject. Open: T-2422 per-agent authorisation, Track A. Operator: "yes", recorded as CONFIRMED by O-7 and
   OQ-2; round-4 clause "trusted within a cooperating fleet: every act attributable and authorised, agents isolated
   from each other; not a boundary between hostile tenants".
11. 2026-10-28, Q6 (how hard to cut the surface). Searched first: T-2548 GO (subtract the analytics MCP family,
   2026-08-20) gated on an external-consumer check never discharged (value review C-01); 40-46 deprecated tools still
   compiled after their soak (C-02); arc-005 MCP slimming stale. Operator: no decision without data, and the data must
   be fleet-wide and cover all agents, not this host. Measured: usage is recorded per host for CLI and MCP (MCP
   logging proven live: one call appeared at once), but zero MCP tool calls over ~1 day here while sessions start
   364 MCP servers (agents may not find deferred MCP tools); ~1 day kept (size rotation, polling noise); usage rows
   carry no caller; dispatched workers write to a stray /tmp sink; the daily fleet rollup (T-3466) carries no usage and
   only .107 reports, missing 2 of 7 days. Operator then widened it: include claude -p workers, standing long-lived
   session workers, orchestration workers and non-Anthropic seats ("all workers!"), with worker quality visible.
   Outcome: Q6 DEFERRED until T-3615 (attributed fleet usage + per-worker-kind outcome telemetry, all hosts) has
   gathered data; then analysis and decisions.
12. 2026-10-28, Q7 (is governance and operations machinery part of TermLink). Searched first: no earlier ruling on
   where ops machinery lives or on a canary cap. Agent correction: an earlier one-liner had narrowed Q7 to "7 chores
   out of runme"; the synthesis question covers runme, the review queue and the canary registry. Briefed A (move all
   out) / B (cap) / C (lower D1) / D (split by owner + cap), recommending D. Operator question: "do we also need to
   differentiate between the TermLink development project, which is this session right now, and the TermLink
   application … just as AEF does it … it has a vendored instance of itself, so it does both". Agent: yes; a release
   ships only `-p termlink`; objectives.yaml and the value drivers mix both layers; the reviewers counted dev-project
   guards as product surface. Revised to D′ (AEF and ring20 chores leave runme; governance guards stay as dev-project
   tooling; cap on dev-project canaries; objectives and drivers split into product and project sets). Operator: "yes".
   Outcome: D′ RULED (T-3612 Decisions). Round 4 writes purpose, objectives and drivers for the product, plus a short
   separate set for the development project; cap number and D1 weight fold into round 4.
13. 2026-10-28, round 4 go. Agent asked "may I write the draft now?" with options A (draft, then straight to outside
   review) / B (wait for T-3615 telemetry) / C (draft, operator reads first, then outside review) / D (stop);
   recommended C. Operator first asked for help ("do we need to decide something?"), got a plain restatement, then
   ruled "ok C". Draft v0.1 written: `docs/reports/T-3612-round4-draft.md` (purpose, P-1..P-7, product drivers,
   non-goals, development-project set DP-1..DP-4, what adoption would change, four open points). Next: the operator
   reads it; outside review only after their corrections.
14. 2026-10-28, operator on draft v0.1: "why is recall dropped???". Agent: dropped from the product set because
   all four seats said F-RECALL as written scores AEF's memory layer (Claude: "belongs to the framework, not the
   substrate"; Codex; Antigravity "scored 0–2 on nearly every TermLink task"); but wrongly also left out of the
   development project's set, where reading back is exactly the discipline the operator has had to enforce. Options:
   A dev project only / B both sets / C drop. Operator: "b". Draft v0.2: product F-RECALL 5, redefined as finding
   back what was said and decided, bounded by Q3 (retention window, time-range reads, no query language); dev
   project F-RECALL 6 in its current meaning plus DP-5 "we read back before we propose". Open: whether the bounded
   product meaning is what the operator meant, and its retire condition.
15. 2026-10-28, outside review round 1 of the draft (claude, codex, antigravity; opencode failed). Synthesis:
   `docs/reports/T-3612-round4-review-synthesis.md`. Agreed points folded into v0.3. Purpose put to the operator:
   A (draft + "or is loudly stuck"), B (antigravity: + waking, attribution), C (codex: promises less), D (terminals
   first); recommended A. Operator: "why also every unit of handed-off work?". Agent: from Q2 ("where a message or job
   can be delivered") and the worker-contract record, but a work request is just a message; offered A-short.
   Operator: "the hand-off work would be part of a message then … it could sound like it's also work orchestration,
   which it is not, I think. But correct me". Agent: right; nuance: for work TermLink also keeps a claim/lease and
   completion record, which is still not orchestration (a delivery receipt for work). Operator: "the essence is that
   it ensures that a message reaches the appropriate recipient … not only brokers … instruments that recipients are
   there … found, addressed or activated". Agent: agreed; correction: "appropriate" must mean the addressed
   recipient or role holder, never "the best one for the job" (orchestration); proposed E. Operator: "E".
   Ruled sentence: "TermLink makes sure every message reaches the recipient it is meant for: it finds that
   recipient, wakes it in its terminal when it must, hands the message over, and shows where it is until an agent
   that can act has it, or it is loudly stuck." Scope sentence: "It is the base layer: it records work handed off
   between agents, and leaves what work to do, and who should do it, to the framework above."
16. 2026-10-29, after the host restart (recorded under T-3624, because T-3612 is at its exploration-commit limit).
   D1's product weight put: A 5 (draft, codex), B 6 (antigravity), C 7, D keep 9; recommended B (canary accretion is now
   scored in the development set, so product D1 need not fall to 5 and should still value recovery work such as the
   takeover tests). Operator: "next" with no ruling; recorded as OPEN.
   Canary cap put: counted 34 canary cron jobs, 99 check scripts (62 guard-layer static members); 12 operational crons
   and 157 fixture suites not counted. Options: A freeze at today's counts, one in one out, quarterly retirement review
   (codex); B antigravity's 35 / 75; C a reduction target by Q1 2027; D no number. Recommended A. Operator: "A".
