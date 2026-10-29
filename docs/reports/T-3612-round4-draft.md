# T-3612 round 4 — draft purpose, objectives and value drivers (for the operator first)

**Status:** DRAFT v0.7, 2026-10-29. Nothing here is adopted. Operator ruling C (2026-10-28): the operator reads and
corrects this first; only then does it go to outside review (claude, codex, antigravity, opencode; at most 2 rounds).
Adoption is the operator's (the charter sentence is human-blessed, `docs/CHARTER.md:17-19`).

**Built from:** the operator's answers Q1–Q7 (`docs/reports/T-3612-purpose-rederivation.md`, Dialogue Log items 5–12),
the round-2 synthesis (`docs/reports/T-3612-synthesis-round2.md` §1, §4, §5), today's `.context/project/objectives.yaml`
(O-1..O-7, operator-revised 2026-10-21), `docs/CHARTER.md`, and `policy/value-drivers.yaml`.

**Two layers (Q7 ruling D′).** Sections 1–4 are about **TermLink, the product**: what a release ships
(`cargo build -p termlink`: hub, CLI, MCP server, sidecar; install script and Homebrew formula). Section 5 is about
**the TermLink development project**: this repo run as an AEF consumer, its tasks, `runme.sh`, guards and canaries,
and the operation of our own fleet.

---

## 1. Purpose

**RULED (operator, 2026-10-28: "E"). The canonical sentence (one sentence, for the charter guard):**

> **TermLink makes sure every message reaches the recipient it is meant for: it finds that recipient, wakes it in its
> terminal when it must, hands the message over, and shows where it is until an agent that can act has it, or it is
> loudly stuck.**

**Scope sentence (the paragraph after it in the charter):** It is the base layer: it records work handed off between
agents, and leaves what work to do, and who should do it, to the framework above.

Where each part comes from:

1. "reaches the recipient it is meant for": the operator, 2026-10-28: the essence is that the message reaches its
   recipient, and TermLink does more than broker: it makes sure the recipient is found, addressed and activated.
   "Meant for" means the addressed recipient or that role's current holder ("main"), never "the best one for the
   job", which would be orchestration.
2. "finds that recipient": presence and five-level addressing (hub, host, project, agent, session) with the ladder
   back, the role "main", and hub-brokered circuits (Q2, Q4, OD-10, OD-18). Routing for deliverability, never for
   policy.
3. "wakes it in its terminal when it must": the founding verb, and the part generic queues cannot replace
   (synthesis §3.3). Under OD-15 activation may also start a role that is not running, or resume a conversation that
   died mid-way, under a grant (not yet built).
4. "until an agent that can act has it": Q1, "if I have trusted delivery, but [the] agent is not responding to it,
   then it's still quite useless".
5. "or it is loudly stuck": the reviewers' point (claude, codex) that TermLink cannot force an agent to act; it can
   make every stop visible. Also T-3618: "delivered" without a reader is how mail was lost.
6. Handed-off work is not named in the sentence (operator, 2026-10-28). A work request is a message, and TermLink is
   indifferent to what a message carries; naming work there reads as orchestration. TermLink's extra role for work, a
   claim, lease and completion record, is in the scope sentence and in P-3.

**Considered and not chosen:** A (the v0.3 sentence with "or is loudly stuck"), A-short (A without handed-off work),
B (antigravity: adds waking and attribution), C (codex: a feature list that promises less), D (terminals first).
The review synthesis, section C.14, has them in full.

**Replaces:** `docs/CHARTER.md:12-15` ("hub-mediated, durable append-log message bus with terminal endpoints …")
and the `objectives.yaml` headline. Those two disagree today (synthesis §4.5). "Strict star" goes (Q4), and so does
"message bus" as the lead noun (Q1).

## 2. Product objectives (replace O-1..O-7)

Each one has a measure. "Not yet" means the measure is named, but whatever would produce it is not built.

