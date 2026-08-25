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

## S2 — Native consumer spike (2026-08-25, post-GO)

Question (IW-1 / A-1, A-3): can a launched agent session acknowledge delivery from inside its
own turn loop, without modifying Claude Code?

**Result: A-1 CONFIRMED, IW-1 disposed — yes, and by construction rather than by convention.**

### The live demonstration

1. An independent process (`land2-v`, a separate shell on the same hub) posted a simulated
   assignment to a fresh topic: `t2838-s2-probe`, offset 0, `metadata.kind=assignment`.
2. The consuming agent — this session — called the in-process MCP tool
   `termlink_channel_ack{topic, up_to: 0}` **from inside its own turn**. Receipt landed at
   offset 1.
3. A third, independent process observed the frontier:
   `termlink channel ack-status t2838-s2-probe` → `up_to: 0` for that consumer.

Claude Code was not modified. The delivery → consumption → sender-observable loop closes with
today's primitives.

### Why this is by construction, not convention

`crates/termlink-mcp/src/tools.rs` has **no autonomous receipt emitter**. Every ack path
(`termlink_channel_ack` at 21474, `termlink_agent_ack` at 20581) is an `async fn` on the tool
struct, reachable only by client invocation. The seven `tokio::spawn` sites are inside tool
bodies (batch/parallel helpers), not a background loop.

An MCP tool call can only be issued by the model during a turn. Combined with the absence of a
background emitter, this yields the property the inception was looking for:

> **A receipt cannot exist unless a turn happened.**

That is A-3 — liveness sourced from the consuming loop rather than a sidecar heartbeat —
obtained as an invariant rather than a detector. It is the by-construction fix the GO criteria
asked S2 to demonstrate.

**Scope limit, stated honestly.** This proves an ack *implies* a turn. It does not prove an
agent will *choose* to ack, and it does not demonstrate the "blocks on the substrate" half of
S2 — a consumer that parks waiting for delivery. Those remain open and belong to the build.

### Identity enforcement is live — and it is the real attribution gate

Attempting the ack with an explicit distinct principal:

```
termlink_channel_ack{topic, up_to: 0, sender_id: "agent-t2838-consumer"}
-> -32014: sender_id="agent-t2838-consumer" does not match identity fingerprint
   d1993c2c... derived from sender_pubkey_hex (T-1427)
```

The hub **enforces** `sender_id == fingerprint(sender_pubkey_hex)`. Two consequences:

- Sender spoofing is correctly prevented. Good.
- The S4 blocker is sharper than recorded there. Per-agent attribution is not a metadata field
  an agent can set; it requires **per-agent keypairs**. All 60 live sessions share one
  fingerprint because they share one key. T-1427's enforcement half is shipped; the
  provisioning half is not. That is a build prerequisite, not hygiene.

### Defect found: `ack-status` lag is self-referential; `unread` is correct

`compute_ack_status` (`channel.rs:8527`) computes `lag = latest_offset.saturating_sub(up_to)`
where `latest_offset` is the topic frontier **including receipt envelopes**. Since each ack is
itself an envelope, every ack raises the frontier it is chasing:

| action | latest | up_to | reported lag |
|---|---|---|---|
| ack up_to=0 | 1 | 0 | 1 |
| ack up_to=1 | 2 | 1 | 1 |

`lag: 0` is unreachable on that surface. But `channel unread` on the identical state returns
`unread_count: 0, first_unread: null` — because its walkers filter `msg_type == "receipt"`
(`channel.rs:3316, 4128, 8089, 8636`).

So consumption **is** confirmable; the two surfaces simply disagree about the same state. One
filters receipts, the other does not, and they are never compared. For the contract:

- Build bounded consumption confirmation on **`unread`**, not on `ack-status` lag.
- `ack-status` (the T-1361 human dashboard) reports a permanent phantom lag of at least 1 for a
  fully-caught-up fleet, which is exactly the "chronically behind" impression the operator has
  been reading off it.

This is the same class 999-AEF reported today as OBS-343/OBS-344 — two implementations of one
job, silently divergent because nothing compares them. Third instance this week.

*Scratch topic `t2838-s2-probe` was left on the hub as evidence; it holds three envelopes and
no real content.*

## S3 — Envelope schema draft (2026-08-25, post-GO)

Question (IW-2 / A-4): what is the minimum viable assignment envelope + result manifest, and
does it round-trip on the existing artifact path?

**Result: A-4 CONFIRMED, IW-2 disposed. It round-trips today with no new wire type.**

### The two envelope types, as drafted and actually posted

`termlink.assignment.v0` — posted with `--msg-type assignment`, offset 3:

```json
{"schema":"termlink.assignment.v0","assignment_id":"AS-0001","issued_by":"orchestrator",
 "agent_profile":{"role":"analyst","min_capabilities":["read","write"]},
 "task":"summarise S4 findings","deadline_unix_ms":0,"artifacts_in":[]}
```

`termlink.result_manifest.v0` — posted with `--msg-type result_manifest --reply-to 3`, offset 4:

```json
{"schema":"termlink.result_manifest.v0","assignment_id":"AS-0001","in_reply_to":3,
 "status":"ok","summary":"...","host":"107","repo":"/opt/termlink",
 "artifacts_out":[{"path":"...","sha256":"...","bytes":14762,"media_type":"text/markdown"},
                  {"path":"...","sha256":"...","bytes":14073,"media_type":"text/markdown"}]}
```

### What the round-trip proved

Read back by an independent process:

| property | result |
|---|---|
| free-form `msg_type` accepted | yes — `assignment`, `result_manifest`, no registration |
| correlation preserved | `metadata.in_reply_to: "3"` set by `--reply-to` |
| dialog grouping preserved | `metadata.conversation_id: "AS-0001"` |
| `artifact_ref` preserved verbatim | `ref://sha256/49307347f678df34...` |
| payload byte-exact | 584-byte JSON, unmodified |
| **every referenced artifact verified** | **2/2 VERIFIED by sha256 + size** |

`MANIFEST_RESOLVES: True`.

The manifest is 584 bytes; the content it describes is 28,835 bytes. Nothing of the file
content crossed the bus, and the manifest size is O(number of artifacts), not O(bytes). A
downstream agent sharing the filesystem consumes conclusions by reference and can verify it got
the exact bytes the producer meant — which is precisely the operator's stated goal.

### Why no new wire type is needed

`msg_type` is a free-form string (`channel post --msg-type`, default `note`). `--reply-to`
already sets `metadata.in_reply_to` (T-1313, the Matrix `m.in_reply_to` analogue).
`--artifact-ref` is documented as an "optional opaque pointer" and is carried inside canonical
signed bytes. The schema therefore lives entirely in the payload, versioned by its own
`schema` field, and old readers see an envelope they can skip by `msg_type`.

This is the charter-compliant shape: non-goal #2 says manifests reference artifacts and never
archive them. `artifacts_out[]` carries `path` + `sha256` + `bytes`, never content.
`crates/termlink-bus/src/artifact_store.rs` (`put -> sha256_hex`, `get(sha)`) remains available
for the cross-host case where the filesystem is not shared, but the common same-host case needs
none of it.

### Residual, and it is the same one S2 found

The manifest asserts `"issued_by":"orchestrator"` and the assignment asserts an
`agent_profile`. Neither is authenticated: the hub verifies only that `sender_id` matches the
key fingerprint, and every agent in this fleet shares one key. So `issued_by` is decoration
today. A profile assertion that cannot be attributed to a distinct principal is a convention,
not a contract — the same per-agent-keypair prerequisite S2 identified, arriving from the other
direction.

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
