# T-3612 round 4 — draft purpose, objectives and value drivers (for the operator first)

**Status:** DRAFT v0.1, 2026-10-28. Nothing here is adopted. Operator ruling C (2026-10-28): the operator reads and
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

**Proposed (one sentence, plus a scope sentence):**

> **TermLink gets every message, and every piece of handed-off work, to an agent that can act on it, in a real
> terminal, on one machine or many, and shows where it is until it gets there.**
> It is the base layer: it finds the recipient and the way to reach it, and leaves what work to do, and who should
> do it, to the framework above.

Where each part comes from:

1. "to an agent that can act on it": Q1, "if I have trusted delivery, but [the] agent is not responding to it, then
   it's still quite useless". Delivered means a live agent that can act has it, not that a hub stored it.
2. "every piece of handed-off work": Q2. The dispatch record and hand-off mechanics are TermLink's. What runs where
   and when is not.
3. "in a real terminal": the founding verb (control terminal sessions) and the part generic queues cannot replace
   (synthesis §3.3).
4. "shows where it is until it gets there": OD-4/OD-5 (received, stored, handed over, answered, on the hub record),
   and today's lesson (T-3618): "delivered" without a reader is how mail was lost.
5. "finds the recipient and the way to reach it": Q2 correction. Five-level addressing (hub, host, project, agent,
   session) with the ladder back, the role "main", and hub-brokered circuits (Q4). This is routing for
   deliverability, never routing for policy.

**Alternative if the operator wants terminals first (Q1 option B):** "TermLink gives a fleet of AI agents and their
operator live terminals they can reach, wake and hand messages and work to, on one machine or many, and shows where
every message is until an agent acts on it."

**Replaces:** `docs/CHARTER.md:12-15` ("hub-mediated, durable append-log message bus with terminal endpoints …")
and the `objectives.yaml` headline. Those two disagree today (synthesis §4.5). "Strict star" goes (Q4), and so does
"message bus" as the lead noun (Q1).

## 2. Product objectives (replace O-1..O-7)

Each one has a measure. "Not yet" means the measure is named, but whatever would produce it is not built.

1. **P-1 Delivered means an agent can act on it** (absorbs O-1 and O-2; Q1).
   - Statement: a message counts as accepted only once it is stored. It is done only when a live agent that can act
     on it has it, or it is loudly stuck. The sender sees each stage. Only reachable peers are listed. Nothing is
     lost in silence.
   - Measures:
     - today: the comms self-test passes for live peers; the waker-liveness, dead-letter and unconfirmed-delivery
       canaries are empty; `scripts/check-unacted-mail.sh` reports none (T-3618).
     - not yet: the sender-visible stage chain on every rail (arc-011 PH-2..PH-4).
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
   - **Open for the operator:** O-3's "every worker gets its own cheap, isolated workspace" (T-3469, clone per claim)
     reads like execution environment, not delivery. Keep it in P-3, or move it to AEF?
4. **P-4 Terminals can be reached, driven and cleaned up** (O-4).
   - Statement: start a session, inject or exec into it, read its output, and wake an idle agent at it, from
     elsewhere. Sessions nobody uses do not pile up.
   - Measures: today, the session-control and session-leak canaries are empty.
5. **P-5 Every act is attributable and authorised** (O-7, Q5).
   - Statement: every message, injection and lifecycle write traces to a verified identity and is accepted only
     from one allowed to make it. Agents are isolated from each other within a cooperating fleet; this is not a
     boundary between hostile tenants.
   - Measures:
     - today: signatures are verified on post and on read; there is a per-agent key.
     - not yet: per-agent authorisation (T-2422), one OS account per agent (OQ-2 target), and the per-peer key
       allow-list replacing the shared hub secret (T-3423).
6. **P-6 What is released installs and runs** (product half of O-5).
   - Statement: tagged releases publish Linux and macOS binaries with checksums, install from the release artifact
     on a clean host, upgrade and roll back.
   - Measures: today, the release-publication canary is empty and the macOS job gates the release build.
7. **P-7 The surface is proportionate** (Q6, **open: pending fleet data**).
   - Statement: every CLI verb and MCP tool has a caller or a reason. What has neither is removed on a schedule.
   - Measures: not yet. Q6 is deferred until T-3615's attributed fleet telemetry has gathered data; the target and
     the cut rate are decided then.

