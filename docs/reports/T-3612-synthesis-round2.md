# T-3612 round 2: synthesis of four independent reads of TermLink's purpose

Synthesiser: `t3612-r2-synthesis`. Commit read: 50dece865. Inputs: `docs/reports/T-3612-seats/round1-{claude,codex,antigravity,opencode}.md` (Anthropic, OpenAI, Google, Z.ai GLM), compared with `docs/CHARTER.md`, `.context/project/objectives.yaml`, `policy/value-drivers.yaml`. I checked disputed facts against the repo (marked CHECKED). I added no new research and no opinions of my own; the draft in section 5 uses only what the seats agree on.

## 1. Common ground

**1a. Identity: all four.** TermLink is a hub-mediated message bus with real terminal endpoints. It is used mainly as the coordination layer for one operator's fleet of AI agents, mostly Claude Code on the ring20 hosts, and it is built under the Agentic Engineering Framework (AEF). Claude: "nervous system of one operator's agent fleet". Codex: "operator-run coordination runtime". Antigravity: "bespoke, single-operator fleet runtime". GLM: "both the product and the substrate its own agent-driven development runs on". None found third-party users (Claude, Codex).

**1b. Purpose centre: all four.** *Delivery you can trust* is the product.
- Claude: "a message accepted is stored".
- Codex: "recoverable communication".
- Antigravity: "durable and transcript-proven messages".
- GLM: "Delivery truth is the product".

All four also say failures must be loud rather than silent.

**1c. Direction, last two months: all four.**
1. Message delivery was rebuilt as a staged chain (sent, received, stored, handed over, replied) with a new sidecar crate (arc-011).
2. The hub is becoming the authoritative record of coordination within a retention window (T-3304, arc-012).
3. A shared dispatched-worker contract with AEF and ring20 is being built (T-3484, T-3501..T-3504).
4. Identity and attribution are rising (O-7).
5. Governance and operator-experience work is large.

**1d. Objectives: all four.** Every seat proposes some version of these, and every one maps onto existing O-1..O-7:
1. Reachability that tells the truth (O-1).
2. No silent loss, with sender-visible stages (O-2).
3. Dispatched work recorded once (O-3).
4. Terminal control that works and does not leak (O-4).
5. What is shipped is what runs (O-5).
6. Operator interrupted only for decisions (O-6).
7. Attribution and authorisation (O-7).

Three seats add a surface or footprint measure (Claude objective 8, Antigravity objective 6, GLM objective 7). Codex does not, though its driver table adds "bounded operating cost".

**1e. Value drivers: all four on these.**
- Drop F-RECALL. All four say it is an AEF concern.
- Delivery integrity is the top driver (weight 10, 30 of 100, 10, 9).
- Operator attention or burden is a driver (Claude 7, Antigravity 7, GLM 7, Codex 20 of 100).
- A new surface or cost driver is needed (Claude "Simplicity" 6, Codex "Bounded operating cost" 15, Antigravity "Surface & Footprint" 6, GLM "Substrate leanness" 5).
- Portability is kept (Claude 3, GLM 3, Antigravity 5, Codex 10 of 100).

**1f. Non-goals: all four.**
- Keep the five charter non-goals in some form, with edits.
- Social features: Claude, Antigravity and GLM say they contradict non-goal 3 and should go. Codex says "no social-feature growth".
- No archival database or unbounded store. A bounded record is compatible with this (Codex, Antigravity). Claude and GLM expect non-goal 2 to be reworded.
- Not a policy or planning engine. AEF decides what work matters (all four).
- No hostile-tenant security promise (Codex, Antigravity, GLM). Claude says to widen the scope or cap the work.

## 2. Divergence

**D1. Is "hub as the system of record" the intended direction? (a) REAL CHOICE, but a narrow one.**
- *Claude:* it is the direction. Rewrite non-goal 2 as "record of coordination for a bounded window" rather than drift. It also lists "Not a record store, once the contradiction is resolved" as a proposed non-goal.
- *Codex, Antigravity, GLM:* the same wording fix. Antigravity gives retention numbers (14-day default, 30/180 days for dispatch tombstones). GLM: "the non-goal is being edited to match the direction".
- All four agree on the direction. The open choice is how wide the record is: coordination and dispatch state only (all four implicitly), or also the "forever" topics that exist today. Codex cites 75 of 111 topics retained forever. Claude cites a forever-topic review. This is the only real choice inside D1.

