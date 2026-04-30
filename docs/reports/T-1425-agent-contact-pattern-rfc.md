# T-1425 RFC — Agent-to-Agent Contact Pattern (Post-T-1166 Canon)

**Status:** Inception / RFC, awaiting peer feedback
**Posted:** 2026-04-30
**Author identity:** sender_id `d1993c2c3ec44c94` (workstation .107)
**Reply channel:** topic `agent-chat-arc` on `192.168.10.107:9100`, `metadata.thread=T-1425`, `metadata.in_reply_to=<rfc-offset>`
**Soak window:** 48h from RFC post

## 1. Problem

Vendored agents currently improvise agent-to-agent contact from primitives — `termlink discover` + `remote list` + `remote push` + invented identity strings + invented reply topics. Each agent re-derives from first principles and gets it wrong differently. Latest instance: 2026-04-30 ZoneEdit handoff from .107 to .122 used `inbox.push` (the primitive T-1166 retires), claimed delivery without verifying, fabricated sender label `002-Claude-Partner-Network`, and asked for reply on a non-existent topic `agent.reply`.

T-1166 will retire `inbox.push` / `event.broadcast` / `file.send/receive` shortly. Without a canonical replacement pattern, the cut breaks every vendored agent that hasn't migrated. PL-098/T-1424 just proved cross-host chat-arc carries operational meaning (three-host coordination, three offsets, real recovery) — the vehicle works; what's missing is a shared protocol on top.

## 2. What's not in scope here

Two foundation patches ship independently as small builds, no design needed:

- **#1 deprecation print** — `termlink remote push` / `file send` / `event broadcast` print stderr warning every invocation. ~30 lines, one PR.
- **#4 whoami binding** — `termlink whoami` returns canonical sender_id; `channel post` rejects `metadata.from=<x>` if it doesn't resolve through whoami. ~50 lines, one PR.

This RFC covers the four picks where there IS a protocol/UX question:

- **#2** new high-level verb: `termlink agent contact <name> <message>`
- **#3** topic self-documentation via `channel describe`
- **#5** `/agent-handoff` claude-code skill (plugin-level)
- **#6** `fw fleet doctor` extension reporting legacy-primitive usage

## 3. Initial design (subject to peer feedback)

### 3.1 The verb

```
termlink agent contact <target> [--message <m>] [--file <path>]
                                [--thread <id>]
                                [--reply-on <topic>]
                                [--ack-required]
                                [--json]
```

Internal sequence:
1. Discover target session via existing `discover` machinery (name, role, tag).
2. Resolve or create a per-pair DM topic — proposal: canonical name `dm:<self_sender_id>:<peer_sender_id>` sorted alphabetically (T-1319 pattern, already exists in the codebase).
3. `whoami` to resolve self identity. Stamp `metadata.from=<self_label>` and let hub verify.
4. Post envelope to the DM topic with `msg_type=request`, `metadata.thread=<thread>` (default: auto-generated), `metadata.requires_ack=<bool>`.
5. Return offset on success. If `--ack-required`, optionally subscribe and wait up to N seconds for an `m.receipt` envelope on the same topic (configurable; default no-wait).
6. If discovery fails (target offline, hub unreachable), behavior depends on Q3 below.

### 3.2 Topic self-documentation

```
termlink channel describe agent-chat-arc \
  "Cross-host agent handoffs. msg_type required. \
   identity (sender_id) authoritative; metadata.from must match whoami. \
   Use metadata.thread=<task-id> to thread by task. \
   Reply via metadata.in_reply_to=<offset>. \
   inbox.push is deprecated — use this topic or `termlink agent contact`."
```

Same trick for any per-pair `dm:*:*` topic auto-created by the verb. Self-describing topic = zero-CLAUDE.md-cost canon.

### 3.3 The skill (plugin-level, not in CLAUDE.md)

`/agent-handoff <target> <task-id>` runs:
1. Verify task exists locally
2. `termlink whoami` (cache canonical identity)
3. `termlink agent contact <target> --thread <task-id> --message "<task summary>"`
4. Verify offset returned, log to task
5. Update task with `posted=offset`, status hint `awaiting-reply`

CLAUDE.md gain: ONE line — "for cross-host handoffs use `/agent-handoff`."

### 3.4 The doctor extension

`fw fleet doctor --legacy-usage` walks last-N-day events, counts `inbox.push` / `file.send` / `event.broadcast` per session per host. T-1166 cut readiness signal: when count flatlines at zero for 7 days, cut is safe.

## 4. Open questions for peers

### Q1 — Topic provisioning: auto-create or explicit?

**Option A (auto-create):** verb creates `dm:<a>:<b>` on first use, retention=forever. Receiver gets a topic they didn't ask for, but it's targeted (only contains messages for them).

**Option B (explicit):** receiver must pre-subscribe to `agent-chat-arc` (or peer announces topic name beforehand). More setup, but receiver controls their topic surface.

**Tradeoff:** A is lower friction for senders, higher topic count for receivers (one DM topic per peer). B inverts that.

### Q2 — Ack semantics: per-message or per-thread?

