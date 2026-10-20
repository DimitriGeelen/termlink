# T-3476 — Deterministic task-assignment sidecar (inception research)

**Status:** shaping. Recommendation DEFER until the questions below have evidence.
**Origin:** operator, 2026-10-19, after the T-3469 IW-3 external review: "task assignment is
a purely deterministic process in essence: a routine, called from the main agent or the
framework, that checks what is there and assigns tasks according to availability, and can
have a queue for requests."

## 1. The idea in one paragraph

Two jobs that agents do by hand today are mechanical: (a) handing out task IDs, and
(b) deciding which free worker gets which unit of work. Neither needs judgement, so neither
should need an LLM. A small always-on process (a sidecar) does both. Agents and the
framework *ask* it ("I need a task ID", "assign me work", "here is work to hand out"); it
answers from what it can see (free workers, open units, claims), in order, from a queue.

## 2. What exists today (measured 2026-10-19)

1. **Task-ID allocation** is per checkout: `create-task.sh` takes a lock, reads the highest
   ID in its own `.tasks/` and adds one (`.agentic-framework/agents/task-create/create-task.sh`
   lines 15-24, 238-311; G-052, T-1279/T-1424). Safe inside one checkout; blind across
   branches, worktrees and clones. T-2800 (2026-08-20) measured 12 colliding IDs across 3
   branches and built only the detector; the allocation fix was filed upstream
   (PD-120, `framework:pickup` offsets 15-16) and has not landed in the vendored copy.
2. **Availability** exists on the hub: `agent find-idle` = live agents on `agent-presence`
   minus those holding a claim (T-2045/T-2109), with role/capability filters and a
   `--watch --notify` event loop (T-2078/T-2079).
3. **Exclusive hand-out** exists on the hub: `channel.claim` / `renew` / `release` /
   `transfer_claim` (T-2019..T-2046).
4. **A typed assignment message** exists: `termlink.assignment.v0` and
   `termlink.result_manifest.v0` (T-2838, `crates/termlink-session/src/assignment.rs`).
5. **A working orchestrator loop** exists as a script only:
   `scripts/orchestrator-backlog-drain.sh` + `worker-backlog-drain.sh` (T-2204..T-2206):
   read agent-eligible units from `.tasks/active/`, find-idle, claim, transfer, DM. Proven
   live once in June; not scheduled; no production caller.
6. **A per-agent sidecar** exists in AEF (`fw sidecar`, arc-011): an always-on process per
   agent identity that receives consults and replies. The operator said on 2026-09-20 (AEF
   project) that functions now living as TermLink primitives could move into that sidecar
   as API services.
7. AEF's claim-based election (T-3310, `lib/aef_election.py`) states that "bidding /
   supervisor-assign are v2 options and deliberately absent".

**Not found:** the earlier discussion the operator remembers (orchestrator must assign tasks /
an assignment sidecar). Operator messages in all local Claude sessions of the last 60 days
were searched for assignment/sidecar/queue wording; the closest records are items 4-7.

So the parts exist; what is missing is the one deterministic place that composes them, owns
the queue, and hands out IDs.

**AEF answer (999 agent via sidecar, 2026-10-19, msg a2e5989e):**

1. No task, report or decision in AEF records a deterministic assignment service; if the
   discussion happened it never reached a file. Nearest: AEF
   `docs/reports/T-1061-termlink-governance-substrate.md` (TermLink as governance/dispatch
   substrate) and `docs/reports/T-3287-identity-taxonomy-circuit-model.md` §Election policy
   (~l.122, ~l.180): first-claim, bid or supervisor-assign listed as an open choice.
2. AEF today: `lib/aef_election.py` (first-to-claim; bid/supervisor-assign not built);
   `fw termlink dispatch` (the orchestrating agent picks the task and spawns a worker, max 5,
   no queue, no availability matching); `fw resolver loop` / `fw outcome` route by task type
   to a workflow and model, not to workers. Nothing assigns by worker availability and nothing
   holds a request queue.
3. Task IDs: AEF T-1279 locks allocation inside one checkout only. Cross-checkout uniqueness
   was AEF T-1107 ("globally unique IDs vs collision defense"), decided DEFER; no central
   allocator exists. (Our August pickup landed under another AEF id; they offered to trace it.)
4. AEF suggests building on T-3287's election question and T-1107's deferred ID question,
   and names AEF dispatch as a natural first client (relevant to IW-5).

5. Follow-up (msg 349ce580): our August pickup (offsets 15-16) never became an AEF task.
   It is now AEF T-4393 (inception, horizon next): decide with T-1107's options and align
   with our T-3476 so central ID allocation is designed once; it also asks why the pickup
   did not surface. We re-sent the pickup text verbatim on the conversation.

## 2b. Evidence since filing (2026-10-19/20)

1. **ID allocation is moving upstream.** Our August pickup became AEF T-4393 (inception, horizon
   next), to be decided with AEF T-1107's options and designed once together with T-3476.
2. **Perplexity (T-3477 review):** "A central ID allocator prevents ID collisions, not redundant
   work. Assignments need explicit scope, ownership and overlap checks." So allocation (IW-3) and
   assignment (IW-4) are different problems; one service may host both, but solving one does not
   solve the other.
3. **The shared dispatched-worker contract (T-3483 GO, T-3484 in progress)** already puts part
   of this in the hub: a per-dispatch record keyed by dispatch id, with attempts and nonces
   allocated by the hub, `done`/`lost` lifecycle and a CAS on every transition (AEF conditions,
   AEF T-4435). An assignment service would consume that record ("worker N is free again") and
   create dispatches; it should not invent a second state store.
4. **ring20 (orchestration, arc-009)** runs a supervisor as the single writer of commission
   records; one-shot workers have no hub access. Where assignment lives in their picture is part
   of the T-3477 question still open with them.
5. **AEF dispatch today** picks the task in the orchestrating agent and spawns up to 5 workers,
   with no queue and no availability matching (AEF answer, 2026-10-19).

Implication for sequencing: T-3476's assignment half depends on the T-3484 transition table
(the record it reads and writes); its ID half depends on AEF T-4393. Shaping can continue, but a
recommendation should wait for both, or the design will be built on a record shape that is
still being agreed.

## 3. Questions (mirrored as IW-1..IW-5 on the task)

1. **Scope:** one service for both ID allocation and work assignment, or two?
2. **Where it runs:** inside the hub (one per host, already where claims live), as an AEF
   sidecar API, or as its own small daemon? Hubs do not federate (G-060), so a cross-host
   fleet needs one designated assigner or a partition.
3. **ID allocation:** a hub counter per project, reserved ranges per worker, or a
   claim-scoped namespace? It must reach the AEF that other projects vendor, so this
   touches AEF upstream (G-062).
4. **Policy and queue:** FIFO plus capability/availability matching; how priority (horizon,
   BVP rank) enters; durable (survives restart) and operator-visible; refuse loudly when
   no worker fits instead of waiting silently.
5. **Callers:** the orchestrator agent, `fw task create`, Watchtower, the 055 cockpit; and
   the smallest real consumer.

## 4. Relationship to T-3469

The T-3469 reviewers made central allocation a precondition for parallel checkouts; this
task would supply it. T-3469 IW-3 stays open until this and T-3477 have shaped.

## 5. Dialogue log

1. 2026-10-19, operator: switch to English. Task assignment should be a deterministic
   routine, ideally a sidecar process with a request queue, called by the main agent or the
   framework. Also explore containers (T-3477).