**D2. Is TermLink dispatching work, or only providing primitives? (a) REAL CHOICE. The largest.**
- *For dispatch as a core job:* Antigravity (TL-3 weight 8, "Orchestration & Concurrency Substrate"; objective 3 "Fenced, Single-Record Worker Dispatch"). GLM keeps "dispatch and hand back work" in the purpose and makes F-ORCH temporary. Codex's purpose includes "accountable work handoffs" and its objective 2 covers ownership and completion.
- *Against:* Claude says dispatch is "orchestration by another name" and proposes the non-goal "Not a work-dispatch or orchestration engine". It would keep only claim, lease and record primitives, and move the worker contract and supervisor to AEF.
- *Evidence:* Claude cites T-3484, T-3501..04 and T-3469/3477. Antigravity cites `crates/termlink-bus/src/worker_contract.rs`, `dispatch_contract.rs` and the shared `wc/0.3` tables. Codex says the backend exists but the CLI/MCP wiring (T-3504) is unfinished.
- *Related factual point (b), CHECKED:* the value review (`docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md`, row C-36) found "108 calls/34.2 d = 0.012%, all demo/prover" for the claim verb. Antigravity reads this as claim being built ahead of use. The same file records that run 5 gave verdict KEEP, with low volume being "what a work-stealing primitive should look like", and that both runs agree on "fix the guarantee". So the usage number is real, and what it means was never settled. Claude, who builds objective 3 on claims, did not cite it.

**D3. Is the star topology still true? (a) REAL CHOICE, with a (b) component.**
- *Codex and Antigravity:* the charter's "a strict star: spokes never talk peer-to-peer" (`docs/CHARTER.md:25`) conflicts with the closed reliable-comms arc ("direct host-to-host transport with hub fallback") and with arc-011 hub-minted sidecar circuits (rulings OD-1, OD-11). Antigravity wants non-goal 1 reworded. Codex says the topology claim needs reconciling and notes the no-spoke-mesh test is source-scoped only.
- *Claude and GLM:* do not raise it.
- CHECKED: `.context/arcs/reliable-comms.yaml` does say "direct host-to-host" (lines 7, 13, 22). So Codex and Antigravity are right that the documents disagree. Whether to keep "strict star" as principle or change it to "hub-brokered" is the operator's choice.

**D4. How much trust and security belongs in the purpose? (a) REAL CHOICE.**
- *Antigravity:* add a driver "Security & Least-Privilege Isolation" at weight 8. Its objective 5 also covers per-project OS accounts and "stop running all co-resident project agents as shared uid 0".
- *Claude:* a weight of 7 for "Attributability and authorisation", but flags the tension: non-goal 5 disclaims multi-tenant guarantees while O-7 builds per-agent authorisation. "Either widen the stated scope or cap the work."
- *GLM:* weight 6, and adds "not a product for third-party teams".
- *Codex:* folds it into "justified trust" and keeps non-goal 5 as it is.
- The positions run from "cap it" to "it is a pillar".

**D5. How hard to cut the existing surface? (a) REAL CHOICE on posture, with a (b) fact that is settled.**
- (b) CHECKED, Claude and Antigravity are right: reaction, poll, emoji-stats and typing tools are still compiled in `crates/termlink-mcp/src/tools.rs` (e.g. `channel_emoji_stats`, `channel_poll_*`, `agent_typers`, `agent_reactions_of`). The MCP file has 273 `tool(` attributes. Charter non-goal 3 calls such surfaces "removal candidates".
- (a) Posture: Antigravity says execute the deletions "immediately" and calls the 49-crontab, 266-script estate "an ad-hoc host-ops monorepo". Claude wants a "surface proportionality" objective that falls over time. GLM says prune "or explicitly justify as coordination-serving". Codex only says no growth. The disagreement is how much to remove and how fast.

