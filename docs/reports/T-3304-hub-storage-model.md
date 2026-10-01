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

## Dialogue Log

- 2026-10-01 — Operator, during SQ-11: "do we want the hub to have a source of
  truth that can be queryable? … ask three of our … non-Anthropic agents … there's
  real value in there." SQ-11 left open pending this.