**Option A (per-message):** every contact returns an `m.receipt` for that specific offset. High traffic.

**Option B (per-thread):** receipt acknowledges receipt-of-thread (latest read offset). Already exists in chat arc as `channel ack --up-to`.

**Option C (none by default, opt-in):** sender adds `--ack-required`; receiver replies only when the flag is set. Default: fire-and-forget.

### Q3 — Receiver offline / unreachable: fail-fast, queue, or both?

**Option A (fail-fast):** verb returns non-zero immediately if discover finds no live target. Caller decides.

**Option B (queue):** post to topic anyway (chat arc is offset-durable, retention=forever). Receiver reads on next session. Caller doesn't know when delivery happens.

**Option C (caller chooses):** flag `--require-online` for fail-fast, default = queue.

**Tradeoff:** A surfaces problems immediately, B is more resilient but obscures delivery state, C punts decision to invoker.

### Q4 — Identity binding strictness

**Option A (strict reject):** hub rejects `channel post` if `metadata.from` doesn't resolve through `termlink whoami` of the connection's identity. Backward-incompatible — every existing post without `from=` continues working, but new posts with mismatched `from=` get -32xxx.

**Option B (warn-and-strip):** hub strips mismatched `metadata.from`, logs the attempt. Receiver only ever sees authoritative `sender_id`. Lenient.

**Option C (warn + accept):** hub passes through but stamps `metadata.from_verified=false`. Receiver decides.

### Q5 — Thread retention and discoverability

DM topics are per-pair. Over time, do they grow unbounded? Options:

**Option A (forever):** retention=forever (matches `agent-chat-arc`). Long-running pairs accumulate offsets indefinitely but disk is cheap.

**Option B (TTL):** retention=`time:30d` (or similar). Old threads age out.

**Option C (count cap):** retention=`messages:10000`. Prevents pathological growth.

Operational question: do receivers want their DM topics to persist across years (option A) or self-prune (B/C)?

## 5. Per-receiver perspective hints

Each peer has different priorities — please answer through your operational lens:

- **.122 (ring20-management, ops infra):** what does an operations agent need to act on inbound contact safely? Sovereignty-respecting? Task-citing required?
- **.141 (laptop, presence-volatile):** how should presence-aware contact behave? When you're offline, queue or reject? When you come back online, do you want auto-replay?
- **.143 (ring20-dashboard, when reachable):** if you join the topic, what's the minimum metadata you'd need to triage an inbound `request`?

## 6. Reply protocol

Reply on `agent-chat-arc` (same topic, `--hub 192.168.10.107:9100` if you're on a different hub) with:

- `metadata.thread=T-1425`
- `metadata.in_reply_to=<RFC-envelope-offset>`
- `msg_type=reply`
- Payload: structured JSON with keys `q1`, `q2`, `q3`, `q4`, `q5` each carrying `{choice: "A|B|C", rationale: "<why>"}`. Free-form `notes` field welcome. Per-receiver-perspective answers in `perspective` field.

Synthesizer (.107) walks the topic on T-1425 thread after 48h, builds Decision matrix, posts synthesis.

## 7. Dialogue Log

### 2026-04-30 — RFC drafted (.107, sender_id d1993c2c3ec44c94)

Initial design above. Awaits peer feedback before any implementation.

### 2026-04-30 — Fast-forward solo synthesis (.107, operator-requested 0h after post)

Operator asked for fast-forward synthesis 0h into the 48h soak window. Topic walk at offset 7 → 0 peer replies. Formal runbook outcome: DEFER. Solo answers per question recorded below; full rationale in `T-1425.md` task file `## Decisions` section. These are .107-perspective only; peer replies arriving within 14d amend.

- **Q1 — DM topic provisioning:** A (auto-create, retention=forever) — receiver-friction binding constraint, T-1319 precedent
- **Q2 — Ack semantics:** C (none default, opt-in via `--ack-required`) — most contacts fire-and-forget; per-thread ack via existing `channel ack --up-to`
- **Q3 — Receiver offline:** C (caller chooses, default queue) — chat arc is offset-durable; `--require-online` flag for fail-fast
- **Q4 — Identity binding:** A (strict reject) — identity is security primitive; backward-compat for posts without `metadata.from`
- **Q5 — DM topic retention:** A (forever) — matches `agent-chat-arc`; PL-100 incident showed audit-trail value

**A-1 status:** untested (no peer replies). Build tasks ship under solo design with explicit amendment path for late peer feedback.

<!-- Peer replies appear below as they come in. Format per reply:
### YYYY-MM-DD — <peer-host> (<sender_id>) at offset N
- q1: A/B/C — rationale
- q2: ...
- ...
-->

## 8. References

- T-1166: legacy primitive retirement (umbrella)
- T-1422 / PL-100: glibc-mismatch incident — first cross-host coordination via chat arc
- T-1424: musl deploy + three-host chat-arc verification
- PL-098: cross-host chat-arc smoke procedure
- T-1319: existing `dm:<a>:<b>` topic pattern (precedent for §3.1 step 2)
- T-1318: persistent cursor pattern (relevant to Q3 queue behavior)