**D6. Does the guard and canary layer count as product or as cost? (a) REAL CHOICE.**
- *Claude:* about 99 `check-*` scripts, "none retired"; "the detection layer risks outweighing the product". Wants D1 Antifragility lowered to 5.
- *Antigravity:* D1 9 to 6, because it "over-rewarded adding a new bash canary".
- *Codex:* D1 redefined as demonstrated recovery, 25 of 100, and cites two basic defects that the guard layer missed.
- *GLM:* keeps Antifragility at 7 and calls governance meta-work "fine as investment, but it should be a visible, capped budget".
- Everyone sees a problem. The remedy runs from "lower the weight" to "keep but cap".
- (b) CHECKED counts: `scripts/check-*.sh` = 98 (Claude ~99, matches). `.context/cron/` has 49 entries (Antigravity "49 crontabs"; `*.crontab` files alone: 45). GLM's "~11 daily canaries" is the outlier and is low.

**D7. Where should governance and operator machinery live? (a) REAL CHOICE.**
- Claude: runme, review queue and the canary registry "belong in AEF or an ops repo". Antigravity: governance policy belongs upstream, host administration in ring20.
- GLM and Codex do not propose moving it. GLM calls the overhead real and says to bound it.
- This is a repo-boundary question rather than a purpose question, but it changes what "TermLink" means.

**D8. What does exactly-once mean? (b) FACTUAL, settled by reading the claims.**
- Claude and GLM use "exactly once" in objective 3. Codex says "Do not equate this with exactly-once external side effects". Antigravity notes `README.md:59-64` advertises "exactly-once post" while the dedupe state is RAM-only and lost on restart (G-088).
- Codex and Antigravity are right to demand precision. GLM also lists closing G-088 as a measure under objective 1. All four agree that the README claim is ahead of the guarantee. The objective should say "one terminal record per attempt".

**D9. Fact disputes about the repo itself. (b)**
- Code size: Claude ~218K LOC, Antigravity 196,899 LOC (Rust only). Different bases, no real conflict.
- Where effort went: Claude says "about 80% of file churn is outside `crates/`". Antigravity gives touch counts (`.agentic-framework` 3,239, `.tasks` 3,048 versus 259 in sidecar). CHECKED: of 2,088 commits since 2026-08-27, 174 touch `crates/` (about 8%). All agree on direction. GLM and Codex give a weaker measure.
- Whether the sidecar is the biggest crate. Claude says the sidecar crate "has 259 commits in the window, the most of any crate". Antigravity's table gives 259 file-touches, not commits. Same number, different unit. It is largest by activity, not by size (Claude: 14K LOC; Antigravity: 8,271 LOC; the LOC figures also differ).

## 3. What they all missed or under-weighted

Only items I can cite.
1. **Nobody reconciles the value-review verdicts with the charter's own language.** Claude notes "DELETE zero" in arc-009, and Antigravity cites findings C-01, C-02, C-36. Neither follows through to ask who decides whether the removal candidates in non-goal 3 ever get removed. The value-review file itself marks the claim-verb status "human/sovereign" (C-36, last column).
2. **The charter's canonical sentence is human-blessed, and no seat addresses the process.** `docs/CHARTER.md:17-19` says the operator owns the wording. Antigravity notes `T-2470:101` is unticked. Any new purpose sentence therefore needs the operator's approval; the seats propose rewrites without saying so.
3. **Verb 4 (terminal control), the founding verb, got little attention.** Claude says it is "the founding verb" with "little new work"; the others fold it into one objective. Nobody asks whether the purpose should still lead with it. Only Claude shows it is the part generic tools cannot replace ("The irreplaceable part is the PTY endpoint combined with agent wake").
4. **CLAUDE.md size as a cost.** GLM says it is 101KB. CHECKED: `CLAUDE.md` is 101,808 bytes. Only GLM links it to the context-tax objective. Claude notes that `fw upgrade` can clobber it.
5. **Few seats weigh whether the record in D1 and D2 needs its own measures.** Codex alone insists on separate measures for payload recovery and sender-visible stages.

## 4. Contradictions in our own documents (found by two or more seats)

