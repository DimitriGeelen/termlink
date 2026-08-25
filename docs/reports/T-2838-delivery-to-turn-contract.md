# T-2838: Delivery-to-turn contract — build it or keep nudging (G-083)

Inception research artifact (C-001). Exploration plan only — no build artifacts.

## The question

Does TermLink build a delivery-to-turn contract — delivery acknowledged from inside the
recipient's own turn loop, plus a typed result-manifest envelope — or does interactive
agent<->agent communication stay dependent on manual nudging?

## The stated goal (operator, 2026-08-24)

TermLink as an interactive communication medium: primarily agent<->agent, also
operator<->agent; topologies 1:1, N:N, N:operator, 1:operator; plus artifact passing by
reference — an orchestrator dispatches an assignment (with an asserted agent profile) to an
agent, the agent works, writes files, and reports back *that those files were written*, so a
downstream agent consumes the conclusions by reference rather than receiving a binary blob.

Operator assessment: *"Partially certainly, but that interactive communication is still flaky
at best."*

## Finding 1 — the cause is already diagnosed in the register

`.context/project/concerns.yaml`, **G-083**, severity **high**, status **watching**,
`detection_lag_days: months`. Trigger event 2026-07-10, operator's words: *"why does
communication between the two not flow, stops, not progresses without manual nudging"*.

Mechanism per the concern: the wake is **PTY keystroke injection**; if the recipient session
is busy or in manual-accept mode the injected text lands in the input box **unsubmitted** and
is discarded on the next `claude --continue` — durably written at its offset, never read.
`scripts/listener-heartbeat.sh` is a background script **fully decoupled from the session's
actual availability**, so `LIVE + armed` is true while the session consumes nothing. The
framework checks reachable-BEFORE (T-2385) but has **no check that the recipient consumed
AFTER**.

Governing learning: **PL-253** — *"a heartbeat proves the PROCESS is alive, not that the
SESSION is listening."*

G-083's resolution field: **"Not yet built — pending operator."**

## Finding 2 — the detector layer is saturated; the mechanism was never built

Built after G-083 was diagnosed: `woken-but-silent` canary, `scripts/wake-confirm.sh`,
`scripts/woken-silent-triage.sh` (T-2416), `scripts/comms-selftest.sh` (T-2482),
`scripts/diagnose-unconsumed.sh` (T-2479), and **G-085** — a concern raised solely because the
G-083 canary could not return to green (on 2026-07-18 all five live entries re-verified as
CONSUMED: 100% false-positive residue while `/canaries` showed FIRING).

Six detection artifacts, zero delivery contract. The framework's detector reflex applied to a
problem that needs a **mechanism**.

## Finding 3 — a missing plane, not a broken bus

Measured crate sizes: `termlink-bus` 5,240 LOC with **no internal deps** (clean leaf);
`termlink-protocol` 3,093; `termlink-session` 23,317; `termlink-hub` 24,550; `termlink-mcp`
50,653; `termlink-cli` 72,207 (and cli depends on mcp — an inversion). That is 122,860 LOC of
CLI+MCP adapters over a 5,240-LOC engine, a 23:1 shell-to-engine ratio.

The engine is sound; the gap is above it. Three planes, one finished:

1. durable log — built
2. **delivery/consumption — under-designed, rings a terminal and hopes**
3. artifacts/results — embryonic

Topology is already served by topics; what is missing is the **last hop** from "durably
written" to "an agent took a turn on it".

## Finding 4 — the artifact instinct is right, the shape is wrong

`crates/termlink-cli/src/commands/file.rs:119` (T-1249) already prefers `channel.post` +
`artifact.put` — posting a *reference* and storing bytes separately. But `cmd_file_send` at
`file.rs:213` does `std::fs::read(file_path)` then base64s the whole file. And for agents
sharing a filesystem — the operator's actual workflow — **no byte transfer is needed at all**;
what is needed is a typed **result manifest** (paths, hashes, summary, status). No such
envelope exists.

`crates/termlink-hub/src/artifact.rs` is 2 handlers (~165 LOC); `termlink_file_send` /
`termlink_file_receive` are 2 of **262** MCP tools. Note the asymmetry: 262 tools, and the one
verb this workflow needs — hand an assignment to an agent and get back a manifest of what it
wrote — is not among them.

## Finding 5 — the charter has a matching gap

`docs/CHARTER.md` commits to *"exchange durable messages"*. It never says *deliver them to an
agent's attention*. The flakiness lives in that missing clause.

Corroborated by `docs/reports/T-2468-termlink-purpose-review.md` gap **P2** (HIGH):
*"Wake→consume silent break (G-083) — the core value prop (reliable comms) has a load-bearing
hole: a message can be rung but never read, nothing surfaces it."* T-2468's verdict —
*"subtract and deepen, not add"* — points here.

## Candidate shape (to be tested, not assumed)

Two delivery modes, honestly separated:

- **Native consumer** — for agents we launch: the agent blocks on the substrate and acks from
  **inside its own turn loop**; liveness is emitted by the consuming loop, not a sidecar
  script, so `LIVE != listening` becomes impossible by construction rather than detectable.
