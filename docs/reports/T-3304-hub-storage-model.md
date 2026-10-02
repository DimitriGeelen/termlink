# T-3304 — Should the TermLink hub be a queryable source of truth?

Status: inception, consulting three non-Anthropic agents (operator request, 2026-10-01).
Raised while discussing SQ-11 (T-2573, subscribe deadline drops collected messages).

## The operator's question (paraphrased faithfully)

Does the hub store messages? Is there retention, is there pruning — or do agents
keep their own old messages and just subscribe to topics? Agents can choose that
for themselves. But do we want the hub to be a **source of truth that can be
queried**? If yes: how do we facilitate it, do we need pruning mechanisms, how do
we chunk, how do subscriptions work, how do topics work, how does an agent select
or discover topics, and how do selections work (time windows, other cuts) and at
what cost?

## Brief sent to each agent

`T-3304-consult/brief.md` (identical text to all three).

## Answers

- `T-3304-consult/codex.md` — OpenAI Codex (ChatGPT subscription)
- `T-3304-consult/glm.md` — Z.AI GLM-5.3 via opencode (Coding Plan subscription)
- `T-3304-consult/qwen.md` — local qwen3:14b via Ollama

## Synthesis (2026-10-01, all three answers in)

Run notes: Codex ran read-only inside the repo. GLM-5.3 was re-run from an empty
folder after its first run grepped a scratch log holding Codex's answer (discarded
as contaminated). qwen3:14b answered from the brief alone; its file was cleaned of
terminal redraw codes only.

### Where they agree

| Question | Codex | GLM-5.3 | qwen3 | Convergence |
|---|---|---|---|---|
| IW-1 stance | Authoritative for **coordination state** within bounded history; archives elsewhere; charter non-goal #2 "too categorical" | Authoritative for **live coordination state** only, bounded memory (days); durable state in purpose-built stores; "75 forever topics = the fleet already treats it as a DB" | Pure transport; agents keep history | 2 of 3: **neither archive nor pure transport — authoritative for coordination, bounded window** |
| IW-2 retention | Profiles: telemetry 24-72h, coordination ~14d, state latest-per-key, archive opt-in with owner; **automatic** incremental sweep | Default bounded (14d or 10k); "forever" needs a reason; keep cron but add **hard cap at append** | Automatic time-based (then argues for manual) | **3 of 3: flip the default away from forever; something must bound growth without the cron** |
| IW-3 query | First: explicit read contract — page says why it ended; **deadline returns resumable progress** | Scan budget + resume hint; time-range; conversation_id index | Time-range first; metadata index | **3 of 3 keep cursor model; 3 of 3 never cross-topic joins / query language; time-range + conversation_id index next** |
| IW-4 topics | Required metadata (purpose, owner, schema, retention, status); catalog *is* the registry | Naming convention + creation metadata + registry topic; describe shows last-append **and last-read** | Registry with tags/descriptions | **3 of 3: required topic metadata + tag-based discovery** |
| "Mistake nobody asked about" | Append log can't claim atomically | Reads have no telemetry/budget — nobody could see the probe topic is unread | No federation | (see verification) |

### Verified against the code

- **Already exists, agents didn't know:** long-poll (`timeout_ms`, `subscribe_blocking`);
  retention-gap detection (`Bus::gap_before`, T-2463); atomic claims with leases
  (`channel.claim`, `CLAIM_CONFLICT` -32015) — Codex's #6 is answered by T-2019, my
  brief omitted it.