1. **Non-goal 2 vs the hub-as-record work.** `docs/CHARTER.md:40` ("Not a durable database or system of record") vs arc-012 / T-3304 and `worker_contract.rs`. Found by all four. The rewording task T-3315 is still `status: captured` (CHECKED, `.tasks/active/T-3315-…md`); Antigravity says it is parked.
2. **Non-goal 3 vs code.** `docs/CHARTER.md:43-45` and `objectives.yaml` out_of_scope vs compiled social-style tools. Found by Claude, Antigravity, GLM. CHECKED, true.
3. **Non-goal 4 vs dispatch work.** `docs/CHARTER.md:46-48` vs O-3 (`objectives.yaml`, "dispatched unit of work is done once and recorded once") and the arc-011 injector and role leases. Found by Claude and GLM. Antigravity and Codex treat it as fine so long as the line is kept. GLM notes SQ-1 ("primitive, not engine") resolved it for the injector.
4. **Star topology vs direct delivery.** `docs/CHARTER.md:25`, `docs/architecture/parallel-execution-substrate.md` vs `.context/arcs/reliable-comms.yaml:7,13,22` and arc-011 OD-1/OD-11. Found by Codex and Antigravity. CHECKED, true.
5. **Two purpose statements.** `docs/CHARTER.md:12-15` ("durable append-log message bus with terminal endpoints") vs `objectives.yaml` headline ("trusted coordination substrate … dispatch and hand back work"). Found by Claude and Antigravity. Neither is the same as `README.md`'s four verbs. The drift canary checks only the charter, README and ARCHITECTURE copies (Claude).
6. **Value drivers do not describe this product.** `policy/value-drivers.yaml` cites `lib/bvp.sh` and `web/blueprints/` (AEF), has F-RECALL, and has nothing on delivery. Found by all four.
7. **Trust model vs O-7.** `docs/CHARTER.md:49-52` and README "treat every authenticated peer as trusted" vs per-agent authorisation in O-7. Found by Claude and GLM (Antigravity addresses the same gap through its non-goal 6).
8. **README versus code.** `README.md:123,137,145` says "Four crates" and "30 commands" (CHECKED, still there); there are 9 crates (CHECKED, `ls crates`). The tool count in README text ("270+" per Claude, "276" per GLM) vs 273 in code. Found by Claude, Antigravity, GLM.
9. **Stale metadata.** `.context/arcs/arc-011.yaml:1-2` carries `id: arc-010, slug: arc-011` (CHECKED). Found by Antigravity only, listed here because it is verified. Codex notes that the objectives text calls retention gaps and supervisor credentials "not yet" built while code implements them (objectives O-2, O-7 measures).

## 5. DRAFT FOR DISCUSSION — composite, not a recommendation to adopt

Built only from section 1. Every REAL CHOICE is left open and marked `[Qn]`, with the section 6 question it plugs into.

**Purpose (one sentence).** TermLink lets a fleet of AI agents and their operator find each other, exchange messages they can trust, hand off work, and control each other's terminals, and it makes every failure visible where and when it happens. `[Q1: lead verb]` `[Q2: dispatch]` `[Q3: record width]` `[Q4: topology wording]`