- **PTY inject** — for sessions we do not control: permitted, but **never reports success
  without a confirmed read-cursor advance**; unconfirmed fails loudly with "unread at offset X".

Plus a typed round trip: assignment envelope (profile + task set) in, result manifest out.

## S4 — Primitive audit (2026-08-25, post-GO)

Question (IW-4 / A-6): do today's receipt and read-cursor primitives support bounded
consumption confirmation without a wire-protocol change?

**Result: A-6 CONFIRMED. No wire-protocol change is needed. IW-4 disposed.**

Evidence, in the order it was gathered:

1. **`receipt` is already a first-class `msg_type` on the wire.** It appears in the META
   type list at seven sites in `crates/termlink-cli/src/commands/channel.rs` and has explicit
   `msg_type == "receipt"` discriminators at 3316, 4128, 8089, 8636. Live proof: offset 392 on
   `agent-chat-arc` is `msg_type: "receipt"`, `metadata: {up_to: "391"}`, payload `up_to=391`.

2. **The envelope already carries a by-reference slot.** `crates/termlink-bus/src/envelope.rs`
   declares `artifact_ref: Option<String>` alongside an open
   `metadata: BTreeMap<String, String>` marked `#[serde(default)]` for backward compatibility.
   A result manifest rides as an existing field plus metadata keys — no new wire type.

3. **That slot is already authenticated.** `channel.rs:944` computes
   `canonical_sign_bytes(topic, msg_type, &payload_bytes, artifact_ref, ts_unix_ms)` — the
   artifact reference is inside canonical signed bytes, not merely carried beside them. A
   manifest reference is therefore tamper-evident today.

4. **Receipts are published to the log, not held locally.** `compute_ack_status(&envelopes,
   &receipts, latest_offset)` (`channel.rs:8527`, called at 8672) derives per-sender lag from
   receipt envelopes read back off the topic. Live proof:
   `termlink channel ack-status agent-chat-arc --json` returns
   `{lag: 5, latest: 396, up_to: 391}`. A third party can observe consumption without
   privileged access.

Conclusion for IW-4: the contract is **not** sequenced behind T-2700. The wire already
expresses everything the delivery/consumption contract needs.

### Partial evidence for IW-5 — the contract is convention, not enforcement

Ack and cursor state by crate (`ack_offset|last_acked|read_cursor|ack_status`):

| crate | hits |
|---|---|
| termlink-bus | 0 |
| termlink-hub | 0 |
| termlink-protocol | 0 |
| termlink-session | 0 |
| termlink-cli | 21 |
| termlink-mcp | 28 |

The hub stores and enforces nothing about consumption; all of it lives in the two client
adapters. So "receipts already work" must not be read as "consumption is guaranteed" — it is
guaranteed only for a client that chooses to send one. That is precisely the failure mode of
the heartbeat (A-3's premise). IW-5 therefore cannot be answered "client-side" on the grounds
that client-side already exists; the existing client-side implementation is the thing that
failed.

### Unanticipated blocker found by S4: receipts are not per-agent attributable

Every one of the 60 live sessions enumerated via `list_sessions` reports the same
`identity_fingerprint: d1993c2c3ec44c94` — across five projects
(010-termlink, 050-email-archive, 0501-opencode-playground,
999-Agentic-Engineering-Framework, 001-CashWeb) and multiple hosts.

Consequence, visible in the live output above: `ack-status` on a topic with 60 potential
consumers returns **a single row**. The substrate cannot currently express the question a
delivery-to-turn contract exists to answer — "did *this* agent consume it?" — because every
agent is the same principal.

This was not in the exploration plan and it changes the build decomposition: identity binding
is a **prerequisite** of the contract, not adjacent hygiene. Two tasks already carry it —
T-1427 (`termlink whoami` identity binding on chat-arc) and T-1457 (register identity on .141
agent-1) — both currently filed as connectivity/hygiene work. They are load-bearing.

## Dialogue Log

**2026-08-24 — operator, on the goal.** Clarified the objective is an interactive communication
medium for agent<->agent and operator<->agent exchange across 1:1, N:N and N:operator
topologies, with artifact passing by reference: *"the orchestrator uses TermLink to dispatch an
assignment to an agent with an asserted agent profile ... and should come back by writing those
files and reporting back those files have been written, so another agent can then work with
those conclusions or those created artifacts instead of getting a big binary blob."*

**Course correction (agent).** Two earlier framings in the same session were wrong, recorded
because the correction is the finding: (1) first proposed an **adapter refactor** — defensible
against the charter's stated purpose, but it does not touch what the operator is feeling;
(2) then, asked whether an **epic re-architecture** was warranted, argued no, because the
dependency graph shows a clean 5,240-LOC leaf engine. Both under-weighted the real answer: the
operator's goal is one layer above the charter sentence, and the plane it needs —
delivery-to-turn — was never built. The bus is sound; the adapters are bloated; **the delivery
plane is missing.**