- **Real gap confirmed:** `Bus::post` (`crates/termlink-bus/src/lib.rs:184`) never
  applies retention. If the sweep cron does not run, nothing caps a topic (GLM's point).
- **qwen's federation point** contradicts charter non-goal #1 (deliberate, G-060) and
  assumes cross-hub coordination is impossible; client-driven cross-posting exists. Low weight.

### Bearing on SQ-11 (IW-5)

Codex and GLM independently ask for exactly SQ-11 option c2: a deadline must hand back
what it found and a resume point that always advances (Codex: "continuation cursor based
on the last record examined"). SQ-11 is a slice of IW-3, not blocked by IW-1.

### Decision queue (operator, one at a time)

1. IW-1 stance: transport / authoritative-for-coordination-bounded / queryable archive (+ charter non-goal #2 wording).
2. IW-2 retention defaults and an append-time cap.
3. IW-3 read contract (includes SQ-11), time-range, conversation_id index.
4. IW-4 topic metadata, discovery, read telemetry.

## IW-4 consultation (2026-10-01)

Brief: `T-3304-consult/iw4-brief.md` (facts, research summary, options A-D). Research:
`T-3304-consult/research-kafka-pulsar.md`, `research-nats-rabbitmq-redis-mqtt-matrix.md`
(27 source links). Five consultants, each from an empty folder (no access to the others):
Codex (`iw4-codex.md`), GLM-5.3 (`iw4-glm.md`), qwen3:14b (`iw4-qwen.md`), and — added at the
operator's prompt — gpt-oss:20b (`iw4-gptoss.md`) and gemma4 (`iw4-gemma4.md`), local via Ollama.

| Question | Codex | GLM-5.3 | qwen3 | gpt-oss | gemma4 |
|---|---|---|---|---|---|
| Option | C | C, sequenced (tracking first) | C | C | C |
| Owner | required, accountable identity | required, validated identity | optional, encouraged | warn, auto-infer | from identity |
| Purpose | required on explicit create | **never blocks**; marked incomplete | optional | warn + nightly audit | structured type, not free text |
| Read tracking | sound; "last fetch" not "last read"; empty vs data fetch | sound; fetch != read | sound | sound | sound |
| Dead-topic N | 30 d, flag | 30 d (>= 2x retention), flag, per-topic override | 30 d, flag | 30 d, flag | 14 d flag, auto-delete 90 d |
| Key catch | rule misses **written-but-unread** topics (ring20 probe) | ensure-topic must not count as activity; track **writers** | — | flag "no reads after last write" | — |
| Wildcards | later | later (would mask typos) | later | later | later |
| Missed | retention-gap signal (= IW-3 step 1) | writer governance | access control | access control | audit trail |

Convergence: 5/5 C; 5/5 no server-side cursors; 5/5 flag rather than delete (gemma4 alone
adds deletion at 90 d); 5/5 wildcards later. 4/5 against a hard block on purpose.
Two catches change the design: Codex's written-but-unread class (the ring20 probe would never
trip "no read AND no write") and GLM's "an ensure/metadata touch is not activity".

## Corrections (2026-10-01, IW-2 walk-through)

- **Codex answer on file is the second run.** The first-batch Codex run (discarded with the
  rest of that batch) said "profiles: telemetry 24-72h, coordination ~14d"; the
  synthesis table above quotes that version. The answer on file says **7-day default plus
  a byte ceiling, incremental background sweeps**. The direction is unchanged.
- **The hub already has a background retention sweeper** (T-2427,
  `crates/termlink-hub/src/retention_sweeper.rs`, env-gated
  `TERMLINK_SWEEP_INTERVAL_SECS`). It is on at .107 (3600 s; 9 runs, 1,561 records
  pruned since 10:52 on 2026-10-01) and off at .122 and .121 (interval 0, via
  `fleet governor-status`). An earlier claim in the IW-2 framing — "only one sweep cron,
  35 bounded topics never swept" — was false.
- **Retention-gap detection is NOT live.** `Bus::gap_before` (T-2463) exists in the bus
  library, but nothing in the hub calls it (only its own tests) — a reader whose cursor was
  swept is not told and silently resumes at the oldest surviving record. The synthesis line
  "already exists: retention-gap detection" is wrong at the hub level.
- **Measured (IW-3):** worst-case filtered read of the largest topic
  (`health:ring20-fedprobe`, 2,777 records, filter matching nothing) = 79 ms; the
  `records` table already has an index on `(topic, ts_unix_ms)`; `ts` is sender-supplied
  with hub-time fallback; CLI `--since/--until` filter client-side only.
- **Verified fact that stands:** `Bus::post` never applies retention, so a hub with the
  sweeper off has no bound at all.

## Dialogue Log

How the question arose, during the SQ-11 (T-2573) walk-through, 2026-10-01:

1. **Operator:** is the 20 s limit about *downloading* messages? Downloads and binary
   blobs can be big and should be asynchronous, not under a time limit.
   **Answer:** no — it caps the hub's server-side walk of its own log. Messages are
   small envelopes returned in bounded pages; binary payloads use the separate chunked
   artifact/file path (`MAX_PAYLOAD_SIZE` 16 MB per frame) and never meet this limit.
2. **Operator:** so the agent asks, the hub searches, and if it takes too long the hub
   just tells it to go away?
   **Answer:** yes, except it does say goodbye (error -32020 with a resume position) —
   the bug is that it discards what it found and the resume position skips past it.
3. **Operator:** why does the agent ask "from position 100 onwards"?
   **Answer:** a topic is an append-only list with numbered offsets; the agent keeps
   its cursor (how far it got) and asks only for what is new.
4. **Operator:** if the agent already has 0-99 and asks from 100, that needs no search.
   **Answer (verified, `crates/termlink-bus/src/meta.rs:276`):** correct — the hub jumps
   to 100 through an index. Only a *filtered* request (by conversation or reply-to) opens
   records one by one; that is the only slow path. This corrected an earlier example
   (an unfiltered read of the 2,777-record probe topic is instant, not a trigger).
5. **Operator:** "you explained it was about searching for a topic, not giving me new
   messages — maybe you're confusing things."
   **Answer:** agreed, the confusion was the agent's wording: there is one operation,
   "give me new messages in topic X from position N", optionally filtered; "search" only
   fits the filtered form.
6. **Operator — the principle question:** does the hub store messages; retention;
   pruning; do agents keep their own history; "do we want the hub to have a source of
   truth that can be queryable?"; how would that be facilitated (pruning, chunking,
   subscriptions, topics, how agents select and ask for topics, time windows, cost
   cuts). "Ask three of our … non-Anthropic agents … there's real value in there."
   → this inception; SQ-11 left open.
7. Consultation run (Codex, GLM-5.3, qwen3:14b); synthesis above; IW-1 presented with
   options A/B/C, value-driver scoring and steelman/strawman.
8. **Operator ruling on IW-1:** "record B, with all this discussion we had" →
   option B, authoritative for coordination within a declared retention window
   (recorded in T-3304 § Decisions). Charter non-goal #2 rewording to come back for
   approval. IW-2..IW-4 and SQ-11 still open.
9. IW-2 first framing (options A-D, recommending a hub timer) — **operator:** "it's
   clearly in favour of C, but also driven by our earlier conversation; look at the
   external reviewers' advice". Re-check found the timer already exists and is on at
   .107 (see Corrections), and the reviewers split on enforcement (Codex: background
   sweep; GLM: cron + cap on post; qwen: inconsistent) while all three agree on a bounded
   default. Reframed as A keep / B default + sweeper everywhere / C B + ceiling on post.
10. **Operator ruling on IW-2:** "Alright, C then" → option C; 14-day default assumed
    (agent recommendation; operator did not name a window).
11. IW-3 presented (A honest reads / B + time-range / C + indexes / D nothing), after
    measuring the slow path (79 ms) and finding the hub never surfaces retention gaps.
12. **Operator ruling on IW-3:** "take your recommendation … vote for B" → option B,
    ordered gap signal → SQ-11 c2 → page end reasons → time-range. SQ-11 recorded as c2
    on T-2573 as part of that recommendation.
13. IW-4 presented (A-D, recommending C). **Operator:** what exactly is the problem, and how
    do Kafka and other open-source systems solve it? → two research agents (Kafka/Pulsar/
    catalogs; NATS/RabbitMQ/Redis/MQTT/Matrix/Pub/Sub), findings saved with sources.
14. **Operator:** consult the three harnesses; then noted other local models → gpt-oss:20b
    and gemma4 added (the first `ollama list` had been truncated to 12 lines, hiding them).
15. IW-4 re-presented as B / C / C-prime after the consultation. **Operator ruling:**
    "let's go with C as suggested and recommended" → C-prime (recorded in T-3304 § Decisions).
    All IW-1..IW-5 now answered; inception go/no-go next.
16. T-3310 D1 (how the hub treats a bare "forever" create, given every client sends forever
    explicitly by default and old binaries cannot send an owner) presented A refuse / B downgrade
    / C accept-and-label / D C-now-A-later, recommending D. **Operator:** comfortable with D, but
    the strawman ("the switch stays off forever") is valid: what keeps the risk remediated?
    **Answer:** four layers: evidence-driven auto-flip after 14 quiet days, a 2026-11-15 backstop
    canary that files a task, loud daily reporting of any opt-out, a fixture test plus mutant
    pinning the flip; revisit_at as the weakest, human layer.
17. **Operator ruling on T-3310 D1:** "proceed as suggested" -> D+ (D with layers 1-4, 14 days,
    2026-11-15). Recorded in T-3310 § Decisions. D2 next.
18. T-3310 D2 (new-client default retention, especially mail) presented A uniform 14 d / B mail by
    count + rest 14 d / C mail forever auto-owned / D keep until read, recommending B.
    **Operator ruling:** "I agree with the recommendation" -> B. Recorded in T-3310 § Decisions.
    D3 next.