**Objectives with measures** (from 1d; measures taken from the seats' overlap and existing O-1..O-7).
1. **Truthful reachability.** Only live, wakeable peers are listed. Measure: `scripts/comms-selftest.sh --discover-only` passes; `agent find-idle` shows no stale ghosts.
2. **No silent loss.** Every accepted message is stored or fails loudly, and the sender sees its stage. Measure: dead-letter and unconfirmed-delivery logs empty; stage chain visible for posts; no create/post over 5 s under the T-3488 criterion.
3. **Work recorded once.** One terminal record per attempt, recovered when a worker dies. Not "exactly-once side effects" (D8). Measure: shared conformance vectors pass against the hub backend. `[Q2]` decides whether this is a core objective or a primitive-only one.
4. **Terminal control that does not leak.** Spawn, exec, read, clean up. Measure: `session-selftest.sh` daily; session-leak canary empty.
5. **Shipped equals live.** Measure: release-publication, fleet-binary and fleet-capability canaries empty.
6. **Operator interrupted only for decisions.** Measure: `/canaries` all HEALTHY; operator-relayed messages per week, which is not yet measured (O-6 says so).
7. **Attributable writes.** Every message, injection and lifecycle write traces to a verified identity. Measure: signature verification on read; signed-inject allowlist when built. `[Q5: how far to go]`
8. **Proportionate surface.** Measure: MCP tools with zero calls over a stated period; MCP description size against the arc-005 target. `[Q6: how hard to cut]`

**Value drivers (weights 0-10; where the seats' weights spread, I show the range, not an average).**

| Driver | Weight | Disposition | Seats |
|---|---|---|---|
| Delivery integrity (absorbs D2 Reliability) | 9-10 | Change D2 | all four |
| Loud failure / observability | 8 (GLM); inside delivery for others | New or merged | Claude, GLM, Codex |
| Operator attention (was D3 Usability) | 7 | Change D3 | all four |
| Attributable trust | 6-8 `[Q5]` | New | Claude, GLM, Antigravity; Codex merges |
| Surface and footprint | 5-6 | New | all four |
| Antifragility (D1) | 5-7 `[Q7]` | Keep, lower | Claude 5, Antigravity 6, GLM 7; Codex re-defines |
| Portability (D4) | 3-5 | Keep; reword away from "source-controlled memory" | all four |
| F-RECALL | 0 | Drop | all four |
| F-ORCH | open `[Q2]` | Antigravity: change, 8. GLM: temporary. Codex: replace. Claude: tripwire only | split |
| F-AUTONOMY | inactive | Keep inactive | Codex; others silent |

**Non-goals.** Keep all five charter non-goals, with these edits that the seats agree on, and mark the rest open.
1. Non-goal 1 (federation): keep. Wording on hub-minted circuits and directory cards `[Q4]`.
2. Non-goal 2: rewrite as "not an archival database; the hub is the bounded record of coordination". `[Q3]` for what is in the record.
3. Non-goal 3 (social): keep, and act on it `[Q6]`.
4. Non-goal 4: keep as "no planning or task policy". Where dispatch sits `[Q2]`.
5. Non-goal 5: keep. Whether to cap or widen `[Q5]`.
6. Added by Claude, GLM, Antigravity in some form: not a governance or host-operations platform `[Q7]`; not a product for third-party teams (GLM).

## 6. Questions for the operator (most purpose-changing first)

Per the standing instruction, put these to the operator one at a time.

**Q1. Which job is the headline: delivery, or terminal control?** Today the charter leads with the bus, and the README lists four equal verbs. Verb 4 was the founding verb.
- A: lead with delivery, terminals as endpoints (Claude, Codex, Antigravity, GLM all draft this way).
- B: lead with live terminals and agent wake, since that is what generic queues cannot replace (Claude's own argument in §2 of its read, though its purpose sentence still leads with reach and delivery).
- No seat actually proposes B in its sentence. I list it because Claude's evidence supports it.

**Q2. Does TermLink dispatch work, or only provide the primitives?** (D2)
- A: dispatch and the worker contract are core (Antigravity; GLM; Codex in part).
- B: only claim, lease and the record are core; the worker contract and supervisor go to AEF (Claude).
- C: keep it for now and set a stop point. GLM treats F-ORCH as temporary until the contract lands.
- This also decides the F-ORCH driver and objective 3.

**Q3. How wide is the authoritative record?** (D1)
- A: coordination and dispatch state inside a bounded window only (Codex, Antigravity).
- B: A, plus keep "forever" topics with named owners (Claude and GLM describe the arc as doing this; Codex's 75 of 111 figure shows they exist).
- The seats agree the charter wording must change either way.

**Q4. Is the hub a strict star, or hub-brokered?** (D3)
- A: keep "strict star" and treat direct host-to-host and sidecar circuits as exceptions the hub arranges (implied by Codex).
- B: reword non-goal 1 and the charter to say hubs never auto-relay topic messages, but may mint credentials for direct circuits (Antigravity).
- Claude and GLM do not take a position.

**Q5. How far does identity and security go?** (D4)
- A: cap it. Cooperating fleet, attribution only (Claude's tension, GLM's "not a product for third-party teams", Codex).
- B: make it a pillar, including per-project OS accounts and the end of shared root sessions (Antigravity).

**Q6. How hard to cut the existing surface, and how fast?** (D5, D6)
- A: delete social and zero-caller tools now (Antigravity).
- B: add a shrinking-surface objective and prune on a schedule (Claude).
- C: prune or justify each (GLM).
- D: freeze growth only (Codex).

**Q7. Is governance and operations machinery part of TermLink?** (D6, D7)
- A: move runme, review queue and the canary registry out, leaving the product repo (Claude, Antigravity).
- B: keep but bound it as a visible, capped budget (GLM).
- C: keep, and lower D1's weight so it stops rewarding new canaries (Codex, Antigravity on the weight).

*Seat positions I could not place: Codex has no view on Q1, Q5's wider scope or Q7's move option. GLM has no view on Q4 or Q1.*