1. **P-1 Delivered means an agent can act on it** (absorbs O-1 and O-2; Q1).
   - Statement: a message counts as accepted only once it is stored. It is done only when a live agent that can act
     on it has it, or it is loudly stuck. The sender sees each stage. Only reachable peers are listed. A reader whose
     position was swept gets a gap notice instead of skipping. Posts stay responsive while the hub's host is under
     load. Nothing is lost in silence. (v0.3: O-2's gap notice and load clause restored, all three reviewers.)
   - Measures:
     - today: the comms self-test passes for live peers; the waker-liveness, dead-letter and unconfirmed-delivery
       canaries are empty; retention-gap signals fire on truncated reads (T-3304 IW-3).
     - not yet: the sender-visible stage chain on every rail (arc-011 PH-2..PH-4); no channel.create or
       channel.post over 5 s while the disk is under 90% busy (T-3488); restart-safe post dedupe (G-088).
     - development-side evidence, not a product measure: `scripts/check-unacted-mail.sh` (T-3618) calls the
       framework's sidecar and reads this fleet's conventions, so it counts under DP-2.
2. **P-2 It finds the way to the recipient** (Q2, Q4, OD-10, OD-15, OD-18).
   - Statement: an address resolves at hub, host, project, agent or session level. A project's new mail reaches its
     one "main" agent. Conversation mail reaches the bound instance, is resumed under a grant, or comes back as a
     visible dead letter, and is never silently redirected. Hubs never relay each other's messages; a hub may broker
     a direct circuit and remains the fallback.
   - Measures:
     - today: the pinned main resolves (`crates/termlink-sidecar/src/book.rs`).
     - next: "who holds main, is it alive" plus its alarm (T-3619).
     - not yet: the R-68 standing takeover tests (T-3360) and direct circuits (OD-1 step 3).
3. **P-3 Handed-off work is done once and recorded once** (O-3, narrowed by Q2).
   - Statement: one authoritative record per unit of work, which TermLink, AEF and ring20 read the same way. Handed
     off without races, recovered when its worker dies. Not exactly-once side effects (synthesis D8).
   - Measures:
     - today: the substrate-smoke and stuck-claims canaries are empty.
     - not yet: the shared conformance vectors pass against the hub backend (worker contract).
   - Workspaces (v0.3, codex and antigravity, not contested): the per-claim isolated checkout mechanism stays in
     P-3, because T-3469 ruled clone-per-claim and `dispatch --isolate` ships. Isolation policy stays with AEF.
4. **P-4 Terminals can be reached, driven and cleaned up** (O-4).
   - Statement: start a session, inject or exec into it, read its output, and wake an idle agent at it, from
     elsewhere. Sessions nobody uses do not pile up.
   - Measures: today, the session-control and session-leak canaries are empty.
5. **P-5 Every act is attributable and authorised** (O-7, Q5).
   - Statement: every message, injection and lifecycle write traces to a verified identity and is accepted only
     from one allowed to make it. The target is agents isolated from each other within a cooperating fleet; it is
     not a boundary between hostile tenants.
   - Measures:
     - today: signatures are verified on post and on read (T-3486); a missing key fails closed (T-3467); inject is
       signed (T-3493). Not yet isolation (v0.3, all three reviewers): co-resident agents on a host still share uid 0
       and the host root key, and fleet hosts still hold .107's full hub secret while scoped tokens replace it
       (T-3615).
     - not yet: per-agent authorisation (T-2422), one OS account per agent (OQ-2 target), and the per-peer key
       allow-list replacing the shared hub secret (T-3423).
6. **P-6 What is released installs and runs** (product half of O-5).
   - Statement: tagged releases publish Linux and macOS binaries with checksums, install from the release artifact
     on a clean host, upgrade and roll back.
   - Measures: today, the release-publication canary is empty and the macOS job gates the release build.
7. **P-7 The surface is proportionate** (Q6, **open: pending fleet data**).
   - Statement: every CLI verb and MCP tool earns its maintenance and context cost through a caller or a documented
     coordination reason. Whether and how fast anything is removed follows fleet evidence. (v0.3: "removed on a
     schedule" decided Q6 before the data; claude, codex.)
   - Measures:
     - today: the off-charter tool count from `scripts/check-charter-drift-freshness.sh`.
     - not yet: Q6 is deferred until T-3615's attributed fleet telemetry has gathered data; the target, the
       thresholds and any schedule are decided then.

## 3. Product value drivers (`policy/value-drivers.yaml`)

D1–D4 are protected: same meaning, weights may change. A free driver is justified only for an axis D1–D4 do not
mean. Delivery and loud failure are what D2 Reliability already means ("predictable, observable, auditable; no
silent failures"), so they raise D2's weight instead of becoming new drivers.

| Driver | Today | Proposed | Why |
|---|---|---|---|
| D1 Antifragility | 9 | 6 (operator, 2026-10-29: B) | Three of four seats: at 9 it rewards adding canaries as product progress; that work is now scored in the development set (D1 9 there). 6 keeps product recovery (P-3 worker recovery, main takeover, conversation resume) valued below delivery, usability and trust. |
| D2 Reliability | 7 | 9 | Delivery to an agent that can act, and no silent loss, is the purpose (Q1). (v0.3: 10 cannot be set, since `fw bvp weight` accepts 0–9.) |
| D3 Usability | 5 | 7 | The operator interrupted only for decisions, and errors that say what to do (all four seats). |
| D4 Portability | 3 | 3 | Linux and macOS, standards (MCP); unchanged. |
| F-TRUST Attributable trust (new) | — | 7 | Not what D1–D4 mean: who may do what, provably (P-5, Q5). Retire when T-3423 and T-2422 are live. |
| F-SURFACE Proportionate surface (new) | — | 5 | Not what D1–D4 mean: fewer, used verbs (P-7). Retire when Q6 is decided and the cut is done. |
| F-CONV-RECALL Conversation recall (new id; product) | — | 5 (operator, 2026-10-28: B) | See below. The product set drops F-RECALL itself (v0.3: reusing the id runs AEF's hard-coded memory scorer, `estimator.py:2631`). |
| F-ORCH Orchestration leverage | 5 | drop | Q2: TermLink does not orchestrate. Hand-off mechanics are inside P-3 and D2. |

**F-CONV-RECALL in the product** (operator ruling B, 2026-10-28; the four seats had said drop it, because as written
F-RECALL scores AEF's memory layer).
- Meaning (v0.3, all three reviewers): an agent can find back what was said and decided in a conversation it picks,
  within the retention window, through the cursor, gap notices, page-end reasons and time-range reads, without silent
  skips. Never a cross-topic query. Keeping long-term records is the consuming project's job, not the hub's.
- Rubric anchors:
  - 0: nothing found back after the session ends.
  - 3: a chosen thread's decisions can be found by time range.
  - 5: work improves finding back for every conversation, not just one.
- Guardrail: inside the Q3 ruling (T-3304 IW-1 B, IW-3 B): a bounded record, time-range reads, never a query language,
  joins or cross-topic queries. Wider recall (searching all history, forever) would reopen that ruling.
- Not D1–D4: none of them means "can be found again".
- Retire when: open (open point 5).

**Scoring tooling** (v0.3, all three reviewers): F-TRUST, F-SURFACE and F-CONV-RECALL each need a declarative
`scoring:` block. Without one they score 0 and the audit WARNs.

That leaves three free drivers (F-TRUST, F-SURFACE, F-CONV-RECALL), under the cap of five. Totals stay indicative,
as in every brief.

## 4. Product non-goals (charter non-goals, reworded where a ruling requires)

1. **No federation of topics.** Hubs never relay each other's messages. Hubs may exchange signed project directory
   cards (OD-11), and may broker a direct circuit between two agents while remaining the fallback path (Q4, OD-1).
   Replaces "strict star".
2. **Not an archival database.** The hub is the authoritative record of coordination within a declared retention
   window. Long-term records belong in purpose-built stores; "forever" topics are an owned exception (Q3, T-3304
   IW-1 B). Folds in T-3315's rewording.
3. **Not a social or engagement platform.** Reactions, pins, polls and leaderboards with no coordination purpose are
   removal candidates (unchanged; acted on through P-7 once Q6 is decided).
4. **No planning or task policy.** TermLink routes for deliverability; what work runs where and when, priorities
   and placement belong to AEF and the projects (Q2).
5. **Not a boundary between hostile tenants.** Trust is a cooperating fleet in which every act is still attributable
   and authorised (Q5).
6. **Not a governance or host-operations platform** (new, Q7). The machinery that builds and runs TermLink belongs to
   the development project (section 5), not to the product.

**Where the code already contradicts these** (v0.3, from the reviewers; each needs a task, not a rewording):

1. Non-goal 1: the strict-star tripwires (`no_spoke_mesh_tripwire.rs`, `no_federation_tripwire.rs`,
   `scripts/check-strict-star.sh`) forbid what OD-1 allows. They must be reworded to guard "no message relay", not
   removed.
2. Non-goal 2: 75 forever topics without owners (T-3314, captured).
3. Non-goal 3: social tools still compiled (polls, emoji stats, top-repliers; 28 allow-listed in
   `.context/checks/charter-drift-allowlist`). Their removal waits on Q6.
4. Non-goal 4: the hub router still serves `orchestrator.route`, `bypass_status` and `bypass_invalidate`
   (`crates/termlink-hub/src/router.rs:78-80`, verified 2026-10-28).
5. Non-goal 6: AEF and ring20 chores are still in `runme.sh` (actions 11, 11b, 11c, 13, 16, 22; Q7 ruled they leave).

## 5. The development project's own set (short; Q7 D′)

These describe how we build and run TermLink, not what it is. They would live in a separate section or file (see 6.3).

1. **DP-1 The operator is interrupted only for decisions** (from O-6). One decision at a time with a recommendation;
   every operator action is one `/opt/termlink/runme.sh`; failures are heard when they happen, in one place.
2. **DP-2 Our fleet runs what we ship and reports from every host** (the fleet half of O-5). Measures: the
   fleet-binary and fleet-capability canaries are empty; the fleet telemetry view names no host missing a day
   (T-3615).
3. **DP-3 Every guard names what it protects, under a cap** (Q7). A canary or check names the P- or DP-objective it
   protects, its owner, its cost and its retirement condition; a new one retires an old one once the cap is reached.
   (v0.3: "product claim" excluded the development guards Q7 kept; claude, codex.) **Cap (operator, 2026-10-29:
   A):** frozen at today's counts, **34 canary cron jobs** and **99 `scripts/check-*.sh`**. One in, one out from
   now, and a quarterly review retires guards whose objective is met and which have not fired. Not counted: the 12
   operational cron jobs (supervisors, rollup, re-mint, heartbeat) and the fixture suites, which test code.
4. **DP-4 The framework stays current and our fixes go upstream.** Re-vendor routine, divergence baseline, and local
   framework fixes filed with AEF rather than kept (G-062).
5. **DP-5 We read back before we propose** (operator ruling B, 2026-10-28). Before a decision is put to the operator,
   earlier rulings, tasks, research and the vector store are searched, and what was found is cited. Rulings stay
   findable. (This session the agent had to be told several times; Q3–Q5 and the credential question turned out to be
   ruled already.)
6. **Drivers:** the development project scores its own work with D1–D4 at today's weights (9/7/5/3) plus F-RECALL at
   6 in its current meaning (`fw recall`, learnings synced into instruction files). That is where D1's emphasis on
   hardening the process belongs.

## 6. What adoption would change (nothing is changed by this draft)

1. **`docs/CHARTER.md`:** canonical purpose (section 1), non-goals (section 4). The charter-sentence drift guard
   extracts ONE sentence between a hard-coded anchor and the word "machines" (`scripts/check-charter-sentence-drift.sh`
   and `scripts/test-charter-sentence-drift.sh`). Its anchor, its end word, and the copies in `README.md` and
   `docs/ARCHITECTURE.md` change in the same commit, or the release gate fails closed. The scope sentence goes in the
   paragraph after the canonical sentence.
2. **`.context/project/objectives.yaml`:** headline = the purpose; objectives P-1..P-7 replace O-1..O-7; a
   `project_objectives:` list for DP-1..DP-5. Aliases: O-1, O-2 → P-1; O-3 → P-3; O-4 → P-4; O-5 → P-6 and DP-2;
   O-6 → DP-1; O-7 → P-5. Cost is small: no arc carries a `supports:` field today, and only T-3371 and T-3477 cite
   O-n ids (claude). T-3371's verification lines check the file's shape and must still pass.
3. **`policy/value-drivers.yaml` (operator, 2026-10-29: A):** the live file holds the DEVELOPMENT set (section 5.6).
   `lib/bvp.sh` reads one driver set, and most scored work is development work. The product set (section 3) goes in
   `policy/value-drivers-product.yaml` and reaches product work through each product arc's `scoped_drivers:`
   (`bvp.sh:1503`). AEF is asked for real per-layer support.
4. **Not touched:** the framework's own `.agentic-framework/policy/value-drivers.yaml` (vendored, AEF's).
5. **Tripwires:** reword the strict-star tripwires to guard "no message relay" before OD-1 circuits land.
6. **Scoring history:** product drivers get new ids (F-CONV-RECALL), so old F-RECALL scores keep their meaning.

## 7. Open points for the operator (one at a time, after you have read the draft)

1. ~~Purpose~~: ruled E (2026-10-28), section 1.
2. ~~P-3 workspace~~: settled in v0.3. It stays in TermLink, by the earlier ruling T-3469 (two reviewers, none
   against).
3. ~~D1's product weight~~: ruled B, 6 (2026-10-29), section 3.
   ~~The DP-3 canary cap~~: ruled A (2026-10-29), section 5.
4. ~~The live driver file~~: ruled A (2026-10-29), section 6.3.
5. Product F-CONV-RECALL: its retire condition.

## Version history

| Version | Date | Change |
|---|---|---|
| 0.1 | 2026-10-28 | First draft, operator ruling C (operator reads first). |
| 0.7 | 2026-10-29 | D1 product weight ruled B: 6. |
| 0.6 | 2026-10-29 | Live driver file ruled A: the development set in policy/value-drivers.yaml; the product set in its own file, applied through product arcs' scoped_drivers; AEF asked for per-layer support. |
| 0.5 | 2026-10-29 | DP-3 cap ruled A: freeze at 34 canaries / 99 check scripts, one in one out, quarterly retirement review. D1 product weight put to the operator (A 5 / B 6 / C 7 / D 9; recommended B); no ruling yet. |
| 0.4 | 2026-10-28 | Purpose ruled E: "makes sure every message reaches the recipient it is meant for: finds, wakes, hands over, shows where it is until an agent that can act has it, or it is loudly stuck". Handed-off work moved out of the sentence into the scope sentence and P-3. |
| 0.3 | 2026-10-28 | Outside review round 1 (claude, codex, antigravity; opencode failed). Folded in what all three or two uncontested agreed on (`T-3612-round4-review-synthesis.md` A and B): D2 9, not 10; product recall gets a new id F-CONV-RECALL with no cross-topic queries and long-term keeping left to consumers; scoring blocks needed; P-1 restores the gap notice and load clause; P-5 isolation moved to "not yet"; P-7 no pre-decided schedule; DP-3 covers development guards; non-goal 1 names OD-11; code contradictions listed; adoption checklist covers the charter guard, aliases, tripwires and score ids; workspace settled. Purpose, D1/cap, driver file and recall retirement stay open for the operator. |
| 0.2 | 2026-10-28 | Operator: "why is recall dropped?". It was dropped from the product set on the four seats' advice, and wrongly left out of the development project's set. Ruling B: recall in both sets (product F-RECALL 5, redefined and bounded by Q3; development project F-RECALL 6 and DP-5). |