## 3. Product value drivers (`policy/value-drivers.yaml`)

D1–D4 are protected: same meaning, weights may change. A free driver is justified only for an axis D1–D4 do not
mean. Delivery and loud failure are what D2 Reliability already means ("predictable, observable, auditable; no
silent failures"), so they raise D2's weight instead of becoming new drivers.

| Driver | Today | Proposed | Why |
|---|---|---|---|
| D1 Antifragility | 9 | 5 | Three of four seats: it rewards adding canaries as product progress. Q7 left the number to round 4. |
| D2 Reliability | 7 | 10 | Delivery to an agent that can act, and no silent loss, is the purpose (Q1). |
| D3 Usability | 5 | 7 | The operator interrupted only for decisions, and errors that say what to do (all four seats). |
| D4 Portability | 3 | 3 | Linux and macOS, standards (MCP); unchanged. |
| F-TRUST Attributable trust (new) | — | 7 | Not what D1–D4 mean: who may do what, provably (P-5, Q5). Retire when T-3423 and T-2422 are live. |
| F-SURFACE Proportionate surface (new) | — | 5 | Not what D1–D4 mean: fewer, used verbs (P-7). Retire when Q6 is decided and the cut is done. |
| F-RECALL Recall leverage | 6 | drop | Describes AEF's memory layer, not this product (all four seats). |
| F-ORCH Orchestration leverage | 5 | drop | Q2: TermLink does not orchestrate. Hand-off mechanics are inside P-3 and D2. |

That leaves two free drivers, under the cap of five. Totals stay indicative, as in every brief.

## 4. Product non-goals (charter non-goals, reworded where a ruling requires)

1. **No federation of topics.** Hubs never relay each other's messages. A hub may broker a direct circuit between two
   agents and remains the fallback path (Q4, OD-1, OD-11). Replaces "strict star".
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

## 5. The development project's own set (short; Q7 D′)

These describe how we build and run TermLink, not what it is. They would live in a separate section or file (see 6.3).

1. **DP-1 The operator is interrupted only for decisions** (from O-6). One decision at a time with a recommendation;
   every operator action is one `/opt/termlink/runme.sh`; failures are heard when they happen, in one place.
2. **DP-2 Our fleet runs what we ship and reports from every host** (the fleet half of O-5). Measures: the
   fleet-binary and fleet-capability canaries are empty; the fleet telemetry view names no host missing a day
   (T-3615).
3. **DP-3 Every guard proves a product claim, under a cap** (Q7). A canary or check names the P-objective it
   protects; a new one retires an old one once the cap is reached. **Open: the cap's number.**
4. **DP-4 The framework stays current and our fixes go upstream.** Re-vendor routine, divergence baseline, and local
   framework fixes filed with AEF rather than kept (G-062).
5. **Drivers:** the development project scores its own work with D1–D4 at today's weights (9/7/5/3). That is where
   D1's emphasis on hardening the process belongs.

## 6. What adoption would change (nothing is changed by this draft)

1. **`docs/CHARTER.md`:** canonical purpose (section 1), non-goals (section 4). The charter-sentence drift canary and
   its copies in `README.md` and `docs/ARCHITECTURE.md` follow in the same change.
2. **`.context/project/objectives.yaml`:** headline = the purpose; objectives P-1..P-7 replace O-1..O-7, with a
   mapping so arcs' `supports: [O-n]` stay resolvable; a `project_objectives:` list for DP-1..DP-4.
3. **`policy/value-drivers.yaml`:** product weights and drivers as in section 3. **Open, tooling:** `lib/bvp.sh`
   reads one driver set. Either the development project's set lives in a second file the tooling does not yet read,
   or we ask AEF for a per-layer driver set. I recommend asking AEF, since every dogfooding consumer has the same
   two layers.
4. **Not touched:** the framework's own `.agentic-framework/policy/value-drivers.yaml` (vendored, AEF's).

## 7. Open points for the operator (one at a time, after you have read the draft)

1. Purpose: the main sentence or the terminals-first alternative (Q1 A or B), or your own wording.
2. P-3: does the per-worker workspace stay in TermLink, or move to AEF?
3. D1's new weight (5 proposed), and the DP-3 canary cap's number.
4. The per-layer driver tooling (6.3): a second file now, or ask AEF first.
